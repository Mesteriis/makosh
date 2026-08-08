use thiserror::Error;

use crate::ai::hub::AiHubError;
use crate::workflows::review_inbox::ReviewInboxWorkflowError;
use makosh_events_postgres::errors::EventStoreError;
use makosh_observations_postgres::errors::ObservationStoreError;

use super::service::attribution_port::AiPersonaAttributionError;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("invalid AI request: {0}")]
    InvalidRequest(&'static str),

    #[error("unknown AI agent `{0}`")]
    UnknownAgent(String),

    #[error("invalid semantic source kind `{0}`")]
    InvalidSourceKind(String),

    #[error("embedding dimension must be {expected}, got {actual}")]
    InvalidEmbeddingDimension { expected: usize, actual: usize },

    #[error("AI run was not found")]
    RunNotFound,

    #[error(transparent)]
    Runtime(#[from] AiHubError),

    #[error(transparent)]
    EventEnvelope(#[from] makosh_events_api::EventEnvelopeError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error("AI persona attribution port was not configured")]
    PersonaAttributionUnavailable,

    #[error(transparent)]
    PersonaAttribution(#[from] AiPersonaAttributionError),

    #[error(transparent)]
    ReviewInboxWorkflow(#[from] ReviewInboxWorkflowError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
