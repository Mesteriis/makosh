use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::postgres::Postgres;
use sqlx::{PgPool, Transaction};

use super::errors::TelegramError;
use super::evidence::link_telegram_entity_in_transaction;
use super::models::messages::TelegramProviderWriteCommand;
use super::rows::row_to_telegram_provider_write_command;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

#[path = "commands/queries.rs"]
pub mod queries;

pub const TELEGRAM_OUTBOX_WORKER_ID: &str = "telegram-outbox-worker";
const COMMAND_QUEUE_ACTOR: &str = "telegram.client.commands";

async fn capture_command_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    command: &TelegramProviderWriteCommand,
    kind_code: &str,
    relationship_kind: &str,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), TelegramError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            kind_code,
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "command_id": command.command_id,
                "account_id": command.account_id,
                "command_kind": command.command_kind,
                "idempotency_key": command.idempotency_key,
                "provider_chat_id": command.provider_chat_id,
                "provider_message_id": command.provider_message_id,
                "capability_state": command.capability_state,
                "action_class": command.action_class,
                "confirmation_decision": command.confirmation_decision,
                "status": command.status,
                "retry_count": command.retry_count,
                "max_retries": command.max_retries,
                "last_error": command.last_error,
                "result_payload": command.result_payload,
                "target_ref": command.target_ref,
                "payload": command.payload,
                "audit_metadata": command.audit_metadata,
                "actor_id": command.actor_id,
                "next_attempt_at": command.next_attempt_at,
                "last_attempt_at": command.last_attempt_at,
                "locked_at": command.locked_at,
                "locked_by": command.locked_by,
                "provider_observed_at": command.provider_observed_at,
                "provider_state": command.provider_state,
                "reconciliation_status": command.reconciliation_status,
                "reconciled_at": command.reconciled_at,
                "dead_lettered_at": command.dead_lettered_at,
                "completed_at": command.completed_at,
                "operation": relationship_kind,
            }),
            match kind_code {
                "TELEGRAM_PROVIDER_WRITE_COMMAND" => {
                    format!("telegram-provider-command://{}", command.command_id)
                }
                _ => format!(
                    "telegram-provider-command://{}/status/{}",
                    command.command_id, relationship_kind
                ),
            },
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
            "provider": "telegram",
        })),
    )
    .await?;
    link_telegram_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "provider_write_command",
        command.command_id.clone(),
        relationship_kind,
        json!({
            "command_kind": command.command_kind,
            "status": command.status,
            "reconciliation_status": command.reconciliation_status,
            "provider_chat_id": command.provider_chat_id,
            "provider_message_id": command.provider_message_id,
        }),
    )
    .await?;
    Ok(())
}

async fn fetch_command_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
) -> Result<TelegramProviderWriteCommand, TelegramError> {
    let row = sqlx::query("SELECT * FROM telegram_provider_write_commands WHERE command_id = $1")
        .bind(command_id)
        .fetch_one(&mut **transaction)
        .await?;
    row_to_telegram_provider_write_command(row)
}

fn stable_short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())[..12].to_owned()
}

pub fn new_command_id() -> String {
    let now = Utc::now();
    format!(
        "tcmd_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("cmd_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_command(
    pool: &PgPool,
    command_id: &str,
    account_id: &str,
    command_kind: &str,
    idempotency_key: &str,
    provider_chat_id: &str,
    provider_message_id: Option<&str>,
    capability_state: &str,
    action_class: &str,
    confirmation_decision: &str,
    actor_id: &str,
    payload: serde_json::Value,
    target_ref: serde_json::Value,
    audit_metadata: serde_json::Value,
) -> Result<TelegramProviderWriteCommand, TelegramError> {
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO telegram_provider_write_commands
            (command_id, account_id, command_kind, idempotency_key, provider_chat_id,
             provider_message_id, capability_state, action_class, confirmation_decision,
             status, retry_count, max_retries, actor_id, payload, target_ref, audit_metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'queued', 0, 3, $10, $11, $12, $13)
        "#,
    )
    .bind(command_id)
    .bind(account_id)
    .bind(command_kind)
    .bind(idempotency_key)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .bind(capability_state)
    .bind(action_class)
    .bind(confirmation_decision)
    .bind(actor_id)
    .bind(&payload)
    .bind(&target_ref)
    .bind(&audit_metadata)
    .execute(&mut *transaction)
    .await?;
    let command = fetch_command_in_transaction(&mut transaction, command_id).await?;
    capture_command_observation_in_transaction(
        &mut transaction,
        &command,
        "TELEGRAM_PROVIDER_WRITE_COMMAND",
        "queued",
        COMMAND_QUEUE_ACTOR,
        command.happened_at,
    )
    .await?;
    transaction.commit().await?;
    Ok(command)
}

pub async fn update_command_status(
    pool: &PgPool,
    command_id: &str,
    status: &str,
    result_payload: serde_json::Value,
    last_error: Option<&str>,
    completed_at: Option<chrono::DateTime<Utc>>,
) -> Result<(), TelegramError> {
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = $2, result_payload = $3, last_error = $4,
            completed_at = $5, updated_at = now()
        WHERE command_id = $1
          AND EXISTS (
                SELECT 1 FROM provider_runtime_leases lease
                WHERE lease.provider = 'telegram'
                  AND lease.account_id = telegram_provider_write_commands.account_id
                  AND lease.state = 'active'
                  AND lease.epoch = telegram_provider_write_commands.lease_epoch
                  AND lease.expires_at > COALESCE($5, now())
          )
        "#,
    )
    .bind(command_id)
    .bind(status)
    .bind(&result_payload)
    .bind(last_error)
    .bind(completed_at)
    .execute(&mut *transaction)
    .await?;
    let command = fetch_command_in_transaction(&mut transaction, command_id).await?;
    capture_command_observation_in_transaction(
        &mut transaction,
        &command,
        "TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS",
        "status_updated",
        COMMAND_QUEUE_ACTOR,
        command.updated_at,
    )
    .await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn retry_command(pool: &PgPool, command_id: &str) -> Result<(), TelegramError> {
    schedule_command_retry(
        pool,
        command_id,
        Utc::now(),
        Utc::now() + Duration::seconds(30),
        "Telegram provider command retry scheduled",
    )
    .await
}

pub async fn schedule_command_retry(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
    next_attempt_at: DateTime<Utc>,
    error_message: &str,
) -> Result<(), TelegramError> {
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = 'retrying',
            next_attempt_at = $3,
            locked_at = NULL,
            locked_by = NULL,
            last_error = $4,
            reconciliation_status = 'not_observed',
            updated_at = $2
        WHERE command_id = $1
          AND EXISTS (
                SELECT 1 FROM provider_runtime_leases lease
                WHERE lease.provider = 'telegram'
                  AND lease.account_id = telegram_provider_write_commands.account_id
                  AND lease.state = 'active'
                  AND lease.epoch = telegram_provider_write_commands.lease_epoch
                  AND lease.expires_at > $2
          )
          AND status = 'executing'
        "#,
    )
    .bind(command_id)
    .bind(now)
    .bind(next_attempt_at)
    .bind(error_message)
    .execute(&mut *transaction)
    .await?;
    let command = fetch_command_in_transaction(&mut transaction, command_id).await?;
    capture_command_observation_in_transaction(
        &mut transaction,
        &command,
        "TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS",
        "retry_scheduled",
        COMMAND_QUEUE_ACTOR,
        now,
    )
    .await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn dead_letter_command(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
    error_message: &str,
) -> Result<(), TelegramError> {
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = 'dead_letter',
            locked_at = NULL,
            locked_by = NULL,
            last_error = $3,
            dead_lettered_at = $2,
            updated_at = $2
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .bind(now)
    .bind(error_message)
    .execute(&mut *transaction)
    .await?;
    let command = fetch_command_in_transaction(&mut transaction, command_id).await?;
    capture_command_observation_in_transaction(
        &mut transaction,
        &command,
        "TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS",
        "dead_lettered",
        COMMAND_QUEUE_ACTOR,
        now,
    )
    .await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn mark_command_awaiting_provider(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
    result_payload: serde_json::Value,
) -> Result<(), TelegramError> {
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = 'executing',
            result_payload = $3,
            last_error = NULL,
            reconciliation_status = 'awaiting_provider',
            locked_at = NULL,
            locked_by = NULL,
            updated_at = $2
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .bind(now)
    .bind(&result_payload)
    .execute(&mut *transaction)
    .await?;
    let command = fetch_command_in_transaction(&mut transaction, command_id).await?;
    capture_command_observation_in_transaction(
        &mut transaction,
        &command,
        "TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS",
        "awaiting_provider",
        COMMAND_QUEUE_ACTOR,
        now,
    )
    .await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn mark_command_reconciled(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
    provider_state: serde_json::Value,
    result_payload: serde_json::Value,
) -> Result<TelegramProviderWriteCommand, TelegramError> {
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = 'completed',
            result_payload = $3,
            last_error = NULL,
            provider_observed_at = $2,
            provider_state = $4,
            reconciliation_status = 'observed',
            reconciled_at = $2,
            completed_at = $2,
            locked_at = NULL,
            locked_by = NULL,
            next_attempt_at = NULL,
            dead_lettered_at = NULL,
            updated_at = $2
        WHERE command_id = $1
          AND EXISTS (
                SELECT 1 FROM provider_runtime_leases lease
                WHERE lease.provider = 'telegram'
                  AND lease.account_id = telegram_provider_write_commands.account_id
                  AND lease.state = 'active'
                  AND lease.epoch = telegram_provider_write_commands.lease_epoch
                  AND lease.expires_at > $2
          )
        RETURNING *
        "#,
    )
    .bind(command_id)
    .bind(now)
    .bind(&result_payload)
    .bind(&provider_state)
    .fetch_one(&mut *transaction)
    .await?;
    let command = row_to_telegram_provider_write_command(row)?;
    capture_command_observation_in_transaction(
        &mut transaction,
        &command,
        "TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS",
        "reconciled",
        COMMAND_QUEUE_ACTOR,
        now,
    )
    .await?;
    transaction.commit().await?;
    Ok(command)
}

pub async fn mark_command_mismatch(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
    provider_state: serde_json::Value,
    result_payload: serde_json::Value,
    error_message: &str,
) -> Result<TelegramProviderWriteCommand, TelegramError> {
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = 'failed',
            result_payload = $3,
            last_error = $4,
            provider_observed_at = $2,
            provider_state = $5,
            reconciliation_status = 'mismatch',
            reconciled_at = $2,
            completed_at = NULL,
            locked_at = NULL,
            locked_by = NULL,
            next_attempt_at = NULL,
            dead_lettered_at = NULL,
            updated_at = $2
        WHERE command_id = $1
          AND EXISTS (
                SELECT 1 FROM provider_runtime_leases lease
                WHERE lease.provider = 'telegram'
                  AND lease.account_id = telegram_provider_write_commands.account_id
                  AND lease.state = 'active'
                  AND lease.epoch = telegram_provider_write_commands.lease_epoch
                  AND lease.expires_at > $2
          )
        RETURNING *
        "#,
    )
    .bind(command_id)
    .bind(now)
    .bind(&result_payload)
    .bind(error_message)
    .bind(&provider_state)
    .fetch_one(&mut *transaction)
    .await?;
    let command = row_to_telegram_provider_write_command(row)?;
    capture_command_observation_in_transaction(
        &mut transaction,
        &command,
        "TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS",
        "mismatch",
        COMMAND_QUEUE_ACTOR,
        now,
    )
    .await?;
    transaction.commit().await?;
    Ok(command)
}

pub async fn manual_retry_command(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
) -> Result<Option<TelegramProviderWriteCommand>, TelegramError> {
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = 'retrying',
            retry_count = 0,
            next_attempt_at = $2,
            last_attempt_at = NULL,
            locked_at = NULL,
            locked_by = NULL,
            provider_observed_at = NULL,
            provider_state = '{}'::jsonb,
            reconciliation_status = 'not_observed',
            reconciled_at = NULL,
            dead_lettered_at = NULL,
            completed_at = NULL,
            last_error = NULL,
            updated_at = $2
        WHERE command_id = $1
          AND status IN ('failed', 'dead_letter', 'retrying')
        RETURNING *
        "#,
    )
    .bind(command_id)
    .bind(now)
    .fetch_optional(&mut *transaction)
    .await?;
    let command = row
        .map(row_to_telegram_provider_write_command)
        .transpose()?;
    if let Some(command) = &command {
        capture_command_observation_in_transaction(
            &mut transaction,
            command,
            "TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS",
            "manual_retry",
            COMMAND_QUEUE_ACTOR,
            now,
        )
        .await?;
    }
    transaction.commit().await?;
    Ok(command)
}

/// Atomically claim commands eligible for provider execution.
///
/// Claimed rows transition to `executing` and increment retry_count before the
/// actor call. `completed` is reserved for provider-observed state.
pub async fn claim_due_commands_for_execution(
    pool: &PgPool,
    account_id: &str,
    now: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let mut transaction = pool.begin().await?;
    let rows = sqlx::query(
        r#"
        WITH due AS (
            SELECT command_id
            FROM telegram_provider_write_commands
            WHERE account_id = $1
              AND status IN ('queued', 'retrying')
              AND retry_count < max_retries
              AND (next_attempt_at IS NULL OR next_attempt_at <= $2)
              AND confirmation_decision IN ('confirmed', 'not_required')
              AND capability_state IN ('available', 'degraded')
              AND command_kind IN (
                  'send_text', 'send_media', 'reply', 'forward',
                  'edit', 'delete', 'react', 'unreact', 'pin', 'unpin',
                  'mark_read', 'mark_unread', 'archive', 'unarchive',
                  'mute', 'unmute', 'join', 'leave', 'folder_add', 'folder_remove',
                  'admin_action'
              )
            ORDER BY COALESCE(next_attempt_at, created_at) ASC, created_at ASC, command_id ASC
            LIMIT $3
            FOR UPDATE SKIP LOCKED
        )
        UPDATE telegram_provider_write_commands command
        SET status = 'executing',
            retry_count = command.retry_count + 1,
            last_attempt_at = $2,
            locked_at = $2,
            locked_by = $4,
            last_error = NULL,
            reconciliation_status = 'awaiting_provider',
            updated_at = $2
        FROM due
        WHERE command.command_id = due.command_id
        RETURNING command.*
        "#,
    )
    .bind(account_id)
    .bind(now)
    .bind(limit)
    .bind(TELEGRAM_OUTBOX_WORKER_ID)
    .fetch_all(&mut *transaction)
    .await?;
    let commands = rows
        .into_iter()
        .map(row_to_telegram_provider_write_command)
        .collect::<Result<Vec<_>, _>>()?;
    for command in &commands {
        capture_command_observation_in_transaction(
            &mut transaction,
            command,
            "TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS",
            "claimed_for_execution",
            COMMAND_QUEUE_ACTOR,
            now,
        )
        .await?;
    }
    transaction.commit().await?;
    Ok(commands)
}

pub async fn recover_stale_executing_commands(
    pool: &PgPool,
    now: DateTime<Utc>,
    stale_before: DateTime<Utc>,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let mut transaction = pool.begin().await?;
    let rows = sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = CASE
                WHEN retry_count >= max_retries THEN 'dead_letter'
                ELSE 'retrying'
            END,
            next_attempt_at = CASE
                WHEN retry_count >= max_retries THEN next_attempt_at
                ELSE $1
            END,
            locked_at = NULL,
            locked_by = NULL,
            last_error = 'Telegram provider command execution was interrupted before provider reconciliation',
            reconciliation_status = 'not_observed',
            dead_lettered_at = CASE
                WHEN retry_count >= max_retries THEN $1
                ELSE dead_lettered_at
            END,
            updated_at = $1
        WHERE status = 'executing'
          AND locked_at IS NOT NULL
          AND locked_at <= $2
        RETURNING *
        "#,
    )
    .bind(now)
    .bind(stale_before)
    .fetch_all(&mut *transaction)
    .await?;
    let commands = rows
        .into_iter()
        .map(row_to_telegram_provider_write_command)
        .collect::<Result<Vec<_>, _>>()?;
    for command in &commands {
        capture_command_observation_in_transaction(
            &mut transaction,
            command,
            "TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS",
            "stale_recovered",
            COMMAND_QUEUE_ACTOR,
            now,
        )
        .await?;
    }
    transaction.commit().await?;
    Ok(commands)
}
