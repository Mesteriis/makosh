use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const PROJECTS_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const PROJECTS_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_projects_owner.sql");

#[must_use]
pub fn projects_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: PROJECTS_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "projects".to_owned(),
        owner_id: "projects".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: PROJECTS_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "projects_owner_initial".to_owned(),
            forward_sql_utf8: PROJECTS_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(PROJECTS_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn all_owner_tables_are_force_rls_and_provider_neutral() {
        validate_storage_bundle(&projects_storage_bundle_v1()).expect("Projects storage bundle");
        let sql = std::str::from_utf8(PROJECTS_SCHEMA_V1).expect("utf8");
        for table in [
            "projects_records",
            "projects_outcomes",
            "projects_references",
            "projects_client_operations",
            "projects_outbox",
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
            "communication_body",
            "document_bytes",
            "credential",
            "private_locator",
        ] {
            assert!(!sql.to_ascii_lowercase().contains(forbidden), "{forbidden}");
        }
    }
}
