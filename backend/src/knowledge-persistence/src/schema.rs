use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const KNOWLEDGE_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const KNOWLEDGE_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_knowledge.sql");

#[must_use]
pub fn knowledge_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: KNOWLEDGE_STORAGE_BUNDLE_REVISION_V1,
        bundle_id: "knowledge".to_owned(),
        owner_id: "knowledge".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: KNOWLEDGE_STORAGE_BUNDLE_REVISION_V1,
            migration_id: "knowledge_initial".to_owned(),
            forward_sql_utf8: KNOWLEDGE_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(KNOWLEDGE_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_knowledge_owned_and_cross_owner_negative() {
        let bundle = knowledge_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid Knowledge storage bundle");
        assert_eq!(bundle.owner_id, "knowledge");
        let sql = std::str::from_utf8(KNOWLEDGE_SCHEMA_V1).expect("utf8");
        for required in [
            "knowledge_reviewed_candidate_inbox",
            "knowledge_state",
            "knowledge_outbox",
            "command_envelope_sha256",
            "command_fingerprint",
            "materialized_blob_declared_bytes",
            "materialized_blob_sha256",
            "materialized_blob_custody_proof",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "review_note_candidate_",
            "communications_",
            "calendar_",
            "contacts_",
            "projects_",
            "obligations_",
            "provider_id",
            "account_id",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
