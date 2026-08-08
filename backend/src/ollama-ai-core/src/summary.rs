use makosh_ai_contracts::{
    compute_provider_summary_generation_request_digest_v1,
    validate_provider_summary_generation_request_v1,
    validate_provider_summary_generation_result_v1,
    wire::{
        AiInferenceCompletenessV1, AiInferenceTerminalStatusV1,
        AiProviderSummaryGenerationRequestV1, AiProviderSummaryGenerationResultV1,
        AiSummaryLanguageV1, AiSummaryLengthV1,
    },
};
use makosh_ollama_ai_api::{OllamaAiRuntimeSettingsV1, valid_ollama_model_name_v1};
use serde::Deserialize;
use zeroize::Zeroizing;

use crate::{OllamaAiCoreErrorV1, OllamaAiRunStateV1, OllamaHttpGenerationV1};

const OLLAMA_SUMMARY_POLICY_V1: &str = "Summarize only the supplied communication. Return one JSON object with exactly summary and language string fields. Language must be english, spanish or russian. Do not infer tasks, deadlines, people or organizations. No markdown.";

#[derive(Clone, Eq, PartialEq)]
pub struct OllamaSummaryRunV1 {
    pub request_id: [u8; 16],
    pub request_digest: [u8; 32],
    pub settings_revision: u64,
    pub selected_model_digest: Option<[u8; 32]>,
    pub revision: u64,
    pub state: OllamaAiRunStateV1,
    pub terminal_result: Option<AiProviderSummaryGenerationResultV1>,
}

pub struct OllamaSummaryGenerationPlanV1 {
    pub request_id: [u8; 16],
    pub request_digest: [u8; 32],
    pub model: String,
    pub model_digest: [u8; 32],
    pub prompt_utf8: Zeroizing<Vec<u8>>,
    pub maximum_output_tokens: u32,
    pub timeout_millis: u64,
    pub settings_revision: u64,
    pub language: i32,
    pub length: i32,
    pub maximum_output_bytes: u32,
}

pub fn accept_ollama_summary_request_v1(
    request: &AiProviderSummaryGenerationRequestV1,
    settings: &OllamaAiRuntimeSettingsV1,
) -> Result<OllamaSummaryRunV1, OllamaAiCoreErrorV1> {
    validate_provider_summary_generation_request_v1(request)
        .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?;
    if settings.settings_revision == 0
        || settings.port == 0
        || settings.timeout_millis == 0
        || !valid_ollama_model_name_v1(&settings.chat_model)
    {
        return Err(OllamaAiCoreErrorV1::InvalidSettings);
    }
    Ok(OllamaSummaryRunV1 {
        request_id: request
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?,
        request_digest: compute_provider_summary_generation_request_digest_v1(request)
            .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?,
        settings_revision: settings.settings_revision,
        selected_model_digest: None,
        revision: 1,
        state: OllamaAiRunStateV1::Accepted,
        terminal_result: None,
    })
}

pub fn begin_ollama_summary_request_v1(
    run: &OllamaSummaryRunV1,
    request: &AiProviderSummaryGenerationRequestV1,
    settings: &OllamaAiRuntimeSettingsV1,
    model_digest: [u8; 32],
) -> Result<(OllamaSummaryRunV1, OllamaSummaryGenerationPlanV1), OllamaAiCoreErrorV1> {
    if run.state != OllamaAiRunStateV1::Accepted
        || run.terminal_result.is_some()
        || run.request_id.as_slice() != request.request_id
        || run.request_digest
            != compute_provider_summary_generation_request_digest_v1(request)
                .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?
        || run.settings_revision != settings.settings_revision
        || run.selected_model_digest.is_some()
        || model_digest == [0; 32]
    {
        return Err(OllamaAiCoreErrorV1::InvalidTransition);
    }
    let source = std::str::from_utf8(&request.input_utf8)
        .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?;
    let prompt = format!(
        "{OLLAMA_SUMMARY_POLICY_V1}\nLanguage: {}\nLength: {}\n\nSource:\n{source}",
        language_name(request.language)?,
        length_name(request.length)?,
    );
    Ok((
        OllamaSummaryRunV1 {
            selected_model_digest: Some(model_digest),
            revision: run.revision + 1,
            state: OllamaAiRunStateV1::Executing,
            ..run.clone()
        },
        OllamaSummaryGenerationPlanV1 {
            request_id: run.request_id,
            request_digest: run.request_digest,
            model: settings.chat_model.clone(),
            model_digest,
            prompt_utf8: Zeroizing::new(prompt.into_bytes()),
            maximum_output_tokens: request.maximum_output_tokens,
            timeout_millis: settings.timeout_millis,
            settings_revision: settings.settings_revision,
            language: request.language,
            length: request.length,
            maximum_output_bytes: request.maximum_output_bytes,
        },
    ))
}

pub fn complete_ollama_summary_request_v1(
    run: &OllamaSummaryRunV1,
    plan: &OllamaSummaryGenerationPlanV1,
    response: OllamaHttpGenerationV1,
) -> Result<OllamaSummaryRunV1, OllamaAiCoreErrorV1> {
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
    let candidate: OllamaSummaryJsonV1 = serde_json::from_slice(&response.content_json_utf8)
        .map_err(|_| OllamaAiCoreErrorV1::InvalidProviderResponse)?;
    if candidate.summary.is_empty() || candidate.summary.len() > plan.maximum_output_bytes as usize
    {
        return Err(OllamaAiCoreErrorV1::InvalidProviderResponse);
    }
    let result = AiProviderSummaryGenerationResultV1 {
        request_id: run.request_id.to_vec(),
        summary_utf8: candidate.summary.into_bytes(),
        resolved_language: resolved_language(plan.language, &candidate.language)?,
        resolved_length: plan.length,
        model_revision_sha256: plan.model_digest.to_vec(),
        input_tokens: response.input_tokens,
        output_tokens: response.output_tokens,
        terminal_status: AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady as i32,
        completeness: AiInferenceCompletenessV1::AiInferenceCompletenessComplete as i32,
        confidence_basis_points: 0,
        provider_settings_revision: plan.settings_revision,
    };
    validate_provider_summary_generation_result_v1(&result)
        .map_err(|_| OllamaAiCoreErrorV1::InvalidProviderResponse)?;
    Ok(OllamaSummaryRunV1 {
        revision: run.revision + 1,
        state: OllamaAiRunStateV1::Ready,
        terminal_result: Some(result),
        ..run.clone()
    })
}

pub fn reject_ollama_summary_request_v1(
    run: &OllamaSummaryRunV1,
    terminal_status: AiInferenceTerminalStatusV1,
) -> Option<OllamaSummaryRunV1> {
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
    let result = AiProviderSummaryGenerationResultV1 {
        request_id: run.request_id.to_vec(),
        terminal_status: terminal_status as i32,
        ..Default::default()
    };
    validate_provider_summary_generation_result_v1(&result)
        .is_ok()
        .then(|| OllamaSummaryRunV1 {
            revision: run.revision + 1,
            state: OllamaAiRunStateV1::Rejected,
            terminal_result: Some(result),
            ..run.clone()
        })
}

#[must_use]
pub fn mark_ollama_summary_uncertain_v1(run: &OllamaSummaryRunV1) -> Option<OllamaSummaryRunV1> {
    (run.state == OllamaAiRunStateV1::Executing && run.terminal_result.is_none()).then(|| {
        OllamaSummaryRunV1 {
            revision: run.revision + 1,
            state: OllamaAiRunStateV1::Uncertain,
            ..run.clone()
        }
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OllamaSummaryJsonV1 {
    summary: String,
    language: String,
}

fn language_name(value: i32) -> Result<&'static str, OllamaAiCoreErrorV1> {
    match AiSummaryLanguageV1::try_from(value) {
        Ok(AiSummaryLanguageV1::AiSummaryLanguageAuto) => Ok("auto"),
        Ok(AiSummaryLanguageV1::AiSummaryLanguageEnglish) => Ok("english"),
        Ok(AiSummaryLanguageV1::AiSummaryLanguageSpanish) => Ok("spanish"),
        Ok(AiSummaryLanguageV1::AiSummaryLanguageRussian) => Ok("russian"),
        _ => Err(OllamaAiCoreErrorV1::InvalidRequest),
    }
}

fn length_name(value: i32) -> Result<&'static str, OllamaAiCoreErrorV1> {
    match AiSummaryLengthV1::try_from(value) {
        Ok(AiSummaryLengthV1::AiSummaryLengthShort) => Ok("short"),
        Ok(AiSummaryLengthV1::AiSummaryLengthStandard) => Ok("standard"),
        Ok(AiSummaryLengthV1::AiSummaryLengthDetailed) => Ok("detailed"),
        _ => Err(OllamaAiCoreErrorV1::InvalidRequest),
    }
}

fn resolved_language(requested: i32, reported: &str) -> Result<i32, OllamaAiCoreErrorV1> {
    let resolved = if reported.eq_ignore_ascii_case("english") {
        AiSummaryLanguageV1::AiSummaryLanguageEnglish
    } else if reported.eq_ignore_ascii_case("spanish") {
        AiSummaryLanguageV1::AiSummaryLanguageSpanish
    } else if reported.eq_ignore_ascii_case("russian") {
        AiSummaryLanguageV1::AiSummaryLanguageRussian
    } else {
        return Err(OllamaAiCoreErrorV1::InvalidProviderResponse);
    };
    if requested != AiSummaryLanguageV1::AiSummaryLanguageAuto as i32
        && requested != resolved as i32
    {
        return Err(OllamaAiCoreErrorV1::InvalidProviderResponse);
    }
    Ok(resolved as i32)
}

#[cfg(test)]
mod tests {
    use makosh_ai_contracts::{AI_LOCAL_EGRESS_POLICY_REVISION_V1, wire::AiEgressPolicyV1};
    use makosh_ollama_ai_api::OllamaAiRuntimeSettingsV1;

    use super::*;

    fn settings() -> OllamaAiRuntimeSettingsV1 {
        OllamaAiRuntimeSettingsV1 {
            settings_revision: 7,
            port: 11434,
            chat_model: "summary-model".to_owned(),
            timeout_millis: 5_000,
        }
    }

    #[test]
    fn summary_policy_has_its_own_structured_output() {
        let request = AiProviderSummaryGenerationRequestV1 {
            request_id: vec![1; 16],
            input_utf8: b"private source".to_vec(),
            language: AiSummaryLanguageV1::AiSummaryLanguageEnglish as i32,
            length: AiSummaryLengthV1::AiSummaryLengthShort as i32,
            maximum_output_bytes: 2048,
            maximum_output_tokens: 256,
            egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
            egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        };
        let accepted = accept_ollama_summary_request_v1(&request, &settings()).expect("accepted");
        let (executing, plan) =
            begin_ollama_summary_request_v1(&accepted, &request, &settings(), [2; 32])
                .expect("executing");
        let ready = complete_ollama_summary_request_v1(
            &executing,
            &plan,
            OllamaHttpGenerationV1 {
                content_json_utf8: Zeroizing::new(
                    br#"{"summary":"Bounded summary","language":"english"}"#.to_vec(),
                ),
                model_digest: [2; 32],
                input_tokens: 8,
                output_tokens: 3,
            },
        )
        .expect("ready");
        assert_eq!(
            ready.terminal_result.expect("result").summary_utf8,
            b"Bounded summary"
        );
    }
}
