use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const ORGANIZATIONS_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const ORGANIZATIONS_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_organizations_owner.sql");

#[must_use]
pub fn organizations_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: ORGANIZATIONS_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "organizations".to_owned(),
        owner_id: "organizations".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: ORGANIZATIONS_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "organizations_owner_initial".to_owned(),
            forward_sql_utf8: ORGANIZATIONS_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(ORGANIZATIONS_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn all_owner_tables_are_force_rls_and_provider_neutral() {
        validate_storage_bundle(&organizations_storage_bundle_v1())
            .expect("Organizations storage bundle");
        let sql = std::str::from_utf8(ORGANIZATIONS_SCHEMA_V1).expect("utf8");
        for table in [
            "organizations_records",
            "organizations_sources",
            "organizations_client_operations",
            "organizations_outbox",
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
            "clearbit",
            "crunchbase",
            "linkedin",
            "credential",
            "private_locator",
        ] {
            assert!(!sql.to_ascii_lowercase().contains(forbidden), "{forbidden}");
        }
    }
}
