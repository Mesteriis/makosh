#![forbid(unsafe_code)]

mod model;
mod repository;
mod row_codec;
mod schema;

pub use model::{
    CheckReviewTaskCandidateDecisionReplayV1, CompleteReviewTaskCandidateSubmissionV1,
    DecideReviewTaskCandidateOperationV1, ListReviewTaskCandidatesV1,
    PersistReviewTaskCandidateMaterializationV1, PersistReviewTaskCandidatePromotionResultV1,
    PersistedReviewTaskCandidateSubmissionV1, REVIEW_TASK_CANDIDATE_MAX_PAGE_SIZE_V1,
    RejectReviewTaskCandidateSubmissionV1, ReserveReviewTaskCandidateSubmissionOutcomeV1,
    ReserveReviewTaskCandidateSubmissionV1, ReviewTaskCandidateBlobCleanupV1,
    ReviewTaskCandidateBlobReceiptV1, ReviewTaskCandidateDecisionOutcomeV1,
    ReviewTaskCandidateInboxOutcomeV1, ReviewTaskCandidateOutboxRecordV1,
    ReviewTaskCandidatePageV1, ReviewTaskCandidatePersistenceErrorV1,
    ReviewTaskCandidateRealtimeTransitionV1,
};
pub use repository::ReviewTaskCandidatePersistenceV1;
pub use schema::{
    REVIEW_TASK_CANDIDATE_SCHEMA_V1, REVIEW_TASK_CANDIDATE_STORAGE_BUNDLE_REVISION_V1,
    REVIEW_TASK_CANDIDATE_STORAGE_BUNDLE_REVISION_V2, review_task_candidate_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-review-task-candidate-persistence";
