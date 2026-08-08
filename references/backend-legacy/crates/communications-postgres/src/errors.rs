use thiserror::Error;

use makosh_communications_api::accounts::CommunicationAccountError;
use makosh_communications_api::evidence::CommunicationEvidenceError;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum CommunicationIngestionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Evidence(#[from] CommunicationEvidenceError),

    #[error(transparent)]
    Account(#[from] CommunicationAccountError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),
}
