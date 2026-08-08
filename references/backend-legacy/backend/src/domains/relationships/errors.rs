use thiserror::Error;

use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum RelationshipStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be between 0.0 and 1.0: {1}")]
    InvalidScore(&'static str, f64),

    #[error("relationship evidence is required")]
    MissingEvidence,

    #[error("observation relationship evidence must use the same source_id and observation_id")]
    InvalidObservationEvidenceSource,

    #[error("relationship evidence observation was not found: {0}")]
    ObservationNotFound(String),

    #[error("relationship was not found")]
    RelationshipNotFound,

    #[error("relationship endpoints must be distinct")]
    IdenticalEndpoints,

    #[error("relationship valid_to must not be earlier than valid_from")]
    InvalidTemporalRange,

    #[error("unknown relationship entity kind stored in database: {0}")]
    UnknownEntityKind(String),

    #[error("unknown relationship evidence source kind stored in database: {0}")]
    UnknownEvidenceSourceKind(String),

    #[error("unknown relationship review state stored in database: {0}")]
    UnknownReviewState(String),
}
