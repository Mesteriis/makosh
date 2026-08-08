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
    COMMUNICATION_SUMMARY_BLOB_CAPABILITY_ID_V1, COMMUNICATION_SUMMARY_INFERENCE_CAPABILITY_ID_V1,
    COMMUNICATION_SUMMARY_STORAGE_CAPABILITY_ID_V1, communication_summary_module_descriptor_v1,
    communication_summary_settings_schema_bytes_v1, communication_summary_settings_schema_v1,
};
pub use blob_materialization::{
    CommunicationSummaryBlobErrorV1, CommunicationSummarySourceBlobReceiptV1,
};
pub use client_port::{
    get_communication_summary_payload_v1, start_communication_summary_payload_v1,
};
pub use event_outbox::{
    CommunicationSummaryEventRelayErrorV1, relay_source_prepare_outbox_once_v1,
};
pub use inference::{
    CommunicationSummaryInferenceErrorV1, complete_communication_summary_inference_v1,
    recover_accepted_communication_summary_once_v1,
};
pub use managed_runtime::{
    CommunicationSummaryManagedRuntimeErrorV1, CommunicationSummaryManagedRuntimeV1,
    CommunicationSummaryRuntimeAdmissionV1,
};
pub use source_results::{
    CommunicationSummarySourceResultErrorV1, consume_summary_source_prepared_once_v1,
    consume_summary_source_rejected_once_v1,
};

pub const PACKAGE: &str = "makosh-communication-summary-runtime";
