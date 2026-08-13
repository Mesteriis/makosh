use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};
pub const CONSISTENCY_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_consistency.sql");
#[must_use]
pub fn consistency_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "consistency".into(),
        owner_id: "consistency".into(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "consistency_projection_initial".into(),
            forward_sql_utf8: CONSISTENCY_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(CONSISTENCY_SCHEMA_V1).to_vec(),
        }],
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use makosh_storage_protocol::validation::validate_storage_bundle;
    #[test]
    fn bundle_is_generation_scoped_force_rls_and_inference_free() {
        validate_storage_bundle(&consistency_storage_bundle_v1()).unwrap();
        let sql = std::str::from_utf8(CONSISTENCY_SCHEMA_V1).unwrap();
        assert_eq!(sql.matches("FORCE ROW LEVEL SECURITY").count(), 5);
        for forbidden in [
            "confidence",
            "risk",
            "inference",
            "credential",
            "provider_payload",
            "private_locator",
            "json",
        ] {
            assert!(!sql.to_ascii_lowercase().contains(forbidden));
        }
    }
}
