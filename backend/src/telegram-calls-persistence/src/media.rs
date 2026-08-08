use makosh_telegram_calls_core::{
    TelegramCallMediaProjection, TelegramCallMediaState, TelegramCallMediaUpdate,
    project_call_media_update,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    TelegramCallsPersistence, TelegramCallsPersistenceError, as_i64, database_error, from_i64,
    optional_i64, optional_u64,
};

impl TelegramCallsPersistence {
    pub async fn ingest_media_update(
        &self,
        update: &TelegramCallMediaUpdate,
    ) -> Result<TelegramCallMediaProjection, TelegramCallsPersistenceError> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        validate_call_fence(&mut transaction, update).await?;
        let current = load_media_for_update(&mut transaction, &update.call_session_id).await?;
        let projected = project_call_media_update(current.as_ref(), update)?;
        if projected.changed {
            persist_media_projection(&mut transaction, &projected.projection).await?;
            persist_media_history(&mut transaction, &projected.projection).await?;
        }
        transaction.commit().await.map_err(database_error)?;
        Ok(projected.projection)
    }

    pub async fn media_projection(
        &self,
        account_id: &str,
        call_session_id: &str,
    ) -> Result<Option<TelegramCallMediaProjection>, TelegramCallsPersistenceError> {
        let row = sqlx::query(
            "SELECT account_id, call_session_id, runtime_generation, provider_revision, \
             media_state, revision, connected_at_unix_seconds, updated_at_unix_seconds, \
             failed_at_unix_seconds \
             FROM makosh_data.telegram_call_media_projection \
             WHERE account_id = $1 AND call_session_id = $2",
        )
        .bind(account_id)
        .bind(call_session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;
        row.as_ref().map(media_from_row).transpose()
    }
}

async fn validate_call_fence(
    transaction: &mut Transaction<'_, Postgres>,
    update: &TelegramCallMediaUpdate,
) -> Result<(), TelegramCallsPersistenceError> {
    let row = sqlx::query(
        "SELECT runtime_generation, revision FROM makosh_data.telegram_call_sessions \
         WHERE account_id = $1 AND call_session_id = $2 FOR UPDATE",
    )
    .bind(&update.account_id)
    .bind(&update.call_session_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?
    .ok_or(TelegramCallsPersistenceError::IdentityConflict)?;
    let runtime_generation = from_i64(row.try_get("runtime_generation").map_err(database_error)?)?;
    let provider_revision = from_i64(row.try_get("revision").map_err(database_error)?)?;
    if runtime_generation != update.runtime_generation
        || provider_revision != update.provider_revision
    {
        return Err(TelegramCallsPersistenceError::IdentityConflict);
    }
    Ok(())
}

async fn load_media_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    call_session_id: &str,
) -> Result<Option<TelegramCallMediaProjection>, TelegramCallsPersistenceError> {
    let row = sqlx::query(
        "SELECT account_id, call_session_id, runtime_generation, provider_revision, \
         media_state, revision, connected_at_unix_seconds, updated_at_unix_seconds, \
         failed_at_unix_seconds \
         FROM makosh_data.telegram_call_media_projection \
         WHERE call_session_id = $1 FOR UPDATE",
    )
    .bind(call_session_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;
    row.as_ref().map(media_from_row).transpose()
}

async fn persist_media_projection(
    transaction: &mut Transaction<'_, Postgres>,
    projection: &TelegramCallMediaProjection,
) -> Result<(), TelegramCallsPersistenceError> {
    sqlx::query(
        "INSERT INTO makosh_data.telegram_call_media_projection ( \
         call_session_id, account_id, runtime_generation, provider_revision, media_state, \
         revision, connected_at_unix_seconds, updated_at_unix_seconds, failed_at_unix_seconds \
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
         ON CONFLICT (call_session_id) DO UPDATE SET \
         runtime_generation = EXCLUDED.runtime_generation, \
         provider_revision = EXCLUDED.provider_revision, media_state = EXCLUDED.media_state, \
         revision = EXCLUDED.revision, \
         connected_at_unix_seconds = EXCLUDED.connected_at_unix_seconds, \
         updated_at_unix_seconds = EXCLUDED.updated_at_unix_seconds, \
         failed_at_unix_seconds = EXCLUDED.failed_at_unix_seconds",
    )
    .bind(&projection.call_session_id)
    .bind(&projection.account_id)
    .bind(as_i64(projection.runtime_generation)?)
    .bind(as_i64(projection.provider_revision)?)
    .bind(projection.state.storage_name())
    .bind(as_i64(projection.revision)?)
    .bind(optional_i64(projection.connected_at_unix_seconds)?)
    .bind(as_i64(projection.updated_at_unix_seconds)?)
    .bind(optional_i64(projection.failed_at_unix_seconds)?)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

async fn persist_media_history(
    transaction: &mut Transaction<'_, Postgres>,
    projection: &TelegramCallMediaProjection,
) -> Result<(), TelegramCallsPersistenceError> {
    sqlx::query(
        "INSERT INTO makosh_data.telegram_call_media_state_history ( \
         call_session_id, revision, runtime_generation, provider_revision, media_state, \
         observed_at_unix_seconds \
         ) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&projection.call_session_id)
    .bind(as_i64(projection.revision)?)
    .bind(as_i64(projection.runtime_generation)?)
    .bind(as_i64(projection.provider_revision)?)
    .bind(projection.state.storage_name())
    .bind(as_i64(projection.updated_at_unix_seconds)?)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

fn media_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<TelegramCallMediaProjection, TelegramCallsPersistenceError> {
    Ok(TelegramCallMediaProjection {
        account_id: row.try_get("account_id").map_err(database_error)?,
        call_session_id: row.try_get("call_session_id").map_err(database_error)?,
        runtime_generation: from_i64(row.try_get("runtime_generation").map_err(database_error)?)?,
        provider_revision: from_i64(row.try_get("provider_revision").map_err(database_error)?)?,
        state: TelegramCallMediaState::from_storage_name(
            &row.try_get::<String, _>("media_state")
                .map_err(database_error)?,
        )
        .ok_or(TelegramCallsPersistenceError::InvalidRow)?,
        revision: from_i64(row.try_get("revision").map_err(database_error)?)?,
        connected_at_unix_seconds: optional_u64(
            row.try_get("connected_at_unix_seconds")
                .map_err(database_error)?,
        )?,
        updated_at_unix_seconds: from_i64(
            row.try_get("updated_at_unix_seconds")
                .map_err(database_error)?,
        )?,
        failed_at_unix_seconds: optional_u64(
            row.try_get("failed_at_unix_seconds")
                .map_err(database_error)?,
        )?,
    })
}
