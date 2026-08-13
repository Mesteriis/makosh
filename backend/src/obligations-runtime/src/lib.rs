#![forbid(unsafe_code)]

mod admission;
mod blob;
mod client;
mod command;
mod event_outbox;
mod managed_runtime;

pub use admission::{
    OBLIGATIONS_STORAGE_CAPABILITY_ID_V1, obligations_module_descriptor_v1,
    obligations_settings_schema_bytes_v1, obligations_settings_schema_v1,
};
pub use client::dispatch_obligations_client_request_v1;
pub use managed_runtime::{
    ObligationsManagedRuntimeErrorV1, ObligationsManagedRuntimeV1, ObligationsRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-obligations-runtime";
