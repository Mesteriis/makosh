#![forbid(unsafe_code)]

mod model;
mod repository;
mod schema;

pub use model::*;
pub use repository::DesktopCallRecordingRepositoryV1;
pub use schema::{SCHEMA_V1, STORAGE_BUNDLE_REVISION_V1, desktop_call_recording_storage_bundle_v1};

pub const PACKAGE: &str = "makosh-desktop-call-recording-persistence";
