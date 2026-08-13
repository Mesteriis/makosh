#![forbid(unsafe_code)]

mod admission;
mod client;
mod event_outbox;
mod managed_runtime;

pub use admission::{
    organizations_module_descriptor_v1, organizations_settings_schema_bytes_v1,
    organizations_settings_schema_v1,
};
pub use client::{OrganizationsClientRuntimeContextV1, dispatch_organizations_client_request_v1};
pub use managed_runtime::{
    OrganizationsManagedRuntimeErrorV1, OrganizationsManagedRuntimeV1,
    OrganizationsRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-organizations-runtime";
