use super::super::types::ApiError;
use crate::domains::personas::api::errors::PersonaProjectionError;
use crate::domains::personas::command_service::PersonaCommandServiceError;
use crate::domains::personas::core::errors::PersonaCoreError;
use crate::domains::personas::identity::errors::PersonaIdentityError;
use crate::domains::personas::memory::errors::PersonaMemoryError;
use makosh_personas_api::{PersonaQueryError, PersonaWriteError};

impl From<PersonaQueryError> for ApiError {
    fn from(error: PersonaQueryError) -> Self {
        tracing::error!(error = %error, "persona read port failed");
        Self::InvalidCommunicationQuery("persona read failed")
    }
}

impl From<PersonaWriteError> for ApiError {
    fn from(error: PersonaWriteError) -> Self {
        tracing::error!(error = %error, "persona write port failed");
        Self::InvalidCommunicationQuery("persona write failed")
    }
}

impl From<PersonaIdentityError> for ApiError {
    fn from(error: PersonaIdentityError) -> Self {
        match error {
            PersonaIdentityError::IdentityCandidateNotFound => Self::PersonaIdentityNotFound,
            PersonaIdentityError::InvalidLimit | PersonaIdentityError::InvalidReviewState(_) => {
                Self::InvalidPersonaIdentityReview(
                    "review_state or limit must be valid for persona identity candidates",
                )
            }
            PersonaIdentityError::InvalidPayload(_)
            | PersonaIdentityError::MissingPayloadField(_)
            | PersonaIdentityError::MissingActorId => {
                Self::InvalidPersonaIdentityReview("invalid identity candidate review payload")
            }
            PersonaIdentityError::Observation(_) => {
                Self::InvalidPersonaIdentityReview("identity candidate evidence observation failed")
            }
            _ => Self::PersonaIdentity(error),
        }
    }
}

impl From<PersonaProjectionError> for ApiError {
    fn from(error: PersonaProjectionError) -> Self {
        Self::PersonaProjection(error)
    }
}

impl From<crate::domains::personas::enrichment::errors::PersonaEnrichmentError> for ApiError {
    fn from(error: crate::domains::personas::enrichment::errors::PersonaEnrichmentError) -> Self {
        match error {
            crate::domains::personas::enrichment::errors::PersonaEnrichmentError::NotFound => {
                ApiError::PersonaIdentityNotFound
            }
            _ => {
                tracing::error!(error = %error, "persona enrichment failed");
                ApiError::InvalidCommunicationQuery("persona enrichment failed")
            }
        }
    }
}

impl From<PersonaMemoryError> for ApiError {
    fn from(error: PersonaMemoryError) -> Self {
        match error {
            PersonaMemoryError::NotFound => ApiError::PersonaIdentityNotFound,
            _ => {
                tracing::error!(error = %error, "persona memory operation failed");
                ApiError::InvalidCommunicationQuery("persona memory operation failed")
            }
        }
    }
}

impl From<PersonaCoreError> for ApiError {
    fn from(error: PersonaCoreError) -> Self {
        match error {
            PersonaCoreError::IdentityNotFound | PersonaCoreError::PersonaNotFound => {
                ApiError::PersonaIdentityNotFound
            }
            _ => {
                tracing::error!(error = %error, "persona core operation failed");
                ApiError::InvalidCommunicationQuery("persona core operation failed")
            }
        }
    }
}

impl From<PersonaCommandServiceError> for ApiError {
    fn from(error: PersonaCommandServiceError) -> Self {
        match error {
            PersonaCommandServiceError::Projection(error) => Self::from(error),
            PersonaCommandServiceError::Core(error) => Self::from(error),
            PersonaCommandServiceError::Enrichment(error) => Self::from(error),
            PersonaCommandServiceError::EnrichmentEngine(error) => Self::from(error),
            PersonaCommandServiceError::Memory(error) => Self::from(error),
            PersonaCommandServiceError::Health(error) => Self::from(error),
            PersonaCommandServiceError::Identity(error) => Self::from(error),
            PersonaCommandServiceError::Investigator(error) => Self::from(error),
            PersonaCommandServiceError::Sqlx(error) => {
                Self::PersonaProjection(PersonaProjectionError::Sqlx(error))
            }
            PersonaCommandServiceError::Observation(error) => {
                tracing::error!(error = %error, "persona manual observation capture failed");
                ApiError::InvalidCommunicationQuery("persona manual observation capture failed")
            }
        }
    }
}
