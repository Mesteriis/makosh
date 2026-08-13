#![forbid(unsafe_code)]
mod admission;
mod client;
mod execution;
mod managed_runtime;
pub use admission::{memory_module_descriptor_v1, memory_settings_schema_bytes_v1};
pub use client::dispatch_memory_client_request_v1;
pub use execution::{
    MemoryExecutionContextV1, MemoryExecutionErrorV1, MemorySourceV1,
    process_memory_source_event_v1,
};
pub use managed_runtime::{
    MemoryManagedRuntimeErrorV1, MemoryManagedRuntimeV1, MemoryRuntimeAdmissionV1,
};
pub const PACKAGE: &str = "makosh-memory-runtime";
