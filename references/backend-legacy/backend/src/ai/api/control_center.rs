use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::ai::control_center::errors::AiControlCenterError;
use crate::ai::control_center::models::{
    AiHubUsageStatsResponse, AiModelAvailabilityUpdateRequest, AiModelCatalogItem,
    AiModelDownloadRequest, AiModelRoute, AiModelRouteUpdateRequest, AiPromptActivateRequest,
    AiPromptCreateRequest, AiPromptEvalRun, AiPromptTemplate, AiPromptTestRequest, AiPromptVersion,
    AiPromptVersionCreateRequest, AiProviderAccount, AiProviderAuthPendingGrant,
    AiProviderAuthStartRequest, AiProviderAuthStartResponse, AiProviderAuthStatusResponse,
    AiProviderCommandKind, AiProviderCommandResponse, AiProviderConsentRequest,
    AiProviderCreateRequest, AiProviderPatchRequest, AiSettingsOverviewResponse,
};
use crate::ai::control_center::provider_auth::{
    connect_pending_ai_provider_auth, start_local_provider_auth,
};
use crate::ai::control_center::store::AiControlCenterStore;
use crate::ai::control_center::vault::store_api_key_in_host_vault;
use crate::app::api_support::formatting::html_escape;
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::integrations::ollama::client::OllamaClient;
use crate::integrations::ollama::client::config::OllamaClientConfig;
use crate::vault::errors::HostVaultError;
use crate::vault::models::VaultMode;
use makosh_events_postgres::store::EventStore;

use super::helpers::{ai_control_center_store, request_actor_id};
use super::models::{AiModelListResponse, AiPromptListResponse, AiProviderListResponse};

pub(crate) async fn get_ai_settings_overview(
    State(state): State<AppState>,
) -> Result<Json<AiSettingsOverviewResponse>, ApiError> {
    Ok(Json(ai_control_center_store(&state)?.overview().await?))
}

#[derive(Debug, Deserialize)]
pub(crate) struct AiUsageStatsQuery {
    window_hours: Option<i32>,
}

pub(crate) async fn get_ai_usage_stats(
    State(state): State<AppState>,
    Query(query): Query<AiUsageStatsQuery>,
) -> Result<Json<AiHubUsageStatsResponse>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .usage_stats(query.window_hours)
            .await?,
    ))
}

pub(crate) async fn get_ai_providers(
    State(state): State<AppState>,
) -> Result<Json<AiProviderListResponse>, ApiError> {
    Ok(Json(AiProviderListResponse {
        items: ai_control_center_store(&state)?.list_providers().await?,
    }))
}

pub(crate) async fn post_ai_provider(
    State(state): State<AppState>,
    Json(request): Json<AiProviderCreateRequest>,
) -> Result<Json<AiProviderAccount>, ApiError> {
    let store = ai_control_center_store(&state)?;
    let api_key = request_api_key(&request.api_key);
    if api_key.is_some() {
        ensure_api_key_provider_kind(&request.provider_kind)?;
        ensure_host_vault_unlocked_for_api_key(&state)?;
    }
    let provider = store.create_provider(&request).await?;
    if let Some(api_key) = api_key {
        let Some(pool) = state.database.pool() else {
            return Err(ApiError::DatabaseNotConfigured);
        };
        store_api_key_in_host_vault(pool, &state.vault, &provider.provider_id, &api_key).await?;
        if request.enabled.unwrap_or(true) {
            activate_api_provider_after_secret(&store, &provider.provider_id).await?;
        }
        let Some(provider) = store.provider(&provider.provider_id).await? else {
            return Err(ApiError::NotFound);
        };
        return Ok(Json(provider));
    }

    Ok(Json(provider))
}

pub(crate) async fn patch_ai_provider(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
    Json(request): Json<AiProviderPatchRequest>,
) -> Result<Json<AiProviderAccount>, ApiError> {
    let store = ai_control_center_store(&state)?;
    let api_key = request_api_key(&request.api_key);
    if api_key.is_some() {
        let current = store
            .provider(&provider_id)
            .await?
            .ok_or(AiControlCenterError::ProviderNotFound)?;
        ensure_api_key_provider_kind(&current.provider_kind)?;
        ensure_host_vault_unlocked_for_api_key(&state)?;
    }
    let provider = store.update_provider(&provider_id, &request).await?;
    if let Some(api_key) = api_key {
        let Some(pool) = state.database.pool() else {
            return Err(ApiError::DatabaseNotConfigured);
        };
        store_api_key_in_host_vault(pool, &state.vault, &provider.provider_id, &api_key).await?;
        if request.enabled != Some(false) && provider.status == "needs_setup" {
            activate_api_provider_after_secret(&store, &provider.provider_id).await?;
        }
        let Some(provider) = store.provider(&provider.provider_id).await? else {
            return Err(ApiError::NotFound);
        };
        return Ok(Json(provider));
    }

    Ok(Json(provider))
}

pub(crate) async fn post_ai_provider_test(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
) -> Result<Json<AiProviderCommandResponse>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .provider_command(&provider_id, AiProviderCommandKind::Test)
            .await?,
    ))
}

pub(crate) async fn post_ai_provider_sync_models(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(provider_id): Path<String>,
) -> Result<Json<AiProviderCommandResponse>, ApiError> {
    let store = ai_control_center_store(&state)?;
    let provider = store
        .provider(&provider_id)
        .await?
        .ok_or(AiControlCenterError::ProviderNotFound)?;
    if provider.provider_kind == "api" {
        ensure_host_vault_unlocked_for_api_key(&state)?;
        let secret_ref = store
            .api_key_secret_ref(&provider.provider_id)
            .await?
            .ok_or_else(|| {
                AiControlCenterError::InvalidRequest(
                    "API provider requires a host-vault API key before model sync".to_owned(),
                )
            })?;
        let api_key = state.vault.read_secret(&secret_ref)?;
        let synced = store
            .sync_openai_compatible_provider_models(
                &provider,
                &api_key,
                &request_actor_id(&headers),
            )
            .await?;
        return Ok(Json(AiProviderCommandResponse {
            provider_id: provider.provider_id,
            command: "sync_models".to_owned(),
            status: "synced".to_owned(),
            message: format!("Synchronized {synced} provider models"),
        }));
    }
    if provider.provider_kind == "built_in" && provider.provider_key == "ollama" {
        let synced = store
            .sync_ollama_provider_models(&provider, &request_actor_id(&headers))
            .await?;
        return Ok(Json(AiProviderCommandResponse {
            provider_id: provider.provider_id,
            command: "sync_models".to_owned(),
            status: "synced".to_owned(),
            message: format!("Synchronized {synced} Ollama models"),
        }));
    }
    if provider.provider_kind == "cli" {
        let synced = store
            .sync_cli_provider_models(&provider, &request_actor_id(&headers))
            .await?;
        return Ok(Json(AiProviderCommandResponse {
            provider_id: provider.provider_id,
            command: "sync_models".to_owned(),
            status: "synced".to_owned(),
            message: format!("Synchronized {synced} CLI settings models"),
        }));
    }

    Ok(Json(
        store
            .provider_command(&provider_id, AiProviderCommandKind::SyncModels)
            .await?,
    ))
}

pub(crate) async fn post_ai_provider_consent(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
    Json(request): Json<AiProviderConsentRequest>,
) -> Result<Json<AiProviderAccount>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .record_consent(&provider_id, &request)
            .await?,
    ))
}

pub(crate) async fn post_ai_provider_auth_start(
    State(state): State<AppState>,
    Json(request): Json<AiProviderAuthStartRequest>,
) -> Result<Json<AiProviderAuthStartResponse>, ApiError> {
    let mut pending = start_local_provider_auth(&request).await?;
    let store = ai_control_center_store(&state)?;
    let provider = if pending.status == "ready" {
        connect_pending_ai_provider_auth(&store, &mut pending).await?
    } else {
        None
    };
    upsert_pending_ai_provider_auth(&state, pending.clone())?;
    Ok(Json(pending.response(provider)))
}

pub(crate) async fn get_ai_provider_auth_status(
    State(state): State<AppState>,
    Path(setup_id): Path<String>,
) -> Result<Json<AiProviderAuthStatusResponse>, ApiError> {
    let mut pending = pending_ai_provider_auth(&state, &setup_id)?;
    let provider =
        connect_pending_ai_provider_auth(&ai_control_center_store(&state)?, &mut pending).await?;
    upsert_pending_ai_provider_auth(&state, pending.clone())?;
    Ok(Json(pending.status_response(provider)))
}

#[derive(Debug, Deserialize)]
pub(crate) struct AiProviderAuthCallbackQuery {
    setup_id: Option<String>,
    state: Option<String>,
}

pub(crate) async fn get_ai_provider_auth_callback(
    State(state): State<AppState>,
    Query(query): Query<AiProviderAuthCallbackQuery>,
) -> (StatusCode, Html<String>) {
    let Some(setup_id) = trimmed_query_value(query.setup_id) else {
        return ai_provider_auth_callback_error_page(
            StatusCode::BAD_REQUEST,
            "Missing AI provider setup id. Start the provider connection again.",
        );
    };
    let Some(callback_state) = trimmed_query_value(query.state) else {
        return ai_provider_auth_callback_error_page(
            StatusCode::BAD_REQUEST,
            "Missing AI provider callback state. Start the provider connection again.",
        );
    };
    let mut pending = match pending_ai_provider_auth(&state, &setup_id) {
        Ok(pending) => pending,
        Err(_) => {
            return ai_provider_auth_callback_error_page(
                StatusCode::BAD_REQUEST,
                "AI provider callback expired or was already removed. Start again.",
            );
        }
    };
    if pending.state != callback_state {
        return ai_provider_auth_callback_error_page(
            StatusCode::BAD_REQUEST,
            "AI provider callback state does not match the pending setup.",
        );
    }

    let store = match ai_control_center_store(&state) {
        Ok(store) => store,
        Err(_) => {
            tracing::error!("AI provider callback store unavailable");
            return ai_provider_auth_callback_error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "AI provider setup is unavailable. Check local backend status.",
            );
        }
    };

    match connect_pending_ai_provider_auth(&store, &mut pending).await {
        Ok(Some(provider)) => {
            if upsert_pending_ai_provider_auth(&state, pending.clone()).is_err() {
                tracing::error!("AI provider callback state update failed");
            }
            ai_provider_auth_callback_success_page(&provider)
        }
        Ok(None) => {
            if upsert_pending_ai_provider_auth(&state, pending.clone()).is_err() {
                tracing::error!("AI provider callback state update failed");
            }
            ai_provider_auth_callback_waiting_page(&pending)
        }
        Err(error) => {
            tracing::error!(error = %error, "AI provider callback completion failed");
            ai_provider_auth_callback_error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "AI provider connection failed. Check local backend status.",
            )
        }
    }
}

pub(crate) async fn get_ai_models(
    State(state): State<AppState>,
) -> Result<Json<AiModelListResponse>, ApiError> {
    Ok(Json(AiModelListResponse {
        items: ai_control_center_store(&state)?.list_models().await?,
    }))
}

pub(crate) async fn patch_ai_model_availability(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AiModelAvailabilityUpdateRequest>,
) -> Result<Json<AiModelCatalogItem>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .update_model_availability(&request, &request_actor_id(&headers))
            .await?,
    ))
}

pub(crate) async fn post_ai_model_download(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AiModelDownloadRequest>,
) -> Result<Json<AiModelCatalogItem>, ApiError> {
    let store = ai_control_center_store(&state)?;
    let provider = store
        .provider(&request.provider_id)
        .await?
        .ok_or(AiControlCenterError::ProviderNotFound)?;
    let existing = store
        .model(&request.provider_id, &request.model_key)
        .await?
        .ok_or(AiControlCenterError::ModelNotFound)?;
    if existing.is_available {
        return Ok(Json(existing));
    }
    if !(provider.provider_kind == "built_in" && provider.provider_key == "ollama") {
        return Err(ApiError::from(AiControlCenterError::InvalidRequest(
            "explicit model download is supported only for built-in Ollama models".to_owned(),
        )));
    }

    let actor_id = request_actor_id(&headers);
    let request_id = format!(
        "ai-model-download-{}-{}-{}",
        request.provider_id.as_str(),
        request.model_key.as_str(),
        Uuid::new_v4()
    );
    let requested_event_id = append_ai_model_download_event(
        &state,
        &request_id,
        "ai.hub.model_download.requested",
        None,
        &actor_id,
        &request,
    )
    .await?;

    let result: Result<AiModelCatalogItem, ApiError> = async {
        ollama_download_client(&provider)?
            .pull_model(&request.model_key)
            .await?;

        store
            .update_model_availability(
                &AiModelAvailabilityUpdateRequest {
                    provider_id: request.provider_id.clone(),
                    model_key: request.model_key.clone(),
                    is_available: true,
                },
                &actor_id,
            )
            .await
            .map_err(ApiError::from)
    }
    .await;

    match result {
        Ok(model) => {
            append_ai_model_download_event(
                &state,
                &request_id,
                "ai.hub.model_download.completed",
                Some(&requested_event_id),
                &actor_id,
                &request,
            )
            .await?;
            Ok(Json(model))
        }
        Err(error) => {
            let _ = append_ai_model_download_event(
                &state,
                &request_id,
                "ai.hub.model_download.failed",
                Some(&requested_event_id),
                &actor_id,
                &request,
            )
            .await;
            Err(error)
        }
    }
}

pub(crate) async fn put_ai_model_route(
    State(state): State<AppState>,
    Path(slot): Path<String>,
    Json(request): Json<AiModelRouteUpdateRequest>,
) -> Result<Json<AiModelRoute>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .put_model_route(&slot, &request)
            .await?,
    ))
}

pub(crate) async fn delete_ai_model_route(
    State(state): State<AppState>,
    Path(slot): Path<String>,
) -> Result<StatusCode, ApiError> {
    ai_control_center_store(&state)?
        .delete_model_route(&slot)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn get_ai_prompts(
    State(state): State<AppState>,
) -> Result<Json<AiPromptListResponse>, ApiError> {
    Ok(Json(AiPromptListResponse {
        items: ai_control_center_store(&state)?.list_prompts().await?,
    }))
}

pub(crate) async fn post_ai_prompt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AiPromptCreateRequest>,
) -> Result<Json<AiPromptTemplate>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .create_prompt(&request, &request_actor_id(&headers))
            .await?,
    ))
}

pub(crate) async fn post_ai_prompt_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(prompt_id): Path<String>,
    Json(request): Json<AiPromptVersionCreateRequest>,
) -> Result<Json<AiPromptVersion>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .create_prompt_version(&prompt_id, &request, &request_actor_id(&headers))
            .await?,
    ))
}

pub(crate) async fn post_ai_prompt_activate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(prompt_id): Path<String>,
    Json(request): Json<AiPromptActivateRequest>,
) -> Result<Json<AiPromptTemplate>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .activate_prompt_version(&prompt_id, &request, &request_actor_id(&headers))
            .await?,
    ))
}

pub(crate) async fn post_ai_prompt_test(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(prompt_id): Path<String>,
    Json(request): Json<AiPromptTestRequest>,
) -> Result<Json<AiPromptEvalRun>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .test_prompt(&prompt_id, &request, &request_actor_id(&headers))
            .await?,
    ))
}

fn request_api_key(api_key: &Option<String>) -> Option<String> {
    api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn ollama_download_client(provider: &AiProviderAccount) -> Result<OllamaClient, ApiError> {
    let base_url = provider
        .config
        .get("base_url")
        .and_then(|value| value.as_str())
        .unwrap_or("http://127.0.0.1:11434");

    Ok(OllamaClient::new(OllamaClientConfig::new(
        base_url,
        "__download__",
        "__download__",
    ))?)
}

fn ensure_api_key_provider_kind(provider_kind: &str) -> Result<(), ApiError> {
    if provider_kind.trim() == "api" {
        return Ok(());
    }
    Err(AiControlCenterError::InvalidRequest(
        "API keys can only be configured for API providers".to_owned(),
    )
    .into())
}

fn ensure_host_vault_unlocked_for_api_key(state: &AppState) -> Result<(), ApiError> {
    match state.vault.status()?.state {
        VaultMode::Unlocked => Ok(()),
        VaultMode::Locked => Err(HostVaultError::Locked.into()),
        VaultMode::Uninitialized => Err(HostVaultError::Uninitialized.into()),
    }
}

async fn activate_api_provider_after_secret(
    store: &AiControlCenterStore,
    provider_id: &str,
) -> Result<(), ApiError> {
    store
        .update_provider(
            provider_id,
            &AiProviderPatchRequest {
                display_name: None,
                base_url: None,
                config: None,
                enabled: Some(true),
                api_key: None,
            },
        )
        .await?;
    Ok(())
}

fn pending_ai_provider_auth(
    state: &AppState,
    setup_id: &str,
) -> Result<AiProviderAuthPendingGrant, ApiError> {
    state
        .account_setup
        .pending_ai_provider_auth
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?
        .get(setup_id.trim())
        .cloned()
        .ok_or_else(|| {
            AiControlCenterError::InvalidRequest(
                "AI provider authorization setup was not found".to_owned(),
            )
            .into()
        })
}

fn upsert_pending_ai_provider_auth(
    state: &AppState,
    pending: AiProviderAuthPendingGrant,
) -> Result<(), ApiError> {
    state
        .account_setup
        .pending_ai_provider_auth
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?
        .insert(pending.setup_id.clone(), pending);
    Ok(())
}

fn trimmed_query_value(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

async fn append_ai_model_download_event(
    state: &AppState,
    request_id: &str,
    event_type: &str,
    causation_id: Option<&str>,
    actor_id: &str,
    request: &AiModelDownloadRequest,
) -> Result<String, ApiError> {
    let event_id = format!("{request_id}:{event_type}");
    let mut builder = NewEventEnvelope::builder(
        event_id.clone(),
        event_type,
        Utc::now(),
        json!({
            "kind": "ai_provider",
            "source_id": request.provider_id.as_str(),
        }),
        json!({
            "kind": "ai_model_download",
            "provider_id": request.provider_id.as_str(),
            "model_key": request.model_key.as_str(),
        }),
    )
    .actor(json!({ "actor_id": actor_id }))
    .payload(json!({
        "provider_id": request.provider_id.as_str(),
        "model_key": request.model_key.as_str(),
    }))
    .provenance(json!({
        "source": "ai_control_center",
        "event_type": event_type,
    }))
    .correlation_id(request_id);
    if let Some(causation_id) = causation_id {
        builder = builder.causation_id(causation_id);
    }

    let event = builder.build()?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    EventStore::new(pool).append(&event).await?;
    Ok(event_id)
}

fn ai_provider_auth_callback_success_page(
    provider: &AiProviderAccount,
) -> (StatusCode, Html<String>) {
    let display_name = html_escape(&provider.display_name);
    let provider_id = html_escape(&provider.provider_id);
    (
        StatusCode::OK,
        Html(format!(
            r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Макошь AI provider</title>
  <style>
    body {{ margin: 0; font-family: system-ui, sans-serif; color: #182033; background: #f5f6f8; }}
    main {{ max-width: 720px; margin: 48px auto; background: #fff; border: 1px solid #d9dee7; border-radius: 8px; padding: 24px; }}
    code {{ display: block; overflow-wrap: anywhere; background: #f8fafc; border: 1px solid #d9dee7; border-radius: 6px; padding: 10px; }}
  </style>
  <script>
    window.setTimeout(function () {{
      try {{
        if (window.opener && !window.opener.closed) {{
          window.opener.postMessage({{ type: 'makosh:ai-provider-connected', providerId: '{provider_id}' }}, '*');
        }}
      }} catch (_error) {{}}
      try {{ window.close(); }} catch (_error) {{}}
    }}, 350);
  </script>
</head>
<body>
  <main>
    <h1>AI provider connected</h1>
    <p>Макошь connected {display_name} through the local callback flow.</p>
    <p>Provider</p>
    <code>{provider_id}</code>
    <p>This tab will close automatically. Return to Макошь settings if it stays open.</p>
  </main>
</body>
</html>"#
        )),
    )
}

fn ai_provider_auth_callback_waiting_page(
    pending: &AiProviderAuthPendingGrant,
) -> (StatusCode, Html<String>) {
    let message = html_escape(&pending.message);
    let command = pending
        .login_command
        .as_deref()
        .map(html_escape)
        .unwrap_or_else(|| "No login command required.".to_owned());
    (
        StatusCode::ACCEPTED,
        Html(format!(
            r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Макошь AI provider</title>
  <style>
    body {{ margin: 0; font-family: system-ui, sans-serif; color: #182033; background: #f5f6f8; }}
    main {{ max-width: 720px; margin: 48px auto; background: #fff; border: 1px solid #d9dee7; border-radius: 8px; padding: 24px; }}
    code {{ display: block; overflow-wrap: anywhere; background: #f8fafc; border: 1px solid #d9dee7; border-radius: 6px; padding: 10px; }}
  </style>
</head>
<body>
  <main>
    <h1>AI provider authorization required</h1>
    <p>{message}</p>
    <p>Command</p>
    <code>{command}</code>
    <p>After the CLI is signed in, reopen the Макошь callback link from Settings.</p>
  </main>
</body>
</html>"#
        )),
    )
}

fn ai_provider_auth_callback_error_page(
    status: StatusCode,
    message: &str,
) -> (StatusCode, Html<String>) {
    let message = html_escape(message);
    (
        status,
        Html(format!(
            r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Макошь AI provider</title>
  <style>
    body {{ margin: 0; font-family: system-ui, sans-serif; color: #182033; background: #f5f6f8; }}
    main {{ max-width: 720px; margin: 48px auto; background: #fff; border: 1px solid #d9dee7; border-radius: 8px; padding: 24px; }}
  </style>
</head>
<body>
  <main>
    <h1>AI provider connection failed</h1>
    <p>{message}</p>
    <p>Return to Макошь and start the AI provider connection again.</p>
  </main>
</body>
</html>"#
        )),
    )
}
