//! Durable Mail message-flag command journal and projection reconciliation.

use makosh_mail_api::{
    message_flags::{
        MailMessageFlagAcceptedV1, MailMessageFlagCommandV1, MailMessageFlagKindV1,
        MailMessageFlagOperationOutcomeV1, MailMessageFlagOperationStatusV1,
        MailMessageFlagStatusRequestV1, validate_message_flag_command,
        validate_message_flag_status, validate_message_flag_status_request,
    },
    message_flags_wire::{decode_message_flag_command, encode_message_flag_command},
};
use sha2::{Digest, Sha256};
use sqlx::{Row, postgres::PgRow};

use crate::MailDurablePersistence;

pub const MAIL_SCHEMA_V12: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.mail_message_flag_operations (
    operation_id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    provider_message_id TEXT NOT NULL,
    flag_kind SMALLINT NOT NULL CHECK (flag_kind BETWEEN 1 AND 2),
    target_value BOOLEAN NOT NULL,
    request_sha256 BYTEA NOT NULL CHECK (octet_length(request_sha256) = 32),
    exact_command_bytes BYTEA NOT NULL
        CHECK (octet_length(exact_command_bytes) BETWEEN 1 AND 4096),
    outcome SMALLINT NOT NULL DEFAULT 1 CHECK (outcome BETWEEN 1 AND 4),
    requested_at_unix_seconds BIGINT NOT NULL CHECK (requested_at_unix_seconds > 0),
    completed_at_unix_seconds BIGINT,
    projection_revision BIGINT CHECK (projection_revision > 0)
);

CREATE INDEX IF NOT EXISTS mail_message_flag_operations_pending_idx
ON makosh_data.mail_message_flag_operations (
    connection_id,
    outcome,
    requested_at_unix_seconds,
    operation_id
);
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessageFlagPersistenceErrorV1 {
    Database,
    InvalidInput,
    ConflictingOperation,
    MissingMessage,
    InvalidRow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailQueuedMessageFlagCommandV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub message_id: String,
    pub kind: MailMessageFlagKindV1,
    pub target_value: bool,
    pub request_sha256: [u8; 32],
    pub exact_command_bytes: Vec<u8>,
}

impl MailDurablePersistence {
    pub async fn enqueue_message_flag_command(
        &self,
        command: &MailMessageFlagCommandV1,
        canonical_command_bytes: &[u8],
        requested_at_unix_seconds: i64,
    ) -> Result<MailMessageFlagAcceptedV1, MailMessageFlagPersistenceErrorV1> {
        if requested_at_unix_seconds <= 0
            || validate_message_flag_command(command).is_err()
            || canonical_command_bytes.is_empty()
            || canonical_command_bytes.len() > 4_096
            || encode_message_flag_command(command)
                .map(|bytes| bytes != canonical_command_bytes)
                .unwrap_or(true)
        {
            return Err(MailMessageFlagPersistenceErrorV1::InvalidInput);
        }
        let request_sha256: [u8; 32] = Sha256::digest(canonical_command_bytes).into();
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?;
        let existing = sqlx::query(
            "SELECT request_sha256 FROM makosh_data.mail_message_flag_operations \
             WHERE operation_id = $1",
        )
        .bind(&command.operation_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?;
        if let Some(row) = existing {
            let existing_hash = row
                .try_get::<Vec<u8>, _>("request_sha256")
                .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?;
            if existing_hash.as_slice() != request_sha256 {
                return Err(MailMessageFlagPersistenceErrorV1::ConflictingOperation);
            }
            transaction
                .commit()
                .await
                .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?;
            return Ok(MailMessageFlagAcceptedV1 {
                operation_id: command.operation_id.clone(),
            });
        }
        let message_exists = sqlx::query(
            "SELECT 1 FROM makosh_data.mail_operational_messages \
             WHERE connection_id = $1 AND message_id = $2",
        )
        .bind(&command.connection_id)
        .bind(&command.message_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?
        .is_some();
        if !message_exists {
            return Err(MailMessageFlagPersistenceErrorV1::MissingMessage);
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.mail_message_flag_operations \
             (operation_id, connection_id, provider_message_id, flag_kind, target_value, \
              request_sha256, exact_command_bytes, requested_at_unix_seconds) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (operation_id) DO NOTHING",
        )
        .bind(&command.operation_id)
        .bind(&command.connection_id)
        .bind(&command.message_id)
        .bind(flag_kind_id(command.kind))
        .bind(command.target_value)
        .bind(request_sha256.as_slice())
        .bind(canonical_command_bytes)
        .bind(requested_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?
        .rows_affected();
        if inserted == 0 {
            let row = sqlx::query(
                "SELECT request_sha256 FROM makosh_data.mail_message_flag_operations \
                 WHERE operation_id = $1",
            )
            .bind(&command.operation_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?;
            let existing_hash = row
                .try_get::<Vec<u8>, _>("request_sha256")
                .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?;
            if existing_hash.as_slice() != request_sha256 {
                return Err(MailMessageFlagPersistenceErrorV1::ConflictingOperation);
            }
        }
        transaction
            .commit()
            .await
            .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?;
        Ok(MailMessageFlagAcceptedV1 {
            operation_id: command.operation_id.clone(),
        })
    }

    pub async fn message_flag_operation_status(
        &self,
        request: &MailMessageFlagStatusRequestV1,
    ) -> Result<Option<MailMessageFlagOperationStatusV1>, MailMessageFlagPersistenceErrorV1> {
        validate_message_flag_status_request(request)
            .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidInput)?;
        let row = sqlx::query(
            "SELECT operation_id, connection_id, message_id, flag_kind, target_value, \
             outcome, requested_at_unix_seconds, completed_at_unix_seconds, projection_revision \
             FROM makosh_data.mail_message_flag_operations \
             WHERE operation_id = $1 AND connection_id = $2",
        )
        .bind(&request.operation_id)
        .bind(&request.connection_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?;
        row.as_ref().map(status_from_row).transpose()
    }

    pub async fn next_message_flag_command(
        &self,
        connection_id: &str,
    ) -> Result<Option<MailQueuedMessageFlagCommandV1>, MailMessageFlagPersistenceErrorV1> {
        if connection_id.is_empty() || connection_id.chars().any(char::is_control) {
            return Err(MailMessageFlagPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT operation_id, connection_id, message_id, flag_kind, target_value, \
             request_sha256, exact_command_bytes \
             FROM makosh_data.mail_message_flag_operations \
             WHERE connection_id = $1 AND outcome = 1 \
             ORDER BY requested_at_unix_seconds, operation_id LIMIT 1",
        )
        .bind(connection_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?;
        row.as_ref().map(queued_from_row).transpose()
    }

    pub async fn complete_message_flag_success(
        &self,
        queued: &MailQueuedMessageFlagCommandV1,
        completed_at_unix_seconds: i64,
    ) -> Result<u64, MailMessageFlagPersistenceErrorV1> {
        validate_queued(queued)?;
        if completed_at_unix_seconds <= 0 {
            return Err(MailMessageFlagPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?;
        let row = sqlx::query(
            "SELECT provider_thread_id, flags, projection_revision \
             FROM makosh_data.mail_operational_messages \
             WHERE connection_id = $1 AND message_id = $2 FOR UPDATE",
        )
        .bind(&queued.connection_id)
        .bind(&queued.message_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?
        .ok_or(MailMessageFlagPersistenceErrorV1::MissingMessage)?;
        let provider_thread_id = row
            .try_get::<String, _>("provider_thread_id")
            .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?;
        let mut flags = row
            .try_get::<Vec<i16>, _>("flags")
            .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?;
        if flags.iter().any(|flag| !(1..=6).contains(flag)) {
            return Err(MailMessageFlagPersistenceErrorV1::InvalidRow);
        }
        let current_revision = row
            .try_get::<i64, _>("projection_revision")
            .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?;
        let flag_id = flag_kind_id(queued.kind);
        let changed = flags.contains(&flag_id) != queued.target_value;
        if queued.target_value {
            flags.push(flag_id);
            flags.sort_unstable();
            flags.dedup();
        } else {
            flags.retain(|flag| *flag != flag_id);
        }
        let revision = if changed {
            current_revision
                .checked_add(1)
                .ok_or(MailMessageFlagPersistenceErrorV1::InvalidRow)?
        } else {
            current_revision
        };
        if changed {
            sqlx::query(
                "UPDATE makosh_data.mail_operational_messages \
                 SET flags = $3, projection_revision = $4, updated_at_unix_seconds = $5 \
                 WHERE connection_id = $1 AND message_id = $2",
            )
            .bind(&queued.connection_id)
            .bind(&queued.message_id)
            .bind(flags)
            .bind(revision)
            .bind(completed_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?;
            let folder_rows = sqlx::query(
                "SELECT folder_id FROM makosh_data.mail_operational_message_folders \
                 WHERE connection_id = $1 AND message_id = $2",
            )
            .bind(&queued.connection_id)
            .bind(&queued.message_id)
            .fetch_all(&mut *transaction)
            .await
            .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?;
            crate::operational::refresh_thread(
                &mut transaction,
                &queued.connection_id,
                &provider_thread_id,
                completed_at_unix_seconds,
            )
            .await
            .map_err(map_operational_error)?;
            for row in &folder_rows {
                let folder_id = row
                    .try_get::<String, _>("folder_id")
                    .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?;
                crate::operational::refresh_folder(
                    &mut transaction,
                    &queued.connection_id,
                    &folder_id,
                    completed_at_unix_seconds,
                )
                .await
                .map_err(map_operational_error)?;
            }
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_message_flag_operations \
             SET outcome = 2, completed_at_unix_seconds = $3, projection_revision = $4 \
             WHERE operation_id = $1 AND connection_id = $2 AND outcome = 1",
        )
        .bind(&queued.operation_id)
        .bind(&queued.connection_id)
        .bind(completed_at_unix_seconds)
        .bind(revision)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?
        .rows_affected();
        if updated != 1 {
            return Err(MailMessageFlagPersistenceErrorV1::InvalidRow);
        }
        transaction
            .commit()
            .await
            .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?;
        u64::try_from(revision).map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)
    }

    pub async fn complete_message_flag_failure(
        &self,
        operation_id: &str,
        connection_id: &str,
        outcome: MailMessageFlagOperationOutcomeV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailMessageFlagPersistenceErrorV1> {
        let outcome_id = match outcome {
            MailMessageFlagOperationOutcomeV1::Rejected => 3,
            MailMessageFlagOperationOutcomeV1::OutcomeUnknown => 4,
            MailMessageFlagOperationOutcomeV1::Pending
            | MailMessageFlagOperationOutcomeV1::Succeeded => {
                return Err(MailMessageFlagPersistenceErrorV1::InvalidInput);
            }
        };
        if operation_id.is_empty() || connection_id.is_empty() || completed_at_unix_seconds <= 0 {
            return Err(MailMessageFlagPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_message_flag_operations \
             SET outcome = $3, completed_at_unix_seconds = $4 \
             WHERE operation_id = $1 AND connection_id = $2 AND outcome = 1",
        )
        .bind(operation_id)
        .bind(connection_id)
        .bind(outcome_id)
        .bind(completed_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map_err(|_| MailMessageFlagPersistenceErrorV1::Database)?
        .rows_affected();
        if updated != 1 {
            return Err(MailMessageFlagPersistenceErrorV1::InvalidRow);
        }
        Ok(())
    }
}

fn validate_queued(
    queued: &MailQueuedMessageFlagCommandV1,
) -> Result<(), MailMessageFlagPersistenceErrorV1> {
    let command = decode_message_flag_command(&queued.exact_command_bytes)
        .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?;
    let request_sha256: [u8; 32] = Sha256::digest(&queued.exact_command_bytes).into();
    if request_sha256 != queued.request_sha256
        || command.operation_id != queued.operation_id
        || command.connection_id != queued.connection_id
        || command.message_id != queued.message_id
        || command.kind != queued.kind
        || command.target_value != queued.target_value
    {
        return Err(MailMessageFlagPersistenceErrorV1::InvalidRow);
    }
    Ok(())
}

fn queued_from_row(
    row: &PgRow,
) -> Result<MailQueuedMessageFlagCommandV1, MailMessageFlagPersistenceErrorV1> {
    let request_sha256 = row
        .try_get::<Vec<u8>, _>("request_sha256")
        .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?
        .try_into()
        .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?;
    let queued = MailQueuedMessageFlagCommandV1 {
        operation_id: row_string(row, "operation_id")?,
        connection_id: row_string(row, "connection_id")?,
        message_id: row_string(row, "message_id")?,
        kind: flag_kind_from_id(row_i16(row, "flag_kind")?)?,
        target_value: row
            .try_get::<bool, _>("target_value")
            .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?,
        request_sha256,
        exact_command_bytes: row
            .try_get::<Vec<u8>, _>("exact_command_bytes")
            .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?,
    };
    validate_queued(&queued)?;
    Ok(queued)
}

fn status_from_row(
    row: &PgRow,
) -> Result<MailMessageFlagOperationStatusV1, MailMessageFlagPersistenceErrorV1> {
    let outcome = outcome_from_id(row_i16(row, "outcome")?)?;
    let status = MailMessageFlagOperationStatusV1 {
        operation_id: row_string(row, "operation_id")?,
        connection_id: row_string(row, "connection_id")?,
        message_id: row_string(row, "message_id")?,
        kind: flag_kind_from_id(row_i16(row, "flag_kind")?)?,
        target_value: row
            .try_get::<bool, _>("target_value")
            .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?,
        outcome,
        requested_at_unix_seconds: row_i64(row, "requested_at_unix_seconds")?,
        completed_at_unix_seconds: row
            .try_get::<Option<i64>, _>("completed_at_unix_seconds")
            .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?,
        projection_revision: row
            .try_get::<Option<i64>, _>("projection_revision")
            .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?
            .map(|value| {
                u64::try_from(value).map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)
            })
            .transpose()?,
    };
    validate_message_flag_status(&status)
        .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)?;
    Ok(status)
}

const fn flag_kind_id(kind: MailMessageFlagKindV1) -> i16 {
    match kind {
        MailMessageFlagKindV1::Read => 1,
        MailMessageFlagKindV1::Starred => 2,
    }
}

fn flag_kind_from_id(
    value: i16,
) -> Result<MailMessageFlagKindV1, MailMessageFlagPersistenceErrorV1> {
    match value {
        1 => Ok(MailMessageFlagKindV1::Read),
        2 => Ok(MailMessageFlagKindV1::Starred),
        _ => Err(MailMessageFlagPersistenceErrorV1::InvalidRow),
    }
}

fn outcome_from_id(
    value: i16,
) -> Result<MailMessageFlagOperationOutcomeV1, MailMessageFlagPersistenceErrorV1> {
    match value {
        1 => Ok(MailMessageFlagOperationOutcomeV1::Pending),
        2 => Ok(MailMessageFlagOperationOutcomeV1::Succeeded),
        3 => Ok(MailMessageFlagOperationOutcomeV1::Rejected),
        4 => Ok(MailMessageFlagOperationOutcomeV1::OutcomeUnknown),
        _ => Err(MailMessageFlagPersistenceErrorV1::InvalidRow),
    }
}

fn row_string(row: &PgRow, column: &str) -> Result<String, MailMessageFlagPersistenceErrorV1> {
    row.try_get::<String, _>(column)
        .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)
}

fn row_i16(row: &PgRow, column: &str) -> Result<i16, MailMessageFlagPersistenceErrorV1> {
    row.try_get::<i16, _>(column)
        .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)
}

fn row_i64(row: &PgRow, column: &str) -> Result<i64, MailMessageFlagPersistenceErrorV1> {
    row.try_get::<i64, _>(column)
        .map_err(|_| MailMessageFlagPersistenceErrorV1::InvalidRow)
}

fn map_operational_error(
    error: crate::MailDurablePersistenceError,
) -> MailMessageFlagPersistenceErrorV1 {
    match error {
        crate::MailDurablePersistenceError::Database => MailMessageFlagPersistenceErrorV1::Database,
        _ => MailMessageFlagPersistenceErrorV1::InvalidRow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_is_owner_local_and_keeps_exact_command_bytes() {
        assert!(MAIL_SCHEMA_V12.contains("makosh_data.mail_message_flag_operations"));
        assert!(MAIL_SCHEMA_V12.contains("exact_command_bytes BYTEA"));
        assert!(MAIL_SCHEMA_V12.contains("request_sha256 BYTEA"));
        assert!(!MAIL_SCHEMA_V12.contains("communications"));
        assert!(!MAIL_SCHEMA_V12.contains("credential"));
        assert!(!MAIL_SCHEMA_V12.contains("FOREIGN KEY"));
    }
}
