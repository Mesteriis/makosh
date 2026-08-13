use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_reviewed_person_match_candidate_promotion.sql");

#[must_use]
pub fn reviewed_person_match_candidate_promotion_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "reviewed_person_match_candidate_promotion".to_owned(),
        owner_id: "reviewed_person_match_candidate_promotion".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "reviewed_person_match_candidate_promotion_initial".to_owned(),
            forward_sql_utf8: REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_storage_protocol::validation::validate_storage_bundle;

    #[test]
    fn storage_is_workflow_owned_force_rls_and_private_negative() {
        validate_storage_bundle(&reviewed_person_match_candidate_promotion_storage_bundle_v1())
            .expect("bundle");
        let sql =
            std::str::from_utf8(REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_SCHEMA_V1).expect("utf8");
        assert!(sql.contains("FORCE ROW LEVEL SECURITY"));
        for forbidden in [
            "normalized_email",
            "normalized_phone",
            "provider_entry_id",
            "credential",
            "raw_payload",
        ] {
            assert!(!sql.contains(forbidden));
        }
    }
}
