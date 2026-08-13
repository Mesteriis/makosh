use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};
pub const GRAPH_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_graph.sql");
#[must_use]
pub fn graph_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "graph".into(),
        owner_id: "graph".into(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "graph_projection_initial".into(),
            forward_sql_utf8: GRAPH_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(GRAPH_SCHEMA_V1).to_vec(),
        }],
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use makosh_storage_protocol::validation::validate_storage_bundle;
    #[test]
    fn bundle_is_generation_scoped_force_rls_and_inference_free() {
        validate_storage_bundle(&graph_storage_bundle_v1()).unwrap();
        let sql = std::str::from_utf8(GRAPH_SCHEMA_V1).unwrap();
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
