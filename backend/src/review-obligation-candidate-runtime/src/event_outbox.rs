use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_review_obligation_candidate_persistence::{
    ReviewObligationCandidatePersistenceErrorV1, ReviewObligationCandidatePersistenceV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReviewObligationCandidateEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(ReviewObligationCandidatePersistenceErrorV1),
    EventUnavailable,
}

pub(crate) async fn relay_review_obligation_candidate_outbox_once_v1(
    persistence: &ReviewObligationCandidatePersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, ReviewObligationCandidateEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(ReviewObligationCandidateEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(record) = persistence
        .unpublished_outbox(logical_owner_id, 1)
        .await
        .map_err(ReviewObligationCandidateEventRelayErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, &record.envelope_bytes)
        .await
        .map_err(|_| ReviewObligationCandidateEventRelayErrorV1::EventUnavailable)?;
    persistence
        .mark_outbox_published(
            logical_owner_id,
            &record.message_id,
            &record.envelope_sha256,
            published_at_unix_millis,
        )
        .await
        .map_err(ReviewObligationCandidateEventRelayErrorV1::Persistence)?;
    Ok(true)
}
