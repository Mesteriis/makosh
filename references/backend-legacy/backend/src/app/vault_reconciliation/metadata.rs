use serde_json::{Value, json};

use makosh_communications_api::accounts::CommunicationProviderKind;

pub(super) fn fallback_provider_account_config(
    provider_kind: CommunicationProviderKind,
    metadata: &Value,
    external_account_id: &str,
) -> Value {
    let connected_services = metadata
        .get("connected_services")
        .cloned()
        .unwrap_or_else(|| json!(["mail"]));
    match provider_kind {
        CommunicationProviderKind::Gmail => json!({
            "auth": "oauth",
            "api": "gmail",
            "connected_services": connected_services,
            "history_stream_id": "gmail:history"
        }),
        CommunicationProviderKind::Icloud => json!({
            "host": "imap.mail.me.com",
            "port": 993,
            "tls": true,
            "mailbox": "INBOX",
            "username": external_account_id,
            "connected_services": connected_services
        }),
        CommunicationProviderKind::Imap => json!({
            "username": external_account_id,
            "connected_services": connected_services
        }),
        CommunicationProviderKind::TelegramUser | CommunicationProviderKind::TelegramBot => {
            json!({
                "runtime": metadata_string(metadata, "runtime")
                    .unwrap_or_else(|| "live_blocked".to_owned()),
                "transcription_enabled": metadata
                    .get("transcription_enabled")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
        }
        CommunicationProviderKind::WhatsappWeb
        | CommunicationProviderKind::WhatsappBusinessCloud => json!({
            "provider_shape": metadata_string(metadata, "provider_shape")
                .unwrap_or_else(|| provider_kind.as_str().to_owned()),
            "runtime_kind": metadata_string(metadata, "runtime")
                .or_else(|| metadata_string(metadata, "runtime_kind"))
                .unwrap_or_else(|| "vault_restored".to_owned()),
            "lifecycle_state": "restored"
        }),
        CommunicationProviderKind::ZulipBot => json!({
            "base_url": metadata_string(metadata, "base_url").unwrap_or_default(),
            "runtime": "api"
        }),
        CommunicationProviderKind::ZoomUser | CommunicationProviderKind::ZoomServerToServer => {
            json!({
                "auth_shape": metadata_string(metadata, "auth_shape").unwrap_or_default(),
                "runtime_kind": "oauth",
                "credential_refs_bound": true
            })
        }
        CommunicationProviderKind::YandexTelemostUser => json!({
            "runtime_kind": "oauth",
            "metadata": metadata
                .get("metadata")
                .cloned()
                .unwrap_or_else(|| json!({}))
        }),
    }
}

pub(super) fn fallback_display_name(
    provider_kind: CommunicationProviderKind,
    label: &str,
    account_id: &str,
) -> String {
    let trimmed = label.trim();
    if !trimmed.is_empty()
        && !matches!(
            trimmed.to_ascii_lowercase().as_str(),
            "imap password" | "oauth credential" | "api token" | "client secret"
        )
    {
        return trimmed.to_owned();
    }
    match provider_kind {
        CommunicationProviderKind::Gmail => "Google Workspace".to_owned(),
        CommunicationProviderKind::Icloud => "iCloud".to_owned(),
        CommunicationProviderKind::Imap => account_id.to_owned(),
        CommunicationProviderKind::TelegramUser => "Telegram".to_owned(),
        CommunicationProviderKind::TelegramBot => "Telegram Bot".to_owned(),
        CommunicationProviderKind::WhatsappWeb => "WhatsApp".to_owned(),
        CommunicationProviderKind::WhatsappBusinessCloud => "WhatsApp Business Cloud".to_owned(),
        CommunicationProviderKind::ZulipBot => "Zulip".to_owned(),
        CommunicationProviderKind::ZoomUser | CommunicationProviderKind::ZoomServerToServer => {
            "Zoom".to_owned()
        }
        CommunicationProviderKind::YandexTelemostUser => "Yandex Telemost".to_owned(),
    }
}

pub(super) fn metadata_string(metadata: &Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

pub(super) fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}
