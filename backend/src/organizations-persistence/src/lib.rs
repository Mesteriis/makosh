#![forbid(unsafe_code)]

mod model;
mod repository;
mod schema;

pub use model::{
    OrganizationLifecycleCommitV1, OrganizationLifecycleMutationV1,
    OrganizationLifecycleOperationOutcomeV1, OrganizationLifecycleOperationV1,
    OrganizationOutboxRecordV1, OrganizationsPersistenceErrorV1,
};
pub use repository::{OrganizationOutboxPublishClaimV1, OrganizationsPersistenceV1};
pub use schema::{
    ORGANIZATIONS_SCHEMA_V1, ORGANIZATIONS_STORAGE_BUNDLE_REVISION_V1,
    organizations_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-organizations-persistence";
