use std::os::unix::net::UnixStream;

use makosh_ai_contracts::{
    communication_summary_inference_contract_reference_v1, validate_summary_inference_request_v1,
    validate_summary_inference_result_v1,
    wire::{
        AiInferenceCompletenessV1, AiInferenceTerminalStatusV1, AiSummaryLanguageV1,
        AiSummaryLengthV1, CommunicationSummaryInferenceRequestV1,
        CommunicationSummaryInferenceResultV1,
    },
};
use makosh_communication_summary_core::{
    CommunicationSummaryCandidateV1, CommunicationSummaryCompletenessV1,
    CommunicationSummaryLanguageV1, CommunicationSummaryLengthV1,
    CommunicationSummaryRejectionCodeV1, CommunicationSummaryStateV1,
    CommunicationSummaryTransitionV1,
};
use makosh_communication_summary_persistence::{
    CommunicationSummaryPersistenceErrorV1, CommunicationSummaryPersistenceV1,
    PersistedCommunicationSummaryRunV1,
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
pub enum CommunicationSummaryInferenceErrorV1 {
    InvalidRequest,
    InvalidResult,
    Persistence(CommunicationSummaryPersistenceErrorV1),
    Unavailable,
}

pub async fn recover_accepted_communication_summary_once_v1(
    persistence: &CommunicationSummaryPersistenceV1,
    logical_owner_id: &str,
    occurred_at_unix_millis: i64,
) -> Result<bool, CommunicationSummaryInferenceErrorV1> {
    let runs = persistence
        .load_recoverable_runs(logical_owner_id)
        .await
        .map_err(CommunicationSummaryInferenceErrorV1::Persistence)?;
    if let Some(run) = runs
        .iter()
        .find(|run| run.status.state == CommunicationSummaryStateV1::Accepted)
    {
        persistence
            .begin_source_preparation(logical_owner_id, &run.draft.run_id, occurred_at_unix_millis)
            .await
            .map_err(CommunicationSummaryInferenceErrorV1::Persistence)?;
        return Ok(true);
    }
    Ok(false)
}

pub async fn complete_communication_summary_inference_v1(
    persistence: &CommunicationSummaryPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run: &PersistedCommunicationSummaryRunV1,
    request: CommunicationSummaryInferenceRequestV1,
    occurred_at_unix_millis: i64,
) -> Result<(), CommunicationSummaryInferenceErrorV1> {
    validate_request_for_run(run, &request)?;
    let transition = match route_inference(channel, dispatcher, request) {
        Ok(result) => terminal_transition(run, result)?,
        Err(RouteInferenceErrorV1::Rejected) => CommunicationSummaryTransitionV1::Reject(
            CommunicationSummaryRejectionCodeV1::InferenceRejected,
        ),
        Err(RouteInferenceErrorV1::Unavailable) => {
            return Err(CommunicationSummaryInferenceErrorV1::Unavailable);
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
        .map_err(CommunicationSummaryInferenceErrorV1::Persistence)?;
    Ok(())
}

fn validate_request_for_run(
    run: &PersistedCommunicationSummaryRunV1,
    request: &CommunicationSummaryInferenceRequestV1,
) -> Result<(), CommunicationSummaryInferenceErrorV1> {
    validate_summary_inference_request_v1(request)
        .map_err(|_| CommunicationSummaryInferenceErrorV1::InvalidRequest)?;
    let context = request
        .context
        .as_ref()
        .ok_or(CommunicationSummaryInferenceErrorV1::InvalidRequest)?;
    if request.run_id.as_slice() != run.draft.run_id
        || request.logical_owner_id != run.logical_owner_id
        || context.request_digest.as_slice()
            != run
                .status
                .inference_request_digest
                .ok_or(CommunicationSummaryInferenceErrorV1::InvalidRequest)?
        || request
            .source
            .as_ref()
            .map(|source| source.sha256.as_slice())
            != run.status.source_sha256.as_ref().map(<[u8; 32]>::as_slice)
    {
        return Err(CommunicationSummaryInferenceErrorV1::InvalidRequest);
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
    request: CommunicationSummaryInferenceRequestV1,
) -> Result<CommunicationSummaryInferenceResultV1, RouteInferenceErrorV1> {
    let request_id: [u8; 16] = request
        .run_id
        .as_slice()
        .try_into()
        .map_err(|_| RouteInferenceErrorV1::Rejected)?;
    let routed = ManagedRuntimeModuleRequestRequestV1 {
        request_id: request_id.to_vec(),
        contract: Some(communication_summary_inference_contract_reference_v1()),
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
        CommunicationSummaryInferenceResultV1::decode(response.response_payload.as_slice())
            .map_err(|_| RouteInferenceErrorV1::Rejected)?;
    validate_summary_inference_result_v1(&result).map_err(|_| RouteInferenceErrorV1::Rejected)?;
    Ok(result)
}

fn terminal_transition(
    run: &PersistedCommunicationSummaryRunV1,
    result: CommunicationSummaryInferenceResultV1,
) -> Result<CommunicationSummaryTransitionV1, CommunicationSummaryInferenceErrorV1> {
    let request_digest = array32(&result.request_digest)?;
    let source_sha256 = array32(&result.source_sha256)?;
    if result.run_id.as_slice() != run.draft.run_id
        || Some(request_digest) != run.status.inference_request_digest
        || Some(source_sha256) != run.status.source_sha256
    {
        return Err(CommunicationSummaryInferenceErrorV1::InvalidResult);
    }
    let terminal = AiInferenceTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| CommunicationSummaryInferenceErrorV1::InvalidResult)?;
    match terminal {
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady => Ok(
            CommunicationSummaryTransitionV1::Complete(CommunicationSummaryCandidateV1 {
                summary_utf8: result.summary_utf8,
                resolved_length: core_length(result.resolved_length)?,
                resolved_language: core_language(result.resolved_language)?,
                completeness: core_completeness(result.completeness)?,
                confidence_basis_points: result.confidence_basis_points,
                request_digest,
                source_sha256,
            }),
        ),
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedPolicy => Ok(
            CommunicationSummaryTransitionV1::Reject(CommunicationSummaryRejectionCodeV1::Policy),
        ),
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedInput => {
            Ok(CommunicationSummaryTransitionV1::Reject(
                CommunicationSummaryRejectionCodeV1::InvalidRequest,
            ))
        }
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable
        | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected => {
            Ok(CommunicationSummaryTransitionV1::Reject(
                CommunicationSummaryRejectionCodeV1::InferenceRejected,
            ))
        }
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusUnspecified => {
            Err(CommunicationSummaryInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_length(
    value: i32,
) -> Result<CommunicationSummaryLengthV1, CommunicationSummaryInferenceErrorV1> {
    match AiSummaryLengthV1::try_from(value)
        .map_err(|_| CommunicationSummaryInferenceErrorV1::InvalidResult)?
    {
        AiSummaryLengthV1::AiSummaryLengthShort => Ok(CommunicationSummaryLengthV1::Short),
        AiSummaryLengthV1::AiSummaryLengthStandard => Ok(CommunicationSummaryLengthV1::Standard),
        AiSummaryLengthV1::AiSummaryLengthDetailed => Ok(CommunicationSummaryLengthV1::Detailed),
        AiSummaryLengthV1::AiSummaryLengthUnspecified => {
            Err(CommunicationSummaryInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_language(
    value: i32,
) -> Result<CommunicationSummaryLanguageV1, CommunicationSummaryInferenceErrorV1> {
    match AiSummaryLanguageV1::try_from(value)
        .map_err(|_| CommunicationSummaryInferenceErrorV1::InvalidResult)?
    {
        AiSummaryLanguageV1::AiSummaryLanguageAuto => Ok(CommunicationSummaryLanguageV1::Auto),
        AiSummaryLanguageV1::AiSummaryLanguageEnglish => {
            Ok(CommunicationSummaryLanguageV1::English)
        }
        AiSummaryLanguageV1::AiSummaryLanguageRussian => {
            Ok(CommunicationSummaryLanguageV1::Russian)
        }
        AiSummaryLanguageV1::AiSummaryLanguageSpanish => {
            Ok(CommunicationSummaryLanguageV1::Spanish)
        }
        AiSummaryLanguageV1::AiSummaryLanguageUnspecified => {
            Err(CommunicationSummaryInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_completeness(
    value: i32,
) -> Result<CommunicationSummaryCompletenessV1, CommunicationSummaryInferenceErrorV1> {
    match AiInferenceCompletenessV1::try_from(value)
        .map_err(|_| CommunicationSummaryInferenceErrorV1::InvalidResult)?
    {
        AiInferenceCompletenessV1::AiInferenceCompletenessComplete => {
            Ok(CommunicationSummaryCompletenessV1::Complete)
        }
        AiInferenceCompletenessV1::AiInferenceCompletenessPartial => {
            Ok(CommunicationSummaryCompletenessV1::Partial)
        }
        AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified => {
            Err(CommunicationSummaryInferenceErrorV1::InvalidResult)
        }
    }
}

fn array32(value: &[u8]) -> Result<[u8; 32], CommunicationSummaryInferenceErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationSummaryInferenceErrorV1::InvalidResult)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_mapping_has_no_provider_identity_branch() {
        assert_eq!(
            core_length(AiSummaryLengthV1::AiSummaryLengthStandard as i32).expect("length"),
            CommunicationSummaryLengthV1::Standard
        );
        assert_eq!(
            core_language(AiSummaryLanguageV1::AiSummaryLanguageRussian as i32).expect("language"),
            CommunicationSummaryLanguageV1::Russian
        );
    }
}
