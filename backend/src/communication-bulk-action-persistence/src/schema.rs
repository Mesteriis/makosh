use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_BULK_ACTION_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const COMMUNICATION_BULK_ACTION_STORAGE_BUNDLE_REVISION_V2: u32 = 2;
pub const COMMUNICATION_BULK_ACTION_STORAGE_BUNDLE_REVISION_V3: u32 = 3;
pub const COMMUNICATION_BULK_ACTION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_bulk_delivery_state.sql");
pub const COMMUNICATION_BULK_ACTION_REALTIME_SCHEMA_V2: &[u8] =
    include_bytes!("../migrations/0002_client_realtime_replay.sql");
pub const COMMUNICATION_BULK_ACTION_OWNER_RLS_SCHEMA_V3: &[u8] =
    include_bytes!("../migrations/0003_owner_rls.sql");

#[must_use]
pub fn communication_bulk_action_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: COMMUNICATION_BULK_ACTION_STORAGE_BUNDLE_REVISION_V3,
        bundle_id: "communication_bulk_action_state".to_owned(),
        owner_id: "communication_bulk_action".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: COMMUNICATION_BULK_ACTION_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "communication_bulk_action_state_initial".to_owned(),
                forward_sql_utf8: COMMUNICATION_BULK_ACTION_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(COMMUNICATION_BULK_ACTION_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: COMMUNICATION_BULK_ACTION_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "communication_bulk_action_client_realtime_replay".to_owned(),
                forward_sql_utf8: COMMUNICATION_BULK_ACTION_REALTIME_SCHEMA_V2.to_vec(),
                sha256: Sha256::digest(COMMUNICATION_BULK_ACTION_REALTIME_SCHEMA_V2).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: COMMUNICATION_BULK_ACTION_STORAGE_BUNDLE_REVISION_V3,
                migration_id: "communication_bulk_action_owner_rls".to_owned(),
                forward_sql_utf8: COMMUNICATION_BULK_ACTION_OWNER_RLS_SCHEMA_V3.to_vec(),
                sha256: Sha256::digest(COMMUNICATION_BULK_ACTION_OWNER_RLS_SCHEMA_V3).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_additive_owner_local_and_bounded() {
        let bundle = communication_bulk_action_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid owner storage bundle");
        assert_eq!(bundle.owner_id, "communication_bulk_action");
        assert_eq!(bundle.revision, 3);
        assert_eq!(bundle.steps.len(), 3);
        let sql = bundle
            .steps
            .iter()
            .map(|step| std::str::from_utf8(&step.forward_sql_utf8).expect("utf8"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(sql.contains("communication_bulk_action_batches"));
        assert!(sql.contains("communication_bulk_action_targets"));
        assert!(sql.contains("target_count BETWEEN 1 AND 100"));
        assert!(sql.contains("attempt_count BETWEEN 0 AND 3"));
        assert!(sql.contains("body_utf8 BYTEA"));
        assert!(sql.contains("communication_bulk_action_realtime"));
        assert!(sql.contains("realtime_sequence"));
        assert!(!sql.contains("communications_"));
        assert!(!sql.contains("mail_"));
        assert!(!sql.contains("telegram_"));
        assert!(!sql.contains("provider"));
        let rls_sql = bundle
            .steps
            .last()
            .and_then(|step| std::str::from_utf8(&step.forward_sql_utf8).ok())
            .expect("RLS migration is UTF-8");
        for table in [
            "communication_bulk_action_batches",
            "communication_bulk_action_targets",
            "communication_bulk_action_realtime",
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
        assert_eq!(rls_sql.matches("CREATE POLICY ").count(), 3);
        assert_eq!(
            rls_sql
                .matches("current_setting('makosh.logical_owner_id', true)")
                .count(),
            6
        );
    }
}
