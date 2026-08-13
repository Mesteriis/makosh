use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const OBLIGATIONS_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const OBLIGATIONS_STORAGE_BUNDLE_REVISION_V2: u32 = 2;
pub const OBLIGATIONS_STORAGE_BUNDLE_REVISION_V3: u32 = 3;
pub const OBLIGATIONS_SCHEMA_V1: &[u8] = include_bytes!("../migrations/0001_obligations_owner.sql");
pub const OBLIGATIONS_LIFECYCLE_OWNER_RLS_SCHEMA_V2: &[u8] =
    include_bytes!("../migrations/0002_obligations_lifecycle_owner_rls.sql");
pub const OBLIGATIONS_PARTIES_EVIDENCE_SCHEMA_V3: &[u8] =
    include_bytes!("../migrations/0003_obligations_parties_evidence.sql");

#[must_use]
pub fn obligations_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: OBLIGATIONS_STORAGE_BUNDLE_REVISION_V3,
        bundle_id: "obligations".to_owned(),
        owner_id: "obligations".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: OBLIGATIONS_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "obligations_initial".to_owned(),
                forward_sql_utf8: OBLIGATIONS_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(OBLIGATIONS_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: OBLIGATIONS_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "obligations_lifecycle_owner_rls".to_owned(),
                forward_sql_utf8: OBLIGATIONS_LIFECYCLE_OWNER_RLS_SCHEMA_V2.to_vec(),
                sha256: Sha256::digest(OBLIGATIONS_LIFECYCLE_OWNER_RLS_SCHEMA_V2).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: OBLIGATIONS_STORAGE_BUNDLE_REVISION_V3,
                migration_id: "obligations_parties_evidence".to_owned(),
                forward_sql_utf8: OBLIGATIONS_PARTIES_EVIDENCE_SCHEMA_V3.to_vec(),
                sha256: Sha256::digest(OBLIGATIONS_PARTIES_EVIDENCE_SCHEMA_V3).to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_obligations_owned_and_cross_owner_negative() {
        let bundle = obligations_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid Obligations storage bundle");
        assert_eq!(bundle.owner_id, "obligations");
        assert_eq!(bundle.revision, 3);
        assert_eq!(bundle.steps.len(), 3);
        let sql = std::str::from_utf8(OBLIGATIONS_SCHEMA_V1).expect("utf8");
        for required in [
            "obligations_reviewed_candidate_inbox",
            "obligations_state",
            "obligations_outbox",
            "command_envelope_sha256",
            "command_fingerprint",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "review_obligation_candidate_",
            "communications_",
            "calendar_",
            "contacts_",
            "projects_",
            "tasks_",
            "provider_id",
            "account_id",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
        let rls = bundle
            .steps
            .iter()
            .map(|step| std::str::from_utf8(&step.forward_sql_utf8).expect("rls utf8"))
            .collect::<Vec<_>>()
            .join("\n");
        for required in [
            "obligations_evidence",
            "obligated_party_id",
            "obligations_client_operations",
            "ENABLE ROW LEVEL SECURITY",
            "FORCE ROW LEVEL SECURITY",
            "current_setting('makosh.logical_owner_id', true)",
        ] {
            assert!(rls.contains(required), "{required}");
        }
        assert!(rls.contains("DROP TABLE makosh_data.obligations_dependencies"));
        assert!(rls.contains("DROP TABLE makosh_data.obligations_checklist"));
    }
}
