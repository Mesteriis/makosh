#![forbid(unsafe_code)]

mod model;
mod repository;
mod schema;

pub use model::{
    ProjectLifecycleCommitV1, ProjectLifecycleMutationV1, ProjectLifecycleOperationOutcomeV1,
    ProjectLifecycleOperationV1, ProjectOutboxRecordV1, ProjectsPersistenceErrorV1,
};
pub use repository::{ProjectOutboxPublishClaimV1, ProjectsPersistenceV1};
pub use schema::{
    PROJECTS_SCHEMA_V1, PROJECTS_STORAGE_BUNDLE_REVISION_V1, projects_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-projects-persistence";
