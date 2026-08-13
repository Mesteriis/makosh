use makosh_documents_persistence::{DocumentsPersistenceErrorV1, DocumentsPersistenceV1};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DocumentsEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(DocumentsPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) async fn relay_documents_outbox_once_v1(
    persistence: &DocumentsPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, DocumentsEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(DocumentsEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(claim) = persistence
        .claim_next_pending_outbox(logical_owner_id)
        .await
        .map_err(DocumentsEventRelayErrorV1::Persistence)?
    else {
        return Ok(false);
    };
    let record = claim.record().clone();
    connection
        .publish_exact(permit, &record.envelope_bytes)
        .await
        .map_err(|_| DocumentsEventRelayErrorV1::EventUnavailable)?;
    claim
        .mark_published(record.envelope_sha256, published_at_unix_millis)
        .await
        .map_err(DocumentsEventRelayErrorV1::Persistence)?;
    Ok(true)
}
