use thiserror::Error;

use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum ConsistencyError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("confidence must be between 0.0 and 1.0: {0}")]
    InvalidConfidence(f64),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be a JSON array or object")]
    InvalidJsonArrayOrObject(&'static str),

    #[error("unknown contradiction source kind stored in database: {0}")]
    UnknownSourceKind(String),

    #[error("unknown contradiction severity stored in database: {0}")]
    UnknownSeverity(String),

    #[error("unknown contradiction review state stored in database: {0}")]
    UnknownReviewState(String),

    #[error("contradiction observation not found: {0}")]
    ObservationNotFound(String),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),
}
