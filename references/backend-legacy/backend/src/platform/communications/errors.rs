use makosh_observations_postgres::errors::ObservationStoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderCommunicationMessagePortError {
    #[error("invalid provider communication message request: {0}")]
    InvalidRequest(String),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    EventStore(#[from] makosh_events_postgres::errors::EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] makosh_events_api::EventEnvelopeError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
