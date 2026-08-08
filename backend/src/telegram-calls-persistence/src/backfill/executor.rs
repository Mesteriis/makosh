use makosh_telegram_calls_core::{
    TELEGRAM_CALLS_REALTIME_BACKFILL_BATCH_SIZE_V1,
    telegram_calls_realtime_backfill_lease_expiry_v1, telegram_calls_realtime_backfill_run_id_v1,
};
use sqlx::{Postgres, Row, Transaction, postgres::PgRow};

use crate::TelegramCallsPersistence;

use super::record::{as_i64, from_i64, load_execution_for_update};
use super::{
    TelegramCallsBackfillBatchV1, TelegramCallsBackfillErrorV1, TelegramCallsBackfillExecutionV1,
    TelegramCallsBackfillPhaseV1, TelegramCallsBackfillStateV1,
};

impl TelegramCallsPersistence {
    pub async fn claim_calls_realtime_backfill_v1(
        &self,
        runtime_generation: u64,
        now_unix_millis: i64,
    ) -> Result<TelegramCallsBackfillExecutionV1, TelegramCallsBackfillErrorV1> {
        if runtime_generation == 0 || now_unix_millis <= 0 {
            return Err(TelegramCallsBackfillErrorV1::InvalidRequest);
        }
        let lease_expires_at = telegram_calls_realtime_backfill_lease_expiry_v1(now_unix_millis)
            .ok_or(TelegramCallsBackfillErrorV1::InvalidRequest)?;
        let run_id = telegram_calls_realtime_backfill_run_id_v1().bytes();
        let mut transaction = self
            .owner_pool()
            .begin()
            .await
            .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
        let current = load_execution_for_update(&mut transaction, &run_id)
            .await?
            .ok_or(TelegramCallsBackfillErrorV1::InvalidRequest)?;
        if current.state == TelegramCallsBackfillStateV1::Succeeded {
            transaction
                .commit()
                .await
                .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
            return Ok(current);
        }
        let claimed = match current.state {
            TelegramCallsBackfillStateV1::Accepted => {
                claim_accepted(
                    &mut transaction,
                    &run_id,
                    runtime_generation,
                    now_unix_millis,
                    lease_expires_at,
                )
                .await?
            }
            TelegramCallsBackfillStateV1::Running => {
                claim_running(
                    &mut transaction,
                    &run_id,
                    &current,
                    runtime_generation,
                    now_unix_millis,
                    lease_expires_at,
                )
                .await?
            }
            TelegramCallsBackfillStateV1::Succeeded => unreachable!(),
        };
        transaction
            .commit()
            .await
            .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
        Ok(claimed)
    }

    pub async fn execute_calls_realtime_backfill_batch_v1(
        &self,
        runtime_generation: u64,
        lease_epoch: u64,
        now_unix_millis: i64,
    ) -> Result<TelegramCallsBackfillBatchV1, TelegramCallsBackfillErrorV1> {
        if runtime_generation == 0 || lease_epoch == 0 || now_unix_millis <= 0 {
            return Err(TelegramCallsBackfillErrorV1::InvalidRequest);
        }
        let run_id = telegram_calls_realtime_backfill_run_id_v1().bytes();
        let mut transaction = self
            .owner_pool()
            .begin()
            .await
            .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
        let current = load_execution_for_update(&mut transaction, &run_id)
            .await?
            .ok_or(TelegramCallsBackfillErrorV1::InvalidRequest)?;
        validate_execution_lease(&current, runtime_generation, lease_epoch, now_unix_millis)?;
        let outcome = match current.phase {
            TelegramCallsBackfillPhaseV1::Rebase => {
                execute_rebase_batch(&mut transaction, &run_id, &current, now_unix_millis).await?
            }
            TelegramCallsBackfillPhaseV1::Backfill => {
                execute_source_batch(&mut transaction, &run_id, &current, now_unix_millis).await?
            }
            TelegramCallsBackfillPhaseV1::Pending | TelegramCallsBackfillPhaseV1::Complete => {
                return Err(TelegramCallsBackfillErrorV1::CorruptExecution);
            }
        };
        transaction
            .commit()
            .await
            .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
        Ok(outcome)
    }
}

async fn claim_accepted(
    transaction: &mut Transaction<'_, Postgres>,
    run_id: &[u8; 16],
    runtime_generation: u64,
    now_unix_millis: i64,
    lease_expires_at: i64,
) -> Result<TelegramCallsBackfillExecutionV1, TelegramCallsBackfillErrorV1> {
    let (original_max, source_count): (i64, i64) = sqlx::query_as(
        "SELECT \
         COALESCE((SELECT MAX(event_sequence) \
             FROM makosh_data.telegram_call_realtime_events), 0), \
         (SELECT COUNT(*) FROM makosh_data.telegram_call_realtime_frames)",
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    let rebase_offset = original_max
        .checked_add(source_count)
        .and_then(|value| value.checked_add(1))
        .filter(|value| *value > original_max)
        .ok_or(TelegramCallsBackfillErrorV1::SequenceOverflow)?;
    original_max
        .checked_add(rebase_offset)
        .ok_or(TelegramCallsBackfillErrorV1::SequenceOverflow)?;
    let phase = if original_max == 0 {
        "backfill"
    } else {
        "rebase"
    };
    sqlx::query(
        "UPDATE makosh_data.telegram_call_realtime_backfill_jobs \
         SET execution_state = 'running', execution_phase = $2, \
             execution_runtime_generation = $3, lease_epoch = 1, \
             lease_expires_at_unix_millis = $4, \
             rebase_original_max_event_sequence = $5, rebase_offset = $6, \
             attempt_count = 1, updated_at_unix_millis = $7 \
         WHERE job_run_id = $1 AND execution_state = 'accepted'",
    )
    .bind(run_id.as_slice())
    .bind(phase)
    .bind(as_i64(runtime_generation)?)
    .bind(lease_expires_at)
    .bind(original_max)
    .bind(rebase_offset)
    .bind(now_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    load_execution_for_update(transaction, run_id)
        .await?
        .ok_or(TelegramCallsBackfillErrorV1::CorruptExecution)
}

async fn claim_running(
    transaction: &mut Transaction<'_, Postgres>,
    run_id: &[u8; 16],
    current: &TelegramCallsBackfillExecutionV1,
    runtime_generation: u64,
    now_unix_millis: i64,
    lease_expires_at: i64,
) -> Result<TelegramCallsBackfillExecutionV1, TelegramCallsBackfillErrorV1> {
    let current_generation = current
        .runtime_generation
        .ok_or(TelegramCallsBackfillErrorV1::CorruptExecution)?;
    if runtime_generation < current_generation {
        return Err(TelegramCallsBackfillErrorV1::StaleLease);
    }
    let current_expiry = current
        .lease_expires_at_unix_millis
        .ok_or(TelegramCallsBackfillErrorV1::CorruptExecution)?;
    if runtime_generation == current_generation && current_expiry > now_unix_millis {
        return Ok(current.clone());
    }
    let next_epoch = current
        .lease_epoch
        .checked_add(1)
        .ok_or(TelegramCallsBackfillErrorV1::SequenceOverflow)?;
    sqlx::query(
        "UPDATE makosh_data.telegram_call_realtime_backfill_jobs \
         SET execution_runtime_generation = $2, lease_epoch = $3, \
             lease_expires_at_unix_millis = $4, attempt_count = attempt_count + 1, \
             updated_at_unix_millis = $5 \
         WHERE job_run_id = $1 AND execution_state = 'running'",
    )
    .bind(run_id.as_slice())
    .bind(as_i64(runtime_generation)?)
    .bind(as_i64(next_epoch)?)
    .bind(lease_expires_at)
    .bind(now_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    load_execution_for_update(transaction, run_id)
        .await?
        .ok_or(TelegramCallsBackfillErrorV1::CorruptExecution)
}

fn validate_execution_lease(
    execution: &TelegramCallsBackfillExecutionV1,
    runtime_generation: u64,
    lease_epoch: u64,
    now_unix_millis: i64,
) -> Result<(), TelegramCallsBackfillErrorV1> {
    if execution.state != TelegramCallsBackfillStateV1::Running
        || execution.runtime_generation != Some(runtime_generation)
        || execution.lease_epoch != lease_epoch
        || execution
            .lease_expires_at_unix_millis
            .is_none_or(|expiry| expiry <= now_unix_millis)
    {
        return Err(TelegramCallsBackfillErrorV1::StaleLease);
    }
    Ok(())
}

async fn execute_rebase_batch(
    transaction: &mut Transaction<'_, Postgres>,
    run_id: &[u8; 16],
    current: &TelegramCallsBackfillExecutionV1,
    now_unix_millis: i64,
) -> Result<TelegramCallsBackfillBatchV1, TelegramCallsBackfillErrorV1> {
    let original_max = as_i64(
        current
            .rebase_original_max_event_sequence
            .ok_or(TelegramCallsBackfillErrorV1::CorruptExecution)?,
    )?;
    let offset = as_i64(
        current
            .rebase_offset
            .ok_or(TelegramCallsBackfillErrorV1::CorruptExecution)?,
    )?;
    let result = sqlx::query(
        "WITH selected AS (\
             SELECT events.event_sequence \
             FROM makosh_data.telegram_call_realtime_events AS events \
             LEFT JOIN makosh_data.telegram_call_realtime_replay_order AS replay \
               ON replay.event_sequence = events.event_sequence \
             WHERE events.event_sequence <= $1 \
               AND replay.event_sequence IS NULL \
             ORDER BY events.event_sequence ASC \
             LIMIT $2 \
             FOR SHARE OF events\
         ) \
         INSERT INTO makosh_data.telegram_call_realtime_replay_order (\
             replay_sequence, event_sequence\
         ) \
         SELECT selected.event_sequence + $3, selected.event_sequence \
         FROM selected",
    )
    .bind(original_max)
    .bind(i64::from(TELEGRAM_CALLS_REALTIME_BACKFILL_BATCH_SIZE_V1))
    .bind(offset)
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    let rebased = u32::try_from(result.rows_affected())
        .map_err(|_| TelegramCallsBackfillErrorV1::SequenceOverflow)?;
    let has_more: bool = sqlx::query_scalar(
        "SELECT EXISTS(\
             SELECT 1 \
             FROM makosh_data.telegram_call_realtime_events AS events \
             LEFT JOIN makosh_data.telegram_call_realtime_replay_order AS replay \
               ON replay.event_sequence = events.event_sequence \
             WHERE events.event_sequence <= $1 \
               AND replay.event_sequence IS NULL\
         )",
    )
    .bind(original_max)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    let phase = if has_more { "rebase" } else { "backfill" };
    let renewed_expiry = telegram_calls_realtime_backfill_lease_expiry_v1(now_unix_millis)
        .ok_or(TelegramCallsBackfillErrorV1::InvalidRequest)?;
    sqlx::query(
        "UPDATE makosh_data.telegram_call_realtime_backfill_jobs \
         SET execution_phase = $2, \
             rebase_mapped_event_count = rebase_mapped_event_count + $3, \
             lease_expires_at_unix_millis = $4, updated_at_unix_millis = $5 \
         WHERE job_run_id = $1",
    )
    .bind(run_id.as_slice())
    .bind(phase)
    .bind(i64::from(rebased))
    .bind(renewed_expiry)
    .bind(now_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    let execution = load_execution_for_update(transaction, run_id)
        .await?
        .ok_or(TelegramCallsBackfillErrorV1::CorruptExecution)?;
    Ok(TelegramCallsBackfillBatchV1 {
        execution,
        source_frames_processed: 0,
        realtime_events_inserted: 0,
        realtime_events_rebased: rebased,
    })
}

async fn execute_source_batch(
    transaction: &mut Transaction<'_, Postgres>,
    run_id: &[u8; 16],
    current: &TelegramCallsBackfillExecutionV1,
    now_unix_millis: i64,
) -> Result<TelegramCallsBackfillBatchV1, TelegramCallsBackfillErrorV1> {
    let rows = source_rows(transaction, current.checkpoint_frame_sequence).await?;
    let original_max = current
        .rebase_original_max_event_sequence
        .ok_or(TelegramCallsBackfillErrorV1::CorruptExecution)?;
    let offset = current
        .rebase_offset
        .ok_or(TelegramCallsBackfillErrorV1::CorruptExecution)?;
    let mut checkpoint = current.checkpoint_frame_sequence;
    let mut inserted = 0_u32;
    for row in &rows {
        let frame = source_frame(row)?;
        validate_source_history(&frame)?;
        if frame.event_sequence.is_some() {
            validate_existing_event(&frame, original_max, offset)?;
        } else {
            let insertion_index = current
                .backfilled_frame_count
                .checked_add(u64::from(inserted))
                .and_then(|value| value.checked_add(1))
                .ok_or(TelegramCallsBackfillErrorV1::SequenceOverflow)?;
            let target_sequence = original_max
                .checked_add(insertion_index)
                .filter(|sequence| *sequence < offset)
                .ok_or(TelegramCallsBackfillErrorV1::SequenceOverflow)?;
            insert_backfilled_event(transaction, &frame, target_sequence).await?;
            inserted = inserted
                .checked_add(1)
                .ok_or(TelegramCallsBackfillErrorV1::SequenceOverflow)?;
        }
        checkpoint = frame.frame_sequence;
    }
    let has_more: bool = sqlx::query_scalar(
        "SELECT EXISTS(\
             SELECT 1 FROM makosh_data.telegram_call_realtime_frames \
             WHERE frame_sequence > $1\
         )",
    )
    .bind(as_i64(checkpoint)?)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    let renewed_expiry = telegram_calls_realtime_backfill_lease_expiry_v1(now_unix_millis)
        .ok_or(TelegramCallsBackfillErrorV1::InvalidRequest)?;
    let processed =
        u32::try_from(rows.len()).map_err(|_| TelegramCallsBackfillErrorV1::SequenceOverflow)?;
    if has_more {
        update_source_progress(
            transaction,
            run_id,
            checkpoint,
            processed,
            inserted,
            renewed_expiry,
            now_unix_millis,
        )
        .await?;
    } else {
        complete_source_progress(
            transaction,
            run_id,
            checkpoint,
            processed,
            inserted,
            renewed_expiry,
            now_unix_millis,
        )
        .await?;
    }
    let execution = load_execution_for_update(transaction, run_id)
        .await?
        .ok_or(TelegramCallsBackfillErrorV1::CorruptExecution)?;
    Ok(TelegramCallsBackfillBatchV1 {
        execution,
        source_frames_processed: processed,
        realtime_events_inserted: inserted,
        realtime_events_rebased: 0,
    })
}

async fn source_rows(
    transaction: &mut Transaction<'_, Postgres>,
    checkpoint: u64,
) -> Result<Vec<PgRow>, TelegramCallsBackfillErrorV1> {
    sqlx::query(
        "SELECT frames.frame_sequence, frames.account_id, frames.call_session_id, \
         frames.call_revision, frames.provider_state, frames.pending_created, \
         frames.pending_received, frames.discard_reason, frames.failure_category, \
         frames.observed_at_unix_seconds, \
         (history.revision IS NOT NULL) AS history_exists, \
         (history.provider_state = frames.provider_state) AS history_state_matches, \
         (history.pending_created = frames.pending_created) AS history_created_matches, \
         (history.pending_received = frames.pending_received) AS history_received_matches, \
         (history.discard_reason IS NOT DISTINCT FROM frames.discard_reason) \
             AS history_discard_matches, \
         (history.failure_category IS NOT DISTINCT FROM frames.failure_category) \
             AS history_failure_matches, \
         (history.observed_at_unix_seconds = frames.observed_at_unix_seconds) \
             AS history_observed_matches, \
         events.event_sequence, events.account_id AS event_account_id, \
         events.event_kind, events.local_muted AS event_local_muted, \
         events.observed_at_unix_seconds AS event_observed_at_unix_seconds, \
         replay.replay_sequence AS event_replay_sequence \
         FROM makosh_data.telegram_call_realtime_frames AS frames \
         LEFT JOIN makosh_data.telegram_call_state_history AS history \
           ON history.call_session_id = frames.call_session_id \
          AND history.revision = frames.call_revision \
         LEFT JOIN makosh_data.telegram_call_realtime_events AS events \
           ON events.call_session_id = frames.call_session_id \
          AND events.call_revision = frames.call_revision \
         LEFT JOIN makosh_data.telegram_call_realtime_replay_order AS replay \
           ON replay.event_sequence = events.event_sequence \
         WHERE frames.frame_sequence > $1 \
         ORDER BY frames.frame_sequence ASC \
         LIMIT $2 \
         FOR SHARE OF frames",
    )
    .bind(as_i64(checkpoint)?)
    .bind(i64::from(TELEGRAM_CALLS_REALTIME_BACKFILL_BATCH_SIZE_V1))
    .fetch_all(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::Database)
}

struct SourceFrameV1 {
    frame_sequence: u64,
    account_id: String,
    call_session_id: String,
    call_revision: u64,
    observed_at_unix_seconds: i64,
    history_matches: bool,
    event_sequence: Option<u64>,
    event_account_id: Option<String>,
    event_kind: Option<String>,
    event_local_muted: Option<bool>,
    event_observed_at_unix_seconds: Option<i64>,
    event_replay_sequence: Option<u64>,
}

fn source_frame(row: &PgRow) -> Result<SourceFrameV1, TelegramCallsBackfillErrorV1> {
    let history_matches = row_bool(row, "history_exists")?
        && row_bool(row, "history_state_matches")?
        && row_bool(row, "history_created_matches")?
        && row_bool(row, "history_received_matches")?
        && row_bool(row, "history_discard_matches")?
        && row_bool(row, "history_failure_matches")?
        && row_bool(row, "history_observed_matches")?;
    Ok(SourceFrameV1 {
        frame_sequence: from_i64(row_i64(row, "frame_sequence")?)?,
        account_id: row_string(row, "account_id")?,
        call_session_id: row_string(row, "call_session_id")?,
        call_revision: from_i64(row_i64(row, "call_revision")?)?,
        observed_at_unix_seconds: row_i64(row, "observed_at_unix_seconds")?,
        history_matches,
        event_sequence: optional_i64(row, "event_sequence")?
            .map(from_i64)
            .transpose()?,
        event_account_id: row
            .try_get("event_account_id")
            .map_err(|_| TelegramCallsBackfillErrorV1::SourceHistoryConflict)?,
        event_kind: row
            .try_get("event_kind")
            .map_err(|_| TelegramCallsBackfillErrorV1::SourceHistoryConflict)?,
        event_local_muted: row
            .try_get("event_local_muted")
            .map_err(|_| TelegramCallsBackfillErrorV1::SourceHistoryConflict)?,
        event_observed_at_unix_seconds: optional_i64(row, "event_observed_at_unix_seconds")?,
        event_replay_sequence: optional_i64(row, "event_replay_sequence")?
            .map(from_i64)
            .transpose()?,
    })
}

fn validate_source_history(frame: &SourceFrameV1) -> Result<(), TelegramCallsBackfillErrorV1> {
    frame
        .history_matches
        .then_some(())
        .ok_or(TelegramCallsBackfillErrorV1::SourceHistoryConflict)
}

fn validate_existing_event(
    frame: &SourceFrameV1,
    original_max: u64,
    offset: u64,
) -> Result<(), TelegramCallsBackfillErrorV1> {
    let expected_replay_sequence = frame
        .event_sequence
        .and_then(|sequence| sequence.checked_add(offset));
    if frame
        .event_sequence
        .is_none_or(|sequence| sequence > original_max)
        || frame.event_replay_sequence != expected_replay_sequence
        || frame.event_account_id.as_deref() != Some(frame.account_id.as_str())
        || frame.event_kind.as_deref() != Some("call")
        || frame.event_local_muted.is_none()
        || frame.event_observed_at_unix_seconds != Some(frame.observed_at_unix_seconds)
    {
        return Err(TelegramCallsBackfillErrorV1::RealtimeEventConflict);
    }
    Ok(())
}

async fn insert_backfilled_event(
    transaction: &mut Transaction<'_, Postgres>,
    frame: &SourceFrameV1,
    target_sequence: u64,
) -> Result<(), TelegramCallsBackfillErrorV1> {
    let event_sequence: i64 = sqlx::query_scalar(
        "INSERT INTO makosh_data.telegram_call_realtime_events (\
             account_id, event_kind, call_session_id, call_revision, \
             operation_id, operation_revision, local_muted, observed_at_unix_seconds\
         ) VALUES ($1, 'call', $2, $3, NULL, NULL, FALSE, $4) \
         RETURNING event_sequence",
    )
    .bind(&frame.account_id)
    .bind(&frame.call_session_id)
    .bind(as_i64(frame.call_revision)?)
    .bind(frame.observed_at_unix_seconds)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::RealtimeEventConflict)?;
    let result = sqlx::query(
        "INSERT INTO makosh_data.telegram_call_realtime_replay_order (\
             replay_sequence, event_sequence\
         ) \
         VALUES ($1, $2)",
    )
    .bind(as_i64(target_sequence)?)
    .bind(event_sequence)
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::RealtimeEventConflict)?;
    (result.rows_affected() == 1)
        .then_some(())
        .ok_or(TelegramCallsBackfillErrorV1::RealtimeEventConflict)
}

async fn update_source_progress(
    transaction: &mut Transaction<'_, Postgres>,
    run_id: &[u8; 16],
    checkpoint: u64,
    processed: u32,
    inserted: u32,
    lease_expires_at: i64,
    now_unix_millis: i64,
) -> Result<(), TelegramCallsBackfillErrorV1> {
    sqlx::query(
        "UPDATE makosh_data.telegram_call_realtime_backfill_jobs \
         SET checkpoint_frame_sequence = $2, \
             processed_frame_count = processed_frame_count + $3, \
             backfilled_frame_count = backfilled_frame_count + $4, \
             lease_expires_at_unix_millis = $5, updated_at_unix_millis = $6 \
         WHERE job_run_id = $1",
    )
    .bind(run_id.as_slice())
    .bind(as_i64(checkpoint)?)
    .bind(i64::from(processed))
    .bind(i64::from(inserted))
    .bind(lease_expires_at)
    .bind(now_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    Ok(())
}

async fn complete_source_progress(
    transaction: &mut Transaction<'_, Postgres>,
    run_id: &[u8; 16],
    checkpoint: u64,
    processed: u32,
    inserted: u32,
    lease_expires_at: i64,
    now_unix_millis: i64,
) -> Result<(), TelegramCallsBackfillErrorV1> {
    let maximum_sequence: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(replay_sequence), 0) \
         FROM makosh_data.telegram_call_realtime_replay_order",
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    let next_sequence = maximum_sequence
        .checked_add(1)
        .filter(|value| *value > 0)
        .ok_or(TelegramCallsBackfillErrorV1::SequenceOverflow)?;
    sqlx::query(
        "INSERT INTO makosh_data.telegram_call_realtime_replay_cursor (\
             cursor_scope, next_sequence\
         ) VALUES ('owner', $1) \
         ON CONFLICT (cursor_scope) DO NOTHING",
    )
    .bind(next_sequence)
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    let persisted_next: i64 = sqlx::query_scalar(
        "SELECT next_sequence \
         FROM makosh_data.telegram_call_realtime_replay_cursor \
         WHERE cursor_scope = 'owner' \
         FOR UPDATE",
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    if persisted_next != next_sequence {
        return Err(TelegramCallsBackfillErrorV1::RealtimeEventConflict);
    }
    sqlx::query(
        "UPDATE makosh_data.telegram_call_realtime_backfill_jobs \
         SET execution_state = 'succeeded', execution_phase = 'complete', \
             checkpoint_frame_sequence = $2, \
             processed_frame_count = processed_frame_count + $3, \
             backfilled_frame_count = backfilled_frame_count + $4, \
             lease_expires_at_unix_millis = $5, \
             updated_at_unix_millis = $6, completed_at_unix_millis = $6 \
         WHERE job_run_id = $1",
    )
    .bind(run_id.as_slice())
    .bind(as_i64(checkpoint)?)
    .bind(i64::from(processed))
    .bind(i64::from(inserted))
    .bind(lease_expires_at)
    .bind(now_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    Ok(())
}

fn row_bool(row: &PgRow, column: &str) -> Result<bool, TelegramCallsBackfillErrorV1> {
    row.try_get(column)
        .map_err(|_| TelegramCallsBackfillErrorV1::SourceHistoryConflict)
}

fn row_i64(row: &PgRow, column: &str) -> Result<i64, TelegramCallsBackfillErrorV1> {
    row.try_get(column)
        .map_err(|_| TelegramCallsBackfillErrorV1::SourceHistoryConflict)
}

fn optional_i64(row: &PgRow, column: &str) -> Result<Option<i64>, TelegramCallsBackfillErrorV1> {
    row.try_get(column)
        .map_err(|_| TelegramCallsBackfillErrorV1::SourceHistoryConflict)
}

fn row_string(row: &PgRow, column: &str) -> Result<String, TelegramCallsBackfillErrorV1> {
    row.try_get(column)
        .map_err(|_| TelegramCallsBackfillErrorV1::SourceHistoryConflict)
}
