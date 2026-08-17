//! Mail-owned durable storage. It never reads or mutates Communications state.

use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
};

use crate::{
    MAIL_SCHEMA_V5, MailAttachmentDispositionV1, MailAttachmentMaterializationV1,
    MailAttachmentSafetyStateV1, MailAttachmentSafetyTransitionV1,
    MailDeliveryAttachmentManifestV1, MailOperationalMaterializationV1,
    operational::record_operational_materializations_in_transaction,
};

pub const MAIL_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.mail_communications_outbox (
    message_id BYTEA PRIMARY KEY,
    envelope_sha256 BYTEA NOT NULL,
    exact_envelope_bytes BYTEA NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    published_at_unix_seconds BIGINT,
    CHECK (octet_length(message_id) = 16),
    CHECK (octet_length(envelope_sha256) = 32),
    CHECK (octet_length(exact_envelope_bytes) > 0)
);
CREATE INDEX IF NOT EXISTS mail_communications_outbox_pending_idx
    ON makosh_data.mail_communications_outbox (created_at_unix_seconds, message_id)
    WHERE published_at_unix_seconds IS NULL;
CREATE TABLE IF NOT EXISTS makosh_data.mail_communications_event_inbox (
    message_id BYTEA PRIMARY KEY,
    envelope_sha256 BYTEA NOT NULL,
    consumed_at_unix_seconds BIGINT NOT NULL,
    CHECK (octet_length(message_id) = 16),
    CHECK (octet_length(envelope_sha256) = 32)
);
CREATE TABLE IF NOT EXISTS makosh_data.mail_attachment_anchor_mappings (
    source_observation_id BYTEA PRIMARY KEY,
    attachment_anchor_id BYTEA NOT NULL UNIQUE,
    correlation_id BYTEA NOT NULL,
    media_cursor_sha256 BYTEA NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    CHECK (octet_length(source_observation_id) = 16),
    CHECK (octet_length(attachment_anchor_id) = 16),
    CHECK (octet_length(correlation_id) = 16),
    CHECK (octet_length(media_cursor_sha256) = 32)
);
CREATE TABLE IF NOT EXISTS makosh_data.mail_attachment_blob_admissions (
    source_observation_id BYTEA PRIMARY KEY,
    attachment_anchor_id BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    started_at_unix_seconds BIGINT NOT NULL,
    completed_at_unix_seconds BIGINT,
    CHECK (octet_length(source_observation_id) = 16),
    CHECK (octet_length(attachment_anchor_id) = 16),
    CHECK (state IN (1, 2, 3)),
    CHECK ((state = 1 AND completed_at_unix_seconds IS NULL)
        OR (state IN (2, 3) AND completed_at_unix_seconds IS NOT NULL))
);
CREATE TABLE IF NOT EXISTS makosh_data.mail_delivery_attempts (
    operation_id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    rfc822_sha256 BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    attempted_at_unix_seconds BIGINT NOT NULL,
    completed_at_unix_seconds BIGINT,
    response_code SMALLINT,
    CHECK (operation_id <> ''),
    CHECK (connection_id <> ''),
    CHECK (octet_length(rfc822_sha256) = 32),
    CHECK (state IN (1, 2, 3)),
    CHECK ((state = 1 AND completed_at_unix_seconds IS NULL AND response_code IS NULL)
        OR (state = 2 AND completed_at_unix_seconds IS NOT NULL AND response_code BETWEEN 200 AND 299)
        OR (state = 3 AND completed_at_unix_seconds IS NOT NULL AND response_code IS NULL))
);
CREATE INDEX IF NOT EXISTS mail_delivery_attempts_unresolved_idx
    ON makosh_data.mail_delivery_attempts (attempted_at_unix_seconds, operation_id)
    WHERE state = 1;
CREATE TABLE IF NOT EXISTS makosh_data.mail_gmail_sync_cursors (
    connection_id TEXT PRIMARY KEY,
    next_page_token TEXT NOT NULL,
    observed_history_id TEXT,
    updated_at_unix_seconds BIGINT NOT NULL,
    CHECK (connection_id <> ''),
    CHECK (next_page_token <> ''),
    CHECK (observed_history_id IS NULL OR observed_history_id <> ''),
    CHECK (updated_at_unix_seconds > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.mail_gmail_history_checkpoints (
    connection_id TEXT PRIMARY KEY,
    start_history_id TEXT NOT NULL,
    next_page_token TEXT,
    updated_at_unix_seconds BIGINT NOT NULL,
    CHECK (connection_id <> ''),
    CHECK (start_history_id <> ''),
    CHECK (next_page_token IS NULL OR next_page_token <> ''),
    CHECK (updated_at_unix_seconds > 0)
);
CREATE TABLE IF NOT EXISTS makosh_data.mail_gmail_oauth_credential_bindings (
    connection_id TEXT PRIMARY KEY,
    access_token_record_id BYTEA NOT NULL,
    access_token_revision BIGINT NOT NULL,
    refresh_credential_record_id BYTEA NOT NULL,
    refresh_credential_revision BIGINT NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    CHECK (connection_id <> ''),
    CHECK (octet_length(access_token_record_id) = 16),
    CHECK (access_token_revision > 0),
    CHECK (octet_length(refresh_credential_record_id) = 16),
    CHECK (refresh_credential_revision > 0),
    CHECK (updated_at_unix_seconds > 0)
);
"#;

pub const MAIL_SCHEMA_V2: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.mail_attachment_security_outbox (
    message_id BYTEA PRIMARY KEY,
    envelope_sha256 BYTEA NOT NULL,
    exact_envelope_bytes BYTEA NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    published_at_unix_seconds BIGINT,
    CHECK (octet_length(message_id) = 16),
    CHECK (octet_length(envelope_sha256) = 32),
    CHECK (octet_length(exact_envelope_bytes) > 0)
);
CREATE INDEX IF NOT EXISTS mail_attachment_security_outbox_pending_idx
    ON makosh_data.mail_attachment_security_outbox (created_at_unix_seconds, message_id)
    WHERE published_at_unix_seconds IS NULL;
"#;

pub const MAIL_SCHEMA_V3: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.mail_delivery_queue (
    operation_id TEXT PRIMARY KEY
        REFERENCES makosh_data.mail_delivery_attempts (operation_id) ON DELETE CASCADE,
    exact_command_bytes BYTEA NOT NULL,
    dispatched_at_unix_seconds BIGINT,
    CHECK (octet_length(exact_command_bytes) > 0),
    CHECK (dispatched_at_unix_seconds IS NULL OR dispatched_at_unix_seconds > 0)
);
CREATE INDEX IF NOT EXISTS mail_delivery_queue_pending_idx
    ON makosh_data.mail_delivery_queue (operation_id)
    WHERE dispatched_at_unix_seconds IS NULL;
"#;

pub const MAIL_SCHEMA_V6: &str = r#"
ALTER TABLE makosh_data.mail_communications_outbox
    ADD COLUMN IF NOT EXISTS causal_sequence BIGINT GENERATED BY DEFAULT AS IDENTITY;
CREATE INDEX IF NOT EXISTS mail_communications_outbox_causal_pending_idx
    ON makosh_data.mail_communications_outbox (causal_sequence)
    WHERE published_at_unix_seconds IS NULL;
"#;

#[derive(Clone)]
pub struct MailDurablePersistence {
    pub(crate) pool: PgPool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailDurablePersistenceError {
    Database,
    InvalidRow,
    MissingOperationalMessage,
    MissingSourceObservation,
    ConflictingAnchorMapping,
    ConflictingEventInbox,
    MissingAttachmentAdmission,
    InvalidAttachmentAdmissionState,
    MissingAttachmentMaterialization,
    ConflictingAttachmentMaterialization,
    ConflictingAttachmentSafetyProjection,
    UnsafeAttachment,
    InvalidAttachmentManifest,
    MissingSyncRun,
    ConflictingSyncOperation,
    ConflictingDeliveryRouteLocator,
    InvalidDeliveryIntentTransition,
    SyncRunInProgress,
    InvalidSyncTransition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAttachmentAnchorMappingOutcomeV1 {
    Applied,
    AlreadyApplied,
    IgnoredForeignSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MailAttachmentAnchorMappingV1 {
    pub source_observation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub correlation_id: [u8; 16],
    pub media_cursor_sha256: [u8; 32],
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAttachmentBlobAdmissionStartOutcomeV1 {
    Started,
    AlreadyStarted,
    AlreadyTerminal,
}

pub struct MailAttachmentBlobAdmissionCompletionV1<'a> {
    pub source_observation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub terminal_state: i16,
    pub terminal_record: &'a OutboxRecordV1,
    pub attachment_security_record: Option<&'a OutboxRecordV1>,
    pub materialization: Option<&'a MailAttachmentMaterializationV1>,
    pub completed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum MailSmtpDeliveryAttemptStateV1 {
    Pending = 1,
    Accepted = 2,
    Rejected = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailDeliveryEnqueueOutcomeV1 {
    Enqueued,
    Existing,
}

pub struct MailDeliveryEnqueueRequestV1<'a> {
    pub operation_id: &'a str,
    pub connection_id: &'a str,
    pub request_sha256: &'a [u8; 32],
    pub exact_command_bytes: &'a [u8],
    pub attachment_anchor_ids: &'a [[u8; 16]],
    pub max_attachment_bytes: u64,
    pub requested_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailQueuedDeliveryV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub request_sha256: [u8; 32],
    pub legacy_rfc822_sha256: Option<[u8; 32]>,
    pub rendered_rfc822_sha256: Option<[u8; 32]>,
    pub exact_command_bytes: Vec<u8>,
    pub attachments: Vec<MailDeliveryAttachmentManifestV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailDeliveryAttemptOutcomeV1 {
    Pending,
    Accepted { response_code: u16 },
    Rejected,
    OutcomeUnknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailDeliveryAttemptV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub outcome: MailDeliveryAttemptOutcomeV1,
    pub requested_at_unix_seconds: i64,
    pub completed_at_unix_seconds: Option<i64>,
}

impl MailDurablePersistence {
    #[must_use]
    pub fn delivery_intent_store(&self) -> crate::MailDeliveryIntentStoreV1 {
        crate::MailDeliveryIntentStoreV1::new(self.pool.clone())
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, MailDurablePersistenceError> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || binding.access().runtime_principal().is_empty()
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let port =
            u16::try_from(pgbouncer_port).map_err(|_| MailDurablePersistenceError::InvalidRow)?;
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
            .map_err(|_| MailDurablePersistenceError::Database)?;
        Ok(Self { pool })
    }

    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Shares the already budgeted Mail-owned pool with a separately admitted
    /// integration persistence build unit. The runtime remains the only
    /// composition root; persistence packages do not import each other.
    #[must_use]
    pub fn owner_local_pool_handle(&self) -> PgPool {
        self.pool.clone()
    }

    pub async fn initialize(&self) -> Result<(), MailDurablePersistenceError> {
        sqlx::raw_sql(MAIL_SCHEMA_V1)
            .execute(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        sqlx::raw_sql(MAIL_SCHEMA_V2)
            .execute(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        sqlx::raw_sql(MAIL_SCHEMA_V3)
            .execute(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        sqlx::raw_sql(crate::MAIL_SCHEMA_V4)
            .execute(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        sqlx::raw_sql(MAIL_SCHEMA_V5)
            .execute(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        sqlx::raw_sql(MAIL_SCHEMA_V6)
            .execute(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        sqlx::raw_sql(crate::MAIL_SCHEMA_V7)
            .execute(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        sqlx::raw_sql(crate::MAIL_SCHEMA_V8)
            .execute(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        sqlx::raw_sql(crate::MAIL_SCHEMA_V9)
            .execute(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        sqlx::raw_sql(crate::MAIL_SCHEMA_V10)
            .execute(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        sqlx::raw_sql(crate::MAIL_ICLOUD_CARDDAV_CREDENTIAL_SCHEMA_V1)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn enqueue_communications_outbox(
        &self,
        record: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        sqlx::query("INSERT INTO makosh_data.mail_communications_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
            .bind(record.message_id().as_slice())
            .bind(record.envelope_sha256().as_slice())
            .bind(record.exact_bytes())
            .bind(created_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn communications_outbox_record(
        &self,
        message_id: [u8; 16],
    ) -> Result<Option<OutboxRecordV1>, MailDurablePersistenceError> {
        sqlx::query(
            "SELECT exact_envelope_bytes FROM makosh_data.mail_communications_outbox WHERE message_id = $1",
        )
        .bind(message_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?
        .map(|row| {
            let bytes: Vec<u8> = row
                .try_get("exact_envelope_bytes")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            OutboxRecordV1::accept(bytes).map_err(|_| MailDurablePersistenceError::InvalidRow)
        })
        .transpose()
    }

    pub async fn persist_attachment_anchor_mapping(
        &self,
        handoff_record: &OutboxRecordV1,
        mapping: &MailAttachmentAnchorMappingV1,
        consumed_at_unix_seconds: i64,
    ) -> Result<MailAttachmentAnchorMappingOutcomeV1, MailDurablePersistenceError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let existing = sqlx::query(
            "SELECT envelope_sha256 FROM makosh_data.mail_communications_event_inbox WHERE message_id = $1",
        )
        .bind(handoff_record.message_id().as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if let Some(row) = existing {
            let digest: Vec<u8> = row
                .try_get("envelope_sha256")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            if digest.as_slice() != handoff_record.envelope_sha256().as_slice() {
                return Err(MailDurablePersistenceError::ConflictingEventInbox);
            }
            transaction
                .commit()
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
            return Ok(MailAttachmentAnchorMappingOutcomeV1::AlreadyApplied);
        }
        let source_exists = sqlx::query(
            "SELECT 1 FROM makosh_data.mail_communications_outbox WHERE message_id = $1",
        )
        .bind(mapping.source_observation_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if source_exists.is_none() {
            return Err(MailDurablePersistenceError::MissingSourceObservation);
        }
        let existing_mapping = sqlx::query("SELECT attachment_anchor_id, correlation_id, media_cursor_sha256 FROM makosh_data.mail_attachment_anchor_mappings WHERE source_observation_id = $1")
            .bind(mapping.source_observation_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        if let Some(row) = existing_mapping {
            let anchor: Vec<u8> = row
                .try_get("attachment_anchor_id")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            let cursor: Vec<u8> = row
                .try_get("media_cursor_sha256")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            let persisted_correlation: Vec<u8> = row
                .try_get("correlation_id")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            if anchor.as_slice() != mapping.attachment_anchor_id.as_slice()
                || persisted_correlation.as_slice() != mapping.correlation_id.as_slice()
                || cursor.as_slice() != mapping.media_cursor_sha256.as_slice()
            {
                return Err(MailDurablePersistenceError::ConflictingAnchorMapping);
            }
        } else {
            sqlx::query("INSERT INTO makosh_data.mail_attachment_anchor_mappings (source_observation_id, attachment_anchor_id, correlation_id, media_cursor_sha256, observed_at_unix_seconds) VALUES ($1, $2, $3, $4, $5)")
                .bind(mapping.source_observation_id.as_slice())
                .bind(mapping.attachment_anchor_id.as_slice())
                .bind(mapping.correlation_id.as_slice())
                .bind(mapping.media_cursor_sha256.as_slice())
                .bind(mapping.observed_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_attachment_safety_projections \
             (attachment_anchor_id, state) VALUES ($1, $2) \
             ON CONFLICT (attachment_anchor_id) DO NOTHING",
        )
        .bind(mapping.attachment_anchor_id.as_slice())
        .bind(MailAttachmentSafetyStateV1::DescriptorOnly as i16)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        sqlx::query("INSERT INTO makosh_data.mail_communications_event_inbox (message_id, envelope_sha256, consumed_at_unix_seconds) VALUES ($1, $2, $3)")
            .bind(handoff_record.message_id().as_slice())
            .bind(handoff_record.envelope_sha256().as_slice())
            .bind(consumed_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        Ok(MailAttachmentAnchorMappingOutcomeV1::Applied)
    }

    pub async fn attachment_anchor_mapping(
        &self,
        source_observation_id: [u8; 16],
    ) -> Result<Option<MailAttachmentAnchorMappingV1>, MailDurablePersistenceError> {
        sqlx::query("SELECT attachment_anchor_id, correlation_id, media_cursor_sha256, observed_at_unix_seconds FROM makosh_data.mail_attachment_anchor_mappings WHERE source_observation_id = $1")
            .bind(source_observation_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?
            .map(|row| {
                let attachment_anchor_id: Vec<u8> = row.try_get("attachment_anchor_id").map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                let correlation_id: Vec<u8> = row.try_get("correlation_id").map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                let media_cursor_sha256: Vec<u8> = row.try_get("media_cursor_sha256").map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                let observed_at_unix_seconds: i64 = row.try_get("observed_at_unix_seconds").map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                let attachment_anchor_id: [u8; 16] = attachment_anchor_id.as_slice().try_into().map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                let correlation_id: [u8; 16] = correlation_id.as_slice().try_into().map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                let media_cursor_sha256: [u8; 32] = media_cursor_sha256.as_slice().try_into().map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                if attachment_anchor_id.iter().all(|byte| *byte == 0)
                    || correlation_id.iter().all(|byte| *byte == 0)
                    || media_cursor_sha256.iter().all(|byte| *byte == 0)
                {
                    return Err(MailDurablePersistenceError::InvalidRow);
                }
                Ok(MailAttachmentAnchorMappingV1 {
                    source_observation_id,
                    attachment_anchor_id,
                    correlation_id,
                    media_cursor_sha256,
                    observed_at_unix_seconds,
                })
            })
            .transpose()
    }

    pub async fn begin_attachment_blob_admission(
        &self,
        source_observation_id: [u8; 16],
        attachment_anchor_id: [u8; 16],
        requested_record: &OutboxRecordV1,
        started_at_unix_seconds: i64,
    ) -> Result<MailAttachmentBlobAdmissionStartOutcomeV1, MailDurablePersistenceError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let mapping = sqlx::query("SELECT attachment_anchor_id FROM makosh_data.mail_attachment_anchor_mappings WHERE source_observation_id = $1")
            .bind(source_observation_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?
            .ok_or(MailDurablePersistenceError::MissingSourceObservation)?;
        let mapped_anchor: Vec<u8> = mapping
            .try_get("attachment_anchor_id")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        if mapped_anchor.as_slice() != attachment_anchor_id.as_slice() {
            return Err(MailDurablePersistenceError::ConflictingAnchorMapping);
        }
        let existing = sqlx::query("SELECT attachment_anchor_id, state FROM makosh_data.mail_attachment_blob_admissions WHERE source_observation_id = $1")
            .bind(source_observation_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        if let Some(row) = existing {
            let anchor: Vec<u8> = row
                .try_get("attachment_anchor_id")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            let state: i16 = row
                .try_get("state")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            if anchor.as_slice() != attachment_anchor_id.as_slice() {
                return Err(MailDurablePersistenceError::ConflictingAnchorMapping);
            }
            let outcome = match state {
                1 => MailAttachmentBlobAdmissionStartOutcomeV1::AlreadyStarted,
                2 | 3 => MailAttachmentBlobAdmissionStartOutcomeV1::AlreadyTerminal,
                _ => return Err(MailDurablePersistenceError::InvalidAttachmentAdmissionState),
            };
            transaction
                .commit()
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
            return Ok(outcome);
        }
        sqlx::query("INSERT INTO makosh_data.mail_attachment_blob_admissions (source_observation_id, attachment_anchor_id, state, started_at_unix_seconds) VALUES ($1, $2, 1, $3)")
            .bind(source_observation_id.as_slice())
            .bind(attachment_anchor_id.as_slice())
            .bind(started_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        insert_communications_outbox(&mut transaction, requested_record, started_at_unix_seconds)
            .await?;
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        Ok(MailAttachmentBlobAdmissionStartOutcomeV1::Started)
    }

    pub async fn complete_attachment_blob_admission(
        &self,
        completion: MailAttachmentBlobAdmissionCompletionV1<'_>,
    ) -> Result<bool, MailDurablePersistenceError> {
        let MailAttachmentBlobAdmissionCompletionV1 {
            source_observation_id,
            attachment_anchor_id,
            terminal_state,
            terminal_record,
            attachment_security_record,
            materialization,
            completed_at_unix_seconds,
        } = completion;
        if !matches!(
            (terminal_state, attachment_security_record, materialization),
            (2, Some(_), Some(_)) | (3, None, None)
        ) {
            return Err(MailDurablePersistenceError::InvalidAttachmentAdmissionState);
        }
        if materialization.is_some_and(|materialization| {
            materialization.source_observation_id != source_observation_id
                || materialization.attachment_anchor_id != attachment_anchor_id
                || !valid_attachment_materialization(materialization)
        }) {
            return Err(MailDurablePersistenceError::InvalidAttachmentAdmissionState);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let outcome = sqlx::query("UPDATE makosh_data.mail_attachment_blob_admissions SET state = $3, completed_at_unix_seconds = $4 WHERE source_observation_id = $1 AND attachment_anchor_id = $2 AND state = 1")
            .bind(source_observation_id.as_slice())
            .bind(attachment_anchor_id.as_slice())
            .bind(terminal_state)
            .bind(completed_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        if outcome.rows_affected() == 0 {
            let existing = sqlx::query("SELECT attachment_anchor_id, state FROM makosh_data.mail_attachment_blob_admissions WHERE source_observation_id = $1")
                .bind(source_observation_id.as_slice())
                .fetch_optional(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?
                .ok_or(MailDurablePersistenceError::MissingAttachmentAdmission)?;
            let anchor: Vec<u8> = existing
                .try_get("attachment_anchor_id")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            let state: i16 = existing
                .try_get("state")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            if anchor.as_slice() != attachment_anchor_id.as_slice() {
                return Err(MailDurablePersistenceError::ConflictingAnchorMapping);
            }
            if !matches!(state, 2 | 3) {
                return Err(MailDurablePersistenceError::InvalidAttachmentAdmissionState);
            }
            transaction
                .commit()
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
            return Ok(false);
        }
        insert_communications_outbox(&mut transaction, terminal_record, completed_at_unix_seconds)
            .await?;
        if let Some(record) = attachment_security_record {
            insert_attachment_security_outbox(&mut transaction, record, completed_at_unix_seconds)
                .await?;
        }
        if let Some(materialization) = materialization {
            insert_attachment_materialization(
                &mut transaction,
                materialization,
                completed_at_unix_seconds,
            )
            .await?;
        }
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        Ok(true)
    }

    pub async fn apply_attachment_safety_transition(
        &self,
        event_record: &OutboxRecordV1,
        transition: MailAttachmentSafetyTransitionV1,
        consumed_at_unix_seconds: i64,
    ) -> Result<bool, MailDurablePersistenceError> {
        if consumed_at_unix_seconds <= 0
            || !valid_attachment_safety_transition(transition)
            || event_record.message_id().iter().all(|byte| *byte == 0)
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let existing = sqlx::query(
            "SELECT envelope_sha256 FROM makosh_data.mail_communications_event_inbox \
             WHERE message_id = $1",
        )
        .bind(event_record.message_id().as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if let Some(row) = existing {
            let digest: Vec<u8> = row
                .try_get("envelope_sha256")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            if digest.as_slice() != event_record.envelope_sha256().as_slice() {
                return Err(MailDurablePersistenceError::ConflictingEventInbox);
            }
            transaction
                .commit()
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
            return Ok(false);
        }
        let mapping_exists = sqlx::query(
            "SELECT 1 FROM makosh_data.mail_attachment_anchor_mappings \
             WHERE attachment_anchor_id = $1",
        )
        .bind(transition.attachment_anchor_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if mapping_exists.is_none() {
            return Err(MailDurablePersistenceError::MissingSourceObservation);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_attachment_safety_projections \
             SET state = $2, evidence_id = $3, observed_at_unix_seconds = $4 \
             WHERE attachment_anchor_id = $1 AND state = $5",
        )
        .bind(transition.attachment_anchor_id.as_slice())
        .bind(transition.next_state as i16)
        .bind(transition.evidence_id.as_slice())
        .bind(transition.observed_at_unix_seconds)
        .bind(transition.expected_state as i16)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(MailDurablePersistenceError::ConflictingAttachmentSafetyProjection);
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_communications_event_inbox \
             (message_id, envelope_sha256, consumed_at_unix_seconds) VALUES ($1, $2, $3)",
        )
        .bind(event_record.message_id().as_slice())
        .bind(event_record.envelope_sha256().as_slice())
        .bind(consumed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        Ok(true)
    }

    pub async fn attachment_safety_state(
        &self,
        attachment_anchor_id: [u8; 16],
    ) -> Result<Option<MailAttachmentSafetyStateV1>, MailDurablePersistenceError> {
        if attachment_anchor_id.iter().all(|byte| *byte == 0) {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let row = sqlx::query(
            "SELECT state FROM makosh_data.mail_attachment_safety_projections \
             WHERE attachment_anchor_id = $1",
        )
        .bind(attachment_anchor_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        row.map(|row| {
            match row
                .try_get::<i16, _>("state")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?
            {
                1 => Ok(MailAttachmentSafetyStateV1::DescriptorOnly),
                2 => Ok(MailAttachmentSafetyStateV1::BlobPending),
                3 => Ok(MailAttachmentSafetyStateV1::BlobAdmitted),
                4 => Ok(MailAttachmentSafetyStateV1::Quarantined),
                5 => Ok(MailAttachmentSafetyStateV1::SafeForDelivery),
                6 => Ok(MailAttachmentSafetyStateV1::Rejected),
                _ => Err(MailDurablePersistenceError::InvalidRow),
            }
        })
        .transpose()
    }

    pub async fn gmail_sync_progress(
        &self,
        connection_id: &str,
    ) -> Result<Option<(String, Option<String>)>, MailDurablePersistenceError> {
        if connection_id.trim().is_empty() {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query("SELECT next_page_token, observed_history_id FROM makosh_data.mail_gmail_sync_cursors WHERE connection_id = $1")
            .bind(connection_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?
            .map(|row| Ok((
                row.try_get("next_page_token").map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                row.try_get("observed_history_id").map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            )))
            .transpose()
    }

    pub async fn gmail_history_checkpoint(
        &self,
        connection_id: &str,
    ) -> Result<Option<(String, Option<String>)>, MailDurablePersistenceError> {
        if connection_id.trim().is_empty() {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query("SELECT start_history_id, next_page_token FROM makosh_data.mail_gmail_history_checkpoints WHERE connection_id = $1")
            .bind(connection_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?
            .map(|row| Ok((
                row.try_get("start_history_id").map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                row.try_get("next_page_token").map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            )))
            .transpose()
    }

    pub async fn record_operational_materializations_and_store_gmail_sync_progress(
        &self,
        materializations: &[MailOperationalMaterializationV1],
        connection_id: &str,
        next_page_token: Option<&str>,
        observed_history_id: Option<&str>,
        updated_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if connection_id.trim().is_empty()
            || updated_at_unix_seconds <= 0
            || next_page_token.is_some_and(|token| token.trim().is_empty())
            || observed_history_id.is_some_and(|history_id| history_id.trim().is_empty())
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        if !materializations.is_empty() {
            record_operational_materializations_in_transaction(
                &mut transaction,
                materializations,
                updated_at_unix_seconds,
            )
            .await?;
        }
        if let Some(next_page_token) = next_page_token {
            sqlx::query("INSERT INTO makosh_data.mail_gmail_sync_cursors (connection_id, next_page_token, observed_history_id, updated_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (connection_id) DO UPDATE SET next_page_token = EXCLUDED.next_page_token, observed_history_id = EXCLUDED.observed_history_id, updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds")
                .bind(connection_id)
                .bind(next_page_token)
                .bind(observed_history_id)
                .bind(updated_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
        } else if let Some(observed_history_id) = observed_history_id {
            sqlx::query("DELETE FROM makosh_data.mail_gmail_sync_cursors WHERE connection_id = $1")
                .bind(connection_id)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
            sqlx::query("INSERT INTO makosh_data.mail_gmail_history_checkpoints (connection_id, start_history_id, next_page_token, updated_at_unix_seconds) VALUES ($1, $2, NULL, $3) ON CONFLICT (connection_id) DO UPDATE SET start_history_id = EXCLUDED.start_history_id, next_page_token = NULL, updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds")
                .bind(connection_id)
                .bind(observed_history_id)
                .bind(updated_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
        } else {
            sqlx::query("DELETE FROM makosh_data.mail_gmail_sync_cursors WHERE connection_id = $1")
                .bind(connection_id)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
        }
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn record_operational_materializations_and_store_gmail_history_checkpoint(
        &self,
        materializations: &[MailOperationalMaterializationV1],
        connection_id: &str,
        start_history_id: &str,
        next_page_token: Option<&str>,
        updated_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if connection_id.trim().is_empty()
            || start_history_id.trim().is_empty()
            || updated_at_unix_seconds <= 0
            || next_page_token.is_some_and(|token| token.trim().is_empty())
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        if !materializations.is_empty() {
            record_operational_materializations_in_transaction(
                &mut transaction,
                materializations,
                updated_at_unix_seconds,
            )
            .await?;
        }
        sqlx::query("INSERT INTO makosh_data.mail_gmail_history_checkpoints (connection_id, start_history_id, next_page_token, updated_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (connection_id) DO UPDATE SET start_history_id = EXCLUDED.start_history_id, next_page_token = EXCLUDED.next_page_token, updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds")
            .bind(connection_id)
            .bind(start_history_id)
            .bind(next_page_token)
            .bind(updated_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn clear_gmail_history_checkpoint(
        &self,
        connection_id: &str,
    ) -> Result<(), MailDurablePersistenceError> {
        if connection_id.trim().is_empty() {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query(
            "DELETE FROM makosh_data.mail_gmail_history_checkpoints WHERE connection_id = $1",
        )
        .bind(connection_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn enqueue_delivery_command(
        &self,
        request: MailDeliveryEnqueueRequestV1<'_>,
    ) -> Result<MailDeliveryEnqueueOutcomeV1, MailDurablePersistenceError> {
        let MailDeliveryEnqueueRequestV1 {
            operation_id,
            connection_id,
            request_sha256,
            exact_command_bytes,
            attachment_anchor_ids,
            max_attachment_bytes,
            requested_at_unix_seconds,
        } = request;
        if operation_id.trim().is_empty()
            || connection_id.trim().is_empty()
            || exact_command_bytes.is_empty()
            || request_sha256.iter().all(|byte| *byte == 0)
            || attachment_anchor_ids.len() > 16
            || max_attachment_bytes == 0
            || requested_at_unix_seconds <= 0
            || attachment_anchor_ids
                .iter()
                .enumerate()
                .any(|(index, anchor_id)| {
                    anchor_id.iter().all(|byte| *byte == 0)
                        || attachment_anchor_ids[..index].contains(anchor_id)
                })
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let inserted = sqlx::query("INSERT INTO makosh_data.mail_delivery_attempts \
            (operation_id, connection_id, rfc822_sha256, request_sha256, state, attempted_at_unix_seconds) \
            VALUES ($1, $2, $3, $3, $4, $5) ON CONFLICT (operation_id) DO NOTHING")
            .bind(operation_id)
            .bind(connection_id)
            .bind(request_sha256.as_slice())
            .bind(MailSmtpDeliveryAttemptStateV1::Pending as i16)
            .bind(requested_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        if inserted.rows_affected() == 1 {
            sqlx::query("INSERT INTO makosh_data.mail_delivery_queue (operation_id, exact_command_bytes) VALUES ($1, $2)")
                .bind(operation_id)
                .bind(exact_command_bytes)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
            let mut total_attachment_bytes = 0_u64;
            for (ordinal, attachment_anchor_id) in attachment_anchor_ids.iter().enumerate() {
                let row = sqlx::query(
                    "SELECT materialization.blob_reference_id, materialization.receipt_sha256, \
                            materialization.declared_size, materialization.filename, \
                            materialization.media_type, materialization.disposition, \
                            projection.evidence_id \
                     FROM makosh_data.mail_attachment_materializations materialization \
                     JOIN makosh_data.mail_attachment_safety_projections projection \
                       ON projection.attachment_anchor_id = materialization.attachment_anchor_id \
                     WHERE materialization.attachment_anchor_id = $1 \
                       AND projection.state = $2 AND projection.evidence_id IS NOT NULL",
                )
                .bind(attachment_anchor_id.as_slice())
                .bind(MailAttachmentSafetyStateV1::SafeForDelivery as i16)
                .fetch_optional(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?
                .ok_or(MailDurablePersistenceError::UnsafeAttachment)?;
                let declared_size: i64 = row
                    .try_get("declared_size")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                let declared_size = u64::try_from(declared_size)
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                total_attachment_bytes = total_attachment_bytes
                    .checked_add(declared_size)
                    .filter(|total| *total <= max_attachment_bytes)
                    .ok_or(MailDurablePersistenceError::InvalidAttachmentManifest)?;
                let ordinal =
                    i16::try_from(ordinal).map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                let blob_reference_id: Vec<u8> = row
                    .try_get("blob_reference_id")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                let receipt_sha256: Vec<u8> = row
                    .try_get("receipt_sha256")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                let safety_evidence_id: Vec<u8> = row
                    .try_get("evidence_id")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                sqlx::query(
                    "INSERT INTO makosh_data.mail_delivery_attachment_manifest \
                     (operation_id, ordinal, attachment_anchor_id, blob_reference_id, \
                      receipt_sha256, declared_size, filename, media_type, disposition, \
                      safety_evidence_id) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                )
                .bind(operation_id)
                .bind(ordinal)
                .bind(attachment_anchor_id.as_slice())
                .bind(blob_reference_id)
                .bind(receipt_sha256)
                .bind(
                    i64::try_from(declared_size)
                        .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                )
                .bind(
                    row.try_get::<Option<String>, _>("filename")
                        .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                )
                .bind(
                    row.try_get::<String, _>("media_type")
                        .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                )
                .bind(
                    row.try_get::<i16, _>("disposition")
                        .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                )
                .bind(safety_evidence_id)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
            }
            transaction
                .commit()
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
            return Ok(MailDeliveryEnqueueOutcomeV1::Enqueued);
        }
        let matching = sqlx::query(
            "SELECT 1 FROM makosh_data.mail_delivery_attempts attempt \
             JOIN makosh_data.mail_delivery_queue queue ON queue.operation_id = attempt.operation_id \
             WHERE attempt.operation_id = $1 AND attempt.connection_id = $2 \
               AND (attempt.request_sha256 = $3 OR attempt.request_sha256 IS NULL) \
               AND queue.exact_command_bytes = $4",
        )
        .bind(operation_id)
        .bind(connection_id)
        .bind(request_sha256.as_slice())
        .bind(exact_command_bytes)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        matching
            .map(|_| MailDeliveryEnqueueOutcomeV1::Existing)
            .ok_or(MailDurablePersistenceError::InvalidRow)
    }

    pub async fn claim_next_delivery(
        &self,
        connection_id: &str,
        dispatched_at_unix_seconds: i64,
    ) -> Result<Option<MailQueuedDeliveryV1>, MailDurablePersistenceError> {
        if connection_id.trim().is_empty() || dispatched_at_unix_seconds <= 0 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let row = sqlx::query(
            "WITH next AS (SELECT queue.operation_id FROM makosh_data.mail_delivery_queue queue \
             JOIN makosh_data.mail_delivery_attempts attempt ON attempt.operation_id = queue.operation_id \
             WHERE queue.dispatched_at_unix_seconds IS NULL AND attempt.state = $1 \
               AND attempt.connection_id = $2 \
             ORDER BY attempt.attempted_at_unix_seconds, queue.operation_id FOR UPDATE SKIP LOCKED LIMIT 1) \
             UPDATE makosh_data.mail_delivery_queue queue SET dispatched_at_unix_seconds = $3 FROM next \
             JOIN makosh_data.mail_delivery_attempts attempt ON attempt.operation_id = next.operation_id \
             WHERE queue.operation_id = next.operation_id \
             RETURNING queue.operation_id, attempt.connection_id, attempt.rfc822_sha256, \
                       attempt.request_sha256, attempt.rendered_rfc822_sha256, \
                       queue.exact_command_bytes",
        )
        .bind(MailSmtpDeliveryAttemptStateV1::Pending as i16)
        .bind(connection_id)
        .bind(dispatched_at_unix_seconds)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        let Some(row) = row else {
            transaction
                .commit()
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
            return Ok(None);
        };
        let operation_id: String = row
            .try_get("operation_id")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        let exact_command_bytes: Vec<u8> = row
            .try_get("exact_command_bytes")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        let legacy_digest: Vec<u8> = row
            .try_get("rfc822_sha256")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        let persisted_request_digest: Option<Vec<u8>> = row
            .try_get("request_sha256")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        let rendered_digest: Option<Vec<u8>> = row
            .try_get("rendered_rfc822_sha256")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        let manifest_rows = sqlx::query(
            "SELECT ordinal, attachment_anchor_id, blob_reference_id, receipt_sha256, \
                    declared_size, filename, media_type, disposition, safety_evidence_id \
             FROM makosh_data.mail_delivery_attachment_manifest \
             WHERE operation_id = $1 ORDER BY ordinal",
        )
        .bind(&operation_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        let attachments = manifest_rows
            .into_iter()
            .map(delivery_attachment_manifest_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        if attachments
            .iter()
            .enumerate()
            .any(|(ordinal, attachment)| usize::from(attachment.ordinal) != ordinal)
        {
            return Err(MailDurablePersistenceError::InvalidAttachmentManifest);
        }
        let legacy_rfc822_sha256 = persisted_request_digest
            .is_none()
            .then(|| {
                legacy_digest
                    .as_slice()
                    .try_into()
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)
            })
            .transpose()?;
        let request_sha256 = persisted_request_digest
            .map(|digest| {
                digest
                    .as_slice()
                    .try_into()
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)
            })
            .transpose()?
            .unwrap_or_else(|| Sha256::digest(&exact_command_bytes).into());
        let rendered_rfc822_sha256 = rendered_digest
            .map(|digest| {
                digest
                    .as_slice()
                    .try_into()
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)
            })
            .transpose()?;
        let queued = MailQueuedDeliveryV1 {
            operation_id,
            connection_id: row
                .try_get("connection_id")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            request_sha256,
            legacy_rfc822_sha256,
            rendered_rfc822_sha256,
            exact_command_bytes,
            attachments,
        };
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        Ok(Some(queued))
    }

    pub async fn record_delivery_rendered_rfc822(
        &self,
        operation_id: &str,
        request_sha256: &[u8; 32],
        rendered_rfc822_sha256: &[u8; 32],
    ) -> Result<(), MailDurablePersistenceError> {
        if operation_id.trim().is_empty()
            || request_sha256.iter().all(|byte| *byte == 0)
            || rendered_rfc822_sha256.iter().all(|byte| *byte == 0)
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query(
            "UPDATE makosh_data.mail_delivery_attempts attempt \
             SET rendered_rfc822_sha256 = $3 \
             WHERE operation_id = $1 AND request_sha256 = $2 AND state = $4 \
               AND (rendered_rfc822_sha256 IS NULL OR rendered_rfc822_sha256 = $3) \
               AND EXISTS (SELECT 1 FROM makosh_data.mail_delivery_queue queue \
                           WHERE queue.operation_id = attempt.operation_id \
                             AND queue.dispatched_at_unix_seconds IS NOT NULL)",
        )
        .bind(operation_id)
        .bind(request_sha256.as_slice())
        .bind(rendered_rfc822_sha256.as_slice())
        .bind(MailSmtpDeliveryAttemptStateV1::Pending as i16)
        .execute(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)
        .and_then(|result| {
            (result.rows_affected() == 1)
                .then_some(())
                .ok_or(MailDurablePersistenceError::InvalidRow)
        })
    }

    pub async fn complete_delivery_accepted(
        &self,
        operation_id: &str,
        rfc822_sha256: &[u8; 32],
        response_code: u16,
        record: &OutboxRecordV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if operation_id.trim().is_empty()
            || !(200..300).contains(&response_code)
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
            "UPDATE makosh_data.mail_delivery_attempts attempt \
             SET state = $3, completed_at_unix_seconds = $4, response_code = $5 \
             WHERE operation_id = $1 AND state = $6 \
               AND (rendered_rfc822_sha256 = $2 \
                    OR (request_sha256 IS NULL AND rfc822_sha256 = $2)) \
               AND EXISTS (SELECT 1 FROM makosh_data.mail_delivery_queue queue \
                           WHERE queue.operation_id = attempt.operation_id \
                             AND queue.dispatched_at_unix_seconds IS NOT NULL)",
        )
        .bind(operation_id)
        .bind(rfc822_sha256.as_slice())
        .bind(MailSmtpDeliveryAttemptStateV1::Accepted as i16)
        .bind(completed_at_unix_seconds)
        .bind(i16::try_from(response_code).map_err(|_| MailDurablePersistenceError::InvalidRow)?)
        .bind(MailSmtpDeliveryAttemptStateV1::Pending as i16)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query("INSERT INTO makosh_data.mail_communications_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
            .bind(record.message_id().as_slice())
            .bind(record.envelope_sha256().as_slice())
            .bind(record.exact_bytes())
            .bind(completed_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn complete_delivery_rejected(
        &self,
        operation_id: &str,
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if operation_id.trim().is_empty() || completed_at_unix_seconds <= 0 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query(
            "UPDATE makosh_data.mail_delivery_attempts attempt \
             SET state = $2, completed_at_unix_seconds = $3 \
             WHERE operation_id = $1 AND state = $4 \
               AND EXISTS (SELECT 1 FROM makosh_data.mail_delivery_queue queue \
                           WHERE queue.operation_id = attempt.operation_id \
                             AND queue.dispatched_at_unix_seconds IS NOT NULL)",
        )
        .bind(operation_id)
        .bind(MailSmtpDeliveryAttemptStateV1::Rejected as i16)
        .bind(completed_at_unix_seconds)
        .bind(MailSmtpDeliveryAttemptStateV1::Pending as i16)
        .execute(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)
        .and_then(|result| {
            (result.rows_affected() == 1)
                .then_some(())
                .ok_or(MailDurablePersistenceError::InvalidRow)
        })
    }

    pub async fn delivery_attempt(
        &self,
        operation_id: &str,
    ) -> Result<Option<MailDeliveryAttemptV1>, MailDurablePersistenceError> {
        if operation_id.trim().is_empty() {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let row = sqlx::query(
            "SELECT attempt.operation_id, attempt.connection_id, attempt.state, \
                    attempt.attempted_at_unix_seconds, attempt.completed_at_unix_seconds, \
                    attempt.response_code, queue.dispatched_at_unix_seconds \
             FROM makosh_data.mail_delivery_attempts attempt \
             JOIN makosh_data.mail_delivery_queue queue ON queue.operation_id = attempt.operation_id \
             WHERE attempt.operation_id = $1",
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        row.map(|row| {
            let state: i16 = row
                .try_get("state")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            let dispatched_at: Option<i64> = row
                .try_get("dispatched_at_unix_seconds")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            let completed_at_unix_seconds: Option<i64> =
                row.try_get("completed_at_unix_seconds")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            let response_code: Option<i16> = row
                .try_get("response_code")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            let outcome = match (
                state,
                dispatched_at,
                completed_at_unix_seconds,
                response_code,
            ) {
                (1, None, None, None) => MailDeliveryAttemptOutcomeV1::Pending,
                (1, Some(_), None, None) => MailDeliveryAttemptOutcomeV1::OutcomeUnknown,
                (2, Some(_), Some(_), Some(code)) if (200..300).contains(&code) => {
                    MailDeliveryAttemptOutcomeV1::Accepted {
                        response_code: u16::try_from(code)
                            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                    }
                }
                (3, Some(_), Some(_), None) => MailDeliveryAttemptOutcomeV1::Rejected,
                _ => return Err(MailDurablePersistenceError::InvalidRow),
            };
            Ok(MailDeliveryAttemptV1 {
                operation_id: row
                    .try_get("operation_id")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                connection_id: row
                    .try_get("connection_id")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                outcome,
                requested_at_unix_seconds: row
                    .try_get("attempted_at_unix_seconds")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                completed_at_unix_seconds,
            })
        })
        .transpose()
    }

    pub async fn pending_communications_outbox(
        &self,
        limit: i64,
    ) -> Result<Vec<OutboxRecordV1>, MailDurablePersistenceError> {
        let rows = sqlx::query("SELECT exact_envelope_bytes FROM makosh_data.mail_communications_outbox WHERE published_at_unix_seconds IS NULL ORDER BY causal_sequence ASC LIMIT $1")
            .bind(limit.clamp(1, 256))
            .fetch_all(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let bytes: Vec<u8> = row
                    .try_get("exact_envelope_bytes")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                OutboxRecordV1::accept(bytes).map_err(|_| MailDurablePersistenceError::InvalidRow)
            })
            .collect()
    }

    pub async fn mark_communications_outbox_published(
        &self,
        message_id: &[u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, MailDurablePersistenceError> {
        sqlx::query("UPDATE makosh_data.mail_communications_outbox SET published_at_unix_seconds = $2 WHERE message_id = $1 AND published_at_unix_seconds IS NULL")
            .bind(message_id.as_slice())
            .bind(published_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() == 1)
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn pending_attachment_security_outbox(
        &self,
        limit: i64,
    ) -> Result<Vec<OutboxRecordV1>, MailDurablePersistenceError> {
        let rows = sqlx::query("SELECT exact_envelope_bytes FROM makosh_data.mail_attachment_security_outbox WHERE published_at_unix_seconds IS NULL ORDER BY created_at_unix_seconds ASC, message_id ASC LIMIT $1")
            .bind(limit.clamp(1, 256))
            .fetch_all(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let bytes: Vec<u8> = row
                    .try_get("exact_envelope_bytes")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                OutboxRecordV1::accept(bytes).map_err(|_| MailDurablePersistenceError::InvalidRow)
            })
            .collect()
    }

    pub async fn mark_attachment_security_outbox_published(
        &self,
        message_id: &[u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, MailDurablePersistenceError> {
        sqlx::query("UPDATE makosh_data.mail_attachment_security_outbox SET published_at_unix_seconds = $2 WHERE message_id = $1 AND published_at_unix_seconds IS NULL")
            .bind(message_id.as_slice())
            .bind(published_at_unix_seconds)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() == 1)
            .map_err(|_| MailDurablePersistenceError::Database)
    }
}

pub(crate) async fn insert_communications_outbox(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    record: &OutboxRecordV1,
    created_at_unix_seconds: i64,
) -> Result<(), MailDurablePersistenceError> {
    sqlx::query("INSERT INTO makosh_data.mail_communications_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
        .bind(record.message_id().as_slice())
        .bind(record.envelope_sha256().as_slice())
        .bind(record.exact_bytes())
        .bind(created_at_unix_seconds)
        .execute(&mut **transaction)
        .await
        .map(|_| ())
        .map_err(|_| MailDurablePersistenceError::Database)
}

async fn insert_attachment_security_outbox(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    record: &OutboxRecordV1,
    created_at_unix_seconds: i64,
) -> Result<(), MailDurablePersistenceError> {
    sqlx::query("INSERT INTO makosh_data.mail_attachment_security_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING")
        .bind(record.message_id().as_slice())
        .bind(record.envelope_sha256().as_slice())
        .bind(record.exact_bytes())
        .bind(created_at_unix_seconds)
        .execute(&mut **transaction)
        .await
        .map(|_| ())
        .map_err(|_| MailDurablePersistenceError::Database)
}

async fn insert_attachment_materialization(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    materialization: &MailAttachmentMaterializationV1,
    materialized_at_unix_seconds: i64,
) -> Result<(), MailDurablePersistenceError> {
    if !valid_attachment_materialization(materialization) || materialized_at_unix_seconds <= 0 {
        return Err(MailDurablePersistenceError::InvalidAttachmentAdmissionState);
    }
    sqlx::query(
        "INSERT INTO makosh_data.mail_attachment_materializations \
         (attachment_anchor_id, source_observation_id, blob_reference_id, receipt_sha256, \
          declared_size, filename, media_type, disposition, materialized_at_unix_seconds) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(materialization.attachment_anchor_id.as_slice())
    .bind(materialization.source_observation_id.as_slice())
    .bind(materialization.blob_reference_id.as_slice())
    .bind(materialization.receipt_sha256.as_slice())
    .bind(
        i64::try_from(materialization.declared_size)
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
    )
    .bind(&materialization.filename)
    .bind(&materialization.media_type)
    .bind(materialization.disposition as i16)
    .bind(materialized_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| MailDurablePersistenceError::ConflictingAttachmentMaterialization)
}

fn valid_attachment_materialization(materialization: &MailAttachmentMaterializationV1) -> bool {
    !materialization
        .source_observation_id
        .iter()
        .all(|byte| *byte == 0)
        && !materialization
            .attachment_anchor_id
            .iter()
            .all(|byte| *byte == 0)
        && !materialization
            .blob_reference_id
            .iter()
            .all(|byte| *byte == 0)
        && !materialization.receipt_sha256.iter().all(|byte| *byte == 0)
        && (1..=16 * 1024 * 1024).contains(&materialization.declared_size)
        && materialization.filename.as_deref().is_none_or(|filename| {
            !filename.is_empty() && filename.len() <= 512 && !filename.contains(['\r', '\n', '\0'])
        })
        && valid_media_type(&materialization.media_type)
}

fn valid_media_type(value: &str) -> bool {
    value.is_ascii()
        && (3..=256).contains(&value.len())
        && !value.contains(char::is_whitespace)
        && !value.contains(['\r', '\n', '\0', '"', ';'])
        && value
            .split_once('/')
            .is_some_and(|(kind, subtype)| !kind.is_empty() && !subtype.is_empty())
}

fn valid_attachment_safety_transition(transition: MailAttachmentSafetyTransitionV1) -> bool {
    !transition
        .attachment_anchor_id
        .iter()
        .all(|byte| *byte == 0)
        && !transition.evidence_id.iter().all(|byte| *byte == 0)
        && (-62_135_596_800..=253_402_300_799).contains(&transition.observed_at_unix_seconds)
        && matches!(
            (transition.expected_state, transition.next_state),
            (
                MailAttachmentSafetyStateV1::DescriptorOnly,
                MailAttachmentSafetyStateV1::BlobPending
            ) | (
                MailAttachmentSafetyStateV1::BlobPending,
                MailAttachmentSafetyStateV1::BlobAdmitted | MailAttachmentSafetyStateV1::Rejected
            ) | (
                MailAttachmentSafetyStateV1::BlobAdmitted,
                MailAttachmentSafetyStateV1::SafeForDelivery
                    | MailAttachmentSafetyStateV1::Quarantined
                    | MailAttachmentSafetyStateV1::Rejected
            )
        )
}

fn delivery_attachment_manifest_from_row(
    row: PgRow,
) -> Result<MailDeliveryAttachmentManifestV1, MailDurablePersistenceError> {
    let ordinal: i16 = row
        .try_get("ordinal")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let attachment_anchor_id: Vec<u8> = row
        .try_get("attachment_anchor_id")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let blob_reference_id: Vec<u8> = row
        .try_get("blob_reference_id")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let receipt_sha256: Vec<u8> = row
        .try_get("receipt_sha256")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let safety_evidence_id: Vec<u8> = row
        .try_get("safety_evidence_id")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let declared_size: i64 = row
        .try_get("declared_size")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let disposition: i16 = row
        .try_get("disposition")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let manifest = MailDeliveryAttachmentManifestV1 {
        ordinal: u8::try_from(ordinal).map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        attachment_anchor_id: attachment_anchor_id
            .as_slice()
            .try_into()
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        blob_reference_id: blob_reference_id
            .as_slice()
            .try_into()
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        receipt_sha256: receipt_sha256
            .as_slice()
            .try_into()
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        declared_size: u64::try_from(declared_size)
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        filename: row
            .try_get("filename")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        media_type: row
            .try_get("media_type")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        disposition: match disposition {
            1 => MailAttachmentDispositionV1::Attachment,
            2 => MailAttachmentDispositionV1::Inline,
            _ => return Err(MailDurablePersistenceError::InvalidRow),
        },
        safety_evidence_id: safety_evidence_id
            .as_slice()
            .try_into()
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
    };
    if manifest.attachment_anchor_id.iter().all(|byte| *byte == 0)
        || manifest.blob_reference_id.iter().all(|byte| *byte == 0)
        || manifest.receipt_sha256.iter().all(|byte| *byte == 0)
        || manifest.safety_evidence_id.iter().all(|byte| *byte == 0)
        || !(1..=16 * 1024 * 1024).contains(&manifest.declared_size)
        || !valid_media_type(&manifest.media_type)
    {
        return Err(MailDurablePersistenceError::InvalidAttachmentManifest);
    }
    Ok(manifest)
}
