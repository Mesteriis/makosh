use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::{PgPool, PgRow, Postgres};

use super::errors::AiError;
use super::evidence::link_ai_entity_in_transaction;
use super::helpers::{validate_limit, validate_non_empty};
use super::types::AiCitation;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

#[derive(Clone)]
pub struct AiRunStore {
    pool: PgPool,
}

impl AiRunStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn start_run(&self, run: &NewAiRun) -> Result<AiAgentRun, AiError> {
        run.validate()?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO ai_agent_runs (
                run_id,
                agent_id,
                status,
                chat_model,
                embedding_model,
                prompt_template_version,
                model_config,
                query,
                actor_id,
                agent_persona_id,
                owner_persona_id,
                causation_id,
                correlation_id,
                requested_event_id,
                started_at,
                updated_at
            )
            VALUES (
                $1, $2, 'requested', $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, now(), now()
            )
            ON CONFLICT (run_id)
            DO UPDATE SET
                status = 'requested',
                agent_id = EXCLUDED.agent_id,
                chat_model = EXCLUDED.chat_model,
                embedding_model = EXCLUDED.embedding_model,
                prompt_template_version = EXCLUDED.prompt_template_version,
                model_config = EXCLUDED.model_config,
                query = EXCLUDED.query,
                answer = NULL,
                citations = '[]'::jsonb,
                error_summary = NULL,
                actor_id = EXCLUDED.actor_id,
                agent_persona_id = EXCLUDED.agent_persona_id,
                owner_persona_id = EXCLUDED.owner_persona_id,
                causation_id = EXCLUDED.causation_id,
                correlation_id = EXCLUDED.correlation_id,
                requested_event_id = EXCLUDED.requested_event_id,
                completed_event_id = NULL,
                failed_event_id = NULL,
                completed_at = NULL,
                duration_ms = NULL,
                started_at = now(),
                updated_at = now()
            RETURNING *
            "#,
        )
        .bind(&run.run_id)
        .bind(&run.agent_id)
        .bind(&run.chat_model)
        .bind(&run.embedding_model)
        .bind(&run.prompt_template_version)
        .bind(&run.model_config)
        .bind(&run.query)
        .bind(&run.actor_id)
        .bind(&run.agent_persona_id)
        .bind(&run.owner_persona_id)
        .bind(&run.causation_id)
        .bind(&run.correlation_id)
        .bind(&run.requested_event_id)
        .fetch_one(&mut *transaction)
        .await?;

        let stored = row_to_ai_agent_run(row)?;
        capture_run_observation(
            &mut transaction,
            &stored,
            "AI_AGENT_RUN",
            "requested",
            "ai.core.runs.start_run",
            stored.started_at,
        )
        .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn complete_run(
        &self,
        run_id: &str,
        answer: &str,
        citations: &[AiCitation],
        duration_ms: i64,
        completed_event_id: &str,
    ) -> Result<AiAgentRun, AiError> {
        let citations = serde_json::to_value(citations)?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE ai_agent_runs
            SET
                status = 'completed',
                answer = $2,
                citations = $3,
                error_summary = NULL,
                completed_event_id = $4,
                completed_at = now(),
                duration_ms = $5,
                updated_at = now()
            WHERE run_id = $1
            RETURNING *
            "#,
        )
        .bind(run_id)
        .bind(answer)
        .bind(citations)
        .bind(completed_event_id)
        .bind(duration_ms)
        .fetch_one(&mut *transaction)
        .await?;

        let stored = row_to_ai_agent_run(row)?;
        capture_run_observation(
            &mut transaction,
            &stored,
            "AI_AGENT_RUN_STATUS",
            "completed",
            "ai.core.runs.complete_run",
            stored.completed_at.unwrap_or(stored.updated_at),
        )
        .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn fail_run(
        &self,
        run_id: &str,
        error_summary: &str,
        duration_ms: i64,
        failed_event_id: &str,
    ) -> Result<AiAgentRun, AiError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE ai_agent_runs
            SET
                status = 'failed',
                error_summary = $2,
                failed_event_id = $3,
                completed_at = now(),
                duration_ms = $4,
                updated_at = now()
            WHERE run_id = $1
            RETURNING *
            "#,
        )
        .bind(run_id)
        .bind(error_summary)
        .bind(failed_event_id)
        .bind(duration_ms)
        .fetch_one(&mut *transaction)
        .await?;

        let stored = row_to_ai_agent_run(row)?;
        capture_run_observation(
            &mut transaction,
            &stored,
            "AI_AGENT_RUN_STATUS",
            "failed",
            "ai.core.runs.fail_run",
            stored.completed_at.unwrap_or(stored.updated_at),
        )
        .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn get_run(&self, run_id: &str) -> Result<Option<AiAgentRun>, AiError> {
        let run_id = validate_non_empty("run_id", run_id)?;
        let row = sqlx::query(
            r#"
            SELECT *
            FROM ai_agent_runs
            WHERE run_id = $1
            "#,
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_ai_agent_run).transpose()
    }

    pub async fn list_runs(&self, limit: i64) -> Result<Vec<AiAgentRun>, AiError> {
        let limit = validate_limit(limit)?;
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM ai_agent_runs
            ORDER BY started_at DESC, run_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_ai_agent_run).collect()
    }
}

async fn capture_run_observation(
    transaction: &mut Transaction<'_, Postgres>,
    run: &AiAgentRun,
    kind_code: &str,
    relationship_kind: &str,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), AiError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            kind_code,
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "run_id": run.run_id,
                "agent_id": run.agent_id,
                "status": run.status,
                "chat_model": run.chat_model,
                "embedding_model": run.embedding_model,
                "prompt_template_version": run.prompt_template_version,
                "model_config": run.model_config,
                "query": run.query,
                "answer": run.answer,
                "citations": run.citations,
                "error_summary": run.error_summary,
                "actor_id": run.actor_id,
                "agent_persona_id": run.agent_persona_id,
                "owner_persona_id": run.owner_persona_id,
                "causation_id": run.causation_id,
                "correlation_id": run.correlation_id,
                "requested_event_id": run.requested_event_id,
                "completed_event_id": run.completed_event_id,
                "failed_event_id": run.failed_event_id,
                "completed_at": run.completed_at,
                "duration_ms": run.duration_ms,
                "operation": relationship_kind,
            }),
            format!("ai-agent-run://{}/{}", run.run_id, relationship_kind),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "agent_run",
        run.run_id.clone(),
        relationship_kind,
        json!({
            "agent_id": run.agent_id,
            "status": run.status,
        }),
    )
    .await?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewAiRun {
    pub run_id: String,
    pub agent_id: String,
    pub chat_model: String,
    pub embedding_model: String,
    pub prompt_template_version: String,
    pub model_config: Value,
    pub query: String,
    pub actor_id: String,
    pub agent_persona_id: Option<String>,
    pub owner_persona_id: Option<String>,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
    pub requested_event_id: String,
}

impl NewAiRun {
    fn validate(&self) -> Result<(), AiError> {
        validate_non_empty("run_id", &self.run_id)?;
        validate_non_empty("agent_id", &self.agent_id)?;
        validate_non_empty("chat_model", &self.chat_model)?;
        validate_non_empty("embedding_model", &self.embedding_model)?;
        validate_non_empty("prompt_template_version", &self.prompt_template_version)?;
        validate_non_empty("query", &self.query)?;
        validate_non_empty("actor_id", &self.actor_id)?;
        if let Some(agent_persona_id) = &self.agent_persona_id {
            validate_non_empty("agent_persona_id", agent_persona_id)?;
        }
        if let Some(owner_persona_id) = &self.owner_persona_id {
            validate_non_empty("owner_persona_id", owner_persona_id)?;
        }
        validate_non_empty("requested_event_id", &self.requested_event_id)?;
        if !self.model_config.is_object() {
            return Err(AiError::InvalidRequest(
                "model_config must be a JSON object",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiAgentRun {
    pub run_id: String,
    pub agent_id: String,
    pub status: String,
    pub chat_model: String,
    pub embedding_model: String,
    pub prompt_template_version: String,
    pub model_config: Value,
    pub query: String,
    pub answer: Option<String>,
    pub citations: Value,
    pub error_summary: Option<String>,
    pub actor_id: String,
    pub agent_persona_id: Option<String>,
    pub owner_persona_id: Option<String>,
    pub causation_id: Option<String>,
    pub correlation_id: Option<String>,
    pub requested_event_id: Option<String>,
    pub completed_event_id: Option<String>,
    pub failed_event_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn row_to_ai_agent_run(row: PgRow) -> Result<AiAgentRun, AiError> {
    Ok(AiAgentRun {
        run_id: row.try_get("run_id")?,
        agent_id: row.try_get("agent_id")?,
        status: row.try_get("status")?,
        chat_model: row.try_get("chat_model")?,
        embedding_model: row.try_get("embedding_model")?,
        prompt_template_version: row.try_get("prompt_template_version")?,
        model_config: row.try_get("model_config")?,
        query: row.try_get("query")?,
        answer: row.try_get("answer")?,
        citations: row.try_get("citations")?,
        error_summary: row.try_get("error_summary")?,
        actor_id: row.try_get("actor_id")?,
        agent_persona_id: row.try_get("agent_persona_id")?,
        owner_persona_id: row.try_get("owner_persona_id")?,
        causation_id: row.try_get("causation_id")?,
        correlation_id: row.try_get("correlation_id")?,
        requested_event_id: row.try_get("requested_event_id")?,
        completed_event_id: row.try_get("completed_event_id")?,
        failed_event_id: row.try_get("failed_event_id")?,
        started_at: row.try_get("started_at")?,
        completed_at: row.try_get("completed_at")?,
        duration_ms: row.try_get("duration_ms")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
