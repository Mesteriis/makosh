use makosh_ai_contracts::{
    compute_provider_explanation_request_digest_v1, validate_provider_explanation_request_v1,
    validate_provider_explanation_result_v1,
    wire::{
        AiExplanationReasonKindV1, AiExplanationReasonV1, AiExplanationSourceBasisV1,
        AiInferenceCompletenessV1, AiInferenceTerminalStatusV1, AiProviderExplanationRequestV1,
        AiProviderExplanationResultV1,
    },
};
use makosh_ollama_ai_api::{OllamaAiRuntimeSettingsV1, valid_ollama_model_name_v1};
use serde::Deserialize;
use zeroize::Zeroizing;

use crate::{OllamaAiCoreErrorV1, OllamaAiRunStateV1, OllamaHttpGenerationV1};

const OLLAMA_EXPLANATION_POLICY_V1: &str = "Explain only why the supplied communication may require attention. Return one JSON object with exactly reasons, completeness and confidence_basis_points. Each reason has exactly kind, explanation, source_basis and confidence_basis_points. kind must be urgency, financial_attention, legal_or_contractual, reply_requested, deadline, attachment_reference, marketing_or_bulk or other_attention. source_basis must be subject, body, canonical_metadata or combined. completeness must be complete or partial. Do not invent facts or new taxonomy. No markdown.";

#[derive(Clone, PartialEq)]
pub struct OllamaExplanationRunV1 {
    pub request_id: [u8; 16],
    pub request_digest: [u8; 32],
    pub settings_revision: u64,
    pub selected_model_digest: Option<[u8; 32]>,
    pub revision: u64,
    pub state: OllamaAiRunStateV1,
    pub terminal_result: Option<AiProviderExplanationResultV1>,
}

pub struct OllamaExplanationPlanV1 {
    pub request_id: [u8; 16],
    pub request_digest: [u8; 32],
    pub model: String,
    pub model_digest: [u8; 32],
    pub prompt_utf8: Zeroizing<Vec<u8>>,
    pub maximum_output_tokens: u32,
    pub timeout_millis: u64,
    pub settings_revision: u64,
    pub maximum_reasons: u32,
    pub maximum_reason_text_bytes: u32,
    pub maximum_response_bytes: u32,
}

pub fn accept_ollama_explanation_request_v1(
    request: &AiProviderExplanationRequestV1,
    settings: &OllamaAiRuntimeSettingsV1,
) -> Result<OllamaExplanationRunV1, OllamaAiCoreErrorV1> {
    validate_provider_explanation_request_v1(request)
        .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?;
    if settings.settings_revision == 0
        || settings.port == 0
        || settings.timeout_millis == 0
        || !valid_ollama_model_name_v1(&settings.chat_model)
    {
        return Err(OllamaAiCoreErrorV1::InvalidSettings);
    }
    Ok(OllamaExplanationRunV1 {
        request_id: request
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?,
        request_digest: compute_provider_explanation_request_digest_v1(request)
            .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?,
        settings_revision: settings.settings_revision,
        selected_model_digest: None,
        revision: 1,
        state: OllamaAiRunStateV1::Accepted,
        terminal_result: None,
    })
}

pub fn begin_ollama_explanation_request_v1(
    run: &OllamaExplanationRunV1,
    request: &AiProviderExplanationRequestV1,
    settings: &OllamaAiRuntimeSettingsV1,
    model_digest: [u8; 32],
) -> Result<(OllamaExplanationRunV1, OllamaExplanationPlanV1), OllamaAiCoreErrorV1> {
    if run.state != OllamaAiRunStateV1::Accepted
        || run.terminal_result.is_some()
        || run.request_id.as_slice() != request.request_id
        || run.request_digest
            != compute_provider_explanation_request_digest_v1(request)
                .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?
        || run.settings_revision != settings.settings_revision
        || run.selected_model_digest.is_some()
        || model_digest == [0; 32]
    {
        return Err(OllamaAiCoreErrorV1::InvalidTransition);
    }
    let source = std::str::from_utf8(&request.input_utf8)
        .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?;
    let prompt = format!("{OLLAMA_EXPLANATION_POLICY_V1}\n\nSource:\n{source}");
    Ok((
        OllamaExplanationRunV1 {
            selected_model_digest: Some(model_digest),
            revision: run.revision + 1,
            state: OllamaAiRunStateV1::Executing,
            ..run.clone()
        },
        OllamaExplanationPlanV1 {
            request_id: run.request_id,
            request_digest: run.request_digest,
            model: settings.chat_model.clone(),
            model_digest,
            prompt_utf8: Zeroizing::new(prompt.into_bytes()),
            maximum_output_tokens: request.maximum_output_tokens,
            timeout_millis: settings.timeout_millis,
            settings_revision: settings.settings_revision,
            maximum_reasons: request.maximum_reasons,
            maximum_reason_text_bytes: request.maximum_reason_text_bytes,
            maximum_response_bytes: request
                .maximum_reasons
                .saturating_mul(request.maximum_reason_text_bytes.saturating_add(128))
                .saturating_add(256),
        },
    ))
}

pub fn complete_ollama_explanation_request_v1(
    run: &OllamaExplanationRunV1,
    plan: &OllamaExplanationPlanV1,
    response: OllamaHttpGenerationV1,
) -> Result<OllamaExplanationRunV1, OllamaAiCoreErrorV1> {
    if run.state != OllamaAiRunStateV1::Executing
        || run.terminal_result.is_some()
        || run.request_id != plan.request_id
        || run.request_digest != plan.request_digest
        || run.settings_revision != plan.settings_revision
        || run.selected_model_digest != Some(plan.model_digest)
        || response.model_digest != plan.model_digest
    {
        return Err(OllamaAiCoreErrorV1::InvalidTransition);
    }
    let candidate: OllamaExplanationJsonV1 = serde_json::from_slice(&response.content_json_utf8)
        .map_err(|_| OllamaAiCoreErrorV1::InvalidProviderResponse)?;
    if candidate.reasons.len() > plan.maximum_reasons as usize {
        return Err(OllamaAiCoreErrorV1::InvalidProviderResponse);
    }
    let result = AiProviderExplanationResultV1 {
        request_id: run.request_id.to_vec(),
        reasons: candidate
            .reasons
            .into_iter()
            .map(|reason| provider_reason(reason, plan.maximum_reason_text_bytes))
            .collect::<Result<Vec<_>, _>>()?,
        model_revision_sha256: plan.model_digest.to_vec(),
        input_tokens: response.input_tokens,
        output_tokens: response.output_tokens,
        terminal_status: AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady as i32,
        completeness: completeness(&candidate.completeness)?,
        confidence_basis_points: candidate.confidence_basis_points,
        provider_settings_revision: plan.settings_revision,
    };
    validate_provider_explanation_result_v1(&result)
        .map_err(|_| OllamaAiCoreErrorV1::InvalidProviderResponse)?;
    Ok(OllamaExplanationRunV1 {
        revision: run.revision + 1,
        state: OllamaAiRunStateV1::Ready,
        terminal_result: Some(result),
        ..run.clone()
    })
}

pub fn reject_ollama_explanation_request_v1(
    run: &OllamaExplanationRunV1,
    terminal_status: AiInferenceTerminalStatusV1,
) -> Option<OllamaExplanationRunV1> {
    if !matches!(
        terminal_status,
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedPolicy
            | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedInput
            | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected
            | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable
    ) || !matches!(
        run.state,
        OllamaAiRunStateV1::Accepted | OllamaAiRunStateV1::Executing
    ) || run.terminal_result.is_some()
    {
        return None;
    }
    let result = AiProviderExplanationResultV1 {
        request_id: run.request_id.to_vec(),
        terminal_status: terminal_status as i32,
        ..Default::default()
    };
    validate_provider_explanation_result_v1(&result)
        .is_ok()
        .then(|| OllamaExplanationRunV1 {
            revision: run.revision + 1,
            state: OllamaAiRunStateV1::Rejected,
            terminal_result: Some(result),
            ..run.clone()
        })
}

#[must_use]
pub fn mark_ollama_explanation_uncertain_v1(
    run: &OllamaExplanationRunV1,
) -> Option<OllamaExplanationRunV1> {
    (run.state == OllamaAiRunStateV1::Executing && run.terminal_result.is_none()).then(|| {
        OllamaExplanationRunV1 {
            revision: run.revision + 1,
            state: OllamaAiRunStateV1::Uncertain,
            ..run.clone()
        }
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OllamaExplanationJsonV1 {
    reasons: Vec<OllamaExplanationReasonJsonV1>,
    completeness: String,
    confidence_basis_points: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OllamaExplanationReasonJsonV1 {
    kind: String,
    explanation: String,
    source_basis: String,
    confidence_basis_points: u32,
}

fn provider_reason(
    reason: OllamaExplanationReasonJsonV1,
    maximum_text_bytes: u32,
) -> Result<AiExplanationReasonV1, OllamaAiCoreErrorV1> {
    if reason.explanation.is_empty()
        || reason.explanation.len() > maximum_text_bytes as usize
        || reason.confidence_basis_points > 10_000
    {
        return Err(OllamaAiCoreErrorV1::InvalidProviderResponse);
    }
    Ok(AiExplanationReasonV1 {
        kind: reason_kind(&reason.kind)?,
        explanation_utf8: reason.explanation.into_bytes(),
        source_basis: source_basis(&reason.source_basis)?,
        confidence_basis_points: reason.confidence_basis_points,
    })
}

fn reason_kind(value: &str) -> Result<i32, OllamaAiCoreErrorV1> {
    let value = match value {
        "urgency" => AiExplanationReasonKindV1::AiExplanationReasonKindUrgency,
        "financial_attention" => {
            AiExplanationReasonKindV1::AiExplanationReasonKindFinancialAttention
        }
        "legal_or_contractual" => {
            AiExplanationReasonKindV1::AiExplanationReasonKindLegalOrContractual
        }
        "reply_requested" => AiExplanationReasonKindV1::AiExplanationReasonKindReplyRequested,
        "deadline" => AiExplanationReasonKindV1::AiExplanationReasonKindDeadline,
        "attachment_reference" => {
            AiExplanationReasonKindV1::AiExplanationReasonKindAttachmentReference
        }
        "marketing_or_bulk" => AiExplanationReasonKindV1::AiExplanationReasonKindMarketingOrBulk,
        "other_attention" => AiExplanationReasonKindV1::AiExplanationReasonKindOtherAttention,
        _ => return Err(OllamaAiCoreErrorV1::InvalidProviderResponse),
    };
    Ok(value as i32)
}

fn source_basis(value: &str) -> Result<i32, OllamaAiCoreErrorV1> {
    let value = match value {
        "subject" => AiExplanationSourceBasisV1::AiExplanationSourceBasisSubject,
        "body" => AiExplanationSourceBasisV1::AiExplanationSourceBasisBody,
        "canonical_metadata" => {
            AiExplanationSourceBasisV1::AiExplanationSourceBasisCanonicalMetadata
        }
        "combined" => AiExplanationSourceBasisV1::AiExplanationSourceBasisCombined,
        _ => return Err(OllamaAiCoreErrorV1::InvalidProviderResponse),
    };
    Ok(value as i32)
}

fn completeness(value: &str) -> Result<i32, OllamaAiCoreErrorV1> {
    match value {
        "complete" => Ok(AiInferenceCompletenessV1::AiInferenceCompletenessComplete as i32),
        "partial" => Ok(AiInferenceCompletenessV1::AiInferenceCompletenessPartial as i32),
        _ => Err(OllamaAiCoreErrorV1::InvalidProviderResponse),
    }
}

#[cfg(test)]
mod tests {
    use makosh_ai_contracts::{AI_LOCAL_EGRESS_POLICY_REVISION_V1, wire::AiEgressPolicyV1};

    use super::*;

    fn settings() -> OllamaAiRuntimeSettingsV1 {
        OllamaAiRuntimeSettingsV1 {
            settings_revision: 7,
            port: 11434,
            chat_model: "explanation-model".to_owned(),
            timeout_millis: 5_000,
        }
    }

    #[test]
    fn explanation_policy_has_closed_reason_taxonomy() {
        let request = AiProviderExplanationRequestV1 {
            request_id: vec![1; 16],
            input_utf8: b"Message body".to_vec(),
            maximum_reasons: 8,
            maximum_reason_text_bytes: 512,
            maximum_output_tokens: 256,
            egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
            egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        };
        let accepted = accept_ollama_explanation_request_v1(&request, &settings()).expect("accept");
        let (executing, plan) =
            begin_ollama_explanation_request_v1(&accepted, &request, &settings(), [2; 32])
                .expect("execute");
        let ready = complete_ollama_explanation_request_v1(
            &executing,
            &plan,
            OllamaHttpGenerationV1 {
                content_json_utf8: Zeroizing::new(
                    r#"{"reasons":[{"kind":"deadline","explanation":"A deadline is stated.","source_basis":"body","confidence_basis_points":8000}],"completeness":"complete","confidence_basis_points":8000}"#
                        .as_bytes()
                        .to_vec(),
                ),
                model_digest: [2; 32],
                input_tokens: 8,
                output_tokens: 3,
            },
        )
        .expect("ready");
        let result = ready.terminal_result.expect("result");
        assert_eq!(result.reasons.len(), 1);
        assert_eq!(
            result.reasons[0].kind,
            AiExplanationReasonKindV1::AiExplanationReasonKindDeadline as i32
        );

        let ready_without_reasons = complete_ollama_explanation_request_v1(
            &executing,
            &plan,
            OllamaHttpGenerationV1 {
                content_json_utf8: Zeroizing::new(
                    br#"{"reasons":[],"completeness":"complete","confidence_basis_points":10000}"#
                        .to_vec(),
                ),
                model_digest: [2; 32],
                input_tokens: 8,
                output_tokens: 1,
            },
        )
        .expect("empty reason list remains valid");
        assert!(
            ready_without_reasons
                .terminal_result
                .expect("result")
                .reasons
                .is_empty(),
            "allows_empty_reason_list_without_fabricating_a_reason"
        );
    }
}
