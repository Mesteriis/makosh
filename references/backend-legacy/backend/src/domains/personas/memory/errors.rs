use thiserror::Error;

use crate::engines::memory::errors::MemoryEngineError;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum PersonaMemoryError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Memory(#[from] MemoryEngineError),
    #[error(transparent)]
    Timeline(#[from] crate::engines::timeline::errors::TimelineEngineError),
    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),
    #[error("fact not found")]
    NotFound,
}
