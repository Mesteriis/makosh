//! Live fail-closed proof for a stale reviewed-candidate Blob custody receipt.

use super::*;

use makosh_events_jetstream::DurableSubjectV1;
use makosh_events_protocol::v1::DurableEnvelopeV1;
use makosh_knowledge_command_api::{
    KnowledgeCommandEnvelopeContextV1,
    build_create_knowledge_note_from_reviewed_candidate_outbox_record_v1,
    wire::{
        CreateKnowledgeNoteFromReviewedCandidateCommandV1,
        KnowledgeNoteCreationFromReviewedCandidateRejectedV1, KnowledgeNoteCreationRejectCodeV1,
        KnowledgeTargetBoundCandidateReceiptV1,
    },
};
use makosh_reviewed_note_candidate_promotion_core::REVIEWED_NOTE_CANDIDATE_PROMOTION_MODULE_ID_V1;

const STALE_BLOB_CANDIDATE_ID_V1: [u8; 16] = [0xc2; 16];

pub(super) fn assert_knowledge_reject_stale_blob_receipt_v1(
    runtime: &tokio::runtime::Runtime,
    store: &SqliteControlStore,
) {
    runtime.block_on(async {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Knowledge stale Blob wall clock")
            .as_secs() as i64;
        let command = build_create_knowledge_note_from_reviewed_candidate_outbox_record_v1(
            CreateKnowledgeNoteFromReviewedCandidateCommandV1 {
                command_id: vec![0xc1; 16],
                approved_candidate_id: STALE_BLOB_CANDIDATE_ID_V1.to_vec(),
                candidate_digest: vec![0xc3; 32],
                source_evidence_id: vec![0xc4; 16],
                source_evidence_revision: 2,
                review_id: vec![0xc5; 16],
                decision_revision: 2,
                decided_by_owner_device_id: vec![0xc6; 16],
                candidate_content: Some(KnowledgeTargetBoundCandidateReceiptV1 {
                    reference_id: vec![0xc7; 16],
                    declared_bytes: 32,
                    sha256: vec![0xc8; 32],
                    custody_transfer_source_proof: vec![0xc9; 32],
                }),
                logical_owner_id: NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
            },
            now + 300,
            &KnowledgeCommandEnvelopeContextV1 {
                module_id: REVIEWED_NOTE_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "stale-blob-conformance-runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: now,
                recorded_at_nanos: 0,
            },
        )
        .expect("stale Blob Knowledge command");
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
            .subscribe("makosh.result.v1.knowledge.>")
            .await
            .expect("subscribe stale Blob Knowledge results");
        let context = async_nats::jetstream::new(client);
        let envelope = DurableEnvelopeV1::decode(command.exact_bytes())
            .expect("decode stale Blob Knowledge command");
        let subject = DurableSubjectV1::from_envelope(&envelope)
            .expect("derive stale Blob Knowledge subject")
            .as_str();
        context
            .publish(subject, command.exact_bytes().to_vec().into())
            .await
            .expect("publish stale Blob Knowledge command")
            .await
            .expect("acknowledge stale Blob Knowledge command");
        let rejected = tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                let message = results.next().await.expect("stale Blob result stream");
                let envelope = DurableEnvelopeV1::decode(message.payload.as_ref())
                    .expect("decode stale Blob Knowledge result");
                let Ok(payload) = KnowledgeNoteCreationFromReviewedCandidateRejectedV1::decode(
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
            KnowledgeNoteCreationRejectCodeV1::KnowledgeNoteCreationRejectCodeBlobMismatch as i32
        );
        let pool = note_candidate_admin_pool_v1().await;
        let knowledge: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM makosh_data.knowledge_state
             WHERE logical_owner_id=$1 AND approved_candidate_id=$2",
        )
        .bind(NOTE_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
        .bind(STALE_BLOB_CANDIDATE_ID_V1.as_slice())
        .fetch_one(&pool)
        .await
        .expect("count stale Blob Knowledge");
        assert_eq!(
            knowledge, 0,
            "stale Blob receipt must not create Knowledge note"
        );
        pool.close().await;
    });
}
