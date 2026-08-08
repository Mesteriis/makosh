use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const REVIEWED_TASK_CANDIDATE_PROMOTION_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const REVIEWED_TASK_CANDIDATE_PROMOTION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_reviewed_task_candidate_promotion.sql");

#[must_use]
pub fn reviewed_task_candidate_promotion_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: REVIEWED_TASK_CANDIDATE_PROMOTION_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "reviewed_task_candidate_promotion".to_owned(),
        owner_id: "reviewed_task_candidate_promotion".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: REVIEWED_TASK_CANDIDATE_PROMOTION_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "reviewed_task_candidate_promotion_initial".to_owned(),
            forward_sql_utf8: REVIEWED_TASK_CANDIDATE_PROMOTION_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(REVIEWED_TASK_CANDIDATE_PROMOTION_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_workflow_owned_and_payload_private() {
        let bundle = reviewed_task_candidate_promotion_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid promotion workflow storage bundle");
        assert_eq!(bundle.owner_id, "reviewed_task_candidate_promotion");
        let sql = std::str::from_utf8(REVIEWED_TASK_CANDIDATE_PROMOTION_SCHEMA_V1).expect("utf8");
        for required in [
            "reviewed_task_candidate_promotion_requests",
            "approval_envelope_sha256",
            "tasks_command_id",
            "reviewed_task_candidate_promotion_result_inbox",
            "reviewed_task_candidate_promotion_outbox",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "candidate_content",
            "source_body",
            "custody_proof",
            "provider_id",
            "account_id",
            "title",
            "due_text",
            "assignee_label",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
