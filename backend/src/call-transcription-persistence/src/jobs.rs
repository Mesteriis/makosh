use makosh_call_transcription_core::{
    CallTranscriptionRejectionV1, CallTranscriptionStateV1, CallTranscriptionTransitionV1,
    PendingTranscriptV1, transition_v1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    CallTranscriptionJobLeaseV1, CallTranscriptionPersistenceErrorV1,
    CallTranscriptionPersistenceV1, ClaimedCallTranscriptionJobV1, CompleteSourceCleanupV1,
    MaterializeTranscriptV1, PersistSttResultV1, RebindTranscriptMaterializationV1,
    call_transcription_job_id_v1,
    model::{
        CALL_TRANSCRIPTION_MAX_ATTEMPTS_V1, CALL_TRANSCRIPTION_MAX_LEASE_MILLIS_V1, valid_id16,
        valid_outbox, valid_owner, valid_sha256, valid_timestamp_millis, valid_worker,
    },
    outbox::insert_outbox,
    realtime::append_realtime,
    repository::{
        completeness_code, exact_update, id16, id32, invalid_input, language_code,
        load_run_for_update, row_error, signed, state_code, storage_error, update_rejection,
    },
};

pub(crate) async fn enqueue_stt_job(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    stt_request_id: [u8; 16],
    stt_request_digest: [u8; 32],
    created_at_unix_millis: i64,
) -> Result<(), CallTranscriptionPersistenceErrorV1> {
    let job_id = call_transcription_job_id_v1(run_id, stt_request_id);
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.call_transcription_jobs (
           logical_owner_id,job_id,run_id,stt_request_id,stt_request_digest,
           state,attempt_count,max_attempts,lease_fence,created_at_unix_millis,
           updated_at_unix_millis
         ) VALUES ($1,$2,$3,$4,$5,1,0,$6,0,$7,$7)
         ON CONFLICT (logical_owner_id,run_id) DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(job_id.as_slice())
    .bind(run_id.as_slice())
    .bind(stt_request_id.as_slice())
    .bind(stt_request_digest.as_slice())
    .bind(i32::try_from(CALL_TRANSCRIPTION_MAX_ATTEMPTS_V1).map_err(invalid_input)?)
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?
    .rows_affected();
    if inserted == 1 {
        return Ok(());
    }
    let existing = sqlx::query(
        "SELECT job_id,stt_request_id,stt_request_digest FROM
         makosh_data.call_transcription_jobs WHERE logical_owner_id=$1 AND run_id=$2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if id16(existing.try_get("job_id").map_err(row_error)?)? != job_id
        || id16(existing.try_get("stt_request_id").map_err(row_error)?)? != stt_request_id
        || id32(existing.try_get("stt_request_digest").map_err(row_error)?)? != stt_request_digest
    {
        return Err(CallTranscriptionPersistenceErrorV1::RequestConflict);
    }
    Ok(())
}

impl CallTranscriptionPersistenceV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn claim_next_job(
        &self,
        logical_owner_id: &str,
        worker_id: &str,
        runtime_generation: u64,
        grant_epoch: u64,
        now_unix_millis: i64,
        lease_millis: u64,
    ) -> Result<Option<ClaimedCallTranscriptionJobV1>, CallTranscriptionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_worker(worker_id)
            || runtime_generation == 0
            || grant_epoch == 0
            || !valid_timestamp_millis(now_unix_millis)
            || !(1..=CALL_TRANSCRIPTION_MAX_LEASE_MILLIS_V1).contains(&lease_millis)
        {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
        }
        let expires_at = now_unix_millis
            .checked_add(i64::try_from(lease_millis).map_err(invalid_input)?)
            .ok_or(CallTranscriptionPersistenceErrorV1::InvalidInput)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let row = sqlx::query(
            "SELECT job_id FROM makosh_data.call_transcription_jobs
             WHERE logical_owner_id=$1 AND attempt_count<max_attempts
               AND (state=1 OR (state=2 AND lease_expires_at_unix_millis<$2))
             ORDER BY created_at_unix_millis,job_id FOR UPDATE SKIP LOCKED LIMIT 1",
        )
        .bind(logical_owner_id)
        .bind(now_unix_millis)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?;
        let Some(row) = row else {
            transaction.commit().await.map_err(storage_error)?;
            return Ok(None);
        };
        let job_id = id16(row.try_get("job_id").map_err(row_error)?)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.call_transcription_jobs SET state=2,
             attempt_count=attempt_count+1,worker_id=$1,runtime_generation=$2,
             grant_epoch=$3,lease_fence=lease_fence+1,lease_expires_at_unix_millis=$4,
             updated_at_unix_millis=$5 WHERE logical_owner_id=$6 AND job_id=$7
             RETURNING run_id,stt_request_id,stt_request_digest,attempt_count,max_attempts,
             lease_fence,lease_expires_at_unix_millis",
        )
        .bind(worker_id)
        .bind(signed(runtime_generation)?)
        .bind(signed(grant_epoch)?)
        .bind(expires_at)
        .bind(now_unix_millis)
        .bind(logical_owner_id)
        .bind(job_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage_error)?;
        let run_id = id16(updated.try_get("run_id").map_err(row_error)?)?;
        let run = load_run_for_update(&mut transaction, logical_owner_id, run_id)
            .await?
            .ok_or(CallTranscriptionPersistenceErrorV1::InvalidRow)?;
        if run.status.state != CallTranscriptionStateV1::AwaitingStt {
            return Err(CallTranscriptionPersistenceErrorV1::RevisionConflict);
        }
        let recording_source = run
            .recording_source
            .ok_or(CallTranscriptionPersistenceErrorV1::InvalidRow)?;
        let claimed = ClaimedCallTranscriptionJobV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            job_id,
            run_id,
            stt_request_id: id16(updated.try_get("stt_request_id").map_err(row_error)?)?,
            stt_request_digest: id32(updated.try_get("stt_request_digest").map_err(row_error)?)?,
            draft: run.draft,
            recording_source,
            attempt_count: u32::try_from(
                updated
                    .try_get::<i32, _>("attempt_count")
                    .map_err(row_error)?,
            )
            .map_err(row_error)?,
            max_attempts: u32::try_from(
                updated
                    .try_get::<i32, _>("max_attempts")
                    .map_err(row_error)?,
            )
            .map_err(row_error)?,
            lease: CallTranscriptionJobLeaseV1 {
                worker_id: worker_id.to_owned(),
                runtime_generation,
                grant_epoch,
                lease_fence: u64::try_from(
                    updated
                        .try_get::<i64, _>("lease_fence")
                        .map_err(row_error)?,
                )
                .map_err(row_error)?,
                lease_expires_at_unix_millis: updated
                    .try_get("lease_expires_at_unix_millis")
                    .map_err(row_error)?,
            },
        };
        if claimed.stt_request_digest != run.status.stt_request_digest.unwrap_or([0; 32]) {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidRow);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(Some(claimed))
    }

    pub async fn persist_stt_result(
        &self,
        input: PersistSttResultV1,
    ) -> Result<(), CallTranscriptionPersistenceErrorV1> {
        validate_stt_result(&input)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let job = lock_leased_job(
            &mut transaction,
            &input.logical_owner_id,
            input.job_id,
            &input.lease,
            input.occurred_at_unix_millis,
        )
        .await?;
        let current = load_run_for_update(&mut transaction, &input.logical_owner_id, job.run_id)
            .await?
            .ok_or(CallTranscriptionPersistenceErrorV1::InvalidRow)?;
        let next = transition_v1(&current.draft, &current.status, input.transition.clone())
            .map_err(|_| CallTranscriptionPersistenceErrorV1::InvalidTransition)?;
        match &input.transition {
            CallTranscriptionTransitionV1::SttCompleted(pending) => {
                let result_receipt = input
                    .result_receipt_sha256
                    .ok_or(CallTranscriptionPersistenceErrorV1::InvalidInput)?;
                persist_pending_result(
                    &mut transaction,
                    &input.logical_owner_id,
                    job.run_id,
                    current.status.state_revision,
                    &next,
                    pending,
                    result_receipt,
                    input.occurred_at_unix_millis,
                )
                .await?;
                finish_job(
                    &mut transaction,
                    &input.logical_owner_id,
                    input.job_id,
                    &input.lease,
                    3,
                    Some(result_receipt),
                    input.occurred_at_unix_millis,
                )
                .await?;
            }
            CallTranscriptionTransitionV1::Reject(_) => {
                update_rejection(
                    &mut transaction,
                    &input.logical_owner_id,
                    job.run_id,
                    current.status.state_revision,
                    &next,
                    input.occurred_at_unix_millis,
                )
                .await?;
                finish_job(
                    &mut transaction,
                    &input.logical_owner_id,
                    input.job_id,
                    &input.lease,
                    4,
                    None,
                    input.occurred_at_unix_millis,
                )
                .await?;
            }
            _ => return Err(CallTranscriptionPersistenceErrorV1::InvalidInput),
        }
        if let Some(outbox) = input.outbox.as_ref() {
            insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                outbox,
                input.occurred_at_unix_millis,
            )
            .await?;
        }
        append_realtime(
            &mut transaction,
            &input.logical_owner_id,
            job.run_id,
            &next,
            input.occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)
    }

    pub async fn materialize_transcript(
        &self,
        input: MaterializeTranscriptV1,
    ) -> Result<(), CallTranscriptionPersistenceErrorV1> {
        if !valid_materialization(&input) {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let job = sqlx::query(
            "SELECT run_id,runtime_generation,grant_epoch FROM
             makosh_data.call_transcription_jobs WHERE logical_owner_id=$1 AND job_id=$2
             AND state=3 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.job_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        .ok_or(CallTranscriptionPersistenceErrorV1::StaleFence)?;
        if id16(job.try_get("run_id").map_err(row_error)?)? != input.run_id
            || u64::try_from(
                job.try_get::<i64, _>("runtime_generation")
                    .map_err(row_error)?,
            )
            .map_err(row_error)?
                != input.runtime_generation
            || u64::try_from(job.try_get::<i64, _>("grant_epoch").map_err(row_error)?)
                .map_err(row_error)?
                != input.grant_epoch
        {
            return Err(CallTranscriptionPersistenceErrorV1::StaleFence);
        }
        let current = load_run_for_update(&mut transaction, &input.logical_owner_id, input.run_id)
            .await?
            .ok_or(CallTranscriptionPersistenceErrorV1::NotFound)?;
        let pending = current
            .status
            .pending_transcript
            .as_ref()
            .ok_or(CallTranscriptionPersistenceErrorV1::InvalidRow)?;
        if pending.transcript_reference_id != input.artifact_reference_id {
            return Err(CallTranscriptionPersistenceErrorV1::RevisionConflict);
        }
        let next = transition_v1(
            &current.draft,
            &current.status,
            CallTranscriptionTransitionV1::TranscriptMaterialized {
                artifact_id: input.artifact_id,
            },
        )
        .map_err(|_| CallTranscriptionPersistenceErrorV1::InvalidTransition)?;
        let artifact = next
            .artifact
            .as_ref()
            .ok_or(CallTranscriptionPersistenceErrorV1::InvalidTransition)?;
        let changed = sqlx::query(
            "UPDATE makosh_data.call_transcription_runs SET state=$1,state_revision=$2,
             pending_transcript_reference_id=NULL,pending_transcript_sha256=NULL,
             pending_transcript_size_bytes=NULL,pending_detected_language=NULL,
             pending_duration_millis=NULL,pending_segment_count=NULL,
             pending_completeness=NULL,pending_confidence_basis_points=NULL,
             artifact_id=$3,artifact_reference_id=$4,artifact_receipt_sha256=$5,
             artifact_transcript_sha256=$6,artifact_transcript_size_bytes=$7,
             artifact_detected_language=$8,artifact_duration_millis=$9,
             artifact_segment_count=$10,artifact_completeness=$11,
             artifact_confidence_basis_points=$12,artifact_runtime_generation=$13,
             artifact_grant_epoch=$14,updated_at_unix_millis=$15
             WHERE logical_owner_id=$16 AND run_id=$17 AND state_revision=$18",
        )
        .bind(state_code(next.state))
        .bind(signed(next.state_revision)?)
        .bind(artifact.artifact_id.as_slice())
        .bind(input.artifact_reference_id.as_slice())
        .bind(input.artifact_receipt_sha256.as_slice())
        .bind(artifact.transcript_sha256.as_slice())
        .bind(signed(artifact.transcript_size_bytes)?)
        .bind(language_code(artifact.detected_language))
        .bind(signed(artifact.duration_millis)?)
        .bind(i32::try_from(artifact.segment_count).map_err(invalid_input)?)
        .bind(completeness_code(artifact.completeness))
        .bind(i32::try_from(artifact.confidence_basis_points).map_err(invalid_input)?)
        .bind(signed(input.runtime_generation)?)
        .bind(signed(input.grant_epoch)?)
        .bind(input.occurred_at_unix_millis)
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(signed(current.status.state_revision)?)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        exact_update(changed)?;
        if let Some(outbox) = input.outbox.as_ref() {
            insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                outbox,
                input.occurred_at_unix_millis,
            )
            .await?;
        }
        append_realtime(
            &mut transaction,
            &input.logical_owner_id,
            input.run_id,
            &next,
            input.occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)
    }

    pub async fn rebind_transcript_materialization(
        &self,
        logical_owner_id: &str,
        input: RebindTranscriptMaterializationV1,
    ) -> Result<(), CallTranscriptionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id16(&input.run_id)
            || !valid_id16(&input.job_id)
            || !valid_id16(&input.transcript_reference_id)
            || !valid_sha256(&input.transcript_receipt_sha256)
            || !valid_sha256(&input.stt_result_receipt_sha256)
            || input.runtime_generation == 0
            || input.grant_epoch == 0
            || !valid_timestamp_millis(input.rebound_at_unix_millis)
        {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let row = sqlx::query(
            "SELECT j.run_id,j.result_receipt_sha256,r.state,
             r.pending_transcript_reference_id,r.pending_transcript_sha256,
             r.stt_result_receipt_sha256
             FROM makosh_data.call_transcription_jobs j
             JOIN makosh_data.call_transcription_runs r
               ON r.logical_owner_id=j.logical_owner_id AND r.run_id=j.run_id
             WHERE j.logical_owner_id=$1 AND j.job_id=$2 AND j.state=3
             FOR UPDATE OF j,r",
        )
        .bind(logical_owner_id)
        .bind(input.job_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        .ok_or(CallTranscriptionPersistenceErrorV1::StaleFence)?;
        let exact = id16(row.try_get("run_id").map_err(row_error)?)? == input.run_id
            && row.try_get::<i16, _>("state").map_err(row_error)?
                == state_code(CallTranscriptionStateV1::MaterializingTranscript)
            && id16(
                row.try_get("pending_transcript_reference_id")
                    .map_err(row_error)?,
            )? == input.transcript_reference_id
            && id32(
                row.try_get("pending_transcript_sha256")
                    .map_err(row_error)?,
            )? == input.transcript_receipt_sha256
            && id32(row.try_get("result_receipt_sha256").map_err(row_error)?)?
                == input.stt_result_receipt_sha256
            && id32(
                row.try_get("stt_result_receipt_sha256")
                    .map_err(row_error)?,
            )? == input.stt_result_receipt_sha256;
        if !exact {
            return Err(CallTranscriptionPersistenceErrorV1::RevisionConflict);
        }
        let changed = sqlx::query(
            "UPDATE makosh_data.call_transcription_jobs SET runtime_generation=$1,
             grant_epoch=$2,updated_at_unix_millis=$3
             WHERE logical_owner_id=$4 AND job_id=$5 AND state=3",
        )
        .bind(signed(input.runtime_generation)?)
        .bind(signed(input.grant_epoch)?)
        .bind(input.rebound_at_unix_millis)
        .bind(logical_owner_id)
        .bind(input.job_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        exact_update(changed)?;
        transaction.commit().await.map_err(storage_error)
    }

    pub async fn complete_source_cleanup(
        &self,
        logical_owner_id: &str,
        input: CompleteSourceCleanupV1,
    ) -> Result<(), CallTranscriptionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id16(&input.run_id)
            || !valid_id16(&input.source_reference_id)
            || !valid_sha256(&input.source_receipt_sha256)
            || !valid_timestamp_millis(input.completed_at_unix_millis)
        {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
        }
        let changed = sqlx::query(
            "UPDATE makosh_data.call_transcription_runs SET
             source_cleanup_completed_at_unix_millis=$1,updated_at_unix_millis=$1
             WHERE logical_owner_id=$2 AND run_id=$3 AND state IN (5,6)
               AND source_reference_id=$4 AND source_receipt_sha256=$5
               AND source_cleanup_completed_at_unix_millis IS NULL
               AND updated_at_unix_millis<=$1",
        )
        .bind(input.completed_at_unix_millis)
        .bind(logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(input.source_reference_id.as_slice())
        .bind(input.source_receipt_sha256.as_slice())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?
        .rows_affected();
        exact_update(changed)
    }

    pub async fn recover_expired_jobs(
        &self,
        logical_owner_id: &str,
        now_unix_millis: i64,
    ) -> Result<u32, CallTranscriptionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || !valid_timestamp_millis(now_unix_millis) {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let retried = sqlx::query(
            "UPDATE makosh_data.call_transcription_jobs SET state=1,worker_id=NULL,
             runtime_generation=NULL,grant_epoch=NULL,lease_expires_at_unix_millis=NULL,
             updated_at_unix_millis=$2 WHERE logical_owner_id=$1 AND state=2
               AND lease_expires_at_unix_millis<=$2 AND attempt_count<max_attempts",
        )
        .bind(logical_owner_id)
        .bind(now_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        let exhausted = sqlx::query(
            "SELECT job_id,run_id FROM makosh_data.call_transcription_jobs
             WHERE logical_owner_id=$1 AND state=2 AND lease_expires_at_unix_millis<=$2
               AND attempt_count>=max_attempts ORDER BY job_id FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(now_unix_millis)
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage_error)?;
        for row in &exhausted {
            let job_id = id16(row.try_get("job_id").map_err(row_error)?)?;
            let run_id = id16(row.try_get("run_id").map_err(row_error)?)?;
            let current = load_run_for_update(&mut transaction, logical_owner_id, run_id)
                .await?
                .ok_or(CallTranscriptionPersistenceErrorV1::InvalidRow)?;
            let next = transition_v1(
                &current.draft,
                &current.status,
                CallTranscriptionTransitionV1::Reject(CallTranscriptionRejectionV1::SttRejected),
            )
            .map_err(|_| CallTranscriptionPersistenceErrorV1::InvalidTransition)?;
            update_rejection(
                &mut transaction,
                logical_owner_id,
                run_id,
                current.status.state_revision,
                &next,
                now_unix_millis,
            )
            .await?;
            let changed = sqlx::query(
                "UPDATE makosh_data.call_transcription_jobs SET state=4,worker_id=NULL,
                 lease_expires_at_unix_millis=NULL,updated_at_unix_millis=$3
                 WHERE logical_owner_id=$1 AND job_id=$2 AND state=2",
            )
            .bind(logical_owner_id)
            .bind(job_id.as_slice())
            .bind(now_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(storage_error)?
            .rows_affected();
            exact_update(changed)?;
            append_realtime(
                &mut transaction,
                logical_owner_id,
                run_id,
                &next,
                now_unix_millis,
            )
            .await?;
        }
        let total = retried
            .checked_add(u64::try_from(exhausted.len()).map_err(row_error)?)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(CallTranscriptionPersistenceErrorV1::InvalidRow)?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(total)
    }
}

struct LockedJobV1 {
    run_id: [u8; 16],
}

async fn lock_leased_job(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    job_id: [u8; 16],
    lease: &CallTranscriptionJobLeaseV1,
    now_unix_millis: i64,
) -> Result<LockedJobV1, CallTranscriptionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT run_id FROM makosh_data.call_transcription_jobs
         WHERE logical_owner_id=$1 AND job_id=$2 AND state=2 AND worker_id=$3
           AND runtime_generation=$4 AND grant_epoch=$5 AND lease_fence=$6
           AND lease_expires_at_unix_millis>=$7 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(job_id.as_slice())
    .bind(&lease.worker_id)
    .bind(signed(lease.runtime_generation)?)
    .bind(signed(lease.grant_epoch)?)
    .bind(signed(lease.lease_fence)?)
    .bind(now_unix_millis)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?
    .ok_or(CallTranscriptionPersistenceErrorV1::StaleFence)?;
    Ok(LockedJobV1 {
        run_id: id16(row.try_get("run_id").map_err(row_error)?)?,
    })
}

#[allow(clippy::too_many_arguments)]
async fn persist_pending_result(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    current_revision: u64,
    next: &makosh_call_transcription_core::CallTranscriptionStatusV1,
    pending: &PendingTranscriptV1,
    result_receipt_sha256: [u8; 32],
    occurred_at_unix_millis: i64,
) -> Result<(), CallTranscriptionPersistenceErrorV1> {
    let changed = sqlx::query(
        "UPDATE makosh_data.call_transcription_runs SET state=$1,state_revision=$2,
         stt_result_receipt_sha256=$3,pending_transcript_reference_id=$4,
         pending_transcript_sha256=$5,pending_transcript_size_bytes=$6,
         pending_detected_language=$7,pending_duration_millis=$8,
         pending_segment_count=$9,pending_completeness=$10,
         pending_confidence_basis_points=$11,updated_at_unix_millis=$12
         WHERE logical_owner_id=$13 AND run_id=$14 AND state_revision=$15",
    )
    .bind(state_code(next.state))
    .bind(signed(next.state_revision)?)
    .bind(result_receipt_sha256.as_slice())
    .bind(pending.transcript_reference_id.as_slice())
    .bind(pending.transcript_sha256.as_slice())
    .bind(signed(pending.transcript_size_bytes)?)
    .bind(language_code(pending.detected_language))
    .bind(signed(pending.duration_millis)?)
    .bind(i32::try_from(pending.segment_count).map_err(invalid_input)?)
    .bind(completeness_code(pending.completeness))
    .bind(i32::try_from(pending.confidence_basis_points).map_err(invalid_input)?)
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(signed(current_revision)?)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?
    .rows_affected();
    exact_update(changed)
}

#[allow(clippy::too_many_arguments)]
async fn finish_job(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    job_id: [u8; 16],
    lease: &CallTranscriptionJobLeaseV1,
    state: i16,
    result_receipt_sha256: Option<[u8; 32]>,
    occurred_at_unix_millis: i64,
) -> Result<(), CallTranscriptionPersistenceErrorV1> {
    let changed = sqlx::query(
        "UPDATE makosh_data.call_transcription_jobs SET state=$1,worker_id=NULL,
         lease_expires_at_unix_millis=NULL,result_receipt_sha256=$2,
         updated_at_unix_millis=$3 WHERE logical_owner_id=$4 AND job_id=$5 AND state=2
           AND worker_id=$6 AND runtime_generation=$7 AND grant_epoch=$8 AND lease_fence=$9",
    )
    .bind(state)
    .bind(result_receipt_sha256.as_ref().map(<[u8; 32]>::as_slice))
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(job_id.as_slice())
    .bind(&lease.worker_id)
    .bind(signed(lease.runtime_generation)?)
    .bind(signed(lease.grant_epoch)?)
    .bind(signed(lease.lease_fence)?)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?
    .rows_affected();
    exact_update(changed)
}

fn validate_stt_result(
    input: &PersistSttResultV1,
) -> Result<(), CallTranscriptionPersistenceErrorV1> {
    if !valid_owner(&input.logical_owner_id)
        || !valid_id16(&input.job_id)
        || !valid_lease(&input.lease, input.occurred_at_unix_millis)
        || input
            .outbox
            .as_ref()
            .is_some_and(|value| !valid_outbox(value))
    {
        return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
    }
    match &input.transition {
        CallTranscriptionTransitionV1::SttCompleted(_) => input
            .result_receipt_sha256
            .filter(valid_sha256)
            .map(|_| ())
            .ok_or(CallTranscriptionPersistenceErrorV1::InvalidInput),
        CallTranscriptionTransitionV1::Reject(
            CallTranscriptionRejectionV1::SttRejected
            | CallTranscriptionRejectionV1::ResultRejected
            | CallTranscriptionRejectionV1::StaleAuthority
            | CallTranscriptionRejectionV1::Policy,
        ) if input.result_receipt_sha256.is_none() => Ok(()),
        _ => Err(CallTranscriptionPersistenceErrorV1::InvalidInput),
    }
}

fn valid_lease(lease: &CallTranscriptionJobLeaseV1, now_unix_millis: i64) -> bool {
    valid_worker(&lease.worker_id)
        && lease.runtime_generation > 0
        && lease.grant_epoch > 0
        && lease.lease_fence > 0
        && valid_timestamp_millis(now_unix_millis)
        && lease.lease_expires_at_unix_millis >= now_unix_millis
}

fn valid_materialization(input: &MaterializeTranscriptV1) -> bool {
    valid_owner(&input.logical_owner_id)
        && valid_id16(&input.job_id)
        && valid_id16(&input.run_id)
        && valid_id16(&input.artifact_id)
        && valid_id16(&input.artifact_reference_id)
        && valid_sha256(&input.artifact_receipt_sha256)
        && input.runtime_generation > 0
        && input.grant_epoch > 0
        && valid_timestamp_millis(input.occurred_at_unix_millis)
        && input.outbox.as_ref().is_none_or(valid_outbox)
}
