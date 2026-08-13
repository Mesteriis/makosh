#![forbid(unsafe_code)]

mod admission;
mod client;
mod event_outbox;
mod managed_runtime;

pub use admission::{
    relationships_module_descriptor_v1, relationships_settings_schema_bytes_v1,
    relationships_settings_schema_v1,
};
pub use client::{RelationshipsClientRuntimeContextV1, dispatch_relationships_client_request_v1};
pub use managed_runtime::{
    RelationshipsManagedRuntimeErrorV1, RelationshipsManagedRuntimeV1,
    RelationshipsRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-relationships-runtime";
