use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_knowledge_persistence::{KnowledgePersistenceErrorV1, KnowledgePersistenceV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KnowledgeEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(KnowledgePersistenceErrorV1),
    EventUnavailable,
}

pub(crate) async fn relay_knowledge_outbox_once_v1(
    persistence: &KnowledgePersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, KnowledgeEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(KnowledgeEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(claim) = persistence
        .claim_next_pending_outbox(logical_owner_id)
        .await
        .map_err(KnowledgeEventRelayErrorV1::Persistence)?
    else {
        return Ok(false);
    };
    let record = claim.record().clone();
    connection
        .publish_exact(permit, &record.envelope_bytes)
        .await
        .map_err(|_| KnowledgeEventRelayErrorV1::EventUnavailable)?;
    claim
        .mark_published(record.envelope_sha256, published_at_unix_millis)
        .await
        .map_err(KnowledgeEventRelayErrorV1::Persistence)?;
    Ok(true)
}
