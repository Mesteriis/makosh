use makosh_attachment_text_extraction_persistence::{
    AttachmentTextExtractionPersistenceErrorV1, AttachmentTextExtractionPersistenceV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

pub(crate) async fn relay_custody_outbox_once_v1(
    persistence: &AttachmentTextExtractionPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<usize, OutboxErrorV1> {
    let records = persistence
        .unpublished_custody_delegation_outbox(logical_owner_id, 64)
        .await
        .map_err(OutboxErrorV1::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, &record.exact_envelope_bytes)
            .await
            .map_err(|_| OutboxErrorV1::Unavailable)?;
        persistence
            .mark_custody_delegation_published(
                logical_owner_id,
                record.message_id,
                record.envelope_sha256,
                published_at_unix_millis,
            )
            .await
            .map_err(OutboxErrorV1::Persistence)?;
        published += 1;
    }
    Ok(published)
}

pub(crate) async fn relay_translation_source_outbox_once_v1(
    persistence: &AttachmentTextExtractionPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<usize, OutboxErrorV1> {
    let records = persistence
        .unpublished_translation_source_outbox(logical_owner_id, 64)
        .await
        .map_err(OutboxErrorV1::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, &record.exact_envelope_bytes)
            .await
            .map_err(|_| OutboxErrorV1::Unavailable)?;
        persistence
            .mark_translation_source_published(
                logical_owner_id,
                record.message_id,
                record.envelope_sha256,
                published_at_unix_millis,
            )
            .await
            .map_err(OutboxErrorV1::Persistence)?;
        published += 1;
    }
    Ok(published)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OutboxErrorV1 {
    Persistence(AttachmentTextExtractionPersistenceErrorV1),
    Unavailable,
}
