//! Telegram Calls exact-byte relay for public Communications call evidence.

use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeOutboxPublisherV1, RuntimePublishPermitV1,
};
use makosh_events_protocol::delivery::{OutboxRelayErrorV1, OutboxRelayOutcomeV1, relay_once};
use makosh_telegram_calls_persistence::{
    TelegramCallEvidenceOutboxStoreV1, TelegramCallsPersistence,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramCallEvidenceOutboxRelayErrorV1 {
    Persistence,
    Unavailable,
}

pub async fn relay_call_evidence_outbox_once_v1(
    persistence: &TelegramCallsPersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, TelegramCallEvidenceOutboxRelayErrorV1> {
    let publisher = RuntimeOutboxPublisherV1::new(connection, permit);
    let mut store = TelegramCallEvidenceOutboxStoreV1::new(persistence, published_at_unix_seconds);
    let mut published = 0;
    for _ in 0..32 {
        match relay_once(&mut store, &publisher).await {
            Ok(OutboxRelayOutcomeV1::Idle) => break,
            Ok(OutboxRelayOutcomeV1::Published { .. }) => published += 1,
            Err(OutboxRelayErrorV1::Persistence) => {
                return Err(TelegramCallEvidenceOutboxRelayErrorV1::Persistence);
            }
            Err(_) => return Err(TelegramCallEvidenceOutboxRelayErrorV1::Unavailable),
        }
    }
    Ok(published)
}
