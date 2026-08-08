use makosh_communications_api::evidence::CommunicationEvidencePortError;

use crate::platform::communications::errors::ProviderCommunicationMessagePortError;
use crate::platform::secrets::database_vault::DatabaseEncryptedVaultError;
use crate::platform::secrets::errors::SecretReferenceError;
use crate::vault::errors::HostVaultError;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, thiserror::Error)]
pub enum TelegramError {
    #[error("invalid Telegram request: {0}")]
    InvalidRequest(String),

    #[error("Telegram TDLib runtime is not available: {0}")]
    TdlibRuntimeUnavailable(String),

    #[error("Telegram TDLib runtime failed: {0}")]
    TdlibRuntime(String),

    #[error("Telegram QR generation failed: {0}")]
    QrGeneration(String),

    #[error("Telegram QR login setup was not found")]
    QrLoginNotFound,

    #[error("Telegram provider account store operation failed: {0}")]
    ProviderAccountStore(String),

    #[error("Telegram media storage operation failed: {0}")]
    MediaStorage(String),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    DatabaseVault(#[from] DatabaseEncryptedVaultError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    CommunicationMessagePort(#[from] ProviderCommunicationMessagePortError),

    #[error(transparent)]
    CommunicationEvidencePort(#[from] CommunicationEvidencePortError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
