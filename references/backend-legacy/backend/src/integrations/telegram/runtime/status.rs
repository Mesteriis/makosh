use chrono::Utc;
use makosh_communications_api::accounts::ProviderAccountLookupPort;
use makosh_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::tdjson::client::TdJsonLibrary;

use crate::platform::config::app_config::AppConfig;

use super::models::TelegramRuntimeStatus;
use super::state::{TelegramRuntimeActorState, TelegramRuntimeState};
use super::validation::validate_non_empty;

pub(super) async fn load_telegram_account(
    provider_account_store: &dyn ProviderAccountLookupPort,
    account_id: &str,
) -> Result<ProviderAccount, TelegramError> {
    let account_id = validate_non_empty("account_id", account_id)?;
    let account = provider_account_store
        .get(&account_id)
        .await
        .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "Telegram account `{account_id}` is not configured"
            ))
        })?;

    if !account.provider_kind.is_telegram() {
        return Err(TelegramError::InvalidRequest(format!(
            "account `{}` is not a Telegram provider account",
            account.account_id
        )));
    }

    Ok(account)
}

pub(super) fn status_from_account(
    config: &AppConfig,
    account: &ProviderAccount,
    actor_state: Option<TelegramRuntimeActorState>,
) -> TelegramRuntimeStatus {
    let runtime_kind = account_runtime_kind(account);
    let default_state = default_state_for_runtime(&runtime_kind);
    let actor_state = actor_state.unwrap_or(default_state);
    let tdjson_path = config.tdjson_path().map(|path| path.display().to_string());
    let (tdjson_runtime_available, tdjson_probe_error) = tdjson_probe(config.tdjson_path());
    let telegram_api_id_configured = config.telegram_api_id().is_some();
    let telegram_api_hash_configured = config.telegram_api_hash().is_some();
    let telegram_app_credentials_configured =
        telegram_api_id_configured && telegram_api_hash_configured;
    let live_send_available = runtime_kind == "tdlib_qr_authorized"
        && actor_state.status == TelegramRuntimeState::Running
        && tdjson_runtime_available
        && telegram_app_credentials_configured;
    let runtime_blockers = runtime_blockers(
        &runtime_kind,
        tdjson_runtime_available,
        telegram_api_id_configured,
        telegram_api_hash_configured,
        actor_state.last_error.as_deref(),
    );

    TelegramRuntimeStatus {
        account_id: account.account_id.clone(),
        provider_kind: account.provider_kind.as_str().to_owned(),
        runtime_kind: runtime_kind.clone(),
        status: actor_state.status.as_str().to_owned(),
        fixture_runtime: runtime_kind == "fixture",
        tdjson_path,
        tdjson_runtime_available,
        tdjson_probe_error,
        telegram_api_id_configured,
        telegram_api_hash_configured,
        telegram_app_credentials_configured,
        live_send_available,
        runtime_blockers,
        last_error: actor_state.last_error,
        updated_at: actor_state.updated_at,
    }
}

fn tdjson_probe(configured_path: Option<&std::path::Path>) -> (bool, Option<String>) {
    match TdJsonLibrary::load(configured_path) {
        Ok(_) => (true, None),
        Err(error) => (false, Some(error.to_string())),
    }
}

fn runtime_blockers(
    runtime_kind: &str,
    tdjson_runtime_available: bool,
    telegram_api_id_configured: bool,
    telegram_api_hash_configured: bool,
    last_error: Option<&str>,
) -> Vec<String> {
    let mut blockers = Vec::new();

    if runtime_kind == "live_blocked" {
        blockers.push("live_tdlib_runtime_blocked".to_owned());
    }
    if runtime_kind == "tdlib_qr_authorized" && !tdjson_runtime_available {
        blockers.push("tdjson_runtime_unavailable".to_owned());
    }
    if runtime_kind == "tdlib_qr_authorized" && !telegram_api_id_configured {
        blockers.push("telegram_api_id_missing".to_owned());
    }
    if runtime_kind == "tdlib_qr_authorized" && !telegram_api_hash_configured {
        blockers.push("telegram_api_hash_missing".to_owned());
    }
    if runtime_kind == "fixture" {
        return blockers;
    }
    if let Some(error) = last_error
        && !error.trim().is_empty()
    {
        blockers.push(error.trim().to_owned());
    }

    blockers
}

fn default_state_for_runtime(runtime_kind: &str) -> TelegramRuntimeActorState {
    let now = Utc::now();
    match runtime_kind {
        "live_blocked" => TelegramRuntimeActorState {
            status: TelegramRuntimeState::Blocked,
            last_error: Some("account runtime is blocked until live TDLib is enabled".to_owned()),
            updated_at: now,
        },
        _ => TelegramRuntimeActorState {
            status: TelegramRuntimeState::Stopped,
            last_error: None,
            updated_at: now,
        },
    }
}

pub(super) fn account_runtime_kind(account: &ProviderAccount) -> String {
    account
        .config
        .get("runtime")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(match account.provider_kind {
            CommunicationProviderKind::TelegramUser | CommunicationProviderKind::TelegramBot => {
                "unknown"
            }
            _ => "unsupported",
        })
        .to_owned()
}
