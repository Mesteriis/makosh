use makosh_events_api::EventEnvelopeError;
use thiserror::Error;

use crate::domains::tasks::core::errors::TaskCoreError;
use crate::engines::obligation::errors::ObligationEngineError;
use makosh_events_postgres::errors::EventStoreError;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum TaskCandidateError {
    #[error("limit must be between 1 and 100")]
    InvalidLimit,

    #[error("field must not be empty: {0}")]
    EmptyField(String),

    #[error("task candidate was not found")]
    TaskCandidateNotFound,

    #[error("review_state must be suggested, user_confirmed, or user_rejected")]
    InvalidReviewState(String),

    #[error("payload must be an object")]
    InvalidPayload(String),

    #[error("payload field was missing: {0}")]
    MissingPayloadField(String),

    #[error("candidate metadata is missing or invalid: {0}")]
    InvalidCandidateMetadata(String),

    #[error("actor_id is missing from event")]
    MissingActorId,

    #[error("invalid review event type")]
    InvalidEventType,

    #[error("invalid task candidate source kind: {0}")]
    InvalidSourceKind(String),

    #[error("task candidate requires observation evidence: {0}")]
    ObservationRequired(String),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    ObligationEngine(#[from] ObligationEngineError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    TaskCore(#[from] TaskCoreError),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
