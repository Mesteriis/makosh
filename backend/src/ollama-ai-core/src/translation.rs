use makosh_ai_contracts::{
    compute_provider_translation_request_digest_v1, validate_provider_translation_request_v1,
    validate_provider_translation_result_v1,
    wire::{
        AiDetectedLanguageV1, AiInferenceCompletenessV1, AiInferenceTerminalStatusV1,
        AiProviderTranslationRequestV1, AiProviderTranslationResultV1, AiTranslationLanguageV1,
    },
};
use makosh_ollama_ai_api::{OllamaAiRuntimeSettingsV1, valid_ollama_model_name_v1};
use serde::Deserialize;
use zeroize::Zeroizing;

use crate::{OllamaAiCoreErrorV1, OllamaAiRunStateV1, OllamaHttpGenerationV1};

const OLLAMA_TRANSLATION_POLICY_V1: &str = "Translate only the supplied text into the exact target language. Return one JSON object with exactly translated_text and detected_source_language string fields. detected_source_language must be unknown, english, spanish or russian. Preserve meaning and formatting. Do not add explanations. No markdown.";

#[derive(Clone, Eq, PartialEq)]
pub struct OllamaTranslationRunV1 {
    pub request_id: [u8; 16],
    pub request_digest: [u8; 32],
    pub settings_revision: u64,
    pub selected_model_digest: Option<[u8; 32]>,
    pub revision: u64,
    pub state: OllamaAiRunStateV1,
    pub terminal_result: Option<AiProviderTranslationResultV1>,
}

pub struct OllamaTranslationPlanV1 {
    pub request_id: [u8; 16],
    pub request_digest: [u8; 32],
    pub model: String,
    pub model_digest: [u8; 32],
    pub prompt_utf8: Zeroizing<Vec<u8>>,
    pub maximum_output_tokens: u32,
    pub timeout_millis: u64,
    pub settings_revision: u64,
    pub target_language: i32,
    pub maximum_output_bytes: u32,
}

pub fn accept_ollama_translation_request_v1(
    request: &AiProviderTranslationRequestV1,
    settings: &OllamaAiRuntimeSettingsV1,
) -> Result<OllamaTranslationRunV1, OllamaAiCoreErrorV1> {
    validate_provider_translation_request_v1(request)
        .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?;
    if settings.settings_revision == 0
        || settings.port == 0
        || settings.timeout_millis == 0
        || !valid_ollama_model_name_v1(&settings.chat_model)
    {
        return Err(OllamaAiCoreErrorV1::InvalidSettings);
    }
    Ok(OllamaTranslationRunV1 {
        request_id: request
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?,
        request_digest: compute_provider_translation_request_digest_v1(request)
            .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?,
        settings_revision: settings.settings_revision,
        selected_model_digest: None,
        revision: 1,
        state: OllamaAiRunStateV1::Accepted,
        terminal_result: None,
    })
}

pub fn begin_ollama_translation_request_v1(
    run: &OllamaTranslationRunV1,
    request: &AiProviderTranslationRequestV1,
    settings: &OllamaAiRuntimeSettingsV1,
    model_digest: [u8; 32],
) -> Result<(OllamaTranslationRunV1, OllamaTranslationPlanV1), OllamaAiCoreErrorV1> {
    if run.state != OllamaAiRunStateV1::Accepted
        || run.terminal_result.is_some()
        || run.request_id.as_slice() != request.request_id
        || run.request_digest
            != compute_provider_translation_request_digest_v1(request)
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
        "{OLLAMA_TRANSLATION_POLICY_V1}\nTarget language: {}\n\nSource:\n{source}",
        target_language_name(request.target_language)?,
    );
    Ok((
        OllamaTranslationRunV1 {
            selected_model_digest: Some(model_digest),
            revision: run.revision + 1,
            state: OllamaAiRunStateV1::Executing,
            ..run.clone()
        },
        OllamaTranslationPlanV1 {
            request_id: run.request_id,
            request_digest: run.request_digest,
            model: settings.chat_model.clone(),
            model_digest,
            prompt_utf8: Zeroizing::new(prompt.into_bytes()),
            maximum_output_tokens: request.maximum_output_tokens,
            timeout_millis: settings.timeout_millis,
            settings_revision: settings.settings_revision,
            target_language: request.target_language,
            maximum_output_bytes: request.maximum_output_bytes,
        },
    ))
}

pub fn complete_ollama_translation_request_v1(
    run: &OllamaTranslationRunV1,
    plan: &OllamaTranslationPlanV1,
    response: OllamaHttpGenerationV1,
) -> Result<OllamaTranslationRunV1, OllamaAiCoreErrorV1> {
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
    let candidate: OllamaTranslationJsonV1 = serde_json::from_slice(&response.content_json_utf8)
        .map_err(|_| OllamaAiCoreErrorV1::InvalidProviderResponse)?;
    if candidate.translated_text.is_empty()
        || candidate.translated_text.len() > plan.maximum_output_bytes as usize
    {
        return Err(OllamaAiCoreErrorV1::InvalidProviderResponse);
    }
    let result = AiProviderTranslationResultV1 {
        request_id: run.request_id.to_vec(),
        translated_text_utf8: candidate.translated_text.into_bytes(),
        detected_source_language: detected_language(&candidate.detected_source_language)?,
        target_language: plan.target_language,
        model_revision_sha256: plan.model_digest.to_vec(),
        input_tokens: response.input_tokens,
        output_tokens: response.output_tokens,
        terminal_status: AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady as i32,
        completeness: AiInferenceCompletenessV1::AiInferenceCompletenessComplete as i32,
        confidence_basis_points: 0,
        provider_settings_revision: plan.settings_revision,
    };
    validate_provider_translation_result_v1(&result)
        .map_err(|_| OllamaAiCoreErrorV1::InvalidProviderResponse)?;
    Ok(OllamaTranslationRunV1 {
        revision: run.revision + 1,
        state: OllamaAiRunStateV1::Ready,
        terminal_result: Some(result),
        ..run.clone()
    })
}

pub fn reject_ollama_translation_request_v1(
    run: &OllamaTranslationRunV1,
    terminal_status: AiInferenceTerminalStatusV1,
) -> Option<OllamaTranslationRunV1> {
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
    let result = AiProviderTranslationResultV1 {
        request_id: run.request_id.to_vec(),
        terminal_status: terminal_status as i32,
        ..Default::default()
    };
    validate_provider_translation_result_v1(&result)
        .is_ok()
        .then(|| OllamaTranslationRunV1 {
            revision: run.revision + 1,
            state: OllamaAiRunStateV1::Rejected,
            terminal_result: Some(result),
            ..run.clone()
        })
}

#[must_use]
pub fn mark_ollama_translation_uncertain_v1(
    run: &OllamaTranslationRunV1,
) -> Option<OllamaTranslationRunV1> {
    (run.state == OllamaAiRunStateV1::Executing && run.terminal_result.is_none()).then(|| {
        OllamaTranslationRunV1 {
            revision: run.revision + 1,
            state: OllamaAiRunStateV1::Uncertain,
            ..run.clone()
        }
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OllamaTranslationJsonV1 {
    translated_text: String,
    detected_source_language: String,
}

fn target_language_name(value: i32) -> Result<&'static str, OllamaAiCoreErrorV1> {
    match AiTranslationLanguageV1::try_from(value) {
        Ok(AiTranslationLanguageV1::AiTranslationLanguageEnglish) => Ok("english"),
        Ok(AiTranslationLanguageV1::AiTranslationLanguageSpanish) => Ok("spanish"),
        Ok(AiTranslationLanguageV1::AiTranslationLanguageRussian) => Ok("russian"),
        _ => Err(OllamaAiCoreErrorV1::InvalidRequest),
    }
}

fn detected_language(value: &str) -> Result<i32, OllamaAiCoreErrorV1> {
    let language = if value.eq_ignore_ascii_case("unknown") {
        AiDetectedLanguageV1::AiDetectedLanguageUnknown
    } else if value.eq_ignore_ascii_case("english") {
        AiDetectedLanguageV1::AiDetectedLanguageEnglish
    } else if value.eq_ignore_ascii_case("spanish") {
        AiDetectedLanguageV1::AiDetectedLanguageSpanish
    } else if value.eq_ignore_ascii_case("russian") {
        AiDetectedLanguageV1::AiDetectedLanguageRussian
    } else {
        return Err(OllamaAiCoreErrorV1::InvalidProviderResponse);
    };
    Ok(language as i32)
}

#[cfg(test)]
mod tests {
    use makosh_ai_contracts::{AI_LOCAL_EGRESS_POLICY_REVISION_V1, wire::AiEgressPolicyV1};

    use super::*;

    fn settings() -> OllamaAiRuntimeSettingsV1 {
        OllamaAiRuntimeSettingsV1 {
            settings_revision: 7,
            port: 11434,
            chat_model: "translation-model".to_owned(),
            timeout_millis: 5_000,
        }
    }

    #[test]
    fn translation_policy_has_closed_result_and_exact_target() {
        let request = AiProviderTranslationRequestV1 {
            request_id: vec![1; 16],
            input_utf8: b"Message body".to_vec(),
            target_language: AiTranslationLanguageV1::AiTranslationLanguageRussian as i32,
            maximum_output_bytes: 2_048,
            maximum_output_tokens: 256,
            egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
            egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        };
        let accepted = accept_ollama_translation_request_v1(&request, &settings()).expect("accept");
        let (executing, plan) =
            begin_ollama_translation_request_v1(&accepted, &request, &settings(), [2; 32])
                .expect("execute");
        let ready = complete_ollama_translation_request_v1(
            &executing,
            &plan,
            OllamaHttpGenerationV1 {
                content_json_utf8: Zeroizing::new(
                    r#"{"translated_text":"Текст сообщения","detected_source_language":"english"}"#
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
        assert_eq!(result.translated_text_utf8, "Текст сообщения".as_bytes());
        assert_eq!(result.target_language, request.target_language);
    }
}
