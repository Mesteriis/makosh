use makosh_events_api::EventEnvelopeError;
use thiserror::Error;

use makosh_events_postgres::errors::EventStoreError;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum ReviewInboxError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("metadata filter must be a JSON object")]
    InvalidMetadataFilter,

    #[error("{0} must be between 0.0 and 1.0: {1}")]
    InvalidScore(&'static str, f64),

    #[error("review item evidence is required")]
    MissingEvidence,

    #[error("review item evidence observation was not found: {0}")]
    ObservationNotFound(String),

    #[error("review item was not found: {0}")]
    ReviewItemNotFound(String),

    #[error("unknown review item kind stored in database: {0}")]
    UnknownItemKind(String),

    #[error("unknown review item status stored in database: {0}")]
    UnknownStatus(String),
}
