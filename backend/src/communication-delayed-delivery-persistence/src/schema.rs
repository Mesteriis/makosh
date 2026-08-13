use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V2: u32 = 2;
pub const COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V3: u32 = 3;
pub const COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V4: u32 = 4;
pub const COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V5: u32 = 5;
pub const COMMUNICATION_DELAYED_DELIVERY_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_delayed_delivery_state.sql");
pub const COMMUNICATION_DELAYED_DELIVERY_SCHEDULER_RECEIPT_SCHEMA_V2: &[u8] =
    include_bytes!("../migrations/0002_scheduler_receipt_outbox.sql");
pub const COMMUNICATION_DELAYED_DELIVERY_CLIENT_REALTIME_SCHEMA_V3: &[u8] =
    include_bytes!("../migrations/0003_client_realtime_replay.sql");
pub const COMMUNICATION_DELAYED_DELIVERY_BODY_CLEANUP_SCHEMA_V4: &[u8] =
    include_bytes!("../migrations/0004_body_cleanup_queue.sql");
pub const COMMUNICATION_DELAYED_DELIVERY_OWNER_RLS_SCHEMA_V5: &[u8] =
    include_bytes!("../migrations/0005_owner_rls.sql");

#[must_use]
pub fn communication_delayed_delivery_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V5,
        bundle_id: "communication_delayed_delivery_state".to_owned(),
        owner_id: "communication_delayed_delivery".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "communication_delayed_delivery_state_initial".to_owned(),
                forward_sql_utf8: COMMUNICATION_DELAYED_DELIVERY_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(COMMUNICATION_DELAYED_DELIVERY_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "communication_delayed_delivery_scheduler_receipts".to_owned(),
                forward_sql_utf8: COMMUNICATION_DELAYED_DELIVERY_SCHEDULER_RECEIPT_SCHEMA_V2
                    .to_vec(),
                sha256: Sha256::digest(COMMUNICATION_DELAYED_DELIVERY_SCHEDULER_RECEIPT_SCHEMA_V2)
                    .to_vec(),
            },
            StorageMigrationStepV1 {
                revision: COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V3,
                migration_id: "communication_delayed_delivery_client_realtime".to_owned(),
                forward_sql_utf8: COMMUNICATION_DELAYED_DELIVERY_CLIENT_REALTIME_SCHEMA_V3.to_vec(),
                sha256: Sha256::digest(COMMUNICATION_DELAYED_DELIVERY_CLIENT_REALTIME_SCHEMA_V3)
                    .to_vec(),
            },
            StorageMigrationStepV1 {
                revision: COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V4,
                migration_id: "communication_delayed_delivery_body_cleanup".to_owned(),
                forward_sql_utf8: COMMUNICATION_DELAYED_DELIVERY_BODY_CLEANUP_SCHEMA_V4.to_vec(),
                sha256: Sha256::digest(COMMUNICATION_DELAYED_DELIVERY_BODY_CLEANUP_SCHEMA_V4)
                    .to_vec(),
            },
            StorageMigrationStepV1 {
                revision: COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V5,
                migration_id: "communication_delayed_delivery_owner_rls".to_owned(),
                forward_sql_utf8: COMMUNICATION_DELAYED_DELIVERY_OWNER_RLS_SCHEMA_V5.to_vec(),
                sha256: Sha256::digest(COMMUNICATION_DELAYED_DELIVERY_OWNER_RLS_SCHEMA_V5).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_is_owner_local_and_stores_no_plaintext_body() {
        let bundle = communication_delayed_delivery_storage_bundle_v1();
        assert_eq!(bundle.owner_id, "communication_delayed_delivery");
        assert_eq!(bundle.revision, 5);
        assert_eq!(bundle.steps.len(), 5);
        let sql = std::str::from_utf8(COMMUNICATION_DELAYED_DELIVERY_SCHEMA_V1)
            .expect("migration is UTF-8");
        assert!(sql.contains("communication_delayed_delivery_operations"));
        assert!(sql.contains("communication_delayed_delivery_scheduler_inbox"));
        assert!(sql.contains("communication_delayed_delivery_outbox"));
        assert!(sql.contains("body_reference_id"));
        assert!(!sql.contains("body_utf8"));
        assert!(!sql.contains("provider"));
        assert!(!sql.contains("account_id"));
        let receipt_sql =
            std::str::from_utf8(COMMUNICATION_DELAYED_DELIVERY_SCHEDULER_RECEIPT_SCHEMA_V2)
                .expect("receipt migration is UTF-8");
        assert!(receipt_sql.contains("scheduler.job_run.acceptance.v1"));
        assert!(receipt_sql.contains("scheduler.job_run.result.v1"));
        let realtime_sql =
            std::str::from_utf8(COMMUNICATION_DELAYED_DELIVERY_CLIENT_REALTIME_SCHEMA_V3)
                .expect("realtime migration is UTF-8");
        assert!(realtime_sql.contains("communication_delayed_delivery_realtime"));
        assert!(realtime_sql.contains("realtime_sequence"));
        assert!(!realtime_sql.contains("body_utf8"));
        let cleanup_sql =
            std::str::from_utf8(COMMUNICATION_DELAYED_DELIVERY_BODY_CLEANUP_SCHEMA_V4)
                .expect("cleanup migration is UTF-8");
        assert!(cleanup_sql.contains("communication_delayed_delivery_body_cleanup"));
        assert!(cleanup_sql.contains("next_attempt_at_unix_millis"));
        assert!(!cleanup_sql.contains("body_utf8"));
        let rls_sql = bundle
            .steps
            .last()
            .and_then(|step| std::str::from_utf8(&step.forward_sql_utf8).ok())
            .expect("RLS migration is UTF-8");
        for table in [
            "communication_delayed_delivery_operations",
            "communication_delayed_delivery_scheduler_inbox",
            "communication_delayed_delivery_outbox",
            "communication_delayed_delivery_scheduler_receipt_outbox",
            "communication_delayed_delivery_realtime",
            "communication_delayed_delivery_body_cleanup",
        ] {
            assert!(
                rls_sql.contains(&format!(
                    "ALTER TABLE makosh_data.{table} ENABLE ROW LEVEL SECURITY"
                )),
                "{table} must enable RLS"
            );
            assert!(
                rls_sql.contains(&format!(
                    "ALTER TABLE makosh_data.{table} FORCE ROW LEVEL SECURITY"
                )),
                "{table} must force RLS"
            );
        }
        assert_eq!(rls_sql.matches("CREATE POLICY ").count(), 6);
        assert_eq!(
            rls_sql
                .matches("current_setting('makosh.logical_owner_id', true)")
                .count(),
            12
        );
    }
}
