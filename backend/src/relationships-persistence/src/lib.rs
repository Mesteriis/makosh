#![forbid(unsafe_code)]

mod model;
mod repository;
mod schema;

pub use model::{
    RelationshipCommitV1, RelationshipMutationV1, RelationshipOperationOutcomeV1,
    RelationshipOperationV1, RelationshipOutboxRecordV1, RelationshipsPersistenceErrorV1,
};
pub use repository::{RelationshipOutboxPublishClaimV1, RelationshipsPersistenceV1};
pub use schema::{
    RELATIONSHIPS_SCHEMA_V1, RELATIONSHIPS_STORAGE_BUNDLE_REVISION_V1,
    relationships_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-relationships-persistence";
