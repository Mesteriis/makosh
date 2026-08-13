#![forbid(unsafe_code)]

mod admission;
mod client;
mod event_outbox;
mod managed_runtime;

pub use admission::{
    projects_module_descriptor_v1, projects_settings_schema_bytes_v1, projects_settings_schema_v1,
};
pub use client::{ProjectsClientRuntimeContextV1, dispatch_projects_client_request_v1};
pub use managed_runtime::{
    ProjectsManagedRuntimeErrorV1, ProjectsManagedRuntimeV1, ProjectsRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-projects-runtime";
