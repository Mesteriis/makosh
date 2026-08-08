#![forbid(unsafe_code)]

mod model;
mod outbox;
mod realtime;
mod repository;
mod schema;

pub use model::{
    CommunicationExplanationBlobCleanupV1, CommunicationExplanationInboxResultV1,
    CommunicationExplanationPersistenceErrorV1, CommunicationExplanationSourceResultV1,
    CreateCommunicationExplanationOutcomeV1, CreateCommunicationExplanationRunV1,
    PersistedCommunicationExplanationRunV1, UnpublishedCommunicationExplanationEventV1,
};
pub use realtime::CommunicationExplanationRealtimeTransitionV1;
pub use repository::CommunicationExplanationPersistenceV1;
pub use schema::{
    COMMUNICATION_EXPLANATION_SCHEMA_V1, COMMUNICATION_EXPLANATION_STORAGE_BUNDLE_REVISION_V1,
    communication_explanation_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-communication-explanation-persistence";
