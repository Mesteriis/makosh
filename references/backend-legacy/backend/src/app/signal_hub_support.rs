use crate::platform::settings::store::ApplicationSettingsStore;
use serde_json::{Map, Value, json};
use sqlx::postgres::PgPool;

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::signal_hub::connections::SignalHubConnectionService;
use crate::domains::signal_hub::health::SignalHubHealthService;
use crate::domains::signal_hub::store::{
    SignalHealth, SignalHealthCheckRequest, SignalHealthSnapshotWrite, SignalHubStore,
};
use crate::integrations::ai_runtime::AiRuntimeClient;
use crate::integrations::ollama::client::OllamaClient;
use crate::integrations::ollama::client::config::OllamaClientConfig;
use crate::integrations::omniroute::client::OmniRouteClient;
use crate::integrations::omniroute::client::config::OmniRouteClientConfig;
use crate::integrations::whatsapp::runtime::contracts::WhatsAppRuntimeStatus;
use crate::platform::config::ai::AiRuntimeProvider;
use crate::platform::config::app_config::AppConfig;
use crate::platform::settings::ai_runtime::AiRuntimeSettings;
use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_communications_api::accounts::ProviderAccount;
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;
use makosh_events_postgres::store::EventStore;

pub(crate) async fn provider_account_or_not_found(
    state: &AppState,
    account_id: &str,
) -> Result<ProviderAccount, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CommunicationProviderAccountStore::new(pool)
        .get(account_id)
        .await?
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn sync_provider_account_signal_connection(
    state: &AppState,
    account: &ProviderAccount,
    secret_ref: Option<&str>,
) -> Result<(), ApiError> {
    let status = provider_account_signal_status(account);
    sync_provider_account_signal_connection_with_status(state, account, status, secret_ref).await
}

pub(crate) async fn sync_provider_account_signal_connection_with_status(
    state: &AppState,
    account: &ProviderAccount,
    status: &str,
    secret_ref: Option<&str>,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let signal_store = SignalHubStore::new(pool.clone());
    let connection_service =
        SignalHubConnectionService::new(signal_store.clone(), EventStore::new(pool));
    signal_store.restore_system_sources().await?;
    let source_code = provider_signal_source_code(account.provider_kind);
    let settings = merged_provider_connection_settings(
        signal_store
            .find_connection_by_account(source_code, &account.account_id)
            .await?
            .as_ref()
            .map(|connection| &connection.settings),
        account,
    );
    connection_service
        .upsert_account_connection(
            source_code,
            &account.account_id,
            &account.display_name,
            status,
            settings,
            secret_ref.map(str::to_owned),
        )
        .await?;
    Ok(())
}

pub(crate) async fn remove_provider_account_signal_connection(
    state: &AppState,
    account: &ProviderAccount,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let source_code = provider_signal_source_code(account.provider_kind);
    SignalHubConnectionService::new(SignalHubStore::new(pool.clone()), EventStore::new(pool))
        .remove_account_connection(source_code, &account.account_id)
        .await?;
    Ok(())
}

pub(crate) async fn sync_whatsapp_runtime_signal_connection(
    state: &AppState,
    account: &ProviderAccount,
    status: &WhatsAppRuntimeStatus,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    sync_whatsapp_runtime_signal_connection_for_pool(
        &pool,
        account,
        status,
        status.session_secret_ref.clone(),
    )
    .await
    .map_err(ApiError::from)
}

pub(crate) async fn sync_whatsapp_runtime_signal_connection_for_pool(
    pool: &PgPool,
    account: &ProviderAccount,
    status: &WhatsAppRuntimeStatus,
    secret_ref: Option<String>,
) -> Result<(), crate::domains::signal_hub::store::SignalHubError> {
    let signal_store = SignalHubStore::new(pool.clone());
    let connection_service =
        SignalHubConnectionService::new(signal_store.clone(), EventStore::new(pool.clone()));
    signal_store.restore_system_sources().await?;
    let source_code = provider_signal_source_code(account.provider_kind);
    let settings = merged_whatsapp_runtime_connection_settings(
        signal_store
            .find_connection_by_account(source_code, &account.account_id)
            .await?
            .as_ref()
            .map(|connection| &connection.settings),
        account,
        status,
    );
    connection_service
        .upsert_account_connection(
            source_code,
            &account.account_id,
            &account.display_name,
            whatsapp_runtime_signal_status(status),
            settings,
            secret_ref,
        )
        .await?;
    Ok(())
}

pub(crate) async fn run_signal_hub_health_check(
    config: &AppConfig,
    pool: PgPool,
    request: &SignalHealthCheckRequest,
) -> Result<SignalHealth, crate::domains::signal_hub::store::SignalHubError> {
    let service = SignalHubHealthService::new(
        SignalHubStore::new(pool.clone()),
        EventStore::new(pool.clone()),
    );

    if request.source_code == "ai" && request.connection_id.is_none() {
        let runtime_state =
            crate::platform::events::runtime::source_runtime_state_from_policies(&pool, "ai")
                .await?;
        let snapshot = match runtime_state {
            "stopped" => SignalHealthSnapshotWrite {
                level: "disabled".to_owned(),
                summary: "AI source is disabled by Signal Hub policy".to_owned(),
                last_ok_at: None,
                last_failure_at: Some(chrono::Utc::now()),
                failure_count: 1,
                consecutive_failure_count: 1,
                next_retry_at: None,
                evidence: json!({
                    "source_code": "ai",
                    "runtime_state": runtime_state,
                    "health_origin": "signal_hub_policy"
                }),
            },
            "paused" | "muted" => SignalHealthSnapshotWrite {
                level: "degraded".to_owned(),
                summary: format!("AI source is {runtime_state} by Signal Hub policy"),
                last_ok_at: None,
                last_failure_at: Some(chrono::Utc::now()),
                failure_count: 1,
                consecutive_failure_count: 1,
                next_retry_at: None,
                evidence: json!({
                    "source_code": "ai",
                    "runtime_state": runtime_state,
                    "health_origin": "signal_hub_policy"
                }),
            },
            _ => ai_runtime_health_snapshot(config, &pool).await?,
        };

        return service.record_snapshot(request, snapshot).await;
    }

    service.run_health_check(request).await
}

fn provider_signal_source_code(provider_kind: CommunicationProviderKind) -> &'static str {
    match provider_kind {
        CommunicationProviderKind::Gmail
        | CommunicationProviderKind::Icloud
        | CommunicationProviderKind::Imap => "mail",
        CommunicationProviderKind::TelegramUser | CommunicationProviderKind::TelegramBot => {
            "telegram"
        }
        CommunicationProviderKind::WhatsappWeb
        | CommunicationProviderKind::WhatsappBusinessCloud => "whatsapp",
        CommunicationProviderKind::ZulipBot => "zulip",
        CommunicationProviderKind::ZoomUser | CommunicationProviderKind::ZoomServerToServer => {
            "zoom"
        }
        CommunicationProviderKind::YandexTelemostUser => "yandex_telemost",
    }
}

async fn ai_runtime_health_snapshot(
    config: &AppConfig,
    pool: &PgPool,
) -> Result<SignalHealthSnapshotWrite, crate::domains::signal_hub::store::SignalHubError> {
    let settings = ApplicationSettingsStore::new(pool.clone())
        .ai_runtime_settings(config)
        .await?;
    let runtime = ai_runtime_client_from_settings(config, &settings);
    let runtime_name = runtime
        .as_ref()
        .map(AiRuntimeClient::runtime_name)
        .unwrap_or(match settings.provider {
            AiRuntimeProvider::Ollama => "ollama",
            AiRuntimeProvider::OmniRoute => "omniroute",
        });

    let version = match runtime.as_ref() {
        Some(runtime) => runtime.version().await,
        None => Ok(None),
    };
    let models = match runtime.as_ref() {
        Some(runtime) => runtime.models().await,
        None => Ok(vec![]),
    };
    let chat_model_available = models
        .as_ref()
        .map(|items| items.iter().any(|item| item == &settings.chat_model))
        .unwrap_or(false);
    let embedding_model_available = models
        .as_ref()
        .map(|items| items.iter().any(|item| item == &settings.embedding_model))
        .unwrap_or(false);
    let healthy = version.is_ok()
        && models.is_ok()
        && chat_model_available
        && embedding_model_available
        && runtime.is_some();

    let runtime_error = runtime
        .is_none()
        .then_some("runtime client could not be initialized".to_owned())
        .or_else(|| version.as_ref().err().map(ToString::to_string))
        .or_else(|| models.as_ref().err().map(ToString::to_string));

    Ok(SignalHealthSnapshotWrite {
        level: if healthy {
            "healthy".to_owned()
        } else {
            "degraded".to_owned()
        },
        summary: if healthy {
            format!("AI runtime {runtime_name} is healthy")
        } else {
            format!("AI runtime {runtime_name} requires attention")
        },
        last_ok_at: healthy.then(chrono::Utc::now),
        last_failure_at: (!healthy).then(chrono::Utc::now),
        failure_count: if healthy { 0 } else { 1 },
        consecutive_failure_count: if healthy { 0 } else { 1 },
        next_retry_at: (!healthy).then(|| chrono::Utc::now() + chrono::Duration::minutes(5)),
        evidence: json!({
            "source_code": "ai",
            "health_origin": "ai_runtime_status",
            "runtime": runtime_name,
            "provider": settings.provider.as_str(),
            "base_url": settings.base_url,
            "version": version.ok().flatten(),
            "chat_model": settings.chat_model,
            "embedding_model": settings.embedding_model,
            "chat_model_available": chat_model_available,
            "embedding_model_available": embedding_model_available,
            "runtime_error": runtime_error,
        }),
    })
}

fn ai_runtime_client_from_settings(
    config: &AppConfig,
    settings: &AiRuntimeSettings,
) -> Option<AiRuntimeClient> {
    match settings.provider {
        AiRuntimeProvider::Ollama => OllamaClient::new(
            OllamaClientConfig::new(
                &settings.base_url,
                &settings.chat_model,
                &settings.embedding_model,
            )
            .with_timeout_seconds(settings.timeout_seconds),
        )
        .ok()
        .map(AiRuntimeClient::Ollama),
        AiRuntimeProvider::OmniRoute => config.omniroute_api_key().cloned().and_then(|api_key| {
            OmniRouteClient::new(
                OmniRouteClientConfig::new(
                    &settings.base_url,
                    &settings.chat_model,
                    &settings.embedding_model,
                    api_key,
                )
                .with_timeout_seconds(settings.timeout_seconds),
            )
            .ok()
            .map(AiRuntimeClient::OmniRoute)
        }),
    }
}

fn provider_account_signal_status(account: &ProviderAccount) -> &'static str {
    match account.provider_kind {
        CommunicationProviderKind::Gmail
        | CommunicationProviderKind::Icloud
        | CommunicationProviderKind::Imap => {
            if account.config.get("auth_state").and_then(Value::as_str) == Some("logged_out") {
                "disconnected"
            } else {
                "connected"
            }
        }
        CommunicationProviderKind::TelegramUser | CommunicationProviderKind::TelegramBot => {
            match account
                .config
                .get("lifecycle_state")
                .and_then(Value::as_str)
            {
                Some("removed") => "removed",
                Some("logged_out") => "disconnected",
                _ => match account.config.get("runtime").and_then(Value::as_str) {
                    Some("live_blocked") => "awaiting_user_action",
                    Some("fixture") | Some("tdlib_qr_authorized") => "connected",
                    _ => "connected",
                },
            }
        }
        CommunicationProviderKind::WhatsappWeb
        | CommunicationProviderKind::WhatsappBusinessCloud => {
            match account.config.get("runtime").and_then(Value::as_str) {
                Some("blocked") | Some("live_blocked") => "awaiting_user_action",
                Some("manual_webview") => "connecting",
                _ => "connected",
            }
        }
        CommunicationProviderKind::ZulipBot => match account
            .config
            .get("lifecycle_state")
            .and_then(Value::as_str)
        {
            Some("removed") => "removed",
            Some("blocked") => "awaiting_user_action",
            _ => "connected",
        },
        CommunicationProviderKind::ZoomUser | CommunicationProviderKind::ZoomServerToServer => {
            match account
                .config
                .get("lifecycle_state")
                .and_then(Value::as_str)
            {
                Some("removed") => "removed",
                Some("blocked") => "awaiting_user_action",
                _ => "connected",
            }
        }
        CommunicationProviderKind::YandexTelemostUser => match account
            .config
            .get("lifecycle_state")
            .and_then(Value::as_str)
        {
            Some("removed") => "removed",
            Some("blocked") => "awaiting_user_action",
            _ => "connected",
        },
    }
}

fn whatsapp_runtime_signal_status(status: &WhatsAppRuntimeStatus) -> &'static str {
    match status.status.as_str() {
        "removed" => "removed",
        "revoked" | "link_required" | "created" | "blocked" => "awaiting_user_action",
        "qr_pending" | "pair_code_pending" | "syncing" | "degraded" => "connecting",
        "available" | "linked" => "connected",
        _ => {
            if status.session_restore_available {
                "connected"
            } else {
                "awaiting_user_action"
            }
        }
    }
}

fn merged_provider_connection_settings(
    current: Option<&Value>,
    account: &ProviderAccount,
) -> Value {
    let mut settings = current
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_else(Map::new);
    settings.insert("account_id".to_owned(), json!(account.account_id));
    settings.insert(
        "provider_kind".to_owned(),
        json!(account.provider_kind.as_str()),
    );
    settings.insert(
        "external_account_id".to_owned(),
        json!(account.external_account_id),
    );
    copy_config_field(&mut settings, &account.config, "runtime");
    copy_config_field(&mut settings, &account.config, "lifecycle_state");
    copy_config_field(&mut settings, &account.config, "auth_state");
    Value::Object(settings)
}

fn merged_whatsapp_runtime_connection_settings(
    current: Option<&Value>,
    account: &ProviderAccount,
    status: &WhatsAppRuntimeStatus,
) -> Value {
    let mut settings = merged_provider_connection_settings(current, account)
        .as_object()
        .cloned()
        .unwrap_or_default();
    settings.insert(
        "whatsapp_runtime_status".to_owned(),
        json!(status.status.as_str()),
    );
    settings.insert(
        "whatsapp_provider_shape".to_owned(),
        json!(status.provider_shape.as_str()),
    );
    settings.insert(
        "whatsapp_runtime_kind".to_owned(),
        json!(status.runtime_kind.as_str()),
    );
    settings.insert(
        "whatsapp_session_restore_available".to_owned(),
        json!(status.session_restore_available),
    );
    settings.insert(
        "whatsapp_live_runtime_available".to_owned(),
        json!(status.live_runtime_available),
    );
    settings.insert(
        "whatsapp_runtime_blockers".to_owned(),
        json!(status.runtime_blockers),
    );
    settings.insert("whatsapp_last_error".to_owned(), json!(status.last_error));
    Value::Object(settings)
}

fn copy_config_field(settings: &mut Map<String, Value>, config: &Value, field: &str) {
    if let Some(value) = config.get(field) {
        settings.insert(field.to_owned(), value.clone());
    }
}
