#![forbid(unsafe_code)]

mod model;
mod outbox;
mod realtime;
mod repository;
mod schema;

pub use model::{
    CommunicationTranslationBlobCleanupV1, CommunicationTranslationInboxResultV1,
    CommunicationTranslationPersistenceErrorV1, CommunicationTranslationSourceResultV1,
    CreateCommunicationTranslationOutcomeV1, CreateCommunicationTranslationRunV1,
    PersistedCommunicationTranslationRunV1, UnpublishedCommunicationTranslationEventV1,
};
pub use realtime::CommunicationTranslationRealtimeTransitionV1;
pub use repository::CommunicationTranslationPersistenceV1;
pub use schema::{
    COMMUNICATION_TRANSLATION_SCHEMA_V1, COMMUNICATION_TRANSLATION_STORAGE_BUNDLE_REVISION_V1,
    communication_translation_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-communication-translation-persistence";
