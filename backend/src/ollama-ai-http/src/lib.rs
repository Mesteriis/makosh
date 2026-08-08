#![forbid(unsafe_code)]

mod model;
mod wire;

pub use model::{OllamaAiHttpErrorV1, OllamaModelRevisionV1};

use std::time::Duration;

use makosh_ollama_ai_api::{OLLAMA_AI_LOOPBACK_HOST_V1, OllamaAiRuntimeSettingsV1};
use makosh_ollama_ai_core::{
    OllamaExplanationPlanV1, OllamaGenerationPlanV1, OllamaHttpGenerationV1,
    OllamaSummaryGenerationPlanV1, OllamaTranslationPlanV1,
};

use crate::{
    model::{
        decode_chat_response_v1, decode_explanation_chat_response_v1, decode_model_revision_v1,
        decode_summary_chat_response_v1, decode_translation_chat_response_v1,
        encode_chat_request_v1, encode_explanation_chat_request_v1, encode_summary_chat_request_v1,
        encode_translation_chat_request_v1,
    },
    wire::execute_json_v1,
};

pub const PACKAGE: &str = "makosh-ollama-ai-http";

pub async fn discover_model_revision_v1(
    settings: &OllamaAiRuntimeSettingsV1,
) -> Result<OllamaModelRevisionV1, OllamaAiHttpErrorV1> {
    validate_transport_settings_v1(settings)?;
    let body = execute_json_v1(
        settings.port,
        "GET",
        "/api/tags",
        &[],
        Duration::from_millis(settings.timeout_millis),
    )
    .await?;
    decode_model_revision_v1(&body, &settings.chat_model)
}

pub async fn generate_reply_v1(
    settings: &OllamaAiRuntimeSettingsV1,
    plan: &OllamaGenerationPlanV1,
) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1> {
    validate_transport_settings_v1(settings)?;
    if settings.settings_revision != plan.settings_revision
        || settings.chat_model != plan.model
        || settings.timeout_millis != plan.timeout_millis
    {
        return Err(OllamaAiHttpErrorV1::InvalidConfiguration);
    }
    let body = encode_chat_request_v1(plan)?;
    let response = execute_json_v1(
        settings.port,
        "POST",
        "/api/chat",
        &body,
        Duration::from_millis(settings.timeout_millis),
    )
    .await?;
    decode_chat_response_v1(&response, plan)
}

pub async fn generate_summary_v1(
    settings: &OllamaAiRuntimeSettingsV1,
    plan: &OllamaSummaryGenerationPlanV1,
) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1> {
    validate_transport_settings_v1(settings)?;
    if settings.settings_revision != plan.settings_revision
        || settings.chat_model != plan.model
        || settings.timeout_millis != plan.timeout_millis
    {
        return Err(OllamaAiHttpErrorV1::InvalidConfiguration);
    }
    let body = encode_summary_chat_request_v1(plan)?;
    let response = execute_json_v1(
        settings.port,
        "POST",
        "/api/chat",
        &body,
        Duration::from_millis(settings.timeout_millis),
    )
    .await?;
    decode_summary_chat_response_v1(&response, plan)
}

pub async fn generate_translation_v1(
    settings: &OllamaAiRuntimeSettingsV1,
    plan: &OllamaTranslationPlanV1,
) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1> {
    validate_transport_settings_v1(settings)?;
    if settings.settings_revision != plan.settings_revision
        || settings.chat_model != plan.model
        || settings.timeout_millis != plan.timeout_millis
    {
        return Err(OllamaAiHttpErrorV1::InvalidConfiguration);
    }
    let body = encode_translation_chat_request_v1(plan)?;
    let response = execute_json_v1(
        settings.port,
        "POST",
        "/api/chat",
        &body,
        Duration::from_millis(settings.timeout_millis),
    )
    .await?;
    decode_translation_chat_response_v1(&response, plan)
}

pub async fn generate_explanation_v1(
    settings: &OllamaAiRuntimeSettingsV1,
    plan: &OllamaExplanationPlanV1,
) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1> {
    validate_transport_settings_v1(settings)?;
    if settings.settings_revision != plan.settings_revision
        || settings.chat_model != plan.model
        || settings.timeout_millis != plan.timeout_millis
    {
        return Err(OllamaAiHttpErrorV1::InvalidConfiguration);
    }
    let body = encode_explanation_chat_request_v1(plan)?;
    let response = execute_json_v1(
        settings.port,
        "POST",
        "/api/chat",
        &body,
        Duration::from_millis(settings.timeout_millis),
    )
    .await?;
    decode_explanation_chat_response_v1(&response, plan)
}

fn validate_transport_settings_v1(
    settings: &OllamaAiRuntimeSettingsV1,
) -> Result<(), OllamaAiHttpErrorV1> {
    if settings.port == 0
        || settings.timeout_millis == 0
        || settings.timeout_millis > 30_000
        || OLLAMA_AI_LOOPBACK_HOST_V1 != "127.0.0.1"
    {
        return Err(OllamaAiHttpErrorV1::InvalidConfiguration);
    }
    Ok(())
}
