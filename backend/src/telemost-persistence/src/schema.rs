use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};
pub const TELEMOST_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_telemost.sql");
#[must_use]
pub fn telemost_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "telemost".into(),
        owner_id: "telemost".into(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "telemost_initial".into(),
            forward_sql_utf8: TELEMOST_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(TELEMOST_SCHEMA_V1).to_vec(),
        }],
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use makosh_storage_protocol::validation::validate_storage_bundle;
    #[test]
    fn bundle_is_force_rls_and_private_free() {
        let b = telemost_storage_bundle_v1();
        validate_storage_bundle(&b).unwrap();
        let sql = std::str::from_utf8(TELEMOST_SCHEMA_V1).unwrap();
        assert_eq!(sql.matches("FORCE ROW LEVEL SECURITY").count(), 3);
        for bad in [
            "credential",
            "access_token",
            "refresh_token",
            "provider_payload",
            "webhook_body",
            "join_url",
            "participant_email",
            "json",
        ] {
            assert!(!sql.to_ascii_lowercase().contains(bad), "{bad}");
        }
    }
}
