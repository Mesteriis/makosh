use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const MAIL_CONTACTS_SYNC_STORAGE_BUNDLE_REVISION_V1: u32 = 6;
pub const MAIL_CONTACTS_SYNC_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_mail_contacts_sync.sql");
pub const MAIL_CONTACTS_SYNC_ORCHESTRATION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0002_mail_contacts_sync_orchestration.sql");
pub const MAIL_CONTACTS_SYNC_REVERSE_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0003_reverse_sync.sql");
pub const MAIL_CONTACTS_SYNC_SCHEDULER_COMPLETION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0004_scheduler_completion.sql");
pub const MAIL_CONTACTS_SYNC_REVERSE_ORIGIN_RUN_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0005_reverse_origin_run.sql");
pub const MAIL_CONTACTS_SYNC_PROVIDER_LINK_RECONCILIATION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0006_provider_link_reconciliation.sql");

#[must_use]
pub fn mail_contacts_sync_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: MAIL_CONTACTS_SYNC_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "mail_contacts_sync".to_owned(),
        owner_id: "mail_contacts_sync".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "mail_contacts_sync_initial".to_owned(),
                forward_sql_utf8: MAIL_CONTACTS_SYNC_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(MAIL_CONTACTS_SYNC_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 2,
                migration_id: "mail_contacts_sync_orchestration".to_owned(),
                forward_sql_utf8: MAIL_CONTACTS_SYNC_ORCHESTRATION_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(MAIL_CONTACTS_SYNC_ORCHESTRATION_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 3,
                migration_id: "mail_contacts_sync_reverse".to_owned(),
                forward_sql_utf8: MAIL_CONTACTS_SYNC_REVERSE_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(MAIL_CONTACTS_SYNC_REVERSE_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 4,
                migration_id: "mail_contacts_sync_scheduler_completion".to_owned(),
                forward_sql_utf8: MAIL_CONTACTS_SYNC_SCHEDULER_COMPLETION_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(MAIL_CONTACTS_SYNC_SCHEDULER_COMPLETION_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 5,
                migration_id: "mail_contacts_sync_reverse_origin_run".to_owned(),
                forward_sql_utf8: MAIL_CONTACTS_SYNC_REVERSE_ORIGIN_RUN_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(MAIL_CONTACTS_SYNC_REVERSE_ORIGIN_RUN_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: MAIL_CONTACTS_SYNC_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "mail_contacts_sync_provider_link_reconciliation".to_owned(),
                forward_sql_utf8: MAIL_CONTACTS_SYNC_PROVIDER_LINK_RECONCILIATION_SCHEMA_V1
                    .to_vec(),
                sha256: Sha256::digest(MAIL_CONTACTS_SYNC_PROVIDER_LINK_RECONCILIATION_SCHEMA_V1)
                    .to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_workflow_owned_and_has_no_foreign_tables_or_provider_secrets() {
        let bundle = mail_contacts_sync_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid storage bundle");
        assert_eq!(bundle.owner_id, "mail_contacts_sync");
        assert_eq!(bundle.steps.len(), 6);
        let sql = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            std::str::from_utf8(MAIL_CONTACTS_SYNC_SCHEMA_V1).expect("utf8"),
            std::str::from_utf8(MAIL_CONTACTS_SYNC_ORCHESTRATION_SCHEMA_V1).expect("utf8"),
            std::str::from_utf8(MAIL_CONTACTS_SYNC_REVERSE_SCHEMA_V1).expect("utf8"),
            std::str::from_utf8(MAIL_CONTACTS_SYNC_SCHEDULER_COMPLETION_SCHEMA_V1).expect("utf8"),
            std::str::from_utf8(MAIL_CONTACTS_SYNC_REVERSE_ORIGIN_RUN_SCHEMA_V1).expect("utf8"),
            std::str::from_utf8(MAIL_CONTACTS_SYNC_PROVIDER_LINK_RECONCILIATION_SCHEMA_V1)
                .expect("utf8")
        );
        for required in [
            "mail_contacts_sync_runs",
            "mail_contacts_sync_inbox",
            "mail_contacts_sync_outbox",
            "mail_contacts_sync_realtime",
            "continuation_cursor",
            "mail_contacts_sync_pages",
            "mail_contacts_sync_entries",
            "mail_contacts_sync_reverse_inbox",
            "mail_contacts_sync_reverse_operations",
            "mail_contacts_sync_scheduler_runs",
            "mail_contacts_sync_provider_link_reconciliation",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "contacts_contacts",
            "mail_accounts",
            "communications_",
            "password",
            "access_token",
            "refresh_token",
            "cookie",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
