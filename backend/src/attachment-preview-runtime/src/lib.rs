#![forbid(unsafe_code)]
//! Managed Attachment Preview workflow composition.

pub mod admission;
mod blob;
pub mod client_port;
mod client_realtime;
mod contracts;
mod event_decode;
mod outbox;
pub mod renderer;
pub mod runtime;

pub use admission::{
    ATTACHMENT_PREVIEW_BLOB_CAPABILITY_ID_V1, ATTACHMENT_PREVIEW_STORAGE_CAPABILITY_ID_V1,
    attachment_preview_module_descriptor_v1, attachment_preview_settings_schema_bytes_v1,
    attachment_preview_settings_schema_v1,
};
pub use renderer::{AttachmentPreviewRendererRuntimeV1, attachment_preview_renderer_identity_v1};

pub const PACKAGE: &str = "makosh-attachment-preview-runtime";
