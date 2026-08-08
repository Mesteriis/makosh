use makosh_speech_to_text_api::{
    validate_speech_to_text_request_v1,
    wire::{
        SpeechLanguageV1, SpeechToTextRejectCodeV1, SpeechToTextRequestV1, SpeechToTextResultV1,
        SpeechToTextTerminalStatusV1,
    },
};
use makosh_whisper_stt_core::{
    WHISPER_STT_POLICY_REVISION_V1, WhisperSttExecutionPlanV1, plan_whisper_stt_execution_v1,
    reject_whisper_stt_result_v1,
};
use makosh_whisper_stt_persistence::{
    PersistedWhisperSttRunV1, WhisperSttPersistenceErrorV1, WhisperSttPersistenceV1,
    WhisperSttReadyMetadataV1, WhisperSttRunIdentityV1, WhisperSttRunStateV1,
    WhisperSttTransitionV1,
};
use prost::Message;

use crate::settings::WhisperSttRuntimeSettingsV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhisperSttPortErrorV1 {
    UnsupportedAudio,
    ProviderRejected,
    Unavailable,
    Uncertain,
}

pub trait WhisperSttExecutionPortV1 {
    fn transcribe(
        &mut self,
        plan: &WhisperSttExecutionPlanV1,
    ) -> Result<SpeechToTextResultV1, WhisperSttPortErrorV1>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhisperSttWorkerErrorV1 {
    InvalidRequest,
    Conflict,
    Unavailable,
    Uncertain,
}

pub async fn execute_whisper_stt_payload_v1(
    persistence: &WhisperSttPersistenceV1,
    port: &mut dyn WhisperSttExecutionPortV1,
    logical_human_owner_id: &str,
    settings: &WhisperSttRuntimeSettingsV1,
    model_revision_sha256: [u8; 32],
    payload: &[u8],
) -> Result<Vec<u8>, WhisperSttWorkerErrorV1> {
    let request = SpeechToTextRequestV1::decode(payload)
        .map_err(|_| WhisperSttWorkerErrorV1::InvalidRequest)?;
    validate_speech_to_text_request_v1(&request)
        .map_err(|_| WhisperSttWorkerErrorV1::InvalidRequest)?;
    validate_policy(&request, logical_human_owner_id, settings)?;
    let plan = plan_whisper_stt_execution_v1(
        request.clone(),
        model_revision_sha256,
        settings.settings_revision,
        settings.thread_count,
        settings.timeout_millis,
    )
    .map_err(|_| WhisperSttWorkerErrorV1::InvalidRequest)?;
    let accepted = accepted_run(&plan)?;
    let outcome = persistence
        .accept_run(accepted)
        .await
        .map_err(persistence_error)?;
    let mut persisted = outcome.persisted;
    match persisted.state {
        WhisperSttRunStateV1::Ready => {
            let refreshed = port.transcribe(&plan).map_err(port_error)?;
            require_ready_match(&persisted, &refreshed)?;
            return Ok(refreshed.encode_to_vec());
        }
        WhisperSttRunStateV1::Rejected => {
            return persisted_rejection(&request, &persisted).map(|value| value.encode_to_vec());
        }
        WhisperSttRunStateV1::Uncertain => return Err(WhisperSttWorkerErrorV1::Uncertain),
        WhisperSttRunStateV1::Executing => {
            persist_uncertain(persistence, persisted).await?;
            return Err(WhisperSttWorkerErrorV1::Uncertain);
        }
        WhisperSttRunStateV1::Accepted => {}
    }
    let mut executing = persisted.clone();
    executing.revision += 1;
    executing.state = WhisperSttRunStateV1::Executing;
    persisted = persistence
        .persist_transition(WhisperSttTransitionV1 {
            current_revision: persisted.revision,
            next: executing,
        })
        .await
        .map_err(persistence_error)?;
    let result = match port.transcribe(&plan) {
        Ok(result) => result,
        Err(WhisperSttPortErrorV1::UnsupportedAudio) => {
            return persist_rejection(
                persistence,
                persisted,
                &request,
                SpeechToTextRejectCodeV1::UnsupportedAudio,
            )
            .await;
        }
        Err(WhisperSttPortErrorV1::ProviderRejected) => {
            return persist_rejection(
                persistence,
                persisted,
                &request,
                SpeechToTextRejectCodeV1::ProviderRejected,
            )
            .await;
        }
        Err(WhisperSttPortErrorV1::Unavailable | WhisperSttPortErrorV1::Uncertain) => {
            persist_uncertain(persistence, persisted).await?;
            return Err(WhisperSttWorkerErrorV1::Uncertain);
        }
    };
    let ready = ready_metadata(&result)?;
    let mut terminal = persisted.clone();
    terminal.revision += 1;
    terminal.state = WhisperSttRunStateV1::Ready;
    terminal.ready = Some(ready);
    persistence
        .persist_transition(WhisperSttTransitionV1 {
            current_revision: persisted.revision,
            next: terminal,
        })
        .await
        .map_err(persistence_error)?;
    Ok(result.encode_to_vec())
}

fn validate_policy(
    request: &SpeechToTextRequestV1,
    logical_human_owner_id: &str,
    settings: &WhisperSttRuntimeSettingsV1,
) -> Result<(), WhisperSttWorkerErrorV1> {
    let source = request
        .source
        .as_ref()
        .ok_or(WhisperSttWorkerErrorV1::InvalidRequest)?;
    let language = SpeechLanguageV1::try_from(request.requested_language)
        .map_err(|_| WhisperSttWorkerErrorV1::InvalidRequest)?;
    let language_allowed = match language {
        SpeechLanguageV1::Auto => settings.allowed_languages_mask != 0,
        SpeechLanguageV1::English | SpeechLanguageV1::Russian | SpeechLanguageV1::Spanish => {
            let bit = 1_u32 << (language as u32 - 1);
            settings.allowed_languages_mask & bit != 0
        }
        SpeechLanguageV1::Unspecified => false,
    };
    if request.logical_owner_id != logical_human_owner_id
        || source.declared_bytes > settings.maximum_source_bytes
        || request.maximum_transcript_bytes > settings.maximum_transcript_bytes
        || !language_allowed
    {
        return Err(WhisperSttWorkerErrorV1::InvalidRequest);
    }
    Ok(())
}

fn accepted_run(
    plan: &WhisperSttExecutionPlanV1,
) -> Result<PersistedWhisperSttRunV1, WhisperSttWorkerErrorV1> {
    let source = plan
        .request
        .source
        .as_ref()
        .ok_or(WhisperSttWorkerErrorV1::InvalidRequest)?;
    Ok(PersistedWhisperSttRunV1 {
        identity: WhisperSttRunIdentityV1 {
            logical_owner_id: plan.request.logical_owner_id.clone(),
            request_id: id16(&plan.request.request_id)?,
            request_digest: id32(&plan.request.request_digest)?,
            source_reference_id: id16(&source.reference_id)?,
            source_declared_bytes: source.declared_bytes,
            source_sha256: id32(&source.sha256)?,
            model_revision_sha256: plan.model_revision_sha256,
            provider_settings_revision: plan.provider_settings_revision,
            provider_policy_revision: WHISPER_STT_POLICY_REVISION_V1,
        },
        revision: 1,
        state: WhisperSttRunStateV1::Accepted,
        ready: None,
        reject_code: None,
    })
}

fn ready_metadata(
    result: &SpeechToTextResultV1,
) -> Result<WhisperSttReadyMetadataV1, WhisperSttWorkerErrorV1> {
    if result.terminal_status != SpeechToTextTerminalStatusV1::Ready as i32 {
        return Err(WhisperSttWorkerErrorV1::Conflict);
    }
    let transcript = result
        .transcript
        .as_ref()
        .ok_or(WhisperSttWorkerErrorV1::Conflict)?;
    Ok(WhisperSttReadyMetadataV1 {
        transcript_reference_id: id16(&transcript.reference_id)?,
        transcript_declared_bytes: transcript.declared_bytes,
        transcript_sha256: id32(&transcript.sha256)?,
        detected_language: u32::try_from(result.detected_language)
            .map_err(|_| WhisperSttWorkerErrorV1::Conflict)?,
        segment_count: result.segment_count,
        completeness: u32::try_from(result.completeness)
            .map_err(|_| WhisperSttWorkerErrorV1::Conflict)?,
        confidence_basis_points: result.confidence_basis_points,
    })
}

fn require_ready_match(
    persisted: &PersistedWhisperSttRunV1,
    result: &SpeechToTextResultV1,
) -> Result<(), WhisperSttWorkerErrorV1> {
    if persisted.ready.as_ref() != Some(&ready_metadata(result)?) {
        return Err(WhisperSttWorkerErrorV1::Conflict);
    }
    Ok(())
}

async fn persist_rejection(
    persistence: &WhisperSttPersistenceV1,
    persisted: PersistedWhisperSttRunV1,
    request: &SpeechToTextRequestV1,
    code: SpeechToTextRejectCodeV1,
) -> Result<Vec<u8>, WhisperSttWorkerErrorV1> {
    let result = reject_whisper_stt_result_v1(request, code)
        .map_err(|_| WhisperSttWorkerErrorV1::Conflict)?;
    let mut rejected = persisted.clone();
    rejected.revision += 1;
    rejected.state = WhisperSttRunStateV1::Rejected;
    rejected.reject_code = Some(code as u32);
    persistence
        .persist_transition(WhisperSttTransitionV1 {
            current_revision: persisted.revision,
            next: rejected,
        })
        .await
        .map_err(persistence_error)?;
    Ok(result.encode_to_vec())
}

async fn persist_uncertain(
    persistence: &WhisperSttPersistenceV1,
    persisted: PersistedWhisperSttRunV1,
) -> Result<(), WhisperSttWorkerErrorV1> {
    let mut uncertain = persisted.clone();
    uncertain.revision += 1;
    uncertain.state = WhisperSttRunStateV1::Uncertain;
    persistence
        .persist_transition(WhisperSttTransitionV1 {
            current_revision: persisted.revision,
            next: uncertain,
        })
        .await
        .map(|_| ())
        .map_err(persistence_error)
}

fn persisted_rejection(
    request: &SpeechToTextRequestV1,
    persisted: &PersistedWhisperSttRunV1,
) -> Result<SpeechToTextResultV1, WhisperSttWorkerErrorV1> {
    let code = persisted
        .reject_code
        .and_then(|value| SpeechToTextRejectCodeV1::try_from(value as i32).ok())
        .ok_or(WhisperSttWorkerErrorV1::Conflict)?;
    reject_whisper_stt_result_v1(request, code).map_err(|_| WhisperSttWorkerErrorV1::Conflict)
}

fn id16(value: &[u8]) -> Result<[u8; 16], WhisperSttWorkerErrorV1> {
    value
        .try_into()
        .map_err(|_| WhisperSttWorkerErrorV1::InvalidRequest)
}

fn id32(value: &[u8]) -> Result<[u8; 32], WhisperSttWorkerErrorV1> {
    value
        .try_into()
        .map_err(|_| WhisperSttWorkerErrorV1::InvalidRequest)
}

fn persistence_error(error: WhisperSttPersistenceErrorV1) -> WhisperSttWorkerErrorV1 {
    match error {
        WhisperSttPersistenceErrorV1::RequestConflict
        | WhisperSttPersistenceErrorV1::RevisionConflict
        | WhisperSttPersistenceErrorV1::InvalidTransition => WhisperSttWorkerErrorV1::Conflict,
        WhisperSttPersistenceErrorV1::InvalidInput | WhisperSttPersistenceErrorV1::InvalidRow => {
            WhisperSttWorkerErrorV1::InvalidRequest
        }
        WhisperSttPersistenceErrorV1::StorageUnavailable => WhisperSttWorkerErrorV1::Unavailable,
    }
}

fn port_error(error: WhisperSttPortErrorV1) -> WhisperSttWorkerErrorV1 {
    match error {
        WhisperSttPortErrorV1::UnsupportedAudio | WhisperSttPortErrorV1::ProviderRejected => {
            WhisperSttWorkerErrorV1::Conflict
        }
        WhisperSttPortErrorV1::Unavailable => WhisperSttWorkerErrorV1::Unavailable,
        WhisperSttPortErrorV1::Uncertain => WhisperSttWorkerErrorV1::Uncertain,
    }
}
