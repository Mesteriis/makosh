#![forbid(unsafe_code)]

mod model;
mod repository;
mod schema;

pub use model::{
    CalendarLifecycleCommitV1, CalendarLifecycleMutationV1, CalendarLifecycleOperationOutcomeV1,
    CalendarLifecycleOperationV1, CalendarOutboxRecordV1, CalendarPersistenceErrorV1,
    CalendarSchedulerCommitV1, CalendarSchedulerInputOutcomeV1, CalendarSchedulerInputV1,
};
pub use repository::{CalendarOutboxPublishClaimV1, CalendarPersistenceV1};
pub use schema::{
    CALENDAR_SCHEMA_V1, CALENDAR_STORAGE_BUNDLE_REVISION_V1, calendar_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-calendar-persistence";
