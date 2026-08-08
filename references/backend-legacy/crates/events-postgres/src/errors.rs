use thiserror::Error;

use makosh_events_api::EventEnvelopeError;

#[derive(Debug, Error)]
pub enum EventStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error(transparent)]
    Envelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("replay position must be non-negative, got {0}")]
    InvalidReplayPosition(i64),

    #[error("event handler failed: {0}")]
    ConsumerHandlerFailed(String),

    #[error("event dead letter was not found: {0}")]
    DeadLetterNotFound(String),

    #[error("event dead letter is not replay-requested: {0}")]
    DeadLetterNotReplayRequested(String),

    #[error("invalid event dead letter review state: {0}")]
    InvalidDeadLetterReviewState(String),
}

impl EventStoreError {
    pub fn is_unique_violation(&self) -> bool {
        match self {
            Self::Sqlx(sqlx::Error::Database(error)) => error.code().as_deref() == Some("23505"),
            _ => false,
        }
    }
}
