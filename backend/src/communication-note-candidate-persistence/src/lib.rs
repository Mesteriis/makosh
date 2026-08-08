#![forbid(unsafe_code)]

mod model;
mod outbox;
mod realtime;
mod repository;
mod schema;

pub use model::{
    CommunicationNoteCandidateBlobCleanupV1, CommunicationNoteCandidateInboxResultV1,
    CommunicationNoteCandidatePersistenceErrorV1, CommunicationNoteCandidateSourceResultV1,
    CreateCommunicationNoteCandidateOutcomeV1, CreateCommunicationNoteCandidateRunV1,
    PersistedCommunicationNoteCandidateRunV1, UnpublishedCommunicationNoteCandidateEventV1,
};
pub use realtime::CommunicationNoteCandidateRealtimeTransitionV1;
pub use repository::CommunicationNoteCandidatePersistenceV1;
pub use schema::{
    COMMUNICATION_NOTE_CANDIDATE_SCHEMA_V1,
    COMMUNICATION_NOTE_CANDIDATE_STORAGE_BUNDLE_REVISION_V1,
    communication_note_candidate_extraction_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-communication-note-candidate-persistence";
