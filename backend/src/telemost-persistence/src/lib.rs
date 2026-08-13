#![forbid(unsafe_code)]
mod repository;
mod schema;
pub use repository::*;
pub use schema::{TELEMOST_SCHEMA_V1, telemost_storage_bundle_v1};
pub const PACKAGE: &str = "makosh-telemost-persistence";
