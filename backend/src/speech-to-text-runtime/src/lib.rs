#![forbid(unsafe_code)]

pub mod admission;
mod managed_ports;
mod managed_runtime;
mod worker;

pub use admission::{
    SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1, SPEECH_TO_TEXT_STORAGE_CAPABILITY_ID_V1,
    speech_to_text_module_descriptor_v1, speech_to_text_settings_schema_bytes_v1,
    speech_to_text_settings_schema_v1,
};
pub use managed_ports::ManagedSpeechToTextExecutionPortsV1;
pub use managed_runtime::{
    SpeechToTextManagedRuntimeErrorV1, SpeechToTextManagedRuntimeV1, SpeechToTextRuntimeAdmissionV1,
};
pub use worker::{
    SpeechToTextExecutionPortsV1, SpeechToTextResponseBlobTargetV1, SpeechToTextWorkerErrorV1,
    execute_speech_to_text_payload_v1,
};

pub const PACKAGE: &str = "makosh-speech-to-text-runtime";
