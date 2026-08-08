//! Mail-owned PostgreSQL persistence for delivery state and Communications outbox.

mod account;
mod attachments;
mod composition;
#[cfg(feature = "conformance-test-support")]
mod conformance;
mod delivery_intent;
mod delivery_intent_inbox;
mod delivery_intent_result_outbox;
mod durable;
mod lifecycle;
mod message_flags;
mod message_location;
mod message_permanent_delete;
mod oauth;
mod operational;
mod provider_location;
mod schema;
mod sync_health;

pub use account::{
    MAIL_ICLOUD_CARDDAV_CREDENTIAL_SCHEMA_V1, MAIL_SCHEMA_V7, MailCredentialBindingV1,
};
pub use attachments::{
    MAIL_SCHEMA_V5, MailAttachmentDispositionV1, MailAttachmentMaterializationV1,
    MailAttachmentSafetyStateV1, MailAttachmentSafetyTransitionV1,
    MailDeliveryAttachmentManifestV1,
};
pub use composition::{MAIL_SCHEMA_V11, MailCompositionPersistenceErrorV1};
#[cfg(feature = "conformance-test-support")]
pub use conformance::MailPersistenceConformanceV1;
pub use delivery_intent::{MAIL_SCHEMA_V18, MailDeliveryRouteLocatorV1};
pub use delivery_intent_inbox::{
    ClaimedMailDeliveryIntentJobV1, MAIL_DELIVERY_INTENT_MAX_ATTEMPTS_V1, MAIL_SCHEMA_V19,
    MAIL_SCHEMA_V20, MailDeliveryIntentAdmissionV1, MailDeliveryIntentInboxOutcomeV1,
    MailDeliveryIntentJobStateV1, MailDeliveryIntentJobV1, MailDeliveryIntentStoreV1,
};
pub use durable::{
    MAIL_SCHEMA_V1, MAIL_SCHEMA_V2, MAIL_SCHEMA_V3, MAIL_SCHEMA_V6,
    MailAttachmentAnchorMappingOutcomeV1, MailAttachmentAnchorMappingV1,
    MailAttachmentBlobAdmissionCompletionV1, MailAttachmentBlobAdmissionStartOutcomeV1,
    MailDeliveryAttemptOutcomeV1, MailDeliveryAttemptV1, MailDeliveryEnqueueOutcomeV1,
    MailDeliveryEnqueueRequestV1, MailDurablePersistence, MailDurablePersistenceError,
    MailQueuedDeliveryV1, MailSmtpDeliveryAttemptStateV1,
};
pub use lifecycle::{MAIL_SCHEMA_V8, MailAccountLifecycleBeginV1};
pub use message_flags::{
    MAIL_SCHEMA_V12, MailMessageFlagPersistenceErrorV1, MailQueuedMessageFlagCommandV1,
};
pub use message_location::{
    MAIL_SCHEMA_V15, MailMessageLocationPersistenceErrorV1, MailMessageLocationReconciliationV1,
    MailQueuedMessageLocationCommandV1,
};
pub use message_permanent_delete::{
    MAIL_SCHEMA_V17, MailMessagePermanentDeletePersistenceErrorV1,
    MailMessagePermanentDeleteTargetV1, MailQueuedMessagePermanentDeleteCommandV1,
};
pub use oauth::{
    GmailOAuthAttemptStartV1, GmailOAuthCredentialBindingV1, GmailOAuthEnqueueOutcomeV1,
    GmailOAuthOperationKindV1, GmailOAuthOperationOutcomeV1, GmailOAuthOperationV1,
    GmailOAuthQueuedOperationV1, GmailOAuthStoredAttemptV1, MAIL_SCHEMA_V4, MAIL_SCHEMA_V16,
};
pub use operational::{
    MAIL_SCHEMA_V9, MailOperationalFolderSnapshotV1, MailOperationalMaterializationV1,
    MailOperationalMessageSnapshotV1,
};
pub use provider_location::{
    MAIL_SCHEMA_V13, MAIL_SCHEMA_V14, MailImapMessageLocatorV1, initial_imap_message_id,
};
pub use schema::{
    MAIL_ICLOUD_CARDDAV_CREDENTIAL_STORAGE_BUNDLE_REVISION_V1, MAIL_STORAGE_BUNDLE_REVISION_V1,
    MAIL_STORAGE_BUNDLE_REVISION_V2, MAIL_STORAGE_BUNDLE_REVISION_V3,
    MAIL_STORAGE_BUNDLE_REVISION_V4, MAIL_STORAGE_BUNDLE_REVISION_V5,
    MAIL_STORAGE_BUNDLE_REVISION_V6, MAIL_STORAGE_BUNDLE_REVISION_V7,
    MAIL_STORAGE_BUNDLE_REVISION_V8, MAIL_STORAGE_BUNDLE_REVISION_V9,
    MAIL_STORAGE_BUNDLE_REVISION_V10, MAIL_STORAGE_BUNDLE_REVISION_V11,
    MAIL_STORAGE_BUNDLE_REVISION_V12, MAIL_STORAGE_BUNDLE_REVISION_V13,
    MAIL_STORAGE_BUNDLE_REVISION_V14, MAIL_STORAGE_BUNDLE_REVISION_V15,
    MAIL_STORAGE_BUNDLE_REVISION_V16, MAIL_STORAGE_BUNDLE_REVISION_V17,
    MAIL_STORAGE_BUNDLE_REVISION_V18, MAIL_STORAGE_BUNDLE_REVISION_V19,
    MAIL_STORAGE_BUNDLE_REVISION_V20, MAIL_STORAGE_BUNDLE_REVISION_V22,
    MAIL_SYNC_DEADLINE_FAILURE_SCHEMA_V1, MAIL_SYNC_DEADLINE_FAILURE_STORAGE_BUNDLE_REVISION_V1,
    MailIcloudCardDavCredentialSchemaErrorV1, MailSyncDeadlineFailureSchemaErrorV1,
    append_mail_icloud_carddav_credential_storage_v1, append_mail_sync_deadline_failure_storage_v1,
    mail_storage_bundle_v1,
};
pub use sync_health::{MAIL_SCHEMA_V10, MailSyncRunStartOutcomeV1};

pub const PACKAGE: &str = "makosh-mail-persistence";
