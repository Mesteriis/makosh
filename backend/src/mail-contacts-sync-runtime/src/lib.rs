#![forbid(unsafe_code)]

mod admission;
mod client_port;
mod client_realtime;
mod commands;
mod contacts_results;
mod event_outbox;
mod managed_runtime;
mod provider_events;
mod provider_link_results;
mod provider_write_results;
mod reverse_change;
mod run_progress;
mod scheduler_completion;
mod scheduler_due;
mod scheduler_execution;
mod settings;
mod source_results;

pub use admission::{
    MAIL_CONTACTS_SYNC_STORAGE_CAPABILITY_ID_V1, mail_contacts_sync_module_descriptor_v1,
};
pub use client_port::{
    MailContactsSyncClientContextV1, get_mail_contacts_sync_payload_v1,
    start_mail_contacts_sync_payload_v1,
};
pub use contacts_results::{
    MailContactsSyncContactsResultErrorV1, consume_contact_upsert_rejected_once_v1,
    consume_contact_upserted_once_v1,
};
pub use managed_runtime::{
    MailContactsSyncManagedRuntimeErrorV1, MailContactsSyncManagedRuntimeV1,
    MailContactsSyncRuntimeAdmissionV1,
};
pub use provider_events::{
    MailContactsSyncProviderEventErrorV1, MailContactsSyncProviderRuntimeContextV1,
    consume_mail_address_book_entry_once_v1, consume_mail_address_book_page_completed_once_v1,
    consume_mail_address_book_page_rejected_once_v1,
};
pub use run_progress::{MailContactsSyncProgressErrorV1, advance_ready_page_v1};
pub use scheduler_completion::{
    MailContactsSyncScheduledCompletionErrorV1, queue_mail_contacts_sync_terminal_once_v1,
};
pub use scheduler_due::{
    DecodedMailContactsSyncDueCommandV1, MailContactsSyncDueAdapterErrorV1,
    MailContactsSyncDueContractV1, MailContactsSyncDueMessageV1,
    MailContactsSyncDueRuntimeContextV1, MailContactsSyncTerminalReceiptBindingV1,
    build_mail_contacts_sync_terminal_receipt_from_binding_v1,
    build_mail_contacts_sync_terminal_receipt_v1, decode_mail_contacts_sync_due_command_v1,
};
pub use scheduler_execution::{
    MailContactsSyncScheduledExecutionContextV1, MailContactsSyncScheduledExecutionErrorV1,
    MailContactsSyncScheduledExecutionOutcomeV1, process_mail_contacts_sync_due_payload_v1,
};
pub use settings::{
    MailContactsSyncRuntimeSettingsV1, decode_mail_contacts_sync_settings_v1,
    mail_contacts_sync_settings_schema_bytes_v1, mail_contacts_sync_settings_schema_v1,
};

pub const PACKAGE: &str = "makosh-mail-contacts-sync-runtime";

/// Cross-owner durable commands remain bounded while allowing the sequential
/// managed event pumps to recover from a short broker or runtime outage.
pub(crate) const MAIL_CONTACTS_SYNC_COMMAND_DEADLINE_SECONDS_V1: i64 = 300;
