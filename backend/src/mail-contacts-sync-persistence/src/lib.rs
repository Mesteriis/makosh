#![forbid(unsafe_code)]

#[cfg(feature = "conformance-test-support")]
mod conformance;
mod model;
mod orchestration;
mod realtime;
mod relay;
mod repository;
mod reverse_model;
mod reverse_sync;
mod scheduled_completion;
mod schema;

#[cfg(feature = "conformance-test-support")]
pub use conformance::MailContactsSyncPersistenceConformanceV1;
pub use model::{
    AcceptScheduledMailContactsSyncDueOutcomeV1, AcceptScheduledMailContactsSyncDueV1,
    AdvanceMailContactsSyncPageV1, CreateMailContactsSyncOutcomeV1, CreateMailContactsSyncRunV1,
    MailContactsSyncAdvanceOutcomeV1, MailContactsSyncContactOutcomeV1,
    MailContactsSyncEntryInputV1, MailContactsSyncEntryOutcomeInputV1,
    MailContactsSyncInboxOutcomeV1, MailContactsSyncPageProgressV1,
    MailContactsSyncPageResultInputV1, MailContactsSyncPersistenceErrorV1,
    MailContactsSyncPersistenceOutcomeV1, MailContactsSyncRealtimeTransitionV1,
    MailContactsSyncScheduledTerminalOutcomeV1, MailContactsSyncTransitionInputV1,
    OutboxEnvelopeV1, PendingMailContactsSyncScheduledTerminalV1, PersistedMailContactsSyncRunV1,
    QueueMailContactsSyncScheduledTerminalV1,
};
pub use repository::MailContactsSyncPersistenceV1;
pub use reverse_model::{
    AcceptContactChangedForMailSyncOutcomeV1, AcceptContactChangedForMailSyncV1,
    CompleteContactMailSyncSourceOutcomeV1, CompleteContactMailSyncSourceV1,
    CompleteContactsProviderLinkOutcomeV1, CompleteContactsProviderLinkV1,
    CompleteMailAddressBookUpsertOutcomeV1, CompleteMailAddressBookUpsertV1,
    MailContactsSyncProviderWriteOutcomeV1, MailContactsSyncReverseOperationSeedV1,
    MailContactsSyncReverseOperationV1,
};
pub use schema::{
    MAIL_CONTACTS_SYNC_ORCHESTRATION_SCHEMA_V1,
    MAIL_CONTACTS_SYNC_PROVIDER_LINK_RECONCILIATION_SCHEMA_V1,
    MAIL_CONTACTS_SYNC_REVERSE_ORIGIN_RUN_SCHEMA_V1, MAIL_CONTACTS_SYNC_REVERSE_SCHEMA_V1,
    MAIL_CONTACTS_SYNC_SCHEDULER_COMPLETION_SCHEMA_V1, MAIL_CONTACTS_SYNC_SCHEMA_V1,
    MAIL_CONTACTS_SYNC_STORAGE_BUNDLE_REVISION_V1, mail_contacts_sync_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-mail-contacts-sync-persistence";
