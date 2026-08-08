use makosh_speech_to_text_api::{
    validate_speech_to_text_request_v1 as validate_wire_request,
    validate_speech_to_text_result_v1 as validate_wire_result,
    wire::{
        SpeechAudioFormatV1 as WireAudioFormatV1, SpeechLanguageV1 as WireLanguageV1,
        SpeechToTextRejectCodeV1 as WireRejectCodeV1, SpeechToTextRequestV1 as WireRequestV1,
        SpeechToTextResultV1 as WireResultV1, SpeechToTextTerminalStatusV1 as WireTerminalStatusV1,
        SpeechTranscriptCompletenessV1 as WireCompletenessV1,
    },
};
use makosh_speech_to_text_core::{
    SpeechAudioFormatV1, SpeechBlobReceiptV1, SpeechLanguageV1, SpeechToTextExecutionReceiptV1,
    SpeechToTextRejectionV1, SpeechToTextRequestV1, SpeechToTextResultV1, SpeechToTextRunStateV1,
    SpeechToTextRunV1, SpeechToTextTerminalV1, SpeechTranscriptArtifactV1,
    SpeechTranscriptCompletenessV1, accept_speech_to_text_v1, begin_speech_to_text_v1,
    complete_speech_to_text_v1, reject_speech_to_text_v1,
};
use makosh_speech_to_text_persistence::{
    PersistedSpeechToTextRunV1, SpeechToTextPersistenceErrorV1, SpeechToTextPersistenceV1,
    SpeechToTextTransitionV1,
};
use prost::Message;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpeechToTextResponseBlobTargetV1 {
    pub owner_id: String,
    pub module_id: String,
    pub capability_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechToTextPortErrorV1 {
    Rejected,
    Unavailable,
}

pub trait SpeechToTextExecutionPortsV1 {
    fn transcribe(
        &mut self,
        request: WireRequestV1,
        response_target: &SpeechToTextResponseBlobTargetV1,
    ) -> Result<WireResultV1, SpeechToTextPortErrorV1>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechToTextWorkerErrorV1 {
    InvalidRequest,
    Conflict,
    Unavailable,
}

pub async fn execute_speech_to_text_payload_v1(
    persistence: &SpeechToTextPersistenceV1,
    ports: &mut dyn SpeechToTextExecutionPortsV1,
    logical_human_owner_id: &str,
    payload: &[u8],
    response_target: SpeechToTextResponseBlobTargetV1,
) -> Result<Vec<u8>, SpeechToTextWorkerErrorV1> {
    if !valid_response_target(&response_target) {
        return Err(SpeechToTextWorkerErrorV1::InvalidRequest);
    }
    let wire_request =
        WireRequestV1::decode(payload).map_err(|_| SpeechToTextWorkerErrorV1::InvalidRequest)?;
    validate_wire_request(&wire_request).map_err(|_| SpeechToTextWorkerErrorV1::InvalidRequest)?;
    if wire_request.logical_owner_id != logical_human_owner_id {
        return Err(SpeechToTextWorkerErrorV1::InvalidRequest);
    }
    let request = core_request(&wire_request)?;
    let accepted =
        accept_speech_to_text_v1(request).map_err(|_| SpeechToTextWorkerErrorV1::InvalidRequest)?;
    let outcome = persistence
        .accept_run(accepted.clone())
        .await
        .map_err(persistence_error)?;

    match outcome.persisted.state {
        SpeechToTextRunStateV1::Rejected => {
            return persisted_rejection_result(&outcome.persisted)
                .map(|value| value.encode_to_vec());
        }
        SpeechToTextRunStateV1::Ready => {
            let refreshed = execute_provider(ports, wire_request, &response_target)?;
            require_persisted_ready_match(&outcome.persisted, &refreshed)?;
            return Ok(refreshed.encode_to_vec());
        }
        SpeechToTextRunStateV1::Accepted | SpeechToTextRunStateV1::Executing => {}
    }

    let executing = if outcome.persisted.state == SpeechToTextRunStateV1::Accepted {
        let next = begin_speech_to_text_v1(&accepted, outcome.persisted.revision)
            .map_err(|_| SpeechToTextWorkerErrorV1::Conflict)?;
        persistence
            .persist_transition(SpeechToTextTransitionV1 {
                current_revision: outcome.persisted.revision,
                next_run: next.clone(),
            })
            .await
            .map_err(persistence_error)?;
        next
    } else {
        SpeechToTextRunV1 {
            request: accepted.request.clone(),
            state: SpeechToTextRunStateV1::Executing,
            revision: outcome.persisted.revision,
            terminal_result: None,
        }
    };

    let wire_result = match ports.transcribe(wire_request.clone(), &response_target) {
        Ok(result) => {
            validate_wire_result(&wire_request, &result)
                .map_err(|_| SpeechToTextWorkerErrorV1::Conflict)?;
            result
        }
        Err(SpeechToTextPortErrorV1::Unavailable) => {
            return Err(SpeechToTextWorkerErrorV1::Unavailable);
        }
        Err(SpeechToTextPortErrorV1::Rejected) => {
            return persist_rejection(
                persistence,
                executing,
                SpeechToTextRejectionV1::ProviderRejected,
            )
            .await;
        }
    };

    let next = match wire_terminal(&wire_result)? {
        SpeechToTextTerminalV1::Ready(artifact) => complete_speech_to_text_v1(
            &executing,
            executing.revision,
            SpeechToTextResultV1 {
                request_id: executing.request.request_id,
                request_digest: executing.request.request_digest,
                source_sha256: executing.request.source.sha256,
                terminal: SpeechToTextTerminalV1::Ready(artifact),
            },
        )
        .map_err(|_| SpeechToTextWorkerErrorV1::Conflict)?,
        SpeechToTextTerminalV1::Rejected(rejection) => {
            reject_speech_to_text_v1(&executing, executing.revision, rejection)
                .map_err(|_| SpeechToTextWorkerErrorV1::Conflict)?
        }
    };
    persistence
        .persist_transition(SpeechToTextTransitionV1 {
            current_revision: executing.revision,
            next_run: next,
        })
        .await
        .map_err(persistence_error)?;
    Ok(wire_result.encode_to_vec())
}

fn execute_provider(
    ports: &mut dyn SpeechToTextExecutionPortsV1,
    request: WireRequestV1,
    target: &SpeechToTextResponseBlobTargetV1,
) -> Result<WireResultV1, SpeechToTextWorkerErrorV1> {
    let result = ports
        .transcribe(request.clone(), target)
        .map_err(|error| match error {
            SpeechToTextPortErrorV1::Rejected => SpeechToTextWorkerErrorV1::Conflict,
            SpeechToTextPortErrorV1::Unavailable => SpeechToTextWorkerErrorV1::Unavailable,
        })?;
    validate_wire_result(&request, &result).map_err(|_| SpeechToTextWorkerErrorV1::Conflict)?;
    Ok(result)
}

async fn persist_rejection(
    persistence: &SpeechToTextPersistenceV1,
    executing: SpeechToTextRunV1,
    rejection: SpeechToTextRejectionV1,
) -> Result<Vec<u8>, SpeechToTextWorkerErrorV1> {
    let rejected = reject_speech_to_text_v1(&executing, executing.revision, rejection)
        .map_err(|_| SpeechToTextWorkerErrorV1::Conflict)?;
    let persisted = persistence
        .persist_transition(SpeechToTextTransitionV1 {
            current_revision: executing.revision,
            next_run: rejected,
        })
        .await
        .map_err(persistence_error)?;
    persisted_rejection_result(&persisted).map(|value| value.encode_to_vec())
}

fn persisted_rejection_result(
    persisted: &PersistedSpeechToTextRunV1,
) -> Result<WireResultV1, SpeechToTextWorkerErrorV1> {
    let rejection = persisted
        .rejection
        .ok_or(SpeechToTextWorkerErrorV1::Conflict)?;
    Ok(WireResultV1 {
        request_id: persisted.request.request_id.to_vec(),
        request_digest: persisted.request.request_digest.to_vec(),
        source_sha256: persisted.request.source_sha256.to_vec(),
        terminal_status: WireTerminalStatusV1::Rejected as i32,
        transcript: None,
        detected_language: WireLanguageV1::Unspecified as i32,
        segment_count: 0,
        completeness: WireCompletenessV1::Unspecified as i32,
        confidence_basis_points: 0,
        execution_receipt: None,
        reject_code: wire_rejection(rejection) as i32,
    })
}

fn require_persisted_ready_match(
    persisted: &PersistedSpeechToTextRunV1,
    result: &WireResultV1,
) -> Result<(), SpeechToTextWorkerErrorV1> {
    let artifact = persisted
        .artifact
        .as_ref()
        .ok_or(SpeechToTextWorkerErrorV1::Conflict)?;
    let transcript = result
        .transcript
        .as_ref()
        .ok_or(SpeechToTextWorkerErrorV1::Conflict)?;
    let receipt = result
        .execution_receipt
        .as_ref()
        .ok_or(SpeechToTextWorkerErrorV1::Conflict)?;
    if result.terminal_status != WireTerminalStatusV1::Ready as i32
        || transcript.reference_id.as_slice() != artifact.reference_id
        || transcript.declared_bytes != artifact.declared_bytes
        || transcript.sha256.as_slice() != artifact.sha256
        || result.detected_language != wire_language(artifact.detected_language) as i32
        || result.segment_count != artifact.segment_count
        || result.completeness != wire_completeness(artifact.completeness) as i32
        || result.confidence_basis_points != artifact.confidence_basis_points
        || receipt.provider_contract_schema_sha256.as_slice()
            != artifact.provider_contract_schema_sha256
        || receipt.model_revision_sha256.as_slice() != artifact.model_revision_sha256
        || receipt.provider_settings_revision != artifact.provider_settings_revision
        || receipt.provider_policy_revision != artifact.provider_policy_revision
    {
        return Err(SpeechToTextWorkerErrorV1::Conflict);
    }
    Ok(())
}

fn core_request(
    request: &WireRequestV1,
) -> Result<SpeechToTextRequestV1, SpeechToTextWorkerErrorV1> {
    let source = request
        .source
        .as_ref()
        .ok_or(SpeechToTextWorkerErrorV1::InvalidRequest)?;
    Ok(SpeechToTextRequestV1 {
        request_id: id16(&request.request_id)?,
        logical_owner_id: request.logical_owner_id.clone(),
        source: SpeechBlobReceiptV1 {
            reference_id: id16(&source.reference_id)?,
            declared_bytes: source.declared_bytes,
            sha256: id32(&source.sha256)?,
            custody_proof: source.custody_transfer_source_proof.clone(),
        },
        audio_format: match WireAudioFormatV1::try_from(request.audio_format) {
            Ok(WireAudioFormatV1::WavPcmS16leMono16000Hz) => {
                SpeechAudioFormatV1::WavPcmS16LeMono16Khz
            }
            _ => return Err(SpeechToTextWorkerErrorV1::InvalidRequest),
        },
        duration_millis: request.duration_millis,
        requested_language: core_language(request.requested_language)?,
        consent_receipt_id: id16(&request.consent_receipt_id)?,
        consent_policy_revision: request.consent_policy_revision,
        maximum_transcript_bytes: request.maximum_transcript_bytes,
        maximum_segments: request.maximum_segments,
        request_digest: id32(&request.request_digest)?,
    })
}

fn wire_terminal(
    result: &WireResultV1,
) -> Result<SpeechToTextTerminalV1, SpeechToTextWorkerErrorV1> {
    match WireTerminalStatusV1::try_from(result.terminal_status) {
        Ok(WireTerminalStatusV1::Ready) => {
            let transcript = result
                .transcript
                .as_ref()
                .ok_or(SpeechToTextWorkerErrorV1::Conflict)?;
            let receipt = result
                .execution_receipt
                .as_ref()
                .ok_or(SpeechToTextWorkerErrorV1::Conflict)?;
            Ok(SpeechToTextTerminalV1::Ready(SpeechTranscriptArtifactV1 {
                receipt: SpeechBlobReceiptV1 {
                    reference_id: id16(&transcript.reference_id)?,
                    declared_bytes: transcript.declared_bytes,
                    sha256: id32(&transcript.sha256)?,
                    custody_proof: transcript.custody_transfer_source_proof.clone(),
                },
                detected_language: core_language(result.detected_language)?,
                segment_count: result.segment_count,
                completeness: match WireCompletenessV1::try_from(result.completeness) {
                    Ok(WireCompletenessV1::Complete) => SpeechTranscriptCompletenessV1::Complete,
                    Ok(WireCompletenessV1::Partial) => SpeechTranscriptCompletenessV1::Partial,
                    _ => return Err(SpeechToTextWorkerErrorV1::Conflict),
                },
                confidence_basis_points: result.confidence_basis_points,
                execution_receipt: SpeechToTextExecutionReceiptV1 {
                    provider_contract_schema_sha256: id32(
                        &receipt.provider_contract_schema_sha256,
                    )?,
                    model_revision_sha256: id32(&receipt.model_revision_sha256)?,
                    provider_settings_revision: receipt.provider_settings_revision,
                    provider_policy_revision: receipt.provider_policy_revision,
                },
            }))
        }
        Ok(WireTerminalStatusV1::Rejected) => Ok(SpeechToTextTerminalV1::Rejected(core_rejection(
            result.reject_code,
        )?)),
        _ => Err(SpeechToTextWorkerErrorV1::Conflict),
    }
}

fn core_language(value: i32) -> Result<SpeechLanguageV1, SpeechToTextWorkerErrorV1> {
    match WireLanguageV1::try_from(value) {
        Ok(WireLanguageV1::Auto) => Ok(SpeechLanguageV1::Auto),
        Ok(WireLanguageV1::English) => Ok(SpeechLanguageV1::English),
        Ok(WireLanguageV1::Russian) => Ok(SpeechLanguageV1::Russian),
        Ok(WireLanguageV1::Spanish) => Ok(SpeechLanguageV1::Spanish),
        _ => Err(SpeechToTextWorkerErrorV1::InvalidRequest),
    }
}

fn core_rejection(value: i32) -> Result<SpeechToTextRejectionV1, SpeechToTextWorkerErrorV1> {
    match WireRejectCodeV1::try_from(value) {
        Ok(WireRejectCodeV1::InvalidRequest) => Ok(SpeechToTextRejectionV1::InvalidRequest),
        Ok(WireRejectCodeV1::ConsentRejected) => Ok(SpeechToTextRejectionV1::ConsentRejected),
        Ok(WireRejectCodeV1::UnsupportedAudio) => Ok(SpeechToTextRejectionV1::UnsupportedAudio),
        Ok(WireRejectCodeV1::ProviderUnavailable) => {
            Ok(SpeechToTextRejectionV1::ProviderUnavailable)
        }
        Ok(WireRejectCodeV1::ProviderRejected) => Ok(SpeechToTextRejectionV1::ProviderRejected),
        Ok(WireRejectCodeV1::Policy) => Ok(SpeechToTextRejectionV1::Policy),
        _ => Err(SpeechToTextWorkerErrorV1::Conflict),
    }
}

fn wire_rejection(value: SpeechToTextRejectionV1) -> WireRejectCodeV1 {
    match value {
        SpeechToTextRejectionV1::InvalidRequest => WireRejectCodeV1::InvalidRequest,
        SpeechToTextRejectionV1::ConsentRejected => WireRejectCodeV1::ConsentRejected,
        SpeechToTextRejectionV1::UnsupportedAudio => WireRejectCodeV1::UnsupportedAudio,
        SpeechToTextRejectionV1::ProviderUnavailable => WireRejectCodeV1::ProviderUnavailable,
        SpeechToTextRejectionV1::ProviderRejected => WireRejectCodeV1::ProviderRejected,
        SpeechToTextRejectionV1::Policy => WireRejectCodeV1::Policy,
    }
}

fn wire_language(value: SpeechLanguageV1) -> WireLanguageV1 {
    match value {
        SpeechLanguageV1::Auto => WireLanguageV1::Auto,
        SpeechLanguageV1::English => WireLanguageV1::English,
        SpeechLanguageV1::Russian => WireLanguageV1::Russian,
        SpeechLanguageV1::Spanish => WireLanguageV1::Spanish,
    }
}

fn wire_completeness(value: SpeechTranscriptCompletenessV1) -> WireCompletenessV1 {
    match value {
        SpeechTranscriptCompletenessV1::Complete => WireCompletenessV1::Complete,
        SpeechTranscriptCompletenessV1::Partial => WireCompletenessV1::Partial,
    }
}

fn valid_response_target(value: &SpeechToTextResponseBlobTargetV1) -> bool {
    [&value.owner_id, &value.module_id, &value.capability_id]
        .into_iter()
        .all(|part| {
            !part.is_empty()
                && part.len() <= 128
                && part.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'_' | b'-' | b'.')
                })
        })
}

fn id16(value: &[u8]) -> Result<[u8; 16], SpeechToTextWorkerErrorV1> {
    value
        .try_into()
        .map_err(|_| SpeechToTextWorkerErrorV1::InvalidRequest)
}

fn id32(value: &[u8]) -> Result<[u8; 32], SpeechToTextWorkerErrorV1> {
    value
        .try_into()
        .map_err(|_| SpeechToTextWorkerErrorV1::InvalidRequest)
}

fn persistence_error(error: SpeechToTextPersistenceErrorV1) -> SpeechToTextWorkerErrorV1 {
    match error {
        SpeechToTextPersistenceErrorV1::StorageUnavailable => {
            SpeechToTextWorkerErrorV1::Unavailable
        }
        SpeechToTextPersistenceErrorV1::RequestConflict
        | SpeechToTextPersistenceErrorV1::RevisionConflict => SpeechToTextWorkerErrorV1::Conflict,
        SpeechToTextPersistenceErrorV1::InvalidInput
        | SpeechToTextPersistenceErrorV1::InvalidRow
        | SpeechToTextPersistenceErrorV1::InvalidTransition => {
            SpeechToTextWorkerErrorV1::InvalidRequest
        }
    }
}
