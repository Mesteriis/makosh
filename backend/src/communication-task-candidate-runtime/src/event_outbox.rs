use makosh_communication_task_candidate_persistence::{
    CommunicationTaskCandidatePersistenceErrorV1, CommunicationTaskCandidatePersistenceV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTaskCandidateEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(CommunicationTaskCandidatePersistenceErrorV1),
    EventUnavailable,
}

pub async fn relay_outbox_once_v1(
    persistence: &CommunicationTaskCandidatePersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, CommunicationTaskCandidateEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(CommunicationTaskCandidateEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(record) = persistence
        .unpublished_events(logical_owner_id, 1)
        .await
        .map_err(CommunicationTaskCandidateEventRelayErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, &record.envelope_bytes)
        .await
        .map_err(|_| CommunicationTaskCandidateEventRelayErrorV1::EventUnavailable)?;
    persistence
        .mark_event_published(
            logical_owner_id,
            &record.message_id,
            &record.envelope_sha256,
            published_at_unix_millis,
        )
        .await
        .map_err(CommunicationTaskCandidateEventRelayErrorV1::Persistence)?;
    Ok(true)
}
