use thiserror::Error;

use crate::ai::control_center::errors::AiControlCenterError;
use crate::domains::calendar::events::errors::CalendarError;
use crate::platform::secrets::errors::SecretReferenceError;
use crate::vault::errors::HostVaultError;
use makosh_communications_postgres::errors::CommunicationIngestionError;

#[derive(Debug, Error)]
pub(super) enum HostVaultReconciliationError {
    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    Calendar(#[from] CalendarError),

    #[error(transparent)]
    AiControlCenter(#[from] AiControlCenterError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
