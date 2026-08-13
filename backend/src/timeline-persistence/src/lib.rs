#![forbid(unsafe_code)]
mod repository;
mod schema;
pub use repository::*;
pub use schema::{TIMELINE_SCHEMA_V1, timeline_storage_bundle_v1};
pub const PACKAGE: &str = "makosh-timeline-persistence";
