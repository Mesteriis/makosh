#![forbid(unsafe_code)]
mod admission;
mod client;
mod execution;
mod managed_runtime;
pub use admission::{timeline_module_descriptor_v1, timeline_settings_schema_bytes_v1};
pub use client::dispatch_timeline_client_request_v1;
pub use execution::{
    TimelineExecutionContextV1, TimelineExecutionErrorV1, TimelineSourceV1,
    process_timeline_source_event_v1,
};
pub use managed_runtime::{
    TimelineManagedRuntimeErrorV1, TimelineManagedRuntimeV1, TimelineRuntimeAdmissionV1,
};
pub const PACKAGE: &str = "makosh-timeline-runtime";
