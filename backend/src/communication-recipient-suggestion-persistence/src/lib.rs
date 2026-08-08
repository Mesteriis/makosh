#![forbid(unsafe_code)]

mod model;
mod outbox;
mod realtime;
mod repository;
mod schema;

pub use model::{
    CommunicationRecipientSuggestionBlobCleanupV1, CommunicationRecipientSuggestionInboxResultV1,
    CommunicationRecipientSuggestionPersistenceErrorV1,
    CommunicationRecipientSuggestionSourceResultV1,
    CreateCommunicationRecipientSuggestionOutcomeV1, CreateCommunicationRecipientSuggestionRunV1,
    PersistedCommunicationRecipientSuggestionRunV1,
    UnpublishedCommunicationRecipientSuggestionEventV1,
};
pub use realtime::CommunicationRecipientSuggestionRealtimeTransitionV1;
pub use repository::CommunicationRecipientSuggestionPersistenceV1;
pub use schema::{
    COMMUNICATION_RECIPIENT_SUGGESTION_SCHEMA_V1,
    COMMUNICATION_RECIPIENT_SUGGESTION_STORAGE_BUNDLE_REVISION_V1,
    communication_recipient_suggestion_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-communication-recipient-suggestion-persistence";
