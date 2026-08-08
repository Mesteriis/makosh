//! Mail-owned delivery-intent inbox, resolved job and result outbox.

use makosh_events_protocol::delivery::OutboxRecordV1;
use sqlx::{PgPool, Row};

use crate::MailDurablePersistenceError;

pub const MAIL_SCHEMA_V19: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.mail_delivery_intent_inbox (
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

CREATE TABLE IF NOT EXISTS makosh_data.mail_delivery_intent_jobs (
    intent_id BYTEA PRIMARY KEY,
    command_message_id BYTEA NOT NULL UNIQUE,
    connection_id TEXT NOT NULL,
    provider_thread_id TEXT NOT NULL,
    reply_to_provider_message_id TEXT,
    recipient TEXT NOT NULL,
    subject TEXT NOT NULL,
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
    completed_at_unix_seconds BIGINT,
    CHECK (octet_length(intent_id) = 16),
    CHECK (octet_length(command_message_id) = 16),
    CHECK (length(connection_id) BETWEEN 1 AND 256),
    CHECK (length(provider_thread_id) BETWEEN 1 AND 512),
    CHECK (
        reply_to_provider_message_id IS NULL OR
        length(reply_to_provider_message_id) BETWEEN 1 AND 512
    ),
    CHECK (length(recipient) BETWEEN 1 AND 512),
    CHECK (octet_length(subject) <= 4096),
    CHECK (octet_length(body_reference_id) = 16),
    CHECK (body_declared_bytes BETWEEN 1 AND 65536),
    CHECK (octet_length(body_sha256) = 32),
    CHECK (octet_length(custody_transfer_source_proof) BETWEEN 1 AND 2048),
    CHECK (length(provider_operation_id) BETWEEN 1 AND 128),
    CHECK (attempt_count >= 0)
);

CREATE INDEX IF NOT EXISTS mail_delivery_intent_jobs_claim_idx
    ON makosh_data.mail_delivery_intent_jobs
        (state, next_attempt_at_unix_seconds, intent_id);

CREATE TABLE IF NOT EXISTS makosh_data.mail_delivery_intent_result_outbox (
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

pub const MAIL_SCHEMA_V20: &str = r#"
ALTER TABLE makosh_data.mail_delivery_intent_jobs
    ADD COLUMN target_body_reference_id BYTEA CHECK (
        target_body_reference_id IS NULL OR
        octet_length(target_body_reference_id) = 16
    ),
    ADD COLUMN target_body_receipt_sha256 BYTEA CHECK (
        target_body_receipt_sha256 IS NULL OR
        octet_length(target_body_receipt_sha256) = 32
    ),
    ADD CONSTRAINT mail_delivery_intent_target_body_receipt_complete CHECK (
        (target_body_reference_id IS NULL AND target_body_receipt_sha256 IS NULL)
        OR (
            target_body_reference_id IS NOT NULL AND
            target_body_receipt_sha256 IS NOT NULL
        )
    );
"#;

pub const MAIL_DELIVERY_INTENT_MAX_ATTEMPTS_V1: i32 = 12;

#[derive(Clone)]
pub struct MailDeliveryIntentStoreV1 {
    pub(crate) pool: PgPool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailDeliveryIntentAdmissionV1 {
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
pub struct MailDeliveryIntentJobV1 {
    pub intent_id: [u8; 16],
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub connection_id: String,
    pub provider_thread_id: String,
    pub reply_to_provider_message_id: Option<String>,
    pub recipient: String,
    pub subject: String,
    pub body_reference_id: [u8; 16],
    pub body_declared_bytes: u64,
    pub body_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
    pub provider_operation_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum MailDeliveryIntentJobStateV1 {
    PendingCustody = 1,
    BodyReady = 2,
    DeliveryQueued = 3,
    Succeeded = 4,
    Rejected = 5,
    OutcomeUnknown = 6,
}

impl MailDeliveryIntentJobStateV1 {
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
pub struct ClaimedMailDeliveryIntentJobV1 {
    pub job: MailDeliveryIntentJobV1,
    pub state: MailDeliveryIntentJobStateV1,
    pub target_body_reference_id: Option<[u8; 16]>,
    pub target_body_receipt_sha256: Option<[u8; 32]>,
    pub worker_id: String,
    pub attempt_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailDeliveryIntentInboxOutcomeV1 {
    Pending,
    RouteNotFound,
    DuplicatePending,
    DuplicateRouteNotFound,
}

impl MailDeliveryIntentStoreV1 {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn accept_command(
        &self,
        admission: &MailDeliveryIntentAdmissionV1,
        route_not_found_result: &OutboxRecordV1,
        consumed_at_unix_seconds: i64,
    ) -> Result<MailDeliveryIntentInboxOutcomeV1, MailDurablePersistenceError> {
        if !valid_admission(admission) || consumed_at_unix_seconds <= 0 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.mail_delivery_intent_inbox
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
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if inserted.rows_affected() == 0 {
            let row = sqlx::query(
                "SELECT envelope_sha256, intent_id, logical_owner_id, state
                 FROM makosh_data.mail_delivery_intent_inbox
                 WHERE message_id = $1",
            )
            .bind(admission.command_message_id.as_slice())
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
            let hash: Vec<u8> = row
                .try_get("envelope_sha256")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            let intent_id: Vec<u8> = row
                .try_get("intent_id")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            let logical_owner_id: String = row
                .try_get("logical_owner_id")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            let state: i16 = row
                .try_get("state")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            if hash.as_slice() != admission.envelope_sha256
                || intent_id.as_slice() != admission.intent_id
                || logical_owner_id != admission.logical_owner_id
            {
                return Err(MailDurablePersistenceError::ConflictingEventInbox);
            }
            transaction
                .commit()
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
            return match state {
                1 => Ok(MailDeliveryIntentInboxOutcomeV1::DuplicatePending),
                2 => Ok(MailDeliveryIntentInboxOutcomeV1::DuplicateRouteNotFound),
                _ => Err(MailDurablePersistenceError::InvalidRow),
            };
        }

        let route = resolve_route(&mut transaction, admission).await?;
        let outcome = if let Some(route) = route {
            sqlx::query(
                "INSERT INTO makosh_data.mail_delivery_intent_jobs
                    (intent_id, command_message_id, connection_id, provider_thread_id,
                     reply_to_provider_message_id, recipient, subject, body_reference_id,
                     body_declared_bytes, body_sha256, custody_transfer_source_proof,
                     provider_operation_id, next_attempt_at_unix_seconds)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
            )
            .bind(admission.intent_id.as_slice())
            .bind(admission.command_message_id.as_slice())
            .bind(route.connection_id)
            .bind(route.provider_thread_id)
            .bind(route.reply_to_provider_message_id)
            .bind(route.recipient)
            .bind(route.subject)
            .bind(admission.body_reference_id.as_slice())
            .bind(
                i64::try_from(admission.body_declared_bytes)
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            )
            .bind(admission.body_sha256.as_slice())
            .bind(&admission.custody_transfer_source_proof)
            .bind(provider_operation_id(admission.intent_id))
            .bind(consumed_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
            sqlx::query(
                "UPDATE makosh_data.mail_delivery_intent_inbox SET state = 1
                 WHERE message_id = $1 AND state = 0",
            )
            .bind(admission.command_message_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
            MailDeliveryIntentInboxOutcomeV1::Pending
        } else {
            crate::delivery_intent_result_outbox::insert_result_outbox(
                &mut transaction,
                admission.intent_id,
                route_not_found_result,
                consumed_at_unix_seconds,
            )
            .await?;
            sqlx::query(
                "UPDATE makosh_data.mail_delivery_intent_inbox SET state = 2
                 WHERE message_id = $1 AND state = 0",
            )
            .bind(admission.command_message_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
            MailDeliveryIntentInboxOutcomeV1::RouteNotFound
        };
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        Ok(outcome)
    }

    pub async fn claim_next_job(
        &self,
        worker_id: &str,
        now_unix_seconds: i64,
        lease_expires_at_unix_seconds: i64,
    ) -> Result<Option<ClaimedMailDeliveryIntentJobV1>, MailDurablePersistenceError> {
        if worker_id.trim().is_empty()
            || worker_id.len() > 128
            || now_unix_seconds <= 0
            || lease_expires_at_unix_seconds <= now_unix_seconds
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let row = sqlx::query(
            "WITH next AS (
                SELECT intent_id
                FROM makosh_data.mail_delivery_intent_jobs
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
             UPDATE makosh_data.mail_delivery_intent_jobs job
             SET claimed_by = $3,
                 lease_expires_at_unix_seconds = $4,
                 attempt_count = CASE
                    WHEN job.state = 3 THEN attempt_count
                    ELSE attempt_count + 1
                 END
             FROM next, makosh_data.mail_delivery_intent_inbox inbox
             WHERE job.intent_id = next.intent_id
               AND inbox.message_id = job.command_message_id
             RETURNING job.*,
                       inbox.envelope_sha256 AS command_envelope_sha256,
                       inbox.logical_owner_id AS logical_owner_id",
        )
        .bind(now_unix_seconds)
        .bind(MAIL_DELIVERY_INTENT_MAX_ATTEMPTS_V1)
        .bind(worker_id)
        .bind(lease_expires_at_unix_seconds)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
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
    ) -> Result<(), MailDurablePersistenceError> {
        if intent_id.iter().all(|byte| *byte == 0)
            || worker_id.trim().is_empty()
            || target_body_reference_id.iter().all(|byte| *byte == 0)
            || target_body_receipt_sha256.iter().all(|byte| *byte == 0)
            || now_unix_seconds <= 0
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_delivery_intent_jobs
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
        .bind(MailDeliveryIntentJobStateV1::BodyReady as i16)
        .bind(target_body_reference_id.as_slice())
        .bind(target_body_receipt_sha256.as_slice())
        .bind(now_unix_seconds)
        .bind(intent_id.as_slice())
        .bind(MailDeliveryIntentJobStateV1::PendingCustody as i16)
        .bind(worker_id)
        .execute(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(MailDurablePersistenceError::InvalidDeliveryIntentTransition);
        }
        Ok(())
    }

    pub async fn mark_delivery_queued(
        &self,
        intent_id: [u8; 16],
        worker_id: &str,
        now_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if intent_id.iter().all(|byte| *byte == 0)
            || worker_id.trim().is_empty()
            || now_unix_seconds <= 0
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_delivery_intent_jobs
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
        .bind(MailDeliveryIntentJobStateV1::DeliveryQueued as i16)
        .bind(now_unix_seconds)
        .bind(intent_id.as_slice())
        .bind(MailDeliveryIntentJobStateV1::BodyReady as i16)
        .bind(worker_id)
        .execute(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(MailDurablePersistenceError::InvalidDeliveryIntentTransition);
        }
        Ok(())
    }

    pub async fn reschedule_claimed_job(
        &self,
        intent_id: [u8; 16],
        worker_id: &str,
        state: MailDeliveryIntentJobStateV1,
        next_attempt_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if intent_id.iter().all(|byte| *byte == 0)
            || worker_id.trim().is_empty()
            || !matches!(
                state,
                MailDeliveryIntentJobStateV1::PendingCustody
                    | MailDeliveryIntentJobStateV1::BodyReady
                    | MailDeliveryIntentJobStateV1::DeliveryQueued
            )
            || next_attempt_at_unix_seconds <= 0
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_delivery_intent_jobs
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
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(MailDurablePersistenceError::InvalidDeliveryIntentTransition);
        }
        Ok(())
    }

    pub async fn complete_claimed_job(
        &self,
        intent_id: [u8; 16],
        worker_id: &str,
        terminal_state: MailDeliveryIntentJobStateV1,
        result: &OutboxRecordV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if intent_id.iter().all(|byte| *byte == 0)
            || worker_id.trim().is_empty()
            || !matches!(
                terminal_state,
                MailDeliveryIntentJobStateV1::Succeeded
                    | MailDeliveryIntentJobStateV1::Rejected
                    | MailDeliveryIntentJobStateV1::OutcomeUnknown
            )
            || completed_at_unix_seconds <= 0
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_delivery_intent_jobs
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
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(MailDurablePersistenceError::InvalidDeliveryIntentTransition);
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
            .map_err(|_| MailDurablePersistenceError::Database)
    }
}

fn claimed_job_from_row(
    row: sqlx::postgres::PgRow,
    worker_id: &str,
) -> Result<ClaimedMailDeliveryIntentJobV1, MailDurablePersistenceError> {
    let target_body_reference_id = optional_id::<16>(&row, "target_body_reference_id")?;
    let target_body_receipt_sha256 = optional_id::<32>(&row, "target_body_receipt_sha256")?;
    if target_body_reference_id.is_some() != target_body_receipt_sha256.is_some() {
        return Err(MailDurablePersistenceError::InvalidRow);
    }
    Ok(ClaimedMailDeliveryIntentJobV1 {
        job: MailDeliveryIntentJobV1 {
            intent_id: required_id::<16>(&row, "intent_id")?,
            command_message_id: required_id::<16>(&row, "command_message_id")?,
            command_envelope_sha256: required_id::<32>(&row, "command_envelope_sha256")?,
            logical_owner_id: required_string(&row, "logical_owner_id")?,
            connection_id: required_string(&row, "connection_id")?,
            provider_thread_id: required_string(&row, "provider_thread_id")?,
            reply_to_provider_message_id: row
                .try_get("reply_to_provider_message_id")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            recipient: required_string(&row, "recipient")?,
            subject: required_string(&row, "subject")?,
            body_reference_id: required_id::<16>(&row, "body_reference_id")?,
            body_declared_bytes: u64::try_from(
                row.try_get::<i64, _>("body_declared_bytes")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            )
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            body_sha256: required_id::<32>(&row, "body_sha256")?,
            custody_transfer_source_proof: row
                .try_get("custody_transfer_source_proof")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            provider_operation_id: required_string(&row, "provider_operation_id")?,
        },
        state: MailDeliveryIntentJobStateV1::from_i16(
            row.try_get("state")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        )
        .ok_or(MailDurablePersistenceError::InvalidRow)?,
        target_body_reference_id,
        target_body_receipt_sha256,
        worker_id: worker_id.to_owned(),
        attempt_count: u32::try_from(
            row.try_get::<i32, _>("attempt_count")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        )
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
    })
}

fn required_id<const WIDTH: usize>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<[u8; WIDTH], MailDurablePersistenceError> {
    row.try_get::<Vec<u8>, _>(column)
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?
        .try_into()
        .map_err(|_| MailDurablePersistenceError::InvalidRow)
}

fn optional_id<const WIDTH: usize>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<Option<[u8; WIDTH]>, MailDurablePersistenceError> {
    row.try_get::<Option<Vec<u8>>, _>(column)
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?
        .map(|value| {
            value
                .try_into()
                .map_err(|_| MailDurablePersistenceError::InvalidRow)
        })
        .transpose()
}

fn required_string(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<String, MailDurablePersistenceError> {
    let value: String = row
        .try_get(column)
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    if value.trim().is_empty() {
        return Err(MailDurablePersistenceError::InvalidRow);
    }
    Ok(value)
}

struct ResolvedMailDeliveryRouteV1 {
    connection_id: String,
    provider_thread_id: String,
    reply_to_provider_message_id: Option<String>,
    recipient: String,
    subject: String,
}

async fn resolve_route(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    admission: &MailDeliveryIntentAdmissionV1,
) -> Result<Option<ResolvedMailDeliveryRouteV1>, MailDurablePersistenceError> {
    let row = if let Some(reply_cursor) = admission.reply_to_source_cursor {
        sqlx::query(
            "SELECT conversation.connection_id, conversation.provider_thread_id,
                    message.provider_message_id AS reply_to_provider_message_id,
                    message.sender AS recipient, message.subject
             FROM makosh_data.mail_delivery_route_accounts AS account
             JOIN makosh_data.mail_delivery_route_conversations AS conversation
               ON conversation.account_cursor = account.account_cursor
             JOIN makosh_data.mail_delivery_route_messages AS message
               ON message.source_cursor = $3
              AND message.account_cursor = account.account_cursor
              AND message.conversation_cursor = conversation.conversation_cursor
             WHERE account.account_cursor = $1
               AND conversation.conversation_cursor = $2
               AND account.active = TRUE",
        )
        .bind(admission.account_source_cursor.as_slice())
        .bind(admission.conversation_source_cursor.as_slice())
        .bind(reply_cursor.as_slice())
        .fetch_optional(&mut **transaction)
        .await
    } else {
        sqlx::query(
            "SELECT conversation.connection_id, conversation.provider_thread_id,
                    NULL::TEXT AS reply_to_provider_message_id,
                    conversation.last_sender AS recipient, conversation.subject
             FROM makosh_data.mail_delivery_route_accounts AS account
             JOIN makosh_data.mail_delivery_route_conversations AS conversation
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
    .map_err(|_| MailDurablePersistenceError::Database)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let recipient: Option<String> = row
        .try_get("recipient")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let Some(recipient) = recipient.filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    Ok(Some(ResolvedMailDeliveryRouteV1 {
        connection_id: row
            .try_get("connection_id")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        provider_thread_id: row
            .try_get("provider_thread_id")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        reply_to_provider_message_id: row
            .try_get("reply_to_provider_message_id")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        recipient,
        subject: reply_subject(
            &row.try_get::<String, _>("subject")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        ),
    }))
}

fn valid_admission(value: &MailDeliveryIntentAdmissionV1) -> bool {
    value.command_message_id.iter().any(|byte| *byte != 0)
        && value.envelope_sha256.iter().any(|byte| *byte != 0)
        && value.intent_id.iter().any(|byte| *byte != 0)
        && !value.logical_owner_id.is_empty()
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
        && (1..=2_048).contains(&value.custody_transfer_source_proof.len())
}

fn provider_operation_id(intent_id: [u8; 16]) -> String {
    let mut value = String::with_capacity(53);
    value.push_str("mail-delivery-intent-");
    for byte in intent_id {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}").expect("write to String");
    }
    value
}

fn reply_subject(value: &str) -> String {
    if value.to_ascii_lowercase().starts_with("re:") {
        value.to_owned()
    } else if value.is_empty() {
        "Re:".to_owned()
    } else {
        format!("Re: {value}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_identity_is_stable_and_does_not_expose_route_or_body() {
        assert_eq!(
            provider_operation_id([0xab; 16]),
            "mail-delivery-intent-abababababababababababababababab"
        );
    }

    #[test]
    fn schema_separates_inbox_jobs_and_result_outbox() {
        assert!(MAIL_SCHEMA_V19.contains("mail_delivery_intent_inbox"));
        assert!(MAIL_SCHEMA_V19.contains("mail_delivery_intent_jobs"));
        assert!(MAIL_SCHEMA_V19.contains("mail_delivery_intent_result_outbox"));
        assert!(!MAIL_SCHEMA_V19.contains("communications_"));
        assert!(!MAIL_SCHEMA_V19.contains("telegram"));
    }

    #[test]
    fn custody_checkpoint_is_complete_and_kept_in_the_mail_owned_job() {
        assert!(MAIL_SCHEMA_V20.contains("target_body_reference_id BYTEA"));
        assert!(MAIL_SCHEMA_V20.contains("target_body_receipt_sha256 BYTEA"));
        assert!(MAIL_SCHEMA_V20.contains("mail_delivery_intent_target_body_receipt_complete"));
        assert!(!MAIL_SCHEMA_V20.contains("communications_"));
    }

    #[test]
    fn reply_subject_is_stable_and_does_not_duplicate_prefix() {
        assert_eq!(reply_subject("Subject"), "Re: Subject");
        assert_eq!(reply_subject("Re: Subject"), "Re: Subject");
        assert_eq!(reply_subject(""), "Re:");
    }
}
