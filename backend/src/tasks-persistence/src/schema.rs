use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const TASKS_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const TASKS_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_tasks.sql");

#[must_use]
pub fn tasks_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: TASKS_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "tasks".to_owned(),
        owner_id: "tasks".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: TASKS_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "tasks_initial".to_owned(),
            forward_sql_utf8: TASKS_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(TASKS_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_tasks_owned_and_cross_owner_negative() {
        let bundle = tasks_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid Tasks storage bundle");
        assert_eq!(bundle.owner_id, "tasks");
        let sql = std::str::from_utf8(TASKS_SCHEMA_V1).expect("utf8");
        for required in [
            "tasks_reviewed_candidate_inbox",
            "tasks_state",
            "tasks_outbox",
            "command_envelope_sha256",
            "command_fingerprint",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "review_task_candidate_",
            "communications_",
            "calendar_",
            "contacts_",
            "projects_",
            "obligations_",
            "provider_id",
            "account_id",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
