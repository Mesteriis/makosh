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
    COMMUNICATION_TRANSLATION_BLOB_CAPABILITY_ID_V1,
    COMMUNICATION_TRANSLATION_INFERENCE_CAPABILITY_ID_V1,
    COMMUNICATION_TRANSLATION_STORAGE_CAPABILITY_ID_V1,
    communication_translation_module_descriptor_v1,
    communication_translation_settings_schema_bytes_v1,
    communication_translation_settings_schema_v1,
};
pub use blob_materialization::{
    CommunicationTranslationBlobErrorV1, CommunicationTranslationSourceBlobReceiptV1,
};
pub use client_port::{
    get_communication_translation_payload_v1, start_communication_translation_payload_v1,
};
pub use event_outbox::{
    CommunicationTranslationEventRelayErrorV1, relay_source_prepare_outbox_once_v1,
};
pub use inference::{
    CommunicationTranslationInferenceErrorV1, complete_communication_translation_inference_v1,
    recover_accepted_communication_translation_once_v1,
};
pub use managed_runtime::{
    CommunicationTranslationManagedRuntimeErrorV1, CommunicationTranslationManagedRuntimeV1,
    CommunicationTranslationRuntimeAdmissionV1,
};
pub use source_results::{
    CommunicationTranslationSourceResultErrorV1, consume_translation_source_prepared_once_v1,
    consume_translation_source_rejected_once_v1,
};

pub const PACKAGE: &str = "makosh-communication-translation-runtime";
