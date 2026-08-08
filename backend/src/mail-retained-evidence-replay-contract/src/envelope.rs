//! Exact Mail replay terminal-result envelope builder.

use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ResultMetadataV1, ResultOutcomeV1, SourceFenceV1, SourceRefV1,
        durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    MAIL_REPLAY_CAPABILITY_ID_V1, MAIL_REPLAY_SOURCE_MODULE_ID_V1, MAIL_REPLAY_TARGET_MODULE_ID_V1,
    mail_replay_command_contract_reference_v1, mail_replay_result_contract_reference_v1,
    validate_mail_replay_command_v1, validate_mail_replay_result_v1,
    wire::{ReplayMailEvidenceCommandV1, ReplayMailEvidenceOutcomeV1, ReplayMailEvidenceResultV1},
};

const COMMAND_MESSAGE_DOMAIN_V1: &[u8] = b"makosh.mail.retained-evidence-replay.command.v1";
const RESULT_MESSAGE_DOMAIN_V1: &[u8] = b"makosh.mail.retained-evidence-replay.result.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailReplayCommandEnvelopeContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
    pub deadline_unix_seconds: i64,
    pub logical_attempt: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailReplayCommandEnvelopeErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailReplayResultEnvelopeContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub completed_at_unix_seconds: i64,
    pub completed_at_nanos: i32,
    pub execution_attempt: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailReplayResultEnvelopeErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
}

pub fn build_mail_replay_command_outbox_v1(
    command: ReplayMailEvidenceCommandV1,
    context: &MailReplayCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailReplayCommandEnvelopeErrorV1> {
    validate_command_context(context)?;
    validate_mail_replay_command_v1(&command)
        .map_err(|_| MailReplayCommandEnvelopeErrorV1::InvalidPayload)?;
    let operation_id = command_id16(&command.operation_id)?;
    let contract = mail_replay_command_contract_reference_v1();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: identifier(COMMAND_MESSAGE_DOMAIN_V1, operation_id).to_vec(),
        contract: Some(ContractRefV1 {
            owner: contract.owner,
            name: contract.name,
            major: contract.major,
            revision: contract.revision,
            schema_sha256: contract.schema_sha256,
        }),
        source: Some(SourceRefV1 {
            module_id: MAIL_REPLAY_SOURCE_MODULE_ID_V1.to_owned(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(Timestamp {
            seconds: context.recorded_at_unix_seconds,
            nanos: context.recorded_at_nanos,
        }),
        partition_key: operation_id.to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: operation_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::OwnerDevice as i32,
            actor_id: command.owner_device_actor_sha256.clone(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: MAIL_REPLAY_SOURCE_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: operation_id.to_vec(),
            target_capability: MAIL_REPLAY_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: Sha256::digest(
                [COMMAND_MESSAGE_DOMAIN_V1, operation_id.as_slice()].concat(),
            )
            .to_vec(),
            deadline: Some(Timestamp {
                seconds: context.deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: context.logical_attempt,
        })),
        payload: command.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailReplayCommandEnvelopeErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(command_outbox_error)
}

pub fn build_mail_replay_result_outbox_v1(
    command_message_id: [u8; 16],
    result: ReplayMailEvidenceResultV1,
    context: &MailReplayResultEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailReplayResultEnvelopeErrorV1> {
    validate_context(context)?;
    validate_mail_replay_result_v1(&result)
        .map_err(|_| MailReplayResultEnvelopeErrorV1::InvalidPayload)?;
    let operation_id = id16(&result.operation_id)?;
    if command_message_id.iter().all(|byte| *byte == 0) {
        return Err(MailReplayResultEnvelopeErrorV1::InvalidPayload);
    }
    let outcome = ReplayMailEvidenceOutcomeV1::try_from(result.outcome)
        .map_err(|_| MailReplayResultEnvelopeErrorV1::InvalidPayload)?;
    let result_outcome = match outcome {
        ReplayMailEvidenceOutcomeV1::Published | ReplayMailEvidenceOutcomeV1::AlreadyPublished => {
            ResultOutcomeV1::Succeeded
        }
        ReplayMailEvidenceOutcomeV1::Rejected => ResultOutcomeV1::Rejected,
        ReplayMailEvidenceOutcomeV1::Unavailable => ResultOutcomeV1::Failed,
        ReplayMailEvidenceOutcomeV1::Unspecified => {
            return Err(MailReplayResultEnvelopeErrorV1::InvalidPayload);
        }
    };
    let completed_at = Timestamp {
        seconds: context.completed_at_unix_seconds,
        nanos: context.completed_at_nanos,
    };
    let contract = mail_replay_result_contract_reference_v1();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: identifier(RESULT_MESSAGE_DOMAIN_V1, operation_id).to_vec(),
        contract: Some(ContractRefV1 {
            owner: contract.owner,
            name: contract.name,
            major: contract.major,
            revision: contract.revision,
            schema_sha256: contract.schema_sha256,
        }),
        source: Some(SourceRefV1 {
            module_id: MAIL_REPLAY_TARGET_MODULE_ID_V1.to_owned(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(completed_at),
        partition_key: operation_id.to_vec(),
        causation_message_id: command_message_id.to_vec(),
        correlation_id: operation_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: MAIL_REPLAY_TARGET_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: MAIL_REPLAY_TARGET_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Result(ResultMetadataV1 {
            command_id: operation_id.to_vec(),
            command_message_id: command_message_id.to_vec(),
            outcome: result_outcome as i32,
            completed_at: Some(Timestamp {
                seconds: context.completed_at_unix_seconds,
                nanos: context.completed_at_nanos,
            }),
            execution_attempt: context.execution_attempt,
        })),
        payload: result.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| MailReplayResultEnvelopeErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(
    context: &MailReplayResultEnvelopeContextV1,
) -> Result<(), MailReplayResultEnvelopeErrorV1> {
    if context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 128
        || !context.runtime_instance_id.is_ascii()
        || context.runtime_generation == 0
        || context.completed_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.completed_at_nanos)
        || context.execution_attempt == 0
    {
        return Err(MailReplayResultEnvelopeErrorV1::InvalidContext);
    }
    Ok(())
}

fn validate_command_context(
    context: &MailReplayCommandEnvelopeContextV1,
) -> Result<(), MailReplayCommandEnvelopeErrorV1> {
    if context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 128
        || !context.runtime_instance_id.is_ascii()
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
        || context.deadline_unix_seconds <= context.recorded_at_unix_seconds
        || context.logical_attempt == 0
    {
        return Err(MailReplayCommandEnvelopeErrorV1::InvalidContext);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailReplayResultEnvelopeErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(MailReplayResultEnvelopeErrorV1::InvalidPayload)
}

fn command_id16(value: &[u8]) -> Result<[u8; 16], MailReplayCommandEnvelopeErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(MailReplayCommandEnvelopeErrorV1::InvalidPayload)
}

fn identifier(domain: &[u8], identity: [u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(identity);
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16]
        .try_into()
        .expect("SHA-256 prefix has fixed size")
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    let digest: [u8; 32] = Sha256::digest(runtime_instance_id.as_bytes()).into();
    digest[..16]
        .try_into()
        .expect("SHA-256 prefix has fixed size")
}

fn outbox_error(_: OutboxRecordError) -> MailReplayResultEnvelopeErrorV1 {
    MailReplayResultEnvelopeErrorV1::InvalidEnvelope
}

fn command_outbox_error(_: OutboxRecordError) -> MailReplayCommandEnvelopeErrorV1 {
    MailReplayCommandEnvelopeErrorV1::InvalidEnvelope
}

#[cfg(test)]
mod tests {
    use makosh_events_protocol::{
        v1::{ResultOutcomeV1, durable_envelope_v1::Semantics},
        validation::envelope::decode_envelope_v1,
    };

    use super::*;
    use crate::wire::{ReplayMailEvidenceFailureV1, ReplayMailEvidenceOutcomeV1};

    #[test]
    fn terminal_result_is_exact_causal_and_metadata_only() {
        let result = ReplayMailEvidenceResultV1 {
            operation_id: vec![1; 16],
            outcome: ReplayMailEvidenceOutcomeV1::Published as i32,
            original_message_ids: vec![vec![2; 16]],
            failure: ReplayMailEvidenceFailureV1::Unspecified as i32,
        };
        let record = build_mail_replay_result_outbox_v1(
            [3; 16],
            result,
            &MailReplayResultEnvelopeContextV1 {
                runtime_instance_id: "mail-runtime-1".to_owned(),
                runtime_generation: 7,
                completed_at_unix_seconds: 1_700_000_000,
                completed_at_nanos: 0,
                execution_attempt: 2,
            },
        )
        .expect("result");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        assert_eq!(envelope.partition_key, vec![1; 16]);
        assert_eq!(envelope.causation_message_id, vec![3; 16]);
        let Semantics::Result(metadata) = envelope.semantics.expect("result") else {
            panic!("result semantics");
        };
        assert_eq!(metadata.outcome, ResultOutcomeV1::Succeeded as i32);
        assert_eq!(metadata.execution_attempt, 2);
    }

    #[test]
    fn command_is_owner_actor_and_exact_capability_bound() {
        let command = ReplayMailEvidenceCommandV1 {
            operation_id: vec![1; 16],
            logical_owner_id: "owner-1".to_owned(),
            owner_device_actor_sha256: vec![2; 32],
            attachment_anchor_id: vec![3; 16],
        };
        let record = build_mail_replay_command_outbox_v1(
            command,
            &MailReplayCommandEnvelopeContextV1 {
                runtime_instance_id: "replay-runtime-1".to_owned(),
                runtime_generation: 11,
                recorded_at_unix_seconds: 1_700_000_000,
                recorded_at_nanos: 0,
                deadline_unix_seconds: 1_700_000_030,
                logical_attempt: 1,
            },
        )
        .expect("command");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        assert_eq!(
            envelope.source.expect("source").module_id,
            MAIL_REPLAY_SOURCE_MODULE_ID_V1
        );
        let Semantics::Command(metadata) = envelope.semantics.expect("command") else {
            panic!("command semantics");
        };
        assert_eq!(metadata.target_capability, MAIL_REPLAY_CAPABILITY_ID_V1);
        assert_eq!(metadata.logical_attempt, 1);
    }
}
