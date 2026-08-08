#![forbid(unsafe_code)]

#[cfg(feature = "conformance-test-support")]
mod conformance;
mod model;
mod outbox;
mod repository;
pub mod schema;

#[cfg(feature = "conformance-test-support")]
pub use conformance::ReviewedTaskCandidatePromotionPersistenceConformanceV1;
pub use model::{
    PersistPromotionApprovalOutcomeV1, PersistPromotionApprovalV1, PersistPromotionResultOutcomeV1,
    PersistPromotionTerminalResultV1, PromotionCorrelationV1,
    ReviewedTaskCandidatePromotionOutcomeV1, UnpublishedPromotionEventV1,
};
pub use repository::ReviewedTaskCandidatePromotionPersistenceV1;

pub const PACKAGE: &str = "makosh-reviewed-task-candidate-promotion-persistence";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedTaskCandidatePromotionPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    ApprovalConflict,
    ResultConflict,
    OutboxConflict,
    NotFound,
}
