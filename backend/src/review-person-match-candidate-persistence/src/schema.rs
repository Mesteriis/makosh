use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
use sha2::{Digest, Sha256};

pub const REVIEW_PERSON_MATCH_CANDIDATE_SCHEMA_V1: &[u8] =
    include_bytes!("../migrations/0001_review_person_match_candidate.sql");

#[must_use]
pub fn review_person_match_candidate_storage_bundle_v1() -> StorageBundleV1 {
    StorageBundleV1 {
        major: 1,
        revision: 1,
        bundle_id: "review_person_match_candidate".to_owned(),
        owner_id: "review".to_owned(),
        steps: vec![StorageMigrationStepV1 {
            revision: 1,
            migration_id: "review_person_match_candidate_initial".to_owned(),
            forward_sql_utf8: REVIEW_PERSON_MATCH_CANDIDATE_SCHEMA_V1.to_vec(),
            sha256: Sha256::digest(REVIEW_PERSON_MATCH_CANDIDATE_SCHEMA_V1).to_vec(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::*;

    #[test]
    fn bundle_is_review_owned_rls_and_contains_no_private_provider_content() {
        let bundle = review_person_match_candidate_storage_bundle_v1();
        validate_storage_bundle(&bundle).expect("storage bundle");
        assert_eq!(bundle.owner_id, "review");
        let sql = std::str::from_utf8(REVIEW_PERSON_MATCH_CANDIDATE_SCHEMA_V1).expect("utf8");
        for required in [
            "review_person_match_candidate_state",
            "review_person_match_candidate_inbox",
            "review_person_match_candidate_outbox",
            "FORCE ROW LEVEL SECURITY",
        ] {
            assert!(sql.contains(required));
        }
        for forbidden in [
            "normalized_email",
            "normalized_phone",
            "provider_entry_id",
            "provider_etag",
            "continuation_cursor",
            "credential",
            "private_locator",
            "raw_payload",
        ] {
            assert!(!sql.contains(forbidden));
        }
    }
}
