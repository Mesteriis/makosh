#![forbid(unsafe_code)]

mod repository;
mod schema;

pub use repository::{
    ApplySearchDocumentV1, SearchCursorRecordV1, SearchEnvelopeRecordV1, SearchHitRecordV1,
    SearchPersistenceErrorV1, SearchPersistenceV1, SearchProjectionStatusRecordV1,
    SearchReplayOutcomeV1,
};
pub use schema::{SEARCH_SCHEMA_V1, search_storage_bundle_v1};

pub const PACKAGE: &str = "makosh-search-persistence";
