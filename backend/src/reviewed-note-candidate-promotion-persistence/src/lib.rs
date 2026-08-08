#![forbid(unsafe_code)]

#[cfg(feature = "conformance-test-support")]
mod conformance;
mod model;
mod outbox;
mod repository;
pub mod schema;

#[cfg(feature = "conformance-test-support")]
pub use conformance::ReviewedNoteCandidatePromotionPersistenceConformanceV1;
pub use model::{
    PersistPromotionApprovalOutcomeV1, PersistPromotionApprovalV1,
    PersistPromotionMaterializationV1, PersistPromotionResultOutcomeV1,
    PersistPromotionTerminalResultV1, PersistPromotionWorkflowFailureV1,
    PersistedPromotionApprovalV1, PromotionBlobReceiptV1, PromotionCorrelationV1,
    REVIEWED_NOTE_CANDIDATE_PROMOTION_MAX_BLOB_BYTES_V1,
    REVIEWED_NOTE_CANDIDATE_PROMOTION_MAX_PROOF_BYTES_V1, ReservePromotionApprovalOutcomeV1,
    ReservePromotionApprovalV1, ReviewedNoteCandidatePromotionOutcomeV1,
    UnpublishedPromotionEventV1,
};
pub use repository::ReviewedNoteCandidatePromotionPersistenceV1;

pub const PACKAGE: &str = "makosh-reviewed-note-candidate-promotion-persistence";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedNoteCandidatePromotionPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    ApprovalConflict,
    ResultConflict,
    OutboxConflict,
    NotFound,
}
