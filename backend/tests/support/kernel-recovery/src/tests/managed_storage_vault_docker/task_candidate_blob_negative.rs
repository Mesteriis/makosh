//! Live fail-closed proof for a stale reviewed-candidate Blob custody receipt.

use super::*;

use makosh_events_jetstream::DurableSubjectV1;
use makosh_events_protocol::v1::DurableEnvelopeV1;
use makosh_reviewed_task_candidate_promotion_core::REVIEWED_TASK_CANDIDATE_PROMOTION_MODULE_ID_V1;
use makosh_tasks_command_api::{
    TasksCommandEnvelopeContextV1, build_create_task_from_reviewed_candidate_outbox_record_v1,
    wire::{
        CreateTaskFromReviewedCandidateCommandV1, TaskCreationFromReviewedCandidateRejectedV1,
        TaskCreationRejectCodeV1, TasksTargetBoundCandidateReceiptV1,
    },
};

const STALE_BLOB_CANDIDATE_ID_V1: [u8; 16] = [0xc2; 16];

pub(super) fn assert_tasks_reject_stale_blob_receipt_v1(
    runtime: &tokio::runtime::Runtime,
    store: &SqliteControlStore,
) {
    runtime.block_on(async {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Tasks stale Blob wall clock")
            .as_secs() as i64;
        let command = build_create_task_from_reviewed_candidate_outbox_record_v1(
            CreateTaskFromReviewedCandidateCommandV1 {
                command_id: vec![0xc1; 16],
                approved_candidate_id: STALE_BLOB_CANDIDATE_ID_V1.to_vec(),
                candidate_digest: vec![0xc3; 32],
                source_evidence_id: vec![0xc4; 16],
                source_evidence_revision: 2,
                review_id: vec![0xc5; 16],
                decision_revision: 2,
                decided_by_owner_device_id: vec![0xc6; 16],
                candidate_content: Some(TasksTargetBoundCandidateReceiptV1 {
                    reference_id: vec![0xc7; 16],
                    declared_bytes: 32,
                    sha256: vec![0xc8; 32],
                    custody_transfer_source_proof: vec![0xc9; 32],
                }),
                logical_owner_id: TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
            },
            now + 300,
            &TasksCommandEnvelopeContextV1 {
                module_id: REVIEWED_TASK_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "stale-blob-conformance-runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: now,
                recorded_at_nanos: 0,
            },
        )
        .expect("stale Blob Tasks command");
        let endpoint = store
            .platform_event_hub_topology()
            .expect("read stale Blob Event Hub topology")
            .expect("stale Blob Event Hub topology")
            .nats_endpoint()
            .to_owned();
        let client = async_nats::connect(endpoint)
            .await
            .expect("connect stale Blob observer");
        let mut results = client
            .subscribe("makosh.result.v1.tasks.>")
            .await
            .expect("subscribe stale Blob Tasks results");
        let context = async_nats::jetstream::new(client);
        let envelope = DurableEnvelopeV1::decode(command.exact_bytes())
            .expect("decode stale Blob Tasks command");
        let subject = DurableSubjectV1::from_envelope(&envelope)
            .expect("derive stale Blob Tasks subject")
            .as_str();
        context
            .publish(subject, command.exact_bytes().to_vec().into())
            .await
            .expect("publish stale Blob Tasks command")
            .await
            .expect("acknowledge stale Blob Tasks command");
        let rejected = tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                let message = results.next().await.expect("stale Blob result stream");
                let envelope = DurableEnvelopeV1::decode(message.payload.as_ref())
                    .expect("decode stale Blob Tasks result");
                let Ok(payload) = TaskCreationFromReviewedCandidateRejectedV1::decode(
                    envelope.payload.as_slice(),
                ) else {
                    continue;
                };
                if payload.command_id == vec![0xc1; 16] {
                    return payload;
                }
            }
        })
        .await
        .expect("stale Blob terminal rejection timeout");
        assert_eq!(rejected.approved_candidate_id, STALE_BLOB_CANDIDATE_ID_V1);
        assert_eq!(
            rejected.code,
            TaskCreationRejectCodeV1::TaskCreationRejectCodeBlobMismatch as i32
        );
        let pool = task_candidate_admin_pool_v1().await;
        let tasks: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM makosh_data.tasks_state
             WHERE logical_owner_id=$1 AND approved_candidate_id=$2",
        )
        .bind(TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
        .bind(STALE_BLOB_CANDIDATE_ID_V1.as_slice())
        .fetch_one(&pool)
        .await
        .expect("count stale Blob Tasks");
        assert_eq!(tasks, 0, "stale Blob receipt must not create Task");
        pool.close().await;
    });
}
