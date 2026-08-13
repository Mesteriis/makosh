#![forbid(unsafe_code)]

mod admission;
mod client;
mod event_outbox;
mod managed_runtime;

pub use admission::{
    documents_module_descriptor_v1, documents_settings_schema_bytes_v1,
    documents_settings_schema_v1,
};
pub use client::{
    DocumentsBlobAttachRequestV1, DocumentsBlobCustodyPortV1, DocumentsBlobReceiptV1,
    DocumentsBlobReleaseRequestV1, DocumentsClientRuntimeContextV1,
    dispatch_documents_client_request_v1,
};
pub use managed_runtime::{
    DocumentsManagedRuntimeErrorV1, DocumentsManagedRuntimeV1, DocumentsRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-documents-runtime";
