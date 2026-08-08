#![forbid(unsafe_code)]

pub mod admission;
mod attachment_translation_worker;
mod explanation_worker;
mod managed_ports;
mod managed_runtime;
mod summary_worker;
mod translation_worker;
mod worker;

pub use admission::{
    AI_INFERENCE_STORAGE_CAPABILITY_ID_V1, ai_inference_module_descriptor_v1,
    ai_inference_settings_schema_bytes_v1, ai_inference_settings_schema_v1,
};
pub use managed_runtime::{
    AiInferenceManagedRuntimeErrorV1, AiInferenceManagedRuntimeV1, AiInferenceRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-ai-inference-runtime";
