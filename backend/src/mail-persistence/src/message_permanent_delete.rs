//! Durable provider-owned permanent deletion journal and operational cleanup.

use makosh_mail_api::{
    message_permanent_delete::{
        MailMessagePermanentDeleteAcceptedV1, MailMessagePermanentDeleteCommandV1,
        MailMessagePermanentDeleteConfirmationV1, MailMessagePermanentDeleteOperationOutcomeV1,
        MailMessagePermanentDeleteOperationStatusV1, MailMessagePermanentDeleteStatusRequestV1,
        validate_message_permanent_delete_command, validate_message_permanent_delete_status,
        validate_message_permanent_delete_status_request,
    },
    message_permanent_delete_wire::{
        decode_message_permanent_delete_command, encode_message_permanent_delete_command,
    },
};
use sha2::{Digest, Sha256};
use sqlx::{Row, postgres::PgRow};

use crate::{
    MailDurablePersistence, MailDurablePersistenceError, MailImapMessageLocatorV1,
    provider_location::validate_imap_locator,
};

const MAIL_FOLDER_KIND_TRASH_DB_VALUE: i16 = 4;

pub const MAIL_SCHEMA_V17: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.mail_message_permanent_delete_operations (
    operation_id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    expected_projection_revision BIGINT NOT NULL,
    confirmation SMALLINT NOT NULL,
    request_sha256 BYTEA NOT NULL,
    exact_command_bytes BYTEA NOT NULL,
    outcome SMALLINT NOT NULL DEFAULT 1,
    requested_at_unix_seconds BIGINT NOT NULL,
    completed_at_unix_seconds BIGINT,
    deletion_projection_revision BIGINT,
    CHECK (operation_id <> ''),
    CHECK (connection_id <> ''),
    CHECK (message_id <> ''),
    CHECK (expected_projection_revision > 0),
    CHECK (confirmation = 1),
    CHECK (octet_length(request_sha256) = 32),
    CHECK (octet_length(exact_command_bytes) BETWEEN 1 AND 4096),
    CHECK (outcome BETWEEN 1 AND 6),
    CHECK (requested_at_unix_seconds > 0),
    CHECK ((outcome = 1 AND completed_at_unix_seconds IS NULL
            AND deletion_projection_revision IS NULL)
        OR (outcome = 2 AND completed_at_unix_seconds IS NOT NULL
            AND deletion_projection_revision > expected_projection_revision)
        OR (outcome BETWEEN 3 AND 6 AND completed_at_unix_seconds IS NOT NULL
            AND deletion_projection_revision IS NULL))
);
CREATE INDEX IF NOT EXISTS mail_message_permanent_delete_pending_idx
    ON makosh_data.mail_message_permanent_delete_operations
    (connection_id, requested_at_unix_seconds, operation_id)
    WHERE outcome = 1;
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessagePermanentDeletePersistenceErrorV1 {
    Database,
    InvalidInput,
    ConflictingOperation,
    MissingMessage,
    StaleProjection,
    NotInTrash,
    InvalidRow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailQueuedMessagePermanentDeleteCommandV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub message_id: String,
    pub expected_projection_revision: u64,
    pub request_sha256: [u8; 32],
    pub exact_command_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessagePermanentDeleteTargetV1 {
    pub provider_message_id: String,
    pub imap_locator: Option<MailImapMessageLocatorV1>,
}

impl MailDurablePersistence {
    pub async fn enqueue_message_permanent_delete_command(
        &self,
        command: &MailMessagePermanentDeleteCommandV1,
        canonical_command_bytes: &[u8],
        requested_at_unix_seconds: i64,
    ) -> Result<MailMessagePermanentDeleteAcceptedV1, MailMessagePermanentDeletePersistenceErrorV1>
    {
        if requested_at_unix_seconds <= 0
            || validate_message_permanent_delete_command(command).is_err()
            || canonical_command_bytes.is_empty()
            || canonical_command_bytes.len() > 4_096
            || encode_message_permanent_delete_command(command)
                .map(|bytes| bytes != canonical_command_bytes)
                .unwrap_or(true)
        {
            return Err(MailMessagePermanentDeletePersistenceErrorV1::InvalidInput);
        }
        let request_sha256: [u8; 32] = Sha256::digest(canonical_command_bytes).into();
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?;
        if let Some(row) = sqlx::query(
            "SELECT request_sha256 \
             FROM makosh_data.mail_message_permanent_delete_operations \
             WHERE operation_id = $1",
        )
        .bind(&command.operation_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?
        {
            let existing: Vec<u8> = row
                .try_get("request_sha256")
                .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?;
            if existing.as_slice() != request_sha256 {
                return Err(MailMessagePermanentDeletePersistenceErrorV1::ConflictingOperation);
            }
            transaction
                .commit()
                .await
                .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?;
            return Ok(MailMessagePermanentDeleteAcceptedV1 {
                operation_id: command.operation_id.clone(),
            });
        }
        validate_current_delete_target(
            &mut transaction,
            &command.connection_id,
            &command.message_id,
            command.expected_projection_revision,
        )
        .await?;
        sqlx::query(
            "INSERT INTO makosh_data.mail_message_permanent_delete_operations \
             (operation_id, connection_id, message_id, expected_projection_revision, confirmation, \
              request_sha256, exact_command_bytes, requested_at_unix_seconds) \
             VALUES ($1, $2, $3, $4, 1, $5, $6, $7)",
        )
        .bind(&command.operation_id)
        .bind(&command.connection_id)
        .bind(&command.message_id)
        .bind(
            i64::try_from(command.expected_projection_revision)
                .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidInput)?,
        )
        .bind(request_sha256.as_slice())
        .bind(canonical_command_bytes)
        .bind(requested_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?;
        Ok(MailMessagePermanentDeleteAcceptedV1 {
            operation_id: command.operation_id.clone(),
        })
    }

    pub async fn message_permanent_delete_operation_status(
        &self,
        request: &MailMessagePermanentDeleteStatusRequestV1,
    ) -> Result<
        Option<MailMessagePermanentDeleteOperationStatusV1>,
        MailMessagePermanentDeletePersistenceErrorV1,
    > {
        validate_message_permanent_delete_status_request(request)
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidInput)?;
        let row = sqlx::query(
            "SELECT operation_id, connection_id, message_id, expected_projection_revision, \
                    confirmation, outcome, requested_at_unix_seconds, \
                    completed_at_unix_seconds, deletion_projection_revision \
             FROM makosh_data.mail_message_permanent_delete_operations \
             WHERE operation_id = $1 AND connection_id = $2",
        )
        .bind(&request.operation_id)
        .bind(&request.connection_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?;
        row.as_ref().map(status_from_row).transpose()
    }

    pub async fn next_message_permanent_delete_command(
        &self,
        connection_id: &str,
    ) -> Result<
        Option<MailQueuedMessagePermanentDeleteCommandV1>,
        MailMessagePermanentDeletePersistenceErrorV1,
    > {
        if connection_id.is_empty() || connection_id.chars().any(char::is_control) {
            return Err(MailMessagePermanentDeletePersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT operation_id, connection_id, message_id, expected_projection_revision, \
                    request_sha256, exact_command_bytes \
             FROM makosh_data.mail_message_permanent_delete_operations \
             WHERE connection_id = $1 AND outcome = 1 \
             ORDER BY requested_at_unix_seconds, operation_id LIMIT 1",
        )
        .bind(connection_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?;
        row.as_ref().map(queued_from_row).transpose()
    }

    pub async fn message_permanent_delete_target(
        &self,
        queued: &MailQueuedMessagePermanentDeleteCommandV1,
    ) -> Result<MailMessagePermanentDeleteTargetV1, MailMessagePermanentDeletePersistenceErrorV1>
    {
        validate_queued(queued)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?;
        validate_current_delete_target(
            &mut transaction,
            &queued.connection_id,
            &queued.message_id,
            queued.expected_projection_revision,
        )
        .await?;
        let locator = sqlx::query(
            "SELECT mailbox_id, uid_validity, uid \
             FROM makosh_data.mail_imap_message_locators \
             WHERE connection_id = $1 AND message_id = $2",
        )
        .bind(&queued.connection_id)
        .bind(&queued.message_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?
        .as_ref()
        .map(locator_from_row)
        .transpose()?;
        transaction
            .commit()
            .await
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?;
        Ok(MailMessagePermanentDeleteTargetV1 {
            provider_message_id: queued.message_id.clone(),
            imap_locator: locator,
        })
    }

    pub async fn complete_message_permanent_delete_success(
        &self,
        queued: &MailQueuedMessagePermanentDeleteCommandV1,
        completed_at_unix_seconds: i64,
    ) -> Result<u64, MailMessagePermanentDeletePersistenceErrorV1> {
        validate_queued(queued)?;
        if completed_at_unix_seconds <= 0 {
            return Err(MailMessagePermanentDeletePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?;
        let (provider_thread_id, old_folders) = validate_current_delete_target(
            &mut transaction,
            &queued.connection_id,
            &queued.message_id,
            queued.expected_projection_revision,
        )
        .await?;
        let deleted = sqlx::query(
            "DELETE FROM makosh_data.mail_operational_messages \
             WHERE connection_id = $1 AND message_id = $2",
        )
        .bind(&queued.connection_id)
        .bind(&queued.message_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?
        .rows_affected();
        if deleted != 1 {
            return Err(MailMessagePermanentDeletePersistenceErrorV1::MissingMessage);
        }
        crate::operational::refresh_thread(
            &mut transaction,
            &queued.connection_id,
            &provider_thread_id,
            completed_at_unix_seconds,
        )
        .await
        .map_err(map_operational_error)?;
        for folder_id in &old_folders {
            crate::operational::refresh_folder(
                &mut transaction,
                &queued.connection_id,
                folder_id,
                completed_at_unix_seconds,
            )
            .await
            .map_err(map_operational_error)?;
        }
        let deletion_revision = queued
            .expected_projection_revision
            .checked_add(1)
            .ok_or(MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_message_permanent_delete_operations \
             SET outcome = 2, completed_at_unix_seconds = $3, \
                 deletion_projection_revision = $4 \
             WHERE operation_id = $1 AND connection_id = $2 AND outcome = 1",
        )
        .bind(&queued.operation_id)
        .bind(&queued.connection_id)
        .bind(completed_at_unix_seconds)
        .bind(
            i64::try_from(deletion_revision)
                .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?,
        )
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?
        .rows_affected();
        if updated != 1 {
            return Err(MailMessagePermanentDeletePersistenceErrorV1::InvalidRow);
        }
        transaction
            .commit()
            .await
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?;
        Ok(deletion_revision)
    }

    pub async fn complete_message_permanent_delete_failure(
        &self,
        operation_id: &str,
        connection_id: &str,
        outcome: MailMessagePermanentDeleteOperationOutcomeV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailMessagePermanentDeletePersistenceErrorV1> {
        let outcome_id = match outcome {
            MailMessagePermanentDeleteOperationOutcomeV1::Rejected => 3,
            MailMessagePermanentDeleteOperationOutcomeV1::Unsupported => 4,
            MailMessagePermanentDeleteOperationOutcomeV1::ReauthorizationRequired => 5,
            MailMessagePermanentDeleteOperationOutcomeV1::OutcomeUnknown => 6,
            MailMessagePermanentDeleteOperationOutcomeV1::Pending
            | MailMessagePermanentDeleteOperationOutcomeV1::Succeeded => {
                return Err(MailMessagePermanentDeletePersistenceErrorV1::InvalidInput);
            }
        };
        if operation_id.is_empty() || connection_id.is_empty() || completed_at_unix_seconds <= 0 {
            return Err(MailMessagePermanentDeletePersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_message_permanent_delete_operations \
             SET outcome = $3, completed_at_unix_seconds = $4 \
             WHERE operation_id = $1 AND connection_id = $2 AND outcome = 1",
        )
        .bind(operation_id)
        .bind(connection_id)
        .bind(outcome_id)
        .bind(completed_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?
        .rows_affected();
        if updated != 1 {
            return Err(MailMessagePermanentDeletePersistenceErrorV1::InvalidRow);
        }
        Ok(())
    }
}

async fn validate_current_delete_target(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    connection_id: &str,
    message_id: &str,
    expected_projection_revision: u64,
) -> Result<(String, Vec<String>), MailMessagePermanentDeletePersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT provider_thread_id, projection_revision \
         FROM makosh_data.mail_operational_messages \
         WHERE connection_id = $1 AND message_id = $2 FOR UPDATE",
    )
    .bind(connection_id)
    .bind(message_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?
    .ok_or(MailMessagePermanentDeletePersistenceErrorV1::MissingMessage)?;
    let revision = u64::try_from(
        row.try_get::<i64, _>("projection_revision")
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?,
    )
    .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?;
    if revision != expected_projection_revision {
        return Err(MailMessagePermanentDeletePersistenceErrorV1::StaleProjection);
    }
    let folders = sqlx::query(
        "SELECT membership.folder_id, folder.kind \
         FROM makosh_data.mail_operational_message_folders membership \
         JOIN makosh_data.mail_operational_folders folder \
           ON folder.connection_id = membership.connection_id \
          AND folder.folder_id = membership.folder_id \
         WHERE membership.connection_id = $1 AND membership.message_id = $2 \
         ORDER BY membership.folder_id",
    )
    .bind(connection_id)
    .bind(message_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::Database)?;
    let trash_count = folders
        .iter()
        .filter(|row| row.try_get::<i16, _>("kind").ok() == Some(MAIL_FOLDER_KIND_TRASH_DB_VALUE))
        .count();
    if trash_count != 1 {
        return Err(MailMessagePermanentDeletePersistenceErrorV1::NotInTrash);
    }
    let folder_ids = folders
        .iter()
        .map(|row| {
            row.try_get("folder_id")
                .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)
        })
        .collect::<Result<Vec<String>, _>>()?;
    Ok((
        row.try_get("provider_thread_id")
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?,
        folder_ids,
    ))
}

fn queued_from_row(
    row: &PgRow,
) -> Result<MailQueuedMessagePermanentDeleteCommandV1, MailMessagePermanentDeletePersistenceErrorV1>
{
    let queued = MailQueuedMessagePermanentDeleteCommandV1 {
        operation_id: row_string(row, "operation_id")?,
        connection_id: row_string(row, "connection_id")?,
        message_id: row_string(row, "message_id")?,
        expected_projection_revision: u64::try_from(row_i64(row, "expected_projection_revision")?)
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?,
        request_sha256: row
            .try_get::<Vec<u8>, _>("request_sha256")
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?
            .try_into()
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?,
        exact_command_bytes: row
            .try_get("exact_command_bytes")
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?,
    };
    validate_queued(&queued)?;
    Ok(queued)
}

fn validate_queued(
    queued: &MailQueuedMessagePermanentDeleteCommandV1,
) -> Result<(), MailMessagePermanentDeletePersistenceErrorV1> {
    let command = decode_message_permanent_delete_command(&queued.exact_command_bytes)
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidInput)?;
    let hash: [u8; 32] = Sha256::digest(&queued.exact_command_bytes).into();
    if hash != queued.request_sha256
        || queued.operation_id != command.operation_id
        || queued.connection_id != command.connection_id
        || queued.message_id != command.message_id
        || queued.expected_projection_revision != command.expected_projection_revision
        || command.confirmation != MailMessagePermanentDeleteConfirmationV1::Confirmed
    {
        return Err(MailMessagePermanentDeletePersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn status_from_row(
    row: &PgRow,
) -> Result<MailMessagePermanentDeleteOperationStatusV1, MailMessagePermanentDeletePersistenceErrorV1>
{
    let status = MailMessagePermanentDeleteOperationStatusV1 {
        operation_id: row_string(row, "operation_id")?,
        connection_id: row_string(row, "connection_id")?,
        message_id: row_string(row, "message_id")?,
        expected_projection_revision: u64::try_from(row_i64(row, "expected_projection_revision")?)
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?,
        confirmation: match row_i16(row, "confirmation")? {
            1 => MailMessagePermanentDeleteConfirmationV1::Confirmed,
            _ => return Err(MailMessagePermanentDeletePersistenceErrorV1::InvalidRow),
        },
        outcome: match row_i16(row, "outcome")? {
            1 => MailMessagePermanentDeleteOperationOutcomeV1::Pending,
            2 => MailMessagePermanentDeleteOperationOutcomeV1::Succeeded,
            3 => MailMessagePermanentDeleteOperationOutcomeV1::Rejected,
            4 => MailMessagePermanentDeleteOperationOutcomeV1::Unsupported,
            5 => MailMessagePermanentDeleteOperationOutcomeV1::ReauthorizationRequired,
            6 => MailMessagePermanentDeleteOperationOutcomeV1::OutcomeUnknown,
            _ => return Err(MailMessagePermanentDeletePersistenceErrorV1::InvalidRow),
        },
        requested_at_unix_seconds: row_i64(row, "requested_at_unix_seconds")?,
        completed_at_unix_seconds: row
            .try_get("completed_at_unix_seconds")
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?,
        deletion_projection_revision: row
            .try_get::<Option<i64>, _>("deletion_projection_revision")
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?
            .map(u64::try_from)
            .transpose()
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?,
    };
    validate_message_permanent_delete_status(&status)
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?;
    Ok(status)
}

fn locator_from_row(
    row: &PgRow,
) -> Result<MailImapMessageLocatorV1, MailMessagePermanentDeletePersistenceErrorV1> {
    let locator = MailImapMessageLocatorV1 {
        mailbox_id: row_string(row, "mailbox_id")?,
        uid_validity: u32::try_from(row_i64(row, "uid_validity")?)
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?,
        uid: u32::try_from(row_i64(row, "uid")?)
            .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?,
    };
    validate_imap_locator(&locator)
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)?;
    Ok(locator)
}

fn row_string(
    row: &PgRow,
    column: &str,
) -> Result<String, MailMessagePermanentDeletePersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)
}

fn row_i16(row: &PgRow, column: &str) -> Result<i16, MailMessagePermanentDeletePersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)
}

fn row_i64(row: &PgRow, column: &str) -> Result<i64, MailMessagePermanentDeletePersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| MailMessagePermanentDeletePersistenceErrorV1::InvalidRow)
}

fn map_operational_error(
    _error: MailDurablePersistenceError,
) -> MailMessagePermanentDeletePersistenceErrorV1 {
    MailMessagePermanentDeletePersistenceErrorV1::Database
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_is_owner_local_and_keeps_the_command_bytes() {
        assert!(MAIL_SCHEMA_V17.contains("makosh_data.mail_message_permanent_delete_operations"));
        assert!(MAIL_SCHEMA_V17.contains("exact_command_bytes"));
        assert!(!MAIL_SCHEMA_V17.contains("communications"));
    }
}
