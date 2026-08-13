#![forbid(unsafe_code)]

#[cfg(feature = "conformance-test-support")]
mod conformance;
mod repository;
mod schema;

#[cfg(feature = "conformance-test-support")]
pub use conformance::{
    ReviewedPersonMatchCandidatePromotionCountsV1,
    ReviewedPersonMatchCandidatePromotionPersistenceConformanceV1,
};

pub use repository::{
    PersistReviewedPersonMatchApprovalFailureV1, PersistReviewedPersonMatchApprovalV1,
    PersistReviewedPersonMatchTerminalV1, ReviewedPersonMatchCandidatePromotionEnvelopeV1,
    ReviewedPersonMatchCandidatePromotionOutboxV1,
    ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    ReviewedPersonMatchCandidatePromotionPersistenceV1,
    ReviewedPersonMatchCandidatePromotionReplayV1,
};
pub use schema::{
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_SCHEMA_V1,
    reviewed_person_match_candidate_promotion_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-reviewed-person-match-candidate-promotion-persistence";
