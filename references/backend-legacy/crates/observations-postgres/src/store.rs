use makosh_events_api::NewEventEnvelope;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};

use makosh_events_postgres::store::EventStore;

use crate::errors::ObservationStoreError;
use makosh_observations_api::models::{
    NewObservation, NewObservationIngestionRun, NewObservationLink, Observation,
    ObservationIngestionRun, ObservationIngestionRunStatus, ObservationKindDefinition,
    ObservationLink, ObservationOriginKind, validate_json_object, validate_non_empty,
};

#[derive(Clone)]
pub struct ObservationStore {
    pool: PgPool,
}

impl ObservationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn capture(
        &self,
        observation: &NewObservation,
    ) -> Result<Observation, ObservationStoreError> {
        observation.validate()?;

        let mut transaction = self.pool.begin().await?;
        let stored = Self::capture_in_transaction(&mut transaction, observation).await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn list_kind_definitions(
        &self,
    ) -> Result<Vec<ObservationKindDefinition>, ObservationStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT
                kind_definition_id,
                code,
                name,
                version,
                category,
                description,
                created_at,
                updated_at
            FROM observation_kind_definitions
            ORDER BY category ASC, code ASC, version ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_kind_definition).collect()
    }

    pub async fn get(
        &self,
        observation_id: &str,
    ) -> Result<Option<Observation>, ObservationStoreError> {
        validate_non_empty("observation_id", observation_id)?;
        let sql = observation_select_sql("WHERE observation.observation_id = $1");
        let row = sqlx::query(&sql)
            .bind(observation_id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(row_to_observation).transpose()
    }

    pub async fn upsert_link(
        &self,
        link: &NewObservationLink,
    ) -> Result<ObservationLink, ObservationStoreError> {
        link.validate()?;
        let mut transaction = self.pool.begin().await?;
        let stored = Self::upsert_link_in_transaction(&mut transaction, link).await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn upsert_link_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        link: &NewObservationLink,
    ) -> Result<ObservationLink, ObservationStoreError> {
        link.validate()?;
        let row = sqlx::query(
            r#"
            INSERT INTO observation_links (
                observation_id,
                domain,
                entity_kind,
                entity_id,
                relationship_kind,
                confidence,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (observation_id, domain, entity_kind, entity_id, relationship_kind)
            DO UPDATE SET
                confidence = EXCLUDED.confidence,
                metadata = EXCLUDED.metadata
            RETURNING
                observation_id,
                domain,
                entity_kind,
                entity_id,
                relationship_kind,
                confidence::float8 AS confidence,
                metadata,
                created_at
            "#,
        )
        .bind(link.observation_id.trim())
        .bind(link.domain.trim())
        .bind(link.entity_kind.trim())
        .bind(link.entity_id.trim())
        .bind(link.relationship_kind.trim())
        .bind(link.confidence)
        .bind(&link.metadata)
        .fetch_one(&mut **transaction)
        .await?;
        row_to_observation_link(row)
    }

    pub async fn list_links(
        &self,
        observation_id: &str,
    ) -> Result<Vec<ObservationLink>, ObservationStoreError> {
        validate_non_empty("observation_id", observation_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                observation_id,
                domain,
                entity_kind,
                entity_id,
                relationship_kind,
                confidence::float8 AS confidence,
                metadata,
                created_at
            FROM observation_links
            WHERE observation_id = $1
            ORDER BY domain ASC, entity_kind ASC, entity_id ASC, relationship_kind ASC
            "#,
        )
        .bind(observation_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_observation_link).collect()
    }

    pub async fn start_ingestion_run(
        &self,
        run: &NewObservationIngestionRun,
    ) -> Result<ObservationIngestionRun, ObservationStoreError> {
        run.validate()?;
        let row = sqlx::query(
            r#"
            INSERT INTO observation_ingestion_runs (
                ingestion_run_id,
                observation_id,
                pipeline,
                status
            )
            VALUES ($1, $2, $3, 'running')
            ON CONFLICT (ingestion_run_id)
            DO UPDATE SET
                observation_id = EXCLUDED.observation_id,
                pipeline = EXCLUDED.pipeline
            RETURNING
                ingestion_run_id,
                observation_id,
                pipeline,
                status,
                started_at,
                finished_at,
                output,
                error_message
            "#,
        )
        .bind(run.ingestion_run_id.trim())
        .bind(run.observation_id.trim())
        .bind(run.pipeline.trim())
        .fetch_one(&self.pool)
        .await?;
        row_to_observation_ingestion_run(row)
    }

    pub async fn finish_ingestion_run(
        &self,
        ingestion_run_id: &str,
        status: ObservationIngestionRunStatus,
        output: &serde_json::Value,
        error_message: Option<&str>,
    ) -> Result<ObservationIngestionRun, ObservationStoreError> {
        validate_non_empty("ingestion_run_id", ingestion_run_id)?;
        validate_json_object("output", output)?;
        if let Some(error_message) = error_message {
            validate_non_empty("error_message", error_message)?;
        }

        let row = sqlx::query(
            r#"
            UPDATE observation_ingestion_runs
            SET
                status = $2,
                finished_at = now(),
                output = $3,
                error_message = $4
            WHERE ingestion_run_id = $1
            RETURNING
                ingestion_run_id,
                observation_id,
                pipeline,
                status,
                started_at,
                finished_at,
                output,
                error_message
            "#,
        )
        .bind(ingestion_run_id)
        .bind(status.as_str())
        .bind(output)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await?;
        row_to_observation_ingestion_run(row)
    }

    pub async fn list_ingestion_runs(
        &self,
        observation_id: &str,
    ) -> Result<Vec<ObservationIngestionRun>, ObservationStoreError> {
        validate_non_empty("observation_id", observation_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                ingestion_run_id,
                observation_id,
                pipeline,
                status,
                started_at,
                finished_at,
                output,
                error_message
            FROM observation_ingestion_runs
            WHERE observation_id = $1
            ORDER BY started_at DESC, ingestion_run_id ASC
            "#,
        )
        .bind(observation_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(row_to_observation_ingestion_run)
            .collect()
    }

    pub async fn capture_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        observation: &NewObservation,
    ) -> Result<Observation, ObservationStoreError> {
        let kind_definition_id = sqlx::query_scalar::<_, String>(
            r#"
            SELECT kind_definition_id
            FROM observation_kind_definitions
            WHERE code = $1
              AND version = 1
            "#,
        )
        .bind(observation.kind_code.trim())
        .fetch_optional(&mut **transaction)
        .await?
        .ok_or_else(|| {
            ObservationStoreError::ObservationKindNotFound(observation.kind_code.trim().to_owned())
        })?;

        let content_hash = content_hash(observation)?;
        let observation_id = observation_id(observation, &content_hash)?;
        let inserted = sqlx::query(observation_insert_sql())
            .bind(&observation_id)
            .bind(&kind_definition_id)
            .bind(observation.origin_kind.as_str())
            .bind(&observation.vault_source_id)
            .bind(observation.observed_at)
            .bind(&observation.payload)
            .bind(observation.confidence)
            .bind(&content_hash)
            .bind(observation.source_ref.trim())
            .bind(&observation.provenance)
            .bind(observation.kind_code.trim())
            .fetch_optional(&mut **transaction)
            .await?;

        if let Some(row) = inserted {
            let stored = row_to_observation(row)?;
            append_observation_captured_event(transaction, &stored).await?;
            return Ok(stored);
        }

        let sql = observation_select_sql("WHERE observation.observation_id = $1");
        let row = sqlx::query(&sql)
            .bind(&observation_id)
            .fetch_one(&mut **transaction)
            .await?;
        row_to_observation(row)
    }
}

fn observation_insert_sql() -> &'static str {
    r#"
    INSERT INTO observations (
        observation_id,
        kind_definition_id,
        origin_kind,
        vault_source_id,
        observed_at,
        payload,
        confidence,
        content_hash,
        source_ref,
        provenance
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
    ON CONFLICT (observation_id) DO NOTHING
    RETURNING
        observation_id,
        kind_definition_id,
        $11::text AS kind_code,
        origin_kind,
        vault_source_id,
        observed_at,
        captured_at,
        payload,
        confidence::float8 AS confidence,
        content_hash,
        source_ref,
        provenance
    "#
}

fn observation_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT
            observation.observation_id,
            observation.kind_definition_id,
            kind.code AS kind_code,
            observation.origin_kind,
            observation.vault_source_id,
            observation.observed_at,
            observation.captured_at,
            observation.payload,
            observation.confidence::float8 AS confidence,
            observation.content_hash,
            observation.source_ref,
            observation.provenance
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        {where_clause}
        "#
    )
}

fn row_to_kind_definition(row: PgRow) -> Result<ObservationKindDefinition, ObservationStoreError> {
    Ok(ObservationKindDefinition {
        kind_definition_id: row.try_get("kind_definition_id")?,
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        version: row.try_get("version")?,
        category: row.try_get("category")?,
        description: row.try_get("description")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_observation(row: PgRow) -> Result<Observation, ObservationStoreError> {
    let origin_kind: String = row.try_get("origin_kind")?;
    Ok(Observation {
        observation_id: row.try_get("observation_id")?,
        kind_definition_id: row.try_get("kind_definition_id")?,
        kind_code: row.try_get("kind_code")?,
        origin_kind: ObservationOriginKind::parse(origin_kind)?,
        vault_source_id: row.try_get("vault_source_id")?,
        observed_at: row.try_get("observed_at")?,
        captured_at: row.try_get("captured_at")?,
        payload: row.try_get("payload")?,
        confidence: row.try_get("confidence")?,
        content_hash: row.try_get("content_hash")?,
        source_ref: row.try_get("source_ref")?,
        provenance: row.try_get("provenance")?,
    })
}

fn row_to_observation_link(row: PgRow) -> Result<ObservationLink, ObservationStoreError> {
    Ok(ObservationLink {
        observation_id: row.try_get("observation_id")?,
        domain: row.try_get("domain")?,
        entity_kind: row.try_get("entity_kind")?,
        entity_id: row.try_get("entity_id")?,
        relationship_kind: row.try_get("relationship_kind")?,
        confidence: row.try_get("confidence")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
    })
}

fn row_to_observation_ingestion_run(
    row: PgRow,
) -> Result<ObservationIngestionRun, ObservationStoreError> {
    let status: String = row.try_get("status")?;
    Ok(ObservationIngestionRun {
        ingestion_run_id: row.try_get("ingestion_run_id")?,
        observation_id: row.try_get("observation_id")?,
        pipeline: row.try_get("pipeline")?,
        status: ObservationIngestionRunStatus::parse(status)?,
        started_at: row.try_get("started_at")?,
        finished_at: row.try_get("finished_at")?,
        output: row.try_get("output")?,
        error_message: row.try_get("error_message")?,
    })
}

async fn append_observation_captured_event(
    transaction: &mut Transaction<'_, Postgres>,
    observation: &Observation,
) -> Result<(), ObservationStoreError> {
    let event = NewEventEnvelope::builder(
        observation_captured_event_id(&observation.observation_id),
        "observation.captured.v1",
        observation.captured_at,
        json!({
            "platform": "observations",
            "source_id": observation.observation_id
        }),
        json!({
            "observation_id": observation.observation_id,
            "kind": "observation",
            "entity_id": observation.observation_id,
            "observation_kind": observation.kind_code
        }),
    )
    .payload(json!({
        "origin_kind": observation.origin_kind.as_str(),
        "vault_source_id": observation.vault_source_id,
        "source_ref": observation.source_ref,
        "confidence": observation.confidence,
        "content_hash": observation.content_hash
    }))
    .provenance(json!({
        "canonical_evidence_store": true
    }))
    .correlation_id(observation.observation_id.clone())
    .build()?;

    EventStore::append_in_transaction(transaction, &event).await?;
    Ok(())
}

pub fn observation_captured_event_id(observation_id: &str) -> String {
    format!("event:v1:observation-captured:{observation_id}")
}

fn observation_id(
    observation: &NewObservation,
    content_hash: &str,
) -> Result<String, ObservationStoreError> {
    let mut digest = Sha256::new();
    digest.update(observation.kind_code.trim().as_bytes());
    digest.update(b"\n");
    digest.update(observation.origin_kind.as_str().as_bytes());
    digest.update(b"\n");
    digest.update(observation.observed_at.to_rfc3339().as_bytes());
    digest.update(b"\n");
    digest.update(observation.source_ref.trim().as_bytes());
    digest.update(b"\n");
    digest.update(content_hash.as_bytes());
    digest.update(b"\n");
    digest.update(serde_json::to_vec(&observation.provenance)?);
    Ok(format!("observation:v1:{:x}", digest.finalize()))
}

fn content_hash(observation: &NewObservation) -> Result<String, ObservationStoreError> {
    let mut digest = Sha256::new();
    digest.update(observation.source_ref.trim().as_bytes());
    digest.update(b"\n");
    digest.update(serde_json::to_vec(&observation.payload)?);
    Ok(format!("sha256:{:x}", digest.finalize()))
}
