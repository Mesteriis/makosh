use makosh_communications_retained_evidence_replay_contract::{
    validate_communications_replay_command_v1,
    wire::{
        ReplayCommunicationsEvidenceCommandV1, ReplayCommunicationsEvidenceFailureV1,
        ReplayCommunicationsEvidenceOutcomeV1, ReplayCommunicationsEvidenceResultV1,
    },
};
use makosh_communications_retained_evidence_replay_persistence::{
    CommunicationsRetainedEvidenceReplayPersistenceV1, RetainedCommunicationsReplayAuditV1,
    RetainedCommunicationsReplayErrorV1, RetainedCommunicationsReplayPhaseV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsRetainedEvidenceReplayErrorV1 {
    InvalidCommand,
    Persistence(RetainedCommunicationsReplayErrorV1),
    PublishUnavailable,
}

pub struct CommunicationsReplayExecutionContextV1<'a> {
    pub registration_id: &'a str,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub logical_attempt: u32,
    pub recorded_at_unix_seconds: i64,
}

pub async fn replay_retained_communications_evidence_v1(
    persistence: &CommunicationsRetainedEvidenceReplayPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    original_contract_publish_permit: &RuntimePublishPermitV1,
    command: &ReplayCommunicationsEvidenceCommandV1,
    context: &CommunicationsReplayExecutionContextV1<'_>,
) -> Result<ReplayCommunicationsEvidenceResultV1, CommunicationsRetainedEvidenceReplayErrorV1> {
    validate_communications_replay_command_v1(command)
        .map_err(|_| CommunicationsRetainedEvidenceReplayErrorV1::InvalidCommand)?;
    let operation_id = id16(&command.operation_id)?;
    let actor_sha256 = sha256(&command.owner_device_actor_sha256)?;
    let attachment_anchor_id = id16(&command.attachment_anchor_id)?;
    let retained = persistence
        .retained_attachment_safety_event(attachment_anchor_id)
        .await
        .map_err(CommunicationsRetainedEvidenceReplayErrorV1::Persistence)?;
    let message_id = *retained.record.message_id();
    let audit = |phase| RetainedCommunicationsReplayAuditV1 {
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
        .append_audit(&audit(RetainedCommunicationsReplayPhaseV1::Authorized))
        .await
        .map_err(CommunicationsRetainedEvidenceReplayErrorV1::Persistence)?;
    if connection
        .publish_exact(
            original_contract_publish_permit,
            retained.record.exact_bytes(),
        )
        .await
        .is_err()
    {
        persistence
            .append_audit(&audit(
                RetainedCommunicationsReplayPhaseV1::PublishUnavailable,
            ))
            .await
            .map_err(CommunicationsRetainedEvidenceReplayErrorV1::Persistence)?;
        return Err(CommunicationsRetainedEvidenceReplayErrorV1::PublishUnavailable);
    }
    persistence
        .append_audit(&audit(RetainedCommunicationsReplayPhaseV1::Published))
        .await
        .map_err(CommunicationsRetainedEvidenceReplayErrorV1::Persistence)?;
    Ok(ReplayCommunicationsEvidenceResultV1 {
        operation_id: command.operation_id.clone(),
        outcome: ReplayCommunicationsEvidenceOutcomeV1::Published as i32,
        original_message_ids: vec![message_id.to_vec()],
        failure: ReplayCommunicationsEvidenceFailureV1::Unspecified as i32,
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsRetainedEvidenceReplayErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationsRetainedEvidenceReplayErrorV1::InvalidCommand)
}

fn sha256(value: &[u8]) -> Result<[u8; 32], CommunicationsRetainedEvidenceReplayErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationsRetainedEvidenceReplayErrorV1::InvalidCommand)
}
