#![forbid(unsafe_code)]

mod admission;
mod blob;
mod command;
mod event_outbox;
mod managed_runtime;

pub use admission::{
    KNOWLEDGE_STORAGE_CAPABILITY_ID_V1, knowledge_module_descriptor_v1,
    knowledge_settings_schema_bytes_v1, knowledge_settings_schema_v1,
};
pub use managed_runtime::{
    KnowledgeManagedRuntimeErrorV1, KnowledgeManagedRuntimeV1, KnowledgeRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-knowledge-runtime";
