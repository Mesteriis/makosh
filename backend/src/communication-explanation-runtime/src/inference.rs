use std::os::unix::net::UnixStream;

use makosh_ai_contracts::{
    communication_explanation_inference_contract_reference_v1,
    validate_explanation_inference_request_v1, validate_explanation_inference_result_v1,
    wire::{
        AiExplanationReasonKindV1, AiExplanationReasonV1, AiExplanationSourceBasisV1,
        AiInferenceCompletenessV1, AiInferenceTerminalStatusV1,
        CommunicationExplanationInferenceRequestV1, CommunicationExplanationInferenceResultV1,
    },
};
use makosh_communication_explanation_core::{
    CommunicationExplanationCandidateV1, CommunicationExplanationCompletenessV1,
    CommunicationExplanationReasonKindV1, CommunicationExplanationReasonV1,
    CommunicationExplanationRejectionCodeV1, CommunicationExplanationSourceBasisV1,
    CommunicationExplanationStateV1, CommunicationExplanationTransitionV1,
};
use makosh_communication_explanation_persistence::{
    CommunicationExplanationPersistenceErrorV1, CommunicationExplanationPersistenceV1,
    PersistedCommunicationExplanationRunV1,
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
pub enum CommunicationExplanationInferenceErrorV1 {
    InvalidRequest,
    InvalidResult,
    Persistence(CommunicationExplanationPersistenceErrorV1),
    Unavailable,
}

pub async fn recover_accepted_communication_explanation_once_v1(
    persistence: &CommunicationExplanationPersistenceV1,
    logical_owner_id: &str,
    occurred_at_unix_millis: i64,
) -> Result<bool, CommunicationExplanationInferenceErrorV1> {
    let runs = persistence
        .load_recoverable_runs(logical_owner_id)
        .await
        .map_err(CommunicationExplanationInferenceErrorV1::Persistence)?;
    if let Some(run) = runs
        .iter()
        .find(|run| run.status.state == CommunicationExplanationStateV1::Accepted)
    {
        persistence
            .begin_source_preparation(logical_owner_id, &run.draft.run_id, occurred_at_unix_millis)
            .await
            .map_err(CommunicationExplanationInferenceErrorV1::Persistence)?;
        return Ok(true);
    }
    Ok(false)
}

pub async fn complete_communication_explanation_inference_v1(
    persistence: &CommunicationExplanationPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run: &PersistedCommunicationExplanationRunV1,
    request: CommunicationExplanationInferenceRequestV1,
    occurred_at_unix_millis: i64,
) -> Result<(), CommunicationExplanationInferenceErrorV1> {
    validate_request_for_run(run, &request)?;
    let transition = match route_inference(channel, dispatcher, request) {
        Ok(result) => terminal_transition(run, result)?,
        Err(RouteInferenceErrorV1::Rejected) => CommunicationExplanationTransitionV1::Reject(
            CommunicationExplanationRejectionCodeV1::InferenceRejected,
        ),
        Err(RouteInferenceErrorV1::Unavailable) => {
            return Err(CommunicationExplanationInferenceErrorV1::Unavailable);
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
        .map_err(CommunicationExplanationInferenceErrorV1::Persistence)?;
    Ok(())
}

fn validate_request_for_run(
    run: &PersistedCommunicationExplanationRunV1,
    request: &CommunicationExplanationInferenceRequestV1,
) -> Result<(), CommunicationExplanationInferenceErrorV1> {
    validate_explanation_inference_request_v1(request)
        .map_err(|_| CommunicationExplanationInferenceErrorV1::InvalidRequest)?;
    let context = request
        .context
        .as_ref()
        .ok_or(CommunicationExplanationInferenceErrorV1::InvalidRequest)?;
    if request.run_id.as_slice() != run.draft.run_id
        || request.logical_owner_id != run.logical_owner_id
        || context.request_digest.as_slice()
            != run
                .status
                .inference_request_digest
                .ok_or(CommunicationExplanationInferenceErrorV1::InvalidRequest)?
        || request
            .source
            .as_ref()
            .map(|source| source.sha256.as_slice())
            != run.status.source_sha256.as_ref().map(<[u8; 32]>::as_slice)
    {
        return Err(CommunicationExplanationInferenceErrorV1::InvalidRequest);
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
    request: CommunicationExplanationInferenceRequestV1,
) -> Result<CommunicationExplanationInferenceResultV1, RouteInferenceErrorV1> {
    let request_id: [u8; 16] = request
        .run_id
        .as_slice()
        .try_into()
        .map_err(|_| RouteInferenceErrorV1::Rejected)?;
    let routed = ManagedRuntimeModuleRequestRequestV1 {
        request_id: request_id.to_vec(),
        contract: Some(communication_explanation_inference_contract_reference_v1()),
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
        CommunicationExplanationInferenceResultV1::decode(response.response_payload.as_slice())
            .map_err(|_| RouteInferenceErrorV1::Rejected)?;
    validate_explanation_inference_result_v1(&result)
        .map_err(|_| RouteInferenceErrorV1::Rejected)?;
    Ok(result)
}

fn terminal_transition(
    run: &PersistedCommunicationExplanationRunV1,
    result: CommunicationExplanationInferenceResultV1,
) -> Result<CommunicationExplanationTransitionV1, CommunicationExplanationInferenceErrorV1> {
    let request_digest = array32(&result.request_digest)?;
    let source_sha256 = array32(&result.source_sha256)?;
    if result.run_id.as_slice() != run.draft.run_id
        || Some(request_digest) != run.status.inference_request_digest
        || Some(source_sha256) != run.status.source_sha256
    {
        return Err(CommunicationExplanationInferenceErrorV1::InvalidResult);
    }
    let terminal = AiInferenceTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| CommunicationExplanationInferenceErrorV1::InvalidResult)?;
    match terminal {
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady => Ok(
            CommunicationExplanationTransitionV1::Complete(CommunicationExplanationCandidateV1 {
                reasons: result
                    .reasons
                    .into_iter()
                    .map(core_reason)
                    .collect::<Result<Vec<_>, _>>()?,
                completeness: core_completeness(result.completeness)?,
                confidence_basis_points: result.confidence_basis_points,
                request_digest,
                source_sha256,
            }),
        ),
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedPolicy => {
            Ok(CommunicationExplanationTransitionV1::Reject(
                CommunicationExplanationRejectionCodeV1::Policy,
            ))
        }
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedInput => {
            Ok(CommunicationExplanationTransitionV1::Reject(
                CommunicationExplanationRejectionCodeV1::InvalidRequest,
            ))
        }
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable
        | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected => {
            Ok(CommunicationExplanationTransitionV1::Reject(
                CommunicationExplanationRejectionCodeV1::InferenceRejected,
            ))
        }
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusUnspecified => {
            Err(CommunicationExplanationInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_reason(
    value: AiExplanationReasonV1,
) -> Result<CommunicationExplanationReasonV1, CommunicationExplanationInferenceErrorV1> {
    Ok(CommunicationExplanationReasonV1 {
        kind: core_reason_kind(value.kind)?,
        explanation_utf8: value.explanation_utf8,
        source_basis: core_source_basis(value.source_basis)?,
        confidence_basis_points: value.confidence_basis_points,
    })
}

fn core_reason_kind(
    value: i32,
) -> Result<CommunicationExplanationReasonKindV1, CommunicationExplanationInferenceErrorV1> {
    match AiExplanationReasonKindV1::try_from(value)
        .map_err(|_| CommunicationExplanationInferenceErrorV1::InvalidResult)?
    {
        AiExplanationReasonKindV1::AiExplanationReasonKindUrgency => {
            Ok(CommunicationExplanationReasonKindV1::Urgency)
        }
        AiExplanationReasonKindV1::AiExplanationReasonKindFinancialAttention => {
            Ok(CommunicationExplanationReasonKindV1::FinancialAttention)
        }
        AiExplanationReasonKindV1::AiExplanationReasonKindLegalOrContractual => {
            Ok(CommunicationExplanationReasonKindV1::LegalOrContractual)
        }
        AiExplanationReasonKindV1::AiExplanationReasonKindReplyRequested => {
            Ok(CommunicationExplanationReasonKindV1::ReplyRequested)
        }
        AiExplanationReasonKindV1::AiExplanationReasonKindDeadline => {
            Ok(CommunicationExplanationReasonKindV1::Deadline)
        }
        AiExplanationReasonKindV1::AiExplanationReasonKindAttachmentReference => {
            Ok(CommunicationExplanationReasonKindV1::AttachmentReference)
        }
        AiExplanationReasonKindV1::AiExplanationReasonKindMarketingOrBulk => {
            Ok(CommunicationExplanationReasonKindV1::MarketingOrBulk)
        }
        AiExplanationReasonKindV1::AiExplanationReasonKindOtherAttention => {
            Ok(CommunicationExplanationReasonKindV1::OtherAttention)
        }
        AiExplanationReasonKindV1::AiExplanationReasonKindUnspecified => {
            Err(CommunicationExplanationInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_source_basis(
    value: i32,
) -> Result<CommunicationExplanationSourceBasisV1, CommunicationExplanationInferenceErrorV1> {
    match AiExplanationSourceBasisV1::try_from(value)
        .map_err(|_| CommunicationExplanationInferenceErrorV1::InvalidResult)?
    {
        AiExplanationSourceBasisV1::AiExplanationSourceBasisSubject => {
            Ok(CommunicationExplanationSourceBasisV1::Subject)
        }
        AiExplanationSourceBasisV1::AiExplanationSourceBasisBody => {
            Ok(CommunicationExplanationSourceBasisV1::Body)
        }
        AiExplanationSourceBasisV1::AiExplanationSourceBasisCanonicalMetadata => {
            Ok(CommunicationExplanationSourceBasisV1::CanonicalMetadata)
        }
        AiExplanationSourceBasisV1::AiExplanationSourceBasisCombined => {
            Ok(CommunicationExplanationSourceBasisV1::Combined)
        }
        AiExplanationSourceBasisV1::AiExplanationSourceBasisUnspecified => {
            Err(CommunicationExplanationInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_completeness(
    value: i32,
) -> Result<CommunicationExplanationCompletenessV1, CommunicationExplanationInferenceErrorV1> {
    match AiInferenceCompletenessV1::try_from(value)
        .map_err(|_| CommunicationExplanationInferenceErrorV1::InvalidResult)?
    {
        AiInferenceCompletenessV1::AiInferenceCompletenessComplete => {
            Ok(CommunicationExplanationCompletenessV1::Complete)
        }
        AiInferenceCompletenessV1::AiInferenceCompletenessPartial => {
            Ok(CommunicationExplanationCompletenessV1::Partial)
        }
        AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified => {
            Err(CommunicationExplanationInferenceErrorV1::InvalidResult)
        }
    }
}

fn array32(value: &[u8]) -> Result<[u8; 32], CommunicationExplanationInferenceErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationExplanationInferenceErrorV1::InvalidResult)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_mapping_has_no_provider_specific_branch() {
        assert_eq!(
            core_reason_kind(AiExplanationReasonKindV1::AiExplanationReasonKindDeadline as i32)
                .expect("reason kind"),
            CommunicationExplanationReasonKindV1::Deadline
        );
    }
}
