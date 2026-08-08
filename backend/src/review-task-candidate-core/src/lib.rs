#![forbid(unsafe_code)]

mod lifecycle;
mod model;

pub use lifecycle::{
    ReviewTaskCandidateTransitionErrorV1, create_review_task_candidate_v1,
    decide_review_task_candidate_v1, record_review_task_candidate_promotion_v1,
};
pub use model::{
    ReviewTaskCandidateDecisionV1, ReviewTaskCandidateDraftV1,
    ReviewTaskCandidatePromotionResultV1, ReviewTaskCandidatePromotionStatusV1,
    ReviewTaskCandidateStateV1, ReviewTaskCandidateTimestampV1, ReviewTaskCandidateV1,
    ReviewTaskCandidateValidationErrorV1, derive_review_task_candidate_id_v1,
    validate_review_task_candidate_v1,
};

pub const PACKAGE: &str = "makosh-review-task-candidate-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const DIGEST_BYTES_V1: usize = 32;
pub const MAX_LOGICAL_OWNER_ID_BYTES_V1: usize = 128;
pub const MAX_TITLE_CHARS_V1: usize = 240;
pub const MAX_HINT_CHARS_V1: usize = 120;
