use axum::Json;
use axum::extract::{Path, State};
use base64::Engine as _;
use serde_json::{Value, json};

use super::helpers::{telegram_api_hash_from_config, telegram_secret_store};
use crate::app::api_support::stores::integration_stores::telegram_provider_runtime_service;
use crate::app::error::types::ApiError;
use crate::app::signal_hub_support::{
    provider_account_or_not_found, sync_provider_account_signal_connection,
};
use crate::app::state::AppState;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::accounts::TelegramLiveAccountSetupRequest;
use crate::integrations::telegram::client::models::qr_login::{
    TelegramQrLoginPasswordRequest, TelegramQrLoginStartRequest, TelegramQrLoginStatus,
    TelegramQrLoginStatusResponse,
};
use crate::integrations::telegram::client::vault::TelegramSecretVault;
use crate::integrations::telegram::tdjson::qr_login::commands::{
    cancel_qr_login, submit_qr_login_password,
};
use crate::integrations::telegram::tdjson::qr_login::driver::start_qr_login;
use crate::integrations::telegram::tdjson::requests::tdlib_database_directory;
use makosh_communications_api::accounts::CommunicationProviderKind;

pub(crate) async fn post_telegram_qr_login_start(
    State(state): State<AppState>,
    Json(request): Json<TelegramQrLoginStartRequest>,
) -> Result<Json<TelegramQrLoginStatusResponse>, ApiError> {
    let request = with_generated_session_encryption_key(request)?.with_app_credentials(
        state.config.telegram_api_id(),
        telegram_api_hash_from_config(&state.config),
    );

    Ok(Json(
        start_qr_login(
            state.config.clone(),
            state.account_setup.pending_telegram_qr_login.clone(),
            request,
        )
        .await?,
    ))
}

pub(crate) async fn get_telegram_qr_login_status(
    State(state): State<AppState>,
    Path(setup_id): Path<String>,
) -> Result<Json<TelegramQrLoginStatusResponse>, ApiError> {
    let (request, response) = {
        let pending = state
            .account_setup
            .pending_telegram_qr_login
            .lock()
            .map_err(|_| ApiError::AccountSetupState)?;
        pending
            .get(setup_id.trim())
            .map(|session| (session.request.clone(), session.response.clone()))
            .ok_or(ApiError::Telegram(TelegramError::QrLoginNotFound))?
    };
    let response = finalize_ready_telegram_qr_login(&state, request, response).await?;

    update_pending_qr_response(&state, &response)?;

    Ok(Json(response))
}

pub(crate) async fn delete_telegram_qr_login(
    State(state): State<AppState>,
    Path(setup_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let setup_id = setup_id.trim().to_owned();
    cancel_qr_login(
        state.account_setup.pending_telegram_qr_login.clone(),
        &setup_id,
    )?;

    Ok(Json(json!({
        "setup_id": setup_id,
        "cancelled": true
    })))
}

pub(crate) async fn post_telegram_qr_login_password(
    State(state): State<AppState>,
    Path(setup_id): Path<String>,
    Json(request): Json<TelegramQrLoginPasswordRequest>,
) -> Result<Json<TelegramQrLoginStatusResponse>, ApiError> {
    Ok(Json(submit_qr_login_password(
        state.account_setup.pending_telegram_qr_login.clone(),
        &setup_id,
        request,
    )?))
}

async fn finalize_ready_telegram_qr_login(
    state: &AppState,
    request: TelegramQrLoginStartRequest,
    response: TelegramQrLoginStatusResponse,
) -> Result<TelegramQrLoginStatusResponse, ApiError> {
    if response.status != TelegramQrLoginStatus::Ready {
        return Ok(response);
    }

    let session_encryption_key = request
        .session_encryption_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(
                "Telegram QR login session key is missing; restart QR login to create a recoverable account."
                    .to_owned(),
            ))
        })?;
    let account_id = response
        .suggested_account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(response.account_id.as_str())
        .to_owned();
    let external_account_id = response
        .suggested_external_account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(request.external_account_id.as_str())
        .to_owned();
    let display_name = request.display_name.trim();
    let display_name = if display_name.is_empty() {
        response
            .suggested_display_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("Telegram")
            .to_owned()
    } else {
        display_name.to_owned()
    };
    let tdlib_data_path = request
        .tdlib_data_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            tdlib_database_directory(&request)
                .to_string_lossy()
                .into_owned()
        });

    let setup_request = TelegramLiveAccountSetupRequest {
        account_id: account_id.clone(),
        provider_kind: CommunicationProviderKind::TelegramUser,
        display_name,
        external_account_id,
        api_id: None,
        api_hash: None,
        bot_token: None,
        session_encryption_key: Some(session_encryption_key),
        tdlib_data_path: Some(tdlib_data_path),
        qr_authorized: true,
        transcription_enabled: request.transcription_enabled,
    };
    let setup_response = telegram_provider_runtime_service(state)?
        .setup_live_blocked_account(
            &telegram_secret_store(state)?,
            &TelegramSecretVault::host(state.vault.clone()),
            &setup_request,
        )
        .await?;
    let account = provider_account_or_not_found(state, &setup_response.account_id).await?;
    sync_provider_account_signal_connection(state, &account, None).await?;

    let mut response = response;
    response.account_id = setup_response.account_id;
    Ok(response)
}

fn update_pending_qr_response(
    state: &AppState,
    response: &TelegramQrLoginStatusResponse,
) -> Result<(), ApiError> {
    let mut pending = state
        .account_setup
        .pending_telegram_qr_login
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?;
    if let Some(session) = pending.get_mut(&response.setup_id) {
        session.response = response.clone();
    }
    Ok(())
}

fn with_generated_session_encryption_key(
    mut request: TelegramQrLoginStartRequest,
) -> Result<TelegramQrLoginStartRequest, ApiError> {
    if request
        .session_encryption_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
    {
        return Ok(request);
    }

    let mut bytes = [0_u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|_| {
        ApiError::Telegram(TelegramError::TdlibRuntime(
            "failed to generate Telegram session encryption key".to_owned(),
        ))
    })?;
    request.session_encryption_key =
        Some(base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes));
    Ok(request)
}
