#![forbid(unsafe_code)]

mod lifecycle;
mod model;

pub use lifecycle::{
    ReviewNoteCandidateTransitionErrorV1, create_review_note_candidate_v1,
    decide_review_note_candidate_v1, record_review_note_candidate_promotion_v1,
};
pub use model::{
    ReviewNoteCandidateDecisionV1, ReviewNoteCandidateDraftV1,
    ReviewNoteCandidatePromotionResultV1, ReviewNoteCandidatePromotionStatusV1,
    ReviewNoteCandidateStateV1, ReviewNoteCandidateTimestampV1, ReviewNoteCandidateV1,
    ReviewNoteCandidateValidationErrorV1, ReviewNoteSourceBasisV1, ReviewNoteTopicHintV1,
    derive_review_note_candidate_id_v1, validate_review_note_candidate_v1,
};

pub const PACKAGE: &str = "makosh-review-note-candidate-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const DIGEST_BYTES_V1: usize = 32;
pub const MAX_LOGICAL_OWNER_ID_BYTES_V1: usize = 128;
pub const MAX_TITLE_CHARS_V1: usize = 240;
pub const MAX_EXCERPT_CHARS_V1: usize = 2_000;
pub const MAX_TOPIC_HINTS_V1: usize = 4;
