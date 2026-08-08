#![forbid(unsafe_code)]

mod model;
mod outbox;
mod realtime;
mod repository;
mod schema;

pub use model::{
    CreateReplySuggestionOutcomeV1, CreateReplySuggestionRunV1, PersistedReplySuggestionRunV1,
    ReplySuggestionBlobCleanupV1, ReplySuggestionInboxResultV1, ReplySuggestionPersistenceErrorV1,
    ReplySuggestionSourceResultV1, UnpublishedReplySuggestionEventV1,
};
pub use realtime::ReplySuggestionRealtimeTransitionV1;
pub use repository::CommunicationReplySuggestionPersistenceV1;
pub use schema::{
    COMMUNICATION_REPLY_SUGGESTION_SCHEMA_V1,
    COMMUNICATION_REPLY_SUGGESTION_STORAGE_BUNDLE_REVISION_V1,
    communication_reply_suggestion_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-communication-reply-suggestion-persistence";
