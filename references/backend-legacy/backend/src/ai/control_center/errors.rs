use thiserror::Error;

use crate::platform::secrets::errors::SecretReferenceError;
use crate::vault::errors::HostVaultError;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum AiControlCenterError {
    #[error("AI provider was not found")]
    ProviderNotFound,

    #[error("AI model was not found")]
    ModelNotFound,

    #[error("AI prompt was not found")]
    PromptNotFound,

    #[error("AI prompt version was not found")]
    PromptVersionNotFound,

    #[error("invalid AI control center request: {0}")]
    InvalidRequest(String),

    #[error("invalid AI control center field `{field}`")]
    EmptyField { field: &'static str },

    #[error("AI control center payload contains secret-like data")]
    SecretLikePayload,

    #[error("AI provider model sync failed: {0}")]
    ProviderModelSync(String),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl AiControlCenterError {
    pub fn is_invalid_request(&self) -> bool {
        matches!(
            self,
            Self::InvalidRequest(_)
                | Self::EmptyField { .. }
                | Self::SecretLikePayload
                | Self::ModelNotFound
                | Self::PromptNotFound
                | Self::PromptVersionNotFound
        )
    }
}
