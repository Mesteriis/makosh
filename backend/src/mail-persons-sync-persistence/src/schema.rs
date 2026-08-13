use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const MAIL_PERSONS_SYNC_STORAGE_BUNDLE_REVISION_V1: u32 = 2;
pub const MAIL_PERSONS_SYNC_INITIAL_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_mail_persons_sync.sql");
pub const MAIL_PERSONS_SYNC_ACCOUNT_SCHEDULER_BINDING_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0002_account_scheduler_binding.sql");

#[must_use]
pub fn mail_persons_sync_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: MAIL_PERSONS_SYNC_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "mail_persons_sync".to_owned(),
        owner_id: "mail_persons_sync".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "mail_persons_sync_initial".to_owned(),
                forward_sql_utf8: MAIL_PERSONS_SYNC_INITIAL_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(MAIL_PERSONS_SYNC_INITIAL_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 2,
                migration_id: "mail_persons_sync_account_scheduler_binding".to_owned(),
                forward_sql_utf8: MAIL_PERSONS_SYNC_ACCOUNT_SCHEDULER_BINDING_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(MAIL_PERSONS_SYNC_ACCOUNT_SCHEDULER_BINDING_SCHEMA_V1)
                    .to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_bundle_appends_owner_scoped_account_scheduler_binding() {
        let bundle = mail_persons_sync_storage_bundle_v1();
        assert_eq!(bundle.revision, 2);
        assert_eq!(bundle.steps.len(), 2);
        assert_eq!(bundle.steps[0].revision, 1);
        assert_eq!(bundle.steps[1].revision, 2);
        let sql = str::from_utf8(&bundle.steps[1].forward_sql_utf8).expect("utf8 migration");
        assert!(sql.contains("mail_persons_sync_account_bindings"));
        assert!(sql.contains("mail_persons_sync_schedule_control_outbox"));
        assert!(sql.contains("FORCE ROW LEVEL SECURITY"));
        assert!(!sql.contains("provider_"));
        assert!(!sql.contains("credential"));
        assert!(!sql.contains("private"));
    }
}
