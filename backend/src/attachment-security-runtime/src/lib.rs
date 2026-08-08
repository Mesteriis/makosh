//! Managed process root for the owner-neutral Attachment Security engine.

pub mod admission;
mod delegation;
mod event_decode;
mod outbox;
mod preview_delegation;
pub mod runtime;
mod scan;
pub mod settings;
mod text_delegation;

pub use scan::AttachmentSecurityScanAdapterErrorV1;

pub const PACKAGE: &str = "makosh-attachment-security-runtime";
