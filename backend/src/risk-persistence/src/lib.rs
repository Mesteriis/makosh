#![forbid(unsafe_code)]
mod repository;
mod schema;
pub use repository::*;
pub use schema::{RISK_SCHEMA_V1, risk_storage_bundle_v1};
pub const PACKAGE: &str = "makosh-risk-persistence";
