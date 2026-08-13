use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ResultMetadataV1, ResultOutcomeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_obligations_api::{
    obligation_created_from_reviewed_candidate_contract_reference_v1,
    obligation_creation_from_reviewed_candidate_rejected_contract_reference_v1,
    wire::{
        ObligationCreatedFromReviewedCandidateV1,
        ObligationCreationFromReviewedCandidateRejectedV1, ObligationCreationRejectCodeV1,
    },
};
use makosh_review_obligation_candidate_promotion_api::{
    ReviewObligationCandidatePromotionEnvelopeContextV1,
    build_review_obligation_candidate_promotion_result_outbox_record_v1,
    wire::{
        ReviewObligationCandidatePromotionFailureCodeV1,
        ReviewObligationCandidatePromotionOutcomeV1, ReviewObligationCandidatePromotionResultV1,
    },
};
use makosh_reviewed_obligation_candidate_promotion_core::derive_reviewed_obligation_candidate_result_id_v1;
use makosh_reviewed_obligation_candidate_promotion_persistence::{
    PersistPromotionTerminalResultV1,
    ReviewedObligationCandidatePromotionOutcomeV1 as StoredOutcome,
    ReviewedObligationCandidatePromotionPersistenceErrorV1,
    ReviewedObligationCandidatePromotionPersistenceV1,
};
use prost::Message;

use crate::validation::{id16, valid_owner, validate_contract};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReviewedObligationCandidatePromotionEventErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(ReviewedObligationCandidatePromotionPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) struct ReviewedObligationCandidatePromotionRuntimeContextV1<'a> {
    pub logical_human_owner_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub(crate) async fn consume_obligation_created_once_v1(
    persistence: &ReviewedObligationCandidatePromotionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &ReviewedObligationCandidatePromotionRuntimeContextV1<'_>,
) -> Result<bool, ReviewedObligationCandidatePromotionEventErrorV1> {
    consume_result_once(
        persistence,
        connection,
        permit,
        runtime,
        ObligationResultKindV1::Created,
    )
    .await
}

pub(crate) async fn consume_obligation_rejected_once_v1(
    persistence: &ReviewedObligationCandidatePromotionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &ReviewedObligationCandidatePromotionRuntimeContextV1<'_>,
) -> Result<bool, ReviewedObligationCandidatePromotionEventErrorV1> {
    consume_result_once(
        persistence,
        connection,
        permit,
        runtime,
        ObligationResultKindV1::Rejected,
    )
    .await
}

#[derive(Clone, Copy)]
enum ObligationResultKindV1 {
    Created,
    Rejected,
}

async fn consume_result_once(
    persistence: &ReviewedObligationCandidatePromotionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &ReviewedObligationCandidatePromotionRuntimeContextV1<'_>,
    kind: ObligationResultKindV1,
) -> Result<bool, ReviewedObligationCandidatePromotionEventErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReviewedObligationCandidatePromotionEventErrorV1::InvalidEnvelope)?;
    let result = decode_result(&record, runtime.logical_human_owner_id, kind)?;
    let correlation = persistence
        .load_correlation(runtime.logical_human_owner_id, &result.command_id)
        .await
        .map_err(ReviewedObligationCandidatePromotionEventErrorV1::Persistence)?;
    if correlation.candidate_id != result.candidate_id {
        return Err(ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload);
    }
    let result_id = derive_reviewed_obligation_candidate_result_id_v1(
        *record.message_id(),
        result.command_id,
        correlation.review_id,
    )
    .map_err(|_| ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload)?;
    let (wire_outcome, obligation_id, failure_code, stored_outcome) = match result.outcome {
        DecodedObligationOutcomeV1::Succeeded { obligation_id } => (
            ReviewObligationCandidatePromotionOutcomeV1::ReviewObligationCandidatePromotionOutcomeSucceeded,
            Some(obligation_id.to_vec()),
            ReviewObligationCandidatePromotionFailureCodeV1::ReviewObligationCandidatePromotionFailureCodeUnspecified,
            StoredOutcome::Succeeded { obligation_id },
        ),
        DecodedObligationOutcomeV1::Failed { failure_code } => (
            ReviewObligationCandidatePromotionOutcomeV1::ReviewObligationCandidatePromotionOutcomeFailed,
            None,
            promotion_failure_code(failure_code)?,
            StoredOutcome::Failed {
                failure_code: failure_code as u16,
            },
        ),
    };
    let review_result = build_review_obligation_candidate_promotion_result_outbox_record_v1(
        *record.message_id(),
        ReviewObligationCandidatePromotionResultV1 {
            result_id: result_id.to_vec(),
            review_id: correlation.review_id.to_vec(),
            candidate_id: correlation.candidate_id.to_vec(),
            expected_review_revision: correlation.decision_revision,
            outcome: wire_outcome as i32,
            obligation_id,
            failure_code: failure_code as i32,
            logical_owner_id: runtime.logical_human_owner_id.to_owned(),
        },
        &promotion_context(runtime),
    )
    .map_err(|_| ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload)?;
    persistence
        .persist_obligations_result_and_review_result(&PersistPromotionTerminalResultV1 {
            logical_owner_id: runtime.logical_human_owner_id.to_owned(),
            obligations_result_message_id: *record.message_id(),
            obligations_result_envelope_sha256: *record.envelope_sha256(),
            obligations_id: result.command_id,
            review_id: correlation.review_id,
            candidate_id: correlation.candidate_id,
            outcome: stored_outcome,
            review_result_outbox: review_result,
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(ReviewedObligationCandidatePromotionEventErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

struct DecodedObligationResultV1 {
    command_id: [u8; 16],
    candidate_id: [u8; 16],
    outcome: DecodedObligationOutcomeV1,
}

enum DecodedObligationOutcomeV1 {
    Succeeded {
        obligation_id: [u8; 16],
    },
    Failed {
        failure_code: ObligationCreationRejectCodeV1,
    },
}

fn decode_result(
    record: &OutboxRecordV1,
    expected_owner: &str,
    kind: ObligationResultKindV1,
) -> Result<DecodedObligationResultV1, ReviewedObligationCandidatePromotionEventErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ReviewedObligationCandidatePromotionEventErrorV1::InvalidEnvelope)?;
    let expected_contract = match kind {
        ObligationResultKindV1::Created => {
            obligation_created_from_reviewed_candidate_contract_reference_v1()
        }
        ObligationResultKindV1::Rejected => {
            obligation_creation_from_reviewed_candidate_rejected_contract_reference_v1()
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
        return Err(ReviewedObligationCandidatePromotionEventErrorV1::InvalidEnvelope);
    };
    let command_id = id16(&command_id)?;
    let expected_outcome = match kind {
        ObligationResultKindV1::Created => ResultOutcomeV1::Succeeded,
        ObligationResultKindV1::Rejected => ResultOutcomeV1::Rejected,
    };
    if command_message_id.as_slice() != command_id
        || envelope.causation_message_id.as_slice() != command_id
        || outcome != expected_outcome as i32
    {
        return Err(ReviewedObligationCandidatePromotionEventErrorV1::InvalidEnvelope);
    }
    let (candidate_id, logical_owner_id, outcome) = match kind {
        ObligationResultKindV1::Created => {
            let payload =
                ObligationCreatedFromReviewedCandidateV1::decode(envelope.payload.as_slice())
                    .map_err(|_| {
                        ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload
                    })?;
            if id16(&payload.command_id)? != command_id || payload.obligation_revision == 0 {
                return Err(ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload);
            }
            (
                id16(&payload.approved_candidate_id)?,
                payload.logical_owner_id,
                DecodedObligationOutcomeV1::Succeeded {
                    obligation_id: id16(&payload.obligation_id)?,
                },
            )
        }
        ObligationResultKindV1::Rejected => {
            let payload = ObligationCreationFromReviewedCandidateRejectedV1::decode(
                envelope.payload.as_slice(),
            )
            .map_err(|_| ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload)?;
            if id16(&payload.command_id)? != command_id {
                return Err(ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload);
            }
            let code = ObligationCreationRejectCodeV1::try_from(payload.code)
                .ok()
                .filter(|value| {
                    *value
                        != ObligationCreationRejectCodeV1::ObligationCreationRejectCodeUnspecified
                })
                .ok_or(ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload)?;
            (
                id16(&payload.approved_candidate_id)?,
                payload.logical_owner_id,
                DecodedObligationOutcomeV1::Failed { failure_code: code },
            )
        }
    };
    if envelope.partition_key.as_slice() != candidate_id
        || envelope.correlation_id.as_slice() != candidate_id
        || logical_owner_id != expected_owner
        || !valid_owner(&logical_owner_id)
    {
        return Err(ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload);
    }
    Ok(DecodedObligationResultV1 {
        command_id,
        candidate_id,
        outcome,
    })
}

fn promotion_failure_code(
    value: ObligationCreationRejectCodeV1,
) -> Result<
    ReviewObligationCandidatePromotionFailureCodeV1,
    ReviewedObligationCandidatePromotionEventErrorV1,
> {
    ReviewObligationCandidatePromotionFailureCodeV1::try_from(value as i32)
        .map_err(|_| ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload)
}

fn promotion_context(
    runtime: &ReviewedObligationCandidatePromotionRuntimeContextV1<'_>,
) -> ReviewObligationCandidatePromotionEnvelopeContextV1 {
    ReviewObligationCandidatePromotionEnvelopeContextV1 {
        module_id: makosh_reviewed_obligation_candidate_promotion_core::REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.to_owned(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
            .unwrap_or_default(),
    }
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> ReviewedObligationCandidatePromotionEventErrorV1 {
    ReviewedObligationCandidatePromotionEventErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use makosh_obligations_api::{
        ObligationsCommandEnvelopeContextV1,
        build_obligation_created_from_reviewed_candidate_outbox_record_v1,
        build_obligation_creation_from_reviewed_candidate_rejected_outbox_record_v1,
        wire::{
            ObligationCreatedFromReviewedCandidateV1,
            ObligationCreationFromReviewedCandidateRejectedV1,
        },
    };

    use super::*;

    fn context() -> ObligationsCommandEnvelopeContextV1 {
        ObligationsCommandEnvelopeContextV1 {
            module_id: "obligations-producer-v1".to_owned(),
            runtime_instance_id: "obligations-runtime-1".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 0,
        }
    }

    #[test]
    fn created_result_requires_exact_obligations_result_semantics() {
        let record = build_obligation_created_from_reviewed_candidate_outbox_record_v1(
            [1; 16],
            ObligationCreatedFromReviewedCandidateV1 {
                command_id: vec![1; 16],
                approved_candidate_id: vec![2; 16],
                obligation_id: vec![3; 16],
                obligation_revision: 1,
                logical_owner_id: "owner-1".to_owned(),
            },
            &context(),
        )
        .expect("created result");
        let decoded = decode_result(&record, "owner-1", ObligationResultKindV1::Created)
            .expect("decoded created result");
        assert_eq!(decoded.command_id, [1; 16]);
        assert_eq!(decoded.candidate_id, [2; 16]);
        assert!(
            matches!(decoded.outcome, DecodedObligationOutcomeV1::Succeeded { obligation_id } if obligation_id == [3; 16])
        );
    }

    #[test]
    fn rejected_result_maps_only_bounded_obligations_failure() {
        let record = build_obligation_creation_from_reviewed_candidate_rejected_outbox_record_v1(
            [4; 16],
            ObligationCreationFromReviewedCandidateRejectedV1 {
                command_id: vec![4; 16],
                approved_candidate_id: vec![5; 16],
                code: ObligationCreationRejectCodeV1::ObligationCreationRejectCodePolicy as i32,
                logical_owner_id: "owner-1".to_owned(),
            },
            &context(),
        )
        .expect("rejected result");
        let decoded = decode_result(&record, "owner-1", ObligationResultKindV1::Rejected)
            .expect("decoded rejected result");
        assert!(matches!(
            decoded.outcome,
            DecodedObligationOutcomeV1::Failed {
                failure_code: ObligationCreationRejectCodeV1::ObligationCreationRejectCodePolicy
            }
        ));
    }
}
