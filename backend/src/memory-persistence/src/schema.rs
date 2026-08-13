use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};
pub const MEMORY_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_memory.sql");
#[must_use]
pub fn memory_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "memory".into(),
        owner_id: "memory".into(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "memory_projection_initial".into(),
            forward_sql_utf8: MEMORY_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(MEMORY_SCHEMA_V1).to_vec(),
        }],
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use makosh_storage_protocol::validation::validate_storage_bundle;
    #[test]
    fn bundle_is_generation_scoped_force_rls_and_private_free() {
        let bundle = memory_storage_bundle_v1();
        validate_storage_bundle(&bundle).unwrap();
        let sql = std::str::from_utf8(MEMORY_SCHEMA_V1).unwrap();
        assert_eq!(sql.matches("FORCE ROW LEVEL SECURITY").count(), 4);
        for forbidden in [
            "body_utf8",
            "title_utf8",
            "credential",
            "provider_payload",
            "private_locator",
            "json",
        ] {
            assert!(!sql.to_ascii_lowercase().contains(forbidden));
        }
    }
}
