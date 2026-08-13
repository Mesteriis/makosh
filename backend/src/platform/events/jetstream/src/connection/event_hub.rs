//! Event Hub topology reconciliation connection.

use std::time::Duration;

use async_nats::jetstream::consumer::{
    AckPolicy, DeliverPolicy, IntoConsumerConfig, ReplayPolicy, pull,
};
use async_nats::jetstream::stream::{
    Config as StreamConfig, DiscardPolicy, RetentionPolicy, StorageType,
};

use crate::topology::{ConsumerSpecV1, EventHubTopologyPlanV1, StreamSpecV1};

const DUPLICATE_WINDOW: Duration = Duration::from_secs(120);
const MAX_ENVELOPE_BYTES: i32 = 262_144;
// Make the NATS durable-pull server default explicit so exact readback has no normalization gap.
const MAX_PULL_WAITING: i64 = 512;

/// Kernel Event Hub administration connection. It never transports owner payloads.
pub struct EventHubJetStreamConnection {
    context: async_nats::jetstream::Context,
}

impl EventHubJetStreamConnection {
    pub(super) const fn new(context: async_nats::jetstream::Context) -> Self {
        Self { context }
    }

    pub async fn reconcile(&self, topology: &EventHubTopologyPlanV1) -> Result<(), String> {
        for stream in topology.streams() {
            self.reconcile_stream(*stream).await?;
        }
        for consumer in topology.consumers() {
            self.reconcile_consumer(consumer).await?;
        }
        Ok(())
    }

    async fn reconcile_stream(&self, specification: StreamSpecV1) -> Result<(), String> {
        let expected = stream_config(specification);
        let stream = self
            .context
            .get_or_create_stream(expected.clone())
            .await
            .map_err(|_| "JetStream stream reconciliation failed".to_owned())?;
        stream_matches(&stream.cached_info().config, &expected)
            .then_some(())
            .ok_or_else(|| {
                "JetStream stream topology conflicts with the declared catalog".to_owned()
            })
    }

    async fn reconcile_consumer(&self, specification: &ConsumerSpecV1) -> Result<(), String> {
        let stream = self
            .context
            .get_stream(specification.stream_kind().stream_name())
            .await
            .map_err(|_| "JetStream consumer stream is unavailable".to_owned())?;
        let expected = canonical_consumer_config(specification);
        let consumer = stream
            .get_or_create_consumer(specification.durable_name(), expected.clone())
            .await
            .map_err(|_| "JetStream consumer reconciliation failed".to_owned())?;
        canonical_consumer_matches(&consumer.cached_info().config, &expected)
            .then_some(())
            .ok_or_else(|| {
                "JetStream consumer topology conflicts with the declared catalog".to_owned()
            })
    }
}

fn stream_config(specification: StreamSpecV1) -> StreamConfig {
    StreamConfig {
        name: specification.kind().stream_name().to_owned(),
        max_bytes: specification.budget().max_bytes(),
        max_age: specification.budget().max_age(),
        num_replicas: specification.budget().replicas(),
        subjects: vec![specification.kind().stream_subject().to_owned()],
        retention: RetentionPolicy::Limits,
        storage: StorageType::File,
        discard: DiscardPolicy::Old,
        max_consumers: 512,
        max_message_size: MAX_ENVELOPE_BYTES,
        duplicate_window: DUPLICATE_WINDOW,
        deny_delete: true,
        deny_purge: true,
        ..StreamConfig::default()
    }
}

pub(super) fn canonical_consumer_config(specification: &ConsumerSpecV1) -> pull::Config {
    let budget = specification.budget();
    let durable_name = specification.durable_name().to_owned();
    pull::Config {
        durable_name: Some(durable_name.clone()),
        name: Some(durable_name),
        deliver_policy: DeliverPolicy::All,
        ack_policy: AckPolicy::Explicit,
        ack_wait: budget.ack_wait(),
        max_deliver: budget.max_deliver(),
        filter_subject: specification.filter_subject().to_owned(),
        max_waiting: MAX_PULL_WAITING,
        max_ack_pending: budget.max_ack_pending(),
        max_batch: budget.max_ack_pending(),
        max_expires: budget.ack_wait(),
        inactive_threshold: Duration::ZERO,
        num_replicas: 1,
        replay_policy: ReplayPolicy::Instant,
        backoff: retry_backoff(budget.ack_wait(), budget.max_deliver()),
        ..pull::Config::default()
    }
}

fn retry_backoff(ack_wait: Duration, max_deliver: i64) -> Vec<Duration> {
    (0..max_deliver)
        .scan(ack_wait, |delay, _| {
            let current = *delay;
            *delay = delay.saturating_mul(2).min(Duration::from_secs(600));
            Some(current)
        })
        .collect()
}

fn stream_matches(actual: &StreamConfig, expected: &StreamConfig) -> bool {
    actual.name == expected.name
        && actual.subjects == expected.subjects
        && actual.max_bytes == expected.max_bytes
        && actual.max_age == expected.max_age
        && actual.num_replicas == expected.num_replicas
        && actual.retention == expected.retention
        && actual.storage == expected.storage
        && actual.discard == expected.discard
        && actual.max_consumers == expected.max_consumers
        && actual.max_message_size == expected.max_message_size
        && actual.duplicate_window == expected.duplicate_window
        && actual.deny_delete == expected.deny_delete
        && actual.deny_purge == expected.deny_purge
}

pub(super) fn canonical_consumer_matches(
    actual: &async_nats::jetstream::consumer::Config,
    expected: &pull::Config,
) -> bool {
    actual == &expected.into_consumer_config()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::{ConsumerBudgetV1, StreamKindV1};

    fn exact_consumer_configs() -> (async_nats::jetstream::consumer::Config, pull::Config) {
        let specification = ConsumerSpecV1::new(
            StreamKindV1::Command,
            "persons-command-v1",
            "makosh.command.v1.persons.command-v1",
            ConsumerBudgetV1::new(32, 4, Duration::from_secs(5)).expect("consumer budget"),
        )
        .expect("consumer specification");
        let expected = canonical_consumer_config(&specification);
        let actual = (&expected).into_consumer_config();
        (actual, expected)
    }

    #[test]
    fn canonical_consumer_match_rejects_every_delivery_topology_drift() {
        let (actual, expected) = exact_consumer_configs();
        assert!(canonical_consumer_matches(&actual, &expected));

        let mut drifted = actual.clone();
        drifted.deliver_subject = Some("makosh.deliver.drift".to_owned());
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.durable_name = Some("persons-command-drift".to_owned());
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.name = Some("drift".to_owned());
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.description = Some("drift".to_owned());
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.deliver_group = Some("drift".to_owned());
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.deliver_policy = DeliverPolicy::New;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.ack_policy = AckPolicy::All;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.ack_wait += Duration::from_millis(1);
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.max_deliver += 1;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.filter_subject.push_str(".drift");
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.replay_policy = ReplayPolicy::Original;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted
            .filter_subjects
            .push("makosh.command.v1.persons.second".to_owned());
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.rate_limit = 1;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.sample_frequency = 1;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.max_waiting += 1;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.max_ack_pending += 1;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.headers_only = true;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.flow_control = true;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.idle_heartbeat += Duration::from_millis(1);
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted
            .metadata
            .insert("drift".to_owned(), "true".to_owned());
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.priority_policy = async_nats::jetstream::consumer::PriorityPolicy::PinnedClient;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.priority_groups.push("drift".to_owned());
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted_json = serde_json::to_value(&actual).expect("consumer JSON");
        drifted_json["pause_until"] = serde_json::json!("2030-01-01T00:00:00Z");
        let drifted = serde_json::from_value(drifted_json).expect("paused consumer JSON");
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.max_batch += 1;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.max_expires += Duration::from_millis(1);
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.max_bytes += 1;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.inactive_threshold += Duration::from_millis(1);
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.num_replicas += 1;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual.clone();
        drifted.memory_storage = !drifted.memory_storage;
        assert!(!canonical_consumer_matches(&drifted, &expected));

        let mut drifted = actual;
        drifted.backoff[0] += Duration::from_millis(1);
        assert!(!canonical_consumer_matches(&drifted, &expected));
    }
}
