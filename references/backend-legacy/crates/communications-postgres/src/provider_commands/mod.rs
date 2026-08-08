use chrono::{DateTime, Utc};
use makosh_communications_api::commands::{
    CommunicationProviderCommand, CommunicationProviderCommandDiagnostic,
    CommunicationProviderCommandDiagnostics, CommunicationProviderCommandStatusCount,
    NewCommunicationProviderCommand, ProviderCommandMirrorPort, ProviderCommandMirrorPortFuture,
    ProviderCommandProjectionPort, ProviderCommandQueuePort, ProviderCommandQueuePortError,
    ProviderCommandQueuePortFuture,
};
use serde_json::Value;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use self::events::{
    EVENT_COMPLETED, EVENT_EXECUTING, EVENT_FAILED, EVENT_REQUESTED, EVENT_RETRY_REQUESTED,
    append_provider_command_event, append_provider_command_events,
};
use makosh_events_postgres::errors::EventStoreError;

mod diagnostics;
mod events;

#[derive(Clone)]
pub struct CommunicationProviderCommandStore {
    pool: PgPool,
}

impl CommunicationProviderCommandStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn mirror_runtime_command(
        &self,
        command: &CommunicationProviderCommand,
    ) -> Result<(), CommunicationProviderCommandError> {
        sqlx::query(
            r#"
            INSERT INTO communication_provider_commands (
                command_id, account_id, channel_kind, command_kind, idempotency_key,
                provider_conversation_id, provider_message_id, target_ref, payload,
                capability_state, action_class, confirmation_decision, status,
                retry_count, max_retries, last_error, result_payload, audit_metadata,
                provider_state, reconciliation_status, next_attempt_at, last_attempt_at,
                provider_observed_at, reconciled_at, dead_lettered_at, actor_id,
                happened_at, completed_at, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25,
                $26, $27, $28, $29, $30
            )
            ON CONFLICT (account_id, idempotency_key)
            DO UPDATE SET
                command_kind = EXCLUDED.command_kind,
                provider_conversation_id = EXCLUDED.provider_conversation_id,
                provider_message_id = EXCLUDED.provider_message_id,
                target_ref = EXCLUDED.target_ref,
                payload = EXCLUDED.payload,
                capability_state = EXCLUDED.capability_state,
                action_class = EXCLUDED.action_class,
                confirmation_decision = EXCLUDED.confirmation_decision,
                status = EXCLUDED.status,
                retry_count = EXCLUDED.retry_count,
                max_retries = EXCLUDED.max_retries,
                last_error = EXCLUDED.last_error,
                result_payload = EXCLUDED.result_payload,
                audit_metadata = EXCLUDED.audit_metadata,
                provider_state = EXCLUDED.provider_state,
                reconciliation_status = EXCLUDED.reconciliation_status,
                next_attempt_at = EXCLUDED.next_attempt_at,
                last_attempt_at = EXCLUDED.last_attempt_at,
                provider_observed_at = EXCLUDED.provider_observed_at,
                reconciled_at = EXCLUDED.reconciled_at,
                dead_lettered_at = EXCLUDED.dead_lettered_at,
                completed_at = EXCLUDED.completed_at,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&command.command_id)
        .bind(&command.account_id)
        .bind(&command.channel_kind)
        .bind(&command.command_kind)
        .bind(&command.idempotency_key)
        .bind(&command.provider_conversation_id)
        .bind(&command.provider_message_id)
        .bind(&command.target_ref)
        .bind(&command.payload)
        .bind(&command.capability_state)
        .bind(&command.action_class)
        .bind(&command.confirmation_decision)
        .bind(&command.status)
        .bind(command.retry_count)
        .bind(command.max_retries)
        .bind(&command.last_error)
        .bind(&command.result_payload)
        .bind(&command.audit_metadata)
        .bind(&command.provider_state)
        .bind(&command.reconciliation_status)
        .bind(command.next_attempt_at)
        .bind(command.last_attempt_at)
        .bind(command.provider_observed_at)
        .bind(command.reconciled_at)
        .bind(command.dead_lettered_at)
        .bind(&command.actor_id)
        .bind(command.happened_at)
        .bind(command.completed_at)
        .bind(command.created_at)
        .bind(command.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn enqueue(
        &self,
        command: &NewCommunicationProviderCommand,
    ) -> Result<CommunicationProviderCommand, CommunicationProviderCommandError> {
        let mut transaction = self.pool.begin().await?;
        let stored = Self::enqueue_in_transaction(&mut transaction, command).await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn enqueue_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        command: &NewCommunicationProviderCommand,
    ) -> Result<CommunicationProviderCommand, CommunicationProviderCommandError> {
        command
            .validate()
            .map_err(|error| CommunicationProviderCommandError::Invalid(error.to_string()))?;
        ensure_canonical_communication_account_in_transaction(transaction, &command.account_id)
            .await?;
        let row = sqlx::query(&provider_command_returning_sql(
            r#"
            INSERT INTO communication_provider_commands (
                command_id, account_id, channel_kind, command_kind, idempotency_key,
                provider_conversation_id, provider_message_id, target_ref, payload,
                capability_state, action_class, confirmation_decision, status,
                retry_count, max_retries, last_error, result_payload, audit_metadata,
                actor_id, happened_at, completed_at, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12, 'queued',
                0, $13, NULL, '{}'::jsonb, '{}'::jsonb,
                $14, now(), NULL, now(), now()
            )
            ON CONFLICT (account_id, idempotency_key)
            DO UPDATE SET updated_at = communication_provider_commands.updated_at
            "#,
        ))
        .bind(command.command_id.trim())
        .bind(command.account_id.trim())
        .bind(command.channel_kind.trim())
        .bind(command.command_kind.trim())
        .bind(command.idempotency_key.trim())
        .bind(command.provider_conversation_id.as_deref())
        .bind(command.provider_message_id.as_deref())
        .bind(&command.target_ref)
        .bind(&command.payload)
        .bind(command.capability_state.trim())
        .bind(command.action_class.trim())
        .bind(command.confirmation_decision.trim())
        .bind(command.max_retries)
        .bind(command.actor_id.trim())
        .fetch_one(&mut **transaction)
        .await?;

        let stored = row_to_provider_command(row)?;
        append_provider_command_event(transaction, EVENT_REQUESTED, &stored, stored.created_at)
            .await?;
        Ok(stored)
    }

    pub async fn claim_due(
        &self,
        account_id: &str,
        channel_kind: &str,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        let limit = limit.clamp(1, 100);
        let mut transaction = self.pool.begin().await?;
        let rows = sqlx::query(&provider_command_returning_sql_with_alias(
            r#"
            UPDATE communication_provider_commands command
            SET status = 'executing',
                retry_count = retry_count + 1,
                last_error = NULL,
                last_attempt_at = $3,
                next_attempt_at = NULL,
                updated_at = $3
            FROM (
                SELECT command_id
                FROM communication_provider_commands
                WHERE account_id = $1
                  AND channel_kind = $2
                  AND status IN ('queued', 'retrying')
                  AND retry_count < max_retries
                  AND (next_attempt_at IS NULL OR next_attempt_at <= $3)
                ORDER BY updated_at ASC, command_id ASC
                LIMIT $4
                FOR UPDATE SKIP LOCKED
            ) due
            WHERE command.command_id = due.command_id
            "#,
            "command",
        ))
        .bind(account_id.trim())
        .bind(channel_kind.trim())
        .bind(now)
        .bind(limit)
        .fetch_all(&mut *transaction)
        .await?;
        let commands = rows
            .into_iter()
            .map(row_to_provider_command)
            .collect::<Result<Vec<_>, _>>()?;
        append_provider_command_events(&mut transaction, EVENT_EXECUTING, &commands, now).await?;
        transaction.commit().await?;
        Ok(commands)
    }

    pub async fn recover_stale_executing(
        &self,
        account_id: &str,
        channel_kind: &str,
        now: DateTime<Utc>,
        execution_lease: chrono::Duration,
    ) -> Result<Vec<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        if execution_lease <= chrono::Duration::zero() {
            return Err(CommunicationProviderCommandError::Invalid(
                "execution_lease must be greater than zero".to_owned(),
            ));
        }
        let stale_before = now.checked_sub_signed(execution_lease).ok_or_else(|| {
            CommunicationProviderCommandError::Invalid(
                "execution_lease is outside the supported timestamp range".to_owned(),
            )
        })?;
        let mut transaction = self.pool.begin().await?;
        let rows = sqlx::query(&provider_command_returning_sql(
            r#"
            UPDATE communication_provider_commands
            SET status = CASE
                    WHEN retry_count >= max_retries THEN 'dead_letter'
                    ELSE 'retrying'
                END,
                next_attempt_at = CASE
                    WHEN retry_count >= max_retries THEN NULL
                    ELSE $3
                END,
                dead_lettered_at = CASE
                    WHEN retry_count >= max_retries THEN $3
                    ELSE NULL
                END,
                last_error = 'provider command execution lease expired before completion',
                result_payload = result_payload || jsonb_build_object(
                    'kind', 'execution_lease_expired',
                    'retryable', retry_count < max_retries
                ),
                updated_at = $3
            WHERE account_id = $1
              AND channel_kind = $2
              AND status = 'executing'
              AND last_attempt_at < $4
            "#,
        ))
        .bind(account_id.trim())
        .bind(channel_kind.trim())
        .bind(now)
        .bind(stale_before)
        .fetch_all(&mut *transaction)
        .await?;
        let commands = rows
            .into_iter()
            .map(row_to_provider_command)
            .collect::<Result<Vec<_>, _>>()?;
        append_provider_command_events(&mut transaction, EVENT_FAILED, &commands, now).await?;
        transaction.commit().await?;
        Ok(commands)
    }

    pub async fn mark_completed(
        &self,
        command_id: &str,
        channel_kind: &str,
        now: DateTime<Utc>,
        result_payload: Value,
    ) -> Result<Option<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        self.mark_completed_at_epoch(command_id, channel_kind, now, result_payload, None)
            .await
    }

    pub async fn mark_completed_at_epoch(
        &self,
        command_id: &str,
        channel_kind: &str,
        now: DateTime<Utc>,
        result_payload: Value,
        lease_epoch: Option<i64>,
    ) -> Result<Option<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("command_id", command_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        if !result_payload.is_object() {
            return Err(CommunicationProviderCommandError::Invalid(
                "result_payload must be a JSON object".to_owned(),
            ));
        }
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(&provider_command_returning_sql(
            r#"
            UPDATE communication_provider_commands
            SET status = 'completed',
                provider_message_id = COALESCE(
                    provider_message_id,
                    NULLIF($4 #>> '{provider_message_id}', ''),
                    NULLIF($4 #>> '{message_id}', '')
                ),
                completed_at = $3,
                result_payload = $4,
                reconciliation_status = CASE
                    WHEN COALESCE(
                        provider_message_id,
                        NULLIF($4 #>> '{provider_message_id}', ''),
                        NULLIF($4 #>> '{message_id}', '')
                    ) IS NULL THEN reconciliation_status
                    ELSE 'awaiting_provider'
                END,
                last_error = NULL,
                updated_at = $3
            WHERE command_id = $1
              AND channel_kind = $2
              AND status = 'executing'
              AND ($5::bigint IS NULL OR lease_epoch = $5)
            "#,
        ))
        .bind(command_id.trim())
        .bind(channel_kind.trim())
        .bind(now)
        .bind(result_payload)
        .bind(lease_epoch)
        .fetch_optional(&mut *transaction)
        .await?;
        let command = row.map(row_to_provider_command).transpose()?;
        if let Some(command) = &command {
            append_provider_command_event(&mut transaction, EVENT_COMPLETED, command, now).await?;
        }
        transaction.commit().await?;
        Ok(command)
    }

    pub async fn mark_observed_by_provider_message(
        &self,
        account_id: &str,
        channel_kind: &str,
        provider_message_id: &str,
        command_kinds: &[&str],
        observed_at: DateTime<Utc>,
        provider_state: Value,
    ) -> Result<Vec<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        validate_non_empty("provider_message_id", provider_message_id)?;
        if command_kinds.is_empty() {
            return Err(CommunicationProviderCommandError::Invalid(
                "command_kinds must not be empty".to_owned(),
            ));
        }
        for command_kind in command_kinds {
            validate_non_empty("command_kind", command_kind)?;
        }
        if !provider_state.is_object() {
            return Err(CommunicationProviderCommandError::Invalid(
                "provider_state must be a JSON object".to_owned(),
            ));
        }
        let command_kinds = command_kinds
            .iter()
            .map(|command_kind| command_kind.trim().to_owned())
            .collect::<Vec<_>>();
        let mut transaction = self.pool.begin().await?;
        let rows = sqlx::query(&provider_command_returning_sql(
            r#"
            UPDATE communication_provider_commands
            SET status = 'completed',
                provider_message_id = COALESCE(provider_message_id, $3),
                result_payload = result_payload || jsonb_build_object(
                    'provider_observed', true,
                    'provider_message_id', $3
                ),
                provider_state = provider_state || $6,
                reconciliation_status = 'observed',
                provider_observed_at = $5,
                reconciled_at = $5,
                completed_at = COALESCE(completed_at, $5),
                last_error = NULL,
                updated_at = $5
            WHERE account_id = $1
              AND channel_kind = $2
              AND command_kind = ANY($4::text[])
              AND status IN ('queued', 'retrying', 'completed', 'executing')
              AND reconciliation_status <> 'observed'
              AND (
                    provider_message_id = $3
                    OR result_payload #>> '{provider_message_id}' = $3
                    OR result_payload #>> '{message_id}' = $3
              )
              AND (
                    (NOT ($6 ? 'is_read' OR $6 ? 'starred'))
                    OR (
                        ($6 ? 'is_read' AND (
                            (command_kind = 'mark_read' AND ($6->>'is_read')::boolean)
                            OR (command_kind = 'mark_unread' AND NOT ($6->>'is_read')::boolean)
                        ))
                        OR ($6 ? 'starred' AND (
                            (command_kind = 'star' AND ($6->>'starred')::boolean)
                            OR (command_kind = 'unstar' AND NOT ($6->>'starred')::boolean)
                        ))
                    )
              )
              AND EXISTS (
                    SELECT 1
                    FROM provider_runtime_leases lease
                    WHERE lease.provider = $2
                      AND lease.account_id = communication_provider_commands.account_id
                      AND lease.state = 'active'
                      AND lease.epoch = communication_provider_commands.lease_epoch
                      AND lease.expires_at > $5
              )
            "#,
        ))
        .bind(account_id.trim())
        .bind(channel_kind.trim())
        .bind(provider_message_id.trim())
        .bind(command_kinds)
        .bind(observed_at)
        .bind(provider_state)
        .fetch_all(&mut *transaction)
        .await?;
        let commands = rows
            .into_iter()
            .map(row_to_provider_command)
            .collect::<Result<Vec<_>, _>>()?;
        append_provider_command_events(&mut transaction, EVENT_COMPLETED, &commands, observed_at)
            .await?;
        transaction.commit().await?;
        Ok(commands)
    }

    pub async fn mark_failed(
        &self,
        command_id: &str,
        channel_kind: &str,
        now: DateTime<Utc>,
        error: &str,
        result_payload: Value,
    ) -> Result<Option<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        self.mark_failed_at_epoch(command_id, channel_kind, now, error, result_payload, None)
            .await
    }

    pub async fn mark_failed_at_epoch(
        &self,
        command_id: &str,
        channel_kind: &str,
        now: DateTime<Utc>,
        error: &str,
        result_payload: Value,
        lease_epoch: Option<i64>,
    ) -> Result<Option<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("command_id", command_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        validate_non_empty("error", error)?;
        if !result_payload.is_object() {
            return Err(CommunicationProviderCommandError::Invalid(
                "result_payload must be a JSON object".to_owned(),
            ));
        }
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(&provider_command_returning_sql(
            r#"
            UPDATE communication_provider_commands
            SET status = CASE
                    WHEN retry_count >= max_retries THEN 'dead_letter'
                    ELSE 'retrying'
                END,
                next_attempt_at = CASE
                    WHEN retry_count >= max_retries THEN NULL
                    ELSE $3 + (
                        LEAST(
                            3600::DOUBLE PRECISION,
                            5::DOUBLE PRECISION * power(
                                2::DOUBLE PRECISION,
                                GREATEST(retry_count - 1, 0)::DOUBLE PRECISION
                            )
                        )
                        + (
                            mod(
                                mod(hashtextextended(command_id || ':' || retry_count::TEXT, 0), 1000)
                                    + 1000,
                                1000
                            )::DOUBLE PRECISION / 1000
                        )
                    ) * INTERVAL '1 second'
                END,
                dead_lettered_at = CASE
                    WHEN retry_count >= max_retries THEN $3
                    ELSE NULL
                END,
                last_error = $4,
                result_payload = $5,
                updated_at = $3
            WHERE command_id = $1
              AND channel_kind = $2
              AND status = 'executing'
              AND ($6::bigint IS NULL OR lease_epoch = $6)
            "#,
        ))
        .bind(command_id.trim())
        .bind(channel_kind.trim())
        .bind(now)
        .bind(error.trim())
        .bind(result_payload)
        .bind(lease_epoch)
        .fetch_optional(&mut *transaction)
        .await?;
        let command = row.map(row_to_provider_command).transpose()?;
        if let Some(command) = &command {
            append_provider_command_event(&mut transaction, EVENT_FAILED, command, now).await?;
        }
        transaction.commit().await?;
        Ok(command)
    }

    pub async fn mark_terminal_failed(
        &self,
        command_id: &str,
        channel_kind: &str,
        now: DateTime<Utc>,
        error: &str,
        result_payload: Value,
    ) -> Result<Option<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        self.mark_terminal_failed_at_epoch(
            command_id,
            channel_kind,
            now,
            error,
            result_payload,
            None,
        )
        .await
    }

    pub async fn mark_terminal_failed_at_epoch(
        &self,
        command_id: &str,
        channel_kind: &str,
        now: DateTime<Utc>,
        error: &str,
        result_payload: Value,
        lease_epoch: Option<i64>,
    ) -> Result<Option<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("command_id", command_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        validate_non_empty("error", error)?;
        if !result_payload.is_object() {
            return Err(CommunicationProviderCommandError::Invalid(
                "result_payload must be a JSON object".to_owned(),
            ));
        }
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(&provider_command_returning_sql(
            r#"
            UPDATE communication_provider_commands
            SET status = 'dead_letter',
                next_attempt_at = NULL,
                dead_lettered_at = $3,
                last_error = $4,
                result_payload = $5,
                updated_at = $3
            WHERE command_id = $1
              AND channel_kind = $2
              AND status = 'executing'
              AND ($6::bigint IS NULL OR lease_epoch = $6)
            "#,
        ))
        .bind(command_id.trim())
        .bind(channel_kind.trim())
        .bind(now)
        .bind(error.trim())
        .bind(result_payload)
        .bind(lease_epoch)
        .fetch_optional(&mut *transaction)
        .await?;
        let command = row.map(row_to_provider_command).transpose()?;
        if let Some(command) = &command {
            append_provider_command_event(&mut transaction, EVENT_FAILED, command, now).await?;
        }
        transaction.commit().await?;
        Ok(command)
    }

    pub async fn manual_retry(
        &self,
        command_id: &str,
        channel_kind: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("command_id", command_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(&provider_command_returning_sql(
            r#"
            UPDATE communication_provider_commands
            SET status = 'retrying',
                retry_count = 0,
                completed_at = NULL,
                last_error = NULL,
                reconciliation_status = 'not_observed',
                next_attempt_at = $3,
                last_attempt_at = NULL,
                provider_observed_at = NULL,
                reconciled_at = NULL,
                dead_lettered_at = NULL,
                updated_at = $3
            WHERE command_id = $1
              AND channel_kind = $2
              AND status IN ('failed', 'dead_letter')
            "#,
        ))
        .bind(command_id.trim())
        .bind(channel_kind.trim())
        .bind(now)
        .fetch_optional(&mut *transaction)
        .await?;
        let command = row.map(row_to_provider_command).transpose()?;
        if let Some(command) = &command {
            append_provider_command_event(&mut transaction, EVENT_RETRY_REQUESTED, command, now)
                .await?;
        }
        transaction.commit().await?;
        Ok(command)
    }

    pub async fn list(
        &self,
        account_id: &str,
        channel_kind: &str,
        limit: i64,
    ) -> Result<Vec<CommunicationProviderCommand>, CommunicationProviderCommandError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("channel_kind", channel_kind)?;
        let rows = sqlx::query(
            r#"
            SELECT
                command_id, account_id, channel_kind, command_kind, idempotency_key,
                provider_conversation_id, provider_message_id, target_ref, payload,
                capability_state, action_class, confirmation_decision, status,
                retry_count, max_retries, last_error, result_payload, audit_metadata,
                provider_state, reconciliation_status, next_attempt_at, last_attempt_at,
                provider_observed_at, reconciled_at, dead_lettered_at,
                actor_id, happened_at, completed_at, created_at, updated_at
            FROM communication_provider_commands
            WHERE account_id = $1
              AND channel_kind = $2
            ORDER BY updated_at DESC, command_id ASC
            LIMIT $3
            "#,
        )
        .bind(account_id.trim())
        .bind(channel_kind.trim())
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_provider_command).collect()
    }
}

impl ProviderCommandMirrorPort for CommunicationProviderCommandStore {
    fn mirror<'a>(
        &'a self,
        command: &'a CommunicationProviderCommand,
    ) -> ProviderCommandMirrorPortFuture<'a, ()> {
        Box::pin(async move {
            self.mirror_runtime_command(command)
                .await
                .map_err(ProviderCommandQueuePortError::new)
        })
    }
}

impl ProviderCommandProjectionPort for CommunicationProviderCommandStore {
    fn list_for_runtime_import<'a>(
        &'a self,
        channel_kind: &'a str,
        limit: i64,
    ) -> ProviderCommandQueuePortFuture<'a, Vec<CommunicationProviderCommand>> {
        Box::pin(async move {
            let rows = sqlx::query(
                r#"
                SELECT
                    command.command_id, command.account_id, command.channel_kind,
                    command.command_kind, command.idempotency_key,
                    command.provider_conversation_id, command.provider_message_id,
                    command.target_ref, command.payload, command.capability_state,
                    command.action_class, command.confirmation_decision, command.status,
                    command.retry_count, command.max_retries, command.last_error,
                    command.result_payload, command.audit_metadata, command.provider_state,
                    command.reconciliation_status, command.next_attempt_at,
                    command.last_attempt_at, command.provider_observed_at,
                    command.reconciled_at, command.dead_lettered_at, command.actor_id,
                    command.happened_at, command.completed_at, command.created_at,
                    command.updated_at
                FROM communication_provider_commands command
                JOIN communication_provider_accounts account
                  ON account.account_id = command.account_id
                WHERE command.channel_kind = $1
                  AND account.provider_kind = 'whatsapp_web'
                  AND COALESCE(account.config->>'runtime', '') NOT IN ('', 'fixture')
                  AND command.status IN ('queued', 'retrying', 'confirmed')
                  AND command.command_kind IN (
                    'send_text', 'reply', 'forward', 'send_media',
                    'download_media', 'send_voice_note', 'edit', 'delete',
                    'react', 'unreact', 'mark_read', 'mark_unread',
                    'archive', 'unarchive', 'mute', 'unmute', 'pin', 'unpin',
                    'join_group', 'leave_group', 'publish_status'
                  )
                ORDER BY command.updated_at ASC, command.command_id ASC
                LIMIT $2
                "#,
            )
            .bind(channel_kind)
            .bind(limit.clamp(1, 100))
            .fetch_all(&self.pool)
            .await
            .map_err(ProviderCommandQueuePortError::new)?;
            rows.into_iter()
                .map(row_to_provider_command)
                .collect::<Result<Vec<_>, _>>()
                .map_err(ProviderCommandQueuePortError::new)
        })
    }
}

impl ProviderCommandQueuePort for CommunicationProviderCommandStore {
    fn claim_due<'a>(
        &'a self,
        account_id: &'a str,
        channel_kind: &'a str,
        now: DateTime<Utc>,
        limit: i64,
    ) -> ProviderCommandQueuePortFuture<'a, Vec<CommunicationProviderCommand>> {
        Box::pin(async move {
            self.claim_due(account_id, channel_kind, now, limit)
                .await
                .map_err(ProviderCommandQueuePortError::new)
        })
    }

    fn mark_completed<'a>(
        &'a self,
        command_id: &'a str,
        channel_kind: &'a str,
        now: DateTime<Utc>,
        result_payload: Value,
    ) -> ProviderCommandQueuePortFuture<'a, Option<CommunicationProviderCommand>> {
        Box::pin(async move {
            self.mark_completed(command_id, channel_kind, now, result_payload)
                .await
                .map_err(ProviderCommandQueuePortError::new)
        })
    }

    fn mark_completed_at_epoch<'a>(
        &'a self,
        command_id: &'a str,
        channel_kind: &'a str,
        now: DateTime<Utc>,
        result_payload: Value,
        lease_epoch: i64,
    ) -> ProviderCommandQueuePortFuture<'a, Option<CommunicationProviderCommand>> {
        Box::pin(async move {
            self.mark_completed_at_epoch(
                command_id,
                channel_kind,
                now,
                result_payload,
                Some(lease_epoch),
            )
            .await
            .map_err(ProviderCommandQueuePortError::new)
        })
    }

    fn mark_failed_at_epoch<'a>(
        &'a self,
        command_id: &'a str,
        channel_kind: &'a str,
        now: DateTime<Utc>,
        error: &'a str,
        result_payload: Value,
        lease_epoch: i64,
    ) -> ProviderCommandQueuePortFuture<'a, Option<CommunicationProviderCommand>> {
        Box::pin(async move {
            self.mark_failed_at_epoch(
                command_id,
                channel_kind,
                now,
                error,
                result_payload,
                Some(lease_epoch),
            )
            .await
            .map_err(ProviderCommandQueuePortError::new)
        })
    }

    fn mark_terminal_failed_at_epoch<'a>(
        &'a self,
        command_id: &'a str,
        channel_kind: &'a str,
        now: DateTime<Utc>,
        error: &'a str,
        result_payload: Value,
        lease_epoch: i64,
    ) -> ProviderCommandQueuePortFuture<'a, Option<CommunicationProviderCommand>> {
        Box::pin(async move {
            self.mark_terminal_failed_at_epoch(
                command_id,
                channel_kind,
                now,
                error,
                result_payload,
                Some(lease_epoch),
            )
            .await
            .map_err(ProviderCommandQueuePortError::new)
        })
    }

    fn mark_failed<'a>(
        &'a self,
        command_id: &'a str,
        channel_kind: &'a str,
        now: DateTime<Utc>,
        error: &'a str,
        result_payload: Value,
    ) -> ProviderCommandQueuePortFuture<'a, Option<CommunicationProviderCommand>> {
        Box::pin(async move {
            self.mark_failed(command_id, channel_kind, now, error, result_payload)
                .await
                .map_err(ProviderCommandQueuePortError::new)
        })
    }

    fn mark_terminal_failed<'a>(
        &'a self,
        command_id: &'a str,
        channel_kind: &'a str,
        now: DateTime<Utc>,
        error: &'a str,
        result_payload: Value,
    ) -> ProviderCommandQueuePortFuture<'a, Option<CommunicationProviderCommand>> {
        Box::pin(async move {
            self.mark_terminal_failed(command_id, channel_kind, now, error, result_payload)
                .await
                .map_err(ProviderCommandQueuePortError::new)
        })
    }

    fn recover_stale_executing<'a>(
        &'a self,
        account_id: &'a str,
        channel_kind: &'a str,
        now: DateTime<Utc>,
        execution_lease: chrono::Duration,
    ) -> ProviderCommandQueuePortFuture<'a, Vec<CommunicationProviderCommand>> {
        Box::pin(async move {
            self.recover_stale_executing(account_id, channel_kind, now, execution_lease)
                .await
                .map_err(ProviderCommandQueuePortError::new)
        })
    }

    fn mark_observed_by_provider_message<'a>(
        &'a self,
        account_id: &'a str,
        channel_kind: &'a str,
        provider_message_id: &'a str,
        command_kinds: &'a [&'a str],
        observed_at: DateTime<Utc>,
        provider_state: Value,
    ) -> ProviderCommandQueuePortFuture<'a, Vec<CommunicationProviderCommand>> {
        Box::pin(async move {
            self.mark_observed_by_provider_message(
                account_id,
                channel_kind,
                provider_message_id,
                command_kinds,
                observed_at,
                provider_state,
            )
            .await
            .map_err(ProviderCommandQueuePortError::new)
        })
    }
}

fn provider_command_returning_sql(prefix: &str) -> String {
    provider_command_returning_sql_with_alias(prefix, "")
}

fn provider_command_returning_sql_with_alias(prefix: &str, alias: &str) -> String {
    let qualifier = if alias.trim().is_empty() {
        String::new()
    } else {
        format!("{}.", alias.trim())
    };
    format!(
        r#"
        {prefix}
        RETURNING
            {qualifier}command_id,
            {qualifier}account_id,
            {qualifier}channel_kind,
            {qualifier}command_kind,
            {qualifier}idempotency_key,
            {qualifier}provider_conversation_id,
            {qualifier}provider_message_id,
            {qualifier}target_ref,
            {qualifier}payload,
            {qualifier}capability_state,
            {qualifier}action_class,
            {qualifier}confirmation_decision,
            {qualifier}status,
            {qualifier}retry_count,
            {qualifier}max_retries,
            {qualifier}lease_epoch,
            {qualifier}last_error,
            {qualifier}result_payload,
            {qualifier}audit_metadata,
            {qualifier}provider_state,
            {qualifier}reconciliation_status,
            {qualifier}next_attempt_at,
            {qualifier}last_attempt_at,
            {qualifier}provider_observed_at,
            {qualifier}reconciled_at,
            {qualifier}dead_lettered_at,
            {qualifier}actor_id,
            {qualifier}happened_at,
            {qualifier}completed_at,
            {qualifier}created_at,
            {qualifier}updated_at
        "#
    )
}

fn row_to_provider_command(
    row: PgRow,
) -> Result<CommunicationProviderCommand, CommunicationProviderCommandError> {
    Ok(CommunicationProviderCommand {
        command_id: row.try_get("command_id")?,
        account_id: row.try_get("account_id")?,
        channel_kind: row.try_get("channel_kind")?,
        command_kind: row.try_get("command_kind")?,
        idempotency_key: row.try_get("idempotency_key")?,
        provider_conversation_id: row.try_get("provider_conversation_id")?,
        provider_message_id: row.try_get("provider_message_id")?,
        target_ref: row.try_get("target_ref")?,
        payload: row.try_get("payload")?,
        capability_state: row.try_get("capability_state")?,
        action_class: row.try_get("action_class")?,
        confirmation_decision: row.try_get("confirmation_decision")?,
        status: row.try_get("status")?,
        retry_count: row.try_get("retry_count")?,
        max_retries: row.try_get("max_retries")?,
        lease_epoch: row.try_get("lease_epoch")?,
        last_error: row.try_get("last_error")?,
        result_payload: row.try_get("result_payload")?,
        audit_metadata: row.try_get("audit_metadata")?,
        provider_state: row.try_get("provider_state")?,
        reconciliation_status: row.try_get("reconciliation_status")?,
        next_attempt_at: row.try_get("next_attempt_at")?,
        last_attempt_at: row.try_get("last_attempt_at")?,
        provider_observed_at: row.try_get("provider_observed_at")?,
        reconciled_at: row.try_get("reconciled_at")?,
        dead_lettered_at: row.try_get("dead_lettered_at")?,
        actor_id: row.try_get("actor_id")?,
        happened_at: row.try_get("happened_at")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

async fn ensure_canonical_communication_account_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO communication_accounts (
            account_id, provider_kind, display_name, external_account_id,
            config, metadata, created_at, updated_at
        )
        SELECT
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            config,
            jsonb_build_object('source_table', 'communication_provider_accounts'),
            created_at,
            updated_at
        FROM communication_provider_accounts
        WHERE account_id = $1
        ON CONFLICT (account_id)
        DO UPDATE SET
            provider_kind = EXCLUDED.provider_kind,
            display_name = EXCLUDED.display_name,
            external_account_id = EXCLUDED.external_account_id,
            config = EXCLUDED.config,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(account_id.trim())
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), CommunicationProviderCommandError> {
    if value.trim().is_empty() {
        return Err(CommunicationProviderCommandError::Invalid(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum CommunicationProviderCommandError {
    #[error("invalid communication provider command: {0}")]
    Invalid(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Event(#[from] EventStoreError),
}
