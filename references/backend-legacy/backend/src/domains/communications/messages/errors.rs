use thiserror::Error;

use crate::domains::communications::storage::errors::CommunicationStorageError;
use crate::platform::communications::rfc822::errors::EmailRfc822ParseError;
use makosh_communications_postgres::provider_commands::CommunicationProviderCommandError;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum MessageProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    CommunicationStorage(#[from] CommunicationStorageError),

    #[error(transparent)]
    Rfc822(#[from] EmailRfc822ParseError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    ProviderCommand(#[from] CommunicationProviderCommandError),

    #[error("raw email payload missing required field or wrong type: {0}")]
    MissingPayloadField(&'static str),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error(
        "raw communication record does not match projected message tuple: raw_record_id={raw_record_id}, account_id={account_id}, provider_record_id={provider_record_id}"
    )]
    RawRecordTupleMismatch {
        raw_record_id: String,
        account_id: String,
        provider_record_id: String,
    },

    #[error("raw communication record was not found: {0}")]
    RawRecordNotFound(String),

    #[error("stored communication message recipients must be a JSON array of strings")]
    InvalidStoredRecipients,

    #[error("communication message metadata must be a JSON object")]
    InvalidMessageMetadata,

    #[error("unsupported raw blob storage kind: {0}")]
    UnsupportedRawBlobStorageKind(String),

    #[error("message query limit must be between 1 and 5000: {0}")]
    InvalidLimit(i64),

    #[error("invalid communication message cursor")]
    InvalidCursor,

    #[error("communication message was not found")]
    MessageNotFound,

    #[error("communication projection write failed: {0}")]
    ProjectionWrite(String),

    #[error("invalid workflow state: {0}")]
    InvalidWorkflowState(String),

    #[error("invalid local message state: {0}")]
    InvalidLocalState(String),

    #[error("invalid importance score: {0}, must be 0-100")]
    InvalidImportanceScore(i16),
}
