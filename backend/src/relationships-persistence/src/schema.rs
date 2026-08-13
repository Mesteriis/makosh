use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const RELATIONSHIPS_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const RELATIONSHIPS_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_relationships_owner.sql");

#[must_use]
pub fn relationships_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: RELATIONSHIPS_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "relationships".to_owned(),
        owner_id: "relationships".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "relationships_owner_initial".to_owned(),
            forward_sql_utf8: RELATIONSHIPS_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(RELATIONSHIPS_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn every_owner_table_is_force_rls_and_private_free() {
        validate_storage_bundle(&relationships_storage_bundle_v1()).expect("bundle");
        let sql = std::str::from_utf8(RELATIONSHIPS_SCHEMA_V1).expect("utf8");
        for table in [
            "relationships_records",
            "relationships_evidence",
            "relationships_client_operations",
            "relationships_outbox",
        ] {
            assert!(sql.contains(&format!("CREATE TABLE makosh_data.{table}")));
            assert!(sql.contains(&format!(
                "ALTER TABLE makosh_data.{table} FORCE ROW LEVEL SECURITY"
            )));
        }
        for forbidden in [
            "raw_payload",
            "private_locator",
            "credential",
            "body_utf8",
            "jsonb",
        ] {
            assert!(!sql.to_ascii_lowercase().contains(forbidden), "{forbidden}");
        }
    }
}
