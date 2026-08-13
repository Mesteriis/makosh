#![forbid(unsafe_code)]

mod model;
mod repository;
mod row_codec;
mod schema;

pub use model::{
    CheckReviewObligationCandidateDecisionReplayV1, CompleteReviewObligationCandidateSubmissionV1,
    DecideReviewObligationCandidateOperationV1, ListReviewObligationCandidatesV1,
    PersistReviewObligationCandidateMaterializationV1,
    PersistReviewObligationCandidatePromotionResultV1,
    PersistedReviewObligationCandidateSubmissionV1, REVIEW_OBLIGATION_CANDIDATE_MAX_PAGE_SIZE_V1,
    RejectReviewObligationCandidateSubmissionV1,
    ReserveReviewObligationCandidateSubmissionOutcomeV1,
    ReserveReviewObligationCandidateSubmissionV1, ReviewObligationCandidateBlobCleanupV1,
    ReviewObligationCandidateBlobReceiptV1, ReviewObligationCandidateDecisionOutcomeV1,
    ReviewObligationCandidateInboxOutcomeV1, ReviewObligationCandidateOutboxRecordV1,
    ReviewObligationCandidatePageV1, ReviewObligationCandidatePersistenceErrorV1,
    ReviewObligationCandidateRealtimeTransitionV1,
};
pub use repository::ReviewObligationCandidatePersistenceV1;
pub use schema::{
    REVIEW_OBLIGATION_CANDIDATE_OWNER_RLS_SCHEMA_V2,
    REVIEW_OBLIGATION_CANDIDATE_PARTIES_EVIDENCE_SCHEMA_V3, REVIEW_OBLIGATION_CANDIDATE_SCHEMA_V1,
    REVIEW_OBLIGATION_CANDIDATE_STORAGE_BUNDLE_REVISION_V1,
    REVIEW_OBLIGATION_CANDIDATE_STORAGE_BUNDLE_REVISION_V2,
    REVIEW_OBLIGATION_CANDIDATE_STORAGE_BUNDLE_REVISION_V3,
    review_obligation_candidate_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-review-obligation-candidate-persistence";
