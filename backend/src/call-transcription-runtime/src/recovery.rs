use std::os::unix::net::UnixStream;

use makosh_call_transcription_core::CallTranscriptionStateV1;
use makosh_call_transcription_persistence::{
    CALL_TRANSCRIPTION_RECOVERY_LIMIT_V1, CallTranscriptionPersistenceErrorV1,
    CallTranscriptionPersistenceV1, CompleteSourceCleanupV1, MaterializeTranscriptV1,
    PersistedCallTranscriptionRunV1, RebindTranscriptMaterializationV1,
    call_transcription_job_id_v1,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};

use crate::{
    blob::{
        CallTranscriptionBlobErrorV1, RecordingCustodyReceiptV1, TranscriptCustodyReceiptV1,
        fresh_source_cleanup_proof_v1, release_recording_custody_v1, verify_transcript_custody_v1,
    },
    stt::{CallTranscriptionSttErrorV1, artifact_id_v1, execute_stt_job_v1},
};

const JOB_LEASE_MILLIS_V1: u64 = 30_000;
const _: () = assert!(
    JOB_LEASE_MILLIS_V1
        <= makosh_call_transcription_persistence::CALL_TRANSCRIPTION_MAX_LEASE_MILLIS_V1
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionRecoveryErrorV1 {
    InvalidAuthority,
    InvalidDurableState,
    Blob(CallTranscriptionBlobErrorV1),
    Persistence(CallTranscriptionPersistenceErrorV1),
    Stt(CallTranscriptionSttErrorV1),
}

#[allow(clippy::too_many_arguments)]
pub async fn recover_call_transcription_once_v1(
    persistence: &CallTranscriptionPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    logical_owner_id: &str,
    worker_id: &str,
    runtime_generation: u64,
    grant_epoch: u64,
    now_unix_millis: i64,
) -> Result<bool, CallTranscriptionRecoveryErrorV1> {
    if logical_owner_id.is_empty()
        || worker_id.is_empty()
        || runtime_generation == 0
        || grant_epoch == 0
        || now_unix_millis <= 0
    {
        return Err(CallTranscriptionRecoveryErrorV1::InvalidAuthority);
    }
    persistence
        .recover_expired_jobs(logical_owner_id, now_unix_millis)
        .await
        .map_err(CallTranscriptionRecoveryErrorV1::Persistence)?;
    let runs = persistence
        .load_recoverable_runs(logical_owner_id, CALL_TRANSCRIPTION_RECOVERY_LIMIT_V1)
        .await
        .map_err(CallTranscriptionRecoveryErrorV1::Persistence)?;

    if let Some(run) = runs
        .iter()
        .find(|run| run.status.state == CallTranscriptionStateV1::MaterializingTranscript)
    {
        recover_materialization(
            persistence,
            channel,
            dispatcher,
            run,
            runtime_generation,
            grant_epoch,
            now_unix_millis,
        )
        .await?;
        return Ok(true);
    }

    if let Some(run) = runs.iter().find(|run| {
        matches!(
            run.status.state,
            CallTranscriptionStateV1::Ready | CallTranscriptionStateV1::Rejected
        ) && run.source_cleanup_completed_at_unix_millis.is_none()
    }) {
        recover_source_cleanup(persistence, channel, dispatcher, run, now_unix_millis).await?;
        return Ok(true);
    }

    let Some(job) = persistence
        .claim_next_job(
            logical_owner_id,
            worker_id,
            runtime_generation,
            grant_epoch,
            now_unix_millis,
            JOB_LEASE_MILLIS_V1,
        )
        .await
        .map_err(CallTranscriptionRecoveryErrorV1::Persistence)?
    else {
        return Ok(false);
    };
    execute_stt_job_v1(persistence, channel, dispatcher, &job, now_unix_millis)
        .await
        .map_err(CallTranscriptionRecoveryErrorV1::Stt)?;
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
async fn recover_materialization(
    persistence: &CallTranscriptionPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run: &PersistedCallTranscriptionRunV1,
    runtime_generation: u64,
    grant_epoch: u64,
    now_unix_millis: i64,
) -> Result<(), CallTranscriptionRecoveryErrorV1> {
    let pending = run
        .status
        .pending_transcript
        .as_ref()
        .ok_or(CallTranscriptionRecoveryErrorV1::InvalidDurableState)?;
    let stt_request_id = run
        .stt_request_id
        .ok_or(CallTranscriptionRecoveryErrorV1::InvalidDurableState)?;
    let stt_result_receipt_sha256 = run
        .stt_result_receipt_sha256
        .ok_or(CallTranscriptionRecoveryErrorV1::InvalidDurableState)?;
    let transcript = TranscriptCustodyReceiptV1 {
        reference_id: pending.transcript_reference_id,
        declared_bytes: pending.transcript_size_bytes,
        receipt_sha256: pending.transcript_sha256,
    };
    verify_transcript_custody_v1(channel, dispatcher, &transcript)
        .map_err(CallTranscriptionRecoveryErrorV1::Blob)?;
    let job_id = call_transcription_job_id_v1(run.run_id, stt_request_id);
    persistence
        .rebind_transcript_materialization(
            &run.logical_owner_id,
            RebindTranscriptMaterializationV1 {
                run_id: run.run_id,
                job_id,
                transcript_reference_id: transcript.reference_id,
                transcript_receipt_sha256: transcript.receipt_sha256,
                stt_result_receipt_sha256,
                runtime_generation,
                grant_epoch,
                rebound_at_unix_millis: now_unix_millis,
            },
        )
        .await
        .map_err(CallTranscriptionRecoveryErrorV1::Persistence)?;
    persistence
        .materialize_transcript(MaterializeTranscriptV1 {
            logical_owner_id: run.logical_owner_id.clone(),
            job_id,
            run_id: run.run_id,
            artifact_id: artifact_id_v1(run.run_id, transcript.receipt_sha256),
            artifact_reference_id: transcript.reference_id,
            artifact_receipt_sha256: transcript.receipt_sha256,
            runtime_generation,
            grant_epoch,
            outbox: None,
            occurred_at_unix_millis: now_unix_millis,
        })
        .await
        .map_err(CallTranscriptionRecoveryErrorV1::Persistence)?;
    let refreshed = persistence
        .load_run(&run.logical_owner_id, run.run_id)
        .await
        .map_err(CallTranscriptionRecoveryErrorV1::Persistence)?;
    recover_source_cleanup(
        persistence,
        channel,
        dispatcher,
        &refreshed,
        now_unix_millis,
    )
    .await
}

async fn recover_source_cleanup(
    persistence: &CallTranscriptionPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run: &PersistedCallTranscriptionRunV1,
    now_unix_millis: i64,
) -> Result<(), CallTranscriptionRecoveryErrorV1> {
    let persisted = run
        .recording_source
        .as_ref()
        .ok_or(CallTranscriptionRecoveryErrorV1::InvalidDurableState)?;
    let source = RecordingCustodyReceiptV1 {
        reference_id: persisted.source.audio_reference_id,
        declared_bytes: persisted.source.declared_bytes,
        receipt_sha256: persisted.source_receipt_sha256,
        custody_transfer_source_proof: Vec::new(),
    };
    let proof = fresh_source_cleanup_proof_v1(channel, dispatcher, &source)
        .map_err(CallTranscriptionRecoveryErrorV1::Blob)?;
    release_recording_custody_v1(
        channel,
        dispatcher,
        run.run_id,
        &source,
        &proof,
        run.status.state == CallTranscriptionStateV1::Ready,
    )
    .map_err(CallTranscriptionRecoveryErrorV1::Blob)?;
    persistence
        .complete_source_cleanup(
            &run.logical_owner_id,
            CompleteSourceCleanupV1 {
                run_id: run.run_id,
                source_reference_id: source.reference_id,
                source_receipt_sha256: source.receipt_sha256,
                completed_at_unix_millis: now_unix_millis,
            },
        )
        .await
        .map_err(CallTranscriptionRecoveryErrorV1::Persistence)
}
