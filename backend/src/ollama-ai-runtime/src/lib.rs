#![forbid(unsafe_code)]

pub mod admission;
mod explanation_worker;
mod managed_runtime;
mod summary_worker;
mod translation_worker;
mod worker;

pub use admission::ollama_ai_module_descriptor_v1;
pub use managed_runtime::{
    OllamaAiManagedRuntimeErrorV1, OllamaAiManagedRuntimeV1, OllamaAiRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-ollama-ai-runtime";
