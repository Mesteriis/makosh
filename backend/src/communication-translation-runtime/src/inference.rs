use std::os::unix::net::UnixStream;

use makosh_ai_contracts::{
    communication_translation_inference_contract_reference_v1,
    validate_translation_inference_request_v1, validate_translation_inference_result_v1,
    wire::{
        AiDetectedLanguageV1, AiInferenceCompletenessV1, AiInferenceTerminalStatusV1,
        AiTranslationLanguageV1, CommunicationTranslationInferenceRequestV1,
        CommunicationTranslationInferenceResultV1,
    },
};
use makosh_communication_translation_core::{
    CommunicationTranslationCandidateV1, CommunicationTranslationCompletenessV1,
    CommunicationTranslationDetectedLanguageV1, CommunicationTranslationLanguageV1,
    CommunicationTranslationRejectionCodeV1, CommunicationTranslationStateV1,
    CommunicationTranslationTransitionV1,
};
use makosh_communication_translation_persistence::{
    CommunicationTranslationPersistenceErrorV1, CommunicationTranslationPersistenceV1,
    PersistedCommunicationTranslationRunV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{
        ManagedRuntimeControlRequestV1, ManagedRuntimeModuleRequestRequestV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::module_request::{
        validate_module_request_request_v1, validate_module_request_response_v1,
    },
};
use prost::Message;

const INFERENCE_DEADLINE_MILLIS_V1: u32 = 30_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTranslationInferenceErrorV1 {
    InvalidRequest,
    InvalidResult,
    Persistence(CommunicationTranslationPersistenceErrorV1),
    Unavailable,
}

pub async fn recover_accepted_communication_translation_once_v1(
    persistence: &CommunicationTranslationPersistenceV1,
    logical_owner_id: &str,
    occurred_at_unix_millis: i64,
) -> Result<bool, CommunicationTranslationInferenceErrorV1> {
    let runs = persistence
        .load_recoverable_runs(logical_owner_id)
        .await
        .map_err(CommunicationTranslationInferenceErrorV1::Persistence)?;
    if let Some(run) = runs
        .iter()
        .find(|run| run.status.state == CommunicationTranslationStateV1::Accepted)
    {
        persistence
            .begin_source_preparation(logical_owner_id, &run.draft.run_id, occurred_at_unix_millis)
            .await
            .map_err(CommunicationTranslationInferenceErrorV1::Persistence)?;
        return Ok(true);
    }
    Ok(false)
}

pub async fn complete_communication_translation_inference_v1(
    persistence: &CommunicationTranslationPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run: &PersistedCommunicationTranslationRunV1,
    request: CommunicationTranslationInferenceRequestV1,
    occurred_at_unix_millis: i64,
) -> Result<(), CommunicationTranslationInferenceErrorV1> {
    validate_request_for_run(run, &request)?;
    let transition = match route_inference(channel, dispatcher, request) {
        Ok(result) => terminal_transition(run, result)?,
        Err(RouteInferenceErrorV1::Rejected) => CommunicationTranslationTransitionV1::Reject(
            CommunicationTranslationRejectionCodeV1::InferenceRejected,
        ),
        Err(RouteInferenceErrorV1::Unavailable) => {
            return Err(CommunicationTranslationInferenceErrorV1::Unavailable);
        }
    };
    persistence
        .persist_inference_transition(
            &run.logical_owner_id,
            &run.draft.run_id,
            transition,
            occurred_at_unix_millis,
        )
        .await
        .map_err(CommunicationTranslationInferenceErrorV1::Persistence)?;
    Ok(())
}

fn validate_request_for_run(
    run: &PersistedCommunicationTranslationRunV1,
    request: &CommunicationTranslationInferenceRequestV1,
) -> Result<(), CommunicationTranslationInferenceErrorV1> {
    validate_translation_inference_request_v1(request)
        .map_err(|_| CommunicationTranslationInferenceErrorV1::InvalidRequest)?;
    let context = request
        .context
        .as_ref()
        .ok_or(CommunicationTranslationInferenceErrorV1::InvalidRequest)?;
    if request.run_id.as_slice() != run.draft.run_id
        || request.logical_owner_id != run.logical_owner_id
        || context.request_digest.as_slice()
            != run
                .status
                .inference_request_digest
                .ok_or(CommunicationTranslationInferenceErrorV1::InvalidRequest)?
        || request
            .source
            .as_ref()
            .map(|source| source.sha256.as_slice())
            != run.status.source_sha256.as_ref().map(<[u8; 32]>::as_slice)
    {
        return Err(CommunicationTranslationInferenceErrorV1::InvalidRequest);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RouteInferenceErrorV1 {
    Rejected,
    Unavailable,
}

fn route_inference(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: CommunicationTranslationInferenceRequestV1,
) -> Result<CommunicationTranslationInferenceResultV1, RouteInferenceErrorV1> {
    let request_id: [u8; 16] = request
        .run_id
        .as_slice()
        .try_into()
        .map_err(|_| RouteInferenceErrorV1::Rejected)?;
    let routed = ManagedRuntimeModuleRequestRequestV1 {
        request_id: request_id.to_vec(),
        contract: Some(communication_translation_inference_contract_reference_v1()),
        request_payload: request.encode_to_vec(),
        deadline_millis: INFERENCE_DEADLINE_MILLIS_V1,
        response_blob_capability_id: String::new(),
    };
    validate_module_request_request_v1(&routed).map_err(|_| RouteInferenceErrorV1::Rejected)?;
    let response = channel
        .request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::RouteModuleRequest(routed)),
            },
            dispatcher,
        )
        .map_err(|_| RouteInferenceErrorV1::Unavailable)?;
    if !response.error_code.is_empty() {
        return Err(RouteInferenceErrorV1::Unavailable);
    }
    let Some(ControlResult::ModuleRequestRoute(response)) = response.result else {
        return Err(RouteInferenceErrorV1::Unavailable);
    };
    validate_module_request_response_v1(&response)
        .map_err(|_| RouteInferenceErrorV1::Unavailable)?;
    if response.request_id.as_slice() != request_id {
        return Err(RouteInferenceErrorV1::Unavailable);
    }
    if !response.error_code.is_empty() {
        return Err(RouteInferenceErrorV1::Rejected);
    }
    let result =
        CommunicationTranslationInferenceResultV1::decode(response.response_payload.as_slice())
            .map_err(|_| RouteInferenceErrorV1::Rejected)?;
    validate_translation_inference_result_v1(&result)
        .map_err(|_| RouteInferenceErrorV1::Rejected)?;
    Ok(result)
}

fn terminal_transition(
    run: &PersistedCommunicationTranslationRunV1,
    result: CommunicationTranslationInferenceResultV1,
) -> Result<CommunicationTranslationTransitionV1, CommunicationTranslationInferenceErrorV1> {
    let request_digest = array32(&result.request_digest)?;
    let source_sha256 = array32(&result.source_sha256)?;
    if result.run_id.as_slice() != run.draft.run_id
        || Some(request_digest) != run.status.inference_request_digest
        || Some(source_sha256) != run.status.source_sha256
    {
        return Err(CommunicationTranslationInferenceErrorV1::InvalidResult);
    }
    let terminal = AiInferenceTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| CommunicationTranslationInferenceErrorV1::InvalidResult)?;
    match terminal {
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady => {
            let target_language = core_target_language(result.target_language)?;
            if target_language != run.draft.target_language {
                return Err(CommunicationTranslationInferenceErrorV1::InvalidResult);
            }
            Ok(CommunicationTranslationTransitionV1::Complete(
                CommunicationTranslationCandidateV1 {
                    translated_text_utf8: result.translated_text_utf8,
                    detected_source_language: core_detected_language(
                        result.detected_source_language,
                    )?,
                    target_language,
                    completeness: core_completeness(result.completeness)?,
                    confidence_basis_points: result.confidence_basis_points,
                    request_digest,
                    source_sha256,
                },
            ))
        }
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedPolicy => {
            Ok(CommunicationTranslationTransitionV1::Reject(
                CommunicationTranslationRejectionCodeV1::Policy,
            ))
        }
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedInput => {
            Ok(CommunicationTranslationTransitionV1::Reject(
                CommunicationTranslationRejectionCodeV1::InvalidRequest,
            ))
        }
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable
        | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected => {
            Ok(CommunicationTranslationTransitionV1::Reject(
                CommunicationTranslationRejectionCodeV1::InferenceRejected,
            ))
        }
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusUnspecified => {
            Err(CommunicationTranslationInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_target_language(
    value: i32,
) -> Result<CommunicationTranslationLanguageV1, CommunicationTranslationInferenceErrorV1> {
    match AiTranslationLanguageV1::try_from(value)
        .map_err(|_| CommunicationTranslationInferenceErrorV1::InvalidResult)?
    {
        AiTranslationLanguageV1::AiTranslationLanguageEnglish => {
            Ok(CommunicationTranslationLanguageV1::English)
        }
        AiTranslationLanguageV1::AiTranslationLanguageRussian => {
            Ok(CommunicationTranslationLanguageV1::Russian)
        }
        AiTranslationLanguageV1::AiTranslationLanguageSpanish => {
            Ok(CommunicationTranslationLanguageV1::Spanish)
        }
        AiTranslationLanguageV1::AiTranslationLanguageUnspecified => {
            Err(CommunicationTranslationInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_detected_language(
    value: i32,
) -> Result<CommunicationTranslationDetectedLanguageV1, CommunicationTranslationInferenceErrorV1> {
    match AiDetectedLanguageV1::try_from(value)
        .map_err(|_| CommunicationTranslationInferenceErrorV1::InvalidResult)?
    {
        AiDetectedLanguageV1::AiDetectedLanguageUnknown => {
            Ok(CommunicationTranslationDetectedLanguageV1::Unknown)
        }
        AiDetectedLanguageV1::AiDetectedLanguageEnglish => {
            Ok(CommunicationTranslationDetectedLanguageV1::English)
        }
        AiDetectedLanguageV1::AiDetectedLanguageRussian => {
            Ok(CommunicationTranslationDetectedLanguageV1::Russian)
        }
        AiDetectedLanguageV1::AiDetectedLanguageSpanish => {
            Ok(CommunicationTranslationDetectedLanguageV1::Spanish)
        }
        AiDetectedLanguageV1::AiDetectedLanguageUnspecified => {
            Err(CommunicationTranslationInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_completeness(
    value: i32,
) -> Result<CommunicationTranslationCompletenessV1, CommunicationTranslationInferenceErrorV1> {
    match AiInferenceCompletenessV1::try_from(value)
        .map_err(|_| CommunicationTranslationInferenceErrorV1::InvalidResult)?
    {
        AiInferenceCompletenessV1::AiInferenceCompletenessComplete => {
            Ok(CommunicationTranslationCompletenessV1::Complete)
        }
        AiInferenceCompletenessV1::AiInferenceCompletenessPartial => {
            Ok(CommunicationTranslationCompletenessV1::Partial)
        }
        AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified => {
            Err(CommunicationTranslationInferenceErrorV1::InvalidResult)
        }
    }
}

fn array32(value: &[u8]) -> Result<[u8; 32], CommunicationTranslationInferenceErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationTranslationInferenceErrorV1::InvalidResult)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_mapping_has_no_provider_specific_branch() {
        assert_eq!(
            core_target_language(AiTranslationLanguageV1::AiTranslationLanguageRussian as i32)
                .expect("target language"),
            CommunicationTranslationLanguageV1::Russian
        );
        assert_eq!(
            core_detected_language(AiDetectedLanguageV1::AiDetectedLanguageEnglish as i32)
                .expect("detected language"),
            CommunicationTranslationDetectedLanguageV1::English
        );
    }
}
