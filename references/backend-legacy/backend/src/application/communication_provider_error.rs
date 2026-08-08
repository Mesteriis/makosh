use thiserror::Error;

use crate::domains::communications::messages::provider_observation_projection::CommunicationSignalProjectionError;
use crate::domains::signal_hub::store::SignalHubError;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::platform::audit::errors::ApiAuditError;

#[derive(Debug, Error)]
pub(crate) enum TelegramMessageWriteError {
    #[error(transparent)]
    Telegram(#[from] TelegramError),
    #[error(transparent)]
    Communication(#[from] makosh_communications_postgres::errors::CommunicationIngestionError),
    #[error(transparent)]
    SignalHub(#[from] SignalHubError),
    #[error(transparent)]
    SignalProjection(#[from] CommunicationSignalProjectionError),
    #[error("{0}")]
    SignalControlBlocked(String),
    #[error(transparent)]
    Audit(#[from] ApiAuditError),
}
