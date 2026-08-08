use thiserror::Error;

use crate::platform::communications::errors::ProviderCommunicationMessagePortError;
use crate::platform::secrets::errors::{SecretReferenceError, SecretResolutionError};
use crate::vault::errors::HostVaultError;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum WhatsappWebError {
    #[error("invalid WhatsApp Web request: {0}")]
    InvalidRequest(String),

    #[error("WhatsApp Web provider account store operation failed: {0}")]
    ProviderAccountStore(String),

    #[error(transparent)]
    CommunicationMessagePort(#[from] ProviderCommunicationMessagePortError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
