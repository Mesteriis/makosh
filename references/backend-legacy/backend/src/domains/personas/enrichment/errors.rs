use thiserror::Error;

use crate::engines::trust::errors::TrustEngineError;
use makosh_events_postgres::errors::EventStoreError;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum PersonaEnrichmentError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Trust(#[from] TrustEngineError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Event(#[from] EventStoreError),

    #[error("persona not found")]
    NotFound,
}
