use thiserror::Error;

use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum ObligationStoreError {
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

    #[error("obligation evidence is required")]
    MissingEvidence,

    #[error("observation obligation evidence must use the same source_id and observation_id")]
    InvalidObservationEvidenceSource,

    #[error("obligation evidence observation was not found: {0}")]
    ObservationNotFound(String),

    #[error("obligation was not found")]
    ObligationNotFound,

    #[error("beneficiary entity kind and id must be provided together")]
    PartialBeneficiary,

    #[error("unknown obligation entity kind stored in database: {0}")]
    UnknownEntityKind(String),

    #[error("unknown obligation evidence source kind stored in database: {0}")]
    UnknownEvidenceSourceKind(String),

    #[error("unknown obligation status stored in database: {0}")]
    UnknownStatus(String),

    #[error("unknown obligation review state stored in database: {0}")]
    UnknownReviewState(String),

    #[error("unknown obligation risk state stored in database: {0}")]
    UnknownRiskState(String),

    #[error("obligation API write failed: {0}")]
    Write(String),
}
