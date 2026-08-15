use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const REVIEW_OBLIGATION_CANDIDATE_STORAGE_BUNDLE_REVISION_V1: u32 = 1;
pub const REVIEW_OBLIGATION_CANDIDATE_STORAGE_BUNDLE_REVISION_V2: u32 = 2;
pub const REVIEW_OBLIGATION_CANDIDATE_STORAGE_BUNDLE_REVISION_V3: u32 = 3;
pub const REVIEW_OBLIGATION_CANDIDATE_STORAGE_OWNER_V1: &str = "review_obligation_candidate";
pub const REVIEW_OBLIGATION_CANDIDATE_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_review_obligation_candidate.sql");
pub const REVIEW_OBLIGATION_CANDIDATE_OWNER_RLS_SCHEMA_V2: &[u8] =
    include_bytes!("../migrations/0002_review_obligation_candidate_owner_rls.sql");
pub const REVIEW_OBLIGATION_CANDIDATE_PARTIES_EVIDENCE_SCHEMA_V3: &[u8] =
    include_bytes!("../migrations/0003_review_obligation_candidate_parties_evidence.sql");

#[must_use]
pub fn review_obligation_candidate_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: REVIEW_OBLIGATION_CANDIDATE_STORAGE_BUNDLE_REVISION_V3,
        bundle_id: "review_obligation_candidate".to_owned(),
        owner_id: REVIEW_OBLIGATION_CANDIDATE_STORAGE_OWNER_V1.to_owned(),
        steps: vec![
            StorageMigrationStepV1 {
                revision: REVIEW_OBLIGATION_CANDIDATE_STORAGE_BUNDLE_REVISION_V1,
                migration_id: "review_obligation_candidate_initial".to_owned(),
                forward_sql_utf8: REVIEW_OBLIGATION_CANDIDATE_SCHEMA_V1.to_vec(),
                sha256: Sha256::digest(REVIEW_OBLIGATION_CANDIDATE_SCHEMA_V1).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: REVIEW_OBLIGATION_CANDIDATE_STORAGE_BUNDLE_REVISION_V2,
                migration_id: "review_obligation_candidate_owner_rls".to_owned(),
                forward_sql_utf8: REVIEW_OBLIGATION_CANDIDATE_OWNER_RLS_SCHEMA_V2.to_vec(),
                sha256: Sha256::digest(REVIEW_OBLIGATION_CANDIDATE_OWNER_RLS_SCHEMA_V2).to_vec(),
            },
            StorageMigrationStepV1 {
                revision: REVIEW_OBLIGATION_CANDIDATE_STORAGE_BUNDLE_REVISION_V3,
                migration_id: "review_obligation_candidate_parties_evidence".to_owned(),
                forward_sql_utf8: REVIEW_OBLIGATION_CANDIDATE_PARTIES_EVIDENCE_SCHEMA_V3.to_vec(),
                sha256: Sha256::digest(REVIEW_OBLIGATION_CANDIDATE_PARTIES_EVIDENCE_SCHEMA_V3)
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
    fn bundle_is_review_obligation_candidate_owned_atomic_and_cross_owner_negative() {
        let bundle = review_obligation_candidate_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("valid Review obligation-candidate storage bundle");
        assert_eq!(
            bundle.owner_id,
            REVIEW_OBLIGATION_CANDIDATE_STORAGE_OWNER_V1
        );
        assert_eq!(bundle.revision, 3);
        let sql = bundle
            .steps
            .iter()
            .map(|step| std::str::from_utf8(&step.forward_sql_utf8).expect("utf8"))
            .collect::<Vec<_>>()
            .join("\n");
        for required in [
            "review_obligation_candidate_submissions",
            "review_obligation_candidate_state",
            "review_obligation_candidate_operations",
            "review_obligation_candidate_promotion_inbox",
            "review_obligation_candidate_outbox",
            "review_obligation_candidate_realtime",
            "review_obligation_candidate_evidence",
            "obligated_party_id",
        ] {
            assert!(sql.contains(required), "{required}");
        }
        assert!(sql.contains("FORCE ROW LEVEL SECURITY"));
        for forbidden in [
            "communications_",
            "obligations_",
            "mail_",
            "telegram_",
            "provider_id",
            "account_id",
            "prompt",
            "model_id",
        ] {
            assert!(!sql.contains(forbidden), "{forbidden}");
        }
    }
}
