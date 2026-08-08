#![forbid(unsafe_code)]

mod admission;
mod blob;
mod managed_runtime;
mod resources;
mod settings;
mod worker;

pub use admission::{
    WHISPER_STT_BLOB_CAPABILITY_ID_V1, WHISPER_STT_MODEL_ARTIFACT_ID_V1, WHISPER_STT_MODULE_ID_V1,
    WHISPER_STT_OWNER_ID_V1, WHISPER_STT_RUNNER_ARTIFACT_ID_V1,
    WHISPER_STT_STORAGE_CAPABILITY_ID_V1, whisper_stt_module_descriptor_v1,
};
pub use managed_runtime::{
    WhisperSttManagedRuntimeErrorV1, WhisperSttManagedRuntimeV1, WhisperSttRuntimeAdmissionV1,
};
pub use resources::{
    PreparedWhisperSttResourcesV1, WhisperSttResourcesErrorV1, prepare_whisper_stt_resources_v1,
};
pub use settings::{
    WhisperSttRuntimeSettingsV1, WhisperSttSettingsErrorV1, decode_whisper_stt_settings_v1,
    whisper_stt_settings_schema_bytes_v1, whisper_stt_settings_schema_v1,
};
pub use worker::{
    WhisperSttExecutionPortV1, WhisperSttPortErrorV1, WhisperSttWorkerErrorV1,
    execute_whisper_stt_payload_v1,
};

pub const PACKAGE: &str = "makosh-whisper-stt-runtime";
