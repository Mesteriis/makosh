use std::time::Instant;

use serde_json::{Value, json};

use crate::integrations::telegram::client::errors::TelegramError;

use super::super::client::TdJsonClient;
use super::super::parsing::events::tdlib_error_message;
use super::constants::QR_GET_ME_TIMEOUT;
use super::types::TelegramQrLoginIdentity;

pub(in crate::integrations::telegram::tdjson) fn fetch_authorized_user_identity(
    client: &TdJsonClient,
) -> Result<Option<TelegramQrLoginIdentity>, TelegramError> {
    client.send_json(&json!({
        "@type": "getMe",
        "@extra": "makosh-get-me"
    }))?;

    let started_at = Instant::now();
    while started_at.elapsed() < QR_GET_ME_TIMEOUT {
        let Some(event) = client.receive_json(1.0)? else {
            continue;
        };

        if event.get("@type").and_then(Value::as_str) == Some("user") {
            return Ok(parse_tdlib_user_identity(&event));
        }

        if event.get("@extra").and_then(Value::as_str) == Some("makosh-get-me") {
            if let Some(message) = tdlib_error_message(&event) {
                return Err(TelegramError::TdlibRuntime(message));
            }
            return Ok(parse_tdlib_user_identity(&event));
        }
    }

    Ok(None)
}

pub(in crate::integrations::telegram::tdjson) fn parse_tdlib_user_identity(
    user: &Value,
) -> Option<TelegramQrLoginIdentity> {
    let user_id = user
        .get("id")
        .and_then(|value| {
            value
                .as_i64()
                .map(|value| value.to_string())
                .or_else(|| value.as_u64().map(|value| value.to_string()))
        })
        .filter(|value| !value.trim().is_empty())?;
    let username = tdlib_user_username(user);
    let safe_user_id = safe_account_identifier(&user_id);
    let suggested_account_id = username
        .as_deref()
        .map(safe_account_identifier)
        .filter(|value| !value.is_empty())
        .map(|username| format!("{safe_user_id}_account_{username}"))
        .unwrap_or_else(|| format!("{safe_user_id}_account"));
    let suggested_display_name = username
        .as_deref()
        .map(|value| format!("@{value}"))
        .unwrap_or_else(|| user_id.clone());
    let suggested_external_account_id = format!("telegram:{user_id}");

    Some(TelegramQrLoginIdentity {
        user_id,
        username,
        suggested_account_id,
        suggested_display_name,
        suggested_external_account_id,
    })
}

fn tdlib_user_username(user: &Value) -> Option<String> {
    user.get("username")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            user.get("usernames")
                .and_then(|value| value.get("active_usernames"))
                .and_then(Value::as_array)
                .and_then(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .find(|value| !value.trim().is_empty())
                })
                .map(str::trim)
                .map(ToOwned::to_owned)
        })
}

fn safe_account_identifier(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_owned();

    if sanitized.is_empty() {
        "telegram".to_owned()
    } else {
        sanitized
    }
}
