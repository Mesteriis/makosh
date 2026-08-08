use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const REVIEW_TASK_CANDIDATE_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const REVIEW_TASK_CANDIDATE_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_review_task_candidate.sql");

#[must_use]
pub fn review_task_candidate_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: REVIEW_TASK_CANDIDATE_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "review_task_candidate".to_owned(),
        owner_id: "review".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: REVIEW_TASK_CANDIDATE_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "review_task_candidate_initial".to_owned(),
            forward_sql_utf8: REVIEW_TASK_CANDIDATE_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(REVIEW_TASK_CANDIDATE_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_review_owned_atomic_and_cross_owner_negative() {
        let bundle = review_task_candidate_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid Review task-candidate storage bundle");
        assert_eq!(bundle.owner_id, "review");
        let sql = std::str::from_utf8(REVIEW_TASK_CANDIDATE_SCHEMA_V1).expect("utf8");
        for required in [
            "review_task_candidate_submissions",
            "review_task_candidate_state",
            "review_task_candidate_operations",
            "review_task_candidate_promotion_inbox",
            "review_task_candidate_outbox",
            "review_task_candidate_realtime",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "communications_",
            "tasks_",
            "mail_",
            "telegram_",
            "provider_id",
            "account_id",
            "prompt",
            "model_id",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
