use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};
pub const OMNIROUTE_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_omniroute.sql");
#[must_use]
pub fn omniroute_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "omniroute".into(),
        owner_id: "omniroute".into(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "omniroute_initial".into(),
            forward_sql_utf8: OMNIROUTE_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(OMNIROUTE_SCHEMA_V1).to_vec(),
        }],
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use makosh_storage_protocol::validation::validate_storage_bundle;
    #[test]
    fn bundle_is_force_rls_and_does_not_store_prompts_or_keys() {
        let b = omniroute_storage_bundle_v1();
        validate_storage_bundle(&b).unwrap();
        let sql = std::str::from_utf8(OMNIROUTE_SCHEMA_V1)
            .unwrap()
            .to_ascii_lowercase();
        assert_eq!(sql.matches("force row level security").count(), 1);
        for bad in [
            "credential",
            "api_key",
            "prompt",
            "provider_payload",
            "response_body",
            "json",
        ] {
            assert!(!sql.contains(bad), "{bad}");
        }
    }
}
