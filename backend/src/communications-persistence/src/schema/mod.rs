//! Immutable Communications-owned schema artifacts.

mod bundle;

pub use bundle::{
    COMMUNICATIONS_BODY_MEDIA_TYPE_STORAGE_BUNDLE_REVISION_V1,
    COMMUNICATIONS_STORAGE_BUNDLE_REVISION_V1, CommunicationsBodyMediaTypeSchemaErrorV1,
    append_communications_body_media_type_storage_v1, communications_storage_bundle_v1,
};

pub const COMMUNICATIONS_SCHEMA_V1: &str =
    include_str!("../../migrations/0001_communications_state.sql");
