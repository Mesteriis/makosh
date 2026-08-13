use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const PERSONS_STORAGE_BUNDLE_REVISION_V1: u32 = 3;
pub const PERSONS_INITIAL_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_persons.sql");
pub const PERSONS_DURABLE_SCHEMA_V2: &[u8] =
    include_bytes!("../migrations/0002_persons_durable.sql");
pub const PERSONS_OUTBOX_ORDER_SCHEMA_V3: &[u8] =
    include_bytes!("../migrations/0003_persons_outbox_order.sql");

#[must_use]
pub fn persons_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: PERSONS_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "persons".to_owned(),
        owner_id: "persons".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "persons_initial".to_owned(),
                forward_sql_utf8: PERSONS_INITIAL_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(PERSONS_INITIAL_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: 2,
                migration_id: "persons_durable".to_owned(),
                forward_sql_utf8: PERSONS_DURABLE_SCHEMA_V2.to_vec(),
                sha256: Sha256::digest(PERSONS_DURABLE_SCHEMA_V2).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: PERSONS_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "persons_outbox_order".to_owned(),
                forward_sql_utf8: PERSONS_OUTBOX_ORDER_SCHEMA_V3.to_vec(),
                sha256: Sha256::digest(PERSONS_OUTBOX_ORDER_SCHEMA_V3).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_forward_only_persons_owned_and_private() {
        let bundle = persons_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid Persons bundle");
        assert_eq!(bundle.owner_id, "persons");
        assert_eq!(bundle.revision, 3);
        assert_eq!(bundle.steps.len(), 3);
        assert_eq!(bundle.steps[0].revision, 1);
        assert_eq!(bundle.steps[1].revision, 2);
        assert_eq!(bundle.steps[2].revision, 3);
        let sql = [
            PERSONS_INITIAL_SCHEMA_V1,
            PERSONS_DURABLE_SCHEMA_V2,
            PERSONS_OUTBOX_ORDER_SCHEMA_V3,
        ]
        .concat();
        let sql = std::str::from_utf8(&sql).expect("utf8").to_lowercase();
        for required in [
            "force row level security",
            "persons_owner_aggregates",
            "persons_sources",
            "persons_lineage",
            "persons_decision_receipts",
            "persons_command_inbox",
            "persons_outbox",
            "outbox_ordinal",
            "semantic_order_key",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "json",
            "makosh_contacts",
            "credential",
            "raw_payload",
            "private_locator",
            "create role",
            "bypassrls",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
