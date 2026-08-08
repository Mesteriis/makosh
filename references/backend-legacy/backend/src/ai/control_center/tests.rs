#[cfg(test)]
use serde_json::{Map, json};

use crate::ai::core::constants::AI_EMBEDDING_DIMENSION;

use super::errors::AiControlCenterError;
use super::models::AiProviderAuthStartRequest;
use super::presets::{capability_slots, provider_presets};
use super::provider_auth::local_provider_auth_callback_url;
use super::validation::{reject_secret_like_json, render_prompt, validate_cli_preset};

#[test]
fn secret_like_provider_payloads_are_rejected() {
    let payload = json!({
        "headers": {
            "authorization_token": "sk-test"
        }
    });

    let error = reject_secret_like_json(&payload).expect_err("secret-like keys must fail");

    assert!(matches!(error, AiControlCenterError::SecretLikePayload));
}

#[test]
fn cli_provider_presets_are_allowlisted() {
    assert!(validate_cli_preset("codex").is_ok());
    assert!(validate_cli_preset("claude").is_ok());
    assert!(validate_cli_preset("makosh").is_ok());

    let error = validate_cli_preset("bash -lc env").expect_err("shell-like presets must fail");

    assert!(matches!(error, AiControlCenterError::InvalidRequest(_)));
}

#[test]
fn provider_presets_include_remote_consent_targets() {
    let presets = provider_presets();

    assert!(presets.iter().any(|preset| preset.provider_key == "openai"));
    assert!(
        presets
            .iter()
            .any(|preset| preset.provider_key == "deepseek")
    );
    assert!(
        presets
            .iter()
            .any(|preset| preset.provider_key == "omniroute")
    );
    assert!(presets.iter().any(|preset| preset.provider_key == "raw"));
    assert!(
        presets
            .iter()
            .any(|preset| preset.provider_key == "openrouter")
    );
    assert!(presets.iter().any(|preset| preset.provider_key == "groq"));
    assert!(
        presets
            .iter()
            .any(|preset| preset.provider_key == "gemini-openai")
    );
    assert!(
        presets
            .iter()
            .any(|preset| preset.provider_key == "ollama" && preset.privacy == "local")
    );
}

#[test]
fn capability_slots_preserve_embedding_dimension_constraint() {
    let slots = capability_slots();
    let embeddings = slots
        .iter()
        .find(|slot| slot.slot == "embeddings")
        .expect("embeddings capability exists");

    assert_eq!(
        embeddings.requires_embedding_dimension,
        Some(AI_EMBEDDING_DIMENSION as i32)
    );
}

#[test]
fn local_provider_callback_url_carries_setup_state() {
    let url = local_provider_auth_callback_url(
        "http://127.0.0.1:8080/api/v1/ai/provider-auth/callback",
        "setup-1",
        "state-1",
    )
    .expect("local callback URL should be accepted");

    assert!(url.contains("setup_id=setup-1"));
    assert!(url.contains("state=state-1"));
}

#[test]
fn api_provider_auth_start_rejects_callback_flow() {
    let request = AiProviderAuthStartRequest {
        provider_kind: "api".to_owned(),
        provider_key: "openai".to_owned(),
        display_name: Some("OpenAI".to_owned()),
        callback_url: "http://127.0.0.1:8080/api/v1/ai/provider-auth/callback".to_owned(),
    };

    let error = request
        .validate()
        .expect_err("API provider auth must use token setup");

    assert!(matches!(error, AiControlCenterError::InvalidRequest(_)));
}

#[test]
fn prompt_rendering_never_needs_source_text_in_events() {
    let mut variables = Map::new();
    variables.insert("entity".to_owned(), json!("Communication"));
    variables.insert("summary".to_owned(), json!("Needs reply"));

    assert_eq!(
        render_prompt("Review {{entity}}: {{summary}}", &variables),
        "Review Communication: Needs reply"
    );
}
