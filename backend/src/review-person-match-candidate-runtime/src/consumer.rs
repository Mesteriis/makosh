use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        ActorKindV1, DurableEnvelopeV1, FenceKindV1, ResultOutcomeV1,
        durable_envelope_v1::Semantics,
    },
};
use makosh_persons_api::persons_confirmed_action_command_id_v1;
use makosh_review_person_match_candidate_persistence::{
    PersistPersonMatchCandidatePromotionResultV1, ReviewPersonMatchCandidateEnvelopeRecordV1,
    ReviewPersonMatchCandidatePersistenceErrorV1, ReviewPersonMatchCandidatePersistenceV1,
};
use makosh_review_person_match_candidate_promotion_api::{
    ReviewPersonMatchCandidatePromotionResultShapeV1,
    review_person_match_candidate_promotion_result_contract_reference_v1,
    review_person_match_candidate_promotion_result_id_v1,
    wire::{
        ReviewPersonMatchCandidatePromotionFailureCodeV1,
        ReviewPersonMatchCandidatePromotionOutcomeV1, ReviewPersonMatchCandidatePromotionResultV1,
    },
};
use prost::Message;

use crate::{
    ReviewPersonMatchCandidateExecutionContextV1, ReviewPersonMatchCandidateExecutionErrorV1,
    process_person_match_candidate_decision_v1, process_persons_review_candidate_v1,
};

pub async fn consume_persons_review_candidate_once_v1(
    persistence: &ReviewPersonMatchCandidatePersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &ReviewPersonMatchCandidateExecutionContextV1,
) -> Result<bool, ReviewPersonMatchCandidateExecutionErrorV1> {
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?
    else {
        return Ok(false);
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    match process_persons_review_candidate_v1(persistence, &record, context).await {
        Ok(_) => {}
        Err(error) if bounded_submission_rejection(error) => {
            delivery.acknowledge().await.map_err(event_error)?;
            return Ok(true);
        }
        Err(error) => return Err(error),
    }
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

pub async fn consume_person_match_candidate_decision_once_v1(
    persistence: &ReviewPersonMatchCandidatePersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &ReviewPersonMatchCandidateExecutionContextV1,
) -> Result<bool, ReviewPersonMatchCandidateExecutionErrorV1> {
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?
    else {
        return Ok(false);
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    match process_person_match_candidate_decision_v1(persistence, &record, context).await {
        Ok(_) => {}
        Err(error) if bounded_decision_rejection(error) => {
            delivery.acknowledge().await.map_err(event_error)?;
            return Ok(true);
        }
        Err(error) => return Err(error),
    }
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

const fn bounded_decision_rejection(error: ReviewPersonMatchCandidateExecutionErrorV1) -> bool {
    matches!(
        error,
        ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope
            | ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload
            | ReviewPersonMatchCandidateExecutionErrorV1::Persistence(
                ReviewPersonMatchCandidatePersistenceErrorV1::InvalidInput
                    | ReviewPersonMatchCandidatePersistenceErrorV1::Conflict
                    | ReviewPersonMatchCandidatePersistenceErrorV1::RevisionConflict
                    | ReviewPersonMatchCandidatePersistenceErrorV1::NotFound
            )
    )
}

const fn bounded_submission_rejection(error: ReviewPersonMatchCandidateExecutionErrorV1) -> bool {
    matches!(
        error,
        ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope
            | ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload
            | ReviewPersonMatchCandidateExecutionErrorV1::Persistence(
                ReviewPersonMatchCandidatePersistenceErrorV1::InvalidInput
                    | ReviewPersonMatchCandidatePersistenceErrorV1::Conflict
                    | ReviewPersonMatchCandidatePersistenceErrorV1::RevisionConflict
            )
    )
}

pub async fn consume_person_match_candidate_promotion_result_once_v1(
    persistence: &ReviewPersonMatchCandidatePersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &ReviewPersonMatchCandidateExecutionContextV1,
) -> Result<bool, ReviewPersonMatchCandidateExecutionErrorV1> {
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?
    else {
        return Ok(false);
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let envelope: DurableEnvelopeV1 = decode_exact(
        record.exact_bytes(),
        ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope,
    )?;
    let payload: ReviewPersonMatchCandidatePromotionResultV1 = decode_exact(
        &envelope.payload,
        ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload,
    )?;
    let expected = review_person_match_candidate_promotion_result_contract_reference_v1();
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let result = match envelope.semantics.as_ref() {
        Some(Semantics::Result(value)) => value,
        _ => return Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope),
    };
    let source = envelope
        .source
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let recorded = envelope
        .recorded_at
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let completed = result
        .completed_at
        .as_ref()
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)?;
    let result_id = id16(&payload.result_id)?;
    let review_id = id16(&payload.review_id)?;
    let candidate_id = id16(&payload.candidate_id)?;
    let decision_id = id16(&payload.decision_id)?;
    let (succeeded, persons_command_id) = classify_promotion_result_shape(&payload)?;
    let causation_message_id = id16(&envelope.causation_message_id)?;
    validate_promotion_result_id(
        causation_message_id,
        decision_id,
        if persons_command_id.is_some() {
            ReviewPersonMatchCandidatePromotionResultShapeV1::PersonsTerminal
        } else {
            ReviewPersonMatchCandidatePromotionResultShapeV1::ActionDigestMismatch
        },
        result_id,
    )?;
    let expected_outcome = if succeeded {
        ResultOutcomeV1::Succeeded
    } else {
        ResultOutcomeV1::Rejected
    };
    if contract.owner != expected.owner
        || contract.name != expected.name
        || contract.major != expected.major
        || contract.revision != expected.revision
        || contract.schema_sha256 != expected.schema_sha256
        || result_id != *record.message_id()
        || result.command_id != decision_id
        || result.command_message_id != payload.decision_id
        || result.outcome != expected_outcome as i32
        || result.execution_attempt != 1
        || source.module_id != "makosh-reviewed-person-match-candidate-promotion-runtime"
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id != source.module_id.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != source.module_id.as_bytes()
        || fence.epoch != source.runtime_generation
        || envelope.partition_key != payload.review_id
        || envelope.correlation_id != payload.review_id
        || payload.logical_owner_id != context.logical_owner_id
        || payload.expected_review_revision == 0
        || payload.completed_at_unix_millis <= 0
        || payload.completed_at_unix_millis > context.now_unix_millis
        || timestamp_millis(recorded.seconds, recorded.nanos)? != payload.completed_at_unix_millis
        || timestamp_millis(completed.seconds, completed.nanos)? != payload.completed_at_unix_millis
    {
        return Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope);
    }
    let current = match persistence
        .load_review(&context.logical_owner_id, review_id)
        .await
    {
        Ok(current) => current,
        Err(ReviewPersonMatchCandidatePersistenceErrorV1::NotFound) => {
            delivery.acknowledge().await.map_err(event_error)?;
            return Ok(true);
        }
        Err(error) => {
            return Err(ReviewPersonMatchCandidateExecutionErrorV1::Persistence(
                error,
            ));
        }
    };
    if let Some(persons_command_id) = persons_command_id {
        let approved_action_digest = current
            .approved_action_digest
            .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)?;
        let expected_persons_command_id =
            persons_confirmed_action_command_id_v1(decision_id, approved_action_digest)
                .map_err(|_| ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)?;
        if persons_command_id != expected_persons_command_id {
            return Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload);
        }
    }
    persistence
        .persist_promotion_result_once(&PersistPersonMatchCandidatePromotionResultV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            result: ReviewPersonMatchCandidateEnvelopeRecordV1 {
                message_id: *record.message_id(),
                envelope_sha256: *record.envelope_sha256(),
                envelope_bytes: record.exact_bytes().to_vec(),
            },
            review_id,
            candidate_id,
            decision_id,
            persons_command_id,
            expected_review_revision: payload.expected_review_revision,
            succeeded,
            completed_at_unix_millis: payload.completed_at_unix_millis,
        })
        .await
        .map_err(ReviewPersonMatchCandidateExecutionErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

fn id16(value: &[u8]) -> Result<[u8; 16], ReviewPersonMatchCandidateExecutionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)
}

fn classify_promotion_result_shape(
    payload: &ReviewPersonMatchCandidatePromotionResultV1,
) -> Result<(bool, Option<[u8; 16]>), ReviewPersonMatchCandidateExecutionErrorV1> {
    let persons_command_id = payload
        .persons_command_id
        .as_deref()
        .map(id16)
        .transpose()?;
    let outcome = ReviewPersonMatchCandidatePromotionOutcomeV1::try_from(payload.outcome)
        .map_err(|_| ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)?;
    let failure = ReviewPersonMatchCandidatePromotionFailureCodeV1::try_from(payload.failure_code)
        .map_err(|_| ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)?;
    let succeeded = match (outcome, failure, persons_command_id) {
        (
            ReviewPersonMatchCandidatePromotionOutcomeV1::ReviewPersonMatchCandidatePromotionOutcomeSucceeded,
            ReviewPersonMatchCandidatePromotionFailureCodeV1::ReviewPersonMatchCandidatePromotionFailureCodeUnspecified,
            Some(_),
        ) => true,
        (
            ReviewPersonMatchCandidatePromotionOutcomeV1::ReviewPersonMatchCandidatePromotionOutcomeFailed,
            ReviewPersonMatchCandidatePromotionFailureCodeV1::ReviewPersonMatchCandidatePromotionFailureCodePersonsRejected,
            Some(_),
        )
        | (
            ReviewPersonMatchCandidatePromotionOutcomeV1::ReviewPersonMatchCandidatePromotionOutcomeFailed,
            ReviewPersonMatchCandidatePromotionFailureCodeV1::ReviewPersonMatchCandidatePromotionFailureCodeActionDigestMismatch,
            None,
        ) => false,
        _ => return Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload),
    };
    Ok((succeeded, persons_command_id))
}

fn validate_promotion_result_id(
    causation_message_id: [u8; 16],
    decision_id: [u8; 16],
    shape: ReviewPersonMatchCandidatePromotionResultShapeV1,
    result_id: [u8; 16],
) -> Result<(), ReviewPersonMatchCandidateExecutionErrorV1> {
    let expected = review_person_match_candidate_promotion_result_id_v1(
        causation_message_id,
        decision_id,
        shape,
    )
    .map_err(|_| ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)?;
    if result_id == expected {
        Ok(())
    } else {
        Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)
    }
}

fn decode_exact<T: Message + Default>(
    bytes: &[u8],
    error: ReviewPersonMatchCandidateExecutionErrorV1,
) -> Result<T, ReviewPersonMatchCandidateExecutionErrorV1> {
    let value = T::decode(bytes).map_err(|_| error)?;
    if value.encode_to_vec() != bytes {
        return Err(error);
    }
    Ok(value)
}

fn timestamp_millis(
    seconds: i64,
    nanos: i32,
) -> Result<i64, ReviewPersonMatchCandidateExecutionErrorV1> {
    if seconds <= 0 || !(0..1_000_000_000).contains(&nanos) || nanos % 1_000_000 != 0 {
        return Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload);
    }
    seconds
        .checked_mul(1_000)
        .and_then(|value| value.checked_add(i64::from(nanos / 1_000_000)))
        .ok_or(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)
}

const fn event_error(_: RuntimePullDeliveryErrorV1) -> ReviewPersonMatchCandidateExecutionErrorV1 {
    ReviewPersonMatchCandidateExecutionErrorV1::EventUnavailable
}

#[cfg(test)]
mod bounded_decision_tests {
    use super::*;

    #[test]
    fn stale_terminal_and_invalid_decisions_are_bounded_but_storage_faults_are_not() {
        for error in [
            ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload,
            ReviewPersonMatchCandidateExecutionErrorV1::Persistence(
                ReviewPersonMatchCandidatePersistenceErrorV1::RevisionConflict,
            ),
            ReviewPersonMatchCandidateExecutionErrorV1::Persistence(
                ReviewPersonMatchCandidatePersistenceErrorV1::InvalidInput,
            ),
        ] {
            assert!(bounded_decision_rejection(error));
        }
        assert!(!bounded_decision_rejection(
            ReviewPersonMatchCandidateExecutionErrorV1::Persistence(
                ReviewPersonMatchCandidatePersistenceErrorV1::StorageUnavailable,
            )
        ));
    }

    #[test]
    fn promotion_result_shapes_bind_failure_code_to_persons_command_presence() {
        let mut payload = ReviewPersonMatchCandidatePromotionResultV1 {
            outcome: ReviewPersonMatchCandidatePromotionOutcomeV1::ReviewPersonMatchCandidatePromotionOutcomeFailed as i32,
            failure_code: ReviewPersonMatchCandidatePromotionFailureCodeV1::ReviewPersonMatchCandidatePromotionFailureCodeActionDigestMismatch as i32,
            persons_command_id: None,
            ..Default::default()
        };
        assert_eq!(classify_promotion_result_shape(&payload), Ok((false, None)));
        payload.persons_command_id = Some(vec![1; 16]);
        assert_eq!(
            classify_promotion_result_shape(&payload),
            Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)
        );
        payload.failure_code = ReviewPersonMatchCandidatePromotionFailureCodeV1::ReviewPersonMatchCandidatePromotionFailureCodePersonsRejected as i32;
        payload.persons_command_id = None;
        assert_eq!(
            classify_promotion_result_shape(&payload),
            Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidPayload)
        );
    }

    #[test]
    fn promotion_result_id_rejects_mutated_causation_and_message_id() {
        let causation = [1; 16];
        let decision = [2; 16];
        let shape = ReviewPersonMatchCandidatePromotionResultShapeV1::PersonsTerminal;
        let result_id =
            review_person_match_candidate_promotion_result_id_v1(causation, decision, shape)
                .expect("canonical result ID");
        assert_eq!(
            validate_promotion_result_id(causation, decision, shape, result_id),
            Ok(())
        );
        assert_eq!(
            validate_promotion_result_id([3; 16], decision, shape, result_id),
            Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)
        );
        assert_eq!(
            validate_promotion_result_id(causation, decision, shape, [4; 16]),
            Err(ReviewPersonMatchCandidateExecutionErrorV1::InvalidEnvelope)
        );
    }
}
