use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1,
        EventMetadataV1, FenceKindV1, ResultMetadataV1, ResultOutcomeV1, SourceFenceV1,
        SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    REVIEW_TASK_CANDIDATE_APPROVED_CONTRACT_NAME_V1, REVIEW_TASK_CANDIDATE_CONTRACT_MAJOR_V1,
    REVIEW_TASK_CANDIDATE_CONTRACT_REVISION_V1, REVIEW_TASK_CANDIDATE_MAX_BLOB_BYTES_V1,
    REVIEW_TASK_CANDIDATE_MAX_PROOF_BYTES_V1, REVIEW_TASK_CANDIDATE_OWNER_V1,
    REVIEW_TASK_CANDIDATE_SCHEMA_SHA256_V1, REVIEW_TASK_CANDIDATE_SUBMISSION_CAPABILITY_ID_V1,
    REVIEW_TASK_CANDIDATE_SUBMISSION_REJECTED_CONTRACT_NAME_V1,
    REVIEW_TASK_CANDIDATE_SUBMIT_CONTRACT_NAME_V1,
    REVIEW_TASK_CANDIDATE_SUBMITTED_CONTRACT_NAME_V1,
    wire::{
        SubmitTaskCandidateForReviewCommandV1, TaskCandidateApprovedForPromotionV1,
        TaskCandidateReviewSubmissionRejectedV1, TaskCandidateReviewSubmittedV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewTaskCandidateEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewTaskCandidateEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_submit_review_task_candidate_outbox_record_v1(
    payload: SubmitTaskCandidateForReviewCommandV1,
    deadline_unix_seconds: i64,
    context: &ReviewTaskCandidateEnvelopeContextV1,
) -> Result<OutboxRecordV1, ReviewTaskCandidateEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let submission_id = validate_submission(&payload)?;
    if deadline_unix_seconds <= context.recorded_at_unix_seconds {
        return Err(ReviewTaskCandidateEnvelopeBuildErrorV1::InvalidPayload);
    }
    build(
        submission_id,
        submission_id,
        Vec::new(),
        REVIEW_TASK_CANDIDATE_SUBMIT_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: submission_id.to_vec(),
            target_capability: REVIEW_TASK_CANDIDATE_SUBMISSION_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: digest(b"review-task-candidate-submit-v1", &submission_id).to_vec(),
            deadline: Some(Timestamp {
                seconds: deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        }),
        payload.encode_to_vec(),
        ActorKindV1::Module,
        context.module_id.as_bytes().to_vec(),
        context,
    )
}

pub fn build_review_task_candidate_submitted_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: TaskCandidateReviewSubmittedV1,
    context: &ReviewTaskCandidateEnvelopeContextV1,
) -> Result<OutboxRecordV1, ReviewTaskCandidateEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let submission_id = id16(&payload.submission_id)?;
    id16(&payload.review_id)?;
    id16(&payload.candidate_id)?;
    id32(&payload.candidate_digest)?;
    if !valid_id(&command_message_id)
        || payload.review_revision == 0
        || !valid_owner(&payload.logical_owner_id)
    {
        return Err(ReviewTaskCandidateEnvelopeBuildErrorV1::InvalidPayload);
    }
    result(
        b"submitted",
        command_message_id,
        submission_id,
        REVIEW_TASK_CANDIDATE_SUBMITTED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Succeeded,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_review_task_candidate_submission_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: TaskCandidateReviewSubmissionRejectedV1,
    context: &ReviewTaskCandidateEnvelopeContextV1,
) -> Result<OutboxRecordV1, ReviewTaskCandidateEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let submission_id = id16(&payload.submission_id)?;
    id16(&payload.candidate_id)?;
    if !valid_id(&command_message_id)
        || payload.code == 0
        || !valid_owner(&payload.logical_owner_id)
    {
        return Err(ReviewTaskCandidateEnvelopeBuildErrorV1::InvalidPayload);
    }
    result(
        b"rejected",
        command_message_id,
        submission_id,
        REVIEW_TASK_CANDIDATE_SUBMISSION_REJECTED_CONTRACT_NAME_V1,
        ResultOutcomeV1::Rejected,
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_review_task_candidate_approved_outbox_record_v1(
    payload: TaskCandidateApprovedForPromotionV1,
    context: &ReviewTaskCandidateEnvelopeContextV1,
) -> Result<OutboxRecordV1, ReviewTaskCandidateEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let review_id = id16(&payload.review_id)?;
    id16(&payload.candidate_id)?;
    id32(&payload.candidate_digest)?;
    id16(&payload.source_evidence_id)?;
    let actor = id16(&payload.decided_by_owner_device_id)?;
    if payload.source_evidence_revision == 0
        || payload.decision_revision == 0
        || !valid_owner(&payload.logical_owner_id)
        || payload.candidate_content.as_ref().is_none_or(|value| {
            !valid_receipt(
                &value.reference_id,
                value.declared_bytes,
                &value.sha256,
                &value.custody_transfer_source_proof,
            )
        })
    {
        return Err(ReviewTaskCandidateEnvelopeBuildErrorV1::InvalidPayload);
    }
    let message_id = digest_with_revision(
        b"review-task-candidate-approved-v1",
        &review_id,
        payload.decision_revision,
    );
    build(
        message_id,
        review_id,
        Vec::new(),
        REVIEW_TASK_CANDIDATE_APPROVED_CONTRACT_NAME_V1,
        Semantics::Event(EventMetadataV1 {
            occurred_at: Some(timestamp(context)),
        }),
        payload.encode_to_vec(),
        ActorKindV1::OwnerDevice,
        actor.to_vec(),
        context,
    )
}

fn result(
    label: &[u8],
    command_message_id: [u8; 16],
    command_id: [u8; 16],
    contract: &str,
    outcome: ResultOutcomeV1,
    payload: Vec<u8>,
    context: &ReviewTaskCandidateEnvelopeContextV1,
) -> Result<OutboxRecordV1, ReviewTaskCandidateEnvelopeBuildErrorV1> {
    build(
        digest(label, &command_id),
        command_id,
        command_message_id.to_vec(),
        contract,
        Semantics::Result(ResultMetadataV1 {
            command_id: command_id.to_vec(),
            command_message_id: command_message_id.to_vec(),
            outcome: outcome as i32,
            completed_at: Some(timestamp(context)),
            execution_attempt: 1,
        }),
        payload,
        ActorKindV1::Module,
        context.module_id.as_bytes().to_vec(),
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
    actor_kind: ActorKindV1,
    actor_id: Vec<u8>,
    context: &ReviewTaskCandidateEnvelopeContextV1,
) -> Result<OutboxRecordV1, ReviewTaskCandidateEnvelopeBuildErrorV1> {
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: REVIEW_TASK_CANDIDATE_OWNER_V1.to_owned(),
            name: contract_name.to_owned(),
            major: REVIEW_TASK_CANDIDATE_CONTRACT_MAJOR_V1,
            revision: REVIEW_TASK_CANDIDATE_CONTRACT_REVISION_V1,
            schema_sha256: REVIEW_TASK_CANDIDATE_SCHEMA_SHA256_V1.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: digest(b"runtime", context.runtime_instance_id.as_bytes())
                .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp(context)),
        partition_key: partition_key.to_vec(),
        causation_message_id,
        correlation_id: partition_key.to_vec(),
        actor: Some(ActorRefV1 {
            kind: actor_kind as i32,
            actor_id,
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
        .map_err(|_| ReviewTaskCandidateEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_submission(
    payload: &SubmitTaskCandidateForReviewCommandV1,
) -> Result<[u8; 16], ReviewTaskCandidateEnvelopeBuildErrorV1> {
    let submission_id = id16(&payload.submission_id)?;
    id16(&payload.candidate_id)?;
    id32(&payload.candidate_digest)?;
    id16(&payload.source_evidence_id)?;
    if payload.source_evidence_revision == 0
        || !valid_owner(&payload.logical_owner_id)
        || payload.candidate_content.as_ref().is_none_or(|value| {
            !valid_receipt(
                &value.reference_id,
                value.declared_bytes,
                &value.sha256,
                &value.custody_transfer_source_proof,
            )
        })
    {
        return Err(ReviewTaskCandidateEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(submission_id)
}

fn validate_context(
    context: &ReviewTaskCandidateEnvelopeContextV1,
) -> Result<(), ReviewTaskCandidateEnvelopeBuildErrorV1> {
    if !valid_owner(&context.module_id)
        || context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 256
        || !context.runtime_instance_id.is_ascii()
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(ReviewTaskCandidateEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn valid_receipt(reference: &[u8], bytes: u64, sha256: &[u8], proof: &[u8]) -> bool {
    reference.len() == 16
        && reference.iter().any(|byte| *byte != 0)
        && (1..=REVIEW_TASK_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&bytes)
        && sha256.len() == 32
        && sha256.iter().any(|byte| *byte != 0)
        && !proof.is_empty()
        && proof.len() <= REVIEW_TASK_CANDIDATE_MAX_PROOF_BYTES_V1
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}
fn valid_id<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}
fn id16(value: &[u8]) -> Result<[u8; 16], ReviewTaskCandidateEnvelopeBuildErrorV1> {
    value
        .try_into()
        .ok()
        .filter(valid_id)
        .ok_or(ReviewTaskCandidateEnvelopeBuildErrorV1::InvalidPayload)
}
fn id32(value: &[u8]) -> Result<[u8; 32], ReviewTaskCandidateEnvelopeBuildErrorV1> {
    value
        .try_into()
        .ok()
        .filter(valid_id)
        .ok_or(ReviewTaskCandidateEnvelopeBuildErrorV1::InvalidPayload)
}
fn timestamp(context: &ReviewTaskCandidateEnvelopeContextV1) -> Timestamp {
    Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    }
}
fn digest(label: &[u8], id: &[u8]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(label);
    hash.update([0]);
    hash.update(id);
    hash.finalize()[..16].try_into().expect("digest prefix")
}
fn digest_with_revision(label: &[u8], id: &[u8], revision: u64) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(label);
    hash.update([0]);
    hash.update(id);
    hash.update(revision.to_be_bytes());
    hash.finalize()[..16].try_into().expect("digest prefix")
}
fn outbox_error(_: OutboxRecordError) -> ReviewTaskCandidateEnvelopeBuildErrorV1 {
    ReviewTaskCandidateEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::{
        ReviewTargetBoundCandidateReceiptV1, ReviewTaskCandidateSubmissionRejectCodeV1,
    };

    fn context() -> ReviewTaskCandidateEnvelopeContextV1 {
        ReviewTaskCandidateEnvelopeContextV1 {
            module_id: "makosh-review-task-candidate-runtime".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 2,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 3,
        }
    }
    fn receipt() -> ReviewTargetBoundCandidateReceiptV1 {
        ReviewTargetBoundCandidateReceiptV1 {
            reference_id: vec![5; 16],
            declared_bytes: 7,
            sha256: vec![6; 32],
            custody_transfer_source_proof: vec![7; 64],
        }
    }

    #[test]
    fn approval_is_human_actor_and_private_text_free() {
        let record = build_review_task_candidate_approved_outbox_record_v1(
            TaskCandidateApprovedForPromotionV1 {
                review_id: vec![1; 16],
                candidate_id: vec![2; 16],
                candidate_digest: vec![3; 32],
                source_evidence_id: vec![4; 16],
                source_evidence_revision: 1,
                decision_revision: 2,
                decided_by_owner_device_id: vec![8; 16],
                candidate_content: Some(receipt()),
                logical_owner_id: "owner-1".to_owned(),
            },
            &context(),
        )
        .expect("record");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        assert_eq!(
            envelope.actor.expect("actor").kind,
            ActorKindV1::OwnerDevice as i32
        );
        assert!(!String::from_utf8_lossy(record.exact_bytes()).contains("private title"));
    }

    #[test]
    fn rejection_is_an_exact_terminal_result() {
        let record = build_review_task_candidate_submission_rejected_outbox_record_v1([9; 16], TaskCandidateReviewSubmissionRejectedV1 { submission_id: vec![1; 16], candidate_id: vec![2; 16], code: ReviewTaskCandidateSubmissionRejectCodeV1::ReviewTaskCandidateSubmissionRejectCodeConflict as i32, logical_owner_id: "owner-1".to_owned() }, &context()).expect("record");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        assert!(matches!(envelope.semantics, Some(Semantics::Result(_))));
    }
}
