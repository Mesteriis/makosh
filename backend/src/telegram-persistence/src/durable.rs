//! Telegram-owned durable command store. It never joins or mutates business tables.

use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_storage_protocol::StorageBindingV1;
use makosh_telegram_api::{
    TelegramAccount, TelegramAttachmentProjection, TelegramChat, TelegramChatAvatar,
    TelegramChatFolder, TelegramChatOperationalState, TelegramChatPosition,
    TelegramChatStateProjection, TelegramCommandRecord, TelegramCredentialBinding,
    TelegramFileSnapshot, TelegramMessageMutation, TelegramMessageProjection,
    TelegramMessageTombstone, TelegramMessageVersion, TelegramOperation, TelegramOperationState,
    TelegramParticipantFilter, TelegramParticipantPage, TelegramProviderCommand,
    TelegramProviderEvent, TelegramReactionObservation, TelegramReactionSummary,
    TelegramRealtimeFrame, TelegramReconciliationState, TelegramRuntimeReconfiguration,
    TelegramRuntimeReconfigurationState, TelegramRuntimeState, TelegramTopic,
    provider_command_chat_id, provider_command_kind, provider_command_message_id,
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::collections::HashSet;

pub const TELEGRAM_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.telegram_runtime_operations (
    operation_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    command_kind TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    command_payload JSONB NOT NULL,
    state TEXT NOT NULL,
    retry_count BIGINT NOT NULL,
    max_retries BIGINT NOT NULL,
    lease_epoch BIGINT NOT NULL,
    reconciliation TEXT NOT NULL,
    last_error TEXT,
    next_attempt_at BIGINT,
    locked_at BIGINT,
    locked_by TEXT,
    provider_observed_at BIGINT,
    reconciled_at BIGINT,
    UNIQUE (account_id, idempotency_key),
    CHECK (length(trim(operation_id)) > 0),
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(command_kind)) > 0),
    CHECK (retry_count >= 0),
    CHECK (max_retries > 0),
    CHECK (lease_epoch >= 0)
);
CREATE INDEX IF NOT EXISTS telegram_runtime_operations_due_idx
    ON makosh_data.telegram_runtime_operations (account_id, state, next_attempt_at, operation_id);
CREATE INDEX IF NOT EXISTS telegram_runtime_operations_lease_idx
    ON makosh_data.telegram_runtime_operations (account_id, lease_epoch, state);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_provider_event_journal (
    account_id TEXT NOT NULL,
    sequence BIGINT NOT NULL,
    provider_cursor TEXT,
    event_payload JSONB NOT NULL,
    PRIMARY KEY (account_id, sequence),
    CHECK (length(trim(account_id)) > 0),
    CHECK (sequence > 0)
);
CREATE INDEX IF NOT EXISTS telegram_provider_event_journal_cursor_idx
    ON makosh_data.telegram_provider_event_journal (account_id, provider_cursor);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_accounts (
    account_id TEXT PRIMARY KEY,
    account_payload JSONB NOT NULL,
    credentials_payload JSONB NOT NULL,
    CHECK (length(trim(account_id)) > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_runtime_reconfigurations (
    reconfiguration_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES makosh_data.telegram_accounts(account_id),
    expected_runtime_epoch BIGINT NOT NULL,
    target_runtime_epoch BIGINT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('accepted', 'applying', 'completed', 'failed')),
    sanitized_reason_code TEXT,
    CHECK (length(trim(reconfiguration_id)) > 0),
    CHECK (length(trim(account_id)) > 0),
    CHECK (expected_runtime_epoch > 0),
    CHECK (target_runtime_epoch = expected_runtime_epoch + 1),
    CHECK (
        (state = 'failed' AND sanitized_reason_code IS NOT NULL)
        OR (state <> 'failed' AND sanitized_reason_code IS NULL)
    )
);
CREATE UNIQUE INDEX IF NOT EXISTS telegram_runtime_reconfigurations_active_account_idx
    ON makosh_data.telegram_runtime_reconfigurations (account_id)
    WHERE state IN ('accepted', 'applying');
CREATE TABLE IF NOT EXISTS makosh_data.telegram_chat_projections (
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    projection_payload JSONB NOT NULL,
    PRIMARY KEY (account_id, provider_chat_id),
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(provider_chat_id)) > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_chat_avatar_projections (
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    projection_payload JSONB NOT NULL,
    PRIMARY KEY (account_id, provider_chat_id),
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(provider_chat_id)) > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_chat_folder_projections (
    account_id TEXT NOT NULL,
    provider_folder_id BIGINT NOT NULL,
    projection_payload JSONB NOT NULL,
    PRIMARY KEY (account_id, provider_folder_id),
    CHECK (length(trim(account_id)) > 0),
    CHECK (provider_folder_id > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_chat_position_projections (
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    list_kind TEXT NOT NULL,
    provider_folder_id BIGINT NOT NULL DEFAULT 0,
    projection_payload JSONB NOT NULL,
    PRIMARY KEY (account_id, provider_chat_id, list_kind, provider_folder_id),
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(provider_chat_id)) > 0),
    CHECK (length(trim(list_kind)) > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_chat_operational_states (
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    projection_payload JSONB NOT NULL,
    PRIMARY KEY (account_id, provider_chat_id),
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(provider_chat_id)) > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_message_projections (
    message_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    observed_at BIGINT NOT NULL,
    projection_payload JSONB NOT NULL,
    CHECK (length(trim(message_id)) > 0),
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(provider_chat_id)) > 0)
);
CREATE INDEX IF NOT EXISTS telegram_message_projections_chat_idx
    ON makosh_data.telegram_message_projections (account_id, provider_chat_id, observed_at DESC, message_id);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_attachment_projections (
    attachment_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    provider_message_id TEXT NOT NULL,
    provider_file_id TEXT NOT NULL,
    projection_payload JSONB NOT NULL,
    CHECK (length(trim(attachment_id)) > 0),
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(provider_chat_id)) > 0),
    CHECK (length(trim(provider_message_id)) > 0),
    CHECK (length(trim(provider_file_id)) > 0)
);
CREATE INDEX IF NOT EXISTS telegram_attachment_projections_file_idx
    ON makosh_data.telegram_attachment_projections (account_id, provider_file_id);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_file_projections (
    account_id TEXT NOT NULL,
    provider_file_id TEXT NOT NULL,
    projection_payload JSONB NOT NULL,
    PRIMARY KEY (account_id, provider_file_id),
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(provider_file_id)) > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_participant_projections (
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    participant_filter TEXT NOT NULL,
    projection_payload JSONB NOT NULL,
    PRIMARY KEY (account_id, provider_chat_id, participant_filter),
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(provider_chat_id)) > 0),
    CHECK (length(trim(participant_filter)) > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_topic_projections (
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    provider_topic_id TEXT NOT NULL,
    projection_payload JSONB NOT NULL,
    PRIMARY KEY (account_id, provider_chat_id, provider_topic_id),
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(provider_chat_id)) > 0),
    CHECK (length(trim(provider_topic_id)) > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_message_versions (
    version_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    provider_message_id TEXT NOT NULL,
    version_number BIGINT NOT NULL,
    projection_payload JSONB NOT NULL,
    PRIMARY KEY (version_id),
    UNIQUE (message_id, version_number),
    CHECK (version_number > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_message_tombstones (
    tombstone_id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    provider_message_id TEXT NOT NULL,
    projection_payload JSONB NOT NULL
);
CREATE INDEX IF NOT EXISTS telegram_message_tombstones_message_idx
    ON makosh_data.telegram_message_tombstones (message_id, tombstone_id);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_message_reactions (
    message_id TEXT PRIMARY KEY,
    projection_payload JSONB NOT NULL
);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_message_mutations (
    message_id TEXT PRIMARY KEY,
    projection_payload JSONB NOT NULL
);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_chat_states (
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    projection_payload JSONB NOT NULL,
    PRIMARY KEY (account_id, provider_chat_id)
);
CREATE TABLE IF NOT EXISTS makosh_data.telegram_communications_outbox (
    message_id BYTEA PRIMARY KEY,
    envelope_sha256 BYTEA NOT NULL,
    exact_envelope_bytes BYTEA NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    published_at_unix_seconds BIGINT,
    CHECK (octet_length(message_id) = 16),
    CHECK (octet_length(envelope_sha256) = 32),
    CHECK (octet_length(exact_envelope_bytes) > 0)
);
CREATE INDEX IF NOT EXISTS telegram_communications_outbox_pending_idx
    ON makosh_data.telegram_communications_outbox (created_at_unix_seconds, message_id)
    WHERE published_at_unix_seconds IS NULL;
"#;

pub struct TelegramDurablePersistence {
    pub(crate) pool: PgPool,
}

fn valid_logical_owner_id(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value.bytes().enumerate().all(|(index, byte)| match byte {
            b'a'..=b'z' | b'0'..=b'9' => true,
            b'_' | b'-' => index > 0,
            _ => false,
        })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramDurablePersistenceError {
    Database,
    Codec,
    InvalidRow,
    ProjectionLimitExceeded,
    RuntimeEpochConflict,
    ReconfigurationCollision,
    ReconfigurationInProgress,
    ReconfigurationUnknown,
    InvalidReconfigurationTransition,
    ConflictingDeliveryRouteLocator,
    ConflictingDeliveryIntentInbox,
    InvalidDeliveryIntentTransition,
    OwnerScopeConflict,
}

impl TelegramDurablePersistence {
    #[must_use]
    pub fn delivery_intent_store(&self) -> crate::TelegramDeliveryIntentStoreV1 {
        crate::TelegramDeliveryIntentStoreV1::new(self.pool.clone())
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, TelegramDurablePersistenceError> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || binding.access().runtime_principal().is_empty()
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
        {
            return Err(TelegramDurablePersistenceError::InvalidRow);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
        let options = PgConnectOptions::new()
            .statement_cache_capacity(0)
            .host(pgbouncer_host)
            .port(port)
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        Ok(Self::new(pool))
    }

    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Shares the already-fenced owner-local pool with a Telegram companion
    /// persistence package. The returned handle does not change credentials,
    /// budgets, role or storage generation.
    #[must_use]
    pub fn shared_owner_pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub async fn initialize(&self) -> Result<(), TelegramDurablePersistenceError> {
        sqlx::raw_sql(TELEGRAM_SCHEMA_V1)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    /// Claims or verifies the single logical owner of this owner-local
    /// database before any Telegram state is accessed. RLS binds the claim to
    /// the stable prefix of the fenced Storage runtime principal, so a later
    /// role epoch can reopen it while another registration remains invisible.
    pub async fn bind_owner_scope(
        &self,
        logical_owner_id: &str,
    ) -> Result<(), TelegramDurablePersistenceError> {
        if !valid_logical_owner_id(logical_owner_id) {
            return Err(TelegramDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        sqlx::query(
            "INSERT INTO makosh_data.telegram_owner_scope
                (singleton, logical_owner_id, runtime_principal_prefix)
             SELECT TRUE, $1, regexp_replace(current_user::text, '_[0-9]+$', '')
             WHERE current_user::text ~ '^storage_[a-f0-9]{16}_[1-9][0-9]*$'
             ON CONFLICT (singleton) DO NOTHING",
        )
        .bind(logical_owner_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        let exact = sqlx::query(
            "SELECT logical_owner_id
             FROM makosh_data.telegram_owner_scope
             WHERE singleton = TRUE
               AND logical_owner_id = $1
               AND runtime_principal_prefix =
                   regexp_replace(current_user::text, '_[0-9]+$', '')",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        if exact.is_none() {
            return Err(TelegramDurablePersistenceError::OwnerScopeConflict);
        }
        transaction
            .commit()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn enqueue_communications_outbox(
        &self,
        record: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<(), TelegramDurablePersistenceError> {
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_communications_outbox
                (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (message_id) DO NOTHING
            "#,
        )
        .bind(record.message_id().as_slice())
        .bind(record.envelope_sha256().as_slice())
        .bind(record.exact_bytes())
        .bind(created_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn pending_communications_outbox(
        &self,
        limit: i64,
    ) -> Result<Vec<OutboxRecordV1>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT exact_envelope_bytes
            FROM makosh_data.telegram_communications_outbox
            WHERE published_at_unix_seconds IS NULL
            ORDER BY created_at_unix_seconds ASC, message_id ASC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 256))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let bytes: Vec<u8> = row
                    .try_get("exact_envelope_bytes")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                OutboxRecordV1::accept(bytes)
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)
            })
            .collect()
    }

    pub async fn mark_communications_outbox_published(
        &self,
        message_id: &[u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, TelegramDurablePersistenceError> {
        sqlx::query(
            r#"
            UPDATE makosh_data.telegram_communications_outbox
            SET published_at_unix_seconds = $2
            WHERE message_id = $1 AND published_at_unix_seconds IS NULL
            "#,
        )
        .bind(message_id.as_slice())
        .bind(published_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected() == 1)
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn upsert_account(
        &self,
        account: &TelegramAccount,
        credentials: &[TelegramCredentialBinding],
    ) -> Result<(), TelegramDurablePersistenceError> {
        let account_payload =
            serde_json::to_value(account).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        let credentials_payload = serde_json::to_value(credentials)
            .map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_accounts (account_id, account_payload, credentials_payload)
            VALUES ($1, $2, $3)
            ON CONFLICT (account_id) DO UPDATE SET
                account_payload = EXCLUDED.account_payload,
                credentials_payload = EXCLUDED.credentials_payload
            "#,
        )
        .bind(&account.account_id)
        .bind(account_payload)
        .bind(credentials_payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn account(
        &self,
        account_id: &str,
    ) -> Result<
        Option<(TelegramAccount, Vec<TelegramCredentialBinding>)>,
        TelegramDurablePersistenceError,
    > {
        let row = sqlx::query(
            "SELECT account_payload, credentials_payload FROM makosh_data.telegram_accounts WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        row.map(|row| {
            let account_payload: Value = row
                .try_get("account_payload")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            let credentials_payload: Value = row
                .try_get("credentials_payload")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            let account = serde_json::from_value(account_payload)
                .map_err(|_| TelegramDurablePersistenceError::Codec)?;
            let credentials = serde_json::from_value(credentials_payload)
                .map_err(|_| TelegramDurablePersistenceError::Codec)?;
            Ok((account, credentials))
        })
        .transpose()
    }

    pub async fn accounts(&self) -> Result<Vec<TelegramAccount>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            "SELECT account_payload FROM makosh_data.telegram_accounts ORDER BY account_id ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("account_payload")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
            })
            .collect()
    }

    pub async fn accept_runtime_reconfiguration(
        &self,
        reconfiguration: &TelegramRuntimeReconfiguration,
    ) -> Result<TelegramRuntimeReconfiguration, TelegramDurablePersistenceError> {
        if reconfiguration.state != TelegramRuntimeReconfigurationState::Accepted
            || reconfiguration.sanitized_reason_code.is_some()
            || reconfiguration.expected_runtime_epoch == 0
            || reconfiguration.target_runtime_epoch
                != reconfiguration
                    .expected_runtime_epoch
                    .checked_add(1)
                    .ok_or(TelegramDurablePersistenceError::RuntimeEpochConflict)?
        {
            return Err(TelegramDurablePersistenceError::InvalidReconfigurationTransition);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        let account_row = sqlx::query(
            "SELECT account_payload FROM makosh_data.telegram_accounts WHERE account_id = $1 FOR UPDATE",
        )
        .bind(&reconfiguration.account_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?
        .ok_or(TelegramDurablePersistenceError::RuntimeEpochConflict)?;
        let account_payload: Value = account_row
            .try_get("account_payload")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
        let account: TelegramAccount = serde_json::from_value(account_payload)
            .map_err(|_| TelegramDurablePersistenceError::Codec)?;

        if let Some(row) = sqlx::query(
            "SELECT * FROM makosh_data.telegram_runtime_reconfigurations WHERE reconfiguration_id = $1",
        )
        .bind(&reconfiguration.reconfiguration_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?
        {
            let existing = row_to_runtime_reconfiguration(&row)?;
            transaction
                .commit()
                .await
                .map_err(|_| TelegramDurablePersistenceError::Database)?;
            return if existing.account_id == reconfiguration.account_id
                && existing.expected_runtime_epoch == reconfiguration.expected_runtime_epoch
                && existing.target_runtime_epoch == reconfiguration.target_runtime_epoch
            {
                Ok(existing)
            } else {
                Err(TelegramDurablePersistenceError::ReconfigurationCollision)
            };
        }

        if account.runtime_epoch != reconfiguration.expected_runtime_epoch
            || !matches!(
                account.runtime_state,
                TelegramRuntimeState::Running | TelegramRuntimeState::Degraded
            )
        {
            return Err(TelegramDurablePersistenceError::RuntimeEpochConflict);
        }
        let active_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM makosh_data.telegram_runtime_reconfigurations
                WHERE account_id = $1 AND state IN ('accepted', 'applying')
            )",
        )
        .bind(&reconfiguration.account_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        if active_exists {
            return Err(TelegramDurablePersistenceError::ReconfigurationInProgress);
        }

        sqlx::query(
            "INSERT INTO makosh_data.telegram_runtime_reconfigurations (
                reconfiguration_id, account_id, expected_runtime_epoch,
                target_runtime_epoch, state, sanitized_reason_code
            ) VALUES ($1, $2, $3, $4, 'accepted', NULL)",
        )
        .bind(&reconfiguration.reconfiguration_id)
        .bind(&reconfiguration.account_id)
        .bind(
            i64::try_from(reconfiguration.expected_runtime_epoch)
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )
        .bind(
            i64::try_from(reconfiguration.target_runtime_epoch)
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )
        .execute(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        Ok(reconfiguration.clone())
    }

    pub async fn runtime_reconfiguration(
        &self,
        reconfiguration_id: &str,
    ) -> Result<Option<TelegramRuntimeReconfiguration>, TelegramDurablePersistenceError> {
        sqlx::query(
            "SELECT * FROM makosh_data.telegram_runtime_reconfigurations WHERE reconfiguration_id = $1",
        )
        .bind(reconfiguration_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?
        .map(|row| row_to_runtime_reconfiguration(&row))
        .transpose()
    }

    pub async fn pending_runtime_reconfiguration(
        &self,
        account_id: &str,
    ) -> Result<Option<TelegramRuntimeReconfiguration>, TelegramDurablePersistenceError> {
        sqlx::query(
            "SELECT * FROM makosh_data.telegram_runtime_reconfigurations
             WHERE account_id = $1 AND state IN ('accepted', 'applying')
             ORDER BY reconfiguration_id ASC LIMIT 1",
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?
        .map(|row| row_to_runtime_reconfiguration(&row))
        .transpose()
    }

    pub async fn mark_runtime_reconfiguration_applying(
        &self,
        reconfiguration_id: &str,
    ) -> Result<TelegramRuntimeReconfiguration, TelegramDurablePersistenceError> {
        let row = sqlx::query(
            "UPDATE makosh_data.telegram_runtime_reconfigurations
             SET state = 'applying'
             WHERE reconfiguration_id = $1 AND state = 'accepted'
             RETURNING *",
        )
        .bind(reconfiguration_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        if let Some(row) = row {
            return row_to_runtime_reconfiguration(&row);
        }
        let existing = self
            .runtime_reconfiguration(reconfiguration_id)
            .await?
            .ok_or(TelegramDurablePersistenceError::ReconfigurationUnknown)?;
        if existing.state == TelegramRuntimeReconfigurationState::Applying {
            Ok(existing)
        } else {
            Err(TelegramDurablePersistenceError::InvalidReconfigurationTransition)
        }
    }

    pub async fn complete_runtime_reconfiguration(
        &self,
        account: &TelegramAccount,
        completed: &TelegramRuntimeReconfiguration,
    ) -> Result<TelegramRuntimeReconfiguration, TelegramDurablePersistenceError> {
        if completed.state != TelegramRuntimeReconfigurationState::Completed
            || completed.sanitized_reason_code.is_some()
            || account.account_id != completed.account_id
            || account.runtime_epoch != completed.target_runtime_epoch
            || account.runtime_state != TelegramRuntimeState::Running
        {
            return Err(TelegramDurablePersistenceError::InvalidReconfigurationTransition);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        let row = sqlx::query(
            "SELECT * FROM makosh_data.telegram_runtime_reconfigurations
             WHERE reconfiguration_id = $1 FOR UPDATE",
        )
        .bind(&completed.reconfiguration_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?
        .ok_or(TelegramDurablePersistenceError::ReconfigurationUnknown)?;
        let existing = row_to_runtime_reconfiguration(&row)?;
        if existing.account_id != completed.account_id
            || existing.expected_runtime_epoch != completed.expected_runtime_epoch
            || existing.target_runtime_epoch != completed.target_runtime_epoch
        {
            return Err(TelegramDurablePersistenceError::ReconfigurationCollision);
        }
        if existing.state == TelegramRuntimeReconfigurationState::Completed {
            transaction
                .commit()
                .await
                .map_err(|_| TelegramDurablePersistenceError::Database)?;
            return Ok(existing);
        }
        if !matches!(
            existing.state,
            TelegramRuntimeReconfigurationState::Accepted
                | TelegramRuntimeReconfigurationState::Applying
        ) {
            return Err(TelegramDurablePersistenceError::InvalidReconfigurationTransition);
        }
        let account_row = sqlx::query(
            "SELECT account_payload FROM makosh_data.telegram_accounts
             WHERE account_id = $1 FOR UPDATE",
        )
        .bind(&completed.account_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?
        .ok_or(TelegramDurablePersistenceError::RuntimeEpochConflict)?;
        let account_payload: Value = account_row
            .try_get("account_payload")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
        let durable_account: TelegramAccount = serde_json::from_value(account_payload)
            .map_err(|_| TelegramDurablePersistenceError::Codec)?;
        if durable_account.runtime_epoch != completed.expected_runtime_epoch {
            return Err(TelegramDurablePersistenceError::RuntimeEpochConflict);
        }
        let account_payload =
            serde_json::to_value(account).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            "UPDATE makosh_data.telegram_accounts SET account_payload = $2 WHERE account_id = $1",
        )
        .bind(&account.account_id)
        .bind(account_payload)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        sqlx::query(
            "UPDATE makosh_data.telegram_runtime_reconfigurations
             SET state = 'completed', sanitized_reason_code = NULL
             WHERE reconfiguration_id = $1",
        )
        .bind(&completed.reconfiguration_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        Ok(completed.clone())
    }

    pub async fn fail_runtime_reconfiguration(
        &self,
        failed: &TelegramRuntimeReconfiguration,
    ) -> Result<TelegramRuntimeReconfiguration, TelegramDurablePersistenceError> {
        if failed.state != TelegramRuntimeReconfigurationState::Failed
            || failed.sanitized_reason_code.is_none()
        {
            return Err(TelegramDurablePersistenceError::InvalidReconfigurationTransition);
        }
        let row = sqlx::query(
            "UPDATE makosh_data.telegram_runtime_reconfigurations
             SET state = 'failed', sanitized_reason_code = $2
             WHERE reconfiguration_id = $1 AND state IN ('accepted', 'applying')
             RETURNING *",
        )
        .bind(&failed.reconfiguration_id)
        .bind(&failed.sanitized_reason_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?
        .ok_or(TelegramDurablePersistenceError::InvalidReconfigurationTransition)?;
        row_to_runtime_reconfiguration(&row)
    }

    pub async fn upsert_chat(
        &self,
        chat: &TelegramChat,
    ) -> Result<(), TelegramDurablePersistenceError> {
        let payload =
            serde_json::to_value(chat).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_chat_projections
                (account_id, provider_chat_id, projection_payload)
            VALUES ($1, $2, $3)
            ON CONFLICT (account_id, provider_chat_id) DO UPDATE SET
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(&chat.account_id)
        .bind(&chat.provider_chat_id)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn upsert_chat_avatar(
        &self,
        avatar: &TelegramChatAvatar,
    ) -> Result<(), TelegramDurablePersistenceError> {
        let payload =
            serde_json::to_value(avatar).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_chat_avatar_projections
                (account_id, provider_chat_id, projection_payload)
            VALUES ($1, $2, $3)
            ON CONFLICT (account_id, provider_chat_id) DO UPDATE SET
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(&avatar.account_id)
        .bind(&avatar.provider_chat_id)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn chat_avatar(
        &self,
        account_id: &str,
        provider_chat_id: &str,
    ) -> Result<Option<TelegramChatAvatar>, TelegramDurablePersistenceError> {
        let row = sqlx::query(
            "SELECT projection_payload FROM makosh_data.telegram_chat_avatar_projections WHERE account_id = $1 AND provider_chat_id = $2",
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        row.map(|row| {
            let payload: Value = row
                .try_get("projection_payload")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
        })
        .transpose()
    }

    pub async fn upsert_chat_folders(
        &self,
        folders: &[TelegramChatFolder],
    ) -> Result<(), TelegramDurablePersistenceError> {
        for folder in folders {
            let payload =
                serde_json::to_value(folder).map_err(|_| TelegramDurablePersistenceError::Codec)?;
            sqlx::query(
                r#"
                INSERT INTO makosh_data.telegram_chat_folder_projections
                    (account_id, provider_folder_id, projection_payload)
                VALUES ($1, $2, $3)
                ON CONFLICT (account_id, provider_folder_id) DO UPDATE SET
                    projection_payload = EXCLUDED.projection_payload
                "#,
            )
            .bind(&folder.account_id)
            .bind(folder.provider_folder_id)
            .bind(payload)
            .execute(&self.pool)
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        }
        Ok(())
    }

    pub async fn upsert_chat_position(
        &self,
        position: &TelegramChatPosition,
    ) -> Result<(), TelegramDurablePersistenceError> {
        if position.order <= 0 {
            return sqlx::query(
                r#"
                DELETE FROM makosh_data.telegram_chat_position_projections
                WHERE account_id = $1
                  AND provider_chat_id = $2
                  AND list_kind = $3
                  AND provider_folder_id = $4
                "#,
            )
            .bind(&position.account_id)
            .bind(&position.provider_chat_id)
            .bind(&position.list_kind)
            .bind(position.provider_folder_id.unwrap_or_default())
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| TelegramDurablePersistenceError::Database);
        }
        let payload =
            serde_json::to_value(position).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_chat_position_projections
                (account_id, provider_chat_id, list_kind, provider_folder_id, projection_payload)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (account_id, provider_chat_id, list_kind, provider_folder_id) DO UPDATE SET
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(&position.account_id)
        .bind(&position.provider_chat_id)
        .bind(&position.list_kind)
        .bind(position.provider_folder_id.unwrap_or_default())
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn upsert_chat_operational_state(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        state: &TelegramChatOperationalState,
    ) -> Result<(), TelegramDurablePersistenceError> {
        let payload =
            serde_json::to_value(state).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_chat_operational_states
                (account_id, provider_chat_id, projection_payload)
            VALUES ($1, $2, $3)
            ON CONFLICT (account_id, provider_chat_id) DO UPDATE SET
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn list_chat_folders(
        &self,
        account_id: &str,
    ) -> Result<Vec<TelegramChatFolder>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_chat_folder_projections
            WHERE account_id = $1
            ORDER BY provider_folder_id ASC
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("projection_payload")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
            })
            .collect()
    }

    pub async fn list_chat_positions(
        &self,
        account_id: &str,
        provider_chat_id: &str,
    ) -> Result<Vec<TelegramChatPosition>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_chat_position_projections
            WHERE account_id = $1 AND provider_chat_id = $2
            ORDER BY list_kind ASC, provider_folder_id ASC
            "#,
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("projection_payload")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
            })
            .collect()
    }

    pub async fn list_chat_positions_for_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<TelegramChatPosition>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_chat_position_projections
            WHERE account_id = $1
            ORDER BY provider_chat_id ASC, list_kind ASC, provider_folder_id ASC
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("projection_payload")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
            })
            .collect()
    }

    pub async fn chat_operational_state(
        &self,
        account_id: &str,
        provider_chat_id: &str,
    ) -> Result<Option<TelegramChatOperationalState>, TelegramDurablePersistenceError> {
        let row = sqlx::query(
            r#"
            SELECT provider_chat_id, projection_payload
            FROM makosh_data.telegram_chat_operational_states
            WHERE account_id = $1 AND provider_chat_id = $2
            "#,
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        row.map(|row| {
            let payload: Value = row
                .try_get("projection_payload")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
        })
        .transpose()
    }

    pub async fn list_chat_operational_states(
        &self,
        account_id: &str,
    ) -> Result<Vec<(String, TelegramChatOperationalState)>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT provider_chat_id, projection_payload
            FROM makosh_data.telegram_chat_operational_states
            WHERE account_id = $1
            ORDER BY provider_chat_id ASC
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let provider_chat_id: String = row
                    .try_get("provider_chat_id")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                let payload: Value = row
                    .try_get("projection_payload")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                let state = serde_json::from_value(payload)
                    .map_err(|_| TelegramDurablePersistenceError::Codec)?;
                Ok((provider_chat_id, state))
            })
            .collect()
    }

    pub async fn list_chats(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramChat>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_chat_projections
            WHERE account_id = $1
            ORDER BY provider_chat_id ASC
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("projection_payload")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
            })
            .collect()
    }

    pub async fn list_chat_avatars(
        &self,
        account_id: &str,
    ) -> Result<Vec<TelegramChatAvatar>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            "SELECT projection_payload FROM makosh_data.telegram_chat_avatar_projections WHERE account_id = $1 ORDER BY provider_chat_id ASC",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("projection_payload")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
            })
            .collect()
    }

    pub async fn upsert_message(
        &self,
        message: &TelegramMessageProjection,
    ) -> Result<(), TelegramDurablePersistenceError> {
        let locator = crate::TelegramDeliveryRouteLocatorV1::from_message(message)?;
        let payload =
            serde_json::to_value(message).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_message_projections
                (message_id, account_id, provider_chat_id, observed_at, projection_payload)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (message_id) DO UPDATE SET
                account_id = EXCLUDED.account_id,
                provider_chat_id = EXCLUDED.provider_chat_id,
                observed_at = EXCLUDED.observed_at,
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(&message.message_id)
        .bind(&message.account_id)
        .bind(&message.provider_chat_id)
        .bind(message.observed_at_unix_seconds)
        .bind(payload)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        crate::delivery_intent::upsert_delivery_route_locator(
            &mut transaction,
            &locator,
            message.observed_at_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn list_messages(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramMessageProjection>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_message_projections
            WHERE account_id = $1 AND provider_chat_id = $2
            ORDER BY observed_at DESC, message_id DESC
            LIMIT $3
            "#,
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("projection_payload")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
            })
            .collect()
    }

    pub async fn message_ids_for_topic(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_topic_id: &str,
        limit: i64,
    ) -> Result<Vec<String>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            "SELECT projection_payload FROM makosh_data.telegram_message_projections WHERE account_id = $1 AND provider_chat_id = $2 ORDER BY observed_at DESC, message_id ASC",
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        let mut message_ids = Vec::new();
        for row in rows {
            let payload: Value = row
                .try_get("projection_payload")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            let message: TelegramMessageProjection = serde_json::from_value(payload)
                .map_err(|_| TelegramDurablePersistenceError::Codec)?;
            if message.provider_topic_id.as_deref() == Some(provider_topic_id) {
                message_ids.push(message.provider_message_id);
                if message_ids.len() == usize::try_from(limit).unwrap_or(usize::MAX) {
                    break;
                }
            }
        }
        Ok(message_ids)
    }

    pub async fn insert_operation(
        &self,
        operation: &TelegramOperation,
        command: &TelegramProviderCommand,
    ) -> Result<bool, TelegramDurablePersistenceError> {
        let payload =
            serde_json::to_value(command).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        let result = sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_runtime_operations (
                operation_id, account_id, command_kind, idempotency_key,
                command_payload, state, retry_count, max_retries, lease_epoch,
                reconciliation, last_error, next_attempt_at, locked_at, locked_by,
                provider_observed_at, reconciled_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (account_id, idempotency_key) DO NOTHING
            "#,
        )
        .bind(&operation.operation_id)
        .bind(&operation.account_id)
        .bind(operation.command_kind.as_str())
        .bind(&operation.idempotency_key)
        .bind(payload)
        .bind(state_name(operation.state))
        .bind(i64::from(operation.retry_count))
        .bind(i64::from(operation.max_retries))
        .bind(
            i64::try_from(operation.lease_epoch)
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )
        .bind(reconciliation_name(operation.reconciliation))
        .bind(&operation.last_error)
        .bind(optional_i64(operation.next_attempt_at_unix_seconds)?)
        .bind(optional_i64(operation.locked_at_unix_seconds)?)
        .bind(&operation.locked_by)
        .bind(optional_i64(operation.provider_observed_at_unix_seconds)?)
        .bind(optional_i64(operation.reconciled_at_unix_seconds)?)
        .execute(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn save_operation(
        &self,
        operation: &TelegramOperation,
    ) -> Result<(), TelegramDurablePersistenceError> {
        sqlx::query(
            r#"
            UPDATE makosh_data.telegram_runtime_operations
            SET state = $2, retry_count = $3, max_retries = $4, lease_epoch = $5,
                reconciliation = $6, last_error = $7, next_attempt_at = $8,
                locked_at = $9, locked_by = $10, provider_observed_at = $11,
                reconciled_at = $12
            WHERE operation_id = $1
            "#,
        )
        .bind(&operation.operation_id)
        .bind(state_name(operation.state))
        .bind(i64::from(operation.retry_count))
        .bind(i64::from(operation.max_retries))
        .bind(
            i64::try_from(operation.lease_epoch)
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )
        .bind(reconciliation_name(operation.reconciliation))
        .bind(&operation.last_error)
        .bind(optional_i64(operation.next_attempt_at_unix_seconds)?)
        .bind(optional_i64(operation.locked_at_unix_seconds)?)
        .bind(&operation.locked_by)
        .bind(optional_i64(operation.provider_observed_at_unix_seconds)?)
        .bind(optional_i64(operation.reconciled_at_unix_seconds)?)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn operations_for_account(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramOperation>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            "SELECT * FROM makosh_data.telegram_runtime_operations WHERE account_id = $1 ORDER BY operation_id ASC LIMIT $2",
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| row_to_operation(row).map(|(operation, _)| operation))
            .collect()
    }

    pub async fn operation(
        &self,
        operation_id: &str,
    ) -> Result<Option<TelegramOperation>, TelegramDurablePersistenceError> {
        let row = sqlx::query(
            "SELECT * FROM makosh_data.telegram_runtime_operations WHERE operation_id = $1",
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        row.map(|row| row_to_operation(row).map(|(operation, _)| operation))
            .transpose()
    }

    pub async fn command_records_for_account(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        provider_message_id: Option<&str>,
        command_kinds: &[String],
        limit: i64,
    ) -> Result<Vec<TelegramCommandRecord>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            "SELECT * FROM makosh_data.telegram_runtime_operations WHERE account_id = $1 ORDER BY operation_id ASC",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        let mut records = Vec::new();
        for row in rows {
            let (operation, command) = row_to_operation(row)?;
            if provider_chat_id
                .is_some_and(|value| provider_command_chat_id(&command) != Some(value))
                || provider_message_id
                    .is_some_and(|value| provider_command_message_id(&command) != Some(value))
                || (!command_kinds.is_empty()
                    && !command_kinds
                        .iter()
                        .any(|kind| provider_command_kind(&command).as_str() == kind))
            {
                continue;
            }
            records.push(TelegramCommandRecord { operation, command });
            if records.len() == usize::try_from(limit).unwrap_or(usize::MAX) {
                break;
            }
        }
        Ok(records)
    }

    pub async fn claim_due_operations(
        &self,
        account_id: &str,
        now_unix_seconds: u64,
        limit: i64,
        worker_id: &str,
    ) -> Result<Vec<(TelegramOperation, TelegramProviderCommand)>, TelegramDurablePersistenceError>
    {
        let now = i64::try_from(now_unix_seconds)
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        let rows = sqlx::query(
            r#"
            WITH due AS (
                SELECT operation_id
                FROM makosh_data.telegram_runtime_operations
                WHERE account_id = $1
                  AND state IN ('accepted', 'retry_scheduled')
                  AND retry_count < max_retries
                  AND (next_attempt_at IS NULL OR next_attempt_at <= $2)
                ORDER BY COALESCE(next_attempt_at, 0), operation_id
                LIMIT $3
                FOR UPDATE SKIP LOCKED
            )
            UPDATE makosh_data.telegram_runtime_operations operation
            SET state = 'running', retry_count = operation.retry_count + 1,
                locked_at = $2, locked_by = $4, next_attempt_at = NULL,
                last_error = NULL
            FROM due
            WHERE operation.operation_id = due.operation_id
            RETURNING operation.*
            "#,
        )
        .bind(account_id)
        .bind(now)
        .bind(limit)
        .bind(worker_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter().map(row_to_operation).collect()
    }

    pub async fn append_provider_event(
        &self,
        frame: &TelegramRealtimeFrame,
    ) -> Result<bool, TelegramDurablePersistenceError> {
        let payload = serde_json::to_value(&frame.event)
            .map_err(|_| TelegramDurablePersistenceError::Codec)?;
        let sequence = i64::try_from(frame.sequence)
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
        let result = sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_provider_event_journal
                (account_id, sequence, provider_cursor, event_payload)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (account_id, sequence) DO NOTHING
            "#,
        )
        .bind(&frame.account_id)
        .bind(sequence)
        .bind(&frame.provider_cursor)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn replay_provider_events_after(
        &self,
        account_id: &str,
        sequence: u64,
        limit: i64,
    ) -> Result<Vec<TelegramRealtimeFrame>, TelegramDurablePersistenceError> {
        let sequence =
            i64::try_from(sequence).map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
        let rows = sqlx::query(
            r#"
            SELECT account_id, sequence, provider_cursor, event_payload
            FROM makosh_data.telegram_provider_event_journal
            WHERE account_id = $1 AND sequence > $2
            ORDER BY sequence ASC
            LIMIT $3
            "#,
        )
        .bind(account_id)
        .bind(sequence)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("event_payload")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                let event: TelegramProviderEvent = serde_json::from_value(payload)
                    .map_err(|_| TelegramDurablePersistenceError::Codec)?;
                Ok(TelegramRealtimeFrame {
                    account_id: row
                        .try_get("account_id")
                        .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
                    sequence: bounded_u64(
                        row.try_get("sequence")
                            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
                    )?,
                    provider_cursor: row
                        .try_get("provider_cursor")
                        .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
                    event,
                })
            })
            .collect()
    }

    pub async fn provider_event_sequence_bounds(
        &self,
        account_id: &str,
    ) -> Result<Option<(u64, u64)>, TelegramDurablePersistenceError> {
        let row = sqlx::query(
            r#"
            SELECT MIN(sequence) AS earliest_sequence, MAX(sequence) AS latest_sequence
            FROM makosh_data.telegram_provider_event_journal
            WHERE account_id = $1
            "#,
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        let earliest = row
            .try_get::<Option<i64>, _>("earliest_sequence")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
        let latest = row
            .try_get::<Option<i64>, _>("latest_sequence")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
        match (earliest, latest) {
            (None, None) => Ok(None),
            (Some(earliest), Some(latest)) => {
                Ok(Some((bounded_u64(earliest)?, bounded_u64(latest)?)))
            }
            _ => Err(TelegramDurablePersistenceError::InvalidRow),
        }
    }

    pub async fn upsert_file(
        &self,
        file: &TelegramFileSnapshot,
    ) -> Result<(), TelegramDurablePersistenceError> {
        let payload =
            serde_json::to_value(file).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_file_projections
                (account_id, provider_file_id, projection_payload)
            VALUES ($1, $2, $3)
            ON CONFLICT (account_id, provider_file_id) DO UPDATE SET
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(&file.account_id)
        .bind(&file.provider_file_id)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn upsert_attachment(
        &self,
        attachment: &TelegramAttachmentProjection,
    ) -> Result<(), TelegramDurablePersistenceError> {
        let payload =
            serde_json::to_value(attachment).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_attachment_projections
                (attachment_id, account_id, provider_chat_id, provider_message_id,
                 provider_file_id, projection_payload)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (attachment_id) DO UPDATE SET
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(&attachment.attachment_id)
        .bind(&attachment.account_id)
        .bind(&attachment.provider_chat_id)
        .bind(&attachment.provider_message_id)
        .bind(&attachment.provider_file_id)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn attachment(
        &self,
        attachment_id: &str,
    ) -> Result<Option<TelegramAttachmentProjection>, TelegramDurablePersistenceError> {
        let row = sqlx::query(
            "SELECT projection_payload FROM makosh_data.telegram_attachment_projections WHERE attachment_id = $1",
        )
        .bind(attachment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        row.map(|row| {
            let payload: Value = row
                .try_get("projection_payload")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
        })
        .transpose()
    }

    pub async fn attachment_for_message(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
    ) -> Result<Option<TelegramAttachmentProjection>, TelegramDurablePersistenceError> {
        let row = sqlx::query(
            "SELECT projection_payload FROM makosh_data.telegram_attachment_projections WHERE account_id = $1 AND provider_chat_id = $2 AND provider_message_id = $3 ORDER BY attachment_id ASC LIMIT 1",
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(provider_message_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        row.map(|row| {
            let payload: Value = row
                .try_get("projection_payload")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
        })
        .transpose()
    }

    pub async fn file(
        &self,
        account_id: &str,
        provider_file_id: &str,
    ) -> Result<Option<TelegramFileSnapshot>, TelegramDurablePersistenceError> {
        let row = sqlx::query(
            "SELECT projection_payload FROM makosh_data.telegram_file_projections WHERE account_id = $1 AND provider_file_id = $2",
        )
        .bind(account_id)
        .bind(provider_file_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        row.map(|row| {
            let payload: Value = row
                .try_get("projection_payload")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
        })
        .transpose()
    }

    pub async fn apply_file_to_attachments(
        &self,
        account_id: &str,
        file: &TelegramFileSnapshot,
    ) -> Result<Vec<TelegramAttachmentProjection>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            "SELECT attachment_id, projection_payload FROM makosh_data.telegram_attachment_projections WHERE account_id = $1 AND provider_file_id = $2",
        )
        .bind(account_id)
        .bind(&file.provider_file_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        let state = if file.is_downloaded {
            makosh_telegram_api::TelegramAttachmentDownloadState::Downloaded
        } else if file.is_downloading {
            makosh_telegram_api::TelegramAttachmentDownloadState::Downloading
        } else {
            makosh_telegram_api::TelegramAttachmentDownloadState::Pending
        };
        let mut updated = Vec::new();
        for row in rows {
            let attachment_id: String = row
                .try_get("attachment_id")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            let payload: Value = row
                .try_get("projection_payload")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            let mut attachment: TelegramAttachmentProjection = serde_json::from_value(payload)
                .map_err(|_| TelegramDurablePersistenceError::Codec)?;
            attachment.state = state;
            attachment.size_bytes = file.size_bytes.or(file.downloaded_size_bytes);
            attachment.blob_ref = file.blob_reference_id.as_ref().and_then(|reference_id| {
                (reference_id.len() == 16).then(|| {
                    format!(
                        "blob-content:{}",
                        reference_id
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>()
                    )
                })
            });
            let payload = serde_json::to_value(&attachment)
                .map_err(|_| TelegramDurablePersistenceError::Codec)?;
            sqlx::query(
                "UPDATE makosh_data.telegram_attachment_projections SET projection_payload = $2 WHERE attachment_id = $1",
            )
            .bind(attachment_id)
            .bind(payload)
            .execute(&self.pool)
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
            updated.push(attachment);
        }
        Ok(updated)
    }

    pub async fn upsert_participants(
        &self,
        page: &TelegramParticipantPage,
    ) -> Result<(), TelegramDurablePersistenceError> {
        let payload =
            serde_json::to_value(page).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_participant_projections
                (account_id, provider_chat_id, participant_filter, projection_payload)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (account_id, provider_chat_id, participant_filter) DO UPDATE SET
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(&page.account_id)
        .bind(&page.provider_chat_id)
        .bind(format!("{:?}", page.filter))
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn upsert_participant(
        &self,
        participant: &makosh_telegram_api::TelegramParticipant,
    ) -> Result<(), TelegramDurablePersistenceError> {
        let mut page = self
            .participants(
                &participant.account_id,
                &participant.provider_chat_id,
                TelegramParticipantFilter::Recent,
            )
            .await?
            .unwrap_or(TelegramParticipantPage {
                account_id: participant.account_id.clone(),
                provider_chat_id: participant.provider_chat_id.clone(),
                filter: TelegramParticipantFilter::Recent,
                items: Vec::new(),
                next_offset: None,
            });
        if let Some(existing) = page
            .items
            .iter_mut()
            .find(|existing| existing.provider_member_id == participant.provider_member_id)
        {
            *existing = participant.clone();
        } else {
            page.items.push(participant.clone());
        }
        self.upsert_participants(&page).await
    }

    pub async fn participants(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        filter: TelegramParticipantFilter,
    ) -> Result<Option<TelegramParticipantPage>, TelegramDurablePersistenceError> {
        let row = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_participant_projections
            WHERE account_id = $1 AND provider_chat_id = $2 AND participant_filter = $3
            "#,
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(format!("{filter:?}"))
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        row.map(|row| {
            let payload: Value = row
                .try_get("projection_payload")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
        })
        .transpose()
    }

    pub async fn upsert_topic(
        &self,
        topic: &TelegramTopic,
    ) -> Result<(), TelegramDurablePersistenceError> {
        let payload =
            serde_json::to_value(topic).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_topic_projections
                (account_id, provider_chat_id, provider_topic_id, projection_payload)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (account_id, provider_chat_id, provider_topic_id) DO UPDATE SET
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(&topic.account_id)
        .bind(&topic.provider_chat_id)
        .bind(&topic.provider_topic_id)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn list_topics(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramTopic>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_topic_projections
            WHERE account_id = $1 AND provider_chat_id = $2
            ORDER BY provider_topic_id ASC
            LIMIT $3
            "#,
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("projection_payload")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
            })
            .collect()
    }

    pub async fn upsert_message_version(
        &self,
        version: &TelegramMessageVersion,
    ) -> Result<(), TelegramDurablePersistenceError> {
        let payload =
            serde_json::to_value(version).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_message_versions
                (version_id, message_id, account_id, provider_chat_id,
                 provider_message_id, version_number, projection_payload)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (message_id, version_number) DO UPDATE SET
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(&version.version_id)
        .bind(&version.message_id)
        .bind(&version.account_id)
        .bind(&version.provider_chat_id)
        .bind(&version.provider_message_id)
        .bind(i64::from(version.version_number))
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn upsert_tombstone(
        &self,
        tombstone: &TelegramMessageTombstone,
    ) -> Result<(), TelegramDurablePersistenceError> {
        let payload =
            serde_json::to_value(tombstone).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_message_tombstones
                (tombstone_id, message_id, account_id, provider_chat_id,
                 provider_message_id, projection_payload)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (tombstone_id) DO UPDATE SET
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(&tombstone.tombstone_id)
        .bind(&tombstone.message_id)
        .bind(&tombstone.account_id)
        .bind(&tombstone.provider_chat_id)
        .bind(&tombstone.provider_message_id)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn replace_reactions(
        &self,
        message_id: &str,
        reactions: &[TelegramReactionObservation],
    ) -> Result<(), TelegramDurablePersistenceError> {
        let payload =
            serde_json::to_value(reactions).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_message_reactions (message_id, projection_payload)
            VALUES ($1, $2)
            ON CONFLICT (message_id) DO UPDATE SET
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(message_id)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn replace_message_mutations(
        &self,
        message_id: &str,
        mutations: &[TelegramMessageMutation],
    ) -> Result<(), TelegramDurablePersistenceError> {
        let payload =
            serde_json::to_value(mutations).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_message_mutations (message_id, projection_payload)
            VALUES ($1, $2)
            ON CONFLICT (message_id) DO UPDATE SET
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(message_id)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn message_mutations(
        &self,
        message_id: &str,
    ) -> Result<Vec<TelegramMessageMutation>, TelegramDurablePersistenceError> {
        let row = sqlx::query(
            "SELECT projection_payload FROM makosh_data.telegram_message_mutations WHERE message_id = $1",
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        let Some(row) = row else {
            return Ok(Vec::new());
        };
        let payload: Value = row
            .try_get("projection_payload")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
        serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
    }

    pub async fn message_references(
        &self,
        message_id: &str,
    ) -> Result<
        Option<makosh_telegram_api::TelegramMessageReferences>,
        TelegramDurablePersistenceError,
    > {
        let row = sqlx::query(
            "SELECT projection_payload FROM makosh_data.telegram_message_projections WHERE message_id = $1",
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        row.map(|row| {
            let payload: Value = row
                .try_get("projection_payload")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            let message: makosh_telegram_api::TelegramMessageProjection =
                serde_json::from_value(payload)
                    .map_err(|_| TelegramDurablePersistenceError::Codec)?;
            Ok(message.references)
        })
        .transpose()
    }

    pub async fn reply_chain(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
        limit: i64,
    ) -> Result<Vec<makosh_telegram_api::TelegramMessageProjection>, TelegramDurablePersistenceError>
    {
        let mut messages = self
            .list_messages(account_id, provider_chat_id, i64::MAX)
            .await?;
        messages.sort_by(|left, right| left.provider_message_id.cmp(&right.provider_message_id));
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        let mut next = Some((provider_chat_id.to_owned(), provider_message_id.to_owned()));
        while let Some((chat_id, message_id)) = next {
            if !visited.insert((chat_id.clone(), message_id.clone())) || chain.len() >= 128 {
                break;
            }
            let Some(index) = messages.iter().position(|message| {
                message.provider_chat_id == chat_id && message.provider_message_id == message_id
            }) else {
                break;
            };
            let message = messages[index].clone();
            next = message.references.reply_to.as_ref().map(|reference| {
                (
                    reference.provider_chat_id.clone(),
                    reference.provider_message_id.clone(),
                )
            });
            chain.push(message);
            if chain.len() >= usize::try_from(limit).unwrap_or(usize::MAX) {
                break;
            }
        }
        Ok(chain)
    }

    pub async fn forward_chain(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
        limit: i64,
    ) -> Result<Vec<makosh_telegram_api::TelegramMessageProjection>, TelegramDurablePersistenceError>
    {
        let rows = sqlx::query(
            "SELECT projection_payload FROM makosh_data.telegram_message_projections WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        let messages = rows
            .into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("projection_payload")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                serde_json::from_value::<makosh_telegram_api::TelegramMessageProjection>(payload)
                    .map_err(|_| TelegramDurablePersistenceError::Codec)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        let mut next = Some((provider_chat_id.to_owned(), provider_message_id.to_owned()));
        while let Some((chat_id, message_id)) = next {
            if !visited.insert((chat_id.clone(), message_id.clone())) || chain.len() >= 128 {
                break;
            }
            let Some(message) = messages.iter().find(|message| {
                message.provider_chat_id == chat_id && message.provider_message_id == message_id
            }) else {
                break;
            };
            next = message
                .references
                .forward_origin
                .as_ref()
                .and_then(|origin| {
                    Some((
                        origin.provider_chat_id.as_ref()?.clone(),
                        origin.provider_message_id.as_ref()?.clone(),
                    ))
                });
            chain.push(message.clone());
            if chain.len() >= usize::try_from(limit).unwrap_or(usize::MAX) {
                break;
            }
        }
        Ok(chain)
    }

    pub async fn upsert_chat_state(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        state: &TelegramChatStateProjection,
    ) -> Result<(), TelegramDurablePersistenceError> {
        let payload =
            serde_json::to_value(state).map_err(|_| TelegramDurablePersistenceError::Codec)?;
        sqlx::query(
            r#"
            INSERT INTO makosh_data.telegram_chat_states
                (account_id, provider_chat_id, projection_payload)
            VALUES ($1, $2, $3)
            ON CONFLICT (account_id, provider_chat_id) DO UPDATE SET
                projection_payload = EXCLUDED.projection_payload
            "#,
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| TelegramDurablePersistenceError::Database)
    }

    pub async fn chat_state(
        &self,
        account_id: &str,
        provider_chat_id: &str,
    ) -> Result<Option<TelegramChatStateProjection>, TelegramDurablePersistenceError> {
        let row = sqlx::query(
            "SELECT projection_payload FROM makosh_data.telegram_chat_states WHERE account_id = $1 AND provider_chat_id = $2",
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        row.map(|row| {
            let payload: Value = row
                .try_get("projection_payload")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
        })
        .transpose()
    }

    pub async fn list_message_versions(
        &self,
        message_id: &str,
    ) -> Result<Vec<TelegramMessageVersion>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_message_versions
            WHERE message_id = $1
            ORDER BY version_number ASC
            "#,
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("projection_payload")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
            })
            .collect()
    }

    pub async fn list_tombstones(
        &self,
        message_id: &str,
    ) -> Result<Vec<TelegramMessageTombstone>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_message_tombstones
            WHERE message_id = $1
            ORDER BY tombstone_id ASC
            "#,
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let payload: Value = row
                    .try_get("projection_payload")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
            })
            .collect()
    }

    pub async fn reactions(
        &self,
        message_id: &str,
    ) -> Result<Vec<TelegramReactionObservation>, TelegramDurablePersistenceError> {
        let row = sqlx::query(
            "SELECT projection_payload FROM makosh_data.telegram_message_reactions WHERE message_id = $1",
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        let Some(row) = row else {
            return Ok(Vec::new());
        };
        let payload: Value = row
            .try_get("projection_payload")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
        serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
    }

    pub async fn list_messages_for_account(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramMessageProjection>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_message_projections
            WHERE account_id = $1
            ORDER BY provider_chat_id, observed_at, message_id
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(validated_projection_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter().map(projection_from_row).collect()
    }

    pub async fn list_files_for_account(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramFileSnapshot>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_file_projections
            WHERE account_id = $1
            ORDER BY provider_file_id
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(validated_projection_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter().map(projection_from_row).collect()
    }

    pub async fn list_attachments_for_account(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramAttachmentProjection>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_attachment_projections
            WHERE account_id = $1
            ORDER BY attachment_id
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(validated_projection_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter().map(projection_from_row).collect()
    }

    pub async fn list_participant_pages_for_account(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramParticipantPage>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_participant_projections
            WHERE account_id = $1
            ORDER BY provider_chat_id, participant_filter
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(validated_projection_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter().map(projection_from_row).collect()
    }

    pub async fn list_topics_for_account(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramTopic>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_topic_projections
            WHERE account_id = $1
            ORDER BY provider_chat_id, provider_topic_id
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(validated_projection_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter().map(projection_from_row).collect()
    }

    pub async fn list_message_versions_for_account(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramMessageVersion>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_message_versions
            WHERE account_id = $1
            ORDER BY message_id, version_number
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(validated_projection_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter().map(projection_from_row).collect()
    }

    pub async fn list_tombstones_for_account(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<TelegramMessageTombstone>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT projection_payload
            FROM makosh_data.telegram_message_tombstones
            WHERE account_id = $1
            ORDER BY message_id, tombstone_id
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(validated_projection_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter().map(projection_from_row).collect()
    }

    pub async fn list_reactions_for_account(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<(String, Vec<TelegramReactionObservation>)>, TelegramDurablePersistenceError>
    {
        let rows = sqlx::query(
            r#"
            SELECT reactions.message_id, reactions.projection_payload
            FROM makosh_data.telegram_message_reactions AS reactions
            JOIN makosh_data.telegram_message_projections AS messages
              ON messages.message_id = reactions.message_id
            WHERE messages.account_id = $1
            ORDER BY reactions.message_id
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(validated_projection_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter().map(keyed_projection_from_row).collect()
    }

    pub async fn list_message_mutations_for_account(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<(String, Vec<TelegramMessageMutation>)>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT mutations.message_id, mutations.projection_payload
            FROM makosh_data.telegram_message_mutations AS mutations
            JOIN makosh_data.telegram_message_projections AS messages
              ON messages.message_id = mutations.message_id
            WHERE messages.account_id = $1
            ORDER BY mutations.message_id
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(validated_projection_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter().map(keyed_projection_from_row).collect()
    }

    pub async fn list_chat_states_for_account(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<Vec<(String, TelegramChatStateProjection)>, TelegramDurablePersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT provider_chat_id, projection_payload
            FROM makosh_data.telegram_chat_states
            WHERE account_id = $1
            ORDER BY provider_chat_id
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(validated_projection_query_limit(limit)?)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let provider_chat_id = row
                    .try_get("provider_chat_id")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
                projection_from_row(row).map(|state| (provider_chat_id, state))
            })
            .collect()
    }

    pub async fn reaction_summary(
        &self,
        message_id: &str,
    ) -> Result<Vec<TelegramReactionSummary>, TelegramDurablePersistenceError> {
        let reactions = self.reactions(message_id).await?;
        let mut summary = std::collections::HashMap::<String, TelegramReactionSummary>::new();
        for reaction in reactions {
            let entry = summary
                .entry(reaction.emoji.clone())
                .or_insert(TelegramReactionSummary {
                    emoji: reaction.emoji.clone(),
                    count: 0,
                    is_active: false,
                });
            if reaction.is_active {
                entry.count = entry.count.saturating_add(1);
            }
            entry.is_active |= reaction.is_active && reaction.is_outgoing;
        }
        let mut values = summary.into_values().collect::<Vec<_>>();
        values.sort_by(|left, right| left.emoji.cmp(&right.emoji));
        Ok(values)
    }
}

fn projection_from_row<T: DeserializeOwned>(
    row: sqlx::postgres::PgRow,
) -> Result<T, TelegramDurablePersistenceError> {
    let payload: Value = row
        .try_get("projection_payload")
        .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
    serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)
}

fn keyed_projection_from_row<T: DeserializeOwned>(
    row: sqlx::postgres::PgRow,
) -> Result<(String, T), TelegramDurablePersistenceError> {
    let message_id = row
        .try_get("message_id")
        .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
    projection_from_row(row).map(|projection| (message_id, projection))
}

fn validated_projection_query_limit(limit: i64) -> Result<i64, TelegramDurablePersistenceError> {
    (1..=100_000)
        .contains(&limit)
        .then(|| limit + 1)
        .ok_or(TelegramDurablePersistenceError::InvalidRow)
}

fn row_to_runtime_reconfiguration(
    row: &sqlx::postgres::PgRow,
) -> Result<TelegramRuntimeReconfiguration, TelegramDurablePersistenceError> {
    let state = match row
        .try_get::<String, _>("state")
        .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?
        .as_str()
    {
        "accepted" => TelegramRuntimeReconfigurationState::Accepted,
        "applying" => TelegramRuntimeReconfigurationState::Applying,
        "completed" => TelegramRuntimeReconfigurationState::Completed,
        "failed" => TelegramRuntimeReconfigurationState::Failed,
        _ => return Err(TelegramDurablePersistenceError::InvalidRow),
    };
    Ok(TelegramRuntimeReconfiguration {
        reconfiguration_id: row
            .try_get("reconfiguration_id")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        account_id: row
            .try_get("account_id")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        expected_runtime_epoch: bounded_u64(
            row.try_get("expected_runtime_epoch")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )?,
        target_runtime_epoch: bounded_u64(
            row.try_get("target_runtime_epoch")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )?,
        state,
        sanitized_reason_code: row
            .try_get("sanitized_reason_code")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
    })
}

fn row_to_operation(
    row: sqlx::postgres::PgRow,
) -> Result<(TelegramOperation, TelegramProviderCommand), TelegramDurablePersistenceError> {
    let payload: Value = row
        .try_get("command_payload")
        .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
    let command: TelegramProviderCommand =
        serde_json::from_value(payload).map_err(|_| TelegramDurablePersistenceError::Codec)?;
    if row
        .try_get::<String, _>("command_kind")
        .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?
        != provider_command_kind(&command).as_str()
    {
        return Err(TelegramDurablePersistenceError::InvalidRow);
    }
    let operation = TelegramOperation {
        operation_id: row
            .try_get("operation_id")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        account_id: row
            .try_get("account_id")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        command_kind: provider_command_kind(&command),
        idempotency_key: row
            .try_get("idempotency_key")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        state: parse_state(
            &row.try_get::<String, _>("state")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )?,
        retry_count: bounded_u32(
            row.try_get::<i64, _>("retry_count")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )?,
        max_retries: bounded_u32(
            row.try_get::<i64, _>("max_retries")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )?,
        lease_epoch: bounded_u64(
            row.try_get::<i64, _>("lease_epoch")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )?,
        reconciliation: parse_reconciliation(
            &row.try_get::<String, _>("reconciliation")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )?,
        last_error: row
            .try_get("last_error")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        next_attempt_at_unix_seconds: optional_u64(
            row.try_get("next_attempt_at")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )?,
        locked_at_unix_seconds: optional_u64(
            row.try_get("locked_at")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )?,
        locked_by: row
            .try_get("locked_by")
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        provider_observed_at_unix_seconds: optional_u64(
            row.try_get("provider_observed_at")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )?,
        reconciled_at_unix_seconds: optional_u64(
            row.try_get("reconciled_at")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )?,
    };
    Ok((operation, command))
}

fn state_name(state: TelegramOperationState) -> &'static str {
    match state {
        TelegramOperationState::Accepted => "accepted",
        TelegramOperationState::Running => "running",
        TelegramOperationState::AwaitingProvider => "awaiting_provider",
        TelegramOperationState::Completed => "completed",
        TelegramOperationState::Failed => "failed",
        TelegramOperationState::RetryScheduled => "retry_scheduled",
        TelegramOperationState::DeadLetter => "dead_letter",
    }
}

fn reconciliation_name(state: TelegramReconciliationState) -> &'static str {
    match state {
        TelegramReconciliationState::NotObserved => "not_observed",
        TelegramReconciliationState::AwaitingProvider => "awaiting_provider",
        TelegramReconciliationState::Observed => "observed",
        TelegramReconciliationState::Mismatch => "mismatch",
    }
}

fn parse_state(value: &str) -> Result<TelegramOperationState, TelegramDurablePersistenceError> {
    match value {
        "accepted" => Ok(TelegramOperationState::Accepted),
        "running" => Ok(TelegramOperationState::Running),
        "awaiting_provider" => Ok(TelegramOperationState::AwaitingProvider),
        "completed" => Ok(TelegramOperationState::Completed),
        "failed" => Ok(TelegramOperationState::Failed),
        "retry_scheduled" => Ok(TelegramOperationState::RetryScheduled),
        "dead_letter" => Ok(TelegramOperationState::DeadLetter),
        _ => Err(TelegramDurablePersistenceError::InvalidRow),
    }
}

fn parse_reconciliation(
    value: &str,
) -> Result<TelegramReconciliationState, TelegramDurablePersistenceError> {
    match value {
        "not_observed" => Ok(TelegramReconciliationState::NotObserved),
        "awaiting_provider" => Ok(TelegramReconciliationState::AwaitingProvider),
        "observed" => Ok(TelegramReconciliationState::Observed),
        "mismatch" => Ok(TelegramReconciliationState::Mismatch),
        _ => Err(TelegramDurablePersistenceError::InvalidRow),
    }
}

fn optional_i64(value: Option<u64>) -> Result<Option<i64>, TelegramDurablePersistenceError> {
    value
        .map(|value| i64::try_from(value).map_err(|_| TelegramDurablePersistenceError::InvalidRow))
        .transpose()
}

fn optional_u64(value: Option<i64>) -> Result<Option<u64>, TelegramDurablePersistenceError> {
    value.map(bounded_u64).transpose()
}

fn bounded_u64(value: i64) -> Result<u64, TelegramDurablePersistenceError> {
    u64::try_from(value).map_err(|_| TelegramDurablePersistenceError::InvalidRow)
}

fn bounded_u32(value: i64) -> Result<u32, TelegramDurablePersistenceError> {
    u32::try_from(value).map_err(|_| TelegramDurablePersistenceError::InvalidRow)
}
