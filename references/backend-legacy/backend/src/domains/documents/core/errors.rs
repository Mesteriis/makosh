use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocumentImportError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    ObservationStore(#[from] makosh_observations_postgres::errors::ObservationStoreError),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("document_kind must be markdown or pdf: {0}")]
    InvalidDocumentKind(String),

    #[error(
        "document_kind change rejected for document_id={document_id}: existing={existing_kind}, new={new_kind}"
    )]
    DocumentKindChange {
        document_id: String,
        existing_kind: String,
        new_kind: String,
    },

    #[error("document import upsert skipped unexpectedly for document_id={0}")]
    UpsertSkipped(String),
}

#[derive(Debug, Error)]
pub enum DocumentImportWithProcessingError {
    #[error(transparent)]
    DocumentImport(#[from] DocumentImportError),

    #[error(transparent)]
    Processing(#[from] crate::domains::documents::processing::errors::DocumentProcessingError),
}
