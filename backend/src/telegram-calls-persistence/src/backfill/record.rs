use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow};

use makosh_telegram_calls_core::{
    TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_MAJOR_V1, TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_NAME_V1,
    TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_OWNER_V1, TELEGRAM_CALLS_REALTIME_BACKFILL_SCOPE_V1,
};

use super::command::{parse_backfill_command_v1, parse_completed_backfill_command_v1};
use super::{
    TelegramCallsBackfillErrorV1, TelegramCallsBackfillExecutionV1, TelegramCallsBackfillPhaseV1,
    TelegramCallsBackfillStateV1,
};

const EXECUTION_SELECT: &str = "\
SELECT job_run_id, job_owner, job_name, job_major, scope_id, command_message_id, \
command_envelope_bytes, command_envelope_sha256, execution_state, execution_phase, \
execution_runtime_generation, lease_epoch, lease_expires_at_unix_millis, \
checkpoint_frame_sequence, processed_frame_count, backfilled_frame_count, \
rebase_original_max_event_sequence, rebase_offset, rebase_mapped_event_count, \
attempt_count, accepted_at_unix_millis, updated_at_unix_millis, \
completed_at_unix_millis \
FROM makosh_data.telegram_call_realtime_backfill_jobs \
WHERE job_run_id = $1";

const EXECUTION_SELECT_FOR_UPDATE: &str = "\
SELECT job_run_id, job_owner, job_name, job_major, scope_id, command_message_id, \
command_envelope_bytes, command_envelope_sha256, execution_state, execution_phase, \
execution_runtime_generation, lease_epoch, lease_expires_at_unix_millis, \
checkpoint_frame_sequence, processed_frame_count, backfilled_frame_count, \
rebase_original_max_event_sequence, rebase_offset, rebase_mapped_event_count, \
attempt_count, accepted_at_unix_millis, updated_at_unix_millis, \
completed_at_unix_millis \
FROM makosh_data.telegram_call_realtime_backfill_jobs \
WHERE job_run_id = $1 \
FOR UPDATE";

pub(super) async fn load_execution(
    pool: &PgPool,
    run_id: &[u8; 16],
) -> Result<Option<TelegramCallsBackfillExecutionV1>, TelegramCallsBackfillErrorV1> {
    let row = sqlx::query(EXECUTION_SELECT)
        .bind(run_id.as_slice())
        .fetch_optional(pool)
        .await
        .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    row.as_ref().map(execution_from_row).transpose()
}

pub(super) async fn load_execution_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    run_id: &[u8; 16],
) -> Result<Option<TelegramCallsBackfillExecutionV1>, TelegramCallsBackfillErrorV1> {
    let row = sqlx::query(EXECUTION_SELECT_FOR_UPDATE)
        .bind(run_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
    row.as_ref().map(execution_from_row).transpose()
}

fn execution_from_row(
    row: &PgRow,
) -> Result<TelegramCallsBackfillExecutionV1, TelegramCallsBackfillErrorV1> {
    let state = state(row_value::<String>(row, "execution_state")?.as_str())?;
    let phase = phase(row_value::<String>(row, "execution_phase")?.as_str())?;
    let envelope_bytes: Vec<u8> = row_value(row, "command_envelope_bytes")?;
    let parsed = if state == TelegramCallsBackfillStateV1::Succeeded
        && phase == TelegramCallsBackfillPhaseV1::Complete
    {
        parse_completed_backfill_command_v1(&envelope_bytes)?
    } else {
        parse_backfill_command_v1(&envelope_bytes)?
    };
    let run_id: Vec<u8> = row_value(row, "job_run_id")?;
    let message_id: Vec<u8> = row_value(row, "command_message_id")?;
    let envelope_sha256: Vec<u8> = row_value(row, "command_envelope_sha256")?;
    let job_owner: String = row_value(row, "job_owner")?;
    let job_name: String = row_value(row, "job_name")?;
    let job_major: i32 = row_value(row, "job_major")?;
    let scope_id: String = row_value(row, "scope_id")?;
    let accepted_at_unix_millis: i64 = row_value(row, "accepted_at_unix_millis")?;
    if run_id != parsed.run_id
        || message_id != parsed.message_id
        || envelope_sha256 != parsed.envelope_sha256
        || job_owner != TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_OWNER_V1
        || job_name != TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_NAME_V1
        || job_major != i32::from(TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_MAJOR_V1)
        || scope_id != TELEGRAM_CALLS_REALTIME_BACKFILL_SCOPE_V1
        || accepted_at_unix_millis != parsed.accepted_at_unix_millis
    {
        return Err(TelegramCallsBackfillErrorV1::CorruptExecution);
    }
    Ok(TelegramCallsBackfillExecutionV1 {
        state,
        phase,
        runtime_generation: optional_u64(row_value(row, "execution_runtime_generation")?)?,
        lease_epoch: from_i64(row_value(row, "lease_epoch")?)?,
        lease_expires_at_unix_millis: row_value(row, "lease_expires_at_unix_millis")?,
        checkpoint_frame_sequence: from_i64(row_value(row, "checkpoint_frame_sequence")?)?,
        processed_frame_count: from_i64(row_value(row, "processed_frame_count")?)?,
        backfilled_frame_count: from_i64(row_value(row, "backfilled_frame_count")?)?,
        rebase_original_max_event_sequence: optional_u64(row_value(
            row,
            "rebase_original_max_event_sequence",
        )?)?,
        rebase_offset: optional_u64(row_value(row, "rebase_offset")?)?,
        rebase_mapped_event_count: from_i64(row_value(row, "rebase_mapped_event_count")?)?,
        attempt_count: from_i64(row_value(row, "attempt_count")?)?,
        accepted_at_unix_millis,
        updated_at_unix_millis: row_value(row, "updated_at_unix_millis")?,
        completed_at_unix_millis: row_value(row, "completed_at_unix_millis")?,
    })
}

fn row_value<T>(row: &PgRow, column: &str) -> Result<T, TelegramCallsBackfillErrorV1>
where
    for<'decode> T: sqlx::Decode<'decode, Postgres> + sqlx::Type<Postgres>,
{
    row.try_get(column)
        .map_err(|_| TelegramCallsBackfillErrorV1::CorruptExecution)
}

fn state(value: &str) -> Result<TelegramCallsBackfillStateV1, TelegramCallsBackfillErrorV1> {
    match value {
        "accepted" => Ok(TelegramCallsBackfillStateV1::Accepted),
        "running" => Ok(TelegramCallsBackfillStateV1::Running),
        "succeeded" => Ok(TelegramCallsBackfillStateV1::Succeeded),
        _ => Err(TelegramCallsBackfillErrorV1::CorruptExecution),
    }
}

fn phase(value: &str) -> Result<TelegramCallsBackfillPhaseV1, TelegramCallsBackfillErrorV1> {
    match value {
        "pending" => Ok(TelegramCallsBackfillPhaseV1::Pending),
        "rebase" => Ok(TelegramCallsBackfillPhaseV1::Rebase),
        "backfill" => Ok(TelegramCallsBackfillPhaseV1::Backfill),
        "complete" => Ok(TelegramCallsBackfillPhaseV1::Complete),
        _ => Err(TelegramCallsBackfillErrorV1::CorruptExecution),
    }
}

pub(super) fn as_i64(value: u64) -> Result<i64, TelegramCallsBackfillErrorV1> {
    i64::try_from(value).map_err(|_| TelegramCallsBackfillErrorV1::InvalidRequest)
}

pub(super) fn from_i64(value: i64) -> Result<u64, TelegramCallsBackfillErrorV1> {
    u64::try_from(value).map_err(|_| TelegramCallsBackfillErrorV1::CorruptExecution)
}

fn optional_u64(value: Option<i64>) -> Result<Option<u64>, TelegramCallsBackfillErrorV1> {
    value.map(from_i64).transpose()
}
