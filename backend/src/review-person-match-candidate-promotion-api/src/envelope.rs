use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1, ResultMetadataV1,
        ResultOutcomeV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_CONTRACT_NAME_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_SCHEMA_SHA256_V1,
    wire::{
        ReviewPersonMatchCandidatePromotionOutcomeV1, ReviewPersonMatchCandidatePromotionResultV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewPersonMatchCandidatePromotionEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewPersonMatchCandidatePromotionResultShapeV1 {
    PersonsTerminal,
    ActionDigestMismatch,
}

pub fn review_person_match_candidate_promotion_result_id_v1(
    causation_message_id: [u8; 16],
    decision_id: [u8; 16],
    shape: ReviewPersonMatchCandidatePromotionResultShapeV1,
) -> Result<[u8; 16], ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1> {
    if causation_message_id.iter().all(|byte| *byte == 0)
        || decision_id.iter().all(|byte| *byte == 0)
    {
        return Err(ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1::InvalidPayload);
    }
    let domain = match shape {
        ReviewPersonMatchCandidatePromotionResultShapeV1::PersonsTerminal => {
            b"reviewed-person-match-promotion-result-v1".as_slice()
        }
        ReviewPersonMatchCandidatePromotionResultShapeV1::ActionDigestMismatch => {
            b"reviewed-person-match-promotion-local-failure-v1".as_slice()
        }
    };
    Ok(digest16(domain, &causation_message_id, &decision_id))
}

pub fn build_review_person_match_candidate_promotion_result_outbox_record_v1(
    causation_message_id: [u8; 16],
    payload: ReviewPersonMatchCandidatePromotionResultV1,
    context: &ReviewPersonMatchCandidatePromotionEnvelopeContextV1,
) -> Result<OutboxRecordV1, ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1> {
    let result_id = id16(&payload.result_id)?;
    let review_id = id16(&payload.review_id)?;
    id16(&payload.candidate_id)?;
    let decision_id = id16(&payload.decision_id)?;
    if payload.expected_review_revision == 0
        || payload.completed_at_unix_millis != context.recorded_at_unix_millis
        || !valid_owner(&payload.logical_owner_id)
        || !matches!(
            ReviewPersonMatchCandidatePromotionOutcomeV1::try_from(payload.outcome),
            Ok(ReviewPersonMatchCandidatePromotionOutcomeV1::ReviewPersonMatchCandidatePromotionOutcomeSucceeded)
                | Ok(ReviewPersonMatchCandidatePromotionOutcomeV1::ReviewPersonMatchCandidatePromotionOutcomeFailed)
        )
    {
        return Err(ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1::InvalidPayload);
    }
    let succeeded = payload.outcome
        == ReviewPersonMatchCandidatePromotionOutcomeV1::ReviewPersonMatchCandidatePromotionOutcomeSucceeded as i32;
    let persons_command = payload
        .persons_command_id
        .as_ref()
        .map(|value| id16(value))
        .transpose()?;
    let failure = crate::wire::ReviewPersonMatchCandidatePromotionFailureCodeV1::try_from(
        payload.failure_code,
    )
    .map_err(|_| ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1::InvalidPayload)?;
    let shape = match (
        succeeded,
        persons_command,
        failure,
    ) {
        (
            true,
            Some(_),
            crate::wire::ReviewPersonMatchCandidatePromotionFailureCodeV1::ReviewPersonMatchCandidatePromotionFailureCodeUnspecified,
        )
        | (
            false,
            Some(_),
            crate::wire::ReviewPersonMatchCandidatePromotionFailureCodeV1::ReviewPersonMatchCandidatePromotionFailureCodePersonsRejected,
        ) => ReviewPersonMatchCandidatePromotionResultShapeV1::PersonsTerminal,
        (
            false,
            None,
            crate::wire::ReviewPersonMatchCandidatePromotionFailureCodeV1::ReviewPersonMatchCandidatePromotionFailureCodeActionDigestMismatch,
        ) => ReviewPersonMatchCandidatePromotionResultShapeV1::ActionDigestMismatch,
        _ => return Err(ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1::InvalidPayload),
    };
    if result_id
        != review_person_match_candidate_promotion_result_id_v1(
            causation_message_id,
            decision_id,
            shape,
        )?
    {
        return Err(ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1::InvalidPayload);
    }
    validate_context(context)?;
    let timestamp = Timestamp {
        seconds: context.recorded_at_unix_millis / 1_000,
        nanos: i32::try_from((context.recorded_at_unix_millis % 1_000) * 1_000_000)
            .map_err(|_| ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1::InvalidContext)?,
    };
    let payload_bytes = payload.encode_to_vec();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: result_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1.to_owned(),
            name: REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_CONTRACT_NAME_V1.to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: REVIEW_PERSON_MATCH_CANDIDATE_PROMOTION_SCHEMA_SHA256_V1.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: digest16(
                b"review-person-match-promotion-runtime-v1",
                context.runtime_instance_id.as_bytes(),
                context.module_id.as_bytes(),
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp),
        partition_key: review_id.to_vec(),
        causation_message_id: causation_message_id.to_vec(),
        correlation_id: review_id.to_vec(),
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
        semantics: Some(Semantics::Result(ResultMetadataV1 {
            command_id: payload.decision_id.clone(),
            command_message_id: payload.decision_id.clone(),
            outcome: if succeeded {
                ResultOutcomeV1::Succeeded as i32
            } else {
                ResultOutcomeV1::Rejected as i32
            },
            completed_at: Some(timestamp),
            execution_attempt: 1,
        })),
        payload: payload_bytes,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(
    context: &ReviewPersonMatchCandidatePromotionEnvelopeContextV1,
) -> Result<(), ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1> {
    if !context.module_id.is_empty()
        && context.module_id.len() <= 128
        && context.module_id.is_ascii()
        && !context.runtime_instance_id.is_empty()
        && context.runtime_instance_id.len() <= 128
        && context.runtime_instance_id.is_ascii()
        && context.runtime_generation > 0
        && context.recorded_at_unix_millis > 0
    {
        Ok(())
    } else {
        Err(ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1::InvalidContext)
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1::InvalidPayload)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn digest16(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut hash = Sha256::new();
    for value in [label, first, second] {
        hash.update((value.len() as u64).to_be_bytes());
        hash.update(value);
    }
    hash.finalize()[..16].try_into().expect("SHA-256 prefix")
}

const fn outbox_error(
    _: OutboxRecordError,
) -> ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1 {
    ReviewPersonMatchCandidatePromotionEnvelopeBuildErrorV1::OutboxRejected
}
