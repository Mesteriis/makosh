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
    CREATE_TASK_FROM_REVIEWED_CANDIDATE_CONTRACT_NAME_V1,
    TASK_CREATED_FROM_REVIEWED_CANDIDATE_CONTRACT_NAME_V1,
    TASK_CREATION_FROM_REVIEWED_CANDIDATE_REJECTED_CONTRACT_NAME_V1,
    TASKS_COMMAND_CONTRACT_MAJOR_V1, TASKS_COMMAND_CONTRACT_REVISION_V1,
    TASKS_COMMAND_SCHEMA_SHA256_V1, TASKS_OWNER_ID_V1,
    TASKS_REVIEWED_CANDIDATE_COMMAND_CAPABILITY_ID_V1, TASKS_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1,
    TASKS_REVIEWED_CANDIDATE_MAX_PROOF_BYTES_V1,
    wire::{
        CreateTaskFromReviewedCandidateCommandV1, TaskCreatedFromReviewedCandidateV1,
        TaskCreationFromReviewedCandidateRejectedV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TasksCommandEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TasksCommandEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_create_task_from_reviewed_candidate_outbox_record_v1(
    payload: CreateTaskFromReviewedCandidateCommandV1,
    deadline_unix_seconds: i64,
    context: &TasksCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, TasksCommandEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let command_id = validate_command(&payload)?;
    if deadline_unix_seconds <= context.recorded_at_unix_seconds {
        return Err(TasksCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    build(
        command_id,
        id16(&payload.approved_candidate_id)?,
        Vec::new(),
        CREATE_TASK_FROM_REVIEWED_CANDIDATE_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: command_id.to_vec(),
            target_capability: TASKS_REVIEWED_CANDIDATE_COMMAND_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: digest(
                b"tasks-reviewed-candidate-idempotency-v1",
                payload.logical_owner_id.as_bytes(),
                &payload.approved_candidate_id,
            )
            .to_vec(),
            deadline: Some(Timestamp {
                seconds: deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        }),
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_task_created_from_reviewed_candidate_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: TaskCreatedFromReviewedCandidateV1,
    context: &TasksCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, TasksCommandEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let command_id = id16(&payload.command_id)?;
    let candidate_id = id16(&payload.approved_candidate_id)?;
    id16(&payload.task_id)?;
    if !nonzero(&command_message_id)
        || payload.task_revision == 0
        || !valid_owner(&payload.logical_owner_id)
    {
        return Err(TasksCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    result(
        b"tasks-reviewed-candidate-created-v1",
        command_message_id,
        command_id,
        candidate_id,
        TASK_CREATED_FROM_REVIEWED_CANDIDATE_CONTRACT_NAME_V1,
        ResultOutcomeV1::Succeeded,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_task_creation_from_reviewed_candidate_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: TaskCreationFromReviewedCandidateRejectedV1,
    context: &TasksCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, TasksCommandEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let command_id = id16(&payload.command_id)?;
    let candidate_id = id16(&payload.approved_candidate_id)?;
    if !nonzero(&command_message_id) || payload.code == 0 || !valid_owner(&payload.logical_owner_id)
    {
        return Err(TasksCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    result(
        b"tasks-reviewed-candidate-rejected-v1",
        command_message_id,
        command_id,
        candidate_id,
        TASK_CREATION_FROM_REVIEWED_CANDIDATE_REJECTED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Rejected,
        payload.encode_to_vec(),
        context,
    )
}

#[allow(clippy::too_many_arguments)]
fn result(
    label: &[u8],
    command_message_id: [u8; 16],
    command_id: [u8; 16],
    candidate_id: [u8; 16],
    contract_name: &str,
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
    context: &TasksCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, TasksCommandEnvelopeBuildErrorV1> {
    build(
        digest(label, &command_id, &candidate_id),
        candidate_id,
        command_message_id.to_vec(),
        contract_name,
        Semantics::Result(ResultMetadataV1 {
            command_id: command_id.to_vec(),
            command_message_id: command_message_id.to_vec(),
            outcome: outcome as i32,
            completed_at: Some(timestamp(context)),
            execution_attempt: 1,
        }),
        payload,
        context,
    )
}

#[allow(clippy::too_many_arguments)]
fn build(
    message_id: [u8; 16],
    partition_key: [u8; 16],
    causation_message_id: Vec<u8>,
    contract_name: &str,
    semantics: Semantics,
    payload: Vec<u8>,
    context: &TasksCommandEnvelopeContextV1,
) -> Result<OutboxRecordV1, TasksCommandEnvelopeBuildErrorV1> {
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: TASKS_OWNER_ID_V1.to_owned(),
            name: contract_name.to_owned(),
            major: TASKS_COMMAND_CONTRACT_MAJOR_V1,
            revision: TASKS_COMMAND_CONTRACT_REVISION_V1,
            schema_sha256: TASKS_COMMAND_SCHEMA_SHA256_V1.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: digest(
                b"tasks-runtime-instance-v1",
                context.runtime_instance_id.as_bytes(),
                b"source",
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp(context)),
        partition_key: partition_key.to_vec(),
        causation_message_id,
        correlation_id: partition_key.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: context.module_id.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: context.module_id.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(semantics),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| TasksCommandEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_command(
    payload: &CreateTaskFromReviewedCandidateCommandV1,
) -> Result<[u8; 16], TasksCommandEnvelopeBuildErrorV1> {
    let command_id = id16(&payload.command_id)?;
    id16(&payload.approved_candidate_id)?;
    id32(&payload.candidate_digest)?;
    id16(&payload.source_evidence_id)?;
    id16(&payload.review_id)?;
    id16(&payload.decided_by_owner_device_id)?;
    if payload.source_evidence_revision == 0
        || payload.decision_revision == 0
        || !valid_owner(&payload.logical_owner_id)
        || payload.candidate_content.as_ref().is_none_or(|receipt| {
            id16(&receipt.reference_id).is_err()
                || receipt.declared_bytes == 0
                || receipt.declared_bytes > TASKS_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1
                || id32(&receipt.sha256).is_err()
                || receipt.custody_transfer_source_proof.is_empty()
                || receipt.custody_transfer_source_proof.len()
                    > TASKS_REVIEWED_CANDIDATE_MAX_PROOF_BYTES_V1
        })
    {
        return Err(TasksCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(command_id)
}

fn validate_context(
    context: &TasksCommandEnvelopeContextV1,
) -> Result<(), TasksCommandEnvelopeBuildErrorV1> {
    if context.module_id.is_empty()
        || context.module_id.len() > 128
        || context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 128
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(TasksCommandEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn id16(bytes: &[u8]) -> Result<[u8; 16], TasksCommandEnvelopeBuildErrorV1> {
    let value: [u8; 16] = bytes
        .try_into()
        .map_err(|_| TasksCommandEnvelopeBuildErrorV1::InvalidPayload)?;
    if !nonzero(&value) {
        return Err(TasksCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(value)
}

fn id32(bytes: &[u8]) -> Result<[u8; 32], TasksCommandEnvelopeBuildErrorV1> {
    let value: [u8; 32] = bytes
        .try_into()
        .map_err(|_| TasksCommandEnvelopeBuildErrorV1::InvalidPayload)?;
    if !nonzero(&value) {
        return Err(TasksCommandEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(value)
}

fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn timestamp(context: &TasksCommandEnvelopeContextV1) -> Timestamp {
    Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    }
}

fn digest(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(label);
    hasher.update([0]);
    hasher.update(first);
    hasher.update([0]);
    hasher.update(second);
    hasher.finalize()[..16].try_into().expect("fixed digest")
}

fn outbox_error(_: OutboxRecordError) -> TasksCommandEnvelopeBuildErrorV1 {
    TasksCommandEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::TasksTargetBoundCandidateReceiptV1;

    fn context() -> TasksCommandEnvelopeContextV1 {
        TasksCommandEnvelopeContextV1 {
            module_id: "promotion-workflow".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 3,
        }
    }

    fn command() -> CreateTaskFromReviewedCandidateCommandV1 {
        CreateTaskFromReviewedCandidateCommandV1 {
            command_id: vec![1; 16],
            approved_candidate_id: vec![2; 16],
            candidate_digest: vec![3; 32],
            source_evidence_id: vec![4; 16],
            source_evidence_revision: 5,
            review_id: vec![6; 16],
            decision_revision: 7,
            decided_by_owner_device_id: vec![8; 16],
            candidate_content: Some(TasksTargetBoundCandidateReceiptV1 {
                reference_id: vec![9; 16],
                declared_bytes: 32,
                sha256: vec![10; 32],
                custody_transfer_source_proof: vec![11; 32],
            }),
            logical_owner_id: "owner-1".to_owned(),
        }
    }

    #[test]
    fn command_envelope_is_tasks_owned_and_idempotent_by_candidate() {
        let record = build_create_task_from_reviewed_candidate_outbox_record_v1(
            command(),
            1_800_000_100,
            &context(),
        )
        .expect("command");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        assert_eq!(envelope.contract.expect("contract").owner, "tasks");
        let Some(Semantics::Command(metadata)) = envelope.semantics else {
            panic!("command semantics");
        };
        assert_eq!(
            metadata.target_capability,
            TASKS_REVIEWED_CANDIDATE_COMMAND_CAPABILITY_ID_V1
        );
        assert_eq!(metadata.idempotency_key.len(), 16);
    }

    #[test]
    fn command_rejects_missing_decision_and_blob_proof() {
        let mut payload = command();
        payload.decision_revision = 0;
        assert_eq!(
            build_create_task_from_reviewed_candidate_outbox_record_v1(
                payload,
                1_800_000_100,
                &context(),
            ),
            Err(TasksCommandEnvelopeBuildErrorV1::InvalidPayload)
        );
        let mut payload = command();
        payload
            .candidate_content
            .as_mut()
            .expect("receipt")
            .custody_transfer_source_proof
            .clear();
        assert_eq!(
            build_create_task_from_reviewed_candidate_outbox_record_v1(
                payload,
                1_800_000_100,
                &context(),
            ),
            Err(TasksCommandEnvelopeBuildErrorV1::InvalidPayload)
        );
    }
}
