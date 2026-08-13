#![forbid(unsafe_code)]
mod admission;
mod client;
mod execution;
mod managed_runtime;
pub use admission::{consistency_module_descriptor_v1, consistency_settings_schema_bytes_v1};
pub use client::dispatch_consistency_client_request_v1;
pub use execution::{
    ConsistencyExecutionContextV1, ConsistencyExecutionErrorV1, ConsistencySourceV1,
    process_consistency_source_event_v1,
};
pub use managed_runtime::{
    ConsistencyManagedRuntimeErrorV1, ConsistencyManagedRuntimeV1, ConsistencyRuntimeAdmissionV1,
};
pub const PACKAGE: &str = "makosh-consistency-runtime";
