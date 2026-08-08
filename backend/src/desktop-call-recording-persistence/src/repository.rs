use makosh_desktop_call_recording_core::RecordingStateV1;
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    CaptureStartedWriteV1, ExactOutboxRecordV1, HostCommandCompletionV1, LeasedHostCommandV1,
    NewRecordingRunV1, PendingOutboxV1, PendingRealtimeV1, PersistedRecordingRunV1,
    PersistenceErrorV1, RealtimeTransitionV1, RejectRecordingWriteV1, TerminalRecordingMetadataV1,
};

#[derive(Clone)]
pub struct DesktopCallRecordingRepositoryV1 {
    pool: PgPool,
}

impl DesktopCallRecordingRepositoryV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, PersistenceErrorV1> {
        if database_id.is_empty()
            || database_id != binding.identity().database_id()
            || pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || binding.access().runtime_principal().is_empty()
        {
            return Err(PersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(
                u16::try_from(pgbouncer_port)
                    .map_err(|_| PersistenceErrorV1::StorageUnavailable)?,
            )
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(storage)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), PersistenceErrorV1> {
        sqlx::query("SELECT 1 FROM makosh_data.desktop_call_recording_runs LIMIT 0")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage)
    }

    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn accept_or_replay(
        &self,
        run: &NewRecordingRunV1,
        begin_command_id: [u8; 16],
        realtime: &RealtimeTransitionV1,
    ) -> Result<(PersistedRecordingRunV1, bool), PersistenceErrorV1> {
        validate_new(run, &begin_command_id, realtime)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        if let Some(existing) =
            load_by_operation(&mut tx, &run.logical_owner_id, &run.operation_id).await?
        {
            tx.commit().await.map_err(storage)?;
            return if same_request(&existing, run) {
                Ok((existing, true))
            } else {
                Err(PersistenceErrorV1::Conflict)
            };
        }
        sqlx::query("INSERT INTO makosh_data.desktop_call_recording_runs (logical_owner_id, operation_id, request_sha256, call_evidence_id, call_evidence_revision, recording_evidence_id, recording_revision, run_state, device_actor_sha256, challenge_id, challenge_expires_at_unix_ms, maximum_duration_millis, consent_policy_revision) VALUES ($1,$2,$3,$4,$5,$6,1,1,$7,$8,$9,$10,$11)")
            .bind(&run.logical_owner_id).bind(run.operation_id.as_slice()).bind(run.request_sha256.as_slice())
            .bind(run.call_evidence_id.as_slice()).bind(to_i64(run.call_evidence_revision)?)
            .bind(run.recording_evidence_id.as_slice()).bind(run.device_actor_sha256.as_slice())
            .bind(run.challenge_id.as_slice()).bind(run.challenge_expires_at_unix_ms)
            .bind(to_i64(run.maximum_duration_millis)?).bind(i32::try_from(run.consent_policy_revision).map_err(|_| PersistenceErrorV1::InvalidInput)?)
            .execute(&mut *tx).await.map_err(storage)?;
        sqlx::query("INSERT INTO makosh_data.desktop_call_recording_host_commands (command_id, logical_owner_id, recording_evidence_id, command_kind, command_revision) VALUES ($1,$2,$3,1,1)")
            .bind(begin_command_id.as_slice()).bind(&run.logical_owner_id).bind(run.recording_evidence_id.as_slice())
            .execute(&mut *tx).await.map_err(storage)?;
        insert_realtime(
            &mut tx,
            &run.logical_owner_id,
            &run.recording_evidence_id,
            1,
            realtime,
        )
        .await?;
        tx.commit().await.map_err(storage)?;
        let persisted = self
            .get(&run.logical_owner_id, &run.recording_evidence_id)
            .await?
            .ok_or(PersistenceErrorV1::InvalidRow)?;
        Ok((persisted, false))
    }

    pub async fn get(
        &self,
        owner: &str,
        recording_id: &[u8; 16],
    ) -> Result<Option<PersistedRecordingRunV1>, PersistenceErrorV1> {
        if !valid_owner(owner) || zero(recording_id) {
            return Err(PersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(RUN_COLUMNS_BY_RECORDING)
            .bind(owner)
            .bind(recording_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage)?;
        row.map(decode_run).transpose()
    }

    pub async fn mark_capturing(
        &self,
        write: &CaptureStartedWriteV1,
    ) -> Result<PersistedRecordingRunV1, PersistenceErrorV1> {
        if !valid_owner(&write.logical_owner_id)
            || zero(&write.recording_evidence_id)
            || write.expected_revision == 0
            || write.started_at_unix_ms <= 0
            || zero(&write.consent_receipt_id)
            || zero(&write.command_id)
            || zero(&write.claim_sha256)
        {
            return Err(PersistenceErrorV1::InvalidInput);
        }
        let mut tx = self.pool.begin().await.map_err(storage)?;
        let next_revision = write
            .expected_revision
            .checked_add(1)
            .ok_or(PersistenceErrorV1::InvalidInput)?;
        let command = sqlx::query("UPDATE makosh_data.desktop_call_recording_host_commands SET completed_at_unix_ms=$1 WHERE command_id=$2 AND logical_owner_id=$3 AND recording_evidence_id=$4 AND command_kind=1 AND leased_by_sha256=$5 AND completed_at_unix_ms IS NULL AND lease_expires_at_unix_ms >= $1")
            .bind(write.started_at_unix_ms).bind(write.command_id.as_slice()).bind(&write.logical_owner_id)
            .bind(write.recording_evidence_id.as_slice()).bind(write.claim_sha256.as_slice())
            .execute(&mut *tx).await.map_err(storage)?;
        if command.rows_affected() != 1 {
            return Err(PersistenceErrorV1::Conflict);
        }
        let run = sqlx::query("UPDATE makosh_data.desktop_call_recording_runs SET recording_revision=$1,run_state=2,started_at_unix_ms=$2,consent_receipt_id=$3 WHERE logical_owner_id=$4 AND recording_evidence_id=$5 AND recording_revision=$6 AND run_state=1 AND challenge_expires_at_unix_ms >= $2")
            .bind(to_i64(next_revision)?).bind(write.started_at_unix_ms).bind(write.consent_receipt_id.as_slice())
            .bind(&write.logical_owner_id).bind(write.recording_evidence_id.as_slice()).bind(to_i64(write.expected_revision)?)
            .execute(&mut *tx).await.map_err(storage)?;
        if run.rows_affected() != 1 {
            return Err(PersistenceErrorV1::Conflict);
        }
        insert_realtime(
            &mut tx,
            &write.logical_owner_id,
            &write.recording_evidence_id,
            next_revision,
            &write.realtime,
        )
        .await?;
        tx.commit().await.map_err(storage)?;
        self.get(&write.logical_owner_id, &write.recording_evidence_id)
            .await?
            .ok_or(PersistenceErrorV1::InvalidRow)
    }

    pub async fn mark_materializing(
        &self,
        owner: &str,
        recording_id: &[u8; 16],
        expected_revision: u64,
        host_command_completion: Option<&HostCommandCompletionV1>,
        realtime: &RealtimeTransitionV1,
    ) -> Result<PersistedRecordingRunV1, PersistenceErrorV1> {
        self.transition(TransitionWriteV1 {
            owner,
            recording_id,
            expected_revision,
            expected_state: 2,
            next_state: 3,
            started_at: None,
            error: None,
            host_command_completion,
            outbox: None,
            realtime,
        })
        .await
    }

    pub async fn complete_ready(
        &self,
        owner: &str,
        recording_id: &[u8; 16],
        expected_revision: u64,
        metadata: &TerminalRecordingMetadataV1,
        outbox: &ExactOutboxRecordV1,
        realtime: &RealtimeTransitionV1,
    ) -> Result<PersistedRecordingRunV1, PersistenceErrorV1> {
        validate_terminal(metadata, outbox, realtime)?;
        let mut tx = self.pool.begin().await.map_err(storage)?;
        let next = to_i64(
            expected_revision
                .checked_add(1)
                .ok_or(PersistenceErrorV1::InvalidInput)?,
        )?;
        let result = sqlx::query("UPDATE makosh_data.desktop_call_recording_runs SET recording_revision=$1, run_state=4, ended_at_unix_ms=$2, source_reference_id=$3, source_declared_bytes=$4, source_duration_millis=$5, source_sha256=$6 WHERE logical_owner_id=$7 AND recording_evidence_id=$8 AND recording_revision=$9 AND run_state=3 AND consent_receipt_id=$10")
            .bind(next).bind(metadata.ended_at_unix_ms)
            .bind(metadata.source_reference_id.as_slice()).bind(to_i64(metadata.source_declared_bytes)?)
            .bind(to_i64(metadata.source_duration_millis)?).bind(metadata.source_sha256.as_slice())
            .bind(owner).bind(recording_id.as_slice()).bind(to_i64(expected_revision)?)
            .bind(metadata.consent_receipt_id.as_slice())
            .execute(&mut *tx).await.map_err(storage)?;
        if result.rows_affected() != 1 {
            return Err(PersistenceErrorV1::Conflict);
        }
        insert_outbox(&mut tx, owner, recording_id, outbox).await?;
        insert_realtime(
            &mut tx,
            owner,
            recording_id,
            expected_revision + 1,
            realtime,
        )
        .await?;
        tx.commit().await.map_err(storage)?;
        self.get(owner, recording_id)
            .await?
            .ok_or(PersistenceErrorV1::InvalidRow)
    }

    pub async fn reject(
        &self,
        write: &RejectRecordingWriteV1,
    ) -> Result<PersistedRecordingRunV1, PersistenceErrorV1> {
        if !valid_code(&write.public_error_code) {
            return Err(PersistenceErrorV1::InvalidInput);
        }
        self.transition(TransitionWriteV1 {
            owner: &write.logical_owner_id,
            recording_id: &write.recording_evidence_id,
            expected_revision: write.expected_revision,
            expected_state: state_code(write.expected_state),
            next_state: 5,
            started_at: None,
            error: Some(&write.public_error_code),
            host_command_completion: write.host_command_completion.as_ref(),
            outbox: Some(&write.outbox),
            realtime: &write.realtime,
        })
        .await
    }

    pub async fn claim_host_commands(
        &self,
        claim_sha256: &[u8; 32],
        now_unix_ms: i64,
        lease_millis: i64,
        limit: u32,
    ) -> Result<Vec<LeasedHostCommandV1>, PersistenceErrorV1> {
        if zero(claim_sha256)
            || now_unix_ms <= 0
            || !(1_000..=60_000).contains(&lease_millis)
            || !(1..=16).contains(&limit)
        {
            return Err(PersistenceErrorV1::InvalidInput);
        }
        let rows = sqlx::query("WITH candidates AS (SELECT command_id FROM makosh_data.desktop_call_recording_host_commands WHERE completed_at_unix_ms IS NULL AND (lease_expires_at_unix_ms IS NULL OR lease_expires_at_unix_ms < $1) ORDER BY command_revision, command_id FOR UPDATE SKIP LOCKED LIMIT $2) UPDATE makosh_data.desktop_call_recording_host_commands AS commands SET leased_by_sha256=$3, lease_expires_at_unix_ms=$4 FROM candidates WHERE commands.command_id=candidates.command_id RETURNING commands.command_id, commands.logical_owner_id, commands.recording_evidence_id, commands.command_kind, commands.command_revision")
            .bind(now_unix_ms).bind(i64::from(limit)).bind(claim_sha256.as_slice()).bind(now_unix_ms + lease_millis)
            .fetch_all(&self.pool).await.map_err(storage)?;
        rows.into_iter()
            .map(|row| {
                Ok(LeasedHostCommandV1 {
                    command_id: id16(row.try_get("command_id").map_err(row_error)?)?,
                    logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
                    recording_evidence_id: id16(
                        row.try_get("recording_evidence_id").map_err(row_error)?,
                    )?,
                    command_kind: u16::try_from(
                        row.try_get::<i16, _>("command_kind").map_err(row_error)?,
                    )
                    .map_err(|_| PersistenceErrorV1::InvalidRow)?,
                    command_revision: positive_u64(
                        row.try_get("command_revision").map_err(row_error)?,
                    )?,
                })
            })
            .collect()
    }

    pub async fn request_stop(
        &self,
        owner: &str,
        recording_id: &[u8; 16],
        command_id: [u8; 16],
    ) -> Result<PersistedRecordingRunV1, PersistenceErrorV1> {
        if !valid_owner(owner) || zero(recording_id) || zero(&command_id) {
            return Err(PersistenceErrorV1::InvalidInput);
        }
        let mut tx = self.pool.begin().await.map_err(storage)?;
        let row = sqlx::query(RUN_COLUMNS_BY_RECORDING_FOR_UPDATE)
            .bind(owner)
            .bind(recording_id.as_slice())
            .fetch_optional(&mut *tx)
            .await
            .map_err(storage)?
            .ok_or(PersistenceErrorV1::Conflict)?;
        let run = decode_run(row)?;
        if matches!(
            run.state,
            RecordingStateV1::Ready | RecordingStateV1::Rejected
        ) {
            tx.commit().await.map_err(storage)?;
            return Ok(run);
        }
        sqlx::query("INSERT INTO makosh_data.desktop_call_recording_host_commands (command_id,logical_owner_id,recording_evidence_id,command_kind,command_revision) VALUES ($1,$2,$3,2,$4) ON CONFLICT (command_id) DO NOTHING")
            .bind(command_id.as_slice()).bind(owner).bind(recording_id.as_slice())
            .bind(to_i64(run.recording_revision)?).execute(&mut *tx).await.map_err(storage)?;
        tx.commit().await.map_err(storage)?;
        Ok(run)
    }

    pub async fn complete_host_command(
        &self,
        command_id: [u8; 16],
        claim_sha256: [u8; 32],
        completed_at_unix_ms: i64,
    ) -> Result<(), PersistenceErrorV1> {
        if zero(&command_id) || zero(&claim_sha256) || completed_at_unix_ms <= 0 {
            return Err(PersistenceErrorV1::InvalidInput);
        }
        let result = sqlx::query("UPDATE makosh_data.desktop_call_recording_host_commands SET completed_at_unix_ms=$1 WHERE command_id=$2 AND leased_by_sha256=$3 AND completed_at_unix_ms IS NULL AND lease_expires_at_unix_ms >= $1")
            .bind(completed_at_unix_ms).bind(command_id.as_slice()).bind(claim_sha256.as_slice())
            .execute(&self.pool).await.map_err(storage)?;
        if result.rows_affected() == 1 {
            Ok(())
        } else {
            Err(PersistenceErrorV1::Conflict)
        }
    }

    pub async fn pending_outbox(
        &self,
        limit: u32,
    ) -> Result<Vec<PendingOutboxV1>, PersistenceErrorV1> {
        if !(1..=128).contains(&limit) {
            return Err(PersistenceErrorV1::InvalidInput);
        }
        let rows = sqlx::query("SELECT sequence_id,event_id,logical_owner_id,recording_evidence_id,contract_name,envelope_sha256,exact_envelope_bytes FROM makosh_data.desktop_call_recording_outbox WHERE delivered_at_unix_ms IS NULL ORDER BY sequence_id LIMIT $1")
            .bind(i64::from(limit)).fetch_all(&self.pool).await.map_err(storage)?;
        rows.into_iter().map(decode_outbox).collect()
    }

    pub async fn mark_outbox_delivered(
        &self,
        event_id: [u8; 16],
        envelope_sha256: [u8; 32],
        delivered_at_unix_ms: i64,
    ) -> Result<(), PersistenceErrorV1> {
        if zero(&event_id) || zero(&envelope_sha256) || delivered_at_unix_ms <= 0 {
            return Err(PersistenceErrorV1::InvalidInput);
        }
        let result = sqlx::query("UPDATE makosh_data.desktop_call_recording_outbox SET delivered_at_unix_ms=$1 WHERE event_id=$2 AND envelope_sha256=$3 AND delivered_at_unix_ms IS NULL")
            .bind(delivered_at_unix_ms).bind(event_id.as_slice()).bind(envelope_sha256.as_slice())
            .execute(&self.pool).await.map_err(storage)?;
        if result.rows_affected() == 1 {
            Ok(())
        } else {
            Err(PersistenceErrorV1::Conflict)
        }
    }

    pub async fn pending_realtime(
        &self,
        limit: u32,
    ) -> Result<Vec<PendingRealtimeV1>, PersistenceErrorV1> {
        if !(1..=128).contains(&limit) {
            return Err(PersistenceErrorV1::InvalidInput);
        }
        let rows = sqlx::query("SELECT sequence_id,logical_owner_id,recording_evidence_id,recording_revision,occurred_at_unix_ms,payload_bytes,payload_sha256 FROM makosh_data.desktop_call_recording_realtime WHERE published_at_unix_ms IS NULL ORDER BY sequence_id LIMIT $1")
            .bind(i64::from(limit)).fetch_all(&self.pool).await.map_err(storage)?;
        rows.into_iter().map(decode_realtime).collect()
    }

    pub async fn mark_realtime_published(
        &self,
        sequence_id: i64,
        payload_sha256: [u8; 32],
        published_at_unix_ms: i64,
    ) -> Result<(), PersistenceErrorV1> {
        if sequence_id <= 0 || zero(&payload_sha256) || published_at_unix_ms <= 0 {
            return Err(PersistenceErrorV1::InvalidInput);
        }
        let result = sqlx::query("UPDATE makosh_data.desktop_call_recording_realtime SET published_at_unix_ms=$1 WHERE sequence_id=$2 AND payload_sha256=$3 AND published_at_unix_ms IS NULL")
            .bind(published_at_unix_ms).bind(sequence_id).bind(payload_sha256.as_slice())
            .execute(&self.pool).await.map_err(storage)?;
        if result.rows_affected() == 1 {
            Ok(())
        } else {
            Err(PersistenceErrorV1::Conflict)
        }
    }

    async fn transition(
        &self,
        write: TransitionWriteV1<'_>,
    ) -> Result<PersistedRecordingRunV1, PersistenceErrorV1> {
        if !valid_owner(write.owner)
            || zero(write.recording_id)
            || write.expected_revision == 0
            || write.realtime.payload_bytes.is_empty()
        {
            return Err(PersistenceErrorV1::InvalidInput);
        }
        let mut tx = self.pool.begin().await.map_err(storage)?;
        if let Some(completion) = write.host_command_completion {
            if zero(&completion.command_id)
                || zero(&completion.claim_sha256)
                || completion.completed_at_unix_ms <= 0
            {
                return Err(PersistenceErrorV1::InvalidInput);
            }
            let result = sqlx::query("UPDATE makosh_data.desktop_call_recording_host_commands SET completed_at_unix_ms=$1 WHERE command_id=$2 AND logical_owner_id=$3 AND recording_evidence_id=$4 AND leased_by_sha256=$5 AND completed_at_unix_ms IS NULL AND lease_expires_at_unix_ms >= $1")
                .bind(completion.completed_at_unix_ms).bind(completion.command_id.as_slice())
                .bind(write.owner).bind(write.recording_id.as_slice())
                .bind(completion.claim_sha256.as_slice()).execute(&mut *tx).await.map_err(storage)?;
            if result.rows_affected() != 1 {
                return Err(PersistenceErrorV1::Conflict);
            }
        }
        let result = sqlx::query("UPDATE makosh_data.desktop_call_recording_runs SET recording_revision=$1, run_state=$2, started_at_unix_ms=COALESCE($3,started_at_unix_ms), public_error_code=$4 WHERE logical_owner_id=$5 AND recording_evidence_id=$6 AND recording_revision=$7 AND run_state=$8")
            .bind(to_i64(write.expected_revision + 1)?).bind(write.next_state).bind(write.started_at).bind(write.error).bind(write.owner).bind(write.recording_id.as_slice()).bind(to_i64(write.expected_revision)?).bind(write.expected_state)
            .execute(&mut *tx).await.map_err(storage)?;
        if result.rows_affected() != 1 {
            return Err(PersistenceErrorV1::Conflict);
        }
        if let Some(value) = write.outbox {
            insert_outbox(&mut tx, write.owner, write.recording_id, value).await?;
        }
        insert_realtime(
            &mut tx,
            write.owner,
            write.recording_id,
            write.expected_revision + 1,
            write.realtime,
        )
        .await?;
        tx.commit().await.map_err(storage)?;
        self.get(write.owner, write.recording_id)
            .await?
            .ok_or(PersistenceErrorV1::InvalidRow)
    }
}

struct TransitionWriteV1<'a> {
    owner: &'a str,
    recording_id: &'a [u8; 16],
    expected_revision: u64,
    expected_state: i16,
    next_state: i16,
    started_at: Option<i64>,
    error: Option<&'a str>,
    host_command_completion: Option<&'a crate::HostCommandCompletionV1>,
    outbox: Option<&'a ExactOutboxRecordV1>,
    realtime: &'a RealtimeTransitionV1,
}

const RUN_COLUMNS_BY_RECORDING: &str = "SELECT logical_owner_id,operation_id,request_sha256,call_evidence_id,call_evidence_revision,recording_evidence_id,recording_revision,run_state,device_actor_sha256,challenge_id,challenge_expires_at_unix_ms,maximum_duration_millis,consent_policy_revision,started_at_unix_ms,ended_at_unix_ms,consent_receipt_id,source_reference_id,source_declared_bytes,source_duration_millis,source_sha256,public_error_code FROM makosh_data.desktop_call_recording_runs WHERE logical_owner_id=$1 AND recording_evidence_id=$2";
const RUN_COLUMNS_BY_RECORDING_FOR_UPDATE: &str = "SELECT logical_owner_id,operation_id,request_sha256,call_evidence_id,call_evidence_revision,recording_evidence_id,recording_revision,run_state,device_actor_sha256,challenge_id,challenge_expires_at_unix_ms,maximum_duration_millis,consent_policy_revision,started_at_unix_ms,ended_at_unix_ms,consent_receipt_id,source_reference_id,source_declared_bytes,source_duration_millis,source_sha256,public_error_code FROM makosh_data.desktop_call_recording_runs WHERE logical_owner_id=$1 AND recording_evidence_id=$2 FOR UPDATE";
const RUN_COLUMNS_BY_OPERATION: &str = "SELECT logical_owner_id,operation_id,request_sha256,call_evidence_id,call_evidence_revision,recording_evidence_id,recording_revision,run_state,device_actor_sha256,challenge_id,challenge_expires_at_unix_ms,maximum_duration_millis,consent_policy_revision,started_at_unix_ms,ended_at_unix_ms,consent_receipt_id,source_reference_id,source_declared_bytes,source_duration_millis,source_sha256,public_error_code FROM makosh_data.desktop_call_recording_runs WHERE logical_owner_id=$1 AND operation_id=$2";

async fn load_by_operation(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
    operation: &[u8; 16],
) -> Result<Option<PersistedRecordingRunV1>, PersistenceErrorV1> {
    sqlx::query(RUN_COLUMNS_BY_OPERATION)
        .bind(owner)
        .bind(operation.as_slice())
        .fetch_optional(&mut **tx)
        .await
        .map_err(storage)?
        .map(decode_run)
        .transpose()
}

async fn insert_realtime(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
    recording: &[u8; 16],
    revision: u64,
    value: &RealtimeTransitionV1,
) -> Result<(), PersistenceErrorV1> {
    if value.payload_bytes.is_empty()
        || value.payload_bytes.len() > 4096
        || Sha256::digest(&value.payload_bytes).as_slice() != value.payload_sha256
    {
        return Err(PersistenceErrorV1::InvalidInput);
    }
    sqlx::query("INSERT INTO makosh_data.desktop_call_recording_realtime (logical_owner_id,recording_evidence_id,recording_revision,occurred_at_unix_ms,payload_bytes,payload_sha256) VALUES ($1,$2,$3,$4,$5,$6)")
        .bind(owner).bind(recording.as_slice()).bind(to_i64(revision)?).bind(value.occurred_at_unix_ms).bind(&value.payload_bytes).bind(value.payload_sha256.as_slice()).execute(&mut **tx).await.map_err(storage)?;
    Ok(())
}
async fn insert_outbox(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
    recording: &[u8; 16],
    value: &ExactOutboxRecordV1,
) -> Result<(), PersistenceErrorV1> {
    if zero(&value.event_id)
        || !valid_code(&value.contract_name)
        || value.exact_envelope_bytes.is_empty()
        || value.exact_envelope_bytes.len() > 131072
        || Sha256::digest(&value.exact_envelope_bytes).as_slice() != value.envelope_sha256
    {
        return Err(PersistenceErrorV1::InvalidInput);
    }
    sqlx::query("INSERT INTO makosh_data.desktop_call_recording_outbox (event_id,logical_owner_id,recording_evidence_id,contract_name,envelope_sha256,exact_envelope_bytes) VALUES ($1,$2,$3,$4,$5,$6)")
        .bind(value.event_id.as_slice()).bind(owner).bind(recording.as_slice()).bind(&value.contract_name).bind(value.envelope_sha256.as_slice()).bind(&value.exact_envelope_bytes).execute(&mut **tx).await.map_err(storage)?;
    Ok(())
}

fn decode_run(row: sqlx::postgres::PgRow) -> Result<PersistedRecordingRunV1, PersistenceErrorV1> {
    Ok(PersistedRecordingRunV1 {
        logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
        operation_id: id16(row.try_get("operation_id").map_err(row_error)?)?,
        request_sha256: id32(row.try_get("request_sha256").map_err(row_error)?)?,
        call_evidence_id: id16(row.try_get("call_evidence_id").map_err(row_error)?)?,
        call_evidence_revision: positive_u64(
            row.try_get("call_evidence_revision").map_err(row_error)?,
        )?,
        recording_evidence_id: id16(row.try_get("recording_evidence_id").map_err(row_error)?)?,
        recording_revision: positive_u64(row.try_get("recording_revision").map_err(row_error)?)?,
        state: decode_state(row.try_get("run_state").map_err(row_error)?)?,
        device_actor_sha256: id32(row.try_get("device_actor_sha256").map_err(row_error)?)?,
        challenge_id: id16(row.try_get("challenge_id").map_err(row_error)?)?,
        challenge_expires_at_unix_ms: row
            .try_get("challenge_expires_at_unix_ms")
            .map_err(row_error)?,
        maximum_duration_millis: positive_u64(
            row.try_get("maximum_duration_millis").map_err(row_error)?,
        )?,
        consent_policy_revision: u32::try_from(
            row.try_get::<i32, _>("consent_policy_revision")
                .map_err(row_error)?,
        )
        .map_err(|_| PersistenceErrorV1::InvalidRow)?,
        started_at_unix_ms: row.try_get("started_at_unix_ms").map_err(row_error)?,
        ended_at_unix_ms: row.try_get("ended_at_unix_ms").map_err(row_error)?,
        consent_receipt_id: optional_id16(row.try_get("consent_receipt_id").map_err(row_error)?)?,
        source_reference_id: optional_id16(row.try_get("source_reference_id").map_err(row_error)?)?,
        source_declared_bytes: optional_u64(
            row.try_get("source_declared_bytes").map_err(row_error)?,
        )?,
        source_duration_millis: optional_u64(
            row.try_get("source_duration_millis").map_err(row_error)?,
        )?,
        source_sha256: optional_id32(row.try_get("source_sha256").map_err(row_error)?)?,
        public_error_code: row.try_get("public_error_code").map_err(row_error)?,
    })
}
fn decode_outbox(row: sqlx::postgres::PgRow) -> Result<PendingOutboxV1, PersistenceErrorV1> {
    Ok(PendingOutboxV1 {
        sequence_id: row.try_get("sequence_id").map_err(row_error)?,
        event_id: id16(row.try_get("event_id").map_err(row_error)?)?,
        logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
        recording_evidence_id: id16(row.try_get("recording_evidence_id").map_err(row_error)?)?,
        contract_name: row.try_get("contract_name").map_err(row_error)?,
        envelope_sha256: id32(row.try_get("envelope_sha256").map_err(row_error)?)?,
        exact_envelope_bytes: row.try_get("exact_envelope_bytes").map_err(row_error)?,
    })
}
fn decode_realtime(row: sqlx::postgres::PgRow) -> Result<PendingRealtimeV1, PersistenceErrorV1> {
    Ok(PendingRealtimeV1 {
        sequence_id: row.try_get("sequence_id").map_err(row_error)?,
        logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
        recording_evidence_id: id16(row.try_get("recording_evidence_id").map_err(row_error)?)?,
        recording_revision: positive_u64(row.try_get("recording_revision").map_err(row_error)?)?,
        occurred_at_unix_ms: row.try_get("occurred_at_unix_ms").map_err(row_error)?,
        payload_bytes: row.try_get("payload_bytes").map_err(row_error)?,
        payload_sha256: id32(row.try_get("payload_sha256").map_err(row_error)?)?,
    })
}
fn same_request(a: &PersistedRecordingRunV1, b: &NewRecordingRunV1) -> bool {
    a.request_sha256 == b.request_sha256
        && a.call_evidence_id == b.call_evidence_id
        && a.call_evidence_revision == b.call_evidence_revision
        && a.recording_evidence_id == b.recording_evidence_id
        && a.device_actor_sha256 == b.device_actor_sha256
        && a.challenge_id == b.challenge_id
        && a.maximum_duration_millis == b.maximum_duration_millis
        && a.consent_policy_revision == b.consent_policy_revision
}
fn validate_new(
    v: &NewRecordingRunV1,
    c: &[u8; 16],
    r: &RealtimeTransitionV1,
) -> Result<(), PersistenceErrorV1> {
    if !valid_owner(&v.logical_owner_id)
        || zero(&v.operation_id)
        || zero(&v.request_sha256)
        || zero(&v.call_evidence_id)
        || zero(&v.recording_evidence_id)
        || zero(&v.device_actor_sha256)
        || zero(&v.challenge_id)
        || zero(c)
        || v.call_evidence_revision == 0
        || v.maximum_duration_millis == 0
        || v.consent_policy_revision == 0
        || r.occurred_at_unix_ms <= 0
    {
        return Err(PersistenceErrorV1::InvalidInput);
    }
    Ok(())
}
fn validate_terminal(
    v: &TerminalRecordingMetadataV1,
    o: &ExactOutboxRecordV1,
    r: &RealtimeTransitionV1,
) -> Result<(), PersistenceErrorV1> {
    if v.ended_at_unix_ms <= 0
        || zero(&v.consent_receipt_id)
        || zero(&v.source_reference_id)
        || v.source_declared_bytes == 0
        || v.source_duration_millis == 0
        || zero(&v.source_sha256)
        || r.occurred_at_unix_ms <= 0
    {
        return Err(PersistenceErrorV1::InvalidInput);
    }
    if zero(&o.event_id) {
        return Err(PersistenceErrorV1::InvalidInput);
    }
    Ok(())
}
fn state_code(v: RecordingStateV1) -> i16 {
    match v {
        RecordingStateV1::AwaitingConsent => 1,
        RecordingStateV1::Capturing => 2,
        RecordingStateV1::Materializing => 3,
        RecordingStateV1::Ready => 4,
        RecordingStateV1::Rejected => 5,
    }
}
fn decode_state(v: i16) -> Result<RecordingStateV1, PersistenceErrorV1> {
    match v {
        1 => Ok(RecordingStateV1::AwaitingConsent),
        2 => Ok(RecordingStateV1::Capturing),
        3 => Ok(RecordingStateV1::Materializing),
        4 => Ok(RecordingStateV1::Ready),
        5 => Ok(RecordingStateV1::Rejected),
        _ => Err(PersistenceErrorV1::InvalidRow),
    }
}
fn valid_owner(v: &str) -> bool {
    !v.is_empty()
        && v.len() <= 128
        && v.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}
fn valid_code(v: &str) -> bool {
    valid_owner(v) && v.len() <= 96
}
fn zero<const N: usize>(v: &[u8; N]) -> bool {
    v.iter().all(|b| *b == 0)
}
fn id16(v: Vec<u8>) -> Result<[u8; 16], PersistenceErrorV1> {
    v.try_into().map_err(|_| PersistenceErrorV1::InvalidRow)
}
fn id32(v: Vec<u8>) -> Result<[u8; 32], PersistenceErrorV1> {
    v.try_into().map_err(|_| PersistenceErrorV1::InvalidRow)
}
fn optional_id16(v: Option<Vec<u8>>) -> Result<Option<[u8; 16]>, PersistenceErrorV1> {
    v.map(id16).transpose()
}
fn optional_id32(v: Option<Vec<u8>>) -> Result<Option<[u8; 32]>, PersistenceErrorV1> {
    v.map(id32).transpose()
}
fn positive_u64(v: i64) -> Result<u64, PersistenceErrorV1> {
    u64::try_from(v)
        .ok()
        .filter(|v| *v > 0)
        .ok_or(PersistenceErrorV1::InvalidRow)
}
fn optional_u64(v: Option<i64>) -> Result<Option<u64>, PersistenceErrorV1> {
    v.map(|n| u64::try_from(n).map_err(|_| PersistenceErrorV1::InvalidRow))
        .transpose()
}
fn to_i64(v: u64) -> Result<i64, PersistenceErrorV1> {
    i64::try_from(v).map_err(|_| PersistenceErrorV1::InvalidInput)
}
fn storage(_: sqlx::Error) -> PersistenceErrorV1 {
    PersistenceErrorV1::StorageUnavailable
}
fn row_error(_: sqlx::Error) -> PersistenceErrorV1 {
    PersistenceErrorV1::InvalidRow
}
