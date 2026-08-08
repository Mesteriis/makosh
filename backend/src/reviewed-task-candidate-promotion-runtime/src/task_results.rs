use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ResultMetadataV1, ResultOutcomeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_review_task_candidate_promotion_api::{
    ReviewTaskCandidatePromotionEnvelopeContextV1,
    build_review_task_candidate_promotion_result_outbox_record_v1,
    wire::{
        ReviewTaskCandidatePromotionFailureCodeV1, ReviewTaskCandidatePromotionOutcomeV1,
        ReviewTaskCandidatePromotionResultV1,
    },
};
use makosh_reviewed_task_candidate_promotion_core::derive_reviewed_task_candidate_result_id_v1;
use makosh_reviewed_task_candidate_promotion_persistence::{
    PersistPromotionTerminalResultV1, ReviewedTaskCandidatePromotionOutcomeV1 as StoredOutcome,
    ReviewedTaskCandidatePromotionPersistenceErrorV1, ReviewedTaskCandidatePromotionPersistenceV1,
};
use makosh_tasks_command_api::{
    task_created_from_reviewed_candidate_contract_reference_v1,
    task_creation_from_reviewed_candidate_rejected_contract_reference_v1,
    wire::{
        TaskCreatedFromReviewedCandidateV1, TaskCreationFromReviewedCandidateRejectedV1,
        TaskCreationRejectCodeV1,
    },
};
use prost::Message;

use crate::validation::{id16, valid_owner, validate_contract};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReviewedTaskCandidatePromotionEventErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(ReviewedTaskCandidatePromotionPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) struct ReviewedTaskCandidatePromotionRuntimeContextV1<'a> {
    pub logical_human_owner_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub(crate) async fn consume_task_created_once_v1(
    persistence: &ReviewedTaskCandidatePromotionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &ReviewedTaskCandidatePromotionRuntimeContextV1<'_>,
) -> Result<bool, ReviewedTaskCandidatePromotionEventErrorV1> {
    consume_result_once(
        persistence,
        connection,
        permit,
        runtime,
        TaskResultKindV1::Created,
    )
    .await
}

pub(crate) async fn consume_task_rejected_once_v1(
    persistence: &ReviewedTaskCandidatePromotionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &ReviewedTaskCandidatePromotionRuntimeContextV1<'_>,
) -> Result<bool, ReviewedTaskCandidatePromotionEventErrorV1> {
    consume_result_once(
        persistence,
        connection,
        permit,
        runtime,
        TaskResultKindV1::Rejected,
    )
    .await
}

#[derive(Clone, Copy)]
enum TaskResultKindV1 {
    Created,
    Rejected,
}

async fn consume_result_once(
    persistence: &ReviewedTaskCandidatePromotionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &ReviewedTaskCandidatePromotionRuntimeContextV1<'_>,
    kind: TaskResultKindV1,
) -> Result<bool, ReviewedTaskCandidatePromotionEventErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReviewedTaskCandidatePromotionEventErrorV1::InvalidEnvelope)?;
    let result = decode_result(&record, runtime.logical_human_owner_id, kind)?;
    let correlation = persistence
        .load_correlation(runtime.logical_human_owner_id, &result.command_id)
        .await
        .map_err(ReviewedTaskCandidatePromotionEventErrorV1::Persistence)?;
    if correlation.candidate_id != result.candidate_id {
        return Err(ReviewedTaskCandidatePromotionEventErrorV1::InvalidPayload);
    }
    let result_id = derive_reviewed_task_candidate_result_id_v1(
        *record.message_id(),
        result.command_id,
        correlation.review_id,
    )
    .map_err(|_| ReviewedTaskCandidatePromotionEventErrorV1::InvalidPayload)?;
    let (wire_outcome, task_id, failure_code, stored_outcome) = match result.outcome {
        DecodedTaskOutcomeV1::Succeeded { task_id } => (
            ReviewTaskCandidatePromotionOutcomeV1::ReviewTaskCandidatePromotionOutcomeSucceeded,
            Some(task_id.to_vec()),
            ReviewTaskCandidatePromotionFailureCodeV1::ReviewTaskCandidatePromotionFailureCodeUnspecified,
            StoredOutcome::Succeeded { task_id },
        ),
        DecodedTaskOutcomeV1::Failed { failure_code } => (
            ReviewTaskCandidatePromotionOutcomeV1::ReviewTaskCandidatePromotionOutcomeFailed,
            None,
            promotion_failure_code(failure_code)?,
            StoredOutcome::Failed {
                failure_code: failure_code as u16,
            },
        ),
    };
    let review_result = build_review_task_candidate_promotion_result_outbox_record_v1(
        *record.message_id(),
        ReviewTaskCandidatePromotionResultV1 {
            result_id: result_id.to_vec(),
            review_id: correlation.review_id.to_vec(),
            candidate_id: correlation.candidate_id.to_vec(),
            expected_review_revision: correlation.decision_revision,
            outcome: wire_outcome as i32,
            task_id,
            failure_code: failure_code as i32,
            logical_owner_id: runtime.logical_human_owner_id.to_owned(),
        },
        &promotion_context(runtime),
    )
    .map_err(|_| ReviewedTaskCandidatePromotionEventErrorV1::InvalidPayload)?;
    persistence
        .persist_tasks_result_and_review_result(&PersistPromotionTerminalResultV1 {
            logical_owner_id: runtime.logical_human_owner_id.to_owned(),
            tasks_result_message_id: *record.message_id(),
            tasks_result_envelope_sha256: *record.envelope_sha256(),
            tasks_command_id: result.command_id,
            review_id: correlation.review_id,
            candidate_id: correlation.candidate_id,
            outcome: stored_outcome,
            review_result_outbox: review_result,
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(ReviewedTaskCandidatePromotionEventErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

struct DecodedTaskResultV1 {
    command_id: [u8; 16],
    candidate_id: [u8; 16],
    outcome: DecodedTaskOutcomeV1,
}

enum DecodedTaskOutcomeV1 {
    Succeeded {
        task_id: [u8; 16],
    },
    Failed {
        failure_code: TaskCreationRejectCodeV1,
    },
}

fn decode_result(
    record: &OutboxRecordV1,
    expected_owner: &str,
    kind: TaskResultKindV1,
) -> Result<DecodedTaskResultV1, ReviewedTaskCandidatePromotionEventErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ReviewedTaskCandidatePromotionEventErrorV1::InvalidEnvelope)?;
    let expected_contract = match kind {
        TaskResultKindV1::Created => task_created_from_reviewed_candidate_contract_reference_v1(),
        TaskResultKindV1::Rejected => {
            task_creation_from_reviewed_candidate_rejected_contract_reference_v1()
        }
    };
    validate_contract(&envelope, &expected_contract)?;
    let Some(Semantics::Result(ResultMetadataV1 {
        command_id,
        command_message_id,
        outcome,
        ..
    })) = envelope.semantics
    else {
        return Err(ReviewedTaskCandidatePromotionEventErrorV1::InvalidEnvelope);
    };
    let command_id = id16(&command_id)?;
    let expected_outcome = match kind {
        TaskResultKindV1::Created => ResultOutcomeV1::Succeeded,
        TaskResultKindV1::Rejected => ResultOutcomeV1::Rejected,
    };
    if command_message_id.as_slice() != command_id
        || envelope.causation_message_id.as_slice() != command_id
        || outcome != expected_outcome as i32
    {
        return Err(ReviewedTaskCandidatePromotionEventErrorV1::InvalidEnvelope);
    }
    let (candidate_id, logical_owner_id, outcome) = match kind {
        TaskResultKindV1::Created => {
            let payload =
                TaskCreatedFromReviewedCandidateV1::decode(envelope.payload.as_slice())
                    .map_err(|_| ReviewedTaskCandidatePromotionEventErrorV1::InvalidPayload)?;
            if id16(&payload.command_id)? != command_id || payload.task_revision == 0 {
                return Err(ReviewedTaskCandidatePromotionEventErrorV1::InvalidPayload);
            }
            (
                id16(&payload.approved_candidate_id)?,
                payload.logical_owner_id,
                DecodedTaskOutcomeV1::Succeeded {
                    task_id: id16(&payload.task_id)?,
                },
            )
        }
        TaskResultKindV1::Rejected => {
            let payload =
                TaskCreationFromReviewedCandidateRejectedV1::decode(envelope.payload.as_slice())
                    .map_err(|_| ReviewedTaskCandidatePromotionEventErrorV1::InvalidPayload)?;
            if id16(&payload.command_id)? != command_id {
                return Err(ReviewedTaskCandidatePromotionEventErrorV1::InvalidPayload);
            }
            let code = TaskCreationRejectCodeV1::try_from(payload.code)
                .ok()
                .filter(|value| {
                    *value != TaskCreationRejectCodeV1::TaskCreationRejectCodeUnspecified
                })
                .ok_or(ReviewedTaskCandidatePromotionEventErrorV1::InvalidPayload)?;
            (
                id16(&payload.approved_candidate_id)?,
                payload.logical_owner_id,
                DecodedTaskOutcomeV1::Failed { failure_code: code },
            )
        }
    };
    if envelope.partition_key.as_slice() != candidate_id
        || envelope.correlation_id.as_slice() != candidate_id
        || logical_owner_id != expected_owner
        || !valid_owner(&logical_owner_id)
    {
        return Err(ReviewedTaskCandidatePromotionEventErrorV1::InvalidPayload);
    }
    Ok(DecodedTaskResultV1 {
        command_id,
        candidate_id,
        outcome,
    })
}

fn promotion_failure_code(
    value: TaskCreationRejectCodeV1,
) -> Result<ReviewTaskCandidatePromotionFailureCodeV1, ReviewedTaskCandidatePromotionEventErrorV1> {
    ReviewTaskCandidatePromotionFailureCodeV1::try_from(value as i32)
        .map_err(|_| ReviewedTaskCandidatePromotionEventErrorV1::InvalidPayload)
}

fn promotion_context(
    runtime: &ReviewedTaskCandidatePromotionRuntimeContextV1<'_>,
) -> ReviewTaskCandidatePromotionEnvelopeContextV1 {
    ReviewTaskCandidatePromotionEnvelopeContextV1 {
        module_id: makosh_reviewed_task_candidate_promotion_core::REVIEWED_TASK_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.to_owned(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
            .unwrap_or_default(),
    }
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> ReviewedTaskCandidatePromotionEventErrorV1 {
    ReviewedTaskCandidatePromotionEventErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use makosh_tasks_command_api::{
        TasksCommandEnvelopeContextV1, build_task_created_from_reviewed_candidate_outbox_record_v1,
        build_task_creation_from_reviewed_candidate_rejected_outbox_record_v1,
        wire::{TaskCreatedFromReviewedCandidateV1, TaskCreationFromReviewedCandidateRejectedV1},
    };

    use super::*;

    fn context() -> TasksCommandEnvelopeContextV1 {
        TasksCommandEnvelopeContextV1 {
            module_id: "tasks-producer-v1".to_owned(),
            runtime_instance_id: "tasks-runtime-1".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 0,
        }
    }

    #[test]
    fn created_result_requires_exact_tasks_result_semantics() {
        let record = build_task_created_from_reviewed_candidate_outbox_record_v1(
            [1; 16],
            TaskCreatedFromReviewedCandidateV1 {
                command_id: vec![1; 16],
                approved_candidate_id: vec![2; 16],
                task_id: vec![3; 16],
                task_revision: 1,
                logical_owner_id: "owner-1".to_owned(),
            },
            &context(),
        )
        .expect("created result");
        let decoded = decode_result(&record, "owner-1", TaskResultKindV1::Created)
            .expect("decoded created result");
        assert_eq!(decoded.command_id, [1; 16]);
        assert_eq!(decoded.candidate_id, [2; 16]);
        assert!(
            matches!(decoded.outcome, DecodedTaskOutcomeV1::Succeeded { task_id } if task_id == [3; 16])
        );
    }

    #[test]
    fn rejected_result_maps_only_bounded_tasks_failure() {
        let record = build_task_creation_from_reviewed_candidate_rejected_outbox_record_v1(
            [4; 16],
            TaskCreationFromReviewedCandidateRejectedV1 {
                command_id: vec![4; 16],
                approved_candidate_id: vec![5; 16],
                code: TaskCreationRejectCodeV1::TaskCreationRejectCodePolicy as i32,
                logical_owner_id: "owner-1".to_owned(),
            },
            &context(),
        )
        .expect("rejected result");
        let decoded = decode_result(&record, "owner-1", TaskResultKindV1::Rejected)
            .expect("decoded rejected result");
        assert!(matches!(
            decoded.outcome,
            DecodedTaskOutcomeV1::Failed {
                failure_code: TaskCreationRejectCodeV1::TaskCreationRejectCodePolicy
            }
        ));
    }
}
