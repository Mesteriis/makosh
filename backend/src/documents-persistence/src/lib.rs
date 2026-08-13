#![forbid(unsafe_code)]

mod model;
mod repository;
mod schema;

pub use model::{
    CompleteDocumentBlobOperationV1, DocumentBlobOperationKindV1,
    DocumentBlobOperationStartOutcomeV1, DocumentBlobOperationStartV1, DocumentBoundBlobCustodyV1,
    DocumentLifecycleCommitV1, DocumentLifecycleMutationV1, DocumentLifecycleOperationOutcomeV1,
    DocumentLifecycleOperationV1, DocumentOutboxRecordV1, DocumentsPersistenceErrorV1,
};
pub use repository::{DocumentOutboxPublishClaimV1, DocumentsPersistenceV1};
pub use schema::{
    DOCUMENTS_SCHEMA_V1, DOCUMENTS_STORAGE_BUNDLE_REVISION_V1, documents_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-documents-persistence";
