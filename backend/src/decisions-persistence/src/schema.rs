use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const DECISIONS_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const DECISIONS_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_decisions_owner.sql");

#[must_use]
pub fn decisions_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: DECISIONS_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "decisions".to_owned(),
        owner_id: "decisions".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: DECISIONS_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "decisions_owner_initial".to_owned(),
            forward_sql_utf8: DECISIONS_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(DECISIONS_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn all_owner_tables_force_rls_and_exclude_private_source_content() {
        validate_storage_bundle(&decisions_storage_bundle_v1()).expect("bundle");
        let sql = std::str::from_utf8(DECISIONS_SCHEMA_V1).expect("utf8");
        for table in [
            "decisions_records",
            "decisions_alternatives",
            "decisions_evidence_links",
            "decisions_client_operations",
            "decisions_outbox",
        ] {
            assert!(
                sql.contains(&format!("CREATE TABLE makosh_data.{table}")),
                "{table}"
            );
            assert!(
                sql.contains(&format!(
                    "ALTER TABLE makosh_data.{table} FORCE ROW LEVEL SECURITY"
                )),
                "{table}"
            );
        }
        for forbidden in [
            "raw_payload",
            "credential",
            "private_locator",
            "provider_body",
        ] {
            assert!(!sql.to_ascii_lowercase().contains(forbidden), "{forbidden}");
        }
    }
}
