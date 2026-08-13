#![forbid(unsafe_code)]

mod model;
mod repository;
mod schema;

#[cfg(feature = "conformance-test-support")]
mod conformance;

#[cfg(feature = "conformance-test-support")]
pub use conformance::{PersonsPersistenceConformanceV1, PersonsRlsEvidenceV1};
pub use model::{
    ApplyPersonsCommandOutcomeV1, ApplyPersonsCommandV1, LoadedPersonsOwnerV1,
    PERSONS_MAX_ENVELOPE_BYTES_V1, PERSONS_OUTBOX_READ_LIMIT_V1, PERSONS_RECOVERY_ROW_LIMIT_V1,
    PersonsCommandCommitV1, PersonsEnvelopeRecordV1, PersonsOutboxRecordV1,
    PersonsPersistenceErrorV1,
};
pub use repository::PersonsPersistenceV1;
pub use schema::{
    PERSONS_DURABLE_SCHEMA_V2, PERSONS_INITIAL_SCHEMA_V1, PERSONS_OUTBOX_ORDER_SCHEMA_V3,
    PERSONS_STORAGE_BUNDLE_REVISION_V1, persons_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-persons-persistence";
