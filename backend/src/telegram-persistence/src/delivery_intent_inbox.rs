//! Telegram-owned delivery-intent inbox and resolved jobs.

use makosh_events_protocol::delivery::OutboxRecordV1;
use sqlx::{PgPool, Row};

use crate::TelegramDurablePersistenceError;

pub const TELEGRAM_SCHEMA_V3: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.telegram_delivery_intent_inbox (
    message_id BYTEA PRIMARY KEY,
    envelope_sha256 BYTEA NOT NULL,
    intent_id BYTEA NOT NULL UNIQUE,
    logical_owner_id TEXT NOT NULL,
    state SMALLINT NOT NULL CHECK (state BETWEEN 0 AND 2),
    consumed_at_unix_seconds BIGINT NOT NULL,
    CHECK (octet_length(message_id) = 16),
    CHECK (octet_length(envelope_sha256) = 32),
    CHECK (octet_length(intent_id) = 16),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 256)
);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_delivery_intent_jobs (
    intent_id BYTEA PRIMARY KEY,
    command_message_id BYTEA NOT NULL UNIQUE,
    account_id TEXT NOT NULL,
    provider_chat_id TEXT NOT NULL,
    reply_to_provider_message_id TEXT,
    body_reference_id BYTEA NOT NULL,
    body_declared_bytes BIGINT NOT NULL,
    body_sha256 BYTEA NOT NULL,
    custody_transfer_source_proof BYTEA NOT NULL,
    provider_operation_id TEXT NOT NULL UNIQUE,
    state SMALLINT NOT NULL DEFAULT 1 CHECK (state BETWEEN 1 AND 6),
    attempt_count INTEGER NOT NULL DEFAULT 0,
    next_attempt_at_unix_seconds BIGINT NOT NULL,
    claimed_by TEXT,
    lease_expires_at_unix_seconds BIGINT,
    target_body_reference_id BYTEA,
    target_body_receipt_sha256 BYTEA,
    completed_at_unix_seconds BIGINT,
    CHECK (octet_length(intent_id) = 16),
    CHECK (octet_length(command_message_id) = 16),
    CHECK (length(account_id) BETWEEN 1 AND 256),
    CHECK (length(provider_chat_id) BETWEEN 1 AND 512),
    CHECK (
        reply_to_provider_message_id IS NULL OR
        length(reply_to_provider_message_id) BETWEEN 1 AND 512
    ),
    CHECK (octet_length(body_reference_id) = 16),
    CHECK (body_declared_bytes BETWEEN 1 AND 65536),
    CHECK (octet_length(body_sha256) = 32),
    CHECK (octet_length(custody_transfer_source_proof) BETWEEN 1 AND 2048),
    CHECK (length(provider_operation_id) BETWEEN 1 AND 128),
    CHECK (attempt_count >= 0),
    CHECK (
        (target_body_reference_id IS NULL AND target_body_receipt_sha256 IS NULL)
        OR (
            octet_length(target_body_reference_id) = 16 AND
            octet_length(target_body_receipt_sha256) = 32
        )
    )
);

CREATE INDEX IF NOT EXISTS telegram_delivery_intent_jobs_claim_idx
    ON makosh_data.telegram_delivery_intent_jobs
        (state, next_attempt_at_unix_seconds, intent_id);

CREATE TABLE IF NOT EXISTS makosh_data.telegram_delivery_intent_result_outbox (
    message_id BYTEA PRIMARY KEY,
    envelope_sha256 BYTEA NOT NULL,
    exact_envelope_bytes BYTEA NOT NULL,
    intent_id BYTEA NOT NULL UNIQUE,
    created_at_unix_seconds BIGINT NOT NULL,
    published_at_unix_seconds BIGINT,
    CHECK (octet_length(message_id) = 16),
    CHECK (octet_length(envelope_sha256) = 32),
    CHECK (octet_length(exact_envelope_bytes) > 0),
    CHECK (octet_length(intent_id) = 16)
);
"#;

pub const TELEGRAM_DELIVERY_INTENT_MAX_ATTEMPTS_V1: i32 = 12;

#[derive(Clone)]
pub struct TelegramDeliveryIntentStoreV1 {
    pub(crate) pool: PgPool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramDeliveryIntentAdmissionV1 {
    pub command_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub intent_id: [u8; 16],
    pub logical_owner_id: String,
    pub account_source_cursor: [u8; 32],
    pub conversation_source_cursor: [u8; 32],
    pub reply_to_source_cursor: Option<[u8; 32]>,
    pub body_reference_id: [u8; 16],
    pub body_declared_bytes: u64,
    pub body_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramDeliveryIntentJobV1 {
    pub intent_id: [u8; 16],
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub reply_to_provider_message_id: Option<String>,
    pub body_reference_id: [u8; 16],
    pub body_declared_bytes: u64,
    pub body_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
    pub provider_operation_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum TelegramDeliveryIntentJobStateV1 {
    PendingCustody = 1,
    BodyReady = 2,
    DeliveryQueued = 3,
    Succeeded = 4,
    Rejected = 5,
    OutcomeUnknown = 6,
}

impl TelegramDeliveryIntentJobStateV1 {
    const fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::PendingCustody),
            2 => Some(Self::BodyReady),
            3 => Some(Self::DeliveryQueued),
            4 => Some(Self::Succeeded),
            5 => Some(Self::Rejected),
            6 => Some(Self::OutcomeUnknown),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimedTelegramDeliveryIntentJobV1 {
    pub job: TelegramDeliveryIntentJobV1,
    pub state: TelegramDeliveryIntentJobStateV1,
    pub target_body_reference_id: Option<[u8; 16]>,
    pub target_body_receipt_sha256: Option<[u8; 32]>,
    pub worker_id: String,
    pub attempt_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramDeliveryIntentInboxOutcomeV1 {
    Pending,
    RouteNotFound,
    DuplicatePending,
    DuplicateRouteNotFound,
}

impl TelegramDeliveryIntentStoreV1 {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn accept_command(
        &self,
        admission: &TelegramDeliveryIntentAdmissionV1,
        route_not_found_result: &OutboxRecordV1,
        consumed_at_unix_seconds: i64,
    ) -> Result<TelegramDeliveryIntentInboxOutcomeV1, TelegramDurablePersistenceError> {
        if !valid_admission(admission) || consumed_at_unix_seconds <= 0 {
            return Err(TelegramDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.telegram_delivery_intent_inbox
                (message_id, envelope_sha256, intent_id, logical_owner_id, state,
                 consumed_at_unix_seconds)
             VALUES ($1, $2, $3, $4, 0, $5)
             ON CONFLICT (message_id) DO NOTHING",
        )
        .bind(admission.command_message_id.as_slice())
        .bind(admission.envelope_sha256.as_slice())
        .bind(admission.intent_id.as_slice())
        .bind(&admission.logical_owner_id)
        .bind(consumed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        if inserted.rows_affected() == 0 {
            let row = sqlx::query(
                "SELECT envelope_sha256, intent_id, logical_owner_id, state
                 FROM makosh_data.telegram_delivery_intent_inbox
                 WHERE message_id = $1",
            )
            .bind(admission.command_message_id.as_slice())
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
            let hash: Vec<u8> = row
                .try_get("envelope_sha256")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            let intent_id: Vec<u8> = row
                .try_get("intent_id")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            let logical_owner_id: String = row
                .try_get("logical_owner_id")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            let state: i16 = row
                .try_get("state")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
            if hash.as_slice() != admission.envelope_sha256
                || intent_id.as_slice() != admission.intent_id
                || logical_owner_id != admission.logical_owner_id
            {
                return Err(TelegramDurablePersistenceError::ConflictingDeliveryIntentInbox);
            }
            transaction
                .commit()
                .await
                .map_err(|_| TelegramDurablePersistenceError::Database)?;
            return match state {
                1 => Ok(TelegramDeliveryIntentInboxOutcomeV1::DuplicatePending),
                2 => Ok(TelegramDeliveryIntentInboxOutcomeV1::DuplicateRouteNotFound),
                _ => Err(TelegramDurablePersistenceError::InvalidRow),
            };
        }

        let route = resolve_route(&mut transaction, admission).await?;
        let outcome = if let Some(route) = route {
            sqlx::query(
                "INSERT INTO makosh_data.telegram_delivery_intent_jobs
                    (intent_id, command_message_id, account_id, provider_chat_id,
                     reply_to_provider_message_id, body_reference_id, body_declared_bytes,
                     body_sha256, custody_transfer_source_proof, provider_operation_id,
                     next_attempt_at_unix_seconds)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            )
            .bind(admission.intent_id.as_slice())
            .bind(admission.command_message_id.as_slice())
            .bind(route.account_id)
            .bind(route.provider_chat_id)
            .bind(route.reply_to_provider_message_id)
            .bind(admission.body_reference_id.as_slice())
            .bind(
                i64::try_from(admission.body_declared_bytes)
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
            )
            .bind(admission.body_sha256.as_slice())
            .bind(&admission.custody_transfer_source_proof)
            .bind(provider_operation_id(admission.intent_id))
            .bind(consumed_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
            sqlx::query(
                "UPDATE makosh_data.telegram_delivery_intent_inbox SET state = 1
                 WHERE message_id = $1 AND state = 0",
            )
            .bind(admission.command_message_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
            TelegramDeliveryIntentInboxOutcomeV1::Pending
        } else {
            crate::delivery_intent_result_outbox::insert_result_outbox(
                &mut transaction,
                admission.intent_id,
                route_not_found_result,
                consumed_at_unix_seconds,
            )
            .await?;
            sqlx::query(
                "UPDATE makosh_data.telegram_delivery_intent_inbox SET state = 2
                 WHERE message_id = $1 AND state = 0",
            )
            .bind(admission.command_message_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
            TelegramDeliveryIntentInboxOutcomeV1::RouteNotFound
        };
        transaction
            .commit()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        Ok(outcome)
    }

    pub async fn claim_next_job(
        &self,
        worker_id: &str,
        now_unix_seconds: i64,
        lease_expires_at_unix_seconds: i64,
    ) -> Result<Option<ClaimedTelegramDeliveryIntentJobV1>, TelegramDurablePersistenceError> {
        if worker_id.trim().is_empty()
            || worker_id.len() > 128
            || now_unix_seconds <= 0
            || lease_expires_at_unix_seconds <= now_unix_seconds
        {
            return Err(TelegramDurablePersistenceError::InvalidRow);
        }
        let row = sqlx::query(
            "WITH next AS (
                SELECT intent_id
                FROM makosh_data.telegram_delivery_intent_jobs
                WHERE state BETWEEN 1 AND 3
                  AND next_attempt_at_unix_seconds <= $1
                  AND (state = 3 OR attempt_count < $2)
                  AND (
                    claimed_by IS NULL OR
                    lease_expires_at_unix_seconds IS NULL OR
                    lease_expires_at_unix_seconds <= $1
                  )
                ORDER BY next_attempt_at_unix_seconds, intent_id
                FOR UPDATE SKIP LOCKED
                LIMIT 1
             )
             UPDATE makosh_data.telegram_delivery_intent_jobs job
             SET claimed_by = $3,
                 lease_expires_at_unix_seconds = $4,
                 attempt_count = CASE
                    WHEN job.state = 3 THEN attempt_count
                    ELSE attempt_count + 1
                 END
             FROM next, makosh_data.telegram_delivery_intent_inbox inbox
             WHERE job.intent_id = next.intent_id
               AND inbox.message_id = job.command_message_id
             RETURNING job.*,
                       inbox.envelope_sha256 AS command_envelope_sha256,
                       inbox.logical_owner_id AS logical_owner_id",
        )
        .bind(now_unix_seconds)
        .bind(TELEGRAM_DELIVERY_INTENT_MAX_ATTEMPTS_V1)
        .bind(worker_id)
        .bind(lease_expires_at_unix_seconds)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        row.map(|row| claimed_job_from_row(row, worker_id))
            .transpose()
    }

    pub async fn record_target_body_receipt(
        &self,
        intent_id: [u8; 16],
        worker_id: &str,
        target_body_reference_id: [u8; 16],
        target_body_receipt_sha256: [u8; 32],
        now_unix_seconds: i64,
    ) -> Result<(), TelegramDurablePersistenceError> {
        if intent_id.iter().all(|byte| *byte == 0)
            || worker_id.trim().is_empty()
            || target_body_reference_id.iter().all(|byte| *byte == 0)
            || target_body_receipt_sha256.iter().all(|byte| *byte == 0)
            || now_unix_seconds <= 0
        {
            return Err(TelegramDurablePersistenceError::InvalidRow);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.telegram_delivery_intent_jobs
             SET state = $1,
                 target_body_reference_id = $2,
                 target_body_receipt_sha256 = $3,
                 next_attempt_at_unix_seconds = $4,
                 claimed_by = NULL,
                 lease_expires_at_unix_seconds = NULL
             WHERE intent_id = $5
               AND state = $6
               AND claimed_by = $7
               AND lease_expires_at_unix_seconds > $4",
        )
        .bind(TelegramDeliveryIntentJobStateV1::BodyReady as i16)
        .bind(target_body_reference_id.as_slice())
        .bind(target_body_receipt_sha256.as_slice())
        .bind(now_unix_seconds)
        .bind(intent_id.as_slice())
        .bind(TelegramDeliveryIntentJobStateV1::PendingCustody as i16)
        .bind(worker_id)
        .execute(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(TelegramDurablePersistenceError::InvalidDeliveryIntentTransition);
        }
        Ok(())
    }

    pub async fn mark_delivery_queued(
        &self,
        intent_id: [u8; 16],
        worker_id: &str,
        now_unix_seconds: i64,
    ) -> Result<(), TelegramDurablePersistenceError> {
        if intent_id.iter().all(|byte| *byte == 0)
            || worker_id.trim().is_empty()
            || now_unix_seconds <= 0
        {
            return Err(TelegramDurablePersistenceError::InvalidRow);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.telegram_delivery_intent_jobs
             SET state = $1,
                 next_attempt_at_unix_seconds = $2,
                 claimed_by = NULL,
                 lease_expires_at_unix_seconds = NULL
             WHERE intent_id = $3
               AND state = $4
               AND claimed_by = $5
               AND lease_expires_at_unix_seconds > $2
               AND target_body_reference_id IS NOT NULL
               AND target_body_receipt_sha256 IS NOT NULL",
        )
        .bind(TelegramDeliveryIntentJobStateV1::DeliveryQueued as i16)
        .bind(now_unix_seconds)
        .bind(intent_id.as_slice())
        .bind(TelegramDeliveryIntentJobStateV1::BodyReady as i16)
        .bind(worker_id)
        .execute(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(TelegramDurablePersistenceError::InvalidDeliveryIntentTransition);
        }
        Ok(())
    }

    pub async fn reschedule_claimed_job(
        &self,
        intent_id: [u8; 16],
        worker_id: &str,
        state: TelegramDeliveryIntentJobStateV1,
        next_attempt_at_unix_seconds: i64,
    ) -> Result<(), TelegramDurablePersistenceError> {
        if intent_id.iter().all(|byte| *byte == 0)
            || worker_id.trim().is_empty()
            || !matches!(
                state,
                TelegramDeliveryIntentJobStateV1::PendingCustody
                    | TelegramDeliveryIntentJobStateV1::BodyReady
                    | TelegramDeliveryIntentJobStateV1::DeliveryQueued
            )
            || next_attempt_at_unix_seconds <= 0
        {
            return Err(TelegramDurablePersistenceError::InvalidRow);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.telegram_delivery_intent_jobs
             SET next_attempt_at_unix_seconds = $1,
                 claimed_by = NULL,
                 lease_expires_at_unix_seconds = NULL
             WHERE intent_id = $2
               AND state = $3
               AND claimed_by = $4",
        )
        .bind(next_attempt_at_unix_seconds)
        .bind(intent_id.as_slice())
        .bind(state as i16)
        .bind(worker_id)
        .execute(&self.pool)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(TelegramDurablePersistenceError::InvalidDeliveryIntentTransition);
        }
        Ok(())
    }

    pub async fn complete_claimed_job(
        &self,
        intent_id: [u8; 16],
        worker_id: &str,
        terminal_state: TelegramDeliveryIntentJobStateV1,
        result: &OutboxRecordV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), TelegramDurablePersistenceError> {
        if intent_id.iter().all(|byte| *byte == 0)
            || worker_id.trim().is_empty()
            || !matches!(
                terminal_state,
                TelegramDeliveryIntentJobStateV1::Succeeded
                    | TelegramDeliveryIntentJobStateV1::Rejected
                    | TelegramDeliveryIntentJobStateV1::OutcomeUnknown
            )
            || completed_at_unix_seconds <= 0
        {
            return Err(TelegramDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.telegram_delivery_intent_jobs
             SET state = $1,
                 completed_at_unix_seconds = $2,
                 claimed_by = NULL,
                 lease_expires_at_unix_seconds = NULL
             WHERE intent_id = $3
               AND state BETWEEN 1 AND 3
               AND claimed_by = $4
               AND lease_expires_at_unix_seconds > $2",
        )
        .bind(terminal_state as i16)
        .bind(completed_at_unix_seconds)
        .bind(intent_id.as_slice())
        .bind(worker_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TelegramDurablePersistenceError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(TelegramDurablePersistenceError::InvalidDeliveryIntentTransition);
        }
        crate::delivery_intent_result_outbox::insert_result_outbox(
            &mut transaction,
            intent_id,
            result,
            completed_at_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| TelegramDurablePersistenceError::Database)
    }
}

fn claimed_job_from_row(
    row: sqlx::postgres::PgRow,
    worker_id: &str,
) -> Result<ClaimedTelegramDeliveryIntentJobV1, TelegramDurablePersistenceError> {
    let target_body_reference_id = optional_id::<16>(&row, "target_body_reference_id")?;
    let target_body_receipt_sha256 = optional_id::<32>(&row, "target_body_receipt_sha256")?;
    if target_body_reference_id.is_some() != target_body_receipt_sha256.is_some() {
        return Err(TelegramDurablePersistenceError::InvalidRow);
    }
    Ok(ClaimedTelegramDeliveryIntentJobV1 {
        job: TelegramDeliveryIntentJobV1 {
            intent_id: required_id::<16>(&row, "intent_id")?,
            command_message_id: required_id::<16>(&row, "command_message_id")?,
            command_envelope_sha256: required_id::<32>(&row, "command_envelope_sha256")?,
            logical_owner_id: required_string(&row, "logical_owner_id")?,
            account_id: required_string(&row, "account_id")?,
            provider_chat_id: required_string(&row, "provider_chat_id")?,
            reply_to_provider_message_id: row
                .try_get("reply_to_provider_message_id")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
            body_reference_id: required_id::<16>(&row, "body_reference_id")?,
            body_declared_bytes: u64::try_from(
                row.try_get::<i64, _>("body_declared_bytes")
                    .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
            )
            .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
            body_sha256: required_id::<32>(&row, "body_sha256")?,
            custody_transfer_source_proof: row
                .try_get("custody_transfer_source_proof")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
            provider_operation_id: required_string(&row, "provider_operation_id")?,
        },
        state: TelegramDeliveryIntentJobStateV1::from_i16(
            row.try_get("state")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )
        .ok_or(TelegramDurablePersistenceError::InvalidRow)?,
        target_body_reference_id,
        target_body_receipt_sha256,
        worker_id: worker_id.to_owned(),
        attempt_count: u32::try_from(
            row.try_get::<i32, _>("attempt_count")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        )
        .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
    })
}

fn required_id<const WIDTH: usize>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<[u8; WIDTH], TelegramDurablePersistenceError> {
    row.try_get::<Vec<u8>, _>(column)
        .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?
        .try_into()
        .map_err(|_| TelegramDurablePersistenceError::InvalidRow)
}

fn optional_id<const WIDTH: usize>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<Option<[u8; WIDTH]>, TelegramDurablePersistenceError> {
    row.try_get::<Option<Vec<u8>>, _>(column)
        .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?
        .map(|value| {
            value
                .try_into()
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)
        })
        .transpose()
}

struct ResolvedTelegramDeliveryRouteV1 {
    account_id: String,
    provider_chat_id: String,
    reply_to_provider_message_id: Option<String>,
}

async fn resolve_route(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    admission: &TelegramDeliveryIntentAdmissionV1,
) -> Result<Option<ResolvedTelegramDeliveryRouteV1>, TelegramDurablePersistenceError> {
    let row = if let Some(reply_cursor) = admission.reply_to_source_cursor {
        sqlx::query(
            "SELECT account.account_id, conversation.provider_chat_id,
                    message.provider_message_id AS reply_to_provider_message_id
             FROM makosh_data.telegram_delivery_route_accounts account
             JOIN makosh_data.telegram_delivery_route_conversations conversation
               ON conversation.account_cursor = account.account_cursor
             JOIN makosh_data.telegram_delivery_route_messages message
               ON message.account_cursor = account.account_cursor
              AND message.conversation_cursor = conversation.conversation_cursor
             WHERE account.account_cursor = $1
               AND conversation.conversation_cursor = $2
               AND message.source_cursor = $3
               AND account.active = TRUE",
        )
        .bind(admission.account_source_cursor.as_slice())
        .bind(admission.conversation_source_cursor.as_slice())
        .bind(reply_cursor.as_slice())
        .fetch_optional(&mut **transaction)
        .await
    } else {
        sqlx::query(
            "SELECT account.account_id, conversation.provider_chat_id,
                    NULL::TEXT AS reply_to_provider_message_id
             FROM makosh_data.telegram_delivery_route_accounts account
             JOIN makosh_data.telegram_delivery_route_conversations conversation
               ON conversation.account_cursor = account.account_cursor
             WHERE account.account_cursor = $1
               AND conversation.conversation_cursor = $2
               AND account.active = TRUE",
        )
        .bind(admission.account_source_cursor.as_slice())
        .bind(admission.conversation_source_cursor.as_slice())
        .fetch_optional(&mut **transaction)
        .await
    }
    .map_err(|_| TelegramDurablePersistenceError::Database)?;
    row.map(|row| {
        Ok(ResolvedTelegramDeliveryRouteV1 {
            account_id: required_string(&row, "account_id")?,
            provider_chat_id: required_string(&row, "provider_chat_id")?,
            reply_to_provider_message_id: row
                .try_get("reply_to_provider_message_id")
                .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?,
        })
    })
    .transpose()
}

fn valid_admission(value: &TelegramDeliveryIntentAdmissionV1) -> bool {
    value.command_message_id.iter().any(|byte| *byte != 0)
        && value.envelope_sha256.iter().any(|byte| *byte != 0)
        && value.intent_id.iter().any(|byte| *byte != 0)
        && !value.logical_owner_id.trim().is_empty()
        && value.logical_owner_id.len() <= 256
        && value.account_source_cursor.iter().any(|byte| *byte != 0)
        && value
            .conversation_source_cursor
            .iter()
            .any(|byte| *byte != 0)
        && value
            .reply_to_source_cursor
            .is_none_or(|cursor| cursor.iter().any(|byte| *byte != 0))
        && value.body_reference_id.iter().any(|byte| *byte != 0)
        && (1..=65_536).contains(&value.body_declared_bytes)
        && value.body_sha256.iter().any(|byte| *byte != 0)
        && !value.custody_transfer_source_proof.is_empty()
        && value.custody_transfer_source_proof.len() <= 2_048
}

fn provider_operation_id(intent_id: [u8; 16]) -> String {
    let suffix = intent_id
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("telegram-delivery-intent-{suffix}")
}

fn required_string(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<String, TelegramDurablePersistenceError> {
    let value: String = row
        .try_get(column)
        .map_err(|_| TelegramDurablePersistenceError::InvalidRow)?;
    if value.trim().is_empty() {
        return Err(TelegramDurablePersistenceError::InvalidRow);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_separates_inbox_jobs_and_result_outbox() {
        assert!(TELEGRAM_SCHEMA_V3.contains("telegram_delivery_intent_inbox"));
        assert!(TELEGRAM_SCHEMA_V3.contains("telegram_delivery_intent_jobs"));
        assert!(TELEGRAM_SCHEMA_V3.contains("telegram_delivery_intent_result_outbox"));
        assert!(!TELEGRAM_SCHEMA_V3.contains("communications_"));
    }

    #[test]
    fn provider_operation_identity_is_stable_and_private() {
        let value = provider_operation_id([7; 16]);
        assert_eq!(
            value,
            "telegram-delivery-intent-07070707070707070707070707070707"
        );
        assert!(!value.contains("chat"));
        assert!(!value.contains("account"));
    }
}
