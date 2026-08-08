use super::*;
use makosh_communications_api::accounts::ProviderAccount;

// ---------------------------------------------------------------------------

/// Capability states per ADR-0091.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramCapabilityState {
    Available,
    Blocked,
    Degraded,
    Planned,
    Unsupported,
}

impl TelegramCapabilityState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Blocked => "blocked",
            Self::Degraded => "degraded",
            Self::Planned => "planned",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Action classes per ADR-0052.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramActionClass {
    Read,
    LocalWrite,
    ProviderWrite,
    Destructive,
    Export,
    SecretAccess,
    Automation,
}

impl TelegramActionClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::LocalWrite => "local_write",
            Self::ProviderWrite => "provider_write",
            Self::Destructive => "destructive",
            Self::Export => "export",
            Self::SecretAccess => "secret_access",
            Self::Automation => "automation",
        }
    }
}

/// A single operation capability entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelegramOperationCapability {
    pub operation: String,
    pub category: String,
    pub status: String,
    pub action_class: String,
    pub reason: String,
    pub confirmation_required: bool,
    pub closure_gate: bool,
}

impl TelegramOperationCapability {
    pub(super) fn new(
        operation: &str,
        category: &str,
        state: TelegramCapabilityState,
        action_class: TelegramActionClass,
        reason: &str,
        confirmation_required: bool,
        closure_gate: bool,
    ) -> Self {
        Self {
            operation: operation.to_owned(),
            category: category.to_owned(),
            status: state.as_str().to_owned(),
            action_class: action_class.as_str().to_owned(),
            reason: reason.to_owned(),
            confirmation_required,
            closure_gate,
        }
    }
}

/// Detailed per-operation Telegram capability response per ADR-0091.
#[derive(Serialize)]
pub(crate) struct TelegramCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: &'static str,
    pub(crate) account_scope: Option<TelegramCapabilityAccountScope>,
    pub(crate) telegram_app_credentials_configured: bool,
    pub(crate) tdjson_runtime_available: bool,
    pub(crate) qr_login_ready: bool,
    pub(crate) bot_runtime_available: bool,
    pub(crate) capabilities: Vec<TelegramOperationCapability>,
    pub(crate) planned_features: Vec<&'static str>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelegramCapabilityAccountScope {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime_kind: String,
    pub lifecycle_state: String,
}

impl TelegramCapabilitiesResponse {
    pub(crate) fn current(config: &AppConfig) -> Self {
        Self::build(config, None)
    }

    pub(crate) fn current_for_account(config: &AppConfig, account: &ProviderAccount) -> Self {
        Self::build(config, Some(account))
    }

    fn build(config: &AppConfig, account: Option<&ProviderAccount>) -> Self {
        let app_creds = config.telegram_api_id().is_some() && config.telegram_api_hash().is_some();
        let tdjson_ok = tdjson::client::runtime_available(config.tdjson_path());
        let qr_ready = app_creds && tdjson_ok;
        let bot_ok = false; // Bot API runtime not implemented per ADR-0091
        let account_scope = account.map(TelegramCapabilityAccountScope::from_account);

        let capabilities = super::telegram_capability_catalog::telegram_capability_rows(qr_ready);
        let mut response = Self {
            version: "2.1",
            runtime_mode: if let Some(scope) = account_scope.as_ref() {
                match scope.runtime_kind.as_str() {
                    "tdlib_qr_authorized" => "tdlib_qr_authorized",
                    "live_blocked" => "live_blocked",
                    "fixture" => "fixture",
                    _ => "unknown",
                }
            } else if qr_ready {
                "tdlib_qr"
            } else {
                "fixture"
            },
            account_scope,
            telegram_app_credentials_configured: app_creds,
            tdjson_runtime_available: tdjson_ok,
            qr_login_ready: qr_ready,
            bot_runtime_available: bot_ok,
            capabilities,
            planned_features: vec![
                "bot_runtime",
                "voice_recording",
                "voice_send",
                "video_recording",
                "live_calls",
                "session_export",
                "session_import",
                "mtproxy",
                "socks5",
                "ai_summary",
                "translation",
                "bilingual_reply",
                "ai_review_flows",
            ],
            unsupported_features: vec![
                "group_calls",
                "screen_sharing",
                "hidden_recording",
                "telegram_data_fine_tuning",
                "third_party_plugin_execution",
                "chat_export",
            ],
        };
        response.apply_account_scope_overrides();
        response
    }

    fn apply_account_scope_overrides(&mut self) {
        let Some(scope) = self.account_scope.as_ref() else {
            return;
        };
        let provider_kind = scope.provider_kind.as_str();
        let lifecycle_state = scope.lifecycle_state.as_str();
        let runtime_kind = scope.runtime_kind.as_str();
        let is_bot = provider_kind == "telegram_bot";
        let is_removed = lifecycle_state == "removed";
        let is_logged_out = lifecycle_state == "logged_out";

        for capability in &mut self.capabilities {
            match capability.operation.as_str() {
                "auth.qr_start" | "auth.qr_status" | "auth.qr_password" | "auth.qr_cancel"
                    if is_bot =>
                {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use TDLib QR authorization.".to_owned();
                }
                "runtime.tdlib_live" if is_bot => {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use the TDLib user runtime.".to_owned();
                }
                "runtime.bot_live" if !is_bot => {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason = "User accounts do not use the Bot API runtime.".to_owned();
                }
                "messages.send_text" if is_bot && runtime_kind == "fixture" => {
                    capability.status = TelegramCapabilityState::Degraded.as_str().to_owned();
                    capability.reason = "Fixture bot accounts can validate local command flow, but the live Bot API runtime is still missing.".to_owned();
                }
                "messages.send_media" | "media.upload_send" if is_bot => {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason =
                        "Bot media upload/send requires the separate Bot API runtime.".to_owned();
                }
                "messages.send_media" | "media.upload_send"
                    if runtime_kind != "tdlib_qr_authorized" =>
                {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Account `{}` must use tdlib_qr_authorized runtime before media upload/send is available.",
                        scope.account_id
                    );
                }
                "participants.sync" | "participants.join" | "participants.leave" if is_bot => {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use the TDLib participant lifecycle runtime."
                            .to_owned();
                }
                "dialogs.folder_add" | "dialogs.folder_remove" | "dialogs.folder_reassign"
                    if is_bot =>
                {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use the TDLib user dialog-folder runtime.".to_owned();
                }
                "topics.list" | "topics.create" | "topics.close" if is_bot => {
                    capability.status = TelegramCapabilityState::Unsupported.as_str().to_owned();
                    capability.reason =
                        "Bot accounts do not use the TDLib forum topic projection/runtime."
                            .to_owned();
                }
                "participants.sync" | "participants.join" | "participants.leave"
                    if runtime_kind != "tdlib_qr_authorized" =>
                {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Account `{}` must use tdlib_qr_authorized runtime before participant lifecycle commands are available.",
                        scope.account_id
                    );
                }
                "participants.sync" => {
                    capability.status = TelegramCapabilityState::Available.as_str().to_owned();
                    capability.reason = "TDLib provider member roster sync uses provider-observed roster snapshots for groups and TDLib chat metadata for private/saved-message chats.".to_owned();
                }
                "participants.join" => {
                    capability.status = TelegramCapabilityState::Available.as_str().to_owned();
                    capability.reason = "Joining Telegram chats uses the durable provider-write outbox and reconciles completion from provider-observed membership evidence.".to_owned();
                }
                "participants.leave" => {
                    capability.status = TelegramCapabilityState::Available.as_str().to_owned();
                    capability.reason = "Leaving Telegram chats uses the durable provider-write outbox and reconciles completion from provider-observed membership or exhaustive absence evidence.".to_owned();
                }
                "dialogs.folder_add" | "dialogs.folder_remove" | "dialogs.folder_reassign"
                    if runtime_kind != "tdlib_qr_authorized" =>
                {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Account `{}` must use tdlib_qr_authorized runtime before Telegram folder provider-write commands are available.",
                        scope.account_id
                    );
                }
                "dialogs.folder_add" => {
                    capability.status = TelegramCapabilityState::Available.as_str().to_owned();
                    capability.reason = "Adding a chat to a Telegram folder uses the durable provider-write outbox and TDLib chat-position reconciliation for the target folder.".to_owned();
                }
                "dialogs.folder_remove" => {
                    capability.status = TelegramCapabilityState::Available.as_str().to_owned();
                    capability.reason = "Removing a chat from a Telegram folder uses the durable provider-write outbox plus TDLib folder-edit reconciliation when the provider confirms removal.".to_owned();
                }
                "dialogs.folder_reassign" => {
                    capability.status = TelegramCapabilityState::Available.as_str().to_owned();
                    capability.reason = "Reassigning Telegram folder membership computes a durable add/remove command set from the current TDLib folder projection and queues those provider-write commands atomically from one API action.".to_owned();
                }
                "topics.list" if runtime_kind != "tdlib_qr_authorized" => {
                    capability.status = TelegramCapabilityState::Degraded.as_str().to_owned();
                    capability.reason = format!(
                        "Account `{}` can read locally projected forum topics, but live TDLib topic refresh requires tdlib_qr_authorized runtime.",
                        scope.account_id
                    );
                }
                "topics.create" | "topics.close" if runtime_kind != "tdlib_qr_authorized" => {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Account `{}` must use tdlib_qr_authorized runtime before forum topic write commands are available.",
                        scope.account_id
                    );
                }
                _ => {}
            }

            if is_removed {
                if matches!(
                    capability.action_class.as_str(),
                    "local_write"
                        | "provider_write"
                        | "destructive"
                        | "secret_access"
                        | "automation"
                ) || capability.operation.starts_with("sync.")
                    || capability.operation.starts_with("runtime.")
                {
                    capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                    capability.reason = format!(
                        "Account `{}` is removed; this operation is no longer available.",
                        scope.account_id
                    );
                }
            } else if is_logged_out
                && (capability.operation.starts_with("sync.")
                    || capability.operation == "messages.send_text"
                    || capability.operation == "messages.send_media"
                    || capability.operation == "media.upload_send"
                    || capability.operation == "media.download")
            {
                capability.status = TelegramCapabilityState::Blocked.as_str().to_owned();
                capability.reason = format!(
                    "Account `{}` is logged out; re-authorize the runtime before using this operation.",
                    scope.account_id
                );
            }
        }
    }
}

impl TelegramCapabilityAccountScope {
    fn from_account(account: &ProviderAccount) -> Self {
        Self {
            account_id: account.account_id.clone(),
            provider_kind: account.provider_kind.as_str().to_owned(),
            runtime_kind: account
                .config
                .get("runtime")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("unknown")
                .to_owned(),
            lifecycle_state: account
                .config
                .get("lifecycle_state")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("active")
                .to_owned(),
        }
    }
}
