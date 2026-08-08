mod command;
mod executor;
mod record;

use makosh_telegram_calls_core::{
    TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_MAJOR_V1, TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_NAME_V1,
    TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_OWNER_V1, TELEGRAM_CALLS_REALTIME_BACKFILL_SCOPE_V1,
    telegram_calls_realtime_backfill_run_id_v1,
};

use crate::TelegramCallsPersistence;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramCallsBackfillStateV1 {
    Accepted,
    Running,
    Succeeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramCallsBackfillPhaseV1 {
    Pending,
    Rebase,
    Backfill,
    Complete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramCallsBackfillExecutionV1 {
    pub state: TelegramCallsBackfillStateV1,
    pub phase: TelegramCallsBackfillPhaseV1,
    pub runtime_generation: Option<u64>,
    pub lease_epoch: u64,
    pub lease_expires_at_unix_millis: Option<i64>,
    pub checkpoint_frame_sequence: u64,
    pub processed_frame_count: u64,
    pub backfilled_frame_count: u64,
    pub rebase_original_max_event_sequence: Option<u64>,
    pub rebase_offset: Option<u64>,
    pub rebase_mapped_event_count: u64,
    pub attempt_count: u64,
    pub accepted_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
    pub completed_at_unix_millis: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramCallsBackfillBatchV1 {
    pub execution: TelegramCallsBackfillExecutionV1,
    pub source_frames_processed: u32,
    pub realtime_events_inserted: u32,
    pub realtime_events_rebased: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramCallsBackfillErrorV1 {
    Database,
    InvalidRequest,
    InvalidEnvelope,
    InvalidCommand,
    ContractMismatch,
    IdempotencyConflict,
    CorruptExecution,
    StaleLease,
    SourceHistoryConflict,
    RealtimeEventConflict,
    SequenceOverflow,
}

impl TelegramCallsPersistence {
    pub async fn calls_realtime_backfill_execution_v1(
        &self,
    ) -> Result<Option<TelegramCallsBackfillExecutionV1>, TelegramCallsBackfillErrorV1> {
        let run_id = telegram_calls_realtime_backfill_run_id_v1().bytes();
        record::load_execution(self.owner_pool(), &run_id).await
    }

    pub async fn accept_calls_realtime_backfill_v1(
        &self,
        envelope_bytes: &[u8],
    ) -> Result<TelegramCallsBackfillExecutionV1, TelegramCallsBackfillErrorV1> {
        let command = command::parse_backfill_command_v1(envelope_bytes)?;
        let mut transaction = self
            .owner_pool()
            .begin()
            .await
            .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
        sqlx::query(
            "INSERT INTO makosh_data.telegram_call_realtime_backfill_jobs (\
             job_run_id, job_owner, job_name, job_major, scope_id, command_message_id, \
             command_envelope_bytes, command_envelope_sha256, execution_state, execution_phase, \
             lease_epoch, checkpoint_frame_sequence, processed_frame_count, \
             backfilled_frame_count, rebase_mapped_event_count, attempt_count, \
             accepted_at_unix_millis, updated_at_unix_millis) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'accepted', 'pending', \
             0, 0, 0, 0, 0, 0, $9, $9) \
             ON CONFLICT (job_run_id) DO NOTHING",
        )
        .bind(command.run_id.as_slice())
        .bind(TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_OWNER_V1)
        .bind(TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_NAME_V1)
        .bind(i32::from(TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_MAJOR_V1))
        .bind(TELEGRAM_CALLS_REALTIME_BACKFILL_SCOPE_V1)
        .bind(command.message_id.as_slice())
        .bind(&command.envelope_bytes)
        .bind(command.envelope_sha256.as_slice())
        .bind(command.accepted_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
        let execution = record::load_execution_for_update(&mut transaction, &command.run_id)
            .await?
            .ok_or(TelegramCallsBackfillErrorV1::CorruptExecution)?;
        let stored_envelope: Vec<u8> = sqlx::query_scalar(
            "SELECT command_envelope_bytes \
             FROM makosh_data.telegram_call_realtime_backfill_jobs \
             WHERE job_run_id = $1",
        )
        .bind(command.run_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
        if stored_envelope != command.envelope_bytes {
            return Err(TelegramCallsBackfillErrorV1::IdempotencyConflict);
        }
        transaction
            .commit()
            .await
            .map_err(|_| TelegramCallsBackfillErrorV1::Database)?;
        Ok(execution)
    }
}
