//! Zulip-owned credential revision binding and account retirement state.

use makosh_zulip_api::account::{
    ZulipAccountLifecycleCommandV1, ZulipAccountLifecycleReceiptV1, ZulipCredentialBindingStateV1,
    validate_account_lifecycle_command,
};
use sqlx::Row;

use crate::{ZulipDurablePersistence, ZulipDurablePersistenceError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipCredentialBindingV1 {
    pub account_id: String,
    pub configuration_instance_id: String,
    pub credential_revision: u64,
    pub binding_revision: u64,
    pub state: ZulipCredentialBindingStateV1,
    pub applied_runtime_generation: Option<u64>,
}

impl ZulipDurablePersistence {
    pub async fn apply_account_lifecycle(
        &self,
        command: &ZulipAccountLifecycleCommandV1,
        configuration_instance_id: &str,
        updated_at_unix_seconds: i64,
    ) -> Result<ZulipAccountLifecycleReceiptV1, ZulipDurablePersistenceError> {
        validate_account_lifecycle_command(command)
            .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?;
        if configuration_instance_id.trim().is_empty() || updated_at_unix_seconds <= 0 {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        let row = match command {
            ZulipAccountLifecycleCommandV1::BindCredential {
                account_id,
                expected_binding_revision: 0,
                credential_revision,
            } => {
                sqlx::query(
                    "INSERT INTO makosh_data.zulip_account_credential_bindings \
                     (account_id, configuration_instance_id, credential_revision, binding_revision, \
                      state, updated_at_unix_seconds) \
                     VALUES ($1, $2, $3, 1, 2, $4) ON CONFLICT (account_id) DO NOTHING \
                     RETURNING account_id, binding_revision, state",
                )
                .bind(account_id)
                .bind(configuration_instance_id)
                .bind(i64::try_from(*credential_revision).map_err(|_| {
                    ZulipDurablePersistenceError::InvalidRow
                })?)
                .bind(updated_at_unix_seconds)
                .fetch_optional(&self.pool)
                .await
            }
            ZulipAccountLifecycleCommandV1::BindCredential {
                account_id,
                expected_binding_revision,
                credential_revision,
            } => {
                sqlx::query(
                    "UPDATE makosh_data.zulip_account_credential_bindings \
                     SET credential_revision = $1, binding_revision = binding_revision + 1, \
                         state = 2, applied_runtime_generation = NULL, \
                         updated_at_unix_seconds = $2 \
                     WHERE account_id = $3 AND configuration_instance_id = $4 \
                       AND binding_revision = $5 \
                     RETURNING account_id, binding_revision, state",
                )
                .bind(i64::try_from(*credential_revision).map_err(|_| {
                    ZulipDurablePersistenceError::InvalidRow
                })?)
                .bind(updated_at_unix_seconds)
                .bind(account_id)
                .bind(configuration_instance_id)
                .bind(i64::try_from(*expected_binding_revision).map_err(|_| {
                    ZulipDurablePersistenceError::InvalidRow
                })?)
                .fetch_optional(&self.pool)
                .await
            }
            ZulipAccountLifecycleCommandV1::RetireAccount {
                account_id,
                expected_binding_revision,
            } => {
                sqlx::query(
                    "UPDATE makosh_data.zulip_account_credential_bindings \
                     SET binding_revision = binding_revision + 1, state = 4, \
                         applied_runtime_generation = NULL, updated_at_unix_seconds = $1 \
                     WHERE account_id = $2 AND configuration_instance_id = $3 \
                       AND binding_revision = $4 \
                     RETURNING account_id, binding_revision, state",
                )
                .bind(updated_at_unix_seconds)
                .bind(account_id)
                .bind(configuration_instance_id)
                .bind(i64::try_from(*expected_binding_revision).map_err(|_| {
                    ZulipDurablePersistenceError::InvalidRow
                })?)
                .fetch_optional(&self.pool)
                .await
            }
        }
        .map_err(|_| ZulipDurablePersistenceError::Database)?
        .ok_or(ZulipDurablePersistenceError::InvalidRow)?;
        Ok(ZulipAccountLifecycleReceiptV1 {
            account_id: row
                .try_get("account_id")
                .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
            binding_revision: row_u64(&row, "binding_revision")?,
            state: state_from_i16(
                row.try_get("state")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
            )?,
        })
    }

    pub async fn credential_binding(
        &self,
        account_id: &str,
    ) -> Result<Option<ZulipCredentialBindingV1>, ZulipDurablePersistenceError> {
        if account_id.trim().is_empty() {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        sqlx::query(
            "SELECT account_id, configuration_instance_id, credential_revision, binding_revision, \
             state, applied_runtime_generation \
             FROM makosh_data.zulip_account_credential_bindings WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?
        .map(|row| {
            Ok(ZulipCredentialBindingV1 {
                account_id: row
                    .try_get("account_id")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                configuration_instance_id: row
                    .try_get("configuration_instance_id")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                credential_revision: row_u64(&row, "credential_revision")?,
                binding_revision: row_u64(&row, "binding_revision")?,
                state: state_from_i16(
                    row.try_get("state")
                        .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
                )?,
                applied_runtime_generation: row
                    .try_get::<Option<i64>, _>("applied_runtime_generation")
                    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?
                    .map(|value| {
                        u64::try_from(value).map_err(|_| ZulipDurablePersistenceError::InvalidRow)
                    })
                    .transpose()?,
            })
        })
        .transpose()
    }

    pub async fn mark_credential_binding_active(
        &self,
        account_id: &str,
        configuration_instance_id: &str,
        binding_revision: u64,
        credential_revision: u64,
        runtime_generation: u64,
        updated_at_unix_seconds: i64,
    ) -> Result<(), ZulipDurablePersistenceError> {
        if account_id.trim().is_empty()
            || configuration_instance_id.trim().is_empty()
            || binding_revision == 0
            || credential_revision == 0
            || runtime_generation == 0
            || updated_at_unix_seconds <= 0
        {
            return Err(ZulipDurablePersistenceError::InvalidRow);
        }
        let result = sqlx::query(
            "UPDATE makosh_data.zulip_account_credential_bindings \
             SET state = 3, applied_runtime_generation = $1, updated_at_unix_seconds = $2 \
             WHERE account_id = $3 AND configuration_instance_id = $4 \
               AND binding_revision = $5 AND credential_revision = $6 AND state IN (2, 3)",
        )
        .bind(
            i64::try_from(runtime_generation)
                .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
        )
        .bind(updated_at_unix_seconds)
        .bind(account_id)
        .bind(configuration_instance_id)
        .bind(
            i64::try_from(binding_revision)
                .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
        )
        .bind(
            i64::try_from(credential_revision)
                .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
        )
        .execute(&self.pool)
        .await
        .map_err(|_| ZulipDurablePersistenceError::Database)?;
        (result.rows_affected() == 1)
            .then_some(())
            .ok_or(ZulipDurablePersistenceError::InvalidRow)
    }
}

fn row_u64(row: &sqlx::postgres::PgRow, field: &str) -> Result<u64, ZulipDurablePersistenceError> {
    u64::try_from(
        row.try_get::<i64, _>(field)
            .map_err(|_| ZulipDurablePersistenceError::InvalidRow)?,
    )
    .map_err(|_| ZulipDurablePersistenceError::InvalidRow)
}

fn state_from_i16(
    state: i16,
) -> Result<ZulipCredentialBindingStateV1, ZulipDurablePersistenceError> {
    match state {
        2 => Ok(ZulipCredentialBindingStateV1::PendingRestart),
        3 => Ok(ZulipCredentialBindingStateV1::Active),
        4 => Ok(ZulipCredentialBindingStateV1::Retired),
        _ => Err(ZulipDurablePersistenceError::InvalidRow),
    }
}
