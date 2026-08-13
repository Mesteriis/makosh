use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const IDENTITY_RESOLUTION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_identity_resolution.sql");

#[must_use]
pub fn identity_resolution_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "identity_resolution".to_owned(),
        owner_id: "identity_resolution".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "identity_resolution_initial".to_owned(),
            forward_sql_utf8: IDENTITY_RESOLUTION_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(IDENTITY_RESOLUTION_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_storage_protocol::validation::validate_storage_bundle;

    #[test]
    fn bundle_is_force_rls_and_private_free() {
        let bundle = identity_resolution_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("bundle");
        let sql = std::str::from_utf8(IDENTITY_RESOLUTION_SCHEMA_V1).expect("utf8");
        assert_eq!(sql.matches("FORCE ROW LEVEL SECURITY").count(), 3);
        assert_eq!(
            sql.matches("current_setting('makosh.logical_owner_id', true)")
                .count(),
            6
        );
        for forbidden in [
            "email",
            "phone",
            "credential",
            "provider_payload",
            "private_locator",
            "json",
        ] {
            assert!(!sql.to_ascii_lowercase().contains(forbidden), "{forbidden}");
        }
    }
}
