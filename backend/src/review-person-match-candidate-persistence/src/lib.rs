#![forbid(unsafe_code)]

#[cfg(feature = "conformance-test-support")]
mod conformance;
mod repository;
mod schema;

#[cfg(feature = "conformance-test-support")]
pub use conformance::{
    ReviewPersonMatchCandidateDurableCountsV1, ReviewPersonMatchCandidatePersistenceConformanceV1,
    ReviewPersonMatchCandidateRlsEvidenceV1,
};

pub use repository::{
    DecidePersonMatchCandidateOperationV1, PersistPersonMatchCandidatePromotionResultV1,
    ReviewPersonMatchCandidateEnvelopeRecordV1, ReviewPersonMatchCandidateOutboxRecordV1,
    ReviewPersonMatchCandidatePersistenceErrorV1, ReviewPersonMatchCandidatePersistenceV1,
    ReviewPersonMatchCandidateReplayOutcomeV1, SubmitPersonMatchCandidateOperationV1,
};
pub use schema::{
    REVIEW_PERSON_MATCH_CANDIDATE_SCHEMA_V1, review_person_match_candidate_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-review-person-match-candidate-persistence";
