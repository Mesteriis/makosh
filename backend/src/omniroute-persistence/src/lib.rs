#![forbid(unsafe_code)]
mod repository;
mod schema;
pub use repository::*;
pub use schema::{OMNIROUTE_SCHEMA_V1, omniroute_storage_bundle_v1};
pub const PACKAGE: &str = "makosh-omniroute-persistence";
