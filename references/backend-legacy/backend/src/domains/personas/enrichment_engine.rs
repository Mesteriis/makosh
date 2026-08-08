use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::engines::enrichment::{
    engine::EnrichmentEngine, errors::EnrichmentEngineError as SharedEnrichmentEngineError,
};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::review_links::materialize_review_transition_link as materialize_review_link;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichmentResult {
    pub id: String,
    #[serde(alias = "person_id")]
    pub persona_id: String,
    pub source: String,
    pub url: Option<String>,
    pub data: Value,
    pub confidence: f64,
    pub status: String,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EnrichmentResultStore {
    pool: PgPool,
}

impl EnrichmentResultStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        persona_id: &str,
    ) -> Result<Vec<EnrichmentResult>, EnrichmentEngineError> {
        let rows = sqlx::query(
            "SELECT id::text, persona_id, source, url, data, confidence::float8 AS confidence, status, last_checked_at, applied_at, created_at
             FROM enrichment_results WHERE persona_id = $1 ORDER BY created_at DESC"
        ).bind(persona_id).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_enrichment).collect()
    }

    pub async fn upsert(
        &self,
        persona_id: &str,
        source: &str,
        data: Value,
        confidence: f64,
    ) -> Result<EnrichmentResult, EnrichmentEngineError> {
        let extracted_claim = extracted_claim_from_data(&data)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| source.to_owned());
        let candidate = EnrichmentEngine::persona_observation_candidate(
            persona_id,
            source,
            &extracted_claim,
            data,
            confidence,
        )?;

        let row = sqlx::query(
            "INSERT INTO enrichment_results (persona_id, source, data, confidence, status)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING
             RETURNING id::text, persona_id, source, url, data, confidence::float8 AS confidence, status, last_checked_at, applied_at, created_at"
        ).bind(persona_id).bind(&candidate.source).bind(&candidate.data).bind(candidate.confidence).bind(&candidate.review_state).fetch_one(&self.pool).await?;
        row_to_enrichment(row)
    }

    pub async fn apply(&self, id: &str) -> Result<(), EnrichmentEngineError> {
        self.apply_with_observation(id, None, None).await
    }

    pub async fn apply_with_observation(
        &self,
        id: &str,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<(), EnrichmentEngineError> {
        sqlx::query("UPDATE enrichment_results SET status = 'applied', applied_at = now() WHERE id::text = $1")
            .bind(id).execute(&self.pool).await?;
        materialize_review_link(
            &self.pool,
            observation_id,
            "personas",
            "enrichment_result",
            id,
            "status",
            "applied",
            metadata,
        )
        .await?;
        Ok(())
    }

    pub async fn reject(&self, id: &str) -> Result<(), EnrichmentEngineError> {
        self.reject_with_observation(id, None, None).await
    }

    pub async fn reject_with_observation(
        &self,
        id: &str,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<(), EnrichmentEngineError> {
        sqlx::query("UPDATE enrichment_results SET status = 'rejected' WHERE id::text = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        materialize_review_link(
            &self.pool,
            observation_id,
            "personas",
            "enrichment_result",
            id,
            "status",
            "rejected",
            metadata,
        )
        .await?;
        Ok(())
    }
}

fn extracted_claim_from_data(data: &Value) -> Option<&str> {
    data.get("extracted_claim")
        .or_else(|| data.get("claim"))
        .or_else(|| data.get("value"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|claim| !claim.is_empty())
}

fn row_to_enrichment(row: PgRow) -> Result<EnrichmentResult, EnrichmentEngineError> {
    Ok(EnrichmentResult {
        id: row.try_get("id")?,
        persona_id: row.try_get("persona_id")?,
        source: row.try_get("source")?,
        url: row.try_get("url")?,
        data: row.try_get("data")?,
        confidence: row.try_get("confidence")?,
        status: row.try_get("status")?,
        last_checked_at: row.try_get("last_checked_at")?,
        applied_at: row.try_get("applied_at")?,
        created_at: row.try_get("created_at")?,
    })
}

#[derive(Debug, Error)]
pub enum EnrichmentEngineError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Shared(#[from] SharedEnrichmentEngineError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("enrichment not found")]
    NotFound,
}
