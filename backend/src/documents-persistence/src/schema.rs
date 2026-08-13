use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const DOCUMENTS_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const DOCUMENTS_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_documents_owner.sql");

#[must_use]
pub fn documents_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: DOCUMENTS_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "documents".to_owned(),
        owner_id: "documents".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "documents_owner_initial".to_owned(),
            forward_sql_utf8: DOCUMENTS_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(DOCUMENTS_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn every_document_table_is_force_rls_and_content_byte_free() {
        validate_storage_bundle(&documents_storage_bundle_v1()).expect("Documents storage bundle");
        let sql = std::str::from_utf8(DOCUMENTS_SCHEMA_V1).expect("utf8");
        for table in [
            "documents_records",
            "documents_sources",
            "documents_client_operations",
            "documents_blob_operations",
            "documents_outbox",
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
            "content_bytes",
            "body_utf8",
            "private_locator",
            "provider_credential",
        ] {
            assert!(!sql.to_ascii_lowercase().contains(forbidden), "{forbidden}");
        }
    }
}
