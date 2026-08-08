#![forbid(unsafe_code)]

mod settings;

pub use settings::{
    OLLAMA_AI_SETTINGS_SCHEMA_MAJOR_V1, OLLAMA_AI_SETTINGS_SCHEMA_REVISION_V1,
    OllamaAiRuntimeSettingsV1, OllamaAiSettingsErrorV1, decode_ollama_ai_settings_v1,
    ollama_ai_settings_schema_bytes_v1, ollama_ai_settings_schema_v1,
};

pub const PACKAGE: &str = "makosh-ollama-ai-api";
pub const OLLAMA_OWNER_ID_V1: &str = "ollama";
pub const OLLAMA_AI_MODULE_ID_V1: &str = "makosh-ollama-ai-runtime";
pub const OLLAMA_AI_PROVIDER_CAPABILITY_ID_V1: &str = "ai.provider.generate.v1";
pub const OLLAMA_AI_SUMMARY_CAPABILITY_ID_V1: &str = "ai.provider.summarize.v1";
pub const OLLAMA_AI_TRANSLATION_CAPABILITY_ID_V1: &str = "ai.provider.translate.v1";
pub const OLLAMA_AI_EXPLANATION_CAPABILITY_ID_V1: &str = "ai.provider.explain.v1";
pub const OLLAMA_AI_STORAGE_CAPABILITY_ID_V1: &str = "ollama.ai.storage.v1";
pub const OLLAMA_AI_LOOPBACK_HOST_V1: &str = "127.0.0.1";
pub const OLLAMA_AI_MAX_MODEL_BYTES_V1: usize = 128;
pub const OLLAMA_AI_MAX_TIMEOUT_MILLIS_V1: u64 = 30_000;

#[must_use]
pub fn valid_ollama_model_name_v1(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= OLLAMA_AI_MAX_MODEL_BYTES_V1
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'/')
        })
}
