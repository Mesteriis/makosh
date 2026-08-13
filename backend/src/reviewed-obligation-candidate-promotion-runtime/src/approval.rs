use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{EventMetadataV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_obligations_api::{
    ObligationsCommandEnvelopeContextV1,
    build_create_obligation_from_reviewed_candidate_outbox_record_v1,
    wire::{
        CreateObligationFromReviewedCandidateCommandV1, ObligationsTargetBoundCandidateReceiptV1,
    },
};
use makosh_review_obligation_candidate_api::{
    review_obligation_candidate_approved_contract_reference_v1,
    wire::ObligationCandidateApprovedForPromotionV1,
};
use makosh_reviewed_obligation_candidate_promotion_core::derive_reviewed_obligation_candidate_command_id_v1;
use makosh_reviewed_obligation_candidate_promotion_persistence::{
    PersistPromotionApprovalV1, ReviewedObligationCandidatePromotionPersistenceV1,
};
use prost::Message;

use crate::{
    obligation_results::{
        ReviewedObligationCandidatePromotionEventErrorV1,
        ReviewedObligationCandidatePromotionRuntimeContextV1,
    },
    validation::{id16, id32, valid_owner, validate_contract},
};

const OBLIGATIONS_COMMAND_DEADLINE_SECONDS_V1: i64 = 300;

pub(crate) async fn consume_approval_once_v1(
    persistence: &ReviewedObligationCandidatePromotionPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &ReviewedObligationCandidatePromotionRuntimeContextV1<'_>,
) -> Result<bool, ReviewedObligationCandidatePromotionEventErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReviewedObligationCandidatePromotionEventErrorV1::InvalidEnvelope)?;
    let approval = decode_approval(&record, runtime.logical_human_owner_id)?;
    let command_id = derive_reviewed_obligation_candidate_command_id_v1(
        *record.message_id(),
        approval.review_id,
        approval.candidate_id,
        approval.payload.decision_revision,
    )
    .map_err(|_| ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload)?;
    let receipt = approval
        .payload
        .candidate_content
        .ok_or(ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload)?;
    let command = build_create_obligation_from_reviewed_candidate_outbox_record_v1(
        CreateObligationFromReviewedCandidateCommandV1 {
            command_id: command_id.to_vec(),
            approved_candidate_id: approval.candidate_id.to_vec(),
            candidate_digest: approval.candidate_digest.to_vec(),
            source_evidence_id: approval.source_evidence_id.to_vec(),
            source_evidence_revision: approval.payload.source_evidence_revision,
            review_id: approval.review_id.to_vec(),
            decision_revision: approval.payload.decision_revision,
            decided_by_owner_device_id: approval.decided_by_owner_device_id.to_vec(),
            candidate_content: Some(ObligationsTargetBoundCandidateReceiptV1 {
                reference_id: receipt.reference_id,
                declared_bytes: receipt.declared_bytes,
                sha256: receipt.sha256,
                custody_transfer_source_proof: receipt.custody_transfer_source_proof,
            }),
            logical_owner_id: runtime.logical_human_owner_id.to_owned(),
        },
        runtime
            .now_unix_millis
            .checked_div(1_000)
            .and_then(|value| value.checked_add(OBLIGATIONS_COMMAND_DEADLINE_SECONDS_V1))
            .ok_or(ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload)?,
        &obligations_context(runtime),
    )
    .map_err(|_| ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload)?;
    persistence
        .persist_approval_and_obligations(&PersistPromotionApprovalV1 {
            logical_owner_id: runtime.logical_human_owner_id.to_owned(),
            approval_message_id: *record.message_id(),
            approval_envelope_sha256: *record.envelope_sha256(),
            review_id: approval.review_id,
            candidate_id: approval.candidate_id,
            decision_revision: approval.payload.decision_revision,
            obligations_id: command_id,
            obligations_outbox: command,
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(ReviewedObligationCandidatePromotionEventErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

struct DecodedApprovalV1 {
    payload: ObligationCandidateApprovedForPromotionV1,
    review_id: [u8; 16],
    candidate_id: [u8; 16],
    candidate_digest: [u8; 32],
    source_evidence_id: [u8; 16],
    decided_by_owner_device_id: [u8; 16],
}

fn decode_approval(
    record: &OutboxRecordV1,
    expected_owner: &str,
) -> Result<DecodedApprovalV1, ReviewedObligationCandidatePromotionEventErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ReviewedObligationCandidatePromotionEventErrorV1::InvalidEnvelope)?;
    validate_contract(
        &envelope,
        &review_obligation_candidate_approved_contract_reference_v1(),
    )?;
    if !matches!(
        envelope.semantics,
        Some(Semantics::Event(EventMetadataV1 { .. }))
    ) {
        return Err(ReviewedObligationCandidatePromotionEventErrorV1::InvalidEnvelope);
    }
    let payload = ObligationCandidateApprovedForPromotionV1::decode(envelope.payload.as_slice())
        .map_err(|_| ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload)?;
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
        return Err(ReviewedObligationCandidatePromotionEventErrorV1::InvalidPayload);
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

fn obligations_context(
    runtime: &ReviewedObligationCandidatePromotionRuntimeContextV1<'_>,
) -> ObligationsCommandEnvelopeContextV1 {
    ObligationsCommandEnvelopeContextV1 {
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
    use makosh_review_obligation_candidate_api::{
        ReviewObligationCandidateEnvelopeContextV1,
        build_review_obligation_candidate_approved_outbox_record_v1,
        wire::{ObligationCandidateApprovedForPromotionV1, ReviewTargetBoundCandidateReceiptV1},
    };

    use super::*;

    #[test]
    fn approval_decode_preserves_only_typed_obligations_handoff_fields() {
        let record = build_review_obligation_candidate_approved_outbox_record_v1(
            ObligationCandidateApprovedForPromotionV1 {
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
            &ReviewObligationCandidateEnvelopeContextV1 {
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
