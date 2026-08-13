#![forbid(unsafe_code)]

mod model;
mod repository;
mod row_codec;
mod schema;

pub use model::{
    CheckReviewNoteCandidateDecisionReplayV1, CompleteReviewNoteCandidateSubmissionV1,
    DecideReviewNoteCandidateOperationV1, ListReviewNoteCandidatesV1,
    PersistReviewNoteCandidateMaterializationV1, PersistReviewNoteCandidatePromotionResultV1,
    PersistedReviewNoteCandidateSubmissionV1, REVIEW_NOTE_CANDIDATE_MAX_PAGE_SIZE_V1,
    RejectReviewNoteCandidateSubmissionV1, ReserveReviewNoteCandidateSubmissionOutcomeV1,
    ReserveReviewNoteCandidateSubmissionV1, ReviewNoteCandidateBlobCleanupV1,
    ReviewNoteCandidateBlobReceiptV1, ReviewNoteCandidateDecisionOutcomeV1,
    ReviewNoteCandidateInboxOutcomeV1, ReviewNoteCandidateOutboxRecordV1,
    ReviewNoteCandidatePageV1, ReviewNoteCandidatePersistenceErrorV1,
    ReviewNoteCandidateRealtimeTransitionV1,
};
pub use repository::ReviewNoteCandidatePersistenceV1;
pub use schema::{
    REVIEW_NOTE_CANDIDATE_SCHEMA_V1, REVIEW_NOTE_CANDIDATE_STORAGE_BUNDLE_REVISION_V1,
    REVIEW_NOTE_CANDIDATE_STORAGE_BUNDLE_REVISION_V2, review_note_candidate_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-review-note-candidate-persistence";
