#![forbid(unsafe_code)]
mod repository;
mod schema;
pub use repository::*;
pub use schema::{GRAPH_SCHEMA_V1, graph_storage_bundle_v1};
pub const PACKAGE: &str = "makosh-graph-persistence";
