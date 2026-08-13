#![forbid(unsafe_code)]
mod admission;
mod client;
mod execution;
mod managed_runtime;
pub use admission::{risk_module_descriptor_v1, risk_settings_schema_bytes_v1};
pub use client::dispatch_risk_client_request_v1;
pub use execution::{
    RiskExecutionContextV1, RiskExecutionErrorV1, RiskSourceV1, process_risk_source_event_v1,
};
pub use managed_runtime::{
    RiskManagedRuntimeErrorV1, RiskManagedRuntimeV1, RiskRuntimeAdmissionV1,
};
pub const PACKAGE: &str = "makosh-risk-runtime";
