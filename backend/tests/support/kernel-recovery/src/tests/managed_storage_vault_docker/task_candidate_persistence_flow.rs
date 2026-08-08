//! Live owner-local persistence negatives for the reviewed candidate promotion workflow.

use super::*;

use makosh_review_task_candidate_promotion_api::{
    ReviewTaskCandidatePromotionEnvelopeContextV1,
    build_review_task_candidate_promotion_result_outbox_record_v1,
    wire::{
        ReviewTaskCandidatePromotionFailureCodeV1, ReviewTaskCandidatePromotionOutcomeV1,
        ReviewTaskCandidatePromotionResultV1,
    },
};
use makosh_reviewed_task_candidate_promotion_core::{
    REVIEWED_TASK_CANDIDATE_PROMOTION_MODULE_ID_V1, derive_reviewed_task_candidate_command_id_v1,
    derive_reviewed_task_candidate_result_id_v1,
};
use makosh_reviewed_task_candidate_promotion_persistence::{
    PersistPromotionApprovalOutcomeV1, PersistPromotionApprovalV1, PersistPromotionResultOutcomeV1,
    PersistPromotionTerminalResultV1, ReviewedTaskCandidatePromotionOutcomeV1,
    ReviewedTaskCandidatePromotionPersistenceConformanceV1,
    ReviewedTaskCandidatePromotionPersistenceErrorV1,
};
use makosh_tasks_command_api::{
    TasksCommandEnvelopeContextV1, build_create_task_from_reviewed_candidate_outbox_record_v1,
    wire::{CreateTaskFromReviewedCandidateCommandV1, TasksTargetBoundCandidateReceiptV1},
};

pub(super) fn assert_reviewed_task_candidate_persistence_negatives_v1(
    runtime: &tokio::runtime::Runtime,
) {
    runtime.block_on(async {
        let pool = task_candidate_admin_pool_v1().await;
        let persistence =
            ReviewedTaskCandidatePromotionPersistenceConformanceV1::from_disposable_pool(
                pool.clone(),
            );
        let approval_message_id = [0xa1; 16];
        let review_id = [0xa2; 16];
        let candidate_id = [0xa3; 16];
        let decision_revision = 7;
        let tasks_command_id = derive_reviewed_task_candidate_command_id_v1(
            approval_message_id,
            review_id,
            candidate_id,
            decision_revision,
        )
        .expect("promotion conformance command id");
        let tasks_command_outbox = build_create_task_from_reviewed_candidate_outbox_record_v1(
            CreateTaskFromReviewedCandidateCommandV1 {
                command_id: tasks_command_id.to_vec(),
                approved_candidate_id: candidate_id.to_vec(),
                candidate_digest: vec![0xa4; 32],
                source_evidence_id: vec![0xa5; 16],
                source_evidence_revision: 2,
                review_id: review_id.to_vec(),
                decision_revision,
                decided_by_owner_device_id: vec![0xa6; 16],
                candidate_content: Some(TasksTargetBoundCandidateReceiptV1 {
                    reference_id: vec![0xa7; 16],
                    declared_bytes: 32,
                    sha256: vec![0xa8; 32],
                    custody_transfer_source_proof: vec![0xa9; 32],
                }),
                logical_owner_id: TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
            },
            1_900_000_300,
            &tasks_context_v1(),
        )
        .expect("promotion conformance Tasks command");
        let approval = PersistPromotionApprovalV1 {
            logical_owner_id: TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
            approval_message_id,
            approval_envelope_sha256: [0xaa; 32],
            review_id,
            candidate_id,
            decision_revision,
            tasks_command_id,
            tasks_command_outbox,
            occurred_at_unix_millis: 1_900_000_000_000,
        };
        assert_eq!(
            persistence
                .persist_approval_and_tasks_command(&approval)
                .await,
            Ok(PersistPromotionApprovalOutcomeV1::Applied)
        );
        assert_eq!(
            persistence
                .persist_approval_and_tasks_command(&approval)
                .await,
            Ok(PersistPromotionApprovalOutcomeV1::Duplicate)
        );
        let mut approval_conflict = approval.clone();
        approval_conflict.approval_envelope_sha256 = [0xab; 32];
        assert_eq!(
            persistence
                .persist_approval_and_tasks_command(&approval_conflict)
                .await,
            Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::ApprovalConflict)
        );

        let unknown = terminal_result_v1([0xb1; 16], [0xb2; 16], review_id, candidate_id);
        assert_eq!(
            persistence
                .persist_tasks_result_and_review_result(&unknown)
                .await,
            Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::NotFound)
        );

        let terminal = terminal_result_v1([0xb3; 16], tasks_command_id, review_id, candidate_id);
        assert_eq!(
            persistence
                .persist_tasks_result_and_review_result(&terminal)
                .await,
            Ok(PersistPromotionResultOutcomeV1::Applied)
        );
        assert_eq!(
            persistence
                .persist_tasks_result_and_review_result(&terminal)
                .await,
            Ok(PersistPromotionResultOutcomeV1::Duplicate)
        );
        let mut result_hash_conflict = terminal.clone();
        result_hash_conflict.tasks_result_envelope_sha256 = [0xb4; 32];
        assert_eq!(
            persistence
                .persist_tasks_result_and_review_result(&result_hash_conflict)
                .await,
            Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::ResultConflict)
        );
        let mut correlation_conflict = terminal.clone();
        correlation_conflict.candidate_id = [0xb5; 16];
        assert_eq!(
            persistence
                .persist_tasks_result_and_review_result(&correlation_conflict)
                .await,
            Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::ResultConflict)
        );
        let mut outbox_conflict = terminal.clone();
        outbox_conflict.review_result_outbox = promotion_result_outbox_v1(
            terminal.tasks_result_message_id,
            tasks_command_id,
            review_id,
            candidate_id,
            [0xb6; 16],
        );
        assert_eq!(
            persistence
                .persist_tasks_result_and_review_result(&outbox_conflict)
                .await,
            Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::OutboxConflict)
        );
        pool.close().await;
    });
}

fn terminal_result_v1(
    tasks_result_message_id: [u8; 16],
    tasks_command_id: [u8; 16],
    review_id: [u8; 16],
    candidate_id: [u8; 16],
) -> PersistPromotionTerminalResultV1 {
    PersistPromotionTerminalResultV1 {
        logical_owner_id: TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        tasks_result_message_id,
        tasks_result_envelope_sha256: [0xba; 32],
        tasks_command_id,
        review_id,
        candidate_id,
        outcome: ReviewedTaskCandidatePromotionOutcomeV1::Succeeded {
            task_id: [0xbb; 16],
        },
        review_result_outbox: promotion_result_outbox_v1(
            tasks_result_message_id,
            tasks_command_id,
            review_id,
            candidate_id,
            [0xbb; 16],
        ),
        occurred_at_unix_millis: 1_900_000_001_000,
    }
}

fn promotion_result_outbox_v1(
    tasks_result_message_id: [u8; 16],
    tasks_command_id: [u8; 16],
    review_id: [u8; 16],
    candidate_id: [u8; 16],
    task_id: [u8; 16],
) -> makosh_events_protocol::delivery::OutboxRecordV1 {
    let result_id = derive_reviewed_task_candidate_result_id_v1(
        tasks_result_message_id,
        tasks_command_id,
        review_id,
    )
    .expect("promotion conformance result id");
    build_review_task_candidate_promotion_result_outbox_record_v1(
        tasks_result_message_id,
        ReviewTaskCandidatePromotionResultV1 {
            result_id: result_id.to_vec(),
            review_id: review_id.to_vec(),
            candidate_id: candidate_id.to_vec(),
            expected_review_revision: 7,
            outcome: ReviewTaskCandidatePromotionOutcomeV1::ReviewTaskCandidatePromotionOutcomeSucceeded
                as i32,
            task_id: Some(task_id.to_vec()),
            failure_code: ReviewTaskCandidatePromotionFailureCodeV1::ReviewTaskCandidatePromotionFailureCodeUnspecified
                as i32,
            logical_owner_id: TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        },
        &ReviewTaskCandidatePromotionEnvelopeContextV1 {
            module_id: REVIEWED_TASK_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
            runtime_instance_id: "promotion-conformance-runtime-1".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1_900_000_001,
            recorded_at_nanos: 0,
        },
    )
    .expect("promotion conformance Review result")
}

fn tasks_context_v1() -> TasksCommandEnvelopeContextV1 {
    TasksCommandEnvelopeContextV1 {
        module_id: REVIEWED_TASK_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
        runtime_instance_id: "promotion-conformance-runtime-1".to_owned(),
        runtime_generation: 1,
        recorded_at_unix_seconds: 1_900_000_000,
        recorded_at_nanos: 0,
    }
}
