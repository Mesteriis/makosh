//! Mail-owned exact-byte relay for Communications observations.

use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_mail_persistence::{MailDurablePersistence, MailDurablePersistenceError};

pub async fn relay_communications_outbox_once(
    durable: &MailDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, MailCommunicationsOutboxRelayError> {
    let records = durable
        .pending_communications_outbox(64)
        .await
        .map_err(MailCommunicationsOutboxRelayError::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, record.exact_bytes())
            .await
            .map_err(|_| MailCommunicationsOutboxRelayError::Unavailable)?;
        durable
            .mark_communications_outbox_published(record.message_id(), published_at_unix_seconds)
            .await
            .map_err(MailCommunicationsOutboxRelayError::Persistence)?;
        published += 1;
    }
    Ok(published)
}

#[derive(Debug)]
pub enum MailCommunicationsOutboxRelayError {
    Persistence(MailDurablePersistenceError),
    Unavailable,
}
