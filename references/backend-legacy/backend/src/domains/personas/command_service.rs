use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use thiserror::Error;

use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

use super::api::errors::PersonaProjectionError;
use super::api::models::Persona;
use super::api::store::PersonaProjectionStore;
use super::core::errors::PersonaCoreError;
use super::core::identities::{PersonaIdentity, PersonaIdentityStore};
use super::core::interaction_contexts::{
    NewPersonaInteractionContext, PersonaInteractionContext, PersonaInteractionContextStore,
};
use super::core::roles::{PersonaRole, PersonaRoleStore};
use super::enrichment::errors::PersonaEnrichmentError;
use super::enrichment::store::PersonaEnrichmentStore;
use super::enrichment_engine::{EnrichmentEngineError, EnrichmentResultStore};
use super::health::{PersonaHealthError, PersonaHealthStore};
use super::identity::errors::PersonaIdentityError;
use super::identity::models::{PersonaIdentityReviewCommand, PersonaIdentityReviewCommandResult};
use super::identity::store::PersonaIdentityReviewStore;
use super::intelligence::{PersonaIntelligenceService, PersonaMessage};
use super::investigator::errors::InvestigatorError;
use super::investigator::models::{DossierReviewState, DossierSnapshot};
use super::investigator::service::PersonaInvestigator;
use super::memory::cards::{PersonaMemoryCard, PersonaMemoryCardStore};
use super::memory::errors::PersonaMemoryError;
use super::memory::facts::{PersonaFact, PersonaFactStore};
use super::memory::preferences::{PersonaPreference, PersonaPreferenceStore};
use super::memory::relationship_events::{
    NewRelationshipEvent, RelationshipEvent, RelationshipEventStore,
};

#[path = "command_service/runtime.rs"]
mod runtime;

#[derive(Clone)]
pub struct PersonaCommandService {
    pool: PgPool,
}

#[derive(Clone, Debug)]
pub struct ProviderAddressBookEntryPersonaCommand {
    pub source_account_id: String,
    pub provider_address_book_entry_id: String,
    pub display_name: Option<String>,
    pub primary_email: Option<String>,
    pub additional_emails: Vec<String>,
    pub phone_numbers: Vec<String>,
}

fn manual_record_source(requested_source: &str, observation_id: &str) -> String {
    if requested_source.trim() == "manual" {
        format!("observation:{observation_id}")
    } else {
        requested_source.to_owned()
    }
}

async fn first_existing_identity_person_id(
    identity_store: &PersonaIdentityStore,
    identity_type: &str,
    identity_values: &[String],
) -> Result<Option<String>, PersonaCommandServiceError> {
    for identity_value in identity_values {
        if let Some(persona_id) = identity_store
            .find_active_persona_id(identity_type, identity_value)
            .await?
        {
            return Ok(Some(persona_id));
        }
    }
    Ok(None)
}

async fn existing_provider_address_book_link_person_id(
    pool: &sqlx::postgres::PgPool,
    source_account_id: &str,
    provider_address_book_entry_id: &str,
) -> Result<Option<String>, PersonaCommandServiceError> {
    let row = sqlx::query_scalar(
        r#"
        SELECT persona_id
        FROM communication_provider_address_book_links
        WHERE account_id = $1
          AND provider_address_book_entry_id = $2
        "#,
    )
    .bind(source_account_id)
    .bind(provider_address_book_entry_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

fn normalized_address_book_phone_numbers(phone_numbers: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for phone_number in phone_numbers {
        let digits = phone_number
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>();
        if !digits.is_empty() && !normalized.iter().any(|existing| existing == &digits) {
            normalized.push(digits);
        }
    }
    normalized
}

fn provider_address_book_entry_persona_id(
    source_account_id: &str,
    provider_address_book_entry_id: &str,
) -> String {
    let seed = format!(
        "provider_address_book_entry:{}:{}",
        source_account_id.trim(),
        provider_address_book_entry_id.trim()
    );
    let digest = Sha256::digest(seed.as_bytes());
    let hex = format!("{digest:x}");
    format!("persona:v1:provider_address_book_entry:{}", &hex[..24])
}

#[derive(Debug, Error)]
pub enum PersonaCommandServiceError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    Projection(#[from] PersonaProjectionError),
    #[error(transparent)]
    Core(#[from] PersonaCoreError),
    #[error(transparent)]
    Enrichment(#[from] PersonaEnrichmentError),
    #[error(transparent)]
    EnrichmentEngine(#[from] EnrichmentEngineError),
    #[error(transparent)]
    Memory(#[from] PersonaMemoryError),
    #[error(transparent)]
    Health(#[from] PersonaHealthError),
    #[error(transparent)]
    Identity(#[from] PersonaIdentityError),
    #[error(transparent)]
    Investigator(#[from] InvestigatorError),
}
