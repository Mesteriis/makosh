//! Communications replay terminal-result relay.

use makosh_communications_retained_evidence_replay_persistence::{
    CommunicationsRetainedEvidenceReplayPersistenceV1, RetainedCommunicationsReplayErrorV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsReplayResultRelayErrorV1 {
    InvalidTimestamp,
    Persistence(RetainedCommunicationsReplayErrorV1),
    EventUnavailable,
}

pub async fn relay_communications_replay_result_once_v1(
    persistence: &CommunicationsRetainedEvidenceReplayPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<bool, CommunicationsReplayResultRelayErrorV1> {
    if published_at_unix_seconds <= 0 {
        return Err(CommunicationsReplayResultRelayErrorV1::InvalidTimestamp);
    }
    let Some(record) = persistence
        .pending_replay_results(1)
        .await
        .map_err(CommunicationsReplayResultRelayErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, record.exact_bytes())
        .await
        .map_err(|_| CommunicationsReplayResultRelayErrorV1::EventUnavailable)?;
    persistence
        .mark_replay_result_published(*record.message_id(), published_at_unix_seconds)
        .await
        .map_err(CommunicationsReplayResultRelayErrorV1::Persistence)?;
    Ok(true)
}
