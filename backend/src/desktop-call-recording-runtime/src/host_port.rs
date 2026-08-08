use makosh_call_transcription_ingress::{
    CallTranscriptionIngressEnvelopeContextV1, RECORDING_READY_CONTRACT_NAME_V1,
    RECORDING_REJECTED_CONTRACT_NAME_V1, build_recording_ready_outbox_record_v1,
    build_recording_rejected_outbox_record_v1, recording_ready_event_id_v1,
    recording_rejected_event_id_v1,
    wire::{RecordingReadyV1, RecordingRejectedV1},
};
use makosh_desktop_call_recording_api::{
    CANONICAL_AUDIO_FORMAT_V1, CONSENT_PURPOSE_V1,
    host_bridge::{decode_operation_v1, encode_command_lease_v1, encode_observation_accepted_v1},
    wire::{
        BeginDesktopCaptureCommandV1, DesktopCaptureCompletedV1, DesktopCaptureRejectedV1,
        DesktopCaptureStartedV1, DesktopRecordingHostCommandV1, DesktopRecordingHostObservationV1,
        StopDesktopCaptureCommandV1, desktop_recording_host_command_v1::Command,
        desktop_recording_host_observation_v1::Observation,
        desktop_recording_host_operation_v1::Operation,
    },
};
use makosh_desktop_call_recording_core::{RecordingStateV1, validate_canonical_wav_v1};
use makosh_desktop_call_recording_persistence::{
    CaptureStartedWriteV1, ExactOutboxRecordV1, HostCommandCompletionV1, PersistedRecordingRunV1,
    PersistenceErrorV1, RejectRecordingWriteV1, TerminalRecordingMetadataV1,
};
use sha2::{Digest, Sha256};

use crate::{
    client_port::{begin_command_id_v1, realtime_transition, stop_command_id_v1},
    managed_runtime::DesktopRecordingManagedRuntimeV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopRecordingHostPortErrorV1 {
    Protocol,
    Conflict,
    Unavailable,
}

pub async fn handle_host_operation_v1(
    runtime: &mut DesktopRecordingManagedRuntimeV1,
    payload: &[u8],
    now_unix_ms: i64,
) -> Result<Vec<u8>, DesktopRecordingHostPortErrorV1> {
    if now_unix_ms <= 0 {
        return Err(DesktopRecordingHostPortErrorV1::Unavailable);
    }
    let operation = decode_operation_v1(payload)
        .map_err(|_| DesktopRecordingHostPortErrorV1::Protocol)?
        .operation
        .ok_or(DesktopRecordingHostPortErrorV1::Protocol)?;
    match operation {
        Operation::ClaimCommands(claim) => {
            let claim_id = id16(&claim.host_claim_id)?;
            let claim_sha256 = runtime.claim_sha256(claim_id);
            let lease_millis = i64::from(claim.lease_seconds)
                .checked_mul(1_000)
                .ok_or(DesktopRecordingHostPortErrorV1::Protocol)?;
            let leased = runtime
                .persistence()
                .claim_host_commands(&claim_sha256, now_unix_ms, lease_millis, claim.limit)
                .await
                .map_err(persistence_error)?;
            let mut commands = Vec::with_capacity(leased.len());
            for leased in leased {
                let run = runtime
                    .persistence()
                    .get(&leased.logical_owner_id, &leased.recording_evidence_id)
                    .await
                    .map_err(persistence_error)?
                    .ok_or(DesktopRecordingHostPortErrorV1::Conflict)?;
                let command = match leased.command_kind {
                    1 if run.state == RecordingStateV1::AwaitingConsent => {
                        Command::BeginCapture(BeginDesktopCaptureCommandV1 {
                            challenge_id: run.challenge_id.to_vec(),
                            recording_evidence_id: run.recording_evidence_id.to_vec(),
                            device_actor_sha256: run.device_actor_sha256.to_vec(),
                            expires_at_unix_ms: run.challenge_expires_at_unix_ms,
                            maximum_duration_millis: run.maximum_duration_millis,
                            consent_policy_revision: run.consent_policy_revision,
                            consent_purpose: CONSENT_PURPOSE_V1.to_owned(),
                            canonical_audio_format: CANONICAL_AUDIO_FORMAT_V1.to_owned(),
                            call_evidence_id: run.call_evidence_id.to_vec(),
                            call_evidence_revision: run.call_evidence_revision,
                        })
                    }
                    2 if matches!(
                        run.state,
                        RecordingStateV1::AwaitingConsent | RecordingStateV1::Capturing
                    ) =>
                    {
                        Command::StopCapture(StopDesktopCaptureCommandV1 {
                            recording_evidence_id: run.recording_evidence_id.to_vec(),
                        })
                    }
                    _ => {
                        if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                            eprintln!(
                                "developer_desktop_recording_claim_conflict kind={} state={:?}",
                                leased.command_kind, run.state
                            );
                        }
                        return Err(DesktopRecordingHostPortErrorV1::Conflict);
                    }
                };
                commands.push(DesktopRecordingHostCommandV1 {
                    command_id: leased.command_id.to_vec(),
                    command: Some(command),
                });
            }
            Ok(encode_command_lease_v1(commands))
        }
        Operation::Observation(observation) => {
            handle_observation(runtime, observation, now_unix_ms).await
        }
    }
}

async fn handle_observation(
    runtime: &mut DesktopRecordingManagedRuntimeV1,
    observation: DesktopRecordingHostObservationV1,
    now_unix_ms: i64,
) -> Result<Vec<u8>, DesktopRecordingHostPortErrorV1> {
    match observation
        .observation
        .ok_or(DesktopRecordingHostPortErrorV1::Protocol)?
    {
        Observation::CaptureStarted(value) => capture_started(runtime, value, now_unix_ms).await,
        Observation::CaptureCompleted(value) => {
            capture_completed(runtime, value, now_unix_ms).await
        }
        Observation::CaptureRejected(value) => capture_rejected(runtime, value, now_unix_ms).await,
    }
}

async fn capture_started(
    runtime: &mut DesktopRecordingManagedRuntimeV1,
    value: DesktopCaptureStartedV1,
    now_unix_ms: i64,
) -> Result<Vec<u8>, DesktopRecordingHostPortErrorV1> {
    let recording_id = id16(&value.recording_evidence_id)?;
    let challenge_id = id16(&value.challenge_id)?;
    let command_id = id16(&value.command_id)?;
    let claim_sha256 = runtime.claim_sha256(id16(&value.host_claim_id)?);
    let run = load_run(runtime, recording_id).await?;
    if run.state == RecordingStateV1::Capturing
        && run.challenge_id == challenge_id
        && run.started_at_unix_ms == Some(value.started_at_unix_ms)
        && command_id == begin_command_id_v1(&run.logical_owner_id, run.operation_id)
    {
        return Ok(encode_observation_accepted_v1(
            recording_id,
            run.recording_revision,
        ));
    }
    if run.state != RecordingStateV1::AwaitingConsent
        || run.challenge_id != challenge_id
        || run.challenge_expires_at_unix_ms < value.started_at_unix_ms
        || value.started_at_unix_ms > now_unix_ms
        || command_id != begin_command_id_v1(&run.logical_owner_id, run.operation_id)
    {
        return Err(DesktopRecordingHostPortErrorV1::Conflict);
    }
    let consent_receipt_id =
        consent_receipt_id(&run, value.started_at_unix_ms, value.os_permission_revision);
    let revision = run.recording_revision + 1;
    let realtime = realtime_transition(
        recording_id,
        revision,
        RecordingStateV1::Capturing,
        0,
        now_unix_ms,
        "",
    );
    let updated = runtime
        .persistence()
        .mark_capturing(&CaptureStartedWriteV1 {
            logical_owner_id: run.logical_owner_id.clone(),
            recording_evidence_id: recording_id,
            expected_revision: run.recording_revision,
            started_at_unix_ms: value.started_at_unix_ms,
            consent_receipt_id,
            command_id,
            claim_sha256,
            realtime,
        })
        .await
        .map_err(persistence_error)?;
    Ok(encode_observation_accepted_v1(
        recording_id,
        updated.recording_revision,
    ))
}

async fn capture_completed(
    runtime: &mut DesktopRecordingManagedRuntimeV1,
    value: DesktopCaptureCompletedV1,
    now_unix_ms: i64,
) -> Result<Vec<u8>, DesktopRecordingHostPortErrorV1> {
    let recording_id = id16(&value.recording_evidence_id)?;
    let command_id = id16(&value.command_id)?;
    let claim_sha256 = runtime.claim_sha256(id16(&value.host_claim_id)?);
    let run = load_run(runtime, recording_id).await?;
    if run.state == RecordingStateV1::Ready {
        let wav =
            validate_canonical_wav_v1(&value.canonical_wav_bytes, run.maximum_duration_millis)
                .map_err(|_| DesktopRecordingHostPortErrorV1::Protocol)?;
        if run.challenge_id == id16(&value.challenge_id)?
            && run.started_at_unix_ms == Some(value.started_at_unix_ms)
            && run.ended_at_unix_ms == Some(value.ended_at_unix_ms)
            && run.source_declared_bytes == u64::try_from(value.canonical_wav_bytes.len()).ok()
            && run.source_duration_millis == Some(wav.duration_millis)
            && run.source_sha256 == Some(wav.sha256)
            && wav.sha256 == sha256(&value.audio_sha256)?
        {
            return Ok(encode_observation_accepted_v1(
                recording_id,
                run.recording_revision,
            ));
        }
    }
    if run.state != RecordingStateV1::Capturing
        || run.challenge_id != id16(&value.challenge_id)?
        || run.started_at_unix_ms != Some(value.started_at_unix_ms)
        || value.ended_at_unix_ms < value.started_at_unix_ms
        || value.ended_at_unix_ms > now_unix_ms
    {
        return Err(DesktopRecordingHostPortErrorV1::Conflict);
    }
    let elapsed_millis = u64::try_from(value.ended_at_unix_ms - value.started_at_unix_ms)
        .map_err(|_| DesktopRecordingHostPortErrorV1::Protocol)?;
    if elapsed_millis == 0 || elapsed_millis > run.maximum_duration_millis {
        return Err(DesktopRecordingHostPortErrorV1::Protocol);
    }
    let wav = validate_canonical_wav_v1(&value.canonical_wav_bytes, run.maximum_duration_millis)
        .map_err(|_| DesktopRecordingHostPortErrorV1::Protocol)?;
    if wav.sha256 != sha256(&value.audio_sha256)? {
        return Err(DesktopRecordingHostPortErrorV1::Protocol);
    }
    let begin_command = begin_command_id_v1(&run.logical_owner_id, run.operation_id);
    let host_command_completion = if command_id == begin_command {
        None
    } else {
        if command_id != stop_command_id_v1(&run.logical_owner_id, recording_id) {
            return Err(DesktopRecordingHostPortErrorV1::Protocol);
        }
        Some(HostCommandCompletionV1 {
            command_id,
            claim_sha256,
            completed_at_unix_ms: now_unix_ms,
        })
    };
    let materializing_revision = run.recording_revision + 1;
    let materializing_realtime = realtime_transition(
        recording_id,
        materializing_revision,
        RecordingStateV1::Materializing,
        wav.duration_millis,
        now_unix_ms,
        "",
    );
    let materializing = runtime
        .persistence()
        .mark_materializing(
            &run.logical_owner_id,
            &recording_id,
            run.recording_revision,
            host_command_completion.as_ref(),
            &materializing_realtime,
        )
        .await
        .map_err(persistence_error)?;
    let blob =
        match runtime.write_recording_blob(recording_id, value.canonical_wav_bytes, wav.sha256) {
            Ok(blob) => blob,
            Err(_) => {
                return reject_materialization(
                    runtime,
                    &materializing,
                    "blob_unavailable",
                    now_unix_ms,
                )
                .await;
            }
        };
    let consent_receipt_id = materializing
        .consent_receipt_id
        .ok_or(DesktopRecordingHostPortErrorV1::Conflict)?;
    let ready_revision = materializing.recording_revision + 1;
    let event_id = recording_ready_event_id_v1(recording_id, ready_revision);
    let event = RecordingReadyV1 {
        event_id: event_id.to_vec(),
        request_id: materializing.operation_id.to_vec(),
        call_evidence_id: materializing.call_evidence_id.to_vec(),
        call_evidence_revision: materializing.call_evidence_revision,
        recording_evidence_id: recording_id.to_vec(),
        recording_revision: ready_revision,
        consent_receipt_id: consent_receipt_id.to_vec(),
        consent_policy_revision: materializing.consent_policy_revision,
        consent_scope: CONSENT_PURPOSE_V1.to_owned(),
        audio_format: CANONICAL_AUDIO_FORMAT_V1.to_owned(),
        declared_bytes: blob.declared_bytes,
        duration_millis: wav.duration_millis,
        audio_sha256: blob.sha256.to_vec(),
        target_blob_reference_id: blob.reference_id.to_vec(),
        custody_transfer_source_proof: blob.custody_transfer_source_proof,
        logical_owner_id: materializing.logical_owner_id.clone(),
    };
    let record =
        build_recording_ready_outbox_record_v1(event, &event_context(runtime, now_unix_ms))
            .map_err(|_| DesktopRecordingHostPortErrorV1::Protocol)?;
    let outbox = exact_outbox(record, RECORDING_READY_CONTRACT_NAME_V1);
    let realtime = realtime_transition(
        recording_id,
        ready_revision,
        RecordingStateV1::Ready,
        wav.duration_millis,
        now_unix_ms,
        "",
    );
    let ready = runtime
        .persistence()
        .complete_ready(
            &materializing.logical_owner_id,
            &recording_id,
            materializing.recording_revision,
            &TerminalRecordingMetadataV1 {
                ended_at_unix_ms: value.ended_at_unix_ms,
                consent_receipt_id,
                source_reference_id: blob.reference_id,
                source_declared_bytes: blob.declared_bytes,
                source_duration_millis: wav.duration_millis,
                source_sha256: blob.sha256,
            },
            &outbox,
            &realtime,
        )
        .await
        .map_err(persistence_error)?;
    Ok(encode_observation_accepted_v1(
        recording_id,
        ready.recording_revision,
    ))
}

async fn capture_rejected(
    runtime: &mut DesktopRecordingManagedRuntimeV1,
    value: DesktopCaptureRejectedV1,
    now_unix_ms: i64,
) -> Result<Vec<u8>, DesktopRecordingHostPortErrorV1> {
    let recording_id = id16(&value.recording_evidence_id)?;
    let command_id = id16(&value.command_id)?;
    let run = load_run(runtime, recording_id).await?;
    if run.state == RecordingStateV1::Rejected
        && run.challenge_id == id16(&value.challenge_id)?
        && run.public_error_code.as_deref() == Some(public_rejection_code(&value.rejection_code))
    {
        return Ok(encode_observation_accepted_v1(
            recording_id,
            run.recording_revision,
        ));
    }
    if !matches!(
        run.state,
        RecordingStateV1::AwaitingConsent | RecordingStateV1::Capturing
    ) || run.challenge_id != id16(&value.challenge_id)?
    {
        return Err(DesktopRecordingHostPortErrorV1::Conflict);
    }
    let begin_command = begin_command_id_v1(&run.logical_owner_id, run.operation_id);
    let stop_command = stop_command_id_v1(&run.logical_owner_id, recording_id);
    if command_id != begin_command && command_id != stop_command {
        return Err(DesktopRecordingHostPortErrorV1::Protocol);
    }
    let completion = if run.state == RecordingStateV1::AwaitingConsent || command_id == stop_command
    {
        Some(HostCommandCompletionV1 {
            command_id,
            claim_sha256: runtime.claim_sha256(id16(&value.host_claim_id)?),
            completed_at_unix_ms: now_unix_ms,
        })
    } else {
        None
    };
    reject_run(
        runtime,
        &run,
        public_rejection_code(&value.rejection_code),
        completion,
        now_unix_ms,
    )
    .await
}

async fn reject_materialization(
    runtime: &mut DesktopRecordingManagedRuntimeV1,
    run: &PersistedRecordingRunV1,
    code: &str,
    now_unix_ms: i64,
) -> Result<Vec<u8>, DesktopRecordingHostPortErrorV1> {
    reject_run(runtime, run, code, None, now_unix_ms).await
}

async fn reject_run(
    runtime: &mut DesktopRecordingManagedRuntimeV1,
    run: &PersistedRecordingRunV1,
    code: &str,
    completion: Option<HostCommandCompletionV1>,
    now_unix_ms: i64,
) -> Result<Vec<u8>, DesktopRecordingHostPortErrorV1> {
    let revision = run.recording_revision + 1;
    let event_id = recording_rejected_event_id_v1(run.recording_evidence_id, revision);
    let record = build_recording_rejected_outbox_record_v1(
        RecordingRejectedV1 {
            event_id: event_id.to_vec(),
            request_id: run.operation_id.to_vec(),
            call_evidence_id: run.call_evidence_id.to_vec(),
            call_evidence_revision: run.call_evidence_revision,
            recording_evidence_id: run.recording_evidence_id.to_vec(),
            recording_revision: revision,
            rejection_code: code.to_owned(),
            logical_owner_id: run.logical_owner_id.clone(),
        },
        &event_context(runtime, now_unix_ms),
    )
    .map_err(|_| DesktopRecordingHostPortErrorV1::Protocol)?;
    let realtime = realtime_transition(
        run.recording_evidence_id,
        revision,
        RecordingStateV1::Rejected,
        0,
        now_unix_ms,
        code,
    );
    let rejected = runtime
        .persistence()
        .reject(&RejectRecordingWriteV1 {
            logical_owner_id: run.logical_owner_id.clone(),
            recording_evidence_id: run.recording_evidence_id,
            expected_revision: run.recording_revision,
            expected_state: run.state,
            public_error_code: code.to_owned(),
            host_command_completion: completion,
            outbox: exact_outbox(record, RECORDING_REJECTED_CONTRACT_NAME_V1),
            realtime,
        })
        .await
        .map_err(persistence_error)?;
    Ok(encode_observation_accepted_v1(
        run.recording_evidence_id,
        rejected.recording_revision,
    ))
}

async fn load_run(
    runtime: &DesktopRecordingManagedRuntimeV1,
    recording_id: [u8; 16],
) -> Result<PersistedRecordingRunV1, DesktopRecordingHostPortErrorV1> {
    runtime
        .persistence()
        .get(runtime.logical_human_owner_id(), &recording_id)
        .await
        .map_err(persistence_error)?
        .ok_or(DesktopRecordingHostPortErrorV1::Conflict)
}

fn event_context(
    runtime: &DesktopRecordingManagedRuntimeV1,
    now_unix_ms: i64,
) -> CallTranscriptionIngressEnvelopeContextV1 {
    CallTranscriptionIngressEnvelopeContextV1 {
        module_id: makosh_desktop_call_recording_api::MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime.runtime_instance_id().to_owned(),
        runtime_generation: runtime.runtime_generation(),
        recorded_at_unix_seconds: now_unix_ms / 1_000,
        recorded_at_nanos: i32::try_from((now_unix_ms % 1_000) * 1_000_000)
            .expect("millisecond remainder fits nanos"),
    }
}

fn exact_outbox(
    record: makosh_events_protocol::delivery::OutboxRecordV1,
    contract_name: &str,
) -> ExactOutboxRecordV1 {
    ExactOutboxRecordV1 {
        event_id: *record.message_id(),
        contract_name: contract_name.to_owned(),
        exact_envelope_bytes: record.exact_bytes().to_vec(),
        envelope_sha256: *record.envelope_sha256(),
    }
}

fn consent_receipt_id(
    run: &PersistedRecordingRunV1,
    started_at_unix_ms: i64,
    os_permission_revision: u32,
) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.desktop-call-recording.consent-receipt.v1\0");
    hash.update(run.challenge_id);
    hash.update(run.device_actor_sha256);
    hash.update(started_at_unix_ms.to_be_bytes());
    hash.update(os_permission_revision.to_be_bytes());
    hash.update(run.consent_policy_revision.to_be_bytes());
    hash.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

fn public_rejection_code(value: &str) -> &str {
    match value {
        "permission_denied"
        | "consent_cancelled"
        | "capture_unavailable"
        | "capture_interrupted" => value,
        _ => "capture_rejected",
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], DesktopRecordingHostPortErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(DesktopRecordingHostPortErrorV1::Protocol)
}

fn sha256(value: &[u8]) -> Result<[u8; 32], DesktopRecordingHostPortErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(DesktopRecordingHostPortErrorV1::Protocol)
}

fn persistence_error(error: PersistenceErrorV1) -> DesktopRecordingHostPortErrorV1 {
    match error {
        PersistenceErrorV1::InvalidInput => DesktopRecordingHostPortErrorV1::Protocol,
        PersistenceErrorV1::Conflict => DesktopRecordingHostPortErrorV1::Conflict,
        PersistenceErrorV1::StorageUnavailable | PersistenceErrorV1::InvalidRow => {
            DesktopRecordingHostPortErrorV1::Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_host_errors_are_reduced_to_public_codes() {
        assert_eq!(
            public_rejection_code("permission_denied"),
            "permission_denied"
        );
        assert_eq!(
            public_rejection_code("provider_private_detail"),
            "capture_rejected"
        );
    }
}
