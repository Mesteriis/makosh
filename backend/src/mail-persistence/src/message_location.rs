use makosh_mail_api::{
    message_location::{
        MailMessageLocationAcceptedV1, MailMessageLocationCommandV1, MailMessageLocationKindV1,
        MailMessageLocationOperationOutcomeV1, MailMessageLocationOperationStatusV1,
        MailMessageLocationStatusRequestV1, validate_message_location_command,
        validate_message_location_status, validate_message_location_status_request,
    },
    message_location_wire::{decode_message_location_command, encode_message_location_command},
    operational::MailFolderKindV1,
};
use sha2::{Digest, Sha256};
use sqlx::{Row, postgres::PgRow};

use crate::{
    MailDurablePersistence, MailDurablePersistenceError, MailImapMessageLocatorV1,
    MailOperationalFolderSnapshotV1,
    provider_location::{upsert_imap_message_locator, validate_imap_locator},
};

pub const MAIL_SCHEMA_V15: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.mail_message_location_operations (
    operation_id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    location_kind SMALLINT NOT NULL CHECK (location_kind BETWEEN 1 AND 4),
    target_folder_id TEXT,
    request_sha256 BYTEA NOT NULL CHECK (octet_length(request_sha256) = 32),
    exact_command_bytes BYTEA NOT NULL
        CHECK (octet_length(exact_command_bytes) BETWEEN 1 AND 4096),
    outcome SMALLINT NOT NULL DEFAULT 1 CHECK (outcome BETWEEN 1 AND 5),
    requested_at_unix_seconds BIGINT NOT NULL CHECK (requested_at_unix_seconds > 0),
    completed_at_unix_seconds BIGINT,
    projection_revision BIGINT CHECK (projection_revision > 0),
    FOREIGN KEY (connection_id, message_id)
        REFERENCES makosh_data.mail_operational_messages (connection_id, message_id)
        ON DELETE CASCADE,
    CHECK (
        (location_kind = 4 AND target_folder_id IS NOT NULL
            AND target_folder_id <> '' AND octet_length(target_folder_id) <= 512
            AND target_folder_id !~ '[\x00\r\n]')
        OR (location_kind <> 4 AND target_folder_id IS NULL)
    )
);

CREATE INDEX IF NOT EXISTS mail_message_location_operations_pending_idx
ON makosh_data.mail_message_location_operations (
    connection_id,
    outcome,
    requested_at_unix_seconds,
    operation_id
);
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailMessageLocationPersistenceErrorV1 {
    Database,
    InvalidInput,
    ConflictingOperation,
    MissingMessage,
    InvalidRow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailQueuedMessageLocationCommandV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub message_id: String,
    pub kind: MailMessageLocationKindV1,
    pub target_folder_id: Option<String>,
    pub request_sha256: [u8; 32],
    pub exact_command_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessageLocationReconciliationV1 {
    pub folders: Vec<MailOperationalFolderSnapshotV1>,
    pub imap_locator: Option<MailImapMessageLocatorV1>,
}

impl MailDurablePersistence {
    pub async fn enqueue_message_location_command(
        &self,
        command: &MailMessageLocationCommandV1,
        canonical_command_bytes: &[u8],
        requested_at_unix_seconds: i64,
    ) -> Result<MailMessageLocationAcceptedV1, MailMessageLocationPersistenceErrorV1> {
        if requested_at_unix_seconds <= 0
            || validate_message_location_command(command).is_err()
            || canonical_command_bytes.is_empty()
            || canonical_command_bytes.len() > 4_096
            || encode_message_location_command(command)
                .map(|bytes| bytes != canonical_command_bytes)
                .unwrap_or(true)
        {
            return Err(MailMessageLocationPersistenceErrorV1::InvalidInput);
        }
        let request_sha256: [u8; 32] = Sha256::digest(canonical_command_bytes).into();
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
        let existing = sqlx::query(
            "SELECT request_sha256 FROM makosh_data.mail_message_location_operations \
             WHERE operation_id = $1",
        )
        .bind(&command.operation_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
        if let Some(row) = existing {
            let existing_hash = row
                .try_get::<Vec<u8>, _>("request_sha256")
                .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?;
            if existing_hash.as_slice() != request_sha256 {
                return Err(MailMessageLocationPersistenceErrorV1::ConflictingOperation);
            }
            transaction
                .commit()
                .await
                .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
            return Ok(MailMessageLocationAcceptedV1 {
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
        .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?
        .is_some();
        if !message_exists {
            return Err(MailMessageLocationPersistenceErrorV1::MissingMessage);
        }
        if let Some(target_folder_id) = command.target_folder_id.as_deref() {
            let target_exists = sqlx::query(
                "SELECT 1 FROM makosh_data.mail_operational_folders \
                 WHERE connection_id = $1 AND folder_id = $2",
            )
            .bind(&command.connection_id)
            .bind(target_folder_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?
            .is_some();
            if !target_exists {
                return Err(MailMessageLocationPersistenceErrorV1::InvalidInput);
            }
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_message_location_operations \
             (operation_id, connection_id, message_id, location_kind, target_folder_id, \
              request_sha256, exact_command_bytes, requested_at_unix_seconds) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(&command.operation_id)
        .bind(&command.connection_id)
        .bind(&command.message_id)
        .bind(location_kind_id(command.kind))
        .bind(&command.target_folder_id)
        .bind(request_sha256.as_slice())
        .bind(canonical_command_bytes)
        .bind(requested_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
        transaction
            .commit()
            .await
            .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
        Ok(MailMessageLocationAcceptedV1 {
            operation_id: command.operation_id.clone(),
        })
    }

    pub async fn message_location_operation_status(
        &self,
        request: &MailMessageLocationStatusRequestV1,
    ) -> Result<Option<MailMessageLocationOperationStatusV1>, MailMessageLocationPersistenceErrorV1>
    {
        validate_message_location_status_request(request)
            .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidInput)?;
        let row = sqlx::query(
            "SELECT operation_id, connection_id, message_id, location_kind, target_folder_id, \
             outcome, requested_at_unix_seconds, completed_at_unix_seconds, projection_revision \
             FROM makosh_data.mail_message_location_operations \
             WHERE operation_id = $1 AND connection_id = $2",
        )
        .bind(&request.operation_id)
        .bind(&request.connection_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
        row.as_ref().map(status_from_row).transpose()
    }

    pub async fn next_message_location_command(
        &self,
        connection_id: &str,
    ) -> Result<Option<MailQueuedMessageLocationCommandV1>, MailMessageLocationPersistenceErrorV1>
    {
        if connection_id.is_empty() || connection_id.chars().any(char::is_control) {
            return Err(MailMessageLocationPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT operation_id, connection_id, message_id, location_kind, target_folder_id, \
             request_sha256, exact_command_bytes \
             FROM makosh_data.mail_message_location_operations \
             WHERE connection_id = $1 AND outcome = 1 \
             ORDER BY requested_at_unix_seconds, operation_id LIMIT 1",
        )
        .bind(connection_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
        row.as_ref().map(queued_from_row).transpose()
    }

    pub async fn message_location_target_folder(
        &self,
        command: &MailMessageLocationCommandV1,
    ) -> Result<Option<MailOperationalFolderSnapshotV1>, MailMessageLocationPersistenceErrorV1>
    {
        validate_message_location_command(command)
            .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidInput)?;
        let rows = if let Some(target) = command.target_folder_id.as_deref() {
            sqlx::query(
                "SELECT folder_id, display_name, kind \
                 FROM makosh_data.mail_operational_folders \
                 WHERE connection_id = $1 AND folder_id = $2",
            )
            .bind(&command.connection_id)
            .bind(target)
            .fetch_all(&self.pool)
            .await
        } else {
            let kind = match command.kind {
                MailMessageLocationKindV1::Archive => MailFolderKindV1::Archive,
                MailMessageLocationKindV1::Trash => MailFolderKindV1::Trash,
                MailMessageLocationKindV1::Restore => MailFolderKindV1::Inbox,
                MailMessageLocationKindV1::Move => {
                    return Err(MailMessageLocationPersistenceErrorV1::InvalidInput);
                }
            };
            sqlx::query(
                "SELECT folder_id, display_name, kind \
                 FROM makosh_data.mail_operational_folders \
                 WHERE connection_id = $1 AND kind = $2 ORDER BY folder_id LIMIT 2",
            )
            .bind(&command.connection_id)
            .bind(folder_kind_id(kind))
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
        if rows.len() != 1 {
            return Ok(None);
        }
        folder_from_row(&rows[0]).map(Some)
    }

    pub async fn complete_message_location_success(
        &self,
        queued: &MailQueuedMessageLocationCommandV1,
        reconciliation: &MailMessageLocationReconciliationV1,
        completed_at_unix_seconds: i64,
    ) -> Result<u64, MailMessageLocationPersistenceErrorV1> {
        validate_queued(queued)?;
        if completed_at_unix_seconds <= 0 || reconciliation.folders.is_empty() {
            return Err(MailMessageLocationPersistenceErrorV1::InvalidInput);
        }
        if let Some(locator) = reconciliation.imap_locator.as_ref() {
            validate_imap_locator(locator)
                .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidInput)?;
        }
        let mut new_folder_ids = reconciliation
            .folders
            .iter()
            .map(|folder| folder.folder_id.clone())
            .collect::<Vec<_>>();
        new_folder_ids.sort();
        new_folder_ids.dedup();
        if new_folder_ids.len() != reconciliation.folders.len() {
            return Err(MailMessageLocationPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
        let row = sqlx::query(
            "SELECT provider_thread_id, projection_revision \
             FROM makosh_data.mail_operational_messages \
             WHERE connection_id = $1 AND message_id = $2 FOR UPDATE",
        )
        .bind(&queued.connection_id)
        .bind(&queued.message_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?
        .ok_or(MailMessageLocationPersistenceErrorV1::MissingMessage)?;
        let provider_thread_id = row
            .try_get::<String, _>("provider_thread_id")
            .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?;
        let current_revision = row
            .try_get::<i64, _>("projection_revision")
            .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?;
        let old_rows = sqlx::query(
            "SELECT folder_id FROM makosh_data.mail_operational_message_folders \
             WHERE connection_id = $1 AND message_id = $2 ORDER BY folder_id",
        )
        .bind(&queued.connection_id)
        .bind(&queued.message_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
        let old_folder_ids = old_rows
            .iter()
            .map(|row| {
                row.try_get::<String, _>("folder_id")
                    .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let old_locator = sqlx::query(
            "SELECT mailbox_id, uid_validity, uid FROM makosh_data.mail_imap_message_locators \
             WHERE connection_id = $1 AND message_id = $2 FOR UPDATE",
        )
        .bind(&queued.connection_id)
        .bind(&queued.message_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?
        .as_ref()
        .map(locator_from_row)
        .transpose()?;
        let changed = old_folder_ids != new_folder_ids
            || reconciliation
                .imap_locator
                .as_ref()
                .is_some_and(|locator| old_locator.as_ref() != Some(locator));
        for folder in &reconciliation.folders {
            crate::operational::upsert_operational_folder(
                &mut transaction,
                &queued.connection_id,
                folder,
                completed_at_unix_seconds,
            )
            .await
            .map_err(map_operational_error)?;
        }
        sqlx::query(
            "DELETE FROM makosh_data.mail_operational_message_folders \
             WHERE connection_id = $1 AND message_id = $2",
        )
        .bind(&queued.connection_id)
        .bind(&queued.message_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
        for folder_id in &new_folder_ids {
            sqlx::query(
                "INSERT INTO makosh_data.mail_operational_message_folders \
                 (connection_id, provider_message_id, folder_id) VALUES ($1, $2, $3)",
            )
            .bind(&queued.connection_id)
            .bind(&queued.message_id)
            .bind(folder_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
        }
        if let Some(locator) = reconciliation.imap_locator.as_ref() {
            upsert_imap_message_locator(
                &mut transaction,
                &queued.connection_id,
                &queued.message_id,
                locator,
                completed_at_unix_seconds,
            )
            .await
            .map_err(map_operational_error)?;
        }
        let revision = if changed {
            current_revision
                .checked_add(1)
                .ok_or(MailMessageLocationPersistenceErrorV1::InvalidRow)?
        } else {
            current_revision
        };
        if changed {
            sqlx::query(
                "UPDATE makosh_data.mail_operational_messages \
                 SET projection_revision = $3, updated_at_unix_seconds = $4 \
                 WHERE connection_id = $1 AND message_id = $2",
            )
            .bind(&queued.connection_id)
            .bind(&queued.message_id)
            .bind(revision)
            .bind(completed_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
            crate::operational::refresh_thread(
                &mut transaction,
                &queued.connection_id,
                &provider_thread_id,
                completed_at_unix_seconds,
            )
            .await
            .map_err(map_operational_error)?;
            let mut affected_folders = old_folder_ids;
            affected_folders.extend(new_folder_ids);
            affected_folders.sort();
            affected_folders.dedup();
            for folder_id in &affected_folders {
                crate::operational::refresh_folder(
                    &mut transaction,
                    &queued.connection_id,
                    folder_id,
                    completed_at_unix_seconds,
                )
                .await
                .map_err(map_operational_error)?;
            }
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_message_location_operations \
             SET outcome = 2, completed_at_unix_seconds = $3, projection_revision = $4 \
             WHERE operation_id = $1 AND connection_id = $2 AND outcome = 1",
        )
        .bind(&queued.operation_id)
        .bind(&queued.connection_id)
        .bind(completed_at_unix_seconds)
        .bind(revision)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?
        .rows_affected();
        if updated != 1 {
            return Err(MailMessageLocationPersistenceErrorV1::InvalidRow);
        }
        transaction
            .commit()
            .await
            .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?;
        u64::try_from(revision).map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)
    }

    pub async fn complete_message_location_failure(
        &self,
        operation_id: &str,
        connection_id: &str,
        outcome: MailMessageLocationOperationOutcomeV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailMessageLocationPersistenceErrorV1> {
        let outcome_id = match outcome {
            MailMessageLocationOperationOutcomeV1::Rejected => 3,
            MailMessageLocationOperationOutcomeV1::Unsupported => 4,
            MailMessageLocationOperationOutcomeV1::OutcomeUnknown => 5,
            MailMessageLocationOperationOutcomeV1::Pending
            | MailMessageLocationOperationOutcomeV1::Succeeded => {
                return Err(MailMessageLocationPersistenceErrorV1::InvalidInput);
            }
        };
        if operation_id.is_empty() || connection_id.is_empty() || completed_at_unix_seconds <= 0 {
            return Err(MailMessageLocationPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_message_location_operations \
             SET outcome = $3, completed_at_unix_seconds = $4 \
             WHERE operation_id = $1 AND connection_id = $2 AND outcome = 1",
        )
        .bind(operation_id)
        .bind(connection_id)
        .bind(outcome_id)
        .bind(completed_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map_err(|_| MailMessageLocationPersistenceErrorV1::Database)?
        .rows_affected();
        if updated != 1 {
            return Err(MailMessageLocationPersistenceErrorV1::InvalidRow);
        }
        Ok(())
    }
}

fn validate_queued(
    queued: &MailQueuedMessageLocationCommandV1,
) -> Result<(), MailMessageLocationPersistenceErrorV1> {
    let command = decode_message_location_command(&queued.exact_command_bytes)
        .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidInput)?;
    let hash: [u8; 32] = Sha256::digest(&queued.exact_command_bytes).into();
    if hash != queued.request_sha256
        || queued.operation_id != command.operation_id
        || queued.connection_id != command.connection_id
        || queued.message_id != command.message_id
        || queued.kind != command.kind
        || queued.target_folder_id != command.target_folder_id
    {
        return Err(MailMessageLocationPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn queued_from_row(
    row: &PgRow,
) -> Result<MailQueuedMessageLocationCommandV1, MailMessageLocationPersistenceErrorV1> {
    let request_sha256 = row
        .try_get::<Vec<u8>, _>("request_sha256")
        .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?
        .try_into()
        .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?;
    let queued = MailQueuedMessageLocationCommandV1 {
        operation_id: row_string(row, "operation_id")?,
        connection_id: row_string(row, "connection_id")?,
        message_id: row_string(row, "message_id")?,
        kind: location_kind_from_id(row_i16(row, "location_kind")?)?,
        target_folder_id: row
            .try_get::<Option<String>, _>("target_folder_id")
            .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?,
        request_sha256,
        exact_command_bytes: row
            .try_get::<Vec<u8>, _>("exact_command_bytes")
            .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?,
    };
    validate_queued(&queued)?;
    Ok(queued)
}

fn status_from_row(
    row: &PgRow,
) -> Result<MailMessageLocationOperationStatusV1, MailMessageLocationPersistenceErrorV1> {
    let status = MailMessageLocationOperationStatusV1 {
        operation_id: row_string(row, "operation_id")?,
        connection_id: row_string(row, "connection_id")?,
        message_id: row_string(row, "message_id")?,
        kind: location_kind_from_id(row_i16(row, "location_kind")?)?,
        target_folder_id: row
            .try_get::<Option<String>, _>("target_folder_id")
            .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?,
        outcome: outcome_from_id(row_i16(row, "outcome")?)?,
        requested_at_unix_seconds: row_i64(row, "requested_at_unix_seconds")?,
        completed_at_unix_seconds: row
            .try_get::<Option<i64>, _>("completed_at_unix_seconds")
            .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?,
        projection_revision: row
            .try_get::<Option<i64>, _>("projection_revision")
            .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?
            .map(u64::try_from)
            .transpose()
            .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?,
    };
    validate_message_location_status(&status)
        .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?;
    Ok(status)
}

fn folder_from_row(
    row: &PgRow,
) -> Result<MailOperationalFolderSnapshotV1, MailMessageLocationPersistenceErrorV1> {
    Ok(MailOperationalFolderSnapshotV1 {
        folder_id: row_string(row, "folder_id")?,
        display_name: row_string(row, "display_name")?,
        kind: folder_kind_from_id(row_i16(row, "kind")?)?,
    })
}

fn locator_from_row(
    row: &PgRow,
) -> Result<MailImapMessageLocatorV1, MailMessageLocationPersistenceErrorV1> {
    let locator = MailImapMessageLocatorV1 {
        mailbox_id: row_string(row, "mailbox_id")?,
        uid_validity: u32::try_from(row_i64(row, "uid_validity")?)
            .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?,
        uid: u32::try_from(row_i64(row, "uid")?)
            .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?,
    };
    validate_imap_locator(&locator)
        .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)?;
    Ok(locator)
}

const fn location_kind_id(kind: MailMessageLocationKindV1) -> i16 {
    match kind {
        MailMessageLocationKindV1::Archive => 1,
        MailMessageLocationKindV1::Trash => 2,
        MailMessageLocationKindV1::Restore => 3,
        MailMessageLocationKindV1::Move => 4,
    }
}

fn location_kind_from_id(
    value: i16,
) -> Result<MailMessageLocationKindV1, MailMessageLocationPersistenceErrorV1> {
    match value {
        1 => Ok(MailMessageLocationKindV1::Archive),
        2 => Ok(MailMessageLocationKindV1::Trash),
        3 => Ok(MailMessageLocationKindV1::Restore),
        4 => Ok(MailMessageLocationKindV1::Move),
        _ => Err(MailMessageLocationPersistenceErrorV1::InvalidRow),
    }
}

fn outcome_from_id(
    value: i16,
) -> Result<MailMessageLocationOperationOutcomeV1, MailMessageLocationPersistenceErrorV1> {
    match value {
        1 => Ok(MailMessageLocationOperationOutcomeV1::Pending),
        2 => Ok(MailMessageLocationOperationOutcomeV1::Succeeded),
        3 => Ok(MailMessageLocationOperationOutcomeV1::Rejected),
        4 => Ok(MailMessageLocationOperationOutcomeV1::Unsupported),
        5 => Ok(MailMessageLocationOperationOutcomeV1::OutcomeUnknown),
        _ => Err(MailMessageLocationPersistenceErrorV1::InvalidRow),
    }
}

const fn folder_kind_id(kind: MailFolderKindV1) -> i16 {
    match kind {
        MailFolderKindV1::Inbox => 1,
        MailFolderKindV1::Sent => 2,
        MailFolderKindV1::Drafts => 3,
        MailFolderKindV1::Trash => 4,
        MailFolderKindV1::Spam => 5,
        MailFolderKindV1::Archive => 6,
        MailFolderKindV1::ProviderLabel => 7,
    }
}

fn folder_kind_from_id(
    value: i16,
) -> Result<MailFolderKindV1, MailMessageLocationPersistenceErrorV1> {
    match value {
        1 => Ok(MailFolderKindV1::Inbox),
        2 => Ok(MailFolderKindV1::Sent),
        3 => Ok(MailFolderKindV1::Drafts),
        4 => Ok(MailFolderKindV1::Trash),
        5 => Ok(MailFolderKindV1::Spam),
        6 => Ok(MailFolderKindV1::Archive),
        7 => Ok(MailFolderKindV1::ProviderLabel),
        _ => Err(MailMessageLocationPersistenceErrorV1::InvalidRow),
    }
}

fn row_string(row: &PgRow, column: &str) -> Result<String, MailMessageLocationPersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)
}

fn row_i16(row: &PgRow, column: &str) -> Result<i16, MailMessageLocationPersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)
}

fn row_i64(row: &PgRow, column: &str) -> Result<i64, MailMessageLocationPersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| MailMessageLocationPersistenceErrorV1::InvalidRow)
}

fn map_operational_error(
    error: MailDurablePersistenceError,
) -> MailMessageLocationPersistenceErrorV1 {
    match error {
        MailDurablePersistenceError::Database => MailMessageLocationPersistenceErrorV1::Database,
        _ => MailMessageLocationPersistenceErrorV1::InvalidRow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_is_owner_local_and_separates_reversible_location_from_delete() {
        assert!(MAIL_SCHEMA_V15.contains("mail_message_location_operations"));
        assert!(MAIL_SCHEMA_V15.contains("exact_command_bytes"));
        assert!(MAIL_SCHEMA_V15.contains("target_folder_id"));
        assert!(!MAIL_SCHEMA_V15.contains("permanent"));
        assert!(!MAIL_SCHEMA_V15.contains("communications"));
        assert!(!MAIL_SCHEMA_V15.contains("credential"));
    }
}
