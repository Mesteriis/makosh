use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;
use sqlx::Row;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::domains::personas::core::evidence::link_persona_entity_in_transaction;
use crate::domains::personas::enrichment::PERSONA_TRUST_SCORE_CHANGED_EVENT_TYPE;
use crate::domains::personas::intelligence::CommunicationFingerprint;
use makosh_events_postgres::errors::EventStoreError;
use makosh_events_postgres::store::EventStore;

use super::errors::PersonaEnrichmentError;
use super::materialization::{
    sync_favorite_preference_in_transaction, sync_notes_memory_card_in_transaction,
};
use super::models::EnrichedPersona;
use super::rows::{ENRICHED_PERSONA_COLUMNS, row_to_enriched};
use super::store::PersonaEnrichmentStore;

impl PersonaEnrichmentStore {
    pub async fn enrich_person(
        &self,
        persona_id: &str,
        fingerprint: &CommunicationFingerprint,
    ) -> Result<EnrichedPersona, PersonaEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let sql = format!(
            "UPDATE personas SET \
             language = COALESCE($2, personas.language), \
             tone = COALESCE($3, personas.tone), \
             trust_score = COALESCE($4, personas.trust_score), \
             avg_response_hours = COALESCE($5, personas.avg_response_hours), \
             writing_style = COALESCE($6, personas.writing_style), \
             updated_at = now() \
             WHERE persona_id = $1 RETURNING {ENRICHED_PERSONA_COLUMNS}"
        );
        let row = sqlx::query(&sql)
            .bind(persona_id)
            .bind(fingerprint.detected_language.as_deref())
            .bind(fingerprint.typical_tone.as_deref())
            .bind(fingerprint.trust_score)
            .bind(fingerprint.avg_response_hours)
            .bind(fingerprint.writing_style.as_deref())
            .fetch_optional(&mut *transaction)
            .await?;

        let Some(row) = row else {
            return Err(PersonaEnrichmentError::NotFound);
        };
        let enriched = row_to_enriched(row)?;
        append_trust_score_changed_event(
            &mut transaction,
            persona_id,
            fingerprint.trust_score,
            None,
        )
        .await?;
        transaction.commit().await?;

        Ok(enriched)
    }

    pub async fn enrich_persona_with_observation(
        &self,
        persona_id: &str,
        fingerprint: &CommunicationFingerprint,
        observation_id: &str,
    ) -> Result<EnrichedPersona, PersonaEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let sql = format!(
            "UPDATE personas SET \
             language = COALESCE($2, personas.language), \
             tone = COALESCE($3, personas.tone), \
             trust_score = COALESCE($4, personas.trust_score), \
             avg_response_hours = COALESCE($5, personas.avg_response_hours), \
             writing_style = COALESCE($6, personas.writing_style), \
             updated_at = now() \
             WHERE persona_id = $1 RETURNING {ENRICHED_PERSONA_COLUMNS}"
        );
        let row = sqlx::query(&sql)
            .bind(persona_id)
            .bind(fingerprint.detected_language.as_deref())
            .bind(fingerprint.typical_tone.as_deref())
            .bind(fingerprint.trust_score)
            .bind(fingerprint.avg_response_hours)
            .bind(fingerprint.writing_style.as_deref())
            .fetch_optional(&mut *transaction)
            .await?;

        let Some(row) = row else {
            return Err(PersonaEnrichmentError::NotFound);
        };
        let enriched = row_to_enriched(row)?;
        append_trust_score_changed_event(
            &mut transaction,
            persona_id,
            fingerprint.trust_score,
            Some(observation_id),
        )
        .await?;
        link_persona_entity_in_transaction(
            &mut transaction,
            observation_id,
            "persona",
            persona_id,
            Some("profile_enrichment"),
            Some(serde_json::json!({
                "manual_entrypoint": "post_persona_fingerprint"
            })),
        )
        .await?;
        transaction.commit().await?;

        Ok(enriched)
    }

    pub async fn toggle_favorite(&self, persona_id: &str) -> Result<bool, PersonaEnrichmentError> {
        self.toggle_favorite_with_source(persona_id, &format!("personas.is_favorite:{persona_id}"))
            .await
    }

    pub async fn toggle_favorite_with_source(
        &self,
        persona_id: &str,
        source: &str,
    ) -> Result<bool, PersonaEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let is_favorite =
            Self::toggle_favorite_in_transaction(&mut transaction, persona_id, source).await?;
        transaction.commit().await?;
        Ok(is_favorite)
    }

    pub async fn toggle_favorite_with_observation(
        &self,
        persona_id: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<bool, PersonaEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let is_favorite =
            Self::toggle_favorite_in_transaction(&mut transaction, persona_id, source).await?;
        link_persona_entity_in_transaction(
            &mut transaction,
            observation_id,
            "favorite_toggle",
            persona_id,
            None,
            Some(serde_json::json!({
                "is_favorite": is_favorite
            })),
        )
        .await?;
        transaction.commit().await?;
        Ok(is_favorite)
    }

    async fn toggle_favorite_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        persona_id: &str,
        source: &str,
    ) -> Result<bool, PersonaEnrichmentError> {
        let row = sqlx::query(
            "UPDATE personas SET is_favorite = NOT is_favorite, updated_at = now() \
             WHERE persona_id = $1 RETURNING is_favorite",
        )
        .bind(persona_id)
        .fetch_optional(&mut **transaction)
        .await?;
        let Some(row) = row else {
            return Ok(false);
        };
        let is_favorite = row.try_get("is_favorite").unwrap_or(false);
        sync_favorite_preference_in_transaction(transaction, persona_id, is_favorite, source)
            .await?;
        Ok(is_favorite)
    }

    pub async fn set_notes(
        &self,
        persona_id: &str,
        notes: &str,
    ) -> Result<(), PersonaEnrichmentError> {
        self.set_notes_with_source(persona_id, notes, &format!("personas.notes:{persona_id}"))
            .await
    }

    pub async fn set_notes_with_source(
        &self,
        persona_id: &str,
        notes: &str,
        source: &str,
    ) -> Result<(), PersonaEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        Self::set_notes_in_transaction(&mut transaction, persona_id, notes, source).await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn set_notes_with_observation(
        &self,
        persona_id: &str,
        notes: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<(), PersonaEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        Self::set_notes_in_transaction(&mut transaction, persona_id, notes, source).await?;
        link_persona_entity_in_transaction(
            &mut transaction,
            observation_id,
            "notes",
            persona_id,
            None,
            Some(serde_json::json!({
                "manual_entrypoint": "put_person_notes"
            })),
        )
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    async fn set_notes_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        persona_id: &str,
        notes: &str,
        source: &str,
    ) -> Result<(), PersonaEnrichmentError> {
        sqlx::query("UPDATE personas SET notes = $2, updated_at = now() WHERE persona_id = $1")
            .bind(persona_id)
            .bind(notes)
            .execute(&mut **transaction)
            .await?;
        sync_notes_memory_card_in_transaction(transaction, persona_id, notes, source).await?;
        Ok(())
    }
}

async fn append_trust_score_changed_event(
    transaction: &mut Transaction<'_, Postgres>,
    persona_id: &str,
    trust_score: Option<i16>,
    source_observation_id: Option<&str>,
) -> Result<(), PersonaEnrichmentError> {
    let Some(trust_score) = trust_score else {
        return Ok(());
    };

    let event = NewEventEnvelope::builder(
        format!("persona_trust_score_changed:{}", Uuid::now_v7()),
        PERSONA_TRUST_SCORE_CHANGED_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "persona_enrichment",
            "provider": "makosh",
            "source_id": persona_id,
        }),
        json!({
            "kind": "persona",
            "persona_id": persona_id,
        }),
    )
    .payload(json!({
        "persona_id": persona_id,
        "trust_score": trust_score,
        "source_observation_id": source_observation_id,
    }))
    .build()
    .map_err(EventStoreError::from)?;

    EventStore::append_in_transaction(transaction, &event).await?;
    Ok(())
}
