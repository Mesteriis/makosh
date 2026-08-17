//! Runtime publish connection and exact-byte outbox relay.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;

use async_nats::connection::State;
use async_nats::jetstream::consumer::PullConsumer;
use makosh_events_protocol::delivery::{
    ExactOutboxPublisherPortV1, OutboxPublishReceiptV1, OutboxRecordV1, OutboxRelayErrorV1,
};
use makosh_events_protocol::validation::envelope::{decode_envelope_v1, validate_envelope_v1};

use crate::subjects::DurableSubjectV1;

use super::event_hub::{canonical_consumer_config, canonical_consumer_matches};
use super::{RuntimeNatsIdentity, RuntimePublishPermitV1, RuntimeSubscribePermitV1};

const PUBLISH_TIMEOUT: Duration = Duration::from_secs(2);

/// One runtime identity, restricted by server ACL to exact catalog subjects.
pub struct RuntimeJetStreamConnection {
    context: async_nats::jetstream::Context,
    identity: RuntimeNatsIdentity,
    pull_consumers: Mutex<BTreeMap<String, PullConsumer>>,
}

impl RuntimeJetStreamConnection {
    pub(super) const fn new(
        context: async_nats::jetstream::Context,
        identity: RuntimeNatsIdentity,
    ) -> Self {
        Self {
            context,
            identity,
            pull_consumers: Mutex::new(BTreeMap::new()),
        }
    }

    #[must_use]
    pub fn identity(&self) -> &RuntimeNatsIdentity {
        &self.identity
    }

    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.context.client().connection_state() == State::Connected
    }

    /// Publishes original outbox bytes; no decode/re-encode occurs after validation.
    pub async fn publish_exact(
        &self,
        permit: &RuntimePublishPermitV1,
        bytes: &[u8],
    ) -> Result<PublishReceipt, String> {
        let envelope =
            decode_envelope_v1(bytes).map_err(|_| "durable envelope is invalid".to_owned())?;
        validate_envelope_v1(&envelope).map_err(|_| "durable envelope is invalid".to_owned())?;
        let subject = DurableSubjectV1::from_envelope(&envelope)
            .map_err(|_| "durable envelope subject is invalid".to_owned())?;
        permit
            .permits(&self.identity, &subject.as_str())
            .then_some(())
            .ok_or_else(|| {
                "durable envelope exceeds the current runtime publish permit".to_owned()
            })?;
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("Nats-Msg-Id", canonical_message_id(&envelope.message_id));
        let acknowledgement = tokio::time::timeout(
            PUBLISH_TIMEOUT,
            self.context
                .publish_with_headers(subject.as_str(), headers, bytes.to_vec().into()),
        )
        .await
        .map_err(|_| "JetStream publish timed out".to_owned())?
        .map_err(|_| "JetStream publish is unavailable".to_owned())?
        .await
        .map_err(|_| "JetStream publish acknowledgement is unavailable".to_owned())?;
        Ok(PublishReceipt {
            stream: acknowledgement.stream,
            sequence: acknowledgement.sequence,
            duplicate: acknowledgement.duplicate,
        })
    }

    /// Opens only the Event Hub-declared durable pull consumer bound to this runtime grant.
    pub async fn open_pull_consumer(
        &self,
        permit: &RuntimeSubscribePermitV1,
    ) -> Result<PullConsumer, String> {
        permit
            .permits(&self.identity)
            .then_some(())
            .ok_or_else(|| {
                "durable consumer exceeds the current runtime subscribe permit".to_owned()
            })?;
        let specification = permit.consumer();
        let expected = canonical_consumer_config(specification);
        let cache_key = pull_consumer_cache_key(specification);
        if let Some(consumer) = self
            .pull_consumers
            .lock()
            .map_err(|_| "JetStream durable consumer cache is unavailable".to_owned())?
            .get(&cache_key)
            .cloned()
        {
            return canonical_consumer_matches(&consumer.cached_info().config, &expected)
                .then_some(consumer)
                .ok_or_else(|| {
                    "JetStream durable consumer conflicts with the runtime subscribe permit"
                        .to_owned()
                });
        }
        let stream = self
            .context
            .get_stream(specification.stream_kind().stream_name())
            .await
            .map_err(|_| "JetStream consumer stream is unavailable".to_owned())?;
        let consumer: PullConsumer = stream
            .get_consumer(specification.durable_name())
            .await
            .map_err(|_| "JetStream durable consumer is unavailable".to_owned())?;
        if !canonical_consumer_matches(&consumer.cached_info().config, &expected) {
            return Err(
                "JetStream durable consumer conflicts with the runtime subscribe permit".to_owned(),
            );
        }
        self.pull_consumers
            .lock()
            .map_err(|_| "JetStream durable consumer cache is unavailable".to_owned())?
            .insert(cache_key, consumer.clone());
        Ok(consumer)
    }
}

fn pull_consumer_cache_key(specification: &crate::topology::ConsumerSpecV1) -> String {
    format!(
        "{}\0{}",
        specification.stream_kind().stream_name(),
        specification.durable_name()
    )
}

/// Binds an owner-local outbox relay to one exact runtime publish permit.
pub struct RuntimeOutboxPublisherV1<'a> {
    connection: &'a RuntimeJetStreamConnection,
    permit: &'a RuntimePublishPermitV1,
}

impl<'a> RuntimeOutboxPublisherV1<'a> {
    #[must_use]
    pub const fn new(
        connection: &'a RuntimeJetStreamConnection,
        permit: &'a RuntimePublishPermitV1,
    ) -> Self {
        Self { connection, permit }
    }
}

impl ExactOutboxPublisherPortV1 for RuntimeOutboxPublisherV1<'_> {
    #[allow(clippy::manual_async_fn)] // The relay port requires a Send future across all publishers.
    fn publish_exact(
        &self,
        record: &OutboxRecordV1,
    ) -> impl std::future::Future<Output = Result<OutboxPublishReceiptV1, OutboxRelayErrorV1>> + Send
    {
        async move {
            let receipt = self
                .connection
                .publish_exact(self.permit, record.exact_bytes())
                .await
                .map_err(|_| OutboxRelayErrorV1::PublisherUnavailable)?;
            OutboxPublishReceiptV1::new(receipt.stream, receipt.sequence, receipt.duplicate)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishReceipt {
    stream: String,
    sequence: u64,
    duplicate: bool,
}

impl PublishReceipt {
    #[must_use]
    pub fn stream(&self) -> &str {
        &self.stream
    }

    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    #[must_use]
    pub const fn duplicate(&self) -> bool {
        self.duplicate
    }
}

/// Renders the fixed NATS deduplication value for a 16-byte envelope ID.
#[must_use]
pub fn canonical_message_id(value: &[u8]) -> String {
    if value.len() != 16 {
        return String::new();
    }
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        value[0],
        value[1],
        value[2],
        value[3],
        value[4],
        value[5],
        value[6],
        value[7],
        value[8],
        value[9],
        value[10],
        value[11],
        value[12],
        value[13],
        value[14],
        value[15],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::{ConsumerBudgetV1, ConsumerSpecV1, StreamKindV1};

    #[test]
    fn pull_consumer_cache_key_is_stream_and_durable_specific() {
        let budget = ConsumerBudgetV1::new(32, 4, Duration::from_secs(5)).expect("budget");
        let command = ConsumerSpecV1::new(
            StreamKindV1::Command,
            "persons-command-v1",
            "makosh.command.v1.persons.command-v1",
            budget,
        )
        .expect("command consumer");
        let result = ConsumerSpecV1::new(
            StreamKindV1::Result,
            "persons-command-v1",
            "makosh.result.v1.persons.command-v1",
            budget,
        )
        .expect("result consumer");

        assert_eq!(
            pull_consumer_cache_key(&command),
            "MAKOSH_COMMAND_V1\0persons-command-v1"
        );
        assert_ne!(
            pull_consumer_cache_key(&command),
            pull_consumer_cache_key(&result)
        );
    }
}
