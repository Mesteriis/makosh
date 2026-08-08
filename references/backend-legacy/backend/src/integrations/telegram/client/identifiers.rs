use makosh_communications_api::accounts::ProviderAccount;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::TELEGRAM_ACCOUNT_ACTIVE;
use super::errors::TelegramError;
use super::models::accounts::TelegramAccount;

pub(crate) fn telegram_chat_id(account_id: &str, provider_chat_id: &str) -> String {
    format!(
        "telegram_chat:v4:{}",
        stable_hash([account_id, provider_chat_id].join("\0").as_bytes())
    )
}

pub(super) fn telegram_message_id(account_id: &str, provider_message_id: &str) -> String {
    format!(
        "message:v4:telegram:{}",
        stable_hash([account_id, provider_message_id].join("\0").as_bytes())
    )
}

pub(super) fn telegram_raw_record_id(
    account_id: &str,
    record_kind: &str,
    provider_record_id: &str,
) -> String {
    format!(
        "raw:v4:telegram:{}",
        stable_hash(
            [account_id, record_kind, provider_record_id]
                .join("\0")
                .as_bytes()
        )
    )
}

pub(crate) fn telegram_text_preview_hash(text: &str) -> String {
    format!("sha256:{}", stable_hash(text.trim().as_bytes()))
}

pub(super) fn telegram_account_runtime(account: &ProviderAccount) -> String {
    account
        .config
        .get("runtime")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_owned()
}

pub(crate) fn ensure_telegram_account_active(
    account: &ProviderAccount,
) -> Result<(), TelegramError> {
    let lifecycle_state = telegram_account_lifecycle_state(account);
    if lifecycle_state != TELEGRAM_ACCOUNT_ACTIVE {
        return Err(TelegramError::InvalidRequest(format!(
            "Telegram account `{}` is `{}` and cannot run provider operations",
            account.account_id, lifecycle_state
        )));
    }

    Ok(())
}

pub(super) fn telegram_account_from_provider_account(account: ProviderAccount) -> TelegramAccount {
    let runtime = telegram_account_runtime(&account);
    let lifecycle_state = telegram_account_lifecycle_state(&account);
    let transcription_enabled = account
        .config
        .get("transcription_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let tdlib_data_path = account
        .config
        .get("tdlib_data_path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    TelegramAccount {
        account_id: account.account_id,
        provider_kind: account.provider_kind.as_str().to_owned(),
        display_name: account.display_name,
        external_account_id: account.external_account_id,
        runtime,
        lifecycle_state,
        transcription_enabled,
        tdlib_data_path,
        created_at: account.created_at,
        updated_at: account.updated_at,
    }
}

pub(super) fn telegram_account_lifecycle_state(account: &ProviderAccount) -> String {
    account
        .config
        .get("lifecycle_state")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(TELEGRAM_ACCOUNT_ACTIVE)
        .to_owned()
}

pub(super) fn stable_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub(super) fn telegram_secret_ref(
    account_id: &str,
    secret_purpose: ProviderAccountSecretPurpose,
) -> String {
    format!(
        "secret:provider-account:{}:{}",
        account_id.trim(),
        secret_purpose.as_str()
    )
}
