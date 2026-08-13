#![forbid(unsafe_code)]

mod admission;
mod client;
mod execution;
mod managed_runtime;

pub use admission::{search_module_descriptor_v1, search_settings_schema_bytes_v1};
pub use client::dispatch_search_client_request_v1;
pub use execution::{
    SearchExecutionContextV1, SearchExecutionErrorV1, SearchSourceV1,
    process_search_source_event_v1,
};
pub use managed_runtime::{
    SearchManagedRuntimeErrorV1, SearchManagedRuntimeV1, SearchRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-search-runtime";
