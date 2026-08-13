#![forbid(unsafe_code)]
mod repository;
mod schema;
pub use repository::*;
pub use schema::{ZOOM_SCHEMA_V1, zoom_storage_bundle_v1};
pub const PACKAGE: &str = "makosh-zoom-persistence";
