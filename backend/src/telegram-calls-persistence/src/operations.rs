use makosh_telegram_calls_core::{
    TelegramCallCommand, TelegramCallCommandError, TelegramCallDirection,
    TelegramCallFailureCategory, TelegramCallOperation, TelegramCallOperationKind,
    TelegramCallOperationState, TelegramCallSession, TelegramProviderCallState,
    validate_call_command,
};
use sqlx::{Postgres, Row, Transaction, postgres::PgRow};

use crate::{
    TelegramCallsPersistence, TelegramCallsPersistenceError, as_i64, database_error, from_i64,
    optional_i64, optional_u64, session_from_row, validated_limit,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersistedCallOperation {
    pub operation: TelegramCallOperation,
    pub replayed: bool,
}

impl TelegramCallsPersistence {
    pub async fn accept_call_command(
        &self,
        command: &TelegramCallCommand,
        own_provider_user_id: Option<&str>,
        runtime_generation: u64,
        grant_epoch: u64,
        accepted_at_unix_seconds: u64,
    ) -> Result<PersistedCallOperation, TelegramCallsPersistenceError> {
        if runtime_generation == 0 || grant_epoch == 0 || accepted_at_unix_seconds == 0 {
            return Err(TelegramCallsPersistenceError::InvalidRequest(
                "runtime_fence",
            ));
        }
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        if let Some(existing) =
            load_operation_for_update(&mut transaction, command.operation_id()).await?
        {
            if existing.account_id != command.account_id()
                || existing.request_fingerprint_sha256 != command.fingerprint_sha256()
            {
                return Err(TelegramCallsPersistenceError::IdempotencyConflict);
            }
            transaction.commit().await.map_err(database_error)?;
            return Ok(PersistedCallOperation {
                operation: existing,
                replayed: true,
            });
        }

        let current_call = load_command_call_for_update(&mut transaction, command).await?;
        validate_call_command(command, current_call.as_ref(), own_provider_user_id)
            .map_err(command_validation_error)?;
        let operation = TelegramCallOperation::accepted(
            command,
            runtime_generation,
            grant_epoch,
            accepted_at_unix_seconds,
        );
        insert_operation(&mut transaction, &operation).await?;
        persist_operation_history(&mut transaction, &operation).await?;
        crate::realtime::persist_operation_event(&mut transaction, &operation).await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(PersistedCallOperation {
            operation,
            replayed: false,
        })
    }

    pub async fn claim_accepted_call_operations(
        &self,
        account_id: &str,
        runtime_generation: u64,
        grant_epoch: u64,
        now_unix_seconds: u64,
        limit: u32,
    ) -> Result<Vec<TelegramCallOperation>, TelegramCallsPersistenceError> {
        let limit = validated_limit(limit)?;
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let rows = sqlx::query(
            "SELECT operation_id, account_id, call_session_id, operation_kind, operation_state, \
             request_fingerprint_sha256, provider_user_id, requested_mute, runtime_generation, \
             grant_epoch, tdlib_call_id, revision, accepted_at_unix_seconds, \
             updated_at_unix_seconds, completed_at_unix_seconds, failure_category \
             FROM makosh_data.telegram_call_operations \
             WHERE account_id = $1 AND runtime_generation = $2 AND grant_epoch = $3 \
               AND (operation_state = 'accepted' OR ( \
                    operation_state = 'dispatching' AND operation_kind = 'set_local_mute' \
               )) \
             ORDER BY accepted_at_unix_seconds, operation_id \
             LIMIT $4 FOR UPDATE SKIP LOCKED",
        )
        .bind(account_id)
        .bind(as_i64(runtime_generation)?)
        .bind(as_i64(grant_epoch)?)
        .bind(i64::from(limit))
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?;

        let mut claimed = Vec::with_capacity(rows.len());
        for row in &rows {
            let mut operation = operation_from_row(row)?;
            if operation.state == TelegramCallOperationState::Accepted {
                let tdlib_call_id = operation.tdlib_call_id;
                transition_operation(
                    &mut transaction,
                    &mut operation,
                    TelegramCallOperationState::Dispatching,
                    None,
                    tdlib_call_id,
                    now_unix_seconds,
                )
                .await?;
            }
            claimed.push(operation);
        }
        transaction.commit().await.map_err(database_error)?;
        Ok(claimed)
    }

    pub async fn mark_call_operation_awaiting_provider(
        &self,
        account_id: &str,
        operation_id: &str,
        tdlib_call_id: Option<i32>,
        now_unix_seconds: u64,
    ) -> Result<TelegramCallOperation, TelegramCallsPersistenceError> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let mut operation = load_operation_for_update(&mut transaction, operation_id)
            .await?
            .ok_or(TelegramCallsPersistenceError::CommandConflict(
                "operation_id",
            ))?;
        if operation.account_id != account_id
            || operation.state != TelegramCallOperationState::Dispatching
            || tdlib_call_id.is_some_and(|value| value <= 0)
        {
            return Err(TelegramCallsPersistenceError::CommandConflict(
                "operation_state",
            ));
        }
        let tdlib_call_id = tdlib_call_id.or(operation.tdlib_call_id);
        transition_operation(
            &mut transaction,
            &mut operation,
            TelegramCallOperationState::AwaitingProvider,
            None,
            tdlib_call_id,
            now_unix_seconds,
        )
        .await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(operation)
    }

    pub async fn fail_call_operation(
        &self,
        account_id: &str,
        operation_id: &str,
        failure_category: TelegramCallFailureCategory,
        now_unix_seconds: u64,
    ) -> Result<TelegramCallOperation, TelegramCallsPersistenceError> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let mut operation = load_operation_for_update(&mut transaction, operation_id)
            .await?
            .ok_or(TelegramCallsPersistenceError::CommandConflict(
                "operation_id",
            ))?;
        if operation.account_id != account_id || operation.state.is_terminal() {
            return Err(TelegramCallsPersistenceError::CommandConflict(
                "operation_state",
            ));
        }
        let tdlib_call_id = operation.tdlib_call_id;
        transition_operation(
            &mut transaction,
            &mut operation,
            TelegramCallOperationState::Failed,
            Some(failure_category),
            tdlib_call_id,
            now_unix_seconds,
        )
        .await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(operation)
    }

    pub async fn complete_local_mute_operation(
        &self,
        account_id: &str,
        operation_id: &str,
        now_unix_seconds: u64,
    ) -> Result<TelegramCallOperation, TelegramCallsPersistenceError> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let mut operation = load_operation_for_update(&mut transaction, operation_id)
            .await?
            .ok_or(TelegramCallsPersistenceError::CommandConflict(
                "operation_id",
            ))?;
        if operation.account_id != account_id
            || operation.kind != TelegramCallOperationKind::SetLocalMute
            || operation.state != TelegramCallOperationState::Dispatching
        {
            return Err(TelegramCallsPersistenceError::CommandConflict(
                "operation_state",
            ));
        }
        let muted = operation
            .requested_mute
            .ok_or(TelegramCallsPersistenceError::InvalidRow)?;
        sqlx::query(
            "INSERT INTO makosh_data.telegram_call_local_mute ( \
             call_session_id, account_id, muted, operation_id, updated_at_unix_seconds \
             ) VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (call_session_id) DO UPDATE SET \
               account_id = EXCLUDED.account_id, muted = EXCLUDED.muted, \
               operation_id = EXCLUDED.operation_id, \
               updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds",
        )
        .bind(&operation.call_session_id)
        .bind(&operation.account_id)
        .bind(muted)
        .bind(&operation.operation_id)
        .bind(as_i64(now_unix_seconds)?)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        transition_operation(
            &mut transaction,
            &mut operation,
            TelegramCallOperationState::Completed,
            None,
            None,
            now_unix_seconds,
        )
        .await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(operation)
    }

    pub async fn reconcile_stale_call_operations(
        &self,
        account_id: &str,
        runtime_generation: u64,
        grant_epoch: u64,
        now_unix_seconds: u64,
    ) -> Result<u64, TelegramCallsPersistenceError> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let rows = sqlx::query(
            "SELECT operation_id, account_id, call_session_id, operation_kind, operation_state, \
             request_fingerprint_sha256, provider_user_id, requested_mute, runtime_generation, \
             grant_epoch, tdlib_call_id, revision, accepted_at_unix_seconds, \
             updated_at_unix_seconds, completed_at_unix_seconds, failure_category \
             FROM makosh_data.telegram_call_operations \
             WHERE account_id = $1 \
               AND operation_state IN ('accepted', 'dispatching', 'awaiting_provider') \
               AND (runtime_generation <> $2 OR grant_epoch <> $3) \
             ORDER BY operation_id FOR UPDATE",
        )
        .bind(account_id)
        .bind(as_i64(runtime_generation)?)
        .bind(as_i64(grant_epoch)?)
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?;
        let mut failed = 0_u64;
        for row in &rows {
            let mut operation = operation_from_row(row)?;
            let tdlib_call_id = operation.tdlib_call_id;
            let failure_category = match operation.state {
                TelegramCallOperationState::Accepted => TelegramCallFailureCategory::Permission,
                TelegramCallOperationState::Dispatching
                | TelegramCallOperationState::AwaitingProvider => {
                    TelegramCallFailureCategory::Unknown
                }
                TelegramCallOperationState::Completed | TelegramCallOperationState::Failed => {
                    return Err(TelegramCallsPersistenceError::InvalidRow);
                }
            };
            transition_operation(
                &mut transaction,
                &mut operation,
                TelegramCallOperationState::Failed,
                Some(failure_category),
                tdlib_call_id,
                now_unix_seconds,
            )
            .await?;
            failed = failed.saturating_add(1);
        }
        transaction.commit().await.map_err(database_error)?;
        Ok(failed)
    }

    pub async fn call_operation(
        &self,
        account_id: &str,
        operation_id: &str,
    ) -> Result<Option<TelegramCallOperation>, TelegramCallsPersistenceError> {
        let row = sqlx::query(
            "SELECT operation_id, account_id, call_session_id, operation_kind, operation_state, \
             request_fingerprint_sha256, provider_user_id, requested_mute, runtime_generation, \
             grant_epoch, tdlib_call_id, revision, accepted_at_unix_seconds, \
             updated_at_unix_seconds, completed_at_unix_seconds, failure_category \
             FROM makosh_data.telegram_call_operations \
             WHERE account_id = $1 AND operation_id = $2",
        )
        .bind(account_id)
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;
        row.as_ref().map(operation_from_row).transpose()
    }

    pub async fn list_call_operations(
        &self,
        account_id: &str,
        after_operation_id: &str,
        limit: u32,
    ) -> Result<Vec<TelegramCallOperation>, TelegramCallsPersistenceError> {
        let limit = validated_limit(limit)?;
        let rows = sqlx::query(
            "SELECT operation_id, account_id, call_session_id, operation_kind, operation_state, \
             request_fingerprint_sha256, provider_user_id, requested_mute, runtime_generation, \
             grant_epoch, tdlib_call_id, revision, accepted_at_unix_seconds, \
             updated_at_unix_seconds, completed_at_unix_seconds, failure_category \
             FROM makosh_data.telegram_call_operations \
             WHERE account_id = $1 AND ($2 = '' OR operation_id > $2) \
             ORDER BY operation_id LIMIT $3",
        )
        .bind(account_id)
        .bind(after_operation_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;
        rows.iter().map(operation_from_row).collect()
    }

    pub async fn local_mute(
        &self,
        account_id: &str,
        call_session_id: &str,
    ) -> Result<bool, TelegramCallsPersistenceError> {
        sqlx::query_scalar::<_, bool>(
            "SELECT muted FROM makosh_data.telegram_call_local_mute \
             WHERE account_id = $1 AND call_session_id = $2",
        )
        .bind(account_id)
        .bind(call_session_id)
        .fetch_optional(&self.pool)
        .await
        .map(|value| value.unwrap_or(false))
        .map_err(database_error)
    }

    pub async fn pending_outgoing_call_session_id(
        &self,
        account_id: &str,
        runtime_generation: u64,
        tdlib_call_id: i32,
        provider_user_id: &str,
    ) -> Result<Option<String>, TelegramCallsPersistenceError> {
        sqlx::query_scalar(
            "SELECT call_session_id FROM makosh_data.telegram_call_operations \
             WHERE account_id = $1 AND runtime_generation = $2 AND tdlib_call_id = $3 \
               AND provider_user_id = $4 AND operation_kind = 'initiate_audio' \
               AND operation_state IN ('dispatching', 'awaiting_provider') \
             ORDER BY accepted_at_unix_seconds DESC LIMIT 1",
        )
        .bind(account_id)
        .bind(as_i64(runtime_generation)?)
        .bind(tdlib_call_id)
        .bind(provider_user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)
    }
}

pub(crate) async fn reconcile_operations_for_call(
    transaction: &mut Transaction<'_, Postgres>,
    session: &TelegramCallSession,
    observed_at_unix_seconds: u64,
) -> Result<(), TelegramCallsPersistenceError> {
    let rows = sqlx::query(
        "SELECT operation_id, account_id, call_session_id, operation_kind, operation_state, \
         request_fingerprint_sha256, provider_user_id, requested_mute, runtime_generation, \
         grant_epoch, tdlib_call_id, revision, accepted_at_unix_seconds, \
         updated_at_unix_seconds, completed_at_unix_seconds, failure_category \
         FROM makosh_data.telegram_call_operations \
         WHERE account_id = $1 AND call_session_id = $2 \
           AND operation_state IN ('dispatching', 'awaiting_provider') \
         ORDER BY accepted_at_unix_seconds, operation_id FOR UPDATE",
    )
    .bind(&session.account_id)
    .bind(&session.call_session_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(database_error)?;

    for row in &rows {
        let mut operation = operation_from_row(row)?;
        let next = reconciled_state(&operation, session);
        if let Some((state, failure_category)) = next {
            transition_operation(
                transaction,
                &mut operation,
                state,
                failure_category,
                Some(session.tdlib_call_id),
                observed_at_unix_seconds,
            )
            .await?;
        }
    }
    Ok(())
}

fn reconciled_state(
    operation: &TelegramCallOperation,
    session: &TelegramCallSession,
) -> Option<(
    TelegramCallOperationState,
    Option<TelegramCallFailureCategory>,
)> {
    if session.state == TelegramProviderCallState::Error {
        return Some((
            TelegramCallOperationState::Failed,
            Some(
                session
                    .failure_category
                    .unwrap_or(TelegramCallFailureCategory::Unknown),
            ),
        ));
    }
    match operation.kind {
        TelegramCallOperationKind::InitiateAudio
            if session.direction == TelegramCallDirection::Outgoing
                && operation.provider_user_id.as_deref() == Some(&session.provider_user_id) =>
        {
            Some((TelegramCallOperationState::Completed, None))
        }
        TelegramCallOperationKind::AcceptAudio
            if !matches!(session.state, TelegramProviderCallState::Pending) =>
        {
            Some((TelegramCallOperationState::Completed, None))
        }
        TelegramCallOperationKind::Decline | TelegramCallOperationKind::End
            if session.state == TelegramProviderCallState::Discarded =>
        {
            Some((TelegramCallOperationState::Completed, None))
        }
        _ => None,
    }
}

async fn load_command_call_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    command: &TelegramCallCommand,
) -> Result<Option<TelegramCallSession>, TelegramCallsPersistenceError> {
    let row = if matches!(command, TelegramCallCommand::InitiateAudio { .. }) {
        sqlx::query(
            "SELECT call_session_id, account_id, runtime_generation, tdlib_call_id, \
             provider_call_unique_id, provider_user_id, direction, provider_state, \
             pending_created, pending_received, discard_reason, failure_category, revision, \
             created_at_unix_seconds, updated_at_unix_seconds, ended_at_unix_seconds \
             FROM makosh_data.telegram_call_sessions \
             WHERE account_id = $1 AND provider_state NOT IN ('discarded', 'error') \
             ORDER BY created_at_unix_seconds DESC LIMIT 1 FOR UPDATE",
        )
        .bind(command.account_id())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?
    } else {
        sqlx::query(
            "SELECT call_session_id, account_id, runtime_generation, tdlib_call_id, \
             provider_call_unique_id, provider_user_id, direction, provider_state, \
             pending_created, pending_received, discard_reason, failure_category, revision, \
             created_at_unix_seconds, updated_at_unix_seconds, ended_at_unix_seconds \
             FROM makosh_data.telegram_call_sessions \
             WHERE account_id = $1 AND call_session_id = $2 FOR UPDATE",
        )
        .bind(command.account_id())
        .bind(command.call_session_id())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?
    };
    row.as_ref().map(session_from_row).transpose()
}

async fn load_operation_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    operation_id: &str,
) -> Result<Option<TelegramCallOperation>, TelegramCallsPersistenceError> {
    let row = sqlx::query(
        "SELECT operation_id, account_id, call_session_id, operation_kind, operation_state, \
         request_fingerprint_sha256, provider_user_id, requested_mute, runtime_generation, \
         grant_epoch, tdlib_call_id, revision, accepted_at_unix_seconds, \
         updated_at_unix_seconds, completed_at_unix_seconds, failure_category \
         FROM makosh_data.telegram_call_operations \
         WHERE operation_id = $1 FOR UPDATE",
    )
    .bind(operation_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;
    row.as_ref().map(operation_from_row).transpose()
}

async fn insert_operation(
    transaction: &mut Transaction<'_, Postgres>,
    operation: &TelegramCallOperation,
) -> Result<(), TelegramCallsPersistenceError> {
    sqlx::query(
        "INSERT INTO makosh_data.telegram_call_operations ( \
         operation_id, account_id, call_session_id, operation_kind, operation_state, \
         request_fingerprint_sha256, provider_user_id, requested_mute, runtime_generation, \
         grant_epoch, tdlib_call_id, revision, accepted_at_unix_seconds, \
         updated_at_unix_seconds, completed_at_unix_seconds, failure_category \
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
    )
    .bind(&operation.operation_id)
    .bind(&operation.account_id)
    .bind(&operation.call_session_id)
    .bind(operation.kind.storage_name())
    .bind(operation.state.storage_name())
    .bind(operation.request_fingerprint_sha256.as_slice())
    .bind(operation.provider_user_id.as_deref())
    .bind(operation.requested_mute)
    .bind(as_i64(operation.runtime_generation)?)
    .bind(as_i64(operation.grant_epoch)?)
    .bind(operation.tdlib_call_id)
    .bind(as_i64(operation.revision)?)
    .bind(as_i64(operation.accepted_at_unix_seconds)?)
    .bind(as_i64(operation.updated_at_unix_seconds)?)
    .bind(optional_i64(operation.completed_at_unix_seconds)?)
    .bind(
        operation
            .failure_category
            .map(TelegramCallFailureCategory::storage_name),
    )
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

async fn transition_operation(
    transaction: &mut Transaction<'_, Postgres>,
    operation: &mut TelegramCallOperation,
    next_state: TelegramCallOperationState,
    failure_category: Option<TelegramCallFailureCategory>,
    tdlib_call_id: Option<i32>,
    now_unix_seconds: u64,
) -> Result<(), TelegramCallsPersistenceError> {
    if operation.state.is_terminal()
        || now_unix_seconds < operation.updated_at_unix_seconds
        || (next_state == TelegramCallOperationState::Failed) != failure_category.is_some()
    {
        return Err(TelegramCallsPersistenceError::CommandConflict(
            "operation_state",
        ));
    }
    let previous_revision = operation.revision;
    operation.state = next_state;
    operation.failure_category = failure_category;
    operation.tdlib_call_id = tdlib_call_id;
    operation.revision = operation.revision.saturating_add(1);
    operation.updated_at_unix_seconds = now_unix_seconds;
    operation.completed_at_unix_seconds = next_state.is_terminal().then_some(now_unix_seconds);
    let result = sqlx::query(
        "UPDATE makosh_data.telegram_call_operations SET \
         operation_state = $1, tdlib_call_id = $2, revision = $3, \
         updated_at_unix_seconds = $4, completed_at_unix_seconds = $5, \
         failure_category = $6 \
         WHERE operation_id = $7 AND revision = $8",
    )
    .bind(operation.state.storage_name())
    .bind(operation.tdlib_call_id)
    .bind(as_i64(operation.revision)?)
    .bind(as_i64(operation.updated_at_unix_seconds)?)
    .bind(optional_i64(operation.completed_at_unix_seconds)?)
    .bind(
        operation
            .failure_category
            .map(TelegramCallFailureCategory::storage_name),
    )
    .bind(&operation.operation_id)
    .bind(as_i64(previous_revision)?)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    if result.rows_affected() != 1 {
        return Err(TelegramCallsPersistenceError::StateRegression);
    }
    persist_operation_history(transaction, operation).await?;
    crate::realtime::persist_operation_event(transaction, operation).await?;
    Ok(())
}

async fn persist_operation_history(
    transaction: &mut Transaction<'_, Postgres>,
    operation: &TelegramCallOperation,
) -> Result<(), TelegramCallsPersistenceError> {
    sqlx::query(
        "INSERT INTO makosh_data.telegram_call_operation_history ( \
         operation_id, revision, operation_state, tdlib_call_id, \
         updated_at_unix_seconds, completed_at_unix_seconds, failure_category \
         ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&operation.operation_id)
    .bind(as_i64(operation.revision)?)
    .bind(operation.state.storage_name())
    .bind(operation.tdlib_call_id)
    .bind(as_i64(operation.updated_at_unix_seconds)?)
    .bind(optional_i64(operation.completed_at_unix_seconds)?)
    .bind(
        operation
            .failure_category
            .map(TelegramCallFailureCategory::storage_name),
    )
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

fn command_validation_error(error: TelegramCallCommandError) -> TelegramCallsPersistenceError {
    match error {
        TelegramCallCommandError::InvalidRequest(field) => {
            TelegramCallsPersistenceError::InvalidRequest(field)
        }
        TelegramCallCommandError::Conflict(field) => {
            TelegramCallsPersistenceError::CommandConflict(field)
        }
    }
}

pub(crate) fn operation_from_event_row(
    row: &PgRow,
) -> Result<TelegramCallOperation, TelegramCallsPersistenceError> {
    operation_from_columns(
        row,
        "operation_account_id",
        "operation_call_session_id",
        "operation_provider_user_id",
        "operation_runtime_generation",
        "operation_tdlib_call_id",
        "operation_revision",
        "operation_updated_at_unix_seconds",
        "operation_failure_category",
    )
}

fn operation_from_row(row: &PgRow) -> Result<TelegramCallOperation, TelegramCallsPersistenceError> {
    operation_from_columns(
        row,
        "account_id",
        "call_session_id",
        "provider_user_id",
        "runtime_generation",
        "tdlib_call_id",
        "revision",
        "updated_at_unix_seconds",
        "failure_category",
    )
}

#[allow(clippy::too_many_arguments)]
fn operation_from_columns(
    row: &PgRow,
    account_column: &str,
    call_session_column: &str,
    provider_user_column: &str,
    runtime_generation_column: &str,
    tdlib_call_id_column: &str,
    revision_column: &str,
    updated_at_column: &str,
    failure_category_column: &str,
) -> Result<TelegramCallOperation, TelegramCallsPersistenceError> {
    let kind = TelegramCallOperationKind::from_storage_name(
        &row.try_get::<String, _>("operation_kind")
            .map_err(database_error)?,
    )
    .ok_or(TelegramCallsPersistenceError::InvalidRow)?;
    let state = TelegramCallOperationState::from_storage_name(
        &row.try_get::<String, _>("operation_state")
            .map_err(database_error)?,
    )
    .ok_or(TelegramCallsPersistenceError::InvalidRow)?;
    let fingerprint = row
        .try_get::<Vec<u8>, _>("request_fingerprint_sha256")
        .map_err(database_error)?
        .try_into()
        .map_err(|_| TelegramCallsPersistenceError::InvalidRow)?;
    let failure_category = row
        .try_get::<Option<String>, _>(failure_category_column)
        .map_err(database_error)?
        .map(|value| {
            TelegramCallFailureCategory::from_storage_name(&value)
                .ok_or(TelegramCallsPersistenceError::InvalidRow)
        })
        .transpose()?;
    Ok(TelegramCallOperation {
        operation_id: row.try_get("operation_id").map_err(database_error)?,
        account_id: row.try_get(account_column).map_err(database_error)?,
        call_session_id: row.try_get(call_session_column).map_err(database_error)?,
        kind,
        state,
        request_fingerprint_sha256: fingerprint,
        provider_user_id: row.try_get(provider_user_column).map_err(database_error)?,
        requested_mute: row.try_get("requested_mute").map_err(database_error)?,
        runtime_generation: from_i64(
            row.try_get(runtime_generation_column)
                .map_err(database_error)?,
        )?,
        grant_epoch: from_i64(row.try_get("grant_epoch").map_err(database_error)?)?,
        tdlib_call_id: row.try_get(tdlib_call_id_column).map_err(database_error)?,
        revision: from_i64(row.try_get(revision_column).map_err(database_error)?)?,
        accepted_at_unix_seconds: from_i64(
            row.try_get("accepted_at_unix_seconds")
                .map_err(database_error)?,
        )?,
        updated_at_unix_seconds: from_i64(row.try_get(updated_at_column).map_err(database_error)?)?,
        completed_at_unix_seconds: optional_u64(
            row.try_get("completed_at_unix_seconds")
                .map_err(database_error)?,
        )?,
        failure_category,
    })
}
