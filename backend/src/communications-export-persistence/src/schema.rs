use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const COMMUNICATIONS_EXPORT_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const COMMUNICATIONS_EXPORT_STORAGE_BUNDLE_REVISION_V2: u32 = 2;
pub const COMMUNICATIONS_EXPORT_STORAGE_BUNDLE_REVISION_V3: u32 = 3;
pub const COMMUNICATIONS_EXPORT_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_communications_export_state.sql");
pub const COMMUNICATIONS_EXPORT_LOGICAL_OWNER_SCHEMA_V2: &[u8] =
    include_bytes!("../migrations/0002_communications_export_logical_owner.sql");
pub const COMMUNICATIONS_EXPORT_CLIENT_REALTIME_SCHEMA_V3: &[u8] =
    include_bytes!("../migrations/0003_communications_export_client_realtime.sql");

#[must_use]
pub fn communications_export_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: COMMUNICATIONS_EXPORT_STORAGE_BUNDLE_REVISION_V3,
        bundle_id: "communications_export_state".to_owned(),
        owner_id: "communications_export".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: COMMUNICATIONS_EXPORT_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "communications_export_state_initial".to_owned(),
                forward_sql_utf8: COMMUNICATIONS_EXPORT_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(COMMUNICATIONS_EXPORT_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: COMMUNICATIONS_EXPORT_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "communications_export_logical_owner".to_owned(),
                forward_sql_utf8: COMMUNICATIONS_EXPORT_LOGICAL_OWNER_SCHEMA_V2.to_vec(),
                sha256: Sha256::digest(COMMUNICATIONS_EXPORT_LOGICAL_OWNER_SCHEMA_V2).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: COMMUNICATIONS_EXPORT_STORAGE_BUNDLE_REVISION_V3,
                migration_id: "communications_export_client_realtime".to_owned(),
                forward_sql_utf8: COMMUNICATIONS_EXPORT_CLIENT_REALTIME_SCHEMA_V3.to_vec(),
                sha256: Sha256::digest(COMMUNICATIONS_EXPORT_CLIENT_REALTIME_SCHEMA_V3).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_valid_and_owner_scoped() {
        let bundle = communications_export_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("bundle");
        assert_eq!(
            bundle.revision,
            COMMUNICATIONS_EXPORT_STORAGE_BUNDLE_REVISION_V3
        );
        assert_eq!(bundle.steps.len(), 3);
        let sql = std::str::from_utf8(&bundle.steps[0].forward_sql_utf8).expect("utf8");
        assert!(sql.contains("makosh_data.communications_export_jobs"));
        assert!(!sql.contains("logical_owner_id"));
        assert!(!sql.contains("communications_messages"));
        assert!(!sql.contains("mail_"));
        assert!(!sql.contains("telegram_"));
        let successor_sql = std::str::from_utf8(&bundle.steps[1].forward_sql_utf8).expect("utf8");
        assert!(successor_sql.contains("ADD COLUMN logical_owner_id"));
        assert!(successor_sql.contains("logical_owner_id IS NULL"));
        let realtime_sql = std::str::from_utf8(&bundle.steps[2].forward_sql_utf8).expect("utf8");
        assert!(realtime_sql.contains("communications_export_client_realtime"));
        assert!(!realtime_sql.contains("artifact_sha256"));
    }
}
