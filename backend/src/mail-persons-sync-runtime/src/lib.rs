#![forbid(unsafe_code)]

mod account_binding;
mod admission;
mod dispatch;
mod execution;
mod inbound;
mod managed_runtime;
mod page;
mod persons_terminal;
mod scheduler;
mod transport;

pub use account_binding::{
    MailPersonsSyncAccountBindingContextV1, MailPersonsSyncAccountBindingErrorV1,
    consume_mail_person_source_account_lifecycle_once_v1,
};
pub use admission::{
    MAIL_PERSONS_SYNC_MODULE_ID_V1, MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1,
    mail_persons_sync_module_descriptor_v1, mail_persons_sync_settings_schema_bytes_v1,
    mail_persons_sync_settings_schema_v1,
};
pub use dispatch::{MailPersonSourceInputV1, dispatch_mail_person_source_v1};
pub use execution::{
    MailPersonsSyncExecutionContextV1, MailPersonsSyncExecutionErrorV1,
    consume_mail_person_source_once_v1,
};
pub use managed_runtime::{
    MailPersonsSyncManagedRuntimeErrorV1, MailPersonsSyncManagedRuntimeV1,
    MailPersonsSyncRuntimeAdmissionV1,
};
pub use page::{
    MailPersonsSyncPageContextV1, MailPersonsSyncPageErrorV1,
    consume_mail_person_source_page_once_v1,
};
pub use persons_terminal::{
    MailPersonsSyncPersonsTerminalContextV1, MailPersonsSyncPersonsTerminalErrorV1,
    MailPersonsSyncPersonsTerminalKindV1, consume_mail_persons_sync_persons_terminal_once_v1,
};
pub use scheduler::{
    MailPersonsSyncSchedulerContextV1, MailPersonsSyncSchedulerErrorV1,
    consume_mail_persons_sync_due_once_v1, decode_account_scope_v1, encode_account_scope_v1,
};
pub use transport::{
    MailPersonsSyncEnvelopeContextV1, MailPersonsSyncEnvelopeErrorV1,
    build_persons_command_outbox_record_v1, source_runtime_public_id_v1,
};

pub const PACKAGE: &str = "makosh-mail-persons-sync-runtime";
