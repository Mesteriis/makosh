#![forbid(unsafe_code)]

mod explanation;
mod summary;
mod translation;

pub use explanation::{
    OllamaExplanationPlanV1, OllamaExplanationRunV1, accept_ollama_explanation_request_v1,
    begin_ollama_explanation_request_v1, complete_ollama_explanation_request_v1,
    mark_ollama_explanation_uncertain_v1, reject_ollama_explanation_request_v1,
};
pub use summary::{
    OllamaSummaryGenerationPlanV1, OllamaSummaryRunV1, accept_ollama_summary_request_v1,
    begin_ollama_summary_request_v1, complete_ollama_summary_request_v1,
    mark_ollama_summary_uncertain_v1, reject_ollama_summary_request_v1,
};
pub use translation::{
    OllamaTranslationPlanV1, OllamaTranslationRunV1, accept_ollama_translation_request_v1,
    begin_ollama_translation_request_v1, complete_ollama_translation_request_v1,
    mark_ollama_translation_uncertain_v1, reject_ollama_translation_request_v1,
};

use makosh_ai_contracts::{
    AI_MAX_SUBJECT_BYTES_V1, compute_provider_reply_generation_request_digest_v1,
    validate_provider_reply_generation_request_v1, validate_provider_reply_generation_result_v1,
    wire::{
        AiInferenceCompletenessV1, AiInferenceTerminalStatusV1, AiProviderReplyGenerationRequestV1,
        AiProviderReplyGenerationResultV1, AiReplyLanguageV1, AiReplySubjectPolicyV1,
        AiReplyToneV1,
    },
};
use makosh_ollama_ai_api::{OllamaAiRuntimeSettingsV1, valid_ollama_model_name_v1};
use serde::Deserialize;
use zeroize::Zeroizing;

pub const PACKAGE: &str = "makosh-ollama-ai-core";
const OLLAMA_REPLY_POLICY_V1: &str = "Return one JSON object with exactly subject, body and language string fields. Language must be english, spanish or russian. No markdown.";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OllamaAiRunStateV1 {
    Accepted,
    Executing,
    Ready,
    Rejected,
    Uncertain,
}

#[derive(Clone, Eq, PartialEq)]
pub struct OllamaAiRunV1 {
    pub request_id: [u8; 16],
    pub request_digest: [u8; 32],
    pub settings_revision: u64,
    pub selected_model_digest: Option<[u8; 32]>,
    pub revision: u64,
    pub state: OllamaAiRunStateV1,
    pub terminal_result: Option<AiProviderReplyGenerationResultV1>,
}

pub struct OllamaGenerationPlanV1 {
    pub request_id: [u8; 16],
    pub request_digest: [u8; 32],
    pub model: String,
    pub model_digest: [u8; 32],
    pub prompt_utf8: Zeroizing<Vec<u8>>,
    pub maximum_output_tokens: u32,
    pub timeout_millis: u64,
    pub settings_revision: u64,
    pub tone: i32,
    pub language: i32,
    pub subject_policy: i32,
    pub maximum_output_bytes: u32,
}

pub struct OllamaHttpGenerationV1 {
    pub content_json_utf8: Zeroizing<Vec<u8>>,
    pub model_digest: [u8; 32],
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OllamaAiCoreErrorV1 {
    InvalidRequest,
    InvalidSettings,
    InvalidTransition,
    InvalidProviderResponse,
}

pub fn accept_ollama_request_v1(
    request: &AiProviderReplyGenerationRequestV1,
    settings: &OllamaAiRuntimeSettingsV1,
) -> Result<OllamaAiRunV1, OllamaAiCoreErrorV1> {
    validate_provider_reply_generation_request_v1(request)
        .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?;
    if settings.settings_revision == 0
        || settings.port == 0
        || settings.timeout_millis == 0
        || !valid_ollama_model_name_v1(&settings.chat_model)
    {
        return Err(OllamaAiCoreErrorV1::InvalidSettings);
    }
    Ok(OllamaAiRunV1 {
        request_id: request
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?,
        request_digest: compute_provider_reply_generation_request_digest_v1(request)
            .map_err(|_| OllamaAiCoreErrorV1::InvalidRequest)?,
        settings_revision: settings.settings_revision,
        selected_model_digest: None,
        revision: 1,
        state: OllamaAiRunStateV1::Accepted,
        terminal_result: None,
    })
}

pub fn begin_ollama_request_v1(
    run: &OllamaAiRunV1,
    request: &AiProviderReplyGenerationRequestV1,
    settings: &OllamaAiRuntimeSettingsV1,
    model_digest: [u8; 32],
) -> Result<(OllamaAiRunV1, OllamaGenerationPlanV1), OllamaAiCoreErrorV1> {
    if run.state != OllamaAiRunStateV1::Accepted
        || run.terminal_result.is_some()
        || run.request_id.as_slice() != request.request_id
        || run.request_digest
            != compute_provider_reply_generation_request_digest_v1(request)
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
        "{policy}\nTone: {tone}\nLanguage: {language}\nSubject policy: {subject_policy}\n\nSource:\n{source}",
        policy = OLLAMA_REPLY_POLICY_V1,
        tone = tone_name(request.tone)?,
        language = language_name(request.language)?,
        subject_policy = subject_policy_name(request.subject_policy)?,
    );
    let next = OllamaAiRunV1 {
        selected_model_digest: Some(model_digest),
        revision: run.revision + 1,
        state: OllamaAiRunStateV1::Executing,
        ..run.clone()
    };
    let plan = OllamaGenerationPlanV1 {
        request_id: run.request_id,
        request_digest: run.request_digest,
        model: settings.chat_model.clone(),
        model_digest,
        prompt_utf8: Zeroizing::new(prompt.into_bytes()),
        maximum_output_tokens: request.maximum_output_tokens,
        timeout_millis: settings.timeout_millis,
        settings_revision: settings.settings_revision,
        tone: request.tone,
        language: request.language,
        subject_policy: request.subject_policy,
        maximum_output_bytes: request.maximum_output_bytes,
    };
    Ok((next, plan))
}

pub fn complete_ollama_request_v1(
    run: &OllamaAiRunV1,
    plan: &OllamaGenerationPlanV1,
    response: OllamaHttpGenerationV1,
) -> Result<OllamaAiRunV1, OllamaAiCoreErrorV1> {
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
    let mut candidate: OllamaReplyJsonV1 = serde_json::from_slice(&response.content_json_utf8)
        .map_err(|_| OllamaAiCoreErrorV1::InvalidProviderResponse)?;
    if candidate.subject.len() > AI_MAX_SUBJECT_BYTES_V1
        || candidate.body.is_empty()
        || candidate.body.len() > plan.maximum_output_bytes as usize
    {
        return Err(OllamaAiCoreErrorV1::InvalidProviderResponse);
    }
    let resolved_language = resolved_language(plan.language, &candidate.language)?;
    let subject = match AiReplySubjectPolicyV1::try_from(plan.subject_policy) {
        Ok(AiReplySubjectPolicyV1::AiReplySubjectPolicyOmit) if candidate.subject.is_empty() => {
            Vec::new()
        }
        Ok(AiReplySubjectPolicyV1::AiReplySubjectPolicyOmit) => {
            return Err(OllamaAiCoreErrorV1::InvalidProviderResponse);
        }
        Ok(_) => std::mem::take(&mut candidate.subject).into_bytes(),
        Err(_) => return Err(OllamaAiCoreErrorV1::InvalidProviderResponse),
    };
    let result = AiProviderReplyGenerationResultV1 {
        request_id: run.request_id.to_vec(),
        subject_utf8: subject,
        body_utf8: std::mem::take(&mut candidate.body).into_bytes(),
        resolved_tone: plan.tone,
        resolved_language,
        model_revision_sha256: plan.model_digest.to_vec(),
        input_tokens: response.input_tokens,
        output_tokens: response.output_tokens,
        terminal_status: AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady as i32,
        completeness: AiInferenceCompletenessV1::AiInferenceCompletenessComplete as i32,
        confidence_basis_points: 0,
        provider_settings_revision: plan.settings_revision,
    };
    if validate_provider_reply_generation_result_v1(&result).is_err() {
        return Err(OllamaAiCoreErrorV1::InvalidProviderResponse);
    }
    Ok(OllamaAiRunV1 {
        revision: run.revision + 1,
        state: OllamaAiRunStateV1::Ready,
        terminal_result: Some(result),
        ..run.clone()
    })
}

pub fn reject_ollama_request_v1(
    run: &OllamaAiRunV1,
    terminal_status: AiInferenceTerminalStatusV1,
) -> Option<OllamaAiRunV1> {
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
    let result = AiProviderReplyGenerationResultV1 {
        request_id: run.request_id.to_vec(),
        terminal_status: terminal_status as i32,
        ..Default::default()
    };
    validate_provider_reply_generation_result_v1(&result)
        .is_ok()
        .then(|| OllamaAiRunV1 {
            revision: run.revision + 1,
            state: OllamaAiRunStateV1::Rejected,
            terminal_result: Some(result),
            ..run.clone()
        })
}

#[must_use]
pub fn mark_ollama_uncertain_v1(run: &OllamaAiRunV1) -> Option<OllamaAiRunV1> {
    (run.state == OllamaAiRunStateV1::Executing && run.terminal_result.is_none()).then(|| {
        OllamaAiRunV1 {
            revision: run.revision + 1,
            state: OllamaAiRunStateV1::Uncertain,
            ..run.clone()
        }
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OllamaReplyJsonV1 {
    subject: String,
    body: String,
    language: String,
}

fn resolved_language(requested: i32, reported: &str) -> Result<i32, OllamaAiCoreErrorV1> {
    let resolved = if reported.eq_ignore_ascii_case("english") {
        AiReplyLanguageV1::AiReplyLanguageEnglish
    } else if reported.eq_ignore_ascii_case("spanish") {
        AiReplyLanguageV1::AiReplyLanguageSpanish
    } else if reported.eq_ignore_ascii_case("russian") {
        AiReplyLanguageV1::AiReplyLanguageRussian
    } else {
        return Err(OllamaAiCoreErrorV1::InvalidProviderResponse);
    };
    if requested != AiReplyLanguageV1::AiReplyLanguageAuto as i32 && requested != resolved as i32 {
        return Err(OllamaAiCoreErrorV1::InvalidProviderResponse);
    }
    Ok(resolved as i32)
}

fn tone_name(value: i32) -> Result<&'static str, OllamaAiCoreErrorV1> {
    match AiReplyToneV1::try_from(value) {
        Ok(AiReplyToneV1::AiReplyToneNeutral) => Ok("neutral"),
        Ok(AiReplyToneV1::AiReplyToneWarm) => Ok("warm"),
        Ok(AiReplyToneV1::AiReplyToneFormal) => Ok("formal"),
        Ok(AiReplyToneV1::AiReplyToneConcise) => Ok("concise"),
        _ => Err(OllamaAiCoreErrorV1::InvalidRequest),
    }
}

fn language_name(value: i32) -> Result<&'static str, OllamaAiCoreErrorV1> {
    match AiReplyLanguageV1::try_from(value) {
        Ok(AiReplyLanguageV1::AiReplyLanguageAuto) => Ok("source language"),
        Ok(AiReplyLanguageV1::AiReplyLanguageEnglish) => Ok("English"),
        Ok(AiReplyLanguageV1::AiReplyLanguageSpanish) => Ok("Spanish"),
        Ok(AiReplyLanguageV1::AiReplyLanguageRussian) => Ok("Russian"),
        _ => Err(OllamaAiCoreErrorV1::InvalidRequest),
    }
}

fn subject_policy_name(value: i32) -> Result<&'static str, OllamaAiCoreErrorV1> {
    match AiReplySubjectPolicyV1::try_from(value) {
        Ok(AiReplySubjectPolicyV1::AiReplySubjectPolicyPreserve) => {
            Ok("preserve the source subject")
        }
        Ok(AiReplySubjectPolicyV1::AiReplySubjectPolicyGenerateIfMissing) => {
            Ok("generate only when missing")
        }
        Ok(AiReplySubjectPolicyV1::AiReplySubjectPolicyOmit) => Ok("return an empty subject"),
        _ => Err(OllamaAiCoreErrorV1::InvalidRequest),
    }
}

#[cfg(test)]
mod tests {
    use makosh_ai_contracts::{
        AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        wire::{AiEgressPolicyV1, AiReplyLanguageV1, AiReplySubjectPolicyV1, AiReplyToneV1},
    };

    use super::*;

    fn request() -> AiProviderReplyGenerationRequestV1 {
        AiProviderReplyGenerationRequestV1 {
            request_id: vec![1; 16],
            input_utf8: b"Sender: sender@example.test\nSubject: Hello\nBody:\nPrivate".to_vec(),
            tone: AiReplyToneV1::AiReplyToneWarm as i32,
            language: AiReplyLanguageV1::AiReplyLanguageEnglish as i32,
            subject_policy: AiReplySubjectPolicyV1::AiReplySubjectPolicyPreserve as i32,
            maximum_output_bytes: 4_096,
            maximum_output_tokens: 512,
            egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
            egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        }
    }

    fn settings() -> OllamaAiRuntimeSettingsV1 {
        OllamaAiRuntimeSettingsV1 {
            chat_model: "qwen3:4b".to_owned(),
            port: 11_434,
            timeout_millis: 30_000,
            settings_revision: 7,
        }
    }

    #[test]
    fn lifecycle_uses_exact_settings_and_structured_output() {
        let request = request();
        let accepted = accept_ollama_request_v1(&request, &settings()).expect("accepted");
        let (executing, plan) =
            begin_ollama_request_v1(&accepted, &request, &settings(), [9; 32]).expect("plan");
        assert!(
            std::str::from_utf8(&plan.prompt_utf8)
                .expect("prompt")
                .contains("No markdown")
        );
        let ready = complete_ollama_request_v1(
            &executing,
            &plan,
            OllamaHttpGenerationV1 {
                content_json_utf8: Zeroizing::new(
                    br#"{"subject":"Re: Hello","body":"Suggested reply","language":"english"}"#
                        .to_vec(),
                ),
                model_digest: [9; 32],
                input_tokens: 20,
                output_tokens: 8,
            },
        )
        .expect("ready");
        let result = ready.terminal_result.expect("result");
        assert_eq!(result.provider_settings_revision, 7);
        assert_eq!(result.confidence_basis_points, 0);
    }

    #[test]
    fn ambiguous_execution_becomes_terminal_uncertain_without_retry_plan() {
        let request = request();
        let accepted = accept_ollama_request_v1(&request, &settings()).expect("accepted");
        let (executing, _) =
            begin_ollama_request_v1(&accepted, &request, &settings(), [9; 32]).expect("plan");
        let uncertain = mark_ollama_uncertain_v1(&executing).expect("uncertain");
        assert_eq!(uncertain.state, OllamaAiRunStateV1::Uncertain);
        assert!(
            begin_ollama_request_v1(&uncertain, &request, &settings(), [9; 32]).is_err(),
            "uncertain requests must not be sent again automatically"
        );
    }

    #[test]
    fn provider_unavailable_is_a_typed_terminal_rejection() {
        let accepted = accept_ollama_request_v1(&request(), &settings()).expect("accepted request");
        let rejected = reject_ollama_request_v1(
            &accepted,
            AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable,
        )
        .expect("typed rejection");
        assert_eq!(rejected.state, OllamaAiRunStateV1::Rejected);
        assert_eq!(
            rejected.terminal_result.expect("result").terminal_status,
            AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable as i32
        );
    }

    #[test]
    fn provider_language_token_accepts_ascii_case_but_not_free_form() {
        assert_eq!(
            resolved_language(AiReplyLanguageV1::AiReplyLanguageEnglish as i32, "English"),
            Ok(AiReplyLanguageV1::AiReplyLanguageEnglish as i32)
        );
        assert_eq!(
            resolved_language(AiReplyLanguageV1::AiReplyLanguageEnglish as i32, " english"),
            Err(OllamaAiCoreErrorV1::InvalidProviderResponse)
        );
        assert_eq!(
            resolved_language(AiReplyLanguageV1::AiReplyLanguageEnglish as i32, "en"),
            Err(OllamaAiCoreErrorV1::InvalidProviderResponse)
        );
    }
}
