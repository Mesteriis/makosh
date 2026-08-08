#![forbid(unsafe_code)]

mod admission;
mod blob_materialization;
mod client_port;
mod client_realtime;
mod contracts;
mod event_outbox;
mod inference;
mod managed_runtime;
mod source_results;

pub use admission::{
    COMMUNICATION_REPLY_SUGGESTION_BLOB_CAPABILITY_ID_V1,
    COMMUNICATION_REPLY_SUGGESTION_INFERENCE_CAPABILITY_ID_V1,
    COMMUNICATION_REPLY_SUGGESTION_STORAGE_CAPABILITY_ID_V1,
    communication_reply_suggestion_module_descriptor_v1,
    communication_reply_suggestion_settings_schema_bytes_v1,
    communication_reply_suggestion_settings_schema_v1,
};
pub use blob_materialization::{ReplySuggestionBlobErrorV1, ReplySuggestionSourceBlobReceiptV1};
pub use client_port::{get_reply_suggestion_payload_v1, start_reply_suggestion_payload_v1};
pub use event_outbox::{ReplySuggestionEventRelayErrorV1, relay_source_prepare_outbox_once_v1};
pub use inference::{
    ReplySuggestionInferenceErrorV1, complete_reply_suggestion_inference_v1,
    recover_accepted_reply_suggestion_once_v1,
};
pub use managed_runtime::{
    ReplySuggestionManagedRuntimeErrorV1, ReplySuggestionManagedRuntimeV1,
    ReplySuggestionRuntimeAdmissionV1,
};
pub use source_results::{
    ReplySuggestionSourceResultErrorV1, consume_reply_source_prepared_once_v1,
    consume_reply_source_rejected_once_v1,
};

pub const PACKAGE: &str = "makosh-communication-reply-suggestion-runtime";
