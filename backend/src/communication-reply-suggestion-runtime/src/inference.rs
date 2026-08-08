use std::os::unix::net::UnixStream;

use makosh_ai_contracts::{
    communication_reply_inference_contract_reference_v1, validate_reply_inference_request_v1,
    validate_reply_inference_result_v1,
    wire::{
        AiInferenceCompletenessV1, AiInferenceTerminalStatusV1, AiReplyLanguageV1, AiReplyToneV1,
        CommunicationReplySuggestionInferenceRequestV1,
        CommunicationReplySuggestionInferenceResultV1,
    },
};
use makosh_communication_reply_suggestion_core::{
    ReplySuggestionCandidateV1, ReplySuggestionCompletenessV1, ReplySuggestionLanguageV1,
    ReplySuggestionRejectionCodeV1, ReplySuggestionStateV1, ReplySuggestionToneV1,
    ReplySuggestionTransitionV1,
};
use makosh_communication_reply_suggestion_persistence::{
    CommunicationReplySuggestionPersistenceV1, PersistedReplySuggestionRunV1,
    ReplySuggestionPersistenceErrorV1,
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
pub enum ReplySuggestionInferenceErrorV1 {
    InvalidRequest,
    InvalidResult,
    Persistence(ReplySuggestionPersistenceErrorV1),
    Unavailable,
}

pub async fn recover_accepted_reply_suggestion_once_v1(
    persistence: &CommunicationReplySuggestionPersistenceV1,
    logical_owner_id: &str,
    occurred_at_unix_millis: i64,
) -> Result<bool, ReplySuggestionInferenceErrorV1> {
    let runs = persistence
        .load_recoverable_runs(logical_owner_id)
        .await
        .map_err(ReplySuggestionInferenceErrorV1::Persistence)?;
    if let Some(run) = runs
        .iter()
        .find(|run| run.status.state == ReplySuggestionStateV1::Accepted)
    {
        persistence
            .begin_source_preparation(logical_owner_id, &run.draft.run_id, occurred_at_unix_millis)
            .await
            .map_err(ReplySuggestionInferenceErrorV1::Persistence)?;
        return Ok(true);
    }
    Ok(false)
}

pub async fn complete_reply_suggestion_inference_v1(
    persistence: &CommunicationReplySuggestionPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run: &PersistedReplySuggestionRunV1,
    request: CommunicationReplySuggestionInferenceRequestV1,
    occurred_at_unix_millis: i64,
) -> Result<(), ReplySuggestionInferenceErrorV1> {
    validate_request_for_run(run, &request)?;
    let transition = match route_inference(channel, dispatcher, request) {
        Ok(result) => terminal_transition(run, result)?,
        Err(RouteInferenceErrorV1::Rejected) => {
            ReplySuggestionTransitionV1::Reject(ReplySuggestionRejectionCodeV1::InferenceRejected)
        }
        Err(RouteInferenceErrorV1::Unavailable) => {
            return Err(ReplySuggestionInferenceErrorV1::Unavailable);
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
        .map_err(ReplySuggestionInferenceErrorV1::Persistence)?;
    Ok(())
}

fn validate_request_for_run(
    run: &PersistedReplySuggestionRunV1,
    request: &CommunicationReplySuggestionInferenceRequestV1,
) -> Result<(), ReplySuggestionInferenceErrorV1> {
    validate_reply_inference_request_v1(request)
        .map_err(|_| ReplySuggestionInferenceErrorV1::InvalidRequest)?;
    let context = request
        .context
        .as_ref()
        .ok_or(ReplySuggestionInferenceErrorV1::InvalidRequest)?;
    if request.run_id.as_slice() != run.draft.run_id
        || request.logical_owner_id != run.logical_owner_id
        || context.request_digest.as_slice()
            != run
                .status
                .inference_request_digest
                .ok_or(ReplySuggestionInferenceErrorV1::InvalidRequest)?
        || request
            .source
            .as_ref()
            .map(|source| source.sha256.as_slice())
            != run.status.source_sha256.as_ref().map(<[u8; 32]>::as_slice)
    {
        return Err(ReplySuggestionInferenceErrorV1::InvalidRequest);
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
    request: CommunicationReplySuggestionInferenceRequestV1,
) -> Result<CommunicationReplySuggestionInferenceResultV1, RouteInferenceErrorV1> {
    let request_id: [u8; 16] = request
        .run_id
        .as_slice()
        .try_into()
        .map_err(|_| RouteInferenceErrorV1::Rejected)?;
    let routed = ManagedRuntimeModuleRequestRequestV1 {
        request_id: request_id.to_vec(),
        contract: Some(communication_reply_inference_contract_reference_v1()),
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
        CommunicationReplySuggestionInferenceResultV1::decode(response.response_payload.as_slice())
            .map_err(|_| RouteInferenceErrorV1::Rejected)?;
    validate_reply_inference_result_v1(&result).map_err(|_| RouteInferenceErrorV1::Rejected)?;
    Ok(result)
}

fn terminal_transition(
    run: &PersistedReplySuggestionRunV1,
    result: CommunicationReplySuggestionInferenceResultV1,
) -> Result<ReplySuggestionTransitionV1, ReplySuggestionInferenceErrorV1> {
    let request_digest = array32(&result.request_digest)?;
    let source_sha256 = array32(&result.source_sha256)?;
    if result.run_id.as_slice() != run.draft.run_id
        || Some(request_digest) != run.status.inference_request_digest
        || Some(source_sha256) != run.status.source_sha256
    {
        return Err(ReplySuggestionInferenceErrorV1::InvalidResult);
    }
    let terminal = AiInferenceTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| ReplySuggestionInferenceErrorV1::InvalidResult)?;
    match terminal {
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady => Ok(
            ReplySuggestionTransitionV1::Complete(ReplySuggestionCandidateV1 {
                subject_utf8: result.subject_utf8,
                body_utf8: result.body_utf8,
                resolved_tone: core_tone(result.resolved_tone)?,
                resolved_language: core_language(result.resolved_language)?,
                completeness: core_completeness(result.completeness)?,
                confidence_basis_points: result.confidence_basis_points,
                request_digest,
                source_sha256,
            }),
        ),
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedPolicy => Ok(
            ReplySuggestionTransitionV1::Reject(ReplySuggestionRejectionCodeV1::Policy),
        ),
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedInput => Ok(
            ReplySuggestionTransitionV1::Reject(ReplySuggestionRejectionCodeV1::InvalidRequest),
        ),
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable
        | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected => Ok(
            ReplySuggestionTransitionV1::Reject(ReplySuggestionRejectionCodeV1::InferenceRejected),
        ),
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusUnspecified => {
            Err(ReplySuggestionInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_tone(value: i32) -> Result<ReplySuggestionToneV1, ReplySuggestionInferenceErrorV1> {
    match AiReplyToneV1::try_from(value)
        .map_err(|_| ReplySuggestionInferenceErrorV1::InvalidResult)?
    {
        AiReplyToneV1::AiReplyToneNeutral => Ok(ReplySuggestionToneV1::Professional),
        AiReplyToneV1::AiReplyToneWarm => Ok(ReplySuggestionToneV1::Friendly),
        AiReplyToneV1::AiReplyToneFormal => Ok(ReplySuggestionToneV1::Formal),
        AiReplyToneV1::AiReplyToneConcise => Ok(ReplySuggestionToneV1::Concise),
        AiReplyToneV1::AiReplyToneUnspecified => {
            Err(ReplySuggestionInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_language(value: i32) -> Result<ReplySuggestionLanguageV1, ReplySuggestionInferenceErrorV1> {
    match AiReplyLanguageV1::try_from(value)
        .map_err(|_| ReplySuggestionInferenceErrorV1::InvalidResult)?
    {
        AiReplyLanguageV1::AiReplyLanguageAuto => Ok(ReplySuggestionLanguageV1::Source),
        AiReplyLanguageV1::AiReplyLanguageEnglish => Ok(ReplySuggestionLanguageV1::English),
        AiReplyLanguageV1::AiReplyLanguageRussian => Ok(ReplySuggestionLanguageV1::Russian),
        AiReplyLanguageV1::AiReplyLanguageSpanish => Ok(ReplySuggestionLanguageV1::Spanish),
        AiReplyLanguageV1::AiReplyLanguageUnspecified => {
            Err(ReplySuggestionInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_completeness(
    value: i32,
) -> Result<ReplySuggestionCompletenessV1, ReplySuggestionInferenceErrorV1> {
    match AiInferenceCompletenessV1::try_from(value)
        .map_err(|_| ReplySuggestionInferenceErrorV1::InvalidResult)?
    {
        AiInferenceCompletenessV1::AiInferenceCompletenessComplete => {
            Ok(ReplySuggestionCompletenessV1::Complete)
        }
        AiInferenceCompletenessV1::AiInferenceCompletenessPartial => {
            Ok(ReplySuggestionCompletenessV1::Partial)
        }
        AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified => {
            Err(ReplySuggestionInferenceErrorV1::InvalidResult)
        }
    }
}

fn array32(value: &[u8]) -> Result<[u8; 32], ReplySuggestionInferenceErrorV1> {
    value
        .try_into()
        .map_err(|_| ReplySuggestionInferenceErrorV1::InvalidResult)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_mapping_has_no_provider_identity_branch() {
        assert_eq!(
            core_tone(AiReplyToneV1::AiReplyToneWarm as i32).expect("tone"),
            ReplySuggestionToneV1::Friendly
        );
        assert_eq!(
            core_language(AiReplyLanguageV1::AiReplyLanguageRussian as i32).expect("language"),
            ReplySuggestionLanguageV1::Russian
        );
    }
}
