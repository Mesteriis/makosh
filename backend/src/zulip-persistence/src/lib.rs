//! Owner-local Zulip cursor and exact-byte observation outbox persistence.

mod account;
mod delivery_intent;
mod delivery_intent_lifecycle;
mod delivery_intent_result_outbox;
mod operational;
mod owner_rls;
mod schema;

use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_storage_protocol::StorageBindingV1;
use makosh_zulip_api::{ZulipCommandOperationOutcomeV1, ZulipCommandOperationStatusV1};
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub use account::ZulipCredentialBindingV1;
pub use delivery_intent::{ZULIP_DELIVERY_ROUTE_SCHEMA_V1, ZulipDeliveryRouteLocatorV1};
pub use delivery_intent_lifecycle::{
    ClaimedZulipDeliveryIntentJobV1, ZULIP_DELIVERY_INTENT_MAX_ATTEMPTS_V1,
    ZULIP_DELIVERY_INTENT_SCHEMA_V1, ZulipDeliveryIntentAdmissionV1,
    ZulipDeliveryIntentInboxOutcomeV1, ZulipDeliveryIntentJobStateV1, ZulipDeliveryIntentJobV1,
    ZulipDeliveryIntentStoreV1,
};
pub use delivery_intent_result_outbox::ZULIP_DELIVERY_INTENT_RESULT_OUTBOX_SCHEMA_V1;
pub use operational::ZulipOperationalIngestV1;
pub use owner_rls::{
    ZULIP_OWNER_RLS_STORAGE_REVISION_V1, ZULIP_OWNER_RLS_TABLES_V1, zulip_owner_rls_sql_v1,
    zulip_owner_rls_storage_migration_v1,
};
pub use schema::{
    ZULIP_SCHEMA_V2, ZULIP_SCHEMA_V3, ZULIP_STORAGE_BUNDLE_REVISION_V1,
    ZULIP_STORAGE_BUNDLE_REVISION_V2, ZULIP_STORAGE_BUNDLE_REVISION_V3,
    ZULIP_STORAGE_BUNDLE_REVISION_V4, ZULIP_STORAGE_BUNDLE_REVISION_V5,
    ZULIP_STORAGE_BUNDLE_REVISION_V6, ZULIP_STORAGE_BUNDLE_REVISION_V7, zulip_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-zulip-persistence";
pub const ZULIP_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.zulip_provider_cursor (
    account_id TEXT PRIMARY KEY,
    queue_id TEXT NOT NULL,
    last_event_id BIGINT NOT NULL,
    CHECK (length(trim(account_id)) > 0),
    CHECK (length(trim(queue_id)) > 0),
    CHECK (last_event_id >= 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.zulip_communications_outbox (
    message_id BYTEA PRIMARY KEY,
    envelope_sha256 BYTEA NOT NULL,
    exact_envelope_bytes BYTEA NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    published_at_unix_seconds BIGINT,
    CHECK (octet_length(message_id) = 16),
    CHECK (octet_length(envelope_sha256) = 32),
    CHECK (octet_length(exact_envelope_bytes) > 0)
);
CREATE INDEX IF NOT EXISTS zulip_communications_outbox_pending_idx
    ON makosh_data.zulip_communications_outbox (created_at_unix_seconds, message_id)
    WHERE published_at_unix_seconds IS NULL;
CREATE TABLE IF NOT EXISTS makosh_data.zulip_command_operations (
    operation_id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    command_sha256 BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    requested_at_unix_seconds BIGINT NOT NULL,
    completed_at_unix_seconds BIGINT,
    provider_message_id BIGINT,
    blob_ref TEXT,
    CHECK (length(trim(operation_id)) > 0),
    CHECK (length(trim(account_id)) > 0),
    CHECK (octet_length(command_sha256) = 32),
    CHECK (state IN (1, 2, 3)),
    CHECK ((state = 1 AND completed_at_unix_seconds IS NULL AND provider_message_id IS NULL AND blob_ref IS NULL)
        OR (state = 2 AND completed_at_unix_seconds IS NOT NULL AND NOT (provider_message_id IS NOT NULL AND blob_ref IS NOT NULL))
        OR (state = 3 AND completed_at_unix_seconds IS NOT NULL AND provider_message_id IS NULL AND blob_ref IS NULL))
);
CREATE INDEX IF NOT EXISTS zulip_command_operations_unresolved_idx
    ON makosh_data.zulip_command_operations (requested_at_unix_seconds, operation_id)
    WHERE state = 1;
CREATE TABLE IF NOT EXISTS makosh_data.zulip_command_queue (
    operation_id TEXT PRIMARY KEY REFERENCES makosh_data.zulip_command_operations(operation_id),
    exact_command_bytes BYTEA NOT NULL,
    dispatched_at_unix_seconds BIGINT,
    CHECK (octet_length(exact_command_bytes) > 0)
);
CREATE INDEX IF NOT EXISTS zulip_command_queue_pending_idx
    ON makosh_data.zulip_command_queue (operation_id)
    WHERE dispatched_at_unix_seconds IS NULL;
"#;

#[derive(Clone)]
pub struct ZulipDurablePersistence {
    pool: PgPool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipDurablePersistenceError {
    Database,
    InvalidCursor,
    InvalidRow,
    ConflictingDeliveryRouteLocator,
    ConflictingDeliveryIntentInbox,
    InvalidDeliveryIntentTransition,
    OwnerScopeConflict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum ZulipCommandOperationStateV1 {
    OutcomeUnknown = 1,
    Accepted = 2,
    Rejected = 3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipQueueCursorV1 {
    pub account_id: String,
    pub queue_id: String,
    pub last_event_id: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipQueuedCommandV1 {
    pub operation_id: String,
    pub account_id: String,
    pub command_sha256: [u8; 32],
    pub exact_command_bytes: Vec<u8>,
}

impl ZulipDurablePersistence {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, ZulipDurablePersistenceError> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || binding.access().runtime_principal().is_empty()
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
        {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        let port =
            u16::try_from(pgbouncer_port).map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
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
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        Ok(Self { pool })
    }

    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[must_use]
    pub fn delivery_intent_store(&self) -> ZulipDeliveryIntentStoreV1 {
        ZulipDeliveryIntentStoreV1::new(self.pool.clone())
    }

    /// Claims or verifies the one logical human owner before any Zulip state
    /// access. The scope follows the stable fenced Storage-principal prefix
    /// across role epochs and rejects every other registration.
    pub async fn bind_owner_scope(
        &self,
        logical_owner_id: &str,
    ) -> Result<(), ZulipDurablePersistenceError> {
        if !valid_logical_owner_id(logical_owner_id) {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        sqlx::query(
            "INSERT INTO makosh_data.zulip_owner_scope
                (singleton, logical_owner_id, runtime_principal_prefix)
             SELECT TRUE, $1, regexp_replace(current_user::text, '_[0-9]+$', '')
             WHERE current_user::text ~ '^storage_[a-f0-9]{16}_[1-9][0-9]*$'
             ON CONFLICT (singleton) DO NOTHING",
        )
        .bind(logical_owner_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        let exact = sqlx::query(
            "SELECT logical_owner_id
             FROM makosh_data.zulip_owner_scope
             WHERE singleton = TRUE
               AND logical_owner_id = $1
               AND runtime_principal_prefix =
                   regexp_replace(current_user::text, '_[0-9]+$', '')",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        if exact.is_none() {
            return Err(ZulipDurablePersistenceError::OwnerScopeConflict);
        }
        transaction
            .commit()
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)
    }

    pub async fn initialize(&self) -> Result<(), ZulipDurablePersistenceError> {
        sqlx::raw_sql(ZULIP_SCHEMA_V1)
            .execute(&self.pool)
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        sqlx::raw_sql(ZULIP_SCHEMA_V2)
            .execute(&self.pool)
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        sqlx::raw_sql(ZULIP_SCHEMA_V3)
            .execute(&self.pool)
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        sqlx::raw_sql(ZULIP_DELIVERY_ROUTE_SCHEMA_V1)
            .execute(&self.pool)
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        sqlx::raw_sql(ZULIP_DELIVERY_INTENT_SCHEMA_V1)
            .execute(&self.pool)
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        sqlx::raw_sql(ZULIP_DELIVERY_INTENT_RESULT_OUTBOX_SCHEMA_V1)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| ZulipDurablePersistenceError::Database)
    }

    /// Persists a provider command atomically before an external worker may
    /// claim it. A duplicate must carry the exact same bytes and digest.
    pub async fn enqueue_command_operation(
        &self,
        operation_id: &str,
        account_id: &str,
        command_sha256: &[u8; 32],
        exact_command_bytes: &[u8],
        requested_at_unix_seconds: i64,
    ) -> Result<bool, ZulipDurablePersistenceError> {
        validate_command_operation(operation_id, account_id, requested_at_unix_seconds)?;
        if exact_command_bytes.is_empty() {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.zulip_command_operations \
             (operation_id, account_id, command_sha256, state, requested_at_unix_seconds) \
             VALUES ($1, $2, $3, $4, $5) ON CONFLICT (operation_id) DO NOTHING",
        )
        .bind(operation_id)
        .bind(account_id)
        .bind(command_sha256.as_slice())
        .bind(ZulipCommandOperationStateV1::OutcomeUnknown as i16)
        .bind(requested_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        if inserted.rows_affected() == 1 {
            sqlx::query("INSERT INTO makosh_data.zulip_command_queue (operation_id, exact_command_bytes) VALUES ($1, $2)")
                .bind(operation_id)
                .bind(exact_command_bytes)
                .execute(&mut *transaction)
                .await
                .map_err(|_| ZulipDurablePersistenceError::Database)?;
            transaction
                .commit()
                .await
                .map_err(|_| ZulipDurablePersistenceError::Database)?;
            return Ok(true);
        }
        let matching = sqlx::query(
            "SELECT 1 FROM makosh_data.zulip_command_operations operation \
             JOIN makosh_data.zulip_command_queue queue ON queue.operation_id = operation.operation_id \
             WHERE operation.operation_id = $1 AND operation.account_id = $2 \
               AND operation.command_sha256 = $3 AND queue.exact_command_bytes = $4",
        )
        .bind(operation_id)
        .bind(account_id)
        .bind(command_sha256.as_slice())
        .bind(exact_command_bytes)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        matching
            .map(|_| false)
            .ok_or(ZulipDurablePersistenceError::InvalidRow)
    }

    /// Claims one queued command exactly once. The dispatch fence is committed
    /// before external execution, so a crash cannot replay the provider call.
    pub async fn claim_next_command(
        &self,
        dispatched_at_unix_seconds: i64,
    ) -> Result<Option<ZulipQueuedCommandV1>, ZulipDurablePersistenceError> {
        if dispatched_at_unix_seconds <= 0 {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        let row = sqlx::query(
            "WITH next AS (SELECT queue.operation_id FROM makosh_data.zulip_command_queue queue \
             JOIN makosh_data.zulip_command_operations operation ON operation.operation_id = queue.operation_id \
             WHERE queue.dispatched_at_unix_seconds IS NULL AND operation.state = $1 \
             ORDER BY operation.requested_at_unix_seconds, queue.operation_id FOR UPDATE SKIP LOCKED LIMIT 1) \
             UPDATE makosh_data.zulip_command_queue queue SET dispatched_at_unix_seconds = $2 FROM next \
             JOIN makosh_data.zulip_command_operations operation ON operation.operation_id = next.operation_id \
             WHERE queue.operation_id = next.operation_id \
             RETURNING queue.operation_id, operation.account_id, operation.command_sha256, queue.exact_command_bytes",
        )
        .bind(ZulipCommandOperationStateV1::OutcomeUnknown as i16)
        .bind(dispatched_at_unix_seconds)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        row.map(|row| {
            let digest: Vec<u8> = row
                .try_get("command_sha256")
                .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
            let command_sha256: [u8; 32] = digest
                .as_slice()
                .try_into()
                .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
            Ok(ZulipQueuedCommandV1 {
                operation_id: row
                    .try_get("operation_id")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                account_id: row
                    .try_get("account_id")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                command_sha256,
                exact_command_bytes: row
                    .try_get("exact_command_bytes")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
            })
        })
        .transpose()
    }

    pub async fn complete_command_operation(
        &self,
        operation_id: &str,
        command_sha256: &[u8; 32],
        state: ZulipCommandOperationStateV1,
        provider_message_id: Option<i64>,
        blob_ref: Option<&str>,
        completed_at_unix_seconds: i64,
    ) -> Result<(), ZulipDurablePersistenceError> {
        if operation_id.trim().is_empty()
            || completed_at_unix_seconds <= 0
            || state == ZulipCommandOperationStateV1::OutcomeUnknown
            || blob_ref.is_some_and(|value| value.trim().is_empty())
            || (state == ZulipCommandOperationStateV1::Rejected
                && (provider_message_id.is_some() || blob_ref.is_some()))
            || (state == ZulipCommandOperationStateV1::Accepted
                && provider_message_id.is_some()
                && blob_ref.is_some())
        {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        let result = sqlx::query(
            "UPDATE makosh_data.zulip_command_operations SET state = $3, completed_at_unix_seconds = $4, provider_message_id = $5, blob_ref = $6 \
             WHERE operation_id = $1 AND command_sha256 = $2 AND state = $7",
        )
        .bind(operation_id)
        .bind(command_sha256.as_slice())
        .bind(state as i16)
        .bind(completed_at_unix_seconds)
        .bind(provider_message_id)
        .bind(blob_ref)
        .bind(ZulipCommandOperationStateV1::OutcomeUnknown as i16)
        .execute(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        (result.rows_affected() == 1)
            .then_some(())
            .ok_or(ZulipDurablePersistenceError::InvalidRow)
    }

    pub async fn command_operation_status(
        &self,
        operation_id: &str,
    ) -> Result<Option<ZulipCommandOperationStatusV1>, ZulipDurablePersistenceError> {
        if operation_id.trim().is_empty() {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        let row = sqlx::query(
            "SELECT operation_id, account_id, state, requested_at_unix_seconds, completed_at_unix_seconds, provider_message_id, blob_ref \
             FROM makosh_data.zulip_command_operations WHERE operation_id = $1",
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        row.map(|row| {
            let state: i16 = row
                .try_get("state")
                .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
            let provider_message_id: Option<i64> = row
                .try_get("provider_message_id")
                .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
            let blob_ref: Option<String> = row
                .try_get("blob_ref")
                .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
            let completed_at_unix_seconds: Option<i64> =
                row.try_get("completed_at_unix_seconds")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
            let outcome = match state {
                1 if completed_at_unix_seconds.is_none()
                    && provider_message_id.is_none()
                    && blob_ref.is_none() =>
                {
                    ZulipCommandOperationOutcomeV1::OutcomeUnknown
                }
                2 if completed_at_unix_seconds.is_some() => {
                    if provider_message_id.is_some() && blob_ref.is_some() {
                        return Err(ZulipDurablePersistenceError::InvalidRow);
                    }
                    ZulipCommandOperationOutcomeV1::Accepted {
                        provider_message_id,
                        blob_ref,
                    }
                }
                3 if completed_at_unix_seconds.is_some()
                    && provider_message_id.is_none()
                    && blob_ref.is_none() =>
                {
                    ZulipCommandOperationOutcomeV1::Rejected
                }
                _ => return Err(ZulipDurablePersistenceError::InvalidRow),
            };
            Ok(ZulipCommandOperationStatusV1 {
                operation_id: row
                    .try_get("operation_id")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                account_id: row
                    .try_get("account_id")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                outcome,
                requested_at_unix_seconds: row
                    .try_get("requested_at_unix_seconds")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                completed_at_unix_seconds,
            })
        })
        .transpose()
    }

    pub async fn command_operation_was_dispatched(
        &self,
        operation_id: &str,
    ) -> Result<Option<bool>, ZulipDurablePersistenceError> {
        if operation_id.trim().is_empty() {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        sqlx::query(
            "SELECT dispatched_at_unix_seconds
             FROM makosh_data.zulip_command_queue
             WHERE operation_id = $1",
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?
        .map(|row| {
            row.try_get::<Option<i64>, _>("dispatched_at_unix_seconds")
                .map(|value| value.is_some())
                .map_err(|_| ZulipDurablePersistenceError::InvalidRow)
        })
        .transpose()
    }

    /// Advances a queue cursor and stores its observation in the same transaction.
    ///
    /// Returns `false` for an already-observed or stale event in the same queue.
    pub async fn advance_cursor_and_enqueue(
        &self,
        cursor: &ZulipQueueCursorV1,
        record: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<bool, ZulipDurablePersistenceError> {
        validate_cursor(cursor)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        let advanced = sqlx::query(
            "INSERT INTO makosh_data.zulip_provider_cursor (account_id, queue_id, last_event_id) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (account_id) DO UPDATE \
             SET queue_id = EXCLUDED.queue_id, last_event_id = EXCLUDED.last_event_id \
             WHERE zulip_provider_cursor.queue_id <> EXCLUDED.queue_id \
                OR zulip_provider_cursor.last_event_id < EXCLUDED.last_event_id \
             RETURNING account_id",
        )
        .bind(&cursor.account_id)
        .bind(&cursor.queue_id)
        .bind(cursor.last_event_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        if advanced.is_none() {
            transaction
                .commit()
                .await
                .map_err(|_| ZulipDurablePersistenceError::Database)?;
            return Ok(false);
        }
        sqlx::query(
            "INSERT INTO makosh_data.zulip_communications_outbox \
             (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) \
             VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING",
        )
        .bind(record.message_id().as_slice())
        .bind(record.envelope_sha256().as_slice())
        .bind(record.exact_bytes())
        .bind(created_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        Ok(true)
    }

    pub async fn advance_cursor_and_enqueue_many(
        &self,
        cursor: &ZulipQueueCursorV1,
        records: &[OutboxRecordV1],
        created_at_unix_seconds: i64,
    ) -> Result<bool, ZulipDurablePersistenceError> {
        validate_cursor(cursor)?;
        if records.is_empty() {
            return self.advance_cursor(cursor).await;
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        let advanced = sqlx::query(
            "INSERT INTO makosh_data.zulip_provider_cursor (account_id, queue_id, last_event_id) VALUES ($1, $2, $3) \
             ON CONFLICT (account_id) DO UPDATE SET queue_id = EXCLUDED.queue_id, last_event_id = EXCLUDED.last_event_id \
             WHERE zulip_provider_cursor.queue_id <> EXCLUDED.queue_id OR zulip_provider_cursor.last_event_id < EXCLUDED.last_event_id RETURNING account_id",
        ).bind(&cursor.account_id).bind(&cursor.queue_id).bind(cursor.last_event_id)
            .fetch_optional(&mut *transaction).await.map_err(|_| ZulipDurablePersistenceError::Database)?;
        if advanced.is_none() {
            transaction
                .commit()
                .await
                .map_err(|_| ZulipDurablePersistenceError::Database)?;
            return Ok(false);
        }
        for record in records {
            sqlx::query("INSERT INTO makosh_data.zulip_communications_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
                .bind(record.message_id().as_slice()).bind(record.envelope_sha256().as_slice()).bind(record.exact_bytes()).bind(created_at_unix_seconds)
                .execute(&mut *transaction).await.map_err(|_| ZulipDurablePersistenceError::Database)?;
        }
        transaction
            .commit()
            .await
            .map_err(|_| ZulipDurablePersistenceError::Database)?;
        Ok(true)
    }

    /// Records a provider-local event that has no Communications observation.
    ///
    /// Returns `false` for an already-observed or stale event in the same queue.
    pub async fn advance_cursor(
        &self,
        cursor: &ZulipQueueCursorV1,
    ) -> Result<bool, ZulipDurablePersistenceError> {
        validate_cursor(cursor)?;
        sqlx::query(
            "INSERT INTO makosh_data.zulip_provider_cursor (account_id, queue_id, last_event_id) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (account_id) DO UPDATE \
             SET queue_id = EXCLUDED.queue_id, last_event_id = EXCLUDED.last_event_id \
             WHERE zulip_provider_cursor.queue_id <> EXCLUDED.queue_id \
                OR zulip_provider_cursor.last_event_id < EXCLUDED.last_event_id",
        )
        .bind(&cursor.account_id)
        .bind(&cursor.queue_id)
        .bind(cursor.last_event_id)
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected() == 1)
        .map_err(|_| ZulipDurablePersistenceError::Database)
    }

    pub async fn current_cursor(
        &self,
        account_id: &str,
    ) -> Result<Option<ZulipQueueCursorV1>, ZulipDurablePersistenceError> {
        if account_id.trim().is_empty() {
            return Err(ZulipDurablePersistenceError::InvalidCursor);
        }
        let row = sqlx::query(
            "SELECT account_id, queue_id, last_event_id FROM makosh_data.zulip_provider_cursor \
             WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        row.map(|row| {
            let cursor = ZulipQueueCursorV1 {
                account_id: row
                    .try_get("account_id")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                queue_id: row
                    .try_get("queue_id")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                last_event_id: row
                    .try_get("last_event_id")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
            };
            validate_cursor(&cursor)?;
            Ok(cursor)
        })
        .transpose()
    }

    pub async fn pending_communications_outbox(
        &self,
        limit: i64,
    ) -> Result<Vec<OutboxRecordV1>, ZulipDurablePersistenceError> {
        let rows = sqlx::query(
            "SELECT exact_envelope_bytes FROM makosh_data.zulip_communications_outbox \
             WHERE published_at_unix_seconds IS NULL \
             ORDER BY created_at_unix_seconds ASC, message_id ASC LIMIT $1",
        )
        .bind(limit.clamp(1, 256))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let bytes: Vec<u8> = row
                    .try_get("exact_envelope_bytes")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
                OutboxRecordV1::accept(bytes).map_err(|_| ZulipDurablePersistenceError::InvalidRow)
            })
            .collect()
    }

    pub async fn mark_communications_outbox_published(
        &self,
        message_id: &[u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, ZulipDurablePersistenceError> {
        sqlx::query(
            "UPDATE makosh_data.zulip_communications_outbox SET published_at_unix_seconds = $2 \
             WHERE message_id = $1 AND published_at_unix_seconds IS NULL",
        )
        .bind(message_id.as_slice())
        .bind(published_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected() == 1)
        .map_err(|_| ZulipDurablePersistenceError::Database)
    }
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
        && value
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn validate_cursor(cursor: &ZulipQueueCursorV1) -> Result<(), ZulipDurablePersistenceError> {
    if cursor.account_id.trim().is_empty()
        || cursor.queue_id.trim().is_empty()
        || cursor.last_event_id < 0
    {
        return Err(ZulipDurablePersistenceError::InvalidCursor);
    }
    Ok(())
}

fn validate_command_operation(
    operation_id: &str,
    account_id: &str,
    requested_at_unix_seconds: i64,
) -> Result<(), ZulipDurablePersistenceError> {
    if operation_id.trim().is_empty()
        || account_id.trim().is_empty()
        || requested_at_unix_seconds <= 0
    {
        return Err(ZulipDurablePersistenceError::InvalidRow);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ZulipDurablePersistenceError, ZulipQueueCursorV1, validate_cursor};

    #[test]
    fn rejects_empty_or_negative_cursor() {
        assert_eq!(
            validate_cursor(&ZulipQueueCursorV1 {
                account_id: "account".into(),
                queue_id: "queue".into(),
                last_event_id: -1,
            }),
            Err(ZulipDurablePersistenceError::InvalidCursor)
        );
    }
}
