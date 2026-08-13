#![forbid(unsafe_code)]

mod admission;
mod consumer;
mod execution;
mod managed_runtime;

pub use admission::{
    identity_resolution_module_descriptor_v1, identity_resolution_settings_schema_bytes_v1,
};
pub use consumer::consume_persons_identity_evidence_once_v1;
pub use execution::{
    IdentityResolutionExecutionContextV1, IdentityResolutionExecutionErrorV1,
    process_persons_identity_evidence_v1,
};
pub use managed_runtime::{
    IdentityResolutionManagedRuntimeErrorV1, IdentityResolutionManagedRuntimeV1,
    IdentityResolutionRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "makosh-identity-resolution-runtime";
