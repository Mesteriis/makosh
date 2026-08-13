use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, EventMetadataV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    REVIEW_PERSON_MATCH_CANDIDATE_APPROVED_CONTRACT_NAME_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_CONTRACT_MAJOR_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_CONTRACT_REVISION_V1, REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_SCHEMA_SHA256_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_SUBMISSION_REJECTED_CONTRACT_NAME_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_SUBMITTED_CONTRACT_NAME_V1,
    wire::{
        PersonMatchCandidateApprovedForPromotionV1, PersonMatchCandidateReviewSubmissionRejectedV1,
        PersonMatchCandidateReviewSubmittedV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewPersonMatchCandidateEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewPersonMatchCandidateEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_review_person_match_candidate_submitted_outbox_record_v1(
    causation_message_id: [u8; 16],
    payload: PersonMatchCandidateReviewSubmittedV1,
    context: &ReviewPersonMatchCandidateEnvelopeContextV1,
) -> Result<OutboxRecordV1, ReviewPersonMatchCandidateEnvelopeBuildErrorV1> {
    let review_id = id16(&payload.review_id)?;
    id16(&payload.submission_id)?;
    id16(&payload.candidate_id)?;
    id32(&payload.candidate_digest)?;
    if payload.review_revision == 0 || !valid_owner(&payload.logical_owner_id) {
        return Err(ReviewPersonMatchCandidateEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_event(
        digest16(
            b"review-person-match-submitted-v1",
            &causation_message_id,
            &review_id,
        ),
        review_id,
        causation_message_id,
        REVIEW_PERSON_MATCH_CANDIDATE_SUBMITTED_CONTRACT_NAME_V1,
        payload.encode_to_vec(),
        ActorKindV1::Module,
        context.module_id.as_bytes().to_vec(),
        timestamp(context)?,
        context,
    )
}

pub fn build_review_person_match_candidate_submission_rejected_outbox_record_v1(
    causation_message_id: [u8; 16],
    payload: PersonMatchCandidateReviewSubmissionRejectedV1,
    context: &ReviewPersonMatchCandidateEnvelopeContextV1,
) -> Result<OutboxRecordV1, ReviewPersonMatchCandidateEnvelopeBuildErrorV1> {
    let submission_id = id16(&payload.submission_id)?;
    id16(&payload.candidate_id)?;
    if payload.code == 0 || !valid_owner(&payload.logical_owner_id) {
        return Err(ReviewPersonMatchCandidateEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_event(
        digest16(
            b"review-person-match-rejected-v1",
            &causation_message_id,
            &submission_id,
        ),
        submission_id,
        causation_message_id,
        REVIEW_PERSON_MATCH_CANDIDATE_SUBMISSION_REJECTED_CONTRACT_NAME_V1,
        payload.encode_to_vec(),
        ActorKindV1::Module,
        context.module_id.as_bytes().to_vec(),
        timestamp(context)?,
        context,
    )
}

pub fn build_review_person_match_candidate_approved_outbox_record_v1(
    payload: PersonMatchCandidateApprovedForPromotionV1,
    context: &ReviewPersonMatchCandidateEnvelopeContextV1,
) -> Result<OutboxRecordV1, ReviewPersonMatchCandidateEnvelopeBuildErrorV1> {
    let review_id = id16(&payload.review_id)?;
    id16(&payload.candidate_id)?;
    id32(&payload.candidate_digest)?;
    let decision_id = id16(&payload.decision_id)?;
    let device = id16(&payload.decided_by_owner_device_id)?;
    id32(&payload.approved_action_digest)?;
    if payload.decision_revision == 0
        || payload.decided_at_unix_millis <= 0
        || payload
            .approved_action
            .as_ref()
            .is_none_or(|value| value.action.is_none())
        || !valid_owner(&payload.logical_owner_id)
    {
        return Err(ReviewPersonMatchCandidateEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_event(
        digest16(b"review-person-match-approved-v1", &decision_id, &review_id),
        review_id,
        decision_id,
        REVIEW_PERSON_MATCH_CANDIDATE_APPROVED_CONTRACT_NAME_V1,
        payload.encode_to_vec(),
        ActorKindV1::OwnerDevice,
        device.to_vec(),
        timestamp_millis(payload.decided_at_unix_millis)?,
        context,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_event(
    message_id: [u8; 16],
    partition_key: [u8; 16],
    causation_message_id: [u8; 16],
    contract_name: &str,
    payload: Vec<u8>,
    actor_kind: ActorKindV1,
    actor_id: Vec<u8>,
    occurred_at: Timestamp,
    context: &ReviewPersonMatchCandidateEnvelopeContextV1,
) -> Result<OutboxRecordV1, ReviewPersonMatchCandidateEnvelopeBuildErrorV1> {
    validate_context(context)?;
    id16(&message_id)?;
    id16(&partition_key)?;
    id16(&causation_message_id)?;
    let timestamp = timestamp(context)?;
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1.to_owned(),
            name: contract_name.to_owned(),
            major: REVIEW_PERSON_MATCH_CANDIDATE_CONTRACT_MAJOR_V1,
            revision: REVIEW_PERSON_MATCH_CANDIDATE_CONTRACT_REVISION_V1,
            schema_sha256: REVIEW_PERSON_MATCH_CANDIDATE_SCHEMA_SHA256_V1.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: digest16(
                b"review-person-match-runtime-v1",
                context.runtime_instance_id.as_bytes(),
                context.module_id.as_bytes(),
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp),
        partition_key: partition_key.to_vec(),
        causation_message_id: causation_message_id.to_vec(),
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
        semantics: Some(Semantics::Event(EventMetadataV1 {
            occurred_at: Some(occurred_at),
        })),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| ReviewPersonMatchCandidateEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(
    context: &ReviewPersonMatchCandidateEnvelopeContextV1,
) -> Result<(), ReviewPersonMatchCandidateEnvelopeBuildErrorV1> {
    if valid_module(&context.module_id)
        && valid_module(&context.runtime_instance_id)
        && context.runtime_generation > 0
        && context.recorded_at_unix_millis > 0
    {
        Ok(())
    } else {
        Err(ReviewPersonMatchCandidateEnvelopeBuildErrorV1::InvalidContext)
    }
}

fn timestamp(
    context: &ReviewPersonMatchCandidateEnvelopeContextV1,
) -> Result<Timestamp, ReviewPersonMatchCandidateEnvelopeBuildErrorV1> {
    Ok(Timestamp {
        seconds: context.recorded_at_unix_millis / 1_000,
        nanos: i32::try_from((context.recorded_at_unix_millis % 1_000) * 1_000_000)
            .map_err(|_| ReviewPersonMatchCandidateEnvelopeBuildErrorV1::InvalidContext)?,
    })
}

fn timestamp_millis(
    value: i64,
) -> Result<Timestamp, ReviewPersonMatchCandidateEnvelopeBuildErrorV1> {
    if value <= 0 {
        return Err(ReviewPersonMatchCandidateEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(Timestamp {
        seconds: value / 1_000,
        nanos: i32::try_from((value % 1_000) * 1_000_000)
            .map_err(|_| ReviewPersonMatchCandidateEnvelopeBuildErrorV1::InvalidPayload)?,
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], ReviewPersonMatchCandidateEnvelopeBuildErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewPersonMatchCandidateEnvelopeBuildErrorV1::InvalidPayload)
}

fn id32(value: &[u8]) -> Result<[u8; 32], ReviewPersonMatchCandidateEnvelopeBuildErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewPersonMatchCandidateEnvelopeBuildErrorV1::InvalidPayload)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_module(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.is_ascii()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn digest16(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update((label.len() as u64).to_be_bytes());
    hash.update(label);
    for value in [first, second] {
        hash.update((value.len() as u64).to_be_bytes());
        hash.update(value);
    }
    let digest: [u8; 32] = hash.finalize().into();
    digest[..16].try_into().expect("SHA-256 prefix")
}

const fn outbox_error(_: OutboxRecordError) -> ReviewPersonMatchCandidateEnvelopeBuildErrorV1 {
    ReviewPersonMatchCandidateEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use crate::wire::{
        AttachPersonSourceReviewActionV1, PersonMatchCandidateApprovedActionV1,
        person_match_candidate_approved_action_v1::Action,
    };

    use super::*;

    fn context() -> ReviewPersonMatchCandidateEnvelopeContextV1 {
        ReviewPersonMatchCandidateEnvelopeContextV1 {
            module_id: "makosh-review-person-match-candidate-runtime".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 3,
            recorded_at_unix_millis: 1_800_000_000_000,
        }
    }

    #[test]
    fn approved_event_is_deterministic_owner_device_evidence() {
        let payload = PersonMatchCandidateApprovedForPromotionV1 {
            review_id: vec![1; 16],
            candidate_id: vec![2; 16],
            candidate_digest: vec![3; 32],
            decision_id: vec![4; 16],
            decision_revision: 2,
            decided_by_owner_device_id: vec![5; 16],
            decided_at_unix_millis: 1_800_000_000_000,
            approved_action: Some(PersonMatchCandidateApprovedActionV1 {
                action: Some(Action::Attach(AttachPersonSourceReviewActionV1 {
                    from_person_id: vec![6; 16],
                    expected_from_person_revision: 1,
                    to_person_id: vec![7; 16],
                    expected_to_person_revision: 1,
                    source: None,
                    expected_source_revision: 1,
                })),
            }),
            approved_action_digest: vec![8; 32],
            logical_owner_id: "owner-1".to_owned(),
        };
        let first = build_review_person_match_candidate_approved_outbox_record_v1(
            payload.clone(),
            &context(),
        )
        .expect("approval");
        let second =
            build_review_person_match_candidate_approved_outbox_record_v1(payload, &context())
                .expect("approval replay");
        assert_eq!(first.exact_bytes(), second.exact_bytes());
        let envelope = DurableEnvelopeV1::decode(first.exact_bytes()).expect("envelope");
        assert_eq!(
            envelope.actor.expect("actor").kind,
            ActorKindV1::OwnerDevice as i32
        );
    }
}
