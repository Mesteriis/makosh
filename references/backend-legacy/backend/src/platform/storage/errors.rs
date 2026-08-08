use thiserror::Error;

use crate::platform::settings::errors::SettingsError;
use makosh_events_postgres::errors::EventStoreError;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("failed to connect to PostgreSQL")]
    Connect(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Settings(#[from] SettingsError),

    #[error("{0}")]
    Invalid(String),
}
