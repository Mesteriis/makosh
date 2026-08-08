//! Exact Scheduler schedule-control consumption and result publication.

use std::time::Duration;

use async_nats::jetstream::consumer::{AckPolicy, PullConsumer};
use futures_util::StreamExt;
use makosh_events_protocol::{
    RuntimeNatsJwtCredentialV1,
    delivery::{
        ExactOutboxPublisherPortV1, OutboxPublishReceiptV1, OutboxRecordV1, OutboxRelayErrorV1,
    },
    v1::durable_envelope_v1::Semantics,
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::{
    v1::SchedulerRuntimeScheduleControlBindingV1,
    validation::scheduler::validate_scheduler_runtime_schedule_control_binding,
};
use makosh_scheduler_protocol::{
    SchedulerScheduleControlDeliveryErrorV1, SchedulerScheduleControlDeliveryPortV1,
    SchedulerScheduleControlDeliveryV1,
};

use super::receipt::connect_context;

const IDLE_WAIT: Duration = Duration::from_secs(1);
const PUBLISH_TIMEOUT: Duration = Duration::from_secs(2);

/// One exact command consumer and its paired result publisher.
pub struct SchedulerJetStreamScheduleControlPortV1 {
    context: async_nats::jetstream::Context,
    consumer: PullConsumer,
    result_subject: String,
}

impl SchedulerJetStreamScheduleControlPortV1 {
    pub async fn connect(
        endpoint: &str,
        credential: RuntimeNatsJwtCredentialV1,
        binding: &SchedulerRuntimeScheduleControlBindingV1,
    ) -> Result<Self, SchedulerJetStreamScheduleControlPortErrorV1> {
        validate_scheduler_runtime_schedule_control_binding(binding)
            .map_err(|_| SchedulerJetStreamScheduleControlPortErrorV1::InvalidBinding)?;
        let context = connect_context(endpoint, credential)
            .await
            .map_err(map_connect_error)?;
        let consumer = open_consumer(&context, binding).await?;
        Ok(Self {
            context,
            consumer,
            result_subject: binding.result_subject.clone(),
        })
    }

    async fn publish_result(
        &self,
        record: &OutboxRecordV1,
    ) -> Result<OutboxPublishReceiptV1, OutboxRelayErrorV1> {
        let envelope = decode_envelope_v1(record.exact_bytes())
            .map_err(|_| OutboxRelayErrorV1::PublisherUnavailable)?;
        let contract = envelope
            .contract
            .as_ref()
            .ok_or(OutboxRelayErrorV1::PublisherUnavailable)?;
        let subject = format!(
            "makosh.result.v1.{}.{}.v{}",
            contract.owner, contract.name, contract.major
        );
        (matches!(envelope.semantics, Some(Semantics::Result(_)))
            && subject == self.result_subject)
            .then_some(())
            .ok_or(OutboxRelayErrorV1::PublisherUnavailable)?;
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("Nats-Msg-Id", canonical_message_id(record.message_id()));
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
}

impl SchedulerScheduleControlDeliveryPortV1 for SchedulerJetStreamScheduleControlPortV1 {
    type Delivery = SchedulerJetStreamScheduleControlDeliveryV1;

    async fn receive(&mut self) -> Result<Self::Delivery, SchedulerScheduleControlDeliveryErrorV1> {
        loop {
            let mut messages = self
                .consumer
                .fetch()
                .max_messages(1)
                .expires(IDLE_WAIT)
                .messages()
                .await
                .map_err(|_| SchedulerScheduleControlDeliveryErrorV1::Unavailable)?;
            match messages.next().await {
                Some(Ok(message)) => {
                    return Ok(SchedulerJetStreamScheduleControlDeliveryV1 { message });
                }
                Some(Err(_)) => {
                    return Err(SchedulerScheduleControlDeliveryErrorV1::Unavailable);
                }
                None => {}
            }
        }
    }
}

impl ExactOutboxPublisherPortV1 for SchedulerJetStreamScheduleControlPortV1 {
    fn publish_exact(
        &self,
        record: &OutboxRecordV1,
    ) -> impl std::future::Future<Output = Result<OutboxPublishReceiptV1, OutboxRelayErrorV1>> + Send
    {
        self.publish_result(record)
    }
}

pub struct SchedulerJetStreamScheduleControlDeliveryV1 {
    message: async_nats::jetstream::Message,
}

impl SchedulerScheduleControlDeliveryV1 for SchedulerJetStreamScheduleControlDeliveryV1 {
    fn exact_bytes(&self) -> &[u8] {
        self.message.payload.as_ref()
    }

    async fn acknowledge(self) -> Result<(), SchedulerScheduleControlDeliveryErrorV1> {
        self.message
            .ack()
            .await
            .map_err(|_| SchedulerScheduleControlDeliveryErrorV1::Unavailable)
    }
}

async fn open_consumer(
    context: &async_nats::jetstream::Context,
    binding: &SchedulerRuntimeScheduleControlBindingV1,
) -> Result<PullConsumer, SchedulerJetStreamScheduleControlPortErrorV1> {
    let stream = context
        .get_stream(&binding.stream_name)
        .await
        .map_err(|_| SchedulerJetStreamScheduleControlPortErrorV1::Unavailable)?;
    let consumer = stream
        .get_consumer(&binding.durable_name)
        .await
        .map_err(|_| SchedulerJetStreamScheduleControlPortErrorV1::Unavailable)?;
    let actual = &consumer.cached_info().config;
    (actual.durable_name.as_deref() == Some(binding.durable_name.as_str())
        && actual.filter_subject == binding.filter_subject
        && actual.ack_policy == AckPolicy::Explicit
        && actual.ack_wait == Duration::from_millis(u64::from(binding.ack_wait_millis))
        && actual.max_deliver == i64::from(binding.max_deliver)
        && actual.max_ack_pending == i64::from(binding.max_ack_pending))
    .then_some(consumer)
    .ok_or(SchedulerJetStreamScheduleControlPortErrorV1::BindingMismatch)
}

fn canonical_message_id(value: &[u8; 16]) -> String {
    let mut canonical = String::with_capacity(32);
    for byte in value {
        canonical.push_str(&format!("{byte:02x}"));
    }
    canonical
}

fn map_connect_error(
    error: super::receipt::SchedulerJetStreamReceiptPortErrorV1,
) -> SchedulerJetStreamScheduleControlPortErrorV1 {
    match error {
        super::receipt::SchedulerJetStreamReceiptPortErrorV1::ExpiredCredential => {
            SchedulerJetStreamScheduleControlPortErrorV1::ExpiredCredential
        }
        super::receipt::SchedulerJetStreamReceiptPortErrorV1::InvalidCredential => {
            SchedulerJetStreamScheduleControlPortErrorV1::InvalidCredential
        }
        super::receipt::SchedulerJetStreamReceiptPortErrorV1::BindingMismatch
        | super::receipt::SchedulerJetStreamReceiptPortErrorV1::InvalidBinding
        | super::receipt::SchedulerJetStreamReceiptPortErrorV1::Unavailable => {
            SchedulerJetStreamScheduleControlPortErrorV1::Unavailable
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerJetStreamScheduleControlPortErrorV1 {
    BindingMismatch,
    ExpiredCredential,
    InvalidBinding,
    InvalidCredential,
    Unavailable,
}
