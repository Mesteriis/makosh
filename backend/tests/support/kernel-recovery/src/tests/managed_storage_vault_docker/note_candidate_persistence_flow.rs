//! Live owner-local persistence negatives for the reviewed candidate promotion workflow.

use super::*;

use makosh_knowledge_command_api::{
    KnowledgeCommandEnvelopeContextV1,
    build_create_knowledge_note_from_reviewed_candidate_outbox_record_v1,
    wire::{
        CreateKnowledgeNoteFromReviewedCandidateCommandV1, KnowledgeTargetBoundCandidateReceiptV1,
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
use makosh_reviewed_note_candidate_promotion_core::{
    REVIEWED_NOTE_CANDIDATE_PROMOTION_MODULE_ID_V1, derive_reviewed_note_candidate_command_id_v1,
    derive_reviewed_note_candidate_result_id_v1,
};
use makosh_reviewed_note_candidate_promotion_persistence::{
    PersistPromotionApprovalOutcomeV1, PersistPromotionApprovalV1,
    PersistPromotionMaterializationV1, PersistPromotionResultOutcomeV1,
    PersistPromotionTerminalResultV1, PromotionBlobReceiptV1, ReservePromotionApprovalOutcomeV1,
    ReservePromotionApprovalV1, ReviewedNoteCandidatePromotionOutcomeV1,
    ReviewedNoteCandidatePromotionPersistenceConformanceV1,
    ReviewedNoteCandidatePromotionPersistenceErrorV1,
};

const NOTE_CANDIDATE_PERSISTENCE_CONFORMANCE_OWNER_ID_V1: &str =
    "owner-note-persistence-conformance";

pub(super) fn assert_reviewed_note_candidate_persistence_negatives_v1(
    runtime: &tokio::runtime::Runtime,
) {
    runtime.block_on(async {
        let pool = note_candidate_admin_pool_v1().await;
        let persistence =
            ReviewedNoteCandidatePromotionPersistenceConformanceV1::from_disposable_pool(
                pool.clone(),
            );
        let approval_message_id = [0xa1; 16];
        let review_id = [0xa2; 16];
        let candidate_id = [0xa3; 16];
        let decision_revision = 7;
        let knowledge_command_id = derive_reviewed_note_candidate_command_id_v1(
            approval_message_id,
            review_id,
            candidate_id,
            decision_revision,
        )
        .expect("promotion conformance command id");
        let reservation = ReservePromotionApprovalV1 {
            logical_owner_id: NOTE_CANDIDATE_PERSISTENCE_CONFORMANCE_OWNER_ID_V1.to_owned(),
            approval_message_id,
            approval_envelope_sha256: [0xaa; 32],
            review_id,
            candidate_id,
            decision_revision,
            source_blob: PromotionBlobReceiptV1 {
                reference_id: [0xa7; 16],
                declared_bytes: 32,
                sha256: [0xa8; 32],
                custody_proof: vec![0xa9; 32],
            },
            knowledge_command_id,
            occurred_at_unix_millis: 1_900_000_000_000,
        };
        let reserved = persistence
            .reserve_approval(&reservation)
            .await
            .expect("reserve promotion conformance approval");
        assert!(matches!(
            reserved,
            ReservePromotionApprovalOutcomeV1::Reserved(_)
        ));
        let existing = persistence
            .reserve_approval(&reservation)
            .await
            .expect("replay promotion conformance approval reservation");
        assert!(matches!(
            existing,
            ReservePromotionApprovalOutcomeV1::Existing(_)
        ));
        let mut reservation_conflict = reservation.clone();
        reservation_conflict.approval_envelope_sha256 = [0xab; 32];
        assert_eq!(
            persistence.reserve_approval(&reservation_conflict).await,
            Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ApprovalConflict)
        );
        let materialized = persistence
            .persist_materialization(&PersistPromotionMaterializationV1 {
                logical_owner_id: NOTE_CANDIDATE_PERSISTENCE_CONFORMANCE_OWNER_ID_V1.to_owned(),
                approval_message_id,
                materialized_reference_id: [0xad; 16],
                materialized_at_unix_millis: 1_900_000_000_100,
            })
            .await
            .expect("persist promotion conformance Blob materialization");
        assert_eq!(materialized.materialized_reference_id, Some([0xad; 16]));
        let knowledge_command_outbox =
            build_create_knowledge_note_from_reviewed_candidate_outbox_record_v1(
                CreateKnowledgeNoteFromReviewedCandidateCommandV1 {
                    command_id: knowledge_command_id.to_vec(),
                    approved_candidate_id: candidate_id.to_vec(),
                    candidate_digest: vec![0xa4; 32],
                    source_evidence_id: vec![0xa5; 16],
                    source_evidence_revision: 2,
                    review_id: review_id.to_vec(),
                    decision_revision,
                    decided_by_owner_device_id: vec![0xa6; 16],
                    candidate_content: Some(KnowledgeTargetBoundCandidateReceiptV1 {
                        reference_id: vec![0xa7; 16],
                        declared_bytes: 32,
                        sha256: vec![0xa8; 32],
                        custody_transfer_source_proof: vec![0xa9; 32],
                    }),
                    logical_owner_id: NOTE_CANDIDATE_PERSISTENCE_CONFORMANCE_OWNER_ID_V1.to_owned(),
                },
                1_900_000_300,
                &knowledge_context_v1(),
            )
            .expect("promotion conformance Knowledge command");
        let approval = PersistPromotionApprovalV1 {
            logical_owner_id: NOTE_CANDIDATE_PERSISTENCE_CONFORMANCE_OWNER_ID_V1.to_owned(),
            approval_message_id,
            approval_envelope_sha256: [0xaa; 32],
            review_id,
            candidate_id,
            decision_revision,
            knowledge_command_id,
            knowledge_command_outbox,
            occurred_at_unix_millis: 1_900_000_000_000,
        };
        assert_eq!(
            persistence
                .persist_approval_and_knowledge_command(&approval)
                .await,
            Ok(PersistPromotionApprovalOutcomeV1::Applied)
        );
        assert_eq!(
            persistence
                .persist_approval_and_knowledge_command(&approval)
                .await,
            Ok(PersistPromotionApprovalOutcomeV1::Duplicate)
        );
        let mut approval_conflict = approval.clone();
        approval_conflict.approval_envelope_sha256 = [0xab; 32];
        assert_eq!(
            persistence
                .persist_approval_and_knowledge_command(&approval_conflict)
                .await,
            Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ApprovalConflict)
        );

        let unknown = terminal_result_v1([0xb1; 16], [0xb2; 16], review_id, candidate_id);
        assert_eq!(
            persistence
                .persist_knowledge_result_and_review_result(&unknown)
                .await,
            Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::NotFound)
        );

        let terminal =
            terminal_result_v1([0xb3; 16], knowledge_command_id, review_id, candidate_id);
        assert_eq!(
            persistence
                .persist_knowledge_result_and_review_result(&terminal)
                .await,
            Ok(PersistPromotionResultOutcomeV1::Applied)
        );
        assert_eq!(
            persistence
                .persist_knowledge_result_and_review_result(&terminal)
                .await,
            Ok(PersistPromotionResultOutcomeV1::Duplicate)
        );
        let mut result_hash_conflict = terminal.clone();
        result_hash_conflict.knowledge_result_envelope_sha256 = [0xb4; 32];
        assert_eq!(
            persistence
                .persist_knowledge_result_and_review_result(&result_hash_conflict)
                .await,
            Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ResultConflict)
        );
        let mut correlation_conflict = terminal.clone();
        correlation_conflict.candidate_id = [0xb5; 16];
        assert_eq!(
            persistence
                .persist_knowledge_result_and_review_result(&correlation_conflict)
                .await,
            Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ResultConflict)
        );
        let mut outbox_conflict = terminal.clone();
        outbox_conflict.review_result_outbox = promotion_result_outbox_v1(
            terminal.knowledge_result_message_id,
            knowledge_command_id,
            review_id,
            candidate_id,
            [0xb6; 16],
        );
        assert_eq!(
            persistence
                .persist_knowledge_result_and_review_result(&outbox_conflict)
                .await,
            Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::OutboxConflict)
        );
        pool.close().await;
    });
}

fn terminal_result_v1(
    knowledge_result_message_id: [u8; 16],
    knowledge_command_id: [u8; 16],
    review_id: [u8; 16],
    candidate_id: [u8; 16],
) -> PersistPromotionTerminalResultV1 {
    PersistPromotionTerminalResultV1 {
        logical_owner_id: NOTE_CANDIDATE_PERSISTENCE_CONFORMANCE_OWNER_ID_V1.to_owned(),
        knowledge_result_message_id,
        knowledge_result_envelope_sha256: [0xba; 32],
        knowledge_command_id,
        review_id,
        candidate_id,
        outcome: ReviewedNoteCandidatePromotionOutcomeV1::Succeeded {
            note_id: [0xbb; 16],
        },
        review_result_outbox: promotion_result_outbox_v1(
            knowledge_result_message_id,
            knowledge_command_id,
            review_id,
            candidate_id,
            [0xbb; 16],
        ),
        occurred_at_unix_millis: 1_900_000_001_000,
    }
}

fn promotion_result_outbox_v1(
    knowledge_result_message_id: [u8; 16],
    knowledge_command_id: [u8; 16],
    review_id: [u8; 16],
    candidate_id: [u8; 16],
    note_id: [u8; 16],
) -> makosh_events_protocol::delivery::OutboxRecordV1 {
    let result_id = derive_reviewed_note_candidate_result_id_v1(
        knowledge_result_message_id,
        knowledge_command_id,
        review_id,
    )
    .expect("promotion conformance result id");
    build_review_note_candidate_promotion_result_outbox_record_v1(
        knowledge_result_message_id,
        ReviewNoteCandidatePromotionResultV1 {
            result_id: result_id.to_vec(),
            review_id: review_id.to_vec(),
            candidate_id: candidate_id.to_vec(),
            expected_review_revision: 7,
            outcome: ReviewNoteCandidatePromotionOutcomeV1::ReviewNoteCandidatePromotionOutcomeSucceeded
                as i32,
            note_id: Some(note_id.to_vec()),
            failure_code: ReviewNoteCandidatePromotionFailureCodeV1::ReviewNoteCandidatePromotionFailureCodeUnspecified
                as i32,
            logical_owner_id: NOTE_CANDIDATE_PERSISTENCE_CONFORMANCE_OWNER_ID_V1.to_owned(),
        },
        &ReviewNoteCandidatePromotionEnvelopeContextV1 {
            module_id: REVIEWED_NOTE_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
            runtime_instance_id: "promotion-conformance-runtime-1".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1_900_000_001,
            recorded_at_nanos: 0,
        },
    )
    .expect("promotion conformance Review result")
}

fn knowledge_context_v1() -> KnowledgeCommandEnvelopeContextV1 {
    KnowledgeCommandEnvelopeContextV1 {
        module_id: REVIEWED_NOTE_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
        runtime_instance_id: "promotion-conformance-runtime-1".to_owned(),
        runtime_generation: 1,
        recorded_at_unix_seconds: 1_900_000_000,
        recorded_at_nanos: 0,
    }
}
