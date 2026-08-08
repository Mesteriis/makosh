//! Mail-owned purpose-specific credential binding state.

use makosh_mail_api::account::{
    MailBindCredentialRequestV1, MailCredentialBindingReceiptV1, MailCredentialBindingStateV1,
    MailCredentialPurposeV1, validate_bind_credential_request,
};
use sqlx::Row;

use crate::{MailDurablePersistence, MailDurablePersistenceError};

pub const MAIL_SCHEMA_V7: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.mail_account_credential_bindings (
    connection_id TEXT NOT NULL,
    configuration_instance_id TEXT NOT NULL,
    purpose SMALLINT NOT NULL CHECK (purpose IN (1, 2)),
    credential_revision BIGINT NOT NULL CHECK (credential_revision > 0),
    binding_revision BIGINT NOT NULL CHECK (binding_revision > 0),
    state SMALLINT NOT NULL CHECK (state IN (2, 3, 4, 5)),
    applied_runtime_generation BIGINT,
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    PRIMARY KEY (connection_id, purpose)
);
"#;

pub const MAIL_ICLOUD_CARDDAV_CREDENTIAL_SCHEMA_V1: &str =
    include_str!("../migrations/0029_icloud_carddav_credential_bindings.sql");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailCredentialBindingV1 {
    pub connection_id: String,
    pub configuration_instance_id: String,
    pub purpose: MailCredentialPurposeV1,
    pub credential_revision: u64,
    pub binding_revision: u64,
    pub state: MailCredentialBindingStateV1,
    pub applied_runtime_generation: Option<u64>,
}

impl MailDurablePersistence {
    pub async fn bind_account_credential(
        &self,
        request: &MailBindCredentialRequestV1,
        configuration_instance_id: &str,
        updated_at_unix_seconds: i64,
    ) -> Result<MailCredentialBindingReceiptV1, MailDurablePersistenceError> {
        validate_bind_credential_request(request)
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
        if configuration_instance_id.trim().is_empty() || updated_at_unix_seconds <= 0 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let purpose = purpose_to_i16(request.purpose)?;
        let address_book = request.purpose == MailCredentialPurposeV1::IcloudCardDavPassword;
        let row = if request.expected_binding_revision == 0 {
            let statement = if address_book {
                "INSERT INTO makosh_data.mail_icloud_carddav_credential_bindings (
                    connection_id, configuration_instance_id, purpose, credential_revision,
                    binding_revision, state, updated_at_unix_seconds
                 ) VALUES ($1, $2, $3, $4, 1, 2, $5)
                 ON CONFLICT (connection_id) DO NOTHING
                 RETURNING connection_id, purpose, binding_revision, state"
            } else {
                "INSERT INTO makosh_data.mail_account_credential_bindings (
                    connection_id, configuration_instance_id, purpose, credential_revision,
                    binding_revision, state, updated_at_unix_seconds
                 ) VALUES ($1, $2, $3, $4, 1, 2, $5)
                 ON CONFLICT (connection_id, purpose) DO NOTHING
                 RETURNING connection_id, purpose, binding_revision, state"
            };
            sqlx::query(statement)
                .bind(&request.connection_id)
                .bind(configuration_instance_id)
                .bind(purpose)
                .bind(to_i64(request.credential_revision)?)
                .bind(updated_at_unix_seconds)
                .fetch_optional(&self.pool)
                .await
        } else {
            let statement = if address_book {
                "UPDATE makosh_data.mail_icloud_carddav_credential_bindings
                 SET credential_revision = $1, binding_revision = binding_revision + 1,
                     state = 2, applied_runtime_generation = NULL,
                     updated_at_unix_seconds = $2
                 WHERE connection_id = $3 AND configuration_instance_id = $4
                   AND purpose = $5 AND binding_revision = $6 AND state NOT IN (5)
                 RETURNING connection_id, purpose, binding_revision, state"
            } else {
                "UPDATE makosh_data.mail_account_credential_bindings
                 SET credential_revision = $1, binding_revision = binding_revision + 1,
                     state = 2, applied_runtime_generation = NULL,
                     updated_at_unix_seconds = $2
                 WHERE connection_id = $3 AND configuration_instance_id = $4
                   AND purpose = $5 AND binding_revision = $6 AND state NOT IN (5)
                 RETURNING connection_id, purpose, binding_revision, state"
            };
            sqlx::query(statement)
                .bind(to_i64(request.credential_revision)?)
                .bind(updated_at_unix_seconds)
                .bind(&request.connection_id)
                .bind(configuration_instance_id)
                .bind(purpose)
                .bind(to_i64(request.expected_binding_revision)?)
                .fetch_optional(&self.pool)
                .await
        }
        .map_err(|_| MailDurablePersistenceError::Database)?
        .ok_or(MailDurablePersistenceError::InvalidRow)?;
        Ok(MailCredentialBindingReceiptV1 {
            connection_id: row
                .try_get("connection_id")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            purpose: purpose_from_i16(
                row.try_get("purpose")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            )?,
            binding_revision: row_u64(&row, "binding_revision")?,
            state: state_from_i16(
                row.try_get("state")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
            )?,
        })
    }

    pub async fn account_credential_binding(
        &self,
        connection_id: &str,
        purpose: MailCredentialPurposeV1,
    ) -> Result<Option<MailCredentialBindingV1>, MailDurablePersistenceError> {
        if connection_id.trim().is_empty() || !purpose.bindable_by_client() {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let statement = if purpose == MailCredentialPurposeV1::IcloudCardDavPassword {
            "SELECT connection_id, configuration_instance_id, purpose, credential_revision,
                    binding_revision, state, applied_runtime_generation
             FROM makosh_data.mail_icloud_carddav_credential_bindings
             WHERE connection_id = $1 AND purpose = $2"
        } else {
            "SELECT connection_id, configuration_instance_id, purpose, credential_revision,
                    binding_revision, state, applied_runtime_generation
             FROM makosh_data.mail_account_credential_bindings
             WHERE connection_id = $1 AND purpose = $2"
        };
        sqlx::query(statement)
            .bind(connection_id)
            .bind(purpose_to_i16(purpose)?)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?
            .map(decode_binding)
            .transpose()
    }

    pub async fn account_credential_bindings(
        &self,
        connection_id: &str,
    ) -> Result<Vec<MailCredentialBindingV1>, MailDurablePersistenceError> {
        if connection_id.trim().is_empty() {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        sqlx::query(
            "SELECT connection_id, configuration_instance_id, purpose, credential_revision,
                    binding_revision, state, applied_runtime_generation
             FROM makosh_data.mail_account_credential_bindings
             WHERE connection_id = $1
             UNION ALL
             SELECT connection_id, configuration_instance_id, purpose, credential_revision,
                    binding_revision, state, applied_runtime_generation
             FROM makosh_data.mail_icloud_carddav_credential_bindings
             WHERE connection_id = $1
             ORDER BY purpose",
        )
        .bind(connection_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?
        .into_iter()
        .map(decode_binding)
        .collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn mark_account_credential_active(
        &self,
        connection_id: &str,
        configuration_instance_id: &str,
        purpose: MailCredentialPurposeV1,
        binding_revision: u64,
        credential_revision: u64,
        runtime_generation: u64,
        updated_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if connection_id.trim().is_empty()
            || configuration_instance_id.trim().is_empty()
            || !purpose.bindable_by_client()
            || binding_revision == 0
            || credential_revision == 0
            || runtime_generation == 0
            || updated_at_unix_seconds <= 0
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let statement = if purpose == MailCredentialPurposeV1::IcloudCardDavPassword {
            "UPDATE makosh_data.mail_icloud_carddav_credential_bindings
             SET state = 3, applied_runtime_generation = $1, updated_at_unix_seconds = $2
             WHERE connection_id = $3 AND configuration_instance_id = $4
               AND purpose = $5 AND binding_revision = $6 AND credential_revision = $7
               AND state IN (2, 3)"
        } else {
            "UPDATE makosh_data.mail_account_credential_bindings
             SET state = 3, applied_runtime_generation = $1, updated_at_unix_seconds = $2
             WHERE connection_id = $3 AND configuration_instance_id = $4
               AND purpose = $5 AND binding_revision = $6 AND credential_revision = $7
               AND state IN (2, 3)"
        };
        let result = sqlx::query(statement)
            .bind(to_i64(runtime_generation)?)
            .bind(updated_at_unix_seconds)
            .bind(connection_id)
            .bind(configuration_instance_id)
            .bind(purpose_to_i16(purpose)?)
            .bind(to_i64(binding_revision)?)
            .bind(to_i64(credential_revision)?)
            .execute(&self.pool)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        (result.rows_affected() == 1)
            .then_some(())
            .ok_or(MailDurablePersistenceError::InvalidRow)
    }
}

fn decode_binding(
    row: sqlx::postgres::PgRow,
) -> Result<MailCredentialBindingV1, MailDurablePersistenceError> {
    Ok(MailCredentialBindingV1 {
        connection_id: row
            .try_get("connection_id")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        configuration_instance_id: row
            .try_get("configuration_instance_id")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        purpose: purpose_from_i16(
            row.try_get("purpose")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        )?,
        credential_revision: row_u64(&row, "credential_revision")?,
        binding_revision: row_u64(&row, "binding_revision")?,
        state: state_from_i16(
            row.try_get("state")
                .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
        )?,
        applied_runtime_generation: row
            .try_get::<Option<i64>, _>("applied_runtime_generation")
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?
            .map(u64::try_from)
            .transpose()
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
    })
}

fn purpose_to_i16(purpose: MailCredentialPurposeV1) -> Result<i16, MailDurablePersistenceError> {
    match purpose {
        MailCredentialPurposeV1::ImapPassword => Ok(1),
        MailCredentialPurposeV1::SmtpPassword => Ok(2),
        MailCredentialPurposeV1::IcloudCardDavPassword => Ok(5),
        MailCredentialPurposeV1::GmailAccessToken
        | MailCredentialPurposeV1::GmailRefreshCredential => {
            Err(MailDurablePersistenceError::InvalidRow)
        }
    }
}

fn purpose_from_i16(purpose: i16) -> Result<MailCredentialPurposeV1, MailDurablePersistenceError> {
    match purpose {
        1 => Ok(MailCredentialPurposeV1::ImapPassword),
        2 => Ok(MailCredentialPurposeV1::SmtpPassword),
        5 => Ok(MailCredentialPurposeV1::IcloudCardDavPassword),
        _ => Err(MailDurablePersistenceError::InvalidRow),
    }
}

fn state_from_i16(state: i16) -> Result<MailCredentialBindingStateV1, MailDurablePersistenceError> {
    match state {
        2 => Ok(MailCredentialBindingStateV1::PendingRestart),
        3 => Ok(MailCredentialBindingStateV1::Active),
        4 => Ok(MailCredentialBindingStateV1::Retired),
        5 => Ok(MailCredentialBindingStateV1::Deleted),
        _ => Err(MailDurablePersistenceError::InvalidRow),
    }
}

fn row_u64(row: &sqlx::postgres::PgRow, field: &str) -> Result<u64, MailDurablePersistenceError> {
    u64::try_from(
        row.try_get::<i64, _>(field)
            .map_err(|_| MailDurablePersistenceError::InvalidRow)?,
    )
    .map_err(|_| MailDurablePersistenceError::InvalidRow)
}

fn to_i64(value: u64) -> Result<i64, MailDurablePersistenceError> {
    i64::try_from(value).map_err(|_| MailDurablePersistenceError::InvalidRow)
}
