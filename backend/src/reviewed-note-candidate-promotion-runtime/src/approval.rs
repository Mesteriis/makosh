use std::os::unix::net::UnixStream;

use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{EventMetadataV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_knowledge_command_api::{
    KnowledgeCommandEnvelopeContextV1,
    build_create_knowledge_note_from_reviewed_candidate_outbox_record_v1,
    wire::CreateKnowledgeNoteFromReviewedCandidateCommandV1,
};
use makosh_review_note_candidate_api::{
    review_note_candidate_approved_contract_reference_v1, wire::NoteCandidateApprovedForPromotionV1,
};
use makosh_review_note_candidate_promotion_api::{
    ReviewNoteCandidatePromotionEnvelopeContextV1,
    build_review_note_candidate_promotion_result_outbox_record_v1,
    wire::{
        ReviewNoteCandidatePromotionFailureCodeV1, ReviewNoteCandidatePromotionOutcomeV1,
        ReviewNoteCandidatePromotionResultV1,
    },
};
use makosh_reviewed_note_candidate_promotion_core::{
    derive_reviewed_note_candidate_command_id_v1, derive_reviewed_note_candidate_result_id_v1,
};
use makosh_reviewed_note_candidate_promotion_persistence::{
    PersistPromotionApprovalV1, PersistPromotionMaterializationV1,
    PersistPromotionWorkflowFailureV1, PromotionBlobReceiptV1, ReservePromotionApprovalOutcomeV1,
    ReservePromotionApprovalV1, ReviewedNoteCandidatePromotionPersistenceV1,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use prost::Message;

use crate::{
    blob_handoff::{
        PromotionBlobHandoffErrorV1, build_knowledge_receipt_v1, release_source_v1,
        transfer_source_v1,
    },
    note_results::{
        ReviewedNoteCandidatePromotionEventErrorV1, ReviewedNoteCandidatePromotionRuntimeContextV1,
    },
    validation::{id16, id32, valid_owner, validate_contract},
};

const KNOWLEDGE_COMMAND_DEADLINE_SECONDS_V1: i64 = 300;

pub(crate) async fn consume_approval_once_v1(
    persistence: &ReviewedNoteCandidatePromotionPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &ReviewedNoteCandidatePromotionRuntimeContextV1<'_>,
) -> Result<bool, ReviewedNoteCandidatePromotionEventErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidEnvelope)?;
    let approval = decode_approval(&record, runtime.logical_human_owner_id)?;
    let command_id = derive_reviewed_note_candidate_command_id_v1(
        *record.message_id(),
        approval.review_id,
        approval.candidate_id,
        approval.payload.decision_revision,
    )
    .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)?;
    let receipt = approval
        .payload
        .candidate_content
        .as_ref()
        .ok_or(ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)?;
    let source_blob = PromotionBlobReceiptV1 {
        reference_id: id16(&receipt.reference_id)?,
        declared_bytes: receipt.declared_bytes,
        sha256: id32(&receipt.sha256)?,
        custody_proof: receipt.custody_transfer_source_proof.clone(),
    };
    let reservation = persistence
        .reserve_approval(&ReservePromotionApprovalV1 {
            logical_owner_id: runtime.logical_human_owner_id.to_owned(),
            approval_message_id: *record.message_id(),
            approval_envelope_sha256: *record.envelope_sha256(),
            review_id: approval.review_id,
            candidate_id: approval.candidate_id,
            decision_revision: approval.payload.decision_revision,
            source_blob,
            knowledge_command_id: command_id,
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(ReviewedNoteCandidatePromotionEventErrorV1::Persistence)?;
    let mut persisted = match reservation {
        ReservePromotionApprovalOutcomeV1::Reserved(value)
        | ReservePromotionApprovalOutcomeV1::Existing(value) => value,
    };
    if persisted.command_completed || persisted.workflow_failure_result_id.is_some() {
        complete_cleanup_if_required(
            persistence,
            channel,
            dispatcher,
            &persisted,
            runtime.now_unix_millis,
        )
        .await?;
        delivery.acknowledge().await.map_err(event_error)?;
        return Ok(true);
    }
    let materialized_reference_id = if let Some(reference_id) = persisted.materialized_reference_id
    {
        reference_id
    } else {
        let reference_id = transfer_source_v1(
            channel,
            dispatcher,
            persisted.approval_message_id,
            persisted.approval_envelope_sha256,
            &persisted.source_blob,
        )
        .map_err(blob_error)?;
        persisted = persistence
            .persist_materialization(&PersistPromotionMaterializationV1 {
                logical_owner_id: persisted.logical_owner_id.clone(),
                approval_message_id: persisted.approval_message_id,
                materialized_reference_id: reference_id,
                materialized_at_unix_millis: runtime.now_unix_millis,
            })
            .await
            .map_err(ReviewedNoteCandidatePromotionEventErrorV1::Persistence)?;
        reference_id
    };
    let knowledge_receipt = match build_knowledge_receipt_v1(
        channel,
        dispatcher,
        command_id,
        &persisted.source_blob,
        materialized_reference_id,
    ) {
        Ok(receipt) => receipt,
        Err(PromotionBlobHandoffErrorV1::Unavailable) => {
            return Err(ReviewedNoteCandidatePromotionEventErrorV1::EventUnavailable);
        }
        Err(PromotionBlobHandoffErrorV1::InvalidReceipt) => {
            persist_invalid_source_result(persistence, &persisted, runtime, command_id).await?;
            complete_cleanup_if_required(
                persistence,
                channel,
                dispatcher,
                &persisted,
                runtime.now_unix_millis,
            )
            .await?;
            delivery.acknowledge().await.map_err(event_error)?;
            return Ok(true);
        }
    };
    let command = build_create_knowledge_note_from_reviewed_candidate_outbox_record_v1(
        CreateKnowledgeNoteFromReviewedCandidateCommandV1 {
            command_id: command_id.to_vec(),
            approved_candidate_id: approval.candidate_id.to_vec(),
            candidate_digest: approval.candidate_digest.to_vec(),
            source_evidence_id: approval.source_evidence_id.to_vec(),
            source_evidence_revision: approval.payload.source_evidence_revision,
            review_id: approval.review_id.to_vec(),
            decision_revision: approval.payload.decision_revision,
            decided_by_owner_device_id: approval.decided_by_owner_device_id.to_vec(),
            candidate_content: Some(knowledge_receipt),
            logical_owner_id: runtime.logical_human_owner_id.to_owned(),
        },
        runtime
            .now_unix_millis
            .checked_div(1_000)
            .and_then(|value| value.checked_add(KNOWLEDGE_COMMAND_DEADLINE_SECONDS_V1))
            .ok_or(ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)?,
        &knowledge_context(runtime),
    )
    .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)?;
    persistence
        .persist_approval_and_knowledge_command(&PersistPromotionApprovalV1 {
            logical_owner_id: runtime.logical_human_owner_id.to_owned(),
            approval_message_id: *record.message_id(),
            approval_envelope_sha256: *record.envelope_sha256(),
            review_id: approval.review_id,
            candidate_id: approval.candidate_id,
            decision_revision: approval.payload.decision_revision,
            knowledge_command_id: command_id,
            knowledge_command_outbox: command,
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(ReviewedNoteCandidatePromotionEventErrorV1::Persistence)?;
    release_source_v1(
        channel,
        dispatcher,
        persisted.approval_message_id,
        &persisted.source_blob,
        materialized_reference_id,
    )
    .map_err(blob_error)?;
    persistence
        .complete_source_cleanup(
            &persisted.logical_owner_id,
            &persisted.approval_message_id,
            &materialized_reference_id,
            runtime.now_unix_millis,
        )
        .await
        .map_err(ReviewedNoteCandidatePromotionEventErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

async fn persist_invalid_source_result(
    persistence: &ReviewedNoteCandidatePromotionPersistenceV1,
    persisted: &makosh_reviewed_note_candidate_promotion_persistence::PersistedPromotionApprovalV1,
    runtime: &ReviewedNoteCandidatePromotionRuntimeContextV1<'_>,
    command_id: [u8; 16],
) -> Result<(), ReviewedNoteCandidatePromotionEventErrorV1> {
    let result_id = derive_reviewed_note_candidate_result_id_v1(
        persisted.approval_message_id,
        command_id,
        persisted.review_id,
    )
    .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)?;
    let result = build_review_note_candidate_promotion_result_outbox_record_v1(
        persisted.approval_message_id,
        ReviewNoteCandidatePromotionResultV1 {
            result_id: result_id.to_vec(),
            review_id: persisted.review_id.to_vec(),
            candidate_id: persisted.candidate_id.to_vec(),
            expected_review_revision: persisted.decision_revision,
            outcome: ReviewNoteCandidatePromotionOutcomeV1::ReviewNoteCandidatePromotionOutcomeFailed
                as i32,
            note_id: None,
            failure_code: ReviewNoteCandidatePromotionFailureCodeV1::ReviewNoteCandidatePromotionFailureCodeInvalidRequest
                as i32,
            logical_owner_id: persisted.logical_owner_id.clone(),
        },
        &ReviewNoteCandidatePromotionEnvelopeContextV1 {
            module_id: makosh_reviewed_note_candidate_promotion_core::REVIEWED_NOTE_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
            runtime_instance_id: runtime.runtime_instance_id.to_owned(),
            runtime_generation: runtime.runtime_generation,
            recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
            recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
                .unwrap_or_default(),
        },
    )
    .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)?;
    persistence
        .persist_workflow_failure(&PersistPromotionWorkflowFailureV1 {
            logical_owner_id: persisted.logical_owner_id.clone(),
            approval_message_id: persisted.approval_message_id,
            review_id: persisted.review_id,
            candidate_id: persisted.candidate_id,
            knowledge_command_id: command_id,
            failure_code: ReviewNoteCandidatePromotionFailureCodeV1::ReviewNoteCandidatePromotionFailureCodeInvalidRequest
                as u16,
            review_result_outbox: result,
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map(|_| ())
        .map_err(ReviewedNoteCandidatePromotionEventErrorV1::Persistence)
}

async fn complete_cleanup_if_required(
    persistence: &ReviewedNoteCandidatePromotionPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    persisted: &makosh_reviewed_note_candidate_promotion_persistence::PersistedPromotionApprovalV1,
    now_unix_millis: i64,
) -> Result<(), ReviewedNoteCandidatePromotionEventErrorV1> {
    if persisted.cleanup_completed_at_unix_millis.is_some() {
        return Ok(());
    }
    let materialized_reference_id = persisted
        .materialized_reference_id
        .ok_or(ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)?;
    release_source_v1(
        channel,
        dispatcher,
        persisted.approval_message_id,
        &persisted.source_blob,
        materialized_reference_id,
    )
    .map_err(blob_error)?;
    persistence
        .complete_source_cleanup(
            &persisted.logical_owner_id,
            &persisted.approval_message_id,
            &materialized_reference_id,
            now_unix_millis,
        )
        .await
        .map_err(ReviewedNoteCandidatePromotionEventErrorV1::Persistence)
}

fn blob_error(error: PromotionBlobHandoffErrorV1) -> ReviewedNoteCandidatePromotionEventErrorV1 {
    match error {
        PromotionBlobHandoffErrorV1::InvalidReceipt => {
            ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload
        }
        PromotionBlobHandoffErrorV1::Unavailable => {
            ReviewedNoteCandidatePromotionEventErrorV1::EventUnavailable
        }
    }
}

struct DecodedApprovalV1 {
    payload: NoteCandidateApprovedForPromotionV1,
    review_id: [u8; 16],
    candidate_id: [u8; 16],
    candidate_digest: [u8; 32],
    source_evidence_id: [u8; 16],
    decided_by_owner_device_id: [u8; 16],
}

fn decode_approval(
    record: &OutboxRecordV1,
    expected_owner: &str,
) -> Result<DecodedApprovalV1, ReviewedNoteCandidatePromotionEventErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidEnvelope)?;
    validate_contract(
        &envelope,
        &review_note_candidate_approved_contract_reference_v1(),
    )?;
    if !matches!(
        envelope.semantics,
        Some(Semantics::Event(EventMetadataV1 { .. }))
    ) {
        return Err(ReviewedNoteCandidatePromotionEventErrorV1::InvalidEnvelope);
    }
    let payload = NoteCandidateApprovedForPromotionV1::decode(envelope.payload.as_slice())
        .map_err(|_| ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload)?;
    let review_id = id16(&payload.review_id)?;
    let candidate_id = id16(&payload.candidate_id)?;
    if envelope.partition_key.as_slice() != review_id
        || envelope.correlation_id.as_slice() != review_id
        || payload.logical_owner_id != expected_owner
        || !valid_owner(&payload.logical_owner_id)
        || payload.source_evidence_revision == 0
        || payload.decision_revision == 0
        || payload.candidate_content.is_none()
    {
        return Err(ReviewedNoteCandidatePromotionEventErrorV1::InvalidPayload);
    }
    Ok(DecodedApprovalV1 {
        candidate_digest: id32(&payload.candidate_digest)?,
        source_evidence_id: id16(&payload.source_evidence_id)?,
        decided_by_owner_device_id: id16(&payload.decided_by_owner_device_id)?,
        payload,
        review_id,
        candidate_id,
    })
}

fn knowledge_context(
    runtime: &ReviewedNoteCandidatePromotionRuntimeContextV1<'_>,
) -> KnowledgeCommandEnvelopeContextV1 {
    KnowledgeCommandEnvelopeContextV1 {
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
    use makosh_review_note_candidate_api::{
        ReviewNoteCandidateEnvelopeContextV1,
        build_review_note_candidate_approved_outbox_record_v1,
        wire::{NoteCandidateApprovedForPromotionV1, ReviewTargetBoundCandidateReceiptV1},
    };

    use super::*;

    #[test]
    fn approval_decode_preserves_only_typed_knowledge_handoff_fields() {
        let record = build_review_note_candidate_approved_outbox_record_v1(
            NoteCandidateApprovedForPromotionV1 {
                review_id: vec![1; 16],
                candidate_id: vec![2; 16],
                candidate_digest: vec![3; 32],
                source_evidence_id: vec![4; 16],
                source_evidence_revision: 5,
                decision_revision: 6,
                decided_by_owner_device_id: vec![7; 16],
                candidate_content: Some(ReviewTargetBoundCandidateReceiptV1 {
                    reference_id: vec![8; 16],
                    declared_bytes: 9,
                    sha256: vec![10; 32],
                    custody_transfer_source_proof: vec![11; 32],
                }),
                logical_owner_id: "owner-1".to_owned(),
            },
            &ReviewNoteCandidateEnvelopeContextV1 {
                module_id: "review-producer-v1".to_owned(),
                runtime_instance_id: "runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("approved event");
        let decoded = decode_approval(&record, "owner-1").expect("decoded approval");
        assert_eq!(decoded.review_id, [1; 16]);
        assert_eq!(decoded.candidate_id, [2; 16]);
        assert_eq!(decoded.payload.decision_revision, 6);
    }
}
