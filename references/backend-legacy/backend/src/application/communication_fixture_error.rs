use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::provider_observation_projection::CommunicationSignalProjectionError;
use crate::domains::communications::storage::errors::CommunicationStorageError;
use crate::domains::signal_hub::store::SignalHubError;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::platform::calls::errors::CallError;
use crate::platform::communications::errors::ProviderCommunicationMessagePortError;
use crate::workflows::review_inbox::ReviewInboxWorkflowError;
use makosh_events_postgres::errors::EventStoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum CommunicationFixtureIngestError {
    #[error(transparent)]
    Telegram(#[from] TelegramError),
    #[error(transparent)]
    Whatsapp(#[from] WhatsappWebError),
    #[error(transparent)]
    Communication(#[from] makosh_communications_postgres::errors::CommunicationIngestionError),
    #[error(transparent)]
    ProviderMessage(#[from] ProviderCommunicationMessagePortError),
    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),
    #[error(transparent)]
    Storage(#[from] CommunicationStorageError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    SignalHub(#[from] SignalHubError),
    #[error(transparent)]
    SignalProjection(#[from] CommunicationSignalProjectionError),
    #[error("{0}")]
    SignalControlBlocked(String),
    #[error(transparent)]
    Review(#[from] ReviewInboxWorkflowError),
    #[error(transparent)]
    EventStore(#[from] EventStoreError),
    #[error(transparent)]
    Call(#[from] CallError),
    #[error(transparent)]
    PersonaCore(#[from] crate::domains::personas::core::errors::PersonaCoreError),
}
