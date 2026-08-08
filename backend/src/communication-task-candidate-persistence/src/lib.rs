#![forbid(unsafe_code)]

mod model;
mod outbox;
mod realtime;
mod repository;
mod schema;

pub use model::{
    CommunicationTaskCandidateBlobCleanupV1, CommunicationTaskCandidateInboxResultV1,
    CommunicationTaskCandidatePersistenceErrorV1, CommunicationTaskCandidateSourceResultV1,
    CreateCommunicationTaskCandidateOutcomeV1, CreateCommunicationTaskCandidateRunV1,
    PersistedCommunicationTaskCandidateRunV1, UnpublishedCommunicationTaskCandidateEventV1,
};
pub use realtime::CommunicationTaskCandidateRealtimeTransitionV1;
pub use repository::CommunicationTaskCandidatePersistenceV1;
pub use schema::{
    COMMUNICATION_TASK_CANDIDATE_SCHEMA_V1,
    COMMUNICATION_TASK_CANDIDATE_STORAGE_BUNDLE_REVISION_V1,
    communication_task_candidate_extraction_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-communication-task-candidate-persistence";
