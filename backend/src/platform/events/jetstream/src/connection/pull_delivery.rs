//! Generic delivery of opaque bytes from one Kernel-authorized pull consumer.

use std::time::Duration;

use futures_util::StreamExt;

use super::{RuntimeJetStreamConnection, RuntimeSubscribePermitV1};

const RUNTIME_PULL_REQUEST_EXPIRES_V1: Duration = Duration::from_millis(250);
const RUNTIME_PULL_CALL_DEADLINE_V1: Duration = Duration::from_millis(500);

/// One unacknowledged JetStream message. Owner runtimes decide when it is safe
/// to acknowledge after their local inbox transaction has completed.
pub struct RuntimePullDeliveryV1 {
    message: async_nats::jetstream::Message,
}

impl RuntimePullDeliveryV1 {
    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        self.message.payload.as_ref()
    }

    pub async fn acknowledge(self) -> Result<(), RuntimePullDeliveryErrorV1> {
        self.message
            .ack()
            .await
            .map_err(|_| RuntimePullDeliveryErrorV1::Unavailable)
    }

    /// Leaves the delivery uncommitted and asks JetStream to redeliver it after
    /// a bounded delay. This is distinct from a successful acknowledgement and
    /// is used for normal cross-stream dependency reordering.
    pub async fn retry_after(self, delay: Duration) -> Result<(), RuntimePullDeliveryErrorV1> {
        if delay.is_zero() || delay > Duration::from_secs(5) {
            return Err(RuntimePullDeliveryErrorV1::Unavailable);
        }
        self.message
            .ack_with(async_nats::jetstream::AckKind::Nak(Some(delay)))
            .await
            .map_err(|_| RuntimePullDeliveryErrorV1::Unavailable)
    }
}

/// Receives one deadline-bounded delivery from exactly the Event Hub consumer
/// bound to the current runtime identity and grant epoch. The bound keeps an
/// owner runtime responsive to opposite-direction control during broker outage.
pub async fn receive_runtime_pull_delivery(
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
) -> Result<RuntimePullDeliveryV1, RuntimePullDeliveryErrorV1> {
    try_receive_runtime_pull_delivery(connection, permit)
        .await?
        .ok_or(RuntimePullDeliveryErrorV1::Unavailable)
}

/// Tries to receive one deadline-bounded delivery while keeping an idle pull
/// distinct from Event Hub unavailability. Long-running runtimes use this
/// contract so an empty, healthy consumer does not become an outage signal.
pub async fn try_receive_runtime_pull_delivery(
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
) -> Result<Option<RuntimePullDeliveryV1>, RuntimePullDeliveryErrorV1> {
    tokio::time::timeout(RUNTIME_PULL_CALL_DEADLINE_V1, async {
        let consumer = connection
            .open_pull_consumer(permit)
            .await
            .map_err(|_| unavailable_at("open_consumer"))?;
        let mut messages = consumer
            .fetch()
            .max_messages(1)
            .expires(RUNTIME_PULL_REQUEST_EXPIRES_V1)
            .messages()
            .await
            .map_err(|_| unavailable_at("fetch"))?;
        classify_next_delivery(messages.next().await)
            .map(|delivery| delivery.map(|message| RuntimePullDeliveryV1 { message }))
    })
    .await
    .map_err(|_| unavailable_at("deadline"))?
}

fn classify_next_delivery<T, E>(
    next: Option<Result<T, E>>,
) -> Result<Option<T>, RuntimePullDeliveryErrorV1> {
    match next {
        None => Ok(None),
        Some(Ok(delivery)) => Ok(Some(delivery)),
        Some(Err(_)) => Err(unavailable_at("delivery")),
    }
}

fn unavailable_at(stage: &str) -> RuntimePullDeliveryErrorV1 {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_runtime_pull_delivery_unavailable stage={stage}");
    }
    RuntimePullDeliveryErrorV1::Unavailable
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePullDeliveryErrorV1 {
    Unavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_pull_is_idle_not_unavailable() {
        assert_eq!(classify_next_delivery::<u8, ()>(None), Ok(None));
    }

    #[test]
    fn delivery_error_remains_unavailable() {
        assert_eq!(
            classify_next_delivery::<u8, ()>(Some(Err(()))),
            Err(RuntimePullDeliveryErrorV1::Unavailable),
        );
    }
}
