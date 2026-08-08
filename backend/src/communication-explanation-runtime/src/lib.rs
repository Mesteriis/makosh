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
    COMMUNICATION_EXPLANATION_BLOB_CAPABILITY_ID_V1,
    COMMUNICATION_EXPLANATION_INFERENCE_CAPABILITY_ID_V1,
    COMMUNICATION_EXPLANATION_STORAGE_CAPABILITY_ID_V1,
    communication_explanation_module_descriptor_v1,
    communication_explanation_settings_schema_bytes_v1,
    communication_explanation_settings_schema_v1,
};
pub use blob_materialization::{
    CommunicationExplanationBlobErrorV1, CommunicationExplanationSourceBlobReceiptV1,
};
pub use client_port::{
    get_communication_explanation_payload_v1, start_communication_explanation_payload_v1,
};
pub use event_outbox::{
    CommunicationExplanationEventRelayErrorV1, relay_source_prepare_outbox_once_v1,
};
pub use inference::{
    CommunicationExplanationInferenceErrorV1, complete_communication_explanation_inference_v1,
    recover_accepted_communication_explanation_once_v1,
};
pub use managed_runtime::{
    CommunicationExplanationManagedRuntimeErrorV1, CommunicationExplanationManagedRuntimeV1,
    CommunicationExplanationRuntimeAdmissionV1,
};
pub use source_results::{
    CommunicationExplanationSourceResultErrorV1, consume_explanation_source_prepared_once_v1,
    consume_explanation_source_rejected_once_v1,
};

pub const PACKAGE: &str = "makosh-communication-explanation-runtime";
