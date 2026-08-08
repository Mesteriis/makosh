#![forbid(unsafe_code)]

mod model;
mod repository;
mod row_codec;
mod schema;

pub use model::{
    CompleteReviewedCandidateTaskV1, PersistReviewedCandidateMaterializationV1,
    PersistedReviewedCandidateCommandV1, RejectReviewedCandidateTaskV1,
    ReserveReviewedCandidateCommandOutcomeV1, ReserveReviewedCandidateCommandV1,
    TasksBlobCleanupV1, TasksBlobReceiptV1, TasksOutboxRecordV1, TasksPersistenceErrorV1,
};
pub use repository::TasksPersistenceV1;
pub use schema::{TASKS_SCHEMA_V1, TASKS_STORAGE_BUNDLE_REVISION_V1, tasks_storage_bundle_v1};

pub const PACKAGE: &str = "makosh-tasks-persistence";
