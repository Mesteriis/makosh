use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_mail_retained_evidence_replay_contract::{
    validate_mail_replay_command_v1,
    wire::{
        ReplayMailEvidenceCommandV1, ReplayMailEvidenceFailureV1, ReplayMailEvidenceOutcomeV1,
        ReplayMailEvidenceResultV1,
    },
};
use makosh_mail_retained_evidence_replay_persistence::{
    MailRetainedEvidenceReplayPersistenceV1, RetainedMailReplayAuditV1, RetainedMailReplayErrorV1,
    RetainedMailReplayPhaseV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailRetainedEvidenceReplayErrorV1 {
    InvalidCommand,
    Persistence(RetainedMailReplayErrorV1),
    PublishUnavailable,
}

pub struct MailReplayExecutionContextV1<'a> {
    pub registration_id: &'a str,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub logical_attempt: u32,
    pub recorded_at_unix_seconds: i64,
}

pub async fn replay_retained_mail_evidence_v1(
    persistence: &MailRetainedEvidenceReplayPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    original_contract_publish_permit: &RuntimePublishPermitV1,
    command: &ReplayMailEvidenceCommandV1,
    context: &MailReplayExecutionContextV1<'_>,
) -> Result<ReplayMailEvidenceResultV1, MailRetainedEvidenceReplayErrorV1> {
    validate_mail_replay_command_v1(command)
        .map_err(|_| MailRetainedEvidenceReplayErrorV1::InvalidCommand)?;
    let operation_id = id16(&command.operation_id)?;
    let actor_sha256 = sha256(&command.owner_device_actor_sha256)?;
    let attachment_anchor_id = id16(&command.attachment_anchor_id)?;
    let retained = persistence
        .retained_scan_candidate(attachment_anchor_id)
        .await
        .map_err(MailRetainedEvidenceReplayErrorV1::Persistence)?;
    let message_id = *retained.record.message_id();
    let audit = |phase| RetainedMailReplayAuditV1 {
        operation_id,
        logical_owner_id: command.logical_owner_id.clone(),
        owner_device_actor_sha256: actor_sha256,
        producer_registration_id: context.registration_id.to_owned(),
        producer_runtime_generation: context.runtime_generation,
        producer_grant_epoch: context.grant_epoch,
        logical_attempt: context.logical_attempt,
        original_message_id: message_id,
        original_envelope_sha256: *retained.record.envelope_sha256(),
        phase,
        recorded_at_unix_seconds: context.recorded_at_unix_seconds,
    };
    persistence
        .append_audit(&audit(RetainedMailReplayPhaseV1::Authorized))
        .await
        .map_err(MailRetainedEvidenceReplayErrorV1::Persistence)?;
    if connection
        .publish_exact(
            original_contract_publish_permit,
            retained.record.exact_bytes(),
        )
        .await
        .is_err()
    {
        persistence
            .append_audit(&audit(RetainedMailReplayPhaseV1::PublishUnavailable))
            .await
            .map_err(MailRetainedEvidenceReplayErrorV1::Persistence)?;
        return Err(MailRetainedEvidenceReplayErrorV1::PublishUnavailable);
    }
    persistence
        .append_audit(&audit(RetainedMailReplayPhaseV1::Published))
        .await
        .map_err(MailRetainedEvidenceReplayErrorV1::Persistence)?;
    Ok(ReplayMailEvidenceResultV1 {
        operation_id: command.operation_id.clone(),
        outcome: ReplayMailEvidenceOutcomeV1::Published as i32,
        original_message_ids: vec![message_id.to_vec()],
        failure: ReplayMailEvidenceFailureV1::Unspecified as i32,
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailRetainedEvidenceReplayErrorV1> {
    value
        .try_into()
        .map_err(|_| MailRetainedEvidenceReplayErrorV1::InvalidCommand)
}

fn sha256(value: &[u8]) -> Result<[u8; 32], MailRetainedEvidenceReplayErrorV1> {
    value
        .try_into()
        .map_err(|_| MailRetainedEvidenceReplayErrorV1::InvalidCommand)
}
