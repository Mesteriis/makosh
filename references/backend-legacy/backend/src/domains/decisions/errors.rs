use thiserror::Error;

use crate::domains::decisions::extraction::errors::DecisionEngineError;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum DecisionStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be a JSON array")]
    InvalidJsonArray(&'static str),

    #[error("{0} must be between 0.0 and 1.0: {1}")]
    InvalidScore(&'static str, f64),

    #[error("decision evidence is required")]
    MissingEvidence,

    #[error("observation decision evidence must use the same source_id and observation_id")]
    InvalidObservationEvidenceSource,

    #[error("decision evidence observation was not found: {0}")]
    ObservationNotFound(String),

    #[error("decision was not found")]
    DecisionNotFound,

    #[error("limit must be between 1 and 100")]
    InvalidLimit,

    #[error("decided_by entity kind and id must be provided together")]
    PartialDecider,

    #[error("unknown decision entity kind stored in database: {0}")]
    UnknownEntityKind(String),

    #[error("unknown decision evidence source kind stored in database: {0}")]
    UnknownEvidenceSourceKind(String),

    #[error("unknown decision status stored in database: {0}")]
    UnknownStatus(String),

    #[error("decision API write failed: {0}")]
    Write(String),

    #[error("unknown decision review state stored in database: {0}")]
    UnknownReviewState(String),

    #[error(transparent)]
    DecisionEngine(#[from] DecisionEngineError),
}
