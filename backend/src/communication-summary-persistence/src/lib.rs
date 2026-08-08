#![forbid(unsafe_code)]

mod model;
mod outbox;
mod realtime;
mod repository;
mod schema;

pub use model::{
    CommunicationSummaryBlobCleanupV1, CommunicationSummaryInboxResultV1,
    CommunicationSummaryPersistenceErrorV1, CommunicationSummarySourceResultV1,
    CreateCommunicationSummaryOutcomeV1, CreateCommunicationSummaryRunV1,
    PersistedCommunicationSummaryRunV1, UnpublishedCommunicationSummaryEventV1,
};
pub use realtime::CommunicationSummaryRealtimeTransitionV1;
pub use repository::CommunicationSummaryPersistenceV1;
pub use schema::{
    COMMUNICATION_SUMMARY_SCHEMA_V1, COMMUNICATION_SUMMARY_STORAGE_BUNDLE_REVISION_V1,
    communication_summary_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-communication-summary-persistence";
