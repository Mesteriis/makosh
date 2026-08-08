use makosh_mail_api::GmailOAuthAuthorityV1;
use sqlx::Row;
use zeroize::Zeroizing;

use crate::{MailDurablePersistence, MailDurablePersistenceError};

pub const MAIL_SCHEMA_V4: &str = r#"
ALTER TABLE makosh_data.mail_gmail_oauth_credential_bindings
    ADD COLUMN IF NOT EXISTS access_token_expires_at_unix_seconds BIGINT;
ALTER TABLE makosh_data.mail_gmail_oauth_credential_bindings
    ADD COLUMN IF NOT EXISTS scope_sha256 BYTEA;
CREATE TABLE IF NOT EXISTS makosh_data.mail_gmail_oauth_attempts (
    setup_id TEXT PRIMARY KEY,
    operation_id TEXT NOT NULL UNIQUE,
    connection_id TEXT NOT NULL,
    state_sha256 BYTEA NOT NULL,
    authorization_url TEXT NOT NULL,
    code_verifier TEXT NOT NULL,
    settings_revision BIGINT NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    expires_at_unix_seconds BIGINT NOT NULL,
    consumed_by_operation_id TEXT UNIQUE,
    CHECK (setup_id <> ''),
    CHECK (operation_id <> ''),
    CHECK (connection_id <> ''),
    CHECK (octet_length(state_sha256) = 32),
    CHECK (authorization_url <> ''),
    CHECK (code_verifier <> ''),
    CHECK (settings_revision > 0),
    CHECK (created_at_unix_seconds > 0),
    CHECK (expires_at_unix_seconds > created_at_unix_seconds)
);
CREATE INDEX IF NOT EXISTS mail_gmail_oauth_attempts_expiry_idx
    ON makosh_data.mail_gmail_oauth_attempts (expires_at_unix_seconds, setup_id)
    WHERE consumed_by_operation_id IS NULL;
CREATE TABLE IF NOT EXISTS makosh_data.mail_gmail_oauth_operations (
    operation_id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    kind SMALLINT NOT NULL,
    setup_id TEXT,
    authorization_code TEXT,
    authorization_code_sha256 BYTEA,
    state SMALLINT NOT NULL,
    requested_at_unix_seconds BIGINT NOT NULL,
    dispatched_at_unix_seconds BIGINT,
    completed_at_unix_seconds BIGINT,
    access_token_record_id BYTEA,
    access_token_revision BIGINT,
    refresh_credential_record_id BYTEA,
    refresh_credential_revision BIGINT,
    CHECK (operation_id <> ''),
    CHECK (connection_id <> ''),
    CHECK (kind IN (1, 2)),
    CHECK (state IN (1, 2, 3)),
    CHECK (requested_at_unix_seconds > 0),
    CHECK ((kind = 1 AND setup_id IS NOT NULL AND authorization_code_sha256 IS NOT NULL
            AND octet_length(authorization_code_sha256) = 32)
        OR (kind = 2 AND setup_id IS NULL AND authorization_code IS NULL
            AND authorization_code_sha256 IS NULL)),
    CHECK (access_token_record_id IS NULL OR octet_length(access_token_record_id) = 16),
    CHECK (refresh_credential_record_id IS NULL
        OR octet_length(refresh_credential_record_id) = 16),
    CHECK ((state = 1 AND completed_at_unix_seconds IS NULL)
        OR (state IN (2, 3) AND completed_at_unix_seconds IS NOT NULL))
);
CREATE INDEX IF NOT EXISTS mail_gmail_oauth_operations_pending_idx
    ON makosh_data.mail_gmail_oauth_operations (requested_at_unix_seconds, operation_id)
    WHERE state = 1 AND dispatched_at_unix_seconds IS NULL;
"#;

pub const MAIL_SCHEMA_V16: &str = r#"
ALTER TABLE makosh_data.mail_gmail_oauth_attempts
    ADD COLUMN IF NOT EXISTS authority SMALLINT NOT NULL DEFAULT 1
        CHECK (authority IN (1, 2));
ALTER TABLE makosh_data.mail_gmail_oauth_credential_bindings
    ADD COLUMN IF NOT EXISTS permanent_delete_authorized BOOLEAN NOT NULL DEFAULT FALSE;
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthCredentialBindingV1 {
    pub access_token_record_id: [u8; 16],
    pub access_token_revision: u64,
    pub refresh_credential_record_id: [u8; 16],
    pub refresh_credential_revision: u64,
    pub access_token_expires_at_unix_seconds: i64,
    pub scope_sha256: [u8; 32],
    pub permanent_delete_authorized: bool,
    pub contacts_write_authorized: bool,
}

#[derive(Clone, Eq, PartialEq)]
pub struct GmailOAuthAttemptStartV1 {
    pub operation_id: String,
    pub setup_id: String,
    pub connection_id: String,
    pub state_sha256: [u8; 32],
    pub authorization_url: String,
    pub code_verifier: String,
    pub settings_revision: u64,
    pub created_at_unix_seconds: i64,
    pub expires_at_unix_seconds: i64,
    pub authority: GmailOAuthAuthorityV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthStoredAttemptV1 {
    pub operation_id: String,
    pub setup_id: String,
    pub authorization_url: String,
    pub expires_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GmailOAuthEnqueueOutcomeV1 {
    Enqueued,
    Existing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GmailOAuthOperationKindV1 {
    Complete,
    Refresh,
}

pub struct GmailOAuthQueuedOperationV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub kind: GmailOAuthOperationKindV1,
    pub authorization_code: Option<Zeroizing<String>>,
    pub code_verifier: Option<Zeroizing<String>>,
    pub authority: Option<GmailOAuthAuthorityV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GmailOAuthOperationOutcomeV1 {
    Pending,
    Completed,
    Rejected,
    OutcomeUnknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthOperationV1 {
    pub operation_id: String,
    pub kind: GmailOAuthOperationKindV1,
    pub outcome: GmailOAuthOperationOutcomeV1,
    pub requested_at_unix_seconds: i64,
    pub completed_at_unix_seconds: Option<i64>,
}

impl MailDurablePersistence {
    pub async fn start_gmail_oauth_attempt(
        &self,
        attempt: &GmailOAuthAttemptStartV1,
    ) -> Result<GmailOAuthStoredAttemptV1, MailDurablePersistenceError> {
        validate_attempt(attempt)?;
        sqlx::query(
            "INSERT INTO makosh_data.mail_gmail_oauth_attempts \
             (setup_id, operation_id, connection_id, state_sha256, authorization_url, \
              code_verifier, settings_revision, created_at_unix_seconds, expires_at_unix_seconds, \
              authority) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (operation_id) DO NOTHING",
        )
        .bind(&attempt.setup_id)
        .bind(&attempt.operation_id)
        .bind(&attempt.connection_id)
        .bind(attempt.state_sha256.as_slice())
        .bind(&attempt.authorization_url)
        .bind(&attempt.code_verifier)
        .bind(
            i64::try_from(attempt.settings_revision)
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        )
        .bind(attempt.created_at_unix_seconds)
        .bind(attempt.expires_at_unix_seconds)
        .bind(encode_authority(attempt.authority))
        .execute(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        self.gmail_oauth_attempt_by_operation(
            &attempt.operation_id,
            &attempt.connection_id,
            attempt.settings_revision,
            attempt.authority,
        )
        .await?
        .ok_or(MailDurablePersistenceError::InvalidRow)
    }

    pub async fn enqueue_gmail_oauth_complete(
        &self,
        operation_id: &str,
        setup_id: &str,
        submitted_state_sha256: &[u8; 32],
        authorization_code: &str,
        authorization_code_sha256: &[u8; 32],
        requested_at_unix_seconds: i64,
    ) -> Result<GmailOAuthEnqueueOutcomeV1, MailDurablePersistenceError> {
        if !valid_identifier(operation_id)
            || !valid_identifier(setup_id)
            || !valid_secret_carrier(authorization_code)
            || submitted_state_sha256.iter().all(|byte| *byte == 0)
            || authorization_code_sha256.iter().all(|byte| *byte == 0)
            || requested_at_unix_seconds <= 0
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let existing = sqlx::query(
            "SELECT kind, setup_id, authorization_code_sha256 \
             FROM makosh_data.mail_gmail_oauth_operations WHERE operation_id = $1",
        )
        .bind(operation_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if let Some(row) = existing {
            let kind: i16 = row
                .try_get("kind")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            let persisted_setup: Option<String> = row
                .try_get("setup_id")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            let persisted_code_sha256: Option<Vec<u8>> =
                row.try_get("authorization_code_sha256")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
            if kind != 1
                || persisted_setup.as_deref() != Some(setup_id)
                || persisted_code_sha256.as_deref() != Some(authorization_code_sha256.as_slice())
            {
                return Err(MailDurablePersistenceError::InvalidRow);
            }
            transaction
                .commit()
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
            return Ok(GmailOAuthEnqueueOutcomeV1::Existing);
        }
        let attempt = sqlx::query(
            "SELECT connection_id, state_sha256, expires_at_unix_seconds, \
                    consumed_by_operation_id \
             FROM makosh_data.mail_gmail_oauth_attempts \
             WHERE setup_id = $1 FOR UPDATE",
        )
        .bind(setup_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?
        .ok_or(MailDurablePersistenceError::InvalidRow)?;
        let state_sha256: Vec<u8> = attempt
            .try_get("state_sha256")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        let expires_at: i64 = attempt
            .try_get("expires_at_unix_seconds")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        let consumed: Option<String> = attempt
            .try_get("consumed_by_operation_id")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        if state_sha256.as_slice() != submitted_state_sha256
            || expires_at < requested_at_unix_seconds
            || consumed.is_some()
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let connection_id: String = attempt
            .try_get("connection_id")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        sqlx::query(
            "INSERT INTO makosh_data.mail_gmail_oauth_operations \
             (operation_id, connection_id, kind, setup_id, authorization_code, \
              authorization_code_sha256, state, requested_at_unix_seconds) \
             VALUES ($1, $2, 1, $3, $4, $5, 1, $6)",
        )
        .bind(operation_id)
        .bind(connection_id)
        .bind(setup_id)
        .bind(authorization_code)
        .bind(authorization_code_sha256.as_slice())
        .bind(requested_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        sqlx::query(
            "UPDATE makosh_data.mail_gmail_oauth_attempts \
             SET consumed_by_operation_id = $2 \
             WHERE setup_id = $1 AND consumed_by_operation_id IS NULL",
        )
        .bind(setup_id)
        .bind(operation_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        Ok(GmailOAuthEnqueueOutcomeV1::Enqueued)
    }

    pub async fn enqueue_gmail_oauth_refresh(
        &self,
        operation_id: &str,
        connection_id: &str,
        requested_at_unix_seconds: i64,
    ) -> Result<GmailOAuthEnqueueOutcomeV1, MailDurablePersistenceError> {
        if !valid_identifier(operation_id)
            || !valid_identifier(connection_id)
            || requested_at_unix_seconds <= 0
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        if self
            .gmail_oauth_credential_binding(connection_id)
            .await?
            .is_none()
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.mail_gmail_oauth_operations \
             (operation_id, connection_id, kind, state, requested_at_unix_seconds) \
             VALUES ($1, $2, 2, 1, $3) ON CONFLICT (operation_id) DO NOTHING",
        )
        .bind(operation_id)
        .bind(connection_id)
        .bind(requested_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if inserted.rows_affected() == 1 {
            return Ok(GmailOAuthEnqueueOutcomeV1::Enqueued);
        }
        let row = sqlx::query(
            "SELECT kind, connection_id FROM makosh_data.mail_gmail_oauth_operations \
             WHERE operation_id = $1",
        )
        .bind(operation_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        let kind: i16 = row
            .try_get("kind")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        let persisted_connection: String = row
            .try_get("connection_id")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        if kind != 2 || persisted_connection != connection_id {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        Ok(GmailOAuthEnqueueOutcomeV1::Existing)
    }

    pub async fn claim_next_gmail_oauth_operation(
        &self,
        connection_id: &str,
        dispatched_at_unix_seconds: i64,
    ) -> Result<Option<GmailOAuthQueuedOperationV1>, MailDurablePersistenceError> {
        if !valid_identifier(connection_id) || dispatched_at_unix_seconds <= 0 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let row = sqlx::query(
            "SELECT operation.operation_id, operation.connection_id, operation.kind, \
                    operation.authorization_code, attempt.code_verifier, attempt.authority \
             FROM makosh_data.mail_gmail_oauth_operations operation \
             LEFT JOIN makosh_data.mail_gmail_oauth_attempts attempt \
                ON attempt.setup_id = operation.setup_id \
             WHERE operation.state = 1 AND operation.dispatched_at_unix_seconds IS NULL \
               AND operation.connection_id = $1 \
             ORDER BY operation.requested_at_unix_seconds, operation.operation_id \
             LIMIT 1 FOR UPDATE OF operation SKIP LOCKED",
        )
        .bind(connection_id)
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
        let kind: i16 = row
            .try_get("kind")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        let authorization_code: Option<String> = row
            .try_get("authorization_code")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        let code_verifier: Option<String> = row
            .try_get("code_verifier")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        let authority = row
            .try_get::<Option<i16>, _>("authority")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?
            .map(decode_authority)
            .transpose()?;
        let kind = match (kind, authorization_code.as_ref(), code_verifier.as_ref()) {
            (1, Some(_), Some(_)) if authority.is_some() => GmailOAuthOperationKindV1::Complete,
            (2, None, None) if authority.is_none() => GmailOAuthOperationKindV1::Refresh,
            _ => return Err(MailDurablePersistenceError::InvalidRow),
        };
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_gmail_oauth_operations \
             SET dispatched_at_unix_seconds = $2, authorization_code = NULL \
             WHERE operation_id = $1 AND dispatched_at_unix_seconds IS NULL AND state = 1",
        )
        .bind(&operation_id)
        .bind(dispatched_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        Ok(Some(GmailOAuthQueuedOperationV1 {
            operation_id,
            connection_id: row
                .try_get("connection_id")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            kind,
            authorization_code: authorization_code.map(Zeroizing::new),
            code_verifier: code_verifier.map(Zeroizing::new),
            authority,
        }))
    }

    pub async fn checkpoint_gmail_oauth_access_record(
        &self,
        operation_id: &str,
        record_id: &[u8; 16],
        revision: u64,
    ) -> Result<(), MailDurablePersistenceError> {
        checkpoint_record(
            &self.pool,
            operation_id,
            CredentialRecordKind::AccessToken,
            record_id,
            revision,
        )
        .await
    }

    pub async fn checkpoint_gmail_oauth_refresh_record(
        &self,
        operation_id: &str,
        record_id: &[u8; 16],
        revision: u64,
    ) -> Result<(), MailDurablePersistenceError> {
        checkpoint_record(
            &self.pool,
            operation_id,
            CredentialRecordKind::RefreshCredential,
            record_id,
            revision,
        )
        .await
    }

    pub async fn complete_gmail_oauth_operation(
        &self,
        operation_id: &str,
        binding: &GmailOAuthCredentialBindingV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        validate_binding(binding)?;
        if !valid_identifier(operation_id) || completed_at_unix_seconds <= 0 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let connection_id: String = sqlx::query(
            "SELECT connection_id FROM makosh_data.mail_gmail_oauth_operations \
             WHERE operation_id = $1 AND state = 1 AND dispatched_at_unix_seconds IS NOT NULL \
             FOR UPDATE",
        )
        .bind(operation_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?
        .ok_or(MailDurablePersistenceError::InvalidRow)?
        .try_get("connection_id")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        upsert_binding(
            &mut transaction,
            &connection_id,
            binding,
            completed_at_unix_seconds,
        )
        .await?;
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_gmail_oauth_operations \
             SET state = 2, completed_at_unix_seconds = $2 \
             WHERE operation_id = $1 AND state = 1 AND dispatched_at_unix_seconds IS NOT NULL",
        )
        .bind(operation_id)
        .bind(completed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    pub async fn reject_gmail_oauth_operation(
        &self,
        operation_id: &str,
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if !valid_identifier(operation_id) || completed_at_unix_seconds <= 0 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query(
            "UPDATE makosh_data.mail_gmail_oauth_operations \
             SET state = 3, completed_at_unix_seconds = $2 \
             WHERE operation_id = $1 AND state = 1 AND dispatched_at_unix_seconds IS NOT NULL",
        )
        .bind(operation_id)
        .bind(completed_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)
        .and_then(|result| {
            (result.rows_affected() == 1)
                .then_some(())
                .ok_or(MailDurablePersistenceError::InvalidRow)
        })
    }

    pub async fn gmail_oauth_operation(
        &self,
        operation_id: &str,
    ) -> Result<Option<GmailOAuthOperationV1>, MailDurablePersistenceError> {
        if !valid_identifier(operation_id) {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query(
            "SELECT operation_id, kind, state, requested_at_unix_seconds, \
                    dispatched_at_unix_seconds, completed_at_unix_seconds \
             FROM makosh_data.mail_gmail_oauth_operations WHERE operation_id = $1",
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?
        .map(decode_operation)
        .transpose()
    }

    pub async fn gmail_oauth_credential_binding(
        &self,
        connection_id: &str,
    ) -> Result<Option<GmailOAuthCredentialBindingV1>, MailDurablePersistenceError> {
        if !valid_identifier(connection_id) {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query(
            "SELECT access_token_record_id, access_token_revision, \
                    refresh_credential_record_id, refresh_credential_revision, \
                    access_token_expires_at_unix_seconds, scope_sha256 \
                    , permanent_delete_authorized, contacts_write_authorized \
             FROM makosh_data.mail_gmail_oauth_credential_bindings WHERE connection_id = $1",
        )
        .bind(connection_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?
        .map(decode_binding)
        .transpose()
    }

    pub async fn store_gmail_oauth_credential_binding(
        &self,
        connection_id: &str,
        binding: &GmailOAuthCredentialBindingV1,
        updated_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if !valid_identifier(connection_id) || updated_at_unix_seconds <= 0 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        validate_binding(binding)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        upsert_binding(
            &mut transaction,
            connection_id,
            binding,
            updated_at_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)
    }

    async fn gmail_oauth_attempt_by_operation(
        &self,
        operation_id: &str,
        connection_id: &str,
        settings_revision: u64,
        authority: GmailOAuthAuthorityV1,
    ) -> Result<Option<GmailOAuthStoredAttemptV1>, MailDurablePersistenceError> {
        let settings_revision = i64::try_from(settings_revision)
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        sqlx::query(
            "SELECT operation_id, setup_id, authorization_url, expires_at_unix_seconds, authority \
             FROM makosh_data.mail_gmail_oauth_attempts \
             WHERE operation_id = $1 AND connection_id = $2 AND settings_revision = $3 \
               AND authority = $4",
        )
        .bind(operation_id)
        .bind(connection_id)
        .bind(settings_revision)
        .bind(encode_authority(authority))
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?
        .map(|row| {
            Ok(GmailOAuthStoredAttemptV1 {
                operation_id: row
                    .try_get("operation_id")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                setup_id: row
                    .try_get("setup_id")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                authorization_url: row
                    .try_get("authorization_url")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                expires_at_unix_seconds: row
                    .try_get("expires_at_unix_seconds")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            })
        })
        .transpose()
    }
}

fn validate_attempt(attempt: &GmailOAuthAttemptStartV1) -> Result<(), MailDurablePersistenceError> {
    if !valid_identifier(&attempt.operation_id)
        || !valid_identifier(&attempt.setup_id)
        || !valid_identifier(&attempt.connection_id)
        || attempt.state_sha256.iter().all(|byte| *byte == 0)
        || !attempt.authorization_url.starts_with("https://")
        || attempt.authorization_url.len() > 16 * 1024
        || !valid_secret_carrier(&attempt.code_verifier)
        || attempt.settings_revision == 0
        || attempt.created_at_unix_seconds <= 0
        || attempt.expires_at_unix_seconds <= attempt.created_at_unix_seconds
    {
        return Err(MailDurablePersistenceError::InvalidRow);
    }
    Ok(())
}

fn validate_binding(
    binding: &GmailOAuthCredentialBindingV1,
) -> Result<(), MailDurablePersistenceError> {
    if binding.access_token_record_id.iter().all(|byte| *byte == 0)
        || binding.access_token_revision == 0
        || binding
            .refresh_credential_record_id
            .iter()
            .all(|byte| *byte == 0)
        || binding.refresh_credential_revision == 0
        || binding.access_token_expires_at_unix_seconds <= 0
        || binding.scope_sha256.iter().all(|byte| *byte == 0)
    {
        return Err(MailDurablePersistenceError::InvalidRow);
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.is_ascii()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_secret_carrier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 8 * 1024
        && value.is_ascii()
        && !value.contains(['\r', '\n', '\0'])
}

#[derive(Clone, Copy)]
enum CredentialRecordKind {
    AccessToken,
    RefreshCredential,
}

async fn checkpoint_record(
    pool: &sqlx::PgPool,
    operation_id: &str,
    kind: CredentialRecordKind,
    record_id: &[u8; 16],
    revision: u64,
) -> Result<(), MailDurablePersistenceError> {
    if !valid_identifier(operation_id) || record_id.iter().all(|byte| *byte == 0) || revision == 0 {
        return Err(MailDurablePersistenceError::InvalidRow);
    }
    let revision = i64::try_from(revision).map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let result = match kind {
        CredentialRecordKind::AccessToken => {
            sqlx::query(
                "UPDATE makosh_data.mail_gmail_oauth_operations \
                 SET access_token_record_id = $2, access_token_revision = $3 \
                 WHERE operation_id = $1 AND state = 1 \
                    AND dispatched_at_unix_seconds IS NOT NULL",
            )
            .bind(operation_id)
            .bind(record_id.as_slice())
            .bind(revision)
            .execute(pool)
            .await
        }
        CredentialRecordKind::RefreshCredential => {
            sqlx::query(
                "UPDATE makosh_data.mail_gmail_oauth_operations \
                 SET refresh_credential_record_id = $2, refresh_credential_revision = $3 \
                 WHERE operation_id = $1 AND state = 1 \
                    AND dispatched_at_unix_seconds IS NOT NULL",
            )
            .bind(operation_id)
            .bind(record_id.as_slice())
            .bind(revision)
            .execute(pool)
            .await
        }
    }
    .map_err(|_| MailDurablePersistenceError::Database)?;
    (result.rows_affected() == 1)
        .then_some(())
        .ok_or(MailDurablePersistenceError::InvalidRow)
}

async fn upsert_binding(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    connection_id: &str,
    binding: &GmailOAuthCredentialBindingV1,
    updated_at_unix_seconds: i64,
) -> Result<(), MailDurablePersistenceError> {
    sqlx::query(
        "INSERT INTO makosh_data.mail_gmail_oauth_credential_bindings \
         (connection_id, access_token_record_id, access_token_revision, \
          refresh_credential_record_id, refresh_credential_revision, \
          access_token_expires_at_unix_seconds, scope_sha256, \
          permanent_delete_authorized, contacts_write_authorized, updated_at_unix_seconds) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
         ON CONFLICT (connection_id) DO UPDATE SET \
          access_token_record_id = EXCLUDED.access_token_record_id, \
          access_token_revision = EXCLUDED.access_token_revision, \
          refresh_credential_record_id = EXCLUDED.refresh_credential_record_id, \
          refresh_credential_revision = EXCLUDED.refresh_credential_revision, \
          access_token_expires_at_unix_seconds = EXCLUDED.access_token_expires_at_unix_seconds, \
          scope_sha256 = EXCLUDED.scope_sha256, \
          permanent_delete_authorized = EXCLUDED.permanent_delete_authorized, \
          contacts_write_authorized = EXCLUDED.contacts_write_authorized, \
          updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds",
    )
    .bind(connection_id)
    .bind(binding.access_token_record_id.as_slice())
    .bind(
        i64::try_from(binding.access_token_revision)
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
    )
    .bind(binding.refresh_credential_record_id.as_slice())
    .bind(
        i64::try_from(binding.refresh_credential_revision)
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
    )
    .bind(binding.access_token_expires_at_unix_seconds)
    .bind(binding.scope_sha256.as_slice())
    .bind(binding.permanent_delete_authorized)
    .bind(binding.contacts_write_authorized)
    .bind(updated_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| MailDurablePersistenceError::Database)
}

fn decode_binding(
    row: sqlx::postgres::PgRow,
) -> Result<GmailOAuthCredentialBindingV1, MailDurablePersistenceError> {
    let access: Vec<u8> = row
        .try_get("access_token_record_id")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let refresh: Vec<u8> = row
        .try_get("refresh_credential_record_id")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let scope: Vec<u8> = row
        .try_get("scope_sha256")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let binding =
        GmailOAuthCredentialBindingV1 {
            access_token_record_id: access
                .as_slice()
                .try_into()
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            access_token_revision: u64::try_from(
                row.try_get::<i64, _>("access_token_revision")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            )
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            refresh_credential_record_id: refresh
                .as_slice()
                .try_into()
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            refresh_credential_revision: u64::try_from(
                row.try_get::<i64, _>("refresh_credential_revision")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            )
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            access_token_expires_at_unix_seconds: row
                .try_get("access_token_expires_at_unix_seconds")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            scope_sha256: scope
                .as_slice()
                .try_into()
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            permanent_delete_authorized: row
                .try_get("permanent_delete_authorized")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            contacts_write_authorized: row
                .try_get("contacts_write_authorized")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        };
    validate_binding(&binding)?;
    Ok(binding)
}

fn encode_authority(authority: GmailOAuthAuthorityV1) -> i16 {
    match authority {
        GmailOAuthAuthorityV1::Operational => 1,
        GmailOAuthAuthorityV1::PermanentDelete => 2,
    }
}

fn decode_authority(authority: i16) -> Result<GmailOAuthAuthorityV1, MailDurablePersistenceError> {
    match authority {
        1 => Ok(GmailOAuthAuthorityV1::Operational),
        2 => Ok(GmailOAuthAuthorityV1::PermanentDelete),
        _ => Err(MailDurablePersistenceError::InvalidRow),
    }
}

fn decode_operation(
    row: sqlx::postgres::PgRow,
) -> Result<GmailOAuthOperationV1, MailDurablePersistenceError> {
    let kind = match row
        .try_get::<i16, _>("kind")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?
    {
        1 => GmailOAuthOperationKindV1::Complete,
        2 => GmailOAuthOperationKindV1::Refresh,
        _ => return Err(MailDurablePersistenceError::InvalidRow),
    };
    let state: i16 = row
        .try_get("state")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let dispatched: Option<i64> = row
        .try_get("dispatched_at_unix_seconds")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let completed_at_unix_seconds: Option<i64> = row
        .try_get("completed_at_unix_seconds")
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
    let outcome = match (state, dispatched, completed_at_unix_seconds) {
        (1, None, None) => GmailOAuthOperationOutcomeV1::Pending,
        (1, Some(_), None) => GmailOAuthOperationOutcomeV1::OutcomeUnknown,
        (2, Some(_), Some(_)) => GmailOAuthOperationOutcomeV1::Completed,
        (3, Some(_), Some(_)) => GmailOAuthOperationOutcomeV1::Rejected,
        _ => return Err(MailDurablePersistenceError::InvalidRow),
    };
    Ok(GmailOAuthOperationV1 {
        operation_id: row
            .try_get("operation_id")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        kind,
        outcome,
        requested_at_unix_seconds: row
            .try_get("requested_at_unix_seconds")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        completed_at_unix_seconds,
    })
}
