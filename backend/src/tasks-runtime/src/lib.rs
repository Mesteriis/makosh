#![forbid(unsafe_code)]

mod admission;
mod blob;
mod client;
mod command;
mod event_outbox;
mod managed_runtime;

pub use admission::{
    TASKS_STORAGE_CAPABILITY_ID_V1, tasks_module_descriptor_v1, tasks_settings_schema_bytes_v1,
    tasks_settings_schema_v1,
};
pub use client::dispatch_tasks_client_request_v1;
pub use managed_runtime::{
    TasksManagedRuntimeErrorV1, TasksManagedRuntimeV1, TasksRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-tasks-runtime";
