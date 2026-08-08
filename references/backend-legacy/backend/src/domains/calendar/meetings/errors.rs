use thiserror::Error;

use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum MeetingsError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("not found")]
    NotFound,
}
