use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{EventMetadataV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_review_note_candidate_core::{
    ReviewNoteCandidatePromotionResultV1 as DomainPromotionResultV1, ReviewNoteCandidateTimestampV1,
};
use makosh_review_note_candidate_persistence::{
    PersistReviewNoteCandidatePromotionResultV1, ReviewNoteCandidatePersistenceErrorV1,
    ReviewNoteCandidatePersistenceV1,
};
use makosh_review_note_candidate_promotion_api::{
    review_note_candidate_promotion_result_contract_reference_v1,
    wire::{
        ReviewNoteCandidatePromotionFailureCodeV1, ReviewNoteCandidatePromotionOutcomeV1,
        ReviewNoteCandidatePromotionResultV1,
    },
};
use prost::Message;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReviewNoteCandidatePromotionResultErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(ReviewNoteCandidatePersistenceErrorV1),
    EventUnavailable,
}

pub(crate) async fn consume_review_note_candidate_promotion_result_once_v1(
    persistence: &ReviewNoteCandidatePersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    logical_owner_id: &str,
) -> Result<bool, ReviewNoteCandidatePromotionResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReviewNoteCandidatePromotionResultErrorV1::InvalidEnvelope)?;
    let decoded = decode_promotion_result(&record, logical_owner_id)?;
    persistence
        .persist_promotion_result(PersistReviewNoteCandidatePromotionResultV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            result_message_id: *record.message_id(),
            result_envelope_sha256: *record.envelope_sha256(),
            review_id: decoded.review_id,
            candidate_id: decoded.candidate_id,
            expected_review_revision: decoded.expected_review_revision,
            result: decoded.result,
            occurred_at: decoded.occurred_at,
        })
        .await
        .map_err(ReviewNoteCandidatePromotionResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DecodedPromotionResultV1 {
    review_id: [u8; 16],
    candidate_id: [u8; 16],
    expected_review_revision: u64,
    result: DomainPromotionResultV1,
    occurred_at: ReviewNoteCandidateTimestampV1,
}

fn decode_promotion_result(
    record: &OutboxRecordV1,
    expected_owner: &str,
) -> Result<DecodedPromotionResultV1, ReviewNoteCandidatePromotionResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ReviewNoteCandidatePromotionResultErrorV1::InvalidEnvelope)?;
    let expected_contract = review_note_candidate_promotion_result_contract_reference_v1();
    if envelope.contract.as_ref().is_none_or(|actual| {
        actual.owner != expected_contract.owner
            || actual.name != expected_contract.name
            || actual.major != expected_contract.major
            || actual.revision != expected_contract.revision
            || actual.schema_sha256 != expected_contract.schema_sha256
    }) {
        return Err(ReviewNoteCandidatePromotionResultErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Event(EventMetadataV1 {
        occurred_at: Some(occurred_at),
        ..
    })) = envelope.semantics
    else {
        return Err(ReviewNoteCandidatePromotionResultErrorV1::InvalidEnvelope);
    };
    if occurred_at.seconds <= 0 || !(0..1_000_000_000).contains(&occurred_at.nanos) {
        return Err(ReviewNoteCandidatePromotionResultErrorV1::InvalidEnvelope);
    }
    let payload = ReviewNoteCandidatePromotionResultV1::decode(envelope.payload.as_slice())
        .map_err(|_| ReviewNoteCandidatePromotionResultErrorV1::InvalidPayload)?;
    let result_id = id16(&payload.result_id)?;
    let review_id = id16(&payload.review_id)?;
    let candidate_id = id16(&payload.candidate_id)?;
    if result_id != *record.message_id()
        || envelope.partition_key.as_slice() != review_id
        || envelope.correlation_id.as_slice() != review_id
        || id16(&envelope.causation_message_id).is_err()
        || payload.expected_review_revision == 0
        || payload.logical_owner_id != expected_owner
        || !valid_owner(&payload.logical_owner_id)
    {
        return Err(ReviewNoteCandidatePromotionResultErrorV1::InvalidPayload);
    }
    let outcome = ReviewNoteCandidatePromotionOutcomeV1::try_from(payload.outcome)
        .map_err(|_| ReviewNoteCandidatePromotionResultErrorV1::InvalidPayload)?;
    let failure_code = ReviewNoteCandidatePromotionFailureCodeV1::try_from(payload.failure_code)
        .map_err(|_| ReviewNoteCandidatePromotionResultErrorV1::InvalidPayload)?;
    let result = match outcome {
        ReviewNoteCandidatePromotionOutcomeV1::ReviewNoteCandidatePromotionOutcomeSucceeded
            if failure_code
                == ReviewNoteCandidatePromotionFailureCodeV1::ReviewNoteCandidatePromotionFailureCodeUnspecified =>
        {
            DomainPromotionResultV1::Succeeded {
                note_id: id16(
                    payload
                        .note_id
                        .as_deref()
                        .ok_or(ReviewNoteCandidatePromotionResultErrorV1::InvalidPayload)?,
                )?,
            }
        }
        ReviewNoteCandidatePromotionOutcomeV1::ReviewNoteCandidatePromotionOutcomeFailed
            if payload.note_id.is_none()
                && failure_code
                    != ReviewNoteCandidatePromotionFailureCodeV1::ReviewNoteCandidatePromotionFailureCodeUnspecified =>
        {
            DomainPromotionResultV1::Failed
        }
        _ => return Err(ReviewNoteCandidatePromotionResultErrorV1::InvalidPayload),
    };
    Ok(DecodedPromotionResultV1 {
        review_id,
        candidate_id,
        expected_review_revision: payload.expected_review_revision,
        result,
        occurred_at: ReviewNoteCandidateTimestampV1 {
            unix_seconds: occurred_at.seconds,
            nanos: occurred_at.nanos,
        },
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], ReviewNoteCandidatePromotionResultErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewNoteCandidatePromotionResultErrorV1::InvalidPayload)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> ReviewNoteCandidatePromotionResultErrorV1 {
    ReviewNoteCandidatePromotionResultErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use makosh_review_note_candidate_promotion_api::{
        ReviewNoteCandidatePromotionEnvelopeContextV1,
        build_review_note_candidate_promotion_result_outbox_record_v1,
    };

    use super::*;

    fn record(outcome: ReviewNoteCandidatePromotionOutcomeV1) -> OutboxRecordV1 {
        let succeeded = outcome
            == ReviewNoteCandidatePromotionOutcomeV1::ReviewNoteCandidatePromotionOutcomeSucceeded;
        build_review_note_candidate_promotion_result_outbox_record_v1(
            [9; 16],
            ReviewNoteCandidatePromotionResultV1 {
                result_id: vec![1; 16],
                review_id: vec![2; 16],
                candidate_id: vec![3; 16],
                expected_review_revision: 2,
                outcome: outcome as i32,
                note_id: succeeded.then(|| vec![4; 16]),
                failure_code: if succeeded {
                    ReviewNoteCandidatePromotionFailureCodeV1::ReviewNoteCandidatePromotionFailureCodeUnspecified as i32
                } else {
                    ReviewNoteCandidatePromotionFailureCodeV1::ReviewNoteCandidatePromotionFailureCodePolicy as i32
                },
                logical_owner_id: "owner-1".to_owned(),
            },
            &ReviewNoteCandidatePromotionEnvelopeContextV1 {
                module_id: "makosh-reviewed-note-candidate-promotion-runtime".to_owned(),
                runtime_instance_id: "runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("promotion result")
    }

    #[test]
    fn succeeded_result_maps_to_review_transition() {
        let decoded = decode_promotion_result(
            &record(
                ReviewNoteCandidatePromotionOutcomeV1::ReviewNoteCandidatePromotionOutcomeSucceeded,
            ),
            "owner-1",
        )
        .expect("decoded");
        assert_eq!(decoded.review_id, [2; 16]);
        assert_eq!(decoded.candidate_id, [3; 16]);
        assert_eq!(
            decoded.result,
            DomainPromotionResultV1::Succeeded { note_id: [4; 16] }
        );
    }

    #[test]
    fn failed_result_maps_without_leaking_provider_detail() {
        let decoded = decode_promotion_result(
            &record(
                ReviewNoteCandidatePromotionOutcomeV1::ReviewNoteCandidatePromotionOutcomeFailed,
            ),
            "owner-1",
        )
        .expect("decoded");
        assert_eq!(decoded.result, DomainPromotionResultV1::Failed);
    }
}
