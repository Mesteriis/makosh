use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_zulip_persistence::{ZulipDurablePersistence, ZulipDurablePersistenceError};

pub async fn relay_communications_outbox_once(
    durable: &ZulipDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, ZulipCommunicationsOutboxRelayError> {
    let records = durable
        .pending_communications_outbox(64)
        .await
        .map_err(ZulipCommunicationsOutboxRelayError::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, record.exact_bytes())
            .await
            .map_err(|_| ZulipCommunicationsOutboxRelayError::Unavailable)?;
        durable
            .mark_communications_outbox_published(record.message_id(), published_at_unix_seconds)
            .await
            .map_err(ZulipCommunicationsOutboxRelayError::Persistence)?;
        published += 1;
    }
    Ok(published)
}

#[derive(Debug)]
pub enum ZulipCommunicationsOutboxRelayError {
    Persistence(ZulipDurablePersistenceError),
    Unavailable,
}
