use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const SEARCH_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_search.sql");

#[must_use]
pub fn search_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "search".to_owned(),
        owner_id: "search".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "search_projection_initial".to_owned(),
            forward_sql_utf8: SEARCH_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(SEARCH_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_storage_protocol::validation::validate_storage_bundle;

    #[test]
    fn bundle_is_generation_scoped_force_rls_and_plaintext_free() {
        let bundle = search_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("bundle");
        let sql = std::str::from_utf8(SEARCH_SCHEMA_V1).expect("utf8");
        assert_eq!(sql.matches("FORCE ROW LEVEL SECURITY").count(), 5);
        assert!(sql.contains("projection_generation"));
        assert!(sql.contains("source_revision"));
        assert!(sql.contains("deleted_at"));
        for forbidden in [
            "plaintext",
            "body_utf8",
            "title_utf8",
            "credential",
            "provider_payload",
            "private_locator",
            "json",
        ] {
            assert!(!sql.to_ascii_lowercase().contains(forbidden), "{forbidden}");
        }
    }
}
