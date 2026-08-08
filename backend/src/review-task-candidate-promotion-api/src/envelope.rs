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
    REVIEW_TASK_CANDIDATE_PROMOTION_CONTRACT_MAJOR_V1,
    REVIEW_TASK_CANDIDATE_PROMOTION_CONTRACT_REVISION_V1, REVIEW_TASK_CANDIDATE_PROMOTION_OWNER_V1,
    REVIEW_TASK_CANDIDATE_PROMOTION_RESULT_CONTRACT_NAME_V1,
    REVIEW_TASK_CANDIDATE_PROMOTION_SCHEMA_SHA256_V1,
    wire::{ReviewTaskCandidatePromotionOutcomeV1, ReviewTaskCandidatePromotionResultV1},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewTaskCandidatePromotionEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewTaskCandidatePromotionEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_review_task_candidate_promotion_result_outbox_record_v1(
    causation_message_id: [u8; 16],
    payload: ReviewTaskCandidatePromotionResultV1,
    context: &ReviewTaskCandidatePromotionEnvelopeContextV1,
) -> Result<OutboxRecordV1, ReviewTaskCandidatePromotionEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let result_id = id16(&payload.result_id)?;
    let review_id = id16(&payload.review_id)?;
    id16(&payload.candidate_id)?;
    let outcome = ReviewTaskCandidatePromotionOutcomeV1::try_from(payload.outcome)
        .map_err(|_| ReviewTaskCandidatePromotionEnvelopeBuildErrorV1::InvalidPayload)?;
    let outcome_valid = match outcome {
        ReviewTaskCandidatePromotionOutcomeV1::ReviewTaskCandidatePromotionOutcomeSucceeded => {
            payload
                .task_id
                .as_deref()
                .is_some_and(|value| id16(value).is_ok())
                && payload.failure_code == 0
        }
        ReviewTaskCandidatePromotionOutcomeV1::ReviewTaskCandidatePromotionOutcomeFailed => {
            payload.task_id.is_none() && payload.failure_code > 0
        }
        ReviewTaskCandidatePromotionOutcomeV1::ReviewTaskCandidatePromotionOutcomeUnspecified => {
            false
        }
    };
    if causation_message_id.iter().all(|byte| *byte == 0)
        || payload.expected_review_revision == 0
        || !valid_owner(&payload.logical_owner_id)
        || !outcome_valid
    {
        return Err(ReviewTaskCandidatePromotionEnvelopeBuildErrorV1::InvalidPayload);
    }
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: result_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: REVIEW_TASK_CANDIDATE_PROMOTION_OWNER_V1.to_owned(),
            name: REVIEW_TASK_CANDIDATE_PROMOTION_RESULT_CONTRACT_NAME_V1.to_owned(),
            major: REVIEW_TASK_CANDIDATE_PROMOTION_CONTRACT_MAJOR_V1,
            revision: REVIEW_TASK_CANDIDATE_PROMOTION_CONTRACT_REVISION_V1,
            schema_sha256: REVIEW_TASK_CANDIDATE_PROMOTION_SCHEMA_SHA256_V1.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: digest(
                b"reviewed-task-promotion-runtime-v1",
                context.runtime_instance_id.as_bytes(),
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp(context)),
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
        semantics: Some(Semantics::Event(EventMetadataV1 {
            occurred_at: Some(timestamp(context)),
        })),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| ReviewTaskCandidatePromotionEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(
    context: &ReviewTaskCandidatePromotionEnvelopeContextV1,
) -> Result<(), ReviewTaskCandidatePromotionEnvelopeBuildErrorV1> {
    if !valid_owner(&context.module_id)
        || context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 128
        || !context.runtime_instance_id.is_ascii()
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(ReviewTaskCandidatePromotionEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], ReviewTaskCandidatePromotionEnvelopeBuildErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewTaskCandidatePromotionEnvelopeBuildErrorV1::InvalidPayload)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn timestamp(context: &ReviewTaskCandidatePromotionEnvelopeContextV1) -> Timestamp {
    Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    }
}

fn digest(label: &[u8], value: &[u8]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(label);
    hash.update([0]);
    hash.update(value);
    hash.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

fn outbox_error(_: OutboxRecordError) -> ReviewTaskCandidatePromotionEnvelopeBuildErrorV1 {
    ReviewTaskCandidatePromotionEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::{
        ReviewTaskCandidatePromotionFailureCodeV1, ReviewTaskCandidatePromotionOutcomeV1,
    };

    #[test]
    fn exact_result_event_binds_causation_and_review_partition() {
        let record = build_review_task_candidate_promotion_result_outbox_record_v1(
            [9; 16],
            ReviewTaskCandidatePromotionResultV1 {
                result_id: vec![1; 16],
                review_id: vec![2; 16],
                candidate_id: vec![3; 16],
                expected_review_revision: 2,
                outcome: ReviewTaskCandidatePromotionOutcomeV1::ReviewTaskCandidatePromotionOutcomeSucceeded as i32,
                task_id: Some(vec![4; 16]),
                failure_code: ReviewTaskCandidatePromotionFailureCodeV1::ReviewTaskCandidatePromotionFailureCodeUnspecified as i32,
                logical_owner_id: "owner-1".to_owned(),
            },
            &ReviewTaskCandidatePromotionEnvelopeContextV1 {
                module_id: "makosh-reviewed-task-candidate-promotion-runtime".to_owned(),
                runtime_instance_id: "runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("promotion result");
        assert_eq!(record.message_id(), &[1; 16]);
        let envelope =
            makosh_events_protocol::validation::envelope::decode_envelope_v1(record.exact_bytes())
                .expect("envelope");
        assert_eq!(envelope.partition_key, vec![2; 16]);
        assert_eq!(envelope.causation_message_id, vec![9; 16]);
    }
}
