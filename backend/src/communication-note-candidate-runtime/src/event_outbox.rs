use makosh_communication_note_candidate_persistence::{
    CommunicationNoteCandidatePersistenceErrorV1, CommunicationNoteCandidatePersistenceV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationNoteCandidateEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(CommunicationNoteCandidatePersistenceErrorV1),
    EventUnavailable,
}

pub async fn relay_outbox_once_v1(
    persistence: &CommunicationNoteCandidatePersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, CommunicationNoteCandidateEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(CommunicationNoteCandidateEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(record) = persistence
        .unpublished_events(logical_owner_id, 1)
        .await
        .map_err(CommunicationNoteCandidateEventRelayErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, &record.envelope_bytes)
        .await
        .map_err(|_| CommunicationNoteCandidateEventRelayErrorV1::EventUnavailable)?;
    persistence
        .mark_event_published(
            logical_owner_id,
            &record.message_id,
            &record.envelope_sha256,
            published_at_unix_millis,
        )
        .await
        .map_err(CommunicationNoteCandidateEventRelayErrorV1::Persistence)?;
    Ok(true)
}
