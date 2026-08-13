#![forbid(unsafe_code)]

mod lifecycle;
mod model;

pub use lifecycle::{
    ReviewObligationCandidateTransitionErrorV1, create_review_obligation_candidate_v1,
    decide_review_obligation_candidate_v1, record_review_obligation_candidate_promotion_v1,
};
pub use model::{
    ReviewObligationCandidateDecisionV1, ReviewObligationCandidateDraftV1,
    ReviewObligationCandidatePromotionResultV1, ReviewObligationCandidatePromotionStatusV1,
    ReviewObligationCandidateStateV1, ReviewObligationCandidateTimestampV1,
    ReviewObligationCandidateV1, ReviewObligationCandidateValidationErrorV1,
    ReviewObligationEvidenceLinkV1, derive_review_obligation_candidate_id_v1,
    validate_review_obligation_candidate_v1,
};

pub const PACKAGE: &str = "makosh-review-obligation-candidate-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const DIGEST_BYTES_V1: usize = 32;
pub const MAX_LOGICAL_OWNER_ID_BYTES_V1: usize = 128;
pub const MAX_STATEMENT_CHARS_V1: usize = 240;
pub const MAX_HINT_CHARS_V1: usize = 120;
