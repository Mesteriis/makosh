#![forbid(unsafe_code)]

mod model;
mod repository;
mod schema;

#[cfg(feature = "conformance-test-support")]
mod conformance;

#[cfg(feature = "conformance-test-support")]
pub use conformance::{
    MailPersonsSyncAccountLifecycleEvidenceV1, MailPersonsSyncPersistenceConformanceV1,
    MailPersonsSyncRlsEvidenceV1,
};

pub use model::{
    ApplyMailPersonsSyncAccountLifecycleV1, BeginMailPersonsSyncRunV1,
    CompleteMailPersonsSyncPageV1, MAIL_PERSONS_SYNC_MAX_ENVELOPE_BYTES_V1,
    MAIL_PERSONS_SYNC_OUTBOX_READ_LIMIT_V1, MailPersonsSyncAccountLifecycleKindV1,
    MailPersonsSyncEnvelopeRecordV1, MailPersonsSyncExpiredRunContextV1,
    MailPersonsSyncOutboxRecordV1, MailPersonsSyncPageContinuationV1,
    MailPersonsSyncPageFinalizationContextV1, MailPersonsSyncReplayOutcomeV1,
    MailPersonsSyncRunContextV1, MailPersonsSyncScheduleControlOutboxRecordV1,
    MailPersonsSyncSemanticKindV1, MailPersonsSyncSourceCommandContextV1,
    MailPersonsSyncStoredRejectCodeV1, RecordMailPersonsSyncPersonsTerminalV1,
    RejectMailPersonsSyncAccountBusyV1, StageMailPersonsSyncSourceV1, StagedSourceV1,
    mail_persons_sync_semantic_order_key_v1, validate_page_promotion_v1,
};
pub use repository::MailPersonsSyncPersistenceV1;
pub use schema::{
    MAIL_PERSONS_SYNC_ACCOUNT_SCHEDULER_BINDING_SCHEMA_V1, MAIL_PERSONS_SYNC_INITIAL_SCHEMA_V1,
    MAIL_PERSONS_SYNC_STORAGE_BUNDLE_REVISION_V1, mail_persons_sync_storage_bundle_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncPersistenceErrorV1 {
    InvalidInput,
    StorageUnavailable,
    CommandConflict,
    AccountBusy,
    PageIncomplete,
    StateConflict,
    HashMismatch,
}

pub const PACKAGE: &str = "makosh-mail-persons-sync-persistence";
