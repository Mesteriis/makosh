#![forbid(unsafe_code)]

mod admission;
mod client;
mod event_outbox;
mod managed_runtime;

pub use admission::{
    decisions_module_descriptor_v1, decisions_settings_schema_bytes_v1,
    decisions_settings_schema_v1,
};
pub use client::{DecisionsClientRuntimeContextV1, dispatch_decisions_client_request_v1};
pub use managed_runtime::{
    DecisionsManagedRuntimeErrorV1, DecisionsManagedRuntimeV1, DecisionsRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-decisions-runtime";
