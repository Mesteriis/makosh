use makosh_events_api::EventEnvelopeError;
use makosh_observations_api::errors::ObservationValidationError;
use thiserror::Error;

use makosh_events_postgres::errors::EventStoreError;

#[derive(Debug, Error)]
pub enum ObservationStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be between 0.0 and 1.0: {1}")]
    InvalidScore(&'static str, f64),

    #[error("unknown observation origin kind stored in database: {0}")]
    UnknownOriginKind(String),

    #[error("unknown observation ingestion run status stored in database: {0}")]
    UnknownIngestionRunStatus(String),

    #[error("observation kind definition was not found: {0}")]
    ObservationKindNotFound(String),
}

impl From<ObservationValidationError> for ObservationStoreError {
    fn from(error: ObservationValidationError) -> Self {
        match error {
            ObservationValidationError::EmptyField(field) => Self::EmptyField(field),
            ObservationValidationError::InvalidJsonObject(field) => Self::InvalidJsonObject(field),
            ObservationValidationError::InvalidScore(field, value) => {
                Self::InvalidScore(field, value)
            }
            ObservationValidationError::UnknownOriginKind(value) => Self::UnknownOriginKind(value),
            ObservationValidationError::UnknownIngestionRunStatus(value) => {
                Self::UnknownIngestionRunStatus(value)
            }
        }
    }
}
