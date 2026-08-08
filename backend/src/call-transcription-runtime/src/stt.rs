use std::os::unix::net::UnixStream;

use makosh_call_transcription_api::{MAX_SEGMENTS_V1, MAX_TRANSCRIPT_BYTES_V1};
use makosh_call_transcription_core::{
    CallTranscriptionCompletenessV1, CallTranscriptionLanguageV1, CallTranscriptionRejectionV1,
    CallTranscriptionTransitionV1, PendingTranscriptV1, RecordingSourceV1,
};
use makosh_call_transcription_persistence::{
    CallTranscriptionPersistenceErrorV1, CallTranscriptionPersistenceV1,
    ClaimedCallTranscriptionJobV1, CompleteSourceCleanupV1, MaterializeTranscriptV1,
    PersistSttResultV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{
        ManagedRuntimeControlRequestV1, ManagedRuntimeModuleRequestRequestV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::module_request::{
        MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1, validate_module_request_request_v1,
        validate_module_request_response_v1,
    },
};
use makosh_speech_to_text_api::{
    seal_speech_to_text_request_v1, speech_to_text_contract_reference_v1,
    validate_speech_to_text_request_v1, validate_speech_to_text_result_v1,
    wire::{
        SpeechAudioFormatV1, SpeechAudioSourceReceiptV1, SpeechLanguageV1, SpeechToTextRequestV1,
        SpeechToTextResultV1, SpeechToTextTerminalStatusV1, SpeechTranscriptCompletenessV1,
    },
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    admission::BLOB_CAPABILITY_ID_V1,
    blob::{
        CallTranscriptionBlobErrorV1, RecordingCustodyReceiptV1, accept_transcript_custody_v1,
        fresh_source_cleanup_proof_v1, fresh_stt_source_proof_v1, release_recording_custody_v1,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionSttErrorV1 {
    InvalidRequest,
    InvalidResult,
    Blob(CallTranscriptionBlobErrorV1),
    Persistence(CallTranscriptionPersistenceErrorV1),
    Unavailable,
}

#[must_use]
pub fn stt_request_id_v1(run_id: [u8; 16], audio_sha256: [u8; 32]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.call-transcription.stt-request.v1\0");
    digest.update(run_id);
    digest.update(audio_sha256);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

pub fn build_stt_request_v1(
    logical_owner_id: &str,
    request_id: [u8; 16],
    source: &RecordingSourceV1,
    requested_language: CallTranscriptionLanguageV1,
    custody_source_proof: &[u8],
) -> Result<SpeechToTextRequestV1, CallTranscriptionSttErrorV1> {
    seal_speech_to_text_request_v1(SpeechToTextRequestV1 {
        protocol_major: 0,
        request_id: request_id.to_vec(),
        logical_owner_id: logical_owner_id.to_owned(),
        source: Some(SpeechAudioSourceReceiptV1 {
            reference_id: source.audio_reference_id.to_vec(),
            declared_bytes: source.declared_bytes,
            sha256: source.audio_sha256.to_vec(),
            custody_transfer_source_proof: custody_source_proof.to_vec(),
        }),
        audio_format: SpeechAudioFormatV1::WavPcmS16leMono16000Hz as i32,
        duration_millis: source.duration_millis,
        requested_language: speech_language(requested_language) as i32,
        consent_receipt_id: source.consent_receipt_id.to_vec(),
        consent_policy_revision: source.consent_policy_revision,
        maximum_transcript_bytes: MAX_TRANSCRIPT_BYTES_V1 as u32,
        maximum_segments: MAX_SEGMENTS_V1,
        request_digest: Vec::new(),
    })
    .map_err(|_| CallTranscriptionSttErrorV1::InvalidRequest)
}

pub async fn execute_stt_job_v1(
    persistence: &CallTranscriptionPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    job: &ClaimedCallTranscriptionJobV1,
    occurred_at_unix_millis: i64,
) -> Result<bool, CallTranscriptionSttErrorV1> {
    let source = RecordingCustodyReceiptV1 {
        reference_id: job.recording_source.source.audio_reference_id,
        declared_bytes: job.recording_source.source.declared_bytes,
        receipt_sha256: job.recording_source.source_receipt_sha256,
        custody_transfer_source_proof: Vec::new(),
    };
    let fresh_proof = fresh_stt_source_proof_v1(channel, dispatcher, &source)
        .map_err(CallTranscriptionSttErrorV1::Blob)?;
    let request = build_stt_request_v1(
        &job.logical_owner_id,
        job.stt_request_id,
        &job.recording_source.source,
        job.draft.requested_language,
        &fresh_proof,
    )?;
    if request.request_digest.as_slice() != job.stt_request_digest {
        return Err(CallTranscriptionSttErrorV1::InvalidRequest);
    }
    let result = route_stt(channel, dispatcher, &request)?;
    let result_bytes = result.encode_to_vec();
    let result_receipt_sha256: [u8; 32] = Sha256::digest(&result_bytes).into();
    let terminal = SpeechToTextTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| CallTranscriptionSttErrorV1::InvalidResult)?;
    match terminal {
        SpeechToTextTerminalStatusV1::Ready => {
            let transcript = result
                .transcript
                .as_ref()
                .ok_or(CallTranscriptionSttErrorV1::InvalidResult)?;
            let transcript_sha256 = id32(&transcript.sha256)?;
            let custody = accept_transcript_custody_v1(
                channel,
                dispatcher,
                id16(&transcript.reference_id)?,
                transcript.declared_bytes,
                transcript_sha256,
                &transcript.custody_transfer_source_proof,
                job.stt_request_id,
                result_receipt_sha256,
            )
            .map_err(CallTranscriptionSttErrorV1::Blob)?;
            let pending = PendingTranscriptV1 {
                transcript_reference_id: custody.reference_id,
                transcript_sha256: custody.receipt_sha256,
                transcript_size_bytes: custody.declared_bytes,
                detected_language: core_language(result.detected_language)?,
                duration_millis: job.recording_source.source.duration_millis,
                segment_count: result.segment_count,
                completeness: core_completeness(result.completeness)?,
                confidence_basis_points: result.confidence_basis_points,
                stt_request_digest: job.stt_request_digest,
            };
            persistence
                .persist_stt_result(PersistSttResultV1 {
                    logical_owner_id: job.logical_owner_id.clone(),
                    job_id: job.job_id,
                    lease: job.lease.clone(),
                    transition: CallTranscriptionTransitionV1::SttCompleted(pending),
                    result_receipt_sha256: Some(result_receipt_sha256),
                    outbox: None,
                    occurred_at_unix_millis,
                })
                .await
                .map_err(CallTranscriptionSttErrorV1::Persistence)?;
            persistence
                .materialize_transcript(MaterializeTranscriptV1 {
                    logical_owner_id: job.logical_owner_id.clone(),
                    job_id: job.job_id,
                    run_id: job.run_id,
                    artifact_id: artifact_id_v1(job.run_id, custody.receipt_sha256),
                    artifact_reference_id: custody.reference_id,
                    artifact_receipt_sha256: custody.receipt_sha256,
                    runtime_generation: job.lease.runtime_generation,
                    grant_epoch: job.lease.grant_epoch,
                    outbox: None,
                    occurred_at_unix_millis,
                })
                .await
                .map_err(CallTranscriptionSttErrorV1::Persistence)?;
            cleanup_source(
                persistence,
                channel,
                dispatcher,
                job,
                &source,
                true,
                occurred_at_unix_millis,
            )
            .await?;
            Ok(true)
        }
        SpeechToTextTerminalStatusV1::Rejected => {
            persistence
                .persist_stt_result(PersistSttResultV1 {
                    logical_owner_id: job.logical_owner_id.clone(),
                    job_id: job.job_id,
                    lease: job.lease.clone(),
                    transition: CallTranscriptionTransitionV1::Reject(
                        CallTranscriptionRejectionV1::SttRejected,
                    ),
                    result_receipt_sha256: None,
                    outbox: None,
                    occurred_at_unix_millis,
                })
                .await
                .map_err(CallTranscriptionSttErrorV1::Persistence)?;
            cleanup_source(
                persistence,
                channel,
                dispatcher,
                job,
                &source,
                false,
                occurred_at_unix_millis,
            )
            .await?;
            Ok(false)
        }
        SpeechToTextTerminalStatusV1::Unspecified => {
            Err(CallTranscriptionSttErrorV1::InvalidResult)
        }
    }
}

fn route_stt(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: &SpeechToTextRequestV1,
) -> Result<SpeechToTextResultV1, CallTranscriptionSttErrorV1> {
    validate_speech_to_text_request_v1(request)
        .map_err(|_| CallTranscriptionSttErrorV1::InvalidRequest)?;
    let request_id = id16(&request.request_id)?;
    let routed = ManagedRuntimeModuleRequestRequestV1 {
        request_id: request_id.to_vec(),
        contract: Some(speech_to_text_contract_reference_v1()),
        request_payload: request.encode_to_vec(),
        deadline_millis: MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1,
        response_blob_capability_id: BLOB_CAPABILITY_ID_V1.to_owned(),
    };
    validate_module_request_request_v1(&routed)
        .map_err(|_| CallTranscriptionSttErrorV1::InvalidRequest)?;
    let response = channel
        .request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::RouteModuleRequest(routed)),
            },
            dispatcher,
        )
        .map_err(|_| CallTranscriptionSttErrorV1::Unavailable)?;
    if !response.error_code.is_empty() {
        return Err(CallTranscriptionSttErrorV1::Unavailable);
    }
    let Some(ControlResult::ModuleRequestRoute(response)) = response.result else {
        return Err(CallTranscriptionSttErrorV1::Unavailable);
    };
    validate_module_request_response_v1(&response)
        .map_err(|_| CallTranscriptionSttErrorV1::Unavailable)?;
    if response.request_id.as_slice() != request_id {
        return Err(CallTranscriptionSttErrorV1::Unavailable);
    }
    match response.error_code.as_str() {
        "" => {}
        "REJECTED" => return Err(CallTranscriptionSttErrorV1::InvalidResult),
        _ => return Err(CallTranscriptionSttErrorV1::Unavailable),
    }
    let result = SpeechToTextResultV1::decode(response.response_payload.as_slice())
        .map_err(|_| CallTranscriptionSttErrorV1::InvalidResult)?;
    validate_speech_to_text_result_v1(request, &result)
        .map_err(|_| CallTranscriptionSttErrorV1::InvalidResult)?;
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
async fn cleanup_source(
    persistence: &CallTranscriptionPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    job: &ClaimedCallTranscriptionJobV1,
    source: &RecordingCustodyReceiptV1,
    accepted: bool,
    occurred_at_unix_millis: i64,
) -> Result<(), CallTranscriptionSttErrorV1> {
    let cleanup_proof = fresh_source_cleanup_proof_v1(channel, dispatcher, source)
        .map_err(CallTranscriptionSttErrorV1::Blob)?;
    release_recording_custody_v1(
        channel,
        dispatcher,
        job.run_id,
        source,
        &cleanup_proof,
        accepted,
    )
    .map_err(CallTranscriptionSttErrorV1::Blob)?;
    persistence
        .complete_source_cleanup(
            &job.logical_owner_id,
            CompleteSourceCleanupV1 {
                run_id: job.run_id,
                source_reference_id: source.reference_id,
                source_receipt_sha256: source.receipt_sha256,
                completed_at_unix_millis: occurred_at_unix_millis,
            },
        )
        .await
        .map_err(CallTranscriptionSttErrorV1::Persistence)
}

pub(crate) fn artifact_id_v1(run_id: [u8; 16], transcript_sha256: [u8; 32]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.call-transcription.artifact.v1\0");
    digest.update(run_id);
    digest.update(transcript_sha256);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

fn speech_language(value: CallTranscriptionLanguageV1) -> SpeechLanguageV1 {
    match value {
        CallTranscriptionLanguageV1::Auto => SpeechLanguageV1::Auto,
        CallTranscriptionLanguageV1::English => SpeechLanguageV1::English,
        CallTranscriptionLanguageV1::Russian => SpeechLanguageV1::Russian,
        CallTranscriptionLanguageV1::Spanish => SpeechLanguageV1::Spanish,
    }
}

fn core_language(value: i32) -> Result<CallTranscriptionLanguageV1, CallTranscriptionSttErrorV1> {
    match SpeechLanguageV1::try_from(value)
        .map_err(|_| CallTranscriptionSttErrorV1::InvalidResult)?
    {
        SpeechLanguageV1::Auto => Ok(CallTranscriptionLanguageV1::Auto),
        SpeechLanguageV1::English => Ok(CallTranscriptionLanguageV1::English),
        SpeechLanguageV1::Russian => Ok(CallTranscriptionLanguageV1::Russian),
        SpeechLanguageV1::Spanish => Ok(CallTranscriptionLanguageV1::Spanish),
        SpeechLanguageV1::Unspecified => Err(CallTranscriptionSttErrorV1::InvalidResult),
    }
}

fn core_completeness(
    value: i32,
) -> Result<CallTranscriptionCompletenessV1, CallTranscriptionSttErrorV1> {
    match SpeechTranscriptCompletenessV1::try_from(value)
        .map_err(|_| CallTranscriptionSttErrorV1::InvalidResult)?
    {
        SpeechTranscriptCompletenessV1::Complete => Ok(CallTranscriptionCompletenessV1::Complete),
        SpeechTranscriptCompletenessV1::Partial => Ok(CallTranscriptionCompletenessV1::Partial),
        SpeechTranscriptCompletenessV1::Unspecified => {
            Err(CallTranscriptionSttErrorV1::InvalidResult)
        }
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], CallTranscriptionSttErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(CallTranscriptionSttErrorV1::InvalidResult)
}

fn id32(value: &[u8]) -> Result<[u8; 32], CallTranscriptionSttErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(CallTranscriptionSttErrorV1::InvalidResult)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> RecordingSourceV1 {
        RecordingSourceV1 {
            recording_evidence_id: [1; 16],
            recording_revision: 2,
            call_evidence_id: [3; 16],
            call_evidence_revision: 4,
            consent_receipt_id: [5; 16],
            consent_policy_revision: 6,
            audio_reference_id: [7; 16],
            audio_sha256: [8; 32],
            declared_bytes: 32_044,
            duration_millis: 1_000,
        }
    }

    #[test]
    fn refreshed_proof_does_not_change_durable_request_digest() {
        let first = build_stt_request_v1(
            "owner-1",
            [9; 16],
            &source(),
            CallTranscriptionLanguageV1::Auto,
            &[10; 32],
        )
        .expect("request");
        let second = build_stt_request_v1(
            "owner-1",
            [9; 16],
            &source(),
            CallTranscriptionLanguageV1::Auto,
            &[11; 32],
        )
        .expect("request");
        assert_eq!(first.request_digest, second.request_digest);
        assert_ne!(
            first.source.expect("source").custody_transfer_source_proof,
            second.source.expect("source").custody_transfer_source_proof
        );
    }

    #[test]
    fn request_and_artifact_ids_are_stable_and_purpose_separated() {
        let request = stt_request_id_v1([1; 16], [2; 32]);
        assert_eq!(request, stt_request_id_v1([1; 16], [2; 32]));
        assert_ne!(request, artifact_id_v1([1; 16], [2; 32]));
    }
}
