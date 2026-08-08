use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ResultMetadataV1, ResultOutcomeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_knowledge_command_api::{
    knowledge_note_created_from_reviewed_candidate_contract_reference_v1,
    knowledge_note_creation_from_reviewed_candidate_rejected_contract_reference_v1,
    wire::{
        KnowledgeNoteCreatedFromReviewedCandidateV1,
        KnowledgeNoteCreationFromReviewedCandidateRejectedV1, KnowledgeNoteCreationRejectCodeV1,
    },
};
use makosh_review_note_candidate_promotion_api::{
    ReviewNoteCandidatePromotionEnvelopeContextV1,
    build_review_note_candidate_promotion_result_outbox_record_v1,
    wire::{
        ReviewNoteCandidatePromotionFailureCodeV1, ReviewNoteCandidatePromotionOutcomeV1,
        ReviewNoteCandidatePromotionResultV1,
    },
};
use makosh_reviewed_note_candidate_promotion_core::derive_reviewed_note_candidate_result_id_v1;
use makosh_reviewed_note_candidate_promotion_persistence::{
    PersistPromotionTerminalResultV1, ReviewedNoteCandidatePromotionOutcomeV1 as StoredOutcome,
    ReviewedNoteCandidatePromotionPersistenceErrorV1, ReviewedNoteCandidatePromotionPersistenceV1,
};
use prost::Message;

use crate::validation::{id16, valid_owner, validate_contract};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReviewedNoteCandidatePromotionEventErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(ReviewedNoteCandidatePromotionPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) struct ReviewedNoteCandidatePromotionRuntimeContextV1<'a> {
    pub logical_human_owner_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub(crate) async fn consume_note_created_once_v1(
    persistence: &ReviewedNoteCandidatePromotionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &ReviewedNoteCandidatePromotionRuntimeContextV1<'_>,
) -> Result<bool, ReviewedNoteCandidatePromotionEventErrorV1> {
    consume_result_once(
        persistence,
        connection,
        permit,
        runtime,
        NoteResultKindV1::Created,
    )
    .await
}

pub(crate) async fn consume_note_rejected_once_v1(
    persistence: &ReviewedNoteCandidatePromotionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &ReviewedNoteCandidatePromotionRuntimeContextV1<'_>,
) -> Result<bool, ReviewedNoteCandidatePromotionEventErrorV1> {
    consume_result_once(
        persistence,
        connection,
        permit,
        runtime,
        NoteResultKindV1::Rejected,
    )
    .await
}

#[derive(Clone, Copy)]
enum NoteResultKindV1 {
    Created,
    Rejected,
}

async fn consume_result_once(
    persistence: &ReviewedNoteCandidatePromotionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &ReviewedNoteCandidatePromotionRuntimeContextV1<'_>,
    kind: NoteResultKindV1,
) -> Result<bool, ReviewedNoteCandidatePromotionEventErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidEnvelope)?;
    let result = decode_result(&record, runtime.logical_human_owner_id, kind)?;
    let correlation = persistence
        .load_correlation(runtime.logical_human_owner_id, &result.command_id)
        .await
        .map_err(ReviewedNoteCandidatePromotionEventErrorV1::Persistence)?;
    if correlation.candidate_id != result.candidate_id {
        return Err(ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload);
    }
    let result_id = derive_reviewed_note_candidate_result_id_v1(
        *record.message_id(),
        result.command_id,
        correlation.review_id,
    )
    .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)?;
    let (wire_outcome, note_id, failure_code, stored_outcome) = match result.outcome {
        DecodedNoteOutcomeV1::Succeeded { note_id } => (
            ReviewNoteCandidatePromotionOutcomeV1::ReviewNoteCandidatePromotionOutcomeSucceeded,
            Some(note_id.to_vec()),
            ReviewNoteCandidatePromotionFailureCodeV1::ReviewNoteCandidatePromotionFailureCodeUnspecified,
            StoredOutcome::Succeeded { note_id },
        ),
        DecodedNoteOutcomeV1::Failed { failure_code } => (
            ReviewNoteCandidatePromotionOutcomeV1::ReviewNoteCandidatePromotionOutcomeFailed,
            None,
            promotion_failure_code(failure_code)?,
            StoredOutcome::Failed {
                failure_code: failure_code as u16,
            },
        ),
    };
    let review_result = build_review_note_candidate_promotion_result_outbox_record_v1(
        *record.message_id(),
        ReviewNoteCandidatePromotionResultV1 {
            result_id: result_id.to_vec(),
            review_id: correlation.review_id.to_vec(),
            candidate_id: correlation.candidate_id.to_vec(),
            expected_review_revision: correlation.decision_revision,
            outcome: wire_outcome as i32,
            note_id,
            failure_code: failure_code as i32,
            logical_owner_id: runtime.logical_human_owner_id.to_owned(),
        },
        &promotion_context(runtime),
    )
    .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)?;
    persistence
        .persist_knowledge_result_and_review_result(&PersistPromotionTerminalResultV1 {
            logical_owner_id: runtime.logical_human_owner_id.to_owned(),
            knowledge_result_message_id: *record.message_id(),
            knowledge_result_envelope_sha256: *record.envelope_sha256(),
            knowledge_command_id: result.command_id,
            review_id: correlation.review_id,
            candidate_id: correlation.candidate_id,
            outcome: stored_outcome,
            review_result_outbox: review_result,
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(ReviewedNoteCandidatePromotionEventErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

struct DecodedNoteResultV1 {
    command_id: [u8; 16],
    candidate_id: [u8; 16],
    outcome: DecodedNoteOutcomeV1,
}

enum DecodedNoteOutcomeV1 {
    Succeeded {
        note_id: [u8; 16],
    },
    Failed {
        failure_code: KnowledgeNoteCreationRejectCodeV1,
    },
}

fn decode_result(
    record: &OutboxRecordV1,
    expected_owner: &str,
    kind: NoteResultKindV1,
) -> Result<DecodedNoteResultV1, ReviewedNoteCandidatePromotionEventErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidEnvelope)?;
    let expected_contract = match kind {
        NoteResultKindV1::Created => {
            knowledge_note_created_from_reviewed_candidate_contract_reference_v1()
        }
        NoteResultKindV1::Rejected => {
            knowledge_note_creation_from_reviewed_candidate_rejected_contract_reference_v1()
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
        return Err(ReviewedNoteCandidatePromotionEventErrorV1::InvalidEnvelope);
    };
    let command_id = id16(&command_id)?;
    let expected_outcome = match kind {
        NoteResultKindV1::Created => ResultOutcomeV1::Succeeded,
        NoteResultKindV1::Rejected => ResultOutcomeV1::Rejected,
    };
    if command_message_id.as_slice() != command_id
        || envelope.causation_message_id.as_slice() != command_id
        || outcome != expected_outcome as i32
    {
        return Err(ReviewedNoteCandidatePromotionEventErrorV1::InvalidEnvelope);
    }
    let (candidate_id, logical_owner_id, outcome) = match kind {
        NoteResultKindV1::Created => {
            let payload =
                KnowledgeNoteCreatedFromReviewedCandidateV1::decode(envelope.payload.as_slice())
                    .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)?;
            if id16(&payload.command_id)? != command_id || payload.note_revision == 0 {
                return Err(ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload);
            }
            (
                id16(&payload.approved_candidate_id)?,
                payload.logical_owner_id,
                DecodedNoteOutcomeV1::Succeeded {
                    note_id: id16(&payload.note_id)?,
                },
            )
        }
        NoteResultKindV1::Rejected => {
            let payload = KnowledgeNoteCreationFromReviewedCandidateRejectedV1::decode(
                envelope.payload.as_slice(),
            )
            .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)?;
            if id16(&payload.command_id)? != command_id {
                return Err(ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload);
            }
            let code = KnowledgeNoteCreationRejectCodeV1::try_from(payload.code)
                .ok()
                .filter(|value| {
                    *value != KnowledgeNoteCreationRejectCodeV1::KnowledgeNoteCreationRejectCodeUnspecified
                })
                .ok_or(ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)?;
            (
                id16(&payload.approved_candidate_id)?,
                payload.logical_owner_id,
                DecodedNoteOutcomeV1::Failed { failure_code: code },
            )
        }
    };
    if envelope.partition_key.as_slice() != candidate_id
        || envelope.correlation_id.as_slice() != candidate_id
        || logical_owner_id != expected_owner
        || !valid_owner(&logical_owner_id)
    {
        return Err(ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload);
    }
    Ok(DecodedNoteResultV1 {
        command_id,
        candidate_id,
        outcome,
    })
}

fn promotion_failure_code(
    value: KnowledgeNoteCreationRejectCodeV1,
) -> Result<ReviewNoteCandidatePromotionFailureCodeV1, ReviewedNoteCandidatePromotionEventErrorV1> {
    ReviewNoteCandidatePromotionFailureCodeV1::try_from(value as i32)
        .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)
}

fn promotion_context(
    runtime: &ReviewedNoteCandidatePromotionRuntimeContextV1<'_>,
) -> ReviewNoteCandidatePromotionEnvelopeContextV1 {
    ReviewNoteCandidatePromotionEnvelopeContextV1 {
        module_id: makosh_reviewed_note_candidate_promotion_core::REVIEWED_NOTE_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.to_owned(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
            .unwrap_or_default(),
    }
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> ReviewedNoteCandidatePromotionEventErrorV1 {
    ReviewedNoteCandidatePromotionEventErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use makosh_knowledge_command_api::{
        KnowledgeCommandEnvelopeContextV1,
        build_knowledge_note_created_from_reviewed_candidate_outbox_record_v1,
        build_knowledge_note_creation_from_reviewed_candidate_rejected_outbox_record_v1,
        wire::{
            KnowledgeNoteCreatedFromReviewedCandidateV1,
            KnowledgeNoteCreationFromReviewedCandidateRejectedV1,
        },
    };

    use super::*;

    fn context() -> KnowledgeCommandEnvelopeContextV1 {
        KnowledgeCommandEnvelopeContextV1 {
            module_id: "knowledge-producer-v1".to_owned(),
            runtime_instance_id: "knowledge-runtime-1".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 0,
        }
    }

    #[test]
    fn created_result_requires_exact_knowledge_result_semantics() {
        let record = build_knowledge_note_created_from_reviewed_candidate_outbox_record_v1(
            [1; 16],
            KnowledgeNoteCreatedFromReviewedCandidateV1 {
                command_id: vec![1; 16],
                approved_candidate_id: vec![2; 16],
                note_id: vec![3; 16],
                note_revision: 1,
                logical_owner_id: "owner-1".to_owned(),
            },
            &context(),
        )
        .expect("created result");
        let decoded = decode_result(&record, "owner-1", NoteResultKindV1::Created)
            .expect("decoded created result");
        assert_eq!(decoded.command_id, [1; 16]);
        assert_eq!(decoded.candidate_id, [2; 16]);
        assert!(
            matches!(decoded.outcome, DecodedNoteOutcomeV1::Succeeded { note_id } if note_id == [3; 16])
        );
    }

    #[test]
    fn rejected_result_maps_only_bounded_knowledge_failure() {
        let record =
            build_knowledge_note_creation_from_reviewed_candidate_rejected_outbox_record_v1(
                [4; 16],
                KnowledgeNoteCreationFromReviewedCandidateRejectedV1 {
                    command_id: vec![4; 16],
                    approved_candidate_id: vec![5; 16],
                    code: KnowledgeNoteCreationRejectCodeV1::KnowledgeNoteCreationRejectCodePolicy
                        as i32,
                    logical_owner_id: "owner-1".to_owned(),
                },
                &context(),
            )
            .expect("rejected result");
        let decoded = decode_result(&record, "owner-1", NoteResultKindV1::Rejected)
            .expect("decoded rejected result");
        assert!(matches!(
            decoded.outcome,
            DecodedNoteOutcomeV1::Failed {
                failure_code:
                    KnowledgeNoteCreationRejectCodeV1::KnowledgeNoteCreationRejectCodePolicy
            }
        ));
    }
}
