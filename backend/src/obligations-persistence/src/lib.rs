#![forbid(unsafe_code)]

mod lifecycle_repository;
mod model;
mod repository;
mod row_codec;
mod schema;

pub use model::{
    CompleteReviewedCandidateObligationV1, ObligationsBlobCleanupV1, ObligationsBlobReceiptV1,
    ObligationsLifecycleCommitV1, ObligationsLifecycleMutationV1,
    ObligationsLifecycleOperationOutcomeV1, ObligationsLifecycleOperationV1,
    ObligationsOutboxRecordV1, ObligationsPersistenceErrorV1,
    PersistReviewedCandidateMaterializationV1, PersistedReviewedCandidateCommandV1,
    RejectReviewedCandidateObligationV1, ReserveReviewedCandidateCommandOutcomeV1,
    ReserveReviewedCandidateCommandV1,
};
pub use repository::{ObligationsOutboxPublishClaimV1, ObligationsPersistenceV1};
pub use schema::{
    OBLIGATIONS_LIFECYCLE_OWNER_RLS_SCHEMA_V2, OBLIGATIONS_PARTIES_EVIDENCE_SCHEMA_V3,
    OBLIGATIONS_SCHEMA_V1, OBLIGATIONS_STORAGE_BUNDLE_REVISION_V1,
    OBLIGATIONS_STORAGE_BUNDLE_REVISION_V2, OBLIGATIONS_STORAGE_BUNDLE_REVISION_V3,
    obligations_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-obligations-persistence";
