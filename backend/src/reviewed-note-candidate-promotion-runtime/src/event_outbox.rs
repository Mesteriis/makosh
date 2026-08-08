use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_reviewed_note_candidate_promotion_persistence::{
    ReviewedNoteCandidatePromotionPersistenceErrorV1, ReviewedNoteCandidatePromotionPersistenceV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PromotionEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(ReviewedNoteCandidatePromotionPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) async fn relay_promotion_outbox_once_v1(
    persistence: &ReviewedNoteCandidatePromotionPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, PromotionEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(PromotionEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(record) = persistence
        .unpublished_events(logical_owner_id, 1)
        .await
        .map_err(PromotionEventRelayErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, &record.envelope_bytes)
        .await
        .map_err(|_| PromotionEventRelayErrorV1::EventUnavailable)?;
    persistence
        .mark_event_published(
            logical_owner_id,
            &record.message_id,
            &record.envelope_sha256,
            published_at_unix_millis,
        )
        .await
        .map_err(PromotionEventRelayErrorV1::Persistence)?;
    Ok(true)
}
