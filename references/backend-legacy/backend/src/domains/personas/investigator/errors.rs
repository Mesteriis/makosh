use thiserror::Error;

use crate::domains::personas::enrichment::errors::PersonaEnrichmentError;
use crate::domains::personas::memory::errors::PersonaMemoryError;
use makosh_events_postgres::errors::EventStoreError;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Debug, Error)]
pub enum InvestigatorError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Memory(#[from] crate::engines::memory::errors::MemoryEngineError),
    #[error(transparent)]
    Timeline(#[from] crate::engines::timeline::errors::TimelineEngineError),
    #[error(transparent)]
    Trust(#[from] crate::engines::trust::errors::TrustEngineError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Event(#[from] EventStoreError),
    #[error("persona not found")]
    PersonaNotFound,
    #[error("dossier snapshot not found")]
    DossierSnapshotNotFound,
    #[error("review_state must be suggested, user_confirmed, or user_rejected")]
    InvalidDossierReviewState,
}

impl From<PersonaEnrichmentError> for InvestigatorError {
    fn from(error: PersonaEnrichmentError) -> Self {
        match error {
            PersonaEnrichmentError::NotFound => Self::PersonaNotFound,
            PersonaEnrichmentError::Sqlx(error) => Self::Sqlx(error),
            PersonaEnrichmentError::Trust(error) => Self::Trust(error),
            PersonaEnrichmentError::Observation(error) => Self::Observation(error),
            PersonaEnrichmentError::Event(error) => Self::Event(error),
        }
    }
}

impl From<PersonaMemoryError> for InvestigatorError {
    fn from(error: PersonaMemoryError) -> Self {
        match error {
            PersonaMemoryError::NotFound => Self::PersonaNotFound,
            PersonaMemoryError::Sqlx(error) => Self::Sqlx(error),
            PersonaMemoryError::Memory(error) => Self::Memory(error),
            PersonaMemoryError::Timeline(error) => Self::Timeline(error),
            PersonaMemoryError::ObservationStore(error) => Self::Observation(error),
        }
    }
}
