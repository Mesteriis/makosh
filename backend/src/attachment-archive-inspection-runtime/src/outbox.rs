//! Exact custody-command relay; persisted bytes are never re-encoded.

use makosh_attachment_archive_inspection_persistence::{
    ArchiveInspectionPersistenceErrorV1, AttachmentArchiveInspectionPersistenceV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

pub async fn relay_archive_custody_outbox_once_v1(
    persistence: &AttachmentArchiveInspectionPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<usize, ArchiveInspectionOutboxErrorV1> {
    let records = persistence
        .unpublished_custody_delegation_outbox(logical_owner_id, 64)
        .await
        .map_err(ArchiveInspectionOutboxErrorV1::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, &record.exact_envelope_bytes)
            .await
            .map_err(|_| ArchiveInspectionOutboxErrorV1::Unavailable)?;
        persistence
            .mark_custody_delegation_published(
                logical_owner_id,
                record.message_id,
                record.envelope_sha256,
                published_at_unix_millis,
            )
            .await
            .map_err(ArchiveInspectionOutboxErrorV1::Persistence)?;
        published += 1;
    }
    Ok(published)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionOutboxErrorV1 {
    Persistence(ArchiveInspectionPersistenceErrorV1),
    Unavailable,
}
