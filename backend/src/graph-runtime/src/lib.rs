#![forbid(unsafe_code)]
mod admission;
mod client;
mod execution;
mod managed_runtime;
pub use admission::{graph_module_descriptor_v1, graph_settings_schema_bytes_v1};
pub use client::dispatch_graph_client_request_v1;
pub use execution::{
    GraphExecutionContextV1, GraphExecutionErrorV1, GraphSourceV1, process_graph_source_event_v1,
};
pub use managed_runtime::{
    GraphManagedRuntimeErrorV1, GraphManagedRuntimeV1, GraphRuntimeAdmissionV1,
};
pub const PACKAGE: &str = "makosh-graph-runtime";
