use makosh_telegram_calls_core::{TelegramCallOperation, TelegramCallSession};
use sqlx::{PgPool, Postgres, Row, Transaction};

use crate::{
    TelegramCallsPersistenceError, as_i64, database_error, from_i64, session_from_row,
    validated_limit,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TelegramCallRealtimePayload {
    Call {
        session: TelegramCallSession,
        local_muted: bool,
    },
    Operation(TelegramCallOperation),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramCallRealtimeEvent {
    pub sequence: u64,
    pub payload: TelegramCallRealtimePayload,
}

pub(crate) async fn persist_call_event(
    transaction: &mut Transaction<'_, Postgres>,
    session: &TelegramCallSession,
) -> Result<u64, TelegramCallsPersistenceError> {
    let local_muted = sqlx::query_scalar::<_, bool>(
        "SELECT muted FROM makosh_data.telegram_call_local_mute \
         WHERE account_id = $1 AND call_session_id = $2",
    )
    .bind(&session.account_id)
    .bind(&session.call_session_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?
    .unwrap_or(false);
    let row = sqlx::query(
        "INSERT INTO makosh_data.telegram_call_realtime_events ( \
         account_id, event_kind, call_session_id, call_revision, local_muted, \
         observed_at_unix_seconds \
         ) VALUES ($1, 'call', $2, $3, $4, $5) RETURNING event_sequence",
    )
    .bind(&session.account_id)
    .bind(&session.call_session_id)
    .bind(as_i64(session.revision)?)
    .bind(local_muted)
    .bind(as_i64(session.updated_at_unix_seconds)?)
    .fetch_one(&mut **transaction)
    .await
    .map_err(database_error)?;
    let event_sequence = row.try_get("event_sequence").map_err(database_error)?;
    persist_replay_order(transaction, event_sequence).await
}

pub(crate) async fn persist_operation_event(
    transaction: &mut Transaction<'_, Postgres>,
    operation: &TelegramCallOperation,
) -> Result<u64, TelegramCallsPersistenceError> {
    let row = sqlx::query(
        "INSERT INTO makosh_data.telegram_call_realtime_events ( \
         account_id, event_kind, operation_id, operation_revision, local_muted, \
         observed_at_unix_seconds \
         ) VALUES ($1, 'operation', $2, $3, $4, $5) RETURNING event_sequence",
    )
    .bind(&operation.account_id)
    .bind(&operation.operation_id)
    .bind(as_i64(operation.revision)?)
    .bind(operation.requested_mute.unwrap_or(false))
    .bind(as_i64(operation.updated_at_unix_seconds)?)
    .fetch_one(&mut **transaction)
    .await
    .map_err(database_error)?;
    let event_sequence = row.try_get("event_sequence").map_err(database_error)?;
    persist_replay_order(transaction, event_sequence).await
}

async fn persist_replay_order(
    transaction: &mut Transaction<'_, Postgres>,
    event_sequence: i64,
) -> Result<u64, TelegramCallsPersistenceError> {
    let replay_sequence: i64 = sqlx::query_scalar(
        "UPDATE makosh_data.telegram_call_realtime_replay_cursor \
         SET next_sequence = next_sequence + 1 \
         WHERE cursor_scope = 'owner' \
         RETURNING next_sequence - 1",
    )
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?
    .ok_or(TelegramCallsPersistenceError::InvalidRow)?;
    sqlx::query(
        "INSERT INTO makosh_data.telegram_call_realtime_replay_order (\
             replay_sequence, event_sequence\
         ) \
         VALUES ($1, $2)",
    )
    .bind(replay_sequence)
    .bind(event_sequence)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    from_i64(replay_sequence)
}

pub(crate) async fn load_events(
    pool: &PgPool,
    account_id: &str,
    after_sequence: u64,
    limit: u32,
) -> Result<Vec<TelegramCallRealtimeEvent>, TelegramCallsPersistenceError> {
    let limit = validated_limit(limit)?;
    let after_sequence = i64::try_from(after_sequence)
        .map_err(|_| TelegramCallsPersistenceError::InvalidRequest("after_sequence"))?;
    let rows = sqlx::query(
        "SELECT replay.replay_sequence AS event_sequence, e.event_kind, e.local_muted, \
         s.call_session_id, s.account_id, s.runtime_generation, s.tdlib_call_id, \
         s.provider_call_unique_id, s.provider_user_id, s.direction, \
         ch.provider_state, ch.pending_created, ch.pending_received, ch.discard_reason, \
         ch.failure_category, ch.revision, s.created_at_unix_seconds, \
         ch.observed_at_unix_seconds AS updated_at_unix_seconds, \
         CASE WHEN ch.provider_state IN ('discarded', 'error') \
              THEN ch.observed_at_unix_seconds ELSE NULL END AS ended_at_unix_seconds, \
         o.operation_id, o.account_id AS operation_account_id, \
         o.call_session_id AS operation_call_session_id, o.operation_kind, \
         oh.operation_state, o.request_fingerprint_sha256, o.provider_user_id AS operation_provider_user_id, \
         o.requested_mute, o.runtime_generation AS operation_runtime_generation, \
         o.grant_epoch, oh.tdlib_call_id AS operation_tdlib_call_id, \
         oh.revision AS operation_revision, o.accepted_at_unix_seconds, \
         oh.updated_at_unix_seconds AS operation_updated_at_unix_seconds, \
         oh.completed_at_unix_seconds, oh.failure_category AS operation_failure_category \
         FROM makosh_data.telegram_call_realtime_replay_order replay \
         JOIN makosh_data.telegram_call_realtime_events e \
           ON e.event_sequence = replay.event_sequence \
         LEFT JOIN makosh_data.telegram_call_sessions s \
           ON e.event_kind = 'call' AND s.call_session_id = e.call_session_id \
         LEFT JOIN makosh_data.telegram_call_state_history ch \
           ON ch.call_session_id = e.call_session_id AND ch.revision = e.call_revision \
         LEFT JOIN makosh_data.telegram_call_operations o \
           ON e.event_kind = 'operation' AND o.operation_id = e.operation_id \
         LEFT JOIN makosh_data.telegram_call_operation_history oh \
           ON oh.operation_id = e.operation_id AND oh.revision = e.operation_revision \
         WHERE e.account_id = $1 AND replay.replay_sequence > $2 \
         ORDER BY replay.replay_sequence ASC LIMIT $3",
    )
    .bind(account_id)
    .bind(after_sequence)
    .bind(i64::from(limit))
    .fetch_all(pool)
    .await
    .map_err(database_error)?;

    rows.iter()
        .map(|row| {
            let sequence = from_i64(row.try_get("event_sequence").map_err(database_error)?)?;
            let payload = match row
                .try_get::<String, _>("event_kind")
                .map_err(database_error)?
                .as_str()
            {
                "call" => TelegramCallRealtimePayload::Call {
                    session: session_from_row(row)?,
                    local_muted: row.try_get("local_muted").map_err(database_error)?,
                },
                "operation" => TelegramCallRealtimePayload::Operation(
                    crate::operations::operation_from_event_row(row)?,
                ),
                _ => return Err(TelegramCallsPersistenceError::InvalidRow),
            };
            Ok(TelegramCallRealtimeEvent { sequence, payload })
        })
        .collect()
}
