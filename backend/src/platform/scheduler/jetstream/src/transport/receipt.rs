//! Exact Scheduler receipt delivery over an Event Hub-authorized pull consumer.

use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_nats::jetstream::consumer::{AckPolicy, PullConsumer};
use futures_util::StreamExt;
use makosh_events_protocol::RuntimeNatsJwtCredentialV1;
use makosh_runtime_protocol::{
    v1::SchedulerRuntimeReceiptConsumerBindingV1,
    validation::scheduler::validate_scheduler_runtime_receipt_consumer_binding,
};
use makosh_scheduler_protocol::{
    SchedulerReceiptDeliveryErrorV1, SchedulerReceiptDeliveryPortV1, SchedulerReceiptDeliveryV1,
};
use nats_jwt::KeyPair;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const IDLE_RECEIPT_WAIT: Duration = Duration::from_secs(1);

/// One Scheduler-owned pull port whose broker consumer is fixed by Kernel configuration.
pub struct SchedulerJetStreamReceiptPortV1 {
    consumer: PullConsumer,
}

impl SchedulerJetStreamReceiptPortV1 {
    pub async fn connect(
        endpoint: &str,
        credential: RuntimeNatsJwtCredentialV1,
        binding: &SchedulerRuntimeReceiptConsumerBindingV1,
    ) -> Result<Self, SchedulerJetStreamReceiptPortErrorV1> {
        validate_binding(endpoint, binding)?;
        let context = connect_context(endpoint, credential).await?;
        let consumer = open_consumer(&context, binding).await?;
        Ok(Self { consumer })
    }
}

impl SchedulerReceiptDeliveryPortV1 for SchedulerJetStreamReceiptPortV1 {
    type Delivery = SchedulerJetStreamReceiptDeliveryV1;

    async fn receive(&mut self) -> Result<Self::Delivery, SchedulerReceiptDeliveryErrorV1> {
        loop {
            let mut messages = self
                .consumer
                .fetch()
                .max_messages(1)
                .expires(IDLE_RECEIPT_WAIT)
                .messages()
                .await
                .map_err(|_| SchedulerReceiptDeliveryErrorV1::Unavailable)?;
            match messages.next().await {
                Some(Ok(message)) => return Ok(SchedulerJetStreamReceiptDeliveryV1::new(message)),
                Some(Err(_)) => return Err(SchedulerReceiptDeliveryErrorV1::Unavailable),
                None => {}
            }
        }
    }
}

/// Exact broker message held until Scheduler persistence durably commits the receipt.
pub struct SchedulerJetStreamReceiptDeliveryV1 {
    message: async_nats::jetstream::Message,
}

impl SchedulerJetStreamReceiptDeliveryV1 {
    const fn new(message: async_nats::jetstream::Message) -> Self {
        Self { message }
    }
}

impl SchedulerReceiptDeliveryV1 for SchedulerJetStreamReceiptDeliveryV1 {
    fn exact_bytes(&self) -> &[u8] {
        self.message.payload.as_ref()
    }

    async fn acknowledge(self) -> Result<(), SchedulerReceiptDeliveryErrorV1> {
        self.message
            .ack()
            .await
            .map_err(|_| SchedulerReceiptDeliveryErrorV1::Unavailable)
    }
}

pub(super) async fn connect_context(
    endpoint: &str,
    credential: RuntimeNatsJwtCredentialV1,
) -> Result<async_nats::jetstream::Context, SchedulerJetStreamReceiptPortErrorV1> {
    let options = jwt_options(credential)?;
    let client = tokio::time::timeout(
        CONNECT_TIMEOUT,
        options
            .connection_timeout(CONNECT_TIMEOUT)
            .connect(endpoint),
    )
    .await
    .map_err(|_| SchedulerJetStreamReceiptPortErrorV1::Unavailable)?
    .map_err(|_| SchedulerJetStreamReceiptPortErrorV1::Unavailable)?;
    let mut context = async_nats::jetstream::new(client);
    context.set_timeout(REQUEST_TIMEOUT);
    Ok(context)
}

fn jwt_options(
    credential: RuntimeNatsJwtCredentialV1,
) -> Result<async_nats::ConnectOptions, SchedulerJetStreamReceiptPortErrorV1> {
    let (jwt, seed, expires_at_unix_seconds) = credential.into_connection_material();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| SchedulerJetStreamReceiptPortErrorV1::Unavailable)?
        .as_secs();
    if now >= expires_at_unix_seconds {
        return Err(SchedulerJetStreamReceiptPortErrorV1::ExpiredCredential);
    }
    let user_key = Arc::new(
        KeyPair::from_seed(seed.as_str())
            .map_err(|_| SchedulerJetStreamReceiptPortErrorV1::InvalidCredential)?,
    );
    Ok(async_nats::ConnectOptions::with_jwt(
        jwt.to_string(),
        move |nonce| {
            let user_key = Arc::clone(&user_key);
            async move { user_key.sign(&nonce).map_err(async_nats::AuthError::new) }
        },
    ))
}

async fn open_consumer(
    context: &async_nats::jetstream::Context,
    binding: &SchedulerRuntimeReceiptConsumerBindingV1,
) -> Result<PullConsumer, SchedulerJetStreamReceiptPortErrorV1> {
    let stream = context
        .get_stream(&binding.stream_name)
        .await
        .map_err(|_| SchedulerJetStreamReceiptPortErrorV1::Unavailable)?;
    let consumer = stream
        .get_consumer(&binding.durable_name)
        .await
        .map_err(|_| SchedulerJetStreamReceiptPortErrorV1::Unavailable)?;
    consumer_matches_binding(&consumer, binding)
        .then_some(consumer)
        .ok_or(SchedulerJetStreamReceiptPortErrorV1::BindingMismatch)
}

fn validate_binding(
    endpoint: &str,
    binding: &SchedulerRuntimeReceiptConsumerBindingV1,
) -> Result<(), SchedulerJetStreamReceiptPortErrorV1> {
    (endpoint.starts_with("nats://") && !endpoint.contains(['@', '?', '#', ' ']))
        .then_some(())
        .ok_or(SchedulerJetStreamReceiptPortErrorV1::InvalidBinding)?;
    validate_scheduler_runtime_receipt_consumer_binding(binding)
        .map_err(|_| SchedulerJetStreamReceiptPortErrorV1::InvalidBinding)
}

fn consumer_matches_binding(
    consumer: &PullConsumer,
    binding: &SchedulerRuntimeReceiptConsumerBindingV1,
) -> bool {
    let actual = &consumer.cached_info().config;
    actual.durable_name.as_deref() == Some(binding.durable_name.as_str())
        && actual.filter_subject == binding.filter_subject
        && actual.ack_policy == AckPolicy::Explicit
        && actual.ack_wait == Duration::from_millis(u64::from(binding.ack_wait_millis))
        && actual.max_deliver == i64::from(binding.max_deliver)
        && actual.max_ack_pending == i64::from(binding.max_ack_pending)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerJetStreamReceiptPortErrorV1 {
    BindingMismatch,
    ExpiredCredential,
    InvalidBinding,
    InvalidCredential,
    Unavailable,
}
