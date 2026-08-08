//! Telegram-owned exact-byte relay for Communications observations.

use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeOutboxPublisherV1, RuntimePublishPermitV1,
};
use makosh_events_protocol::delivery::{OutboxRelayErrorV1, OutboxRelayOutcomeV1, relay_once};
use makosh_telegram_persistence::{
    TelegramCommunicationsOutboxStoreV1, TelegramDurablePersistence,
};

/// Publishes only records already committed in Telegram-owned PostgreSQL.
/// The permit is derived by Kernel from approved Event Hub topology; this
/// integration never constructs subjects or permissions itself.
pub async fn relay_communications_outbox_once(
    durable: &TelegramDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, TelegramCommunicationsOutboxRelayError> {
    let publisher = RuntimeOutboxPublisherV1::new(connection, permit);
    let mut store = TelegramCommunicationsOutboxStoreV1::new(durable, published_at_unix_seconds);
    let mut published = 0;
    for _ in 0..64 {
        match relay_once(&mut store, &publisher).await {
            Ok(OutboxRelayOutcomeV1::Idle) => break,
            Ok(OutboxRelayOutcomeV1::Published { .. }) => published += 1,
            Err(OutboxRelayErrorV1::Persistence) => {
                return Err(TelegramCommunicationsOutboxRelayError::Persistence);
            }
            Err(_) => return Err(TelegramCommunicationsOutboxRelayError::Unavailable),
        }
    }
    Ok(published)
}

#[derive(Debug)]
pub enum TelegramCommunicationsOutboxRelayError {
    Persistence,
    Unavailable,
}
