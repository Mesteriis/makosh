use makosh_events_api::EventEnvelopeError;
use thiserror::Error;

use makosh_events_postgres::errors::EventStoreError;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum DocumentProcessingError {
    #[error("document processing limit must be between 1 and 100")]
    InvalidLimit,

    #[error("field must not be empty: {0}")]
    EmptyField(&'static str),

    #[error("document processing job not found")]
    JobNotFound,

    #[error("document processing retry requires a failed job")]
    RetryRequiresFailedJob,

    #[error("document processing retry command conflicts with existing event")]
    RetryCommandConflict,

    #[error("document not found")]
    DocumentNotFound,

    #[error("invalid document kind")]
    InvalidStep(String),

    #[error("invalid step value")]
    InvalidStatus(String),

    #[error("invalid artifact kind")]
    InvalidArtifactKind(String),

    #[error("missing document source text")]
    MissingSourceText,

    #[error("OCR backend is not available")]
    OcrBackendUnavailable,

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
