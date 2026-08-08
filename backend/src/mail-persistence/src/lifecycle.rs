//! Mail-owned durable account lifecycle journal and tombstone.

use makosh_mail_api::{
    account::MailCredentialPurposeV1,
    account_lifecycle::{
        MailAccountLifecycleActionV1, MailAccountLifecycleCommandV1, MailAccountLifecycleReceiptV1,
        MailAccountLifecycleStateV1, MailCredentialLifecycleProgressV1,
        MailCredentialLifecycleStateV1, aggregate_lifecycle_state, validate_lifecycle_command,
    },
};
use sqlx::{Postgres, Row, Transaction};

use crate::{MailDurablePersistence, MailDurablePersistenceError};

pub const MAIL_SCHEMA_V8: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.mail_account_lifecycle_operations (
    operation_id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    configuration_instance_id TEXT NOT NULL,
    action SMALLINT NOT NULL CHECK (action IN (1, 2)),
    lifecycle_revision BIGINT NOT NULL CHECK (lifecycle_revision > 0),
    state SMALLINT NOT NULL CHECK (state IN (1, 2, 3, 4)),
    requested_at_unix_seconds BIGINT NOT NULL CHECK (requested_at_unix_seconds > 0),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    UNIQUE (connection_id, lifecycle_revision)
);

CREATE TABLE IF NOT EXISTS makosh_data.mail_account_lifecycle_credentials (
    operation_id TEXT NOT NULL
        REFERENCES makosh_data.mail_account_lifecycle_operations (operation_id),
    purpose SMALLINT NOT NULL CHECK (purpose IN (1, 2, 3, 4)),
    binding_revision BIGINT,
    credential_revision BIGINT NOT NULL CHECK (credential_revision > 0),
    state SMALLINT NOT NULL CHECK (state IN (1, 2, 3, 4)),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    PRIMARY KEY (operation_id, purpose),
    CHECK (binding_revision IS NULL OR binding_revision > 0)
);

CREATE TABLE IF NOT EXISTS makosh_data.mail_account_tombstones (
    connection_id TEXT PRIMARY KEY,
    configuration_instance_id TEXT NOT NULL,
    operation_id TEXT NOT NULL UNIQUE
        REFERENCES makosh_data.mail_account_lifecycle_operations (operation_id),
    lifecycle_revision BIGINT NOT NULL CHECK (lifecycle_revision > 0),
    deleted_at_unix_seconds BIGINT NOT NULL CHECK (deleted_at_unix_seconds > 0)
);
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAccountLifecycleBeginV1 {
    pub receipt: MailAccountLifecycleReceiptV1,
    pub created: bool,
}

impl MailDurablePersistence {
    pub async fn begin_account_lifecycle(
        &self,
        command: &MailAccountLifecycleCommandV1,
        action: MailAccountLifecycleActionV1,
        configuration_instance_id: &str,
        requested_at_unix_seconds: i64,
    ) -> Result<MailAccountLifecycleBeginV1, MailDurablePersistenceError> {
        validate_lifecycle_command(command).map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        if configuration_instance_id.trim().is_empty() || requested_at_unix_seconds <= 0 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        if let Some(existing) = self
            .account_lifecycle_receipt(&command.connection_id, &command.operation_id)
            .await?
        {
            return (existing.action == action)
                .then_some(MailAccountLifecycleBeginV1 {
                    receipt: existing,
                    created: false,
                })
                .ok_or(MailDurablePersistenceError::InvalidRow);
        }

        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let tombstoned = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1
                FROM makosh_data.mail_account_tombstones
                WHERE connection_id = $1
             )",
        )
        .bind(&command.connection_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if tombstoned {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let current_revision = sqlx::query(
            "SELECT lifecycle_revision
             FROM makosh_data.mail_account_lifecycle_operations
             WHERE connection_id = $1
             ORDER BY lifecycle_revision DESC
             LIMIT 1
             FOR UPDATE",
        )
        .bind(&command.connection_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?
        .map(|row| row_u64(&row, "lifecycle_revision"))
        .transpose()?
        .unwrap_or(0);
        if current_revision != command.expected_lifecycle_revision {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let lifecycle_revision = current_revision
            .checked_add(1)
            .ok_or(MailDurablePersistenceError::InvalidRow)?;
        let credentials = lifecycle_credentials(&mut transaction, &command.connection_id).await?;
        let state = aggregate_lifecycle_state(&credentials);
        sqlx::query(
            "INSERT INTO makosh_data.mail_account_lifecycle_operations (
                operation_id, connection_id, configuration_instance_id, action,
                lifecycle_revision, state, requested_at_unix_seconds, updated_at_unix_seconds
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
        )
        .bind(&command.operation_id)
        .bind(&command.connection_id)
        .bind(configuration_instance_id)
        .bind(action_to_i16(action))
        .bind(to_i64(lifecycle_revision)?)
        .bind(lifecycle_state_to_i16(state))
        .bind(requested_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        for progress in &credentials {
            let statement = if progress.purpose == MailCredentialPurposeV1::IcloudCardDavPassword {
                "INSERT INTO makosh_data.mail_icloud_carddav_lifecycle_credentials (
                    operation_id, purpose, binding_revision, credential_revision,
                    state, updated_at_unix_seconds
                 ) VALUES ($1, $2, $3, $4, $5, $6)"
            } else {
                "INSERT INTO makosh_data.mail_account_lifecycle_credentials (
                    operation_id, purpose, binding_revision, credential_revision,
                    state, updated_at_unix_seconds
                 ) VALUES ($1, $2, $3, $4, $5, $6)"
            };
            sqlx::query(statement)
                .bind(&command.operation_id)
                .bind(purpose_to_i16(progress.purpose))
                .bind(progress.binding_revision.map(to_i64).transpose()?)
                .bind(to_i64(progress.credential_revision)?)
                .bind(credential_state_to_i16(progress.state))
                .bind(requested_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
        }
        if state == MailAccountLifecycleStateV1::Completed
            && action == MailAccountLifecycleActionV1::Delete
        {
            insert_account_tombstone(
                &mut transaction,
                command,
                configuration_instance_id,
                lifecycle_revision,
                requested_at_unix_seconds,
            )
            .await?;
        }
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        Ok(MailAccountLifecycleBeginV1 {
            receipt: MailAccountLifecycleReceiptV1 {
                operation_id: command.operation_id.clone(),
                connection_id: command.connection_id.clone(),
                action,
                lifecycle_revision,
                state,
                credentials,
            },
            created: true,
        })
    }

    pub async fn record_account_lifecycle_progress(
        &self,
        connection_id: &str,
        operation_id: &str,
        purpose: MailCredentialPurposeV1,
        state: MailCredentialLifecycleStateV1,
        updated_at_unix_seconds: i64,
    ) -> Result<MailAccountLifecycleReceiptV1, MailDurablePersistenceError> {
        if connection_id.trim().is_empty()
            || operation_id.trim().is_empty()
            || updated_at_unix_seconds <= 0
            || state == MailCredentialLifecycleStateV1::Pending
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        let operation = sqlx::query(
            "SELECT configuration_instance_id, action, lifecycle_revision
             FROM makosh_data.mail_account_lifecycle_operations
             WHERE operation_id = $1 AND connection_id = $2
             FOR UPDATE",
        )
        .bind(operation_id)
        .bind(connection_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?
        .ok_or(MailDurablePersistenceError::InvalidRow)?;
        let lifecycle_statement = if purpose == MailCredentialPurposeV1::IcloudCardDavPassword {
            "UPDATE makosh_data.mail_icloud_carddav_lifecycle_credentials
             SET state = $1, updated_at_unix_seconds = $2
             WHERE operation_id = $3 AND purpose = $4 AND state IN (1, 4)"
        } else {
            "UPDATE makosh_data.mail_account_lifecycle_credentials
             SET state = $1, updated_at_unix_seconds = $2
             WHERE operation_id = $3 AND purpose = $4 AND state IN (1, 4)"
        };
        let result = sqlx::query(lifecycle_statement)
            .bind(credential_state_to_i16(state))
            .bind(updated_at_unix_seconds)
            .bind(operation_id)
            .bind(purpose_to_i16(purpose))
            .execute(&mut *transaction)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        if result.rows_affected() != 1 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let credentials =
            lifecycle_credentials_for_operation(&mut transaction, operation_id).await?;
        let lifecycle_state = aggregate_lifecycle_state(&credentials);
        sqlx::query(
            "UPDATE makosh_data.mail_account_lifecycle_operations
             SET state = $1, updated_at_unix_seconds = $2
             WHERE operation_id = $3",
        )
        .bind(lifecycle_state_to_i16(lifecycle_state))
        .bind(updated_at_unix_seconds)
        .bind(operation_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        let action = action_from_i16(
            operation
                .try_get("action")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        )?;
        if state == MailCredentialLifecycleStateV1::Completed && purpose.bindable_by_client() {
            let progress = credentials
                .iter()
                .find(|progress| progress.purpose == purpose)
                .ok_or(MailDurablePersistenceError::InvalidRow)?;
            let binding_statement = if purpose == MailCredentialPurposeV1::IcloudCardDavPassword {
                "UPDATE makosh_data.mail_icloud_carddav_credential_bindings
                 SET state = $1, applied_runtime_generation = NULL,
                     updated_at_unix_seconds = $2
                 WHERE connection_id = $3 AND purpose = $4
                   AND binding_revision = $5 AND credential_revision = $6"
            } else {
                "UPDATE makosh_data.mail_account_credential_bindings
                 SET state = $1, applied_runtime_generation = NULL,
                     updated_at_unix_seconds = $2
                 WHERE connection_id = $3 AND purpose = $4
                   AND binding_revision = $5 AND credential_revision = $6"
            };
            let result = sqlx::query(binding_statement)
                .bind(match action {
                    MailAccountLifecycleActionV1::Retire => 4_i16,
                    MailAccountLifecycleActionV1::Delete => 5_i16,
                })
                .bind(updated_at_unix_seconds)
                .bind(connection_id)
                .bind(purpose_to_i16(purpose))
                .bind(progress.binding_revision.map(to_i64).transpose()?)
                .bind(to_i64(progress.credential_revision)?)
                .execute(&mut *transaction)
                .await
                .map_err(|_| MailDurablePersistenceError::Database)?;
            if result.rows_affected() != 1 {
                return Err(MailDurablePersistenceError::InvalidRow);
            }
        }
        let lifecycle_revision = row_u64(&operation, "lifecycle_revision")?;
        let configuration_instance_id: String = operation
            .try_get("configuration_instance_id")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        if lifecycle_state == MailAccountLifecycleStateV1::Completed
            && action == MailAccountLifecycleActionV1::Delete
        {
            insert_account_tombstone_values(
                &mut transaction,
                connection_id,
                &configuration_instance_id,
                operation_id,
                lifecycle_revision,
                updated_at_unix_seconds,
            )
            .await?;
        }
        transaction
            .commit()
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        Ok(MailAccountLifecycleReceiptV1 {
            operation_id: operation_id.to_owned(),
            connection_id: connection_id.to_owned(),
            action,
            lifecycle_revision,
            state: lifecycle_state,
            credentials,
        })
    }

    pub async fn account_lifecycle_receipt(
        &self,
        connection_id: &str,
        operation_id: &str,
    ) -> Result<Option<MailAccountLifecycleReceiptV1>, MailDurablePersistenceError> {
        if connection_id.trim().is_empty() || operation_id.trim().is_empty() {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let Some(row) = sqlx::query(
            "SELECT action, lifecycle_revision, state
             FROM makosh_data.mail_account_lifecycle_operations
             WHERE operation_id = $1 AND connection_id = $2",
        )
        .bind(operation_id)
        .bind(connection_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?
        else {
            return Ok(None);
        };
        let credentials = lifecycle_credentials_for_pool(&self.pool, operation_id).await?;
        let receipt = MailAccountLifecycleReceiptV1 {
            operation_id: operation_id.to_owned(),
            connection_id: connection_id.to_owned(),
            action: action_from_i16(
                row.try_get("action")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            )?,
            lifecycle_revision: row_u64(&row, "lifecycle_revision")?,
            state: lifecycle_state_from_i16(
                row.try_get("state")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            )?,
            credentials,
        };
        (receipt.state == aggregate_lifecycle_state(&receipt.credentials))
            .then_some(Some(receipt))
            .ok_or(MailDurablePersistenceError::InvalidRow)
    }

    pub async fn latest_account_lifecycle(
        &self,
        connection_id: &str,
    ) -> Result<Option<MailAccountLifecycleReceiptV1>, MailDurablePersistenceError> {
        if connection_id.trim().is_empty() {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let operation_id = sqlx::query(
            "SELECT operation_id
             FROM makosh_data.mail_account_lifecycle_operations
             WHERE connection_id = $1
             ORDER BY lifecycle_revision DESC
             LIMIT 1",
        )
        .bind(connection_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?
        .map(|row| {
            row.try_get::<String, _>("operation_id")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)
        })
        .transpose()?;
        match operation_id {
            Some(operation_id) => {
                self.account_lifecycle_receipt(connection_id, &operation_id)
                    .await
            }
            None => Ok(None),
        }
    }

    pub async fn account_is_tombstoned(
        &self,
        connection_id: &str,
    ) -> Result<bool, MailDurablePersistenceError> {
        if connection_id.trim().is_empty() {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM makosh_data.mail_account_tombstones WHERE connection_id = $1
             )",
        )
        .bind(connection_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)
    }
}

async fn lifecycle_credentials(
    transaction: &mut Transaction<'_, Postgres>,
    connection_id: &str,
) -> Result<Vec<MailCredentialLifecycleProgressV1>, MailDurablePersistenceError> {
    let basic = sqlx::query(
        "SELECT purpose, binding_revision, credential_revision
         FROM makosh_data.mail_account_credential_bindings
         WHERE connection_id = $1 AND state NOT IN (5)
         UNION ALL
         SELECT purpose, binding_revision, credential_revision
         FROM makosh_data.mail_icloud_carddav_credential_bindings
         WHERE connection_id = $1 AND state NOT IN (5)
         ORDER BY purpose",
    )
    .bind(connection_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| MailDurablePersistenceError::Database)?;
    let mut credentials = basic
        .into_iter()
        .map(|row| {
            Ok(MailCredentialLifecycleProgressV1 {
                purpose: purpose_from_i16(
                    row.try_get("purpose")
                        .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                )?,
                state: MailCredentialLifecycleStateV1::Pending,
                binding_revision: Some(row_u64(&row, "binding_revision")?),
                credential_revision: row_u64(&row, "credential_revision")?,
            })
        })
        .collect::<Result<Vec<_>, MailDurablePersistenceError>>()?;
    if let Some(row) = sqlx::query(
        "SELECT access_token_revision, refresh_credential_revision
         FROM makosh_data.mail_gmail_oauth_credential_bindings
         WHERE connection_id = $1",
    )
    .bind(connection_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| MailDurablePersistenceError::Database)?
    {
        credentials.push(MailCredentialLifecycleProgressV1 {
            purpose: MailCredentialPurposeV1::GmailAccessToken,
            state: MailCredentialLifecycleStateV1::Pending,
            binding_revision: None,
            credential_revision: row_u64(&row, "access_token_revision")?,
        });
        credentials.push(MailCredentialLifecycleProgressV1 {
            purpose: MailCredentialPurposeV1::GmailRefreshCredential,
            state: MailCredentialLifecycleStateV1::Pending,
            binding_revision: None,
            credential_revision: row_u64(&row, "refresh_credential_revision")?,
        });
    }
    credentials.sort_by_key(|progress| progress.purpose);
    credentials.dedup_by_key(|progress| progress.purpose);
    Ok(credentials)
}

async fn lifecycle_credentials_for_operation(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: &str,
) -> Result<Vec<MailCredentialLifecycleProgressV1>, MailDurablePersistenceError> {
    let rows = sqlx::query(
        "SELECT purpose, state, binding_revision, credential_revision
         FROM makosh_data.mail_account_lifecycle_credentials
         WHERE operation_id = $1
         UNION ALL
         SELECT purpose, state, binding_revision, credential_revision
         FROM makosh_data.mail_icloud_carddav_lifecycle_credentials
         WHERE operation_id = $1
         ORDER BY purpose",
    )
    .bind(operation_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| MailDurablePersistenceError::Database)?;
    decode_credentials(rows)
}

async fn lifecycle_credentials_for_pool(
    pool: &sqlx::PgPool,
    operation_id: &str,
) -> Result<Vec<MailCredentialLifecycleProgressV1>, MailDurablePersistenceError> {
    let rows = sqlx::query(
        "SELECT purpose, state, binding_revision, credential_revision
         FROM makosh_data.mail_account_lifecycle_credentials
         WHERE operation_id = $1
         UNION ALL
         SELECT purpose, state, binding_revision, credential_revision
         FROM makosh_data.mail_icloud_carddav_lifecycle_credentials
         WHERE operation_id = $1
         ORDER BY purpose",
    )
    .bind(operation_id)
    .fetch_all(pool)
    .await
    .map_err(|_| MailDurablePersistenceError::Database)?;
    decode_credentials(rows)
}

fn decode_credentials(
    rows: Vec<sqlx::postgres::PgRow>,
) -> Result<Vec<MailCredentialLifecycleProgressV1>, MailDurablePersistenceError> {
    rows.into_iter()
        .map(|row| {
            Ok(MailCredentialLifecycleProgressV1 {
                purpose: purpose_from_i16(
                    row.try_get("purpose")
                        .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                )?,
                state: credential_state_from_i16(
                    row.try_get("state")
                        .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                )?,
                binding_revision: row
                    .try_get::<Option<i64>, _>("binding_revision")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?
                    .map(u64::try_from)
                    .transpose()
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
                credential_revision: row_u64(&row, "credential_revision")?,
            })
        })
        .collect()
}

async fn insert_account_tombstone(
    transaction: &mut Transaction<'_, Postgres>,
    command: &MailAccountLifecycleCommandV1,
    configuration_instance_id: &str,
    lifecycle_revision: u64,
    deleted_at_unix_seconds: i64,
) -> Result<(), MailDurablePersistenceError> {
    insert_account_tombstone_values(
        transaction,
        &command.connection_id,
        configuration_instance_id,
        &command.operation_id,
        lifecycle_revision,
        deleted_at_unix_seconds,
    )
    .await
}

async fn insert_account_tombstone_values(
    transaction: &mut Transaction<'_, Postgres>,
    connection_id: &str,
    configuration_instance_id: &str,
    operation_id: &str,
    lifecycle_revision: u64,
    deleted_at_unix_seconds: i64,
) -> Result<(), MailDurablePersistenceError> {
    sqlx::query(
        "INSERT INTO makosh_data.mail_account_tombstones (
            connection_id, configuration_instance_id, operation_id,
            lifecycle_revision, deleted_at_unix_seconds
         ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(connection_id)
    .bind(configuration_instance_id)
    .bind(operation_id)
    .bind(to_i64(lifecycle_revision)?)
    .bind(deleted_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| MailDurablePersistenceError::Database)?;
    Ok(())
}

fn action_to_i16(action: MailAccountLifecycleActionV1) -> i16 {
    match action {
        MailAccountLifecycleActionV1::Retire => 1,
        MailAccountLifecycleActionV1::Delete => 2,
    }
}

fn action_from_i16(
    action: i16,
) -> Result<MailAccountLifecycleActionV1, MailDurablePersistenceError> {
    match action {
        1 => Ok(MailAccountLifecycleActionV1::Retire),
        2 => Ok(MailAccountLifecycleActionV1::Delete),
        _ => Err(MailDurablePersistenceError::InvalidRow),
    }
}

fn lifecycle_state_to_i16(state: MailAccountLifecycleStateV1) -> i16 {
    match state {
        MailAccountLifecycleStateV1::Pending => 1,
        MailAccountLifecycleStateV1::Completed => 2,
        MailAccountLifecycleStateV1::Rejected => 3,
        MailAccountLifecycleStateV1::OutcomeUnknown => 4,
    }
}

fn lifecycle_state_from_i16(
    state: i16,
) -> Result<MailAccountLifecycleStateV1, MailDurablePersistenceError> {
    match state {
        1 => Ok(MailAccountLifecycleStateV1::Pending),
        2 => Ok(MailAccountLifecycleStateV1::Completed),
        3 => Ok(MailAccountLifecycleStateV1::Rejected),
        4 => Ok(MailAccountLifecycleStateV1::OutcomeUnknown),
        _ => Err(MailDurablePersistenceError::InvalidRow),
    }
}

fn credential_state_to_i16(state: MailCredentialLifecycleStateV1) -> i16 {
    match state {
        MailCredentialLifecycleStateV1::Pending => 1,
        MailCredentialLifecycleStateV1::Completed => 2,
        MailCredentialLifecycleStateV1::Rejected => 3,
        MailCredentialLifecycleStateV1::OutcomeUnknown => 4,
    }
}

fn credential_state_from_i16(
    state: i16,
) -> Result<MailCredentialLifecycleStateV1, MailDurablePersistenceError> {
    match state {
        1 => Ok(MailCredentialLifecycleStateV1::Pending),
        2 => Ok(MailCredentialLifecycleStateV1::Completed),
        3 => Ok(MailCredentialLifecycleStateV1::Rejected),
        4 => Ok(MailCredentialLifecycleStateV1::OutcomeUnknown),
        _ => Err(MailDurablePersistenceError::InvalidRow),
    }
}

fn purpose_to_i16(purpose: MailCredentialPurposeV1) -> i16 {
    match purpose {
        MailCredentialPurposeV1::ImapPassword => 1,
        MailCredentialPurposeV1::SmtpPassword => 2,
        MailCredentialPurposeV1::GmailAccessToken => 3,
        MailCredentialPurposeV1::GmailRefreshCredential => 4,
        MailCredentialPurposeV1::IcloudCardDavPassword => 5,
    }
}

fn purpose_from_i16(purpose: i16) -> Result<MailCredentialPurposeV1, MailDurablePersistenceError> {
    match purpose {
        1 => Ok(MailCredentialPurposeV1::ImapPassword),
        2 => Ok(MailCredentialPurposeV1::SmtpPassword),
        3 => Ok(MailCredentialPurposeV1::GmailAccessToken),
        4 => Ok(MailCredentialPurposeV1::GmailRefreshCredential),
        5 => Ok(MailCredentialPurposeV1::IcloudCardDavPassword),
        _ => Err(MailDurablePersistenceError::InvalidRow),
    }
}

fn row_u64(row: &sqlx::postgres::PgRow, column: &str) -> Result<u64, MailDurablePersistenceError> {
    u64::try_from(
        row.try_get::<i64, _>(column)
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
    )
    .map_err(|_| MailDurablePersistenceError::InvalidRow)
}

fn to_i64(value: u64) -> Result<i64, MailDurablePersistenceError> {
    i64::try_from(value).map_err(|_| MailDurablePersistenceError::InvalidRow)
}
