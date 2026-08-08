use makosh_events_api::EventEnvelopeError;
use std::path::PathBuf;

use makosh_events_postgres::errors::EventStoreError;
use makosh_observations_postgres::errors::ObservationStoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommunicationStorageError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("storage_kind must be local_fs: {0}")]
    InvalidStorageKind(String),

    #[error("storage_path must be relative and stay inside mail blob root: {0}")]
    UnsafeStoragePath(String),

    #[error("sha256 must use sha256:<64 lowercase hex chars>: {0}")]
    InvalidSha256(String),

    #[error("size_bytes must not be negative: {0}")]
    NegativeSizeBytes(i64),

    #[error("blob content is too large to represent as i64 size_bytes")]
    BlobTooLarge,

    #[error("blob size mismatch for {path}: expected {expected}, actual {actual}")]
    BlobSizeMismatch {
        path: PathBuf,
        expected: i64,
        actual: i64,
    },

    #[error("invalid attachment disposition: {0}")]
    InvalidDisposition(String),

    #[error("invalid attachment scan status: {0}")]
    InvalidScanStatus(String),

    #[error("{0} must be a JSON object")]
    NonObjectJson(&'static str),

    #[error("not_scanned attachment scan reports must not include engine, checked_at or summary")]
    InvalidNotScannedReport,
}

#[derive(Debug, Error)]
pub enum AttachmentSafetyScanError {
    #[error("attachment safety scanner failed: {0}")]
    Scanner(String),
}
