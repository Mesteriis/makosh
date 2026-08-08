use makosh_signal_hub_api::runtime_lifecycle::{
    RuntimeLifecyclePort, RuntimeLifecyclePortError, RuntimeLifecyclePortFuture,
    RuntimeLifecycleState, RuntimeLifecycleUpdate,
};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct RuntimeLifecycleStore {
    pool: PgPool,
}

impl RuntimeLifecycleStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(
        &self,
        update: &RuntimeLifecycleUpdate,
    ) -> Result<(), RuntimeLifecyclePortError> {
        validate_update(update)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(RuntimeLifecyclePortError::new)?;
        let source_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM signal_sources WHERE code = $1)",
        )
        .bind(&update.source_code)
        .fetch_one(&mut *transaction)
        .await
        .map_err(RuntimeLifecyclePortError::new)?;

        // Runtime startup can race the idempotent fixture restoration. A later
        // runtime gate establishes the canonical state once the source exists.
        if !source_exists {
            transaction
                .commit()
                .await
                .map_err(RuntimeLifecyclePortError::new)?;
            return Ok(());
        }

        let existing_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id
            FROM signal_runtime_states
            WHERE source_code = $1
              AND connection_id IS NULL
              AND runtime_kind = $2
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(&update.source_code)
        .bind(&update.runtime_kind)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(RuntimeLifecyclePortError::new)?;

        if let Some(id) = existing_id {
            sqlx::query(
                r#"
                UPDATE signal_runtime_states
                SET
                    state = $2,
                    last_started_at = CASE
                        WHEN $2 = 'running' AND state <> 'running' THEN now()
                        ELSE last_started_at
                    END,
                    last_stopped_at = CASE
                        WHEN $2 = 'stopped' AND state <> 'stopped' THEN now()
                        ELSE last_stopped_at
                    END,
                    last_heartbeat_at = CASE WHEN $2 = 'running' THEN now() ELSE last_heartbeat_at END,
                    last_error_at = CASE WHEN $2 = 'error' THEN now() ELSE last_error_at END,
                    last_error_code = CASE WHEN $2 = 'error' THEN $3 ELSE last_error_code END,
                    last_error_message_redacted = CASE WHEN $2 = 'error' THEN $3 ELSE last_error_message_redacted END,
                    metadata = $4,
                    updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(id)
            .bind(update.state.as_str())
            .bind(&update.error_code)
            .bind(&update.metadata)
            .execute(&mut *transaction)
            .await
            .map_err(RuntimeLifecyclePortError::new)?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO signal_runtime_states (
                    id, source_code, runtime_kind, state, last_started_at,
                    last_stopped_at, last_heartbeat_at, last_error_at,
                    last_error_code, last_error_message_redacted, metadata
                )
                VALUES (
                    $1, $2, $3, $4,
                    CASE WHEN $4 = 'running' THEN now() ELSE NULL END,
                    CASE WHEN $4 = 'stopped' THEN now() ELSE NULL END,
                    CASE WHEN $4 = 'running' THEN now() ELSE NULL END,
                    CASE WHEN $4 = 'error' THEN now() ELSE NULL END,
                    CASE WHEN $4 = 'error' THEN $5 ELSE NULL END,
                    CASE WHEN $4 = 'error' THEN $5 ELSE NULL END,
                    $6
                )
                "#,
            )
            .bind(Uuid::now_v7())
            .bind(&update.source_code)
            .bind(&update.runtime_kind)
            .bind(update.state.as_str())
            .bind(&update.error_code)
            .bind(&update.metadata)
            .execute(&mut *transaction)
            .await
            .map_err(RuntimeLifecyclePortError::new)?;
        }

        if update.state == RuntimeLifecycleState::Error {
            upsert_degraded_health(&mut transaction, update).await?;
        }

        transaction
            .commit()
            .await
            .map_err(RuntimeLifecyclePortError::new)
    }
}

impl RuntimeLifecyclePort for RuntimeLifecycleStore {
    fn record_runtime_lifecycle<'a>(
        &'a self,
        update: &'a RuntimeLifecycleUpdate,
    ) -> RuntimeLifecyclePortFuture<'a, ()> {
        Box::pin(async move { self.record(update).await })
    }
}

async fn upsert_degraded_health(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    update: &RuntimeLifecycleUpdate,
) -> Result<(), RuntimeLifecyclePortError> {
    let existing = sqlx::query(
        r#"
        SELECT id, failure_count, consecutive_failure_count
        FROM signal_health
        WHERE source_code = $1 AND connection_id IS NULL
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&update.source_code)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(RuntimeLifecyclePortError::new)?;
    let evidence = json!({
        "runtime_kind": update.runtime_kind,
        "task_name": update.task_name,
        "error_code": update.error_code,
    });
    let summary = format!(
        "{} runtime task {} is degraded",
        update.source_code, update.runtime_kind
    );

    if let Some(row) = existing {
        let id: Uuid = row.try_get("id").map_err(RuntimeLifecyclePortError::new)?;
        let failure_count: i32 = row
            .try_get("failure_count")
            .map_err(RuntimeLifecyclePortError::new)?;
        let consecutive_failure_count: i32 = row
            .try_get("consecutive_failure_count")
            .map_err(RuntimeLifecyclePortError::new)?;
        sqlx::query(
            r#"
            UPDATE signal_health
            SET
                level = 'degraded',
                summary = $2,
                last_failure_at = now(),
                failure_count = $3,
                consecutive_failure_count = $4,
                next_retry_at = NULL,
                evidence = $5,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(summary)
        .bind(failure_count.saturating_add(1))
        .bind(consecutive_failure_count.saturating_add(1))
        .bind(evidence)
        .execute(&mut **transaction)
        .await
        .map_err(RuntimeLifecyclePortError::new)?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO signal_health (
                id, source_code, level, summary, last_failure_at,
                failure_count, consecutive_failure_count, evidence
            )
            VALUES ($1, $2, 'degraded', $3, now(), 1, 1, $4)
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(&update.source_code)
        .bind(summary)
        .bind(evidence)
        .execute(&mut **transaction)
        .await
        .map_err(RuntimeLifecyclePortError::new)?;
    }

    Ok(())
}

fn validate_update(update: &RuntimeLifecycleUpdate) -> Result<(), RuntimeLifecyclePortError> {
    for (name, value) in [
        ("source_code", update.source_code.as_str()),
        ("runtime_kind", update.runtime_kind.as_str()),
        ("task_name", update.task_name.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(RuntimeLifecyclePortError::new(format!(
                "{name} must not be empty"
            )));
        }
    }
    if !update.metadata.is_object() {
        return Err(RuntimeLifecyclePortError::new(
            "runtime lifecycle metadata must be an object",
        ));
    }
    if update.state == RuntimeLifecycleState::Error
        && update
            .error_code
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
    {
        return Err(RuntimeLifecyclePortError::new(
            "error lifecycle state requires an error_code",
        ));
    }
    Ok(())
}
