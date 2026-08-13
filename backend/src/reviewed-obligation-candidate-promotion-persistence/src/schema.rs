use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_BUNDLE_REVISION_V2: u32 = 2;
pub const REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_reviewed_obligation_candidate_promotion.sql");
pub const REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_OWNER_RLS_SCHEMA_V2: &[u8] =
    include_bytes!("../migrations/0002_reviewed_obligation_candidate_promotion_owner_rls.sql");

#[must_use]
pub fn reviewed_obligation_candidate_promotion_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_BUNDLE_REVISION_V2,
        bundle_id: "reviewed_obligation_candidate_promotion".to_owned(),
        owner_id: "reviewed_obligation_candidate_promotion".to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "reviewed_obligation_candidate_promotion_initial".to_owned(),
                forward_sql_utf8: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "reviewed_obligation_candidate_promotion_owner_rls".to_owned(),
                forward_sql_utf8: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_OWNER_RLS_SCHEMA_V2
                    .to_vec(),
                sha256: Sha256::digest(REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_OWNER_RLS_SCHEMA_V2)
                    .to_vec(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_workflow_owned_and_payload_private() {
        let bundle = reviewed_obligation_candidate_promotion_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid promotion workflow storage bundle");
        assert_eq!(bundle.owner_id, "reviewed_obligation_candidate_promotion");
        assert_eq!(bundle.revision, 2);
        assert_eq!(bundle.steps.len(), 2);
        let sql =
            std::str::from_utf8(REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_SCHEMA_V1).expect("utf8");
        for required in [
            "reviewed_obligation_candidate_promotion_requests",
            "approval_envelope_sha256",
            "obligations_id",
            "reviewed_obligation_candidate_promotion_result_inbox",
            "reviewed_obligation_candidate_promotion_outbox",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        for forbidden in [
            "candidate_content",
            "source_body",
            "custody_proof",
            "provider_id",
            "account_id",
            "statement",
            "due_text",
            "assignee_label",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
        let rls = std::str::from_utf8(REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_OWNER_RLS_SCHEMA_V2)
            .expect("rls utf8");
        assert_eq!(rls.matches("ENABLE ROW LEVEL SECURITY").count(), 3);
        assert_eq!(rls.matches("FORCE ROW LEVEL SECURITY").count(), 3);
        assert!(rls.contains("current_setting('makosh.logical_owner_id', true)"));
    }
}
