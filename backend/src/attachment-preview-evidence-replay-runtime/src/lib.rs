#![forbid(unsafe_code)]
//! Managed owner-authorized retained-evidence replay workflow components.

pub mod admission;
pub mod client_port;
mod contracts;
pub mod managed_runtime;
pub mod outbox;
pub mod result_consumer;

pub use admission::{
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_CAPABILITY_ID_V1,
    attachment_preview_evidence_replay_module_descriptor_v1,
    attachment_preview_evidence_replay_settings_schema_bytes_v1,
    attachment_preview_evidence_replay_settings_schema_v1,
};

pub const PACKAGE: &str = "makosh-attachment-preview-evidence-replay-runtime";
