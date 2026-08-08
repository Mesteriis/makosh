use thiserror::Error;

use makosh_events_postgres::errors::EventStoreError;

#[derive(Debug, Error)]
pub enum PersonaTrustError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    RiskEngine(#[from] crate::engines::risk::errors::RiskEngineError),

    #[error(transparent)]
    Observation(#[from] makosh_observations_postgres::errors::ObservationStoreError),

    #[error(transparent)]
    Event(#[from] EventStoreError),
}
