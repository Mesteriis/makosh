use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_relationships_persistence::{
    RelationshipsPersistenceErrorV1, RelationshipsPersistenceV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RelationshipsEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(RelationshipsPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) async fn relay_relationships_outbox_once_v1(
    persistence: &RelationshipsPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, RelationshipsEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(RelationshipsEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(claim) = persistence
        .claim_next_pending_outbox(logical_owner_id)
        .await
        .map_err(RelationshipsEventRelayErrorV1::Persistence)?
    else {
        return Ok(false);
    };
    let record = claim.record().clone();
    connection
        .publish_exact(permit, &record.envelope_bytes)
        .await
        .map_err(|_| RelationshipsEventRelayErrorV1::EventUnavailable)?;
    claim
        .mark_published(record.envelope_sha256, published_at_unix_millis)
        .await
        .map_err(RelationshipsEventRelayErrorV1::Persistence)?;
    Ok(true)
}
