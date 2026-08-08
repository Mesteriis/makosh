//! Exact Scheduler dispatch publication constrained by Kernel-derived subjects.

use std::{collections::BTreeSet, time::Duration};

use makosh_events_protocol::{
    RuntimeNatsJwtCredentialV1,
    delivery::{
        ExactOutboxPublisherPortV1, OutboxPublishReceiptV1, OutboxRecordV1, OutboxRelayErrorV1,
        OwnerOutboxStorePortV1, relay_once,
    },
    v1::{DurableEnvelopeV1, durable_envelope_v1::Semantics},
    validation::envelope::{decode_envelope_v1, validate_envelope_v1},
};
use makosh_runtime_protocol::{
    v1::SchedulerRuntimeDispatchPublisherBindingV1,
    validation::scheduler::validate_scheduler_runtime_dispatch_publisher_binding,
};

use super::receipt::connect_context;

const PUBLISH_TIMEOUT: Duration = Duration::from_secs(2);

/// One Scheduler-owned publisher fenced to Kernel-declared command subjects.
pub struct SchedulerJetStreamDispatchPortV1 {
    context: async_nats::jetstream::Context,
    subjects: BTreeSet<String>,
}

impl SchedulerJetStreamDispatchPortV1 {
    pub async fn connect(
        endpoint: &str,
        credential: RuntimeNatsJwtCredentialV1,
        bindings: &[SchedulerRuntimeDispatchPublisherBindingV1],
    ) -> Result<Self, SchedulerJetStreamDispatchPortErrorV1> {
        let subjects = Self::validate_bindings(bindings)?;
        let context = connect_context(endpoint, credential)
            .await
            .map_err(|_| SchedulerJetStreamDispatchPortErrorV1::Unavailable)?;
        Ok(Self { context, subjects })
    }

    /// Validates the exact Kernel-derived command subjects before any NATS connection.
    pub fn validate_bindings(
        bindings: &[SchedulerRuntimeDispatchPublisherBindingV1],
    ) -> Result<BTreeSet<String>, SchedulerJetStreamDispatchPortErrorV1> {
        validated_subjects(bindings)
    }

    async fn publish_exact(
        &self,
        record: &OutboxRecordV1,
    ) -> Result<OutboxPublishReceiptV1, OutboxRelayErrorV1> {
        let envelope = decode_envelope_v1(record.exact_bytes())
            .map_err(|_| OutboxRelayErrorV1::PublisherUnavailable)?;
        validate_envelope_v1(&envelope).map_err(|_| OutboxRelayErrorV1::PublisherUnavailable)?;
        let subject = command_subject(&envelope).ok_or(OutboxRelayErrorV1::PublisherUnavailable)?;
        self.subjects
            .contains(&subject)
            .then_some(())
            .ok_or(OutboxRelayErrorV1::PublisherUnavailable)?;
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("Nats-Msg-Id", canonical_message_id(&envelope.message_id));
        let acknowledgement = tokio::time::timeout(
            PUBLISH_TIMEOUT,
            self.context.publish_with_headers(
                subject,
                headers,
                record.exact_bytes().to_vec().into(),
            ),
        )
        .await
        .map_err(|_| OutboxRelayErrorV1::PublisherUnavailable)?
        .map_err(|_| OutboxRelayErrorV1::PublisherUnavailable)?
        .await
        .map_err(|_| OutboxRelayErrorV1::PublisherUnavailable)?;
        OutboxPublishReceiptV1::new(
            acknowledgement.stream,
            acknowledgement.sequence,
            acknowledgement.duplicate,
        )
    }

    /// Relays at most one already-persisted dispatch without exposing a generic NATS publisher.
    pub async fn relay_once<S>(&self, store: &mut S) -> Result<bool, SchedulerDispatchRelayErrorV1>
    where
        S: OwnerOutboxStorePortV1,
    {
        relay_once(store, self)
            .await
            .map(|outcome| {
                matches!(
                    outcome,
                    makosh_events_protocol::delivery::OutboxRelayOutcomeV1::Published { .. }
                )
            })
            .map_err(map_relay_error)
    }
}

impl ExactOutboxPublisherPortV1 for SchedulerJetStreamDispatchPortV1 {
    fn publish_exact(
        &self,
        record: &OutboxRecordV1,
    ) -> impl std::future::Future<Output = Result<OutboxPublishReceiptV1, OutboxRelayErrorV1>> + Send
    {
        self.publish_exact(record)
    }
}

fn validated_subjects(
    bindings: &[SchedulerRuntimeDispatchPublisherBindingV1],
) -> Result<BTreeSet<String>, SchedulerJetStreamDispatchPortErrorV1> {
    (!bindings.is_empty())
        .then_some(())
        .ok_or(SchedulerJetStreamDispatchPortErrorV1::InvalidBinding)?;
    let subjects = bindings
        .iter()
        .map(|binding| {
            validate_scheduler_runtime_dispatch_publisher_binding(binding)
                .map_err(|_| SchedulerJetStreamDispatchPortErrorV1::InvalidBinding)
                .map(|_| binding.subject.clone())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    (subjects.len() == bindings.len())
        .then_some(subjects)
        .ok_or(SchedulerJetStreamDispatchPortErrorV1::InvalidBinding)
}

fn command_subject(envelope: &DurableEnvelopeV1) -> Option<String> {
    let contract = envelope.contract.as_ref()?;
    matches!(envelope.semantics, Some(Semantics::Command(_))).then(|| {
        format!(
            "makosh.command.v1.{}.{}.v{}",
            contract.owner, contract.name, contract.major
        )
    })
}

fn canonical_message_id(value: &[u8]) -> String {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerJetStreamDispatchPortErrorV1 {
    InvalidBinding,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerDispatchRelayErrorV1 {
    InvalidEntry,
    InvalidReceipt,
    Persistence,
    PublisherUnavailable,
}

const fn map_relay_error(error: OutboxRelayErrorV1) -> SchedulerDispatchRelayErrorV1 {
    match error {
        OutboxRelayErrorV1::InvalidEntry => SchedulerDispatchRelayErrorV1::InvalidEntry,
        OutboxRelayErrorV1::InvalidReceipt => SchedulerDispatchRelayErrorV1::InvalidReceipt,
        OutboxRelayErrorV1::Persistence => SchedulerDispatchRelayErrorV1::Persistence,
        OutboxRelayErrorV1::PublisherUnavailable => {
            SchedulerDispatchRelayErrorV1::PublisherUnavailable
        }
    }
}
