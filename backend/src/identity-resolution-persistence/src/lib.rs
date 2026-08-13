#![forbid(unsafe_code)]

#[cfg(feature = "conformance-test-support")]
mod conformance;
mod repository;
mod schema;

#[cfg(feature = "conformance-test-support")]
pub use conformance::{
    IdentityResolutionDurableCountsV1, IdentityResolutionPersistenceConformanceV1,
    IdentityResolutionRlsEvidenceV1,
};
pub use repository::{
    ApplyIdentityEvidenceOperationV1, IdentityResolutionEnvelopeRecordV1,
    IdentityResolutionOutboxPublishClaimV1, IdentityResolutionOutboxRecordV1,
    IdentityResolutionPersistenceErrorV1, IdentityResolutionPersistenceV1,
    IdentityResolutionReplayOutcomeV1,
};
pub use schema::{IDENTITY_RESOLUTION_SCHEMA_V1, identity_resolution_storage_bundle_v1};

pub const PACKAGE: &str = "makosh-identity-resolution-persistence";
