use serde_json::Value;

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::qr_login::TelegramQrLoginStartRequest;
use crate::platform::config::app_config::AppConfig;
use makosh_communications_api::accounts::ProviderAccount;

pub(super) fn tdlib_start_request_from_account(
    config: &AppConfig,
    account: &ProviderAccount,
    session_encryption_key: Option<String>,
) -> Result<TelegramQrLoginStartRequest, TelegramError> {
    let api_id = config.telegram_api_id().ok_or_else(|| {
        TelegramError::InvalidRequest(
            "MAKOSH_TELEGRAM_API_ID is required for Telegram TDLib runtime".to_owned(),
        )
    })?;
    let api_hash = config
        .telegram_api_hash()
        .map(|secret| secret.expose_for_runtime().to_owned())
        .ok_or_else(|| {
            TelegramError::InvalidRequest(
                "MAKOSH_TELEGRAM_API_HASH is required for Telegram TDLib runtime".to_owned(),
            )
        })?;
    let tdlib_data_path = account
        .config
        .get("tdlib_data_path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(
                "tdlib_data_path is required for Telegram TDLib runtime".to_owned(),
            )
        })?;

    Ok(TelegramQrLoginStartRequest {
        account_id: account.account_id.clone(),
        display_name: account.display_name.clone(),
        external_account_id: account.external_account_id.clone(),
        api_id: Some(api_id),
        api_hash: Some(api_hash),
        session_encryption_key,
        tdlib_data_path: Some(tdlib_data_path),
        transcription_enabled: false,
    })
}
