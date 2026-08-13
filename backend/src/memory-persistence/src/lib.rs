#![forbid(unsafe_code)]
mod repository;
mod schema;
pub use repository::*;
pub use schema::{MEMORY_SCHEMA_V1, memory_storage_bundle_v1};
pub const PACKAGE: &str = "makosh-memory-persistence";
