//! Mail replay terminal-result relay.

use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_mail_retained_evidence_replay_persistence::{
    MailRetainedEvidenceReplayPersistenceV1, RetainedMailReplayErrorV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailReplayResultRelayErrorV1 {
    InvalidTimestamp,
    Persistence(RetainedMailReplayErrorV1),
    EventUnavailable,
}

pub async fn relay_mail_replay_result_once_v1(
    persistence: &MailRetainedEvidenceReplayPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<bool, MailReplayResultRelayErrorV1> {
    if published_at_unix_seconds <= 0 {
        return Err(MailReplayResultRelayErrorV1::InvalidTimestamp);
    }
    let Some(record) = persistence
        .pending_replay_results(1)
        .await
        .map_err(MailReplayResultRelayErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, record.exact_bytes())
        .await
        .map_err(|_| MailReplayResultRelayErrorV1::EventUnavailable)?;
    persistence
        .mark_replay_result_published(*record.message_id(), published_at_unix_seconds)
        .await
        .map_err(MailReplayResultRelayErrorV1::Persistence)?;
    Ok(true)
}
