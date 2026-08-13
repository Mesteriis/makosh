#![forbid(unsafe_code)]

mod model;
mod repository;
mod schema;

pub use model::{
    DecisionLifecycleCommitV1, DecisionLifecycleMutationV1, DecisionLifecycleOperationOutcomeV1,
    DecisionLifecycleOperationV1, DecisionOutboxRecordV1, DecisionPendingOutboxV1,
    DecisionsPersistenceErrorV1,
};
pub use repository::{DecisionOutboxPublishClaimV1, DecisionsPersistenceV1};

pub use schema::{
    DECISIONS_SCHEMA_V1, DECISIONS_STORAGE_BUNDLE_REVISION_V1, decisions_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-decisions-persistence";
