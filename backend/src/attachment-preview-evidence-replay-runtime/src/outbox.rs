use makosh_attachment_preview_evidence_replay_core::ReplayProducerV1;
use makosh_attachment_preview_evidence_replay_persistence::{
    AttachmentPreviewEvidenceReplayPersistenceV1, ReplayPersistenceErrorV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayCommandRelayErrorV1 {
    InvalidTimestamp,
    Persistence(ReplayPersistenceErrorV1),
    EventUnavailable,
}

pub async fn relay_replay_commands_once_v1(
    persistence: &AttachmentPreviewEvidenceReplayPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    communications_permit: &RuntimePublishPermitV1,
    mail_permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, ReplayCommandRelayErrorV1> {
    if published_at_unix_seconds <= 0 {
        return Err(ReplayCommandRelayErrorV1::InvalidTimestamp);
    }
    let commands = persistence
        .pending_commands(64)
        .await
        .map_err(ReplayCommandRelayErrorV1::Persistence)?;
    let mut published = 0;
    for command in commands {
        let permit = match command.producer {
            ReplayProducerV1::Communications => communications_permit,
            ReplayProducerV1::Mail => mail_permit,
        };
        connection
            .publish_exact(permit, &command.exact_envelope_bytes)
            .await
            .map_err(|_| ReplayCommandRelayErrorV1::EventUnavailable)?;
        persistence
            .mark_command_published(command.message_id, published_at_unix_seconds)
            .await
            .map_err(ReplayCommandRelayErrorV1::Persistence)?;
        published += 1;
    }
    Ok(published)
}
