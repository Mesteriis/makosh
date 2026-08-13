#![forbid(unsafe_code)]
mod repository;
mod schema;
pub use repository::*;
pub use schema::{CONSISTENCY_SCHEMA_V1, consistency_storage_bundle_v1};
pub const PACKAGE: &str = "makosh-consistency-persistence";
