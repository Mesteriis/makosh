#![forbid(unsafe_code)]

mod admission;
mod blob_materialization;
mod client_port;
mod client_realtime;
mod contracts;
mod evaluation;
mod event_outbox;
mod managed_runtime;
mod source_results;

pub use admission::{
    COMMUNICATION_RECIPIENT_SUGGESTION_BLOB_CAPABILITY_ID_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_STORAGE_CAPABILITY_ID_V1,
    communication_recipient_suggestion_module_descriptor_v1,
    communication_recipient_suggestion_settings_schema_bytes_v1,
    communication_recipient_suggestion_settings_schema_v1,
};
pub use blob_materialization::{
    CommunicationRecipientSuggestionBlobErrorV1,
    CommunicationRecipientSuggestionSourceBlobReceiptV1,
};
pub use client_port::{
    get_communication_recipient_suggestion_payload_v1,
    start_communication_recipient_suggestion_payload_v1,
};
pub use evaluation::{
    CommunicationRecipientSuggestionEvaluationErrorV1,
    complete_communication_recipient_suggestion_evaluation_v1,
    recover_accepted_communication_recipient_suggestion_once_v1,
};
pub use event_outbox::{
    CommunicationRecipientSuggestionEventRelayErrorV1, relay_source_prepare_outbox_once_v1,
};
pub use managed_runtime::{
    CommunicationRecipientSuggestionManagedRuntimeErrorV1,
    CommunicationRecipientSuggestionManagedRuntimeV1,
    CommunicationRecipientSuggestionRuntimeAdmissionV1,
};
pub use source_results::{
    CommunicationRecipientSuggestionSourceResultErrorV1, consume_recipient_source_prepared_once_v1,
    consume_recipient_source_rejected_once_v1,
};

pub const PACKAGE: &str = "makosh-communication-recipient-suggestion-runtime";
