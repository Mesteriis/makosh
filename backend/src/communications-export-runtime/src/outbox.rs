//! Exact-byte workflow outbox relay.

use makosh_communications_export_persistence::{
    CommunicationsExportPersistenceErrorV1, CommunicationsExportPersistenceV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsExportOutboxErrorV1 {
    StorageUnavailable,
    EventUnavailable,
}

pub async fn relay_communications_export_outbox_v1(
    persistence: &CommunicationsExportPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, CommunicationsExportOutboxErrorV1> {
    let records = persistence
        .pending_outbox(64)
        .await
        .map_err(persistence_error)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, record.exact_bytes())
            .await
            .map_err(|_| CommunicationsExportOutboxErrorV1::EventUnavailable)?;
        persistence
            .mark_outbox_published(*record.message_id(), published_at_unix_seconds)
            .await
            .map_err(persistence_error)?;
        published += 1;
    }
    Ok(published)
}

fn persistence_error(
    _: CommunicationsExportPersistenceErrorV1,
) -> CommunicationsExportOutboxErrorV1 {
    CommunicationsExportOutboxErrorV1::StorageUnavailable
}
