use chrono::{DateTime, Utc};
use makosh_provider_telegram::tdlib::types::TdlibMediaKind;
use serde::{Deserialize, Serialize};

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::chats::TelegramChat;
use crate::integrations::telegram::client::models::messages::TelegramMessage;

use super::validation::{validate_limit, validate_non_empty};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramRuntimeStartRequest {
    pub account_id: String,
}

impl TelegramRuntimeStartRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramRuntimeStopRequest {
    pub account_id: String,
}

impl TelegramRuntimeStopRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramRuntimeRestartRequest {
    pub account_id: String,
}

impl TelegramRuntimeRestartRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramChatSyncRequest {
    pub account_id: String,
    pub limit: Option<i64>,
}

impl TelegramChatSyncRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        if let Some(limit) = self.limit {
            validate_limit(limit)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChatSyncResponse {
    pub account_id: String,
    pub runtime_kind: String,
    pub status: String,
    pub synced_count: usize,
    pub items: Vec<TelegramChat>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramHistorySyncRequest {
    pub account_id: String,
    pub provider_chat_id: String,
    pub from_message_id: Option<i64>,
    pub mode: Option<TelegramHistorySyncMode>,
    pub limit: Option<i64>,
}

impl TelegramHistorySyncRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        if let Some(from_message_id) = self.from_message_id
            && from_message_id <= 0
        {
            return Err(TelegramError::InvalidRequest(
                "from_message_id must be a positive TDLib message id".to_owned(),
            ));
        }
        if self.mode() == TelegramHistorySyncMode::Older && self.from_message_id.is_none() {
            return Err(TelegramError::InvalidRequest(
                "from_message_id is required when mode=older".to_owned(),
            ));
        }
        if let Some(limit) = self.limit {
            validate_limit(limit)?;
        }
        Ok(())
    }

    pub(super) fn mode(&self) -> TelegramHistorySyncMode {
        self.mode.unwrap_or(TelegramHistorySyncMode::Latest)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramHistorySyncMode {
    Latest,
    Older,
    Full,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramHistorySyncResponse {
    pub account_id: String,
    pub provider_chat_id: String,
    pub runtime_kind: String,
    pub status: String,
    pub synced_count: usize,
    pub has_more: bool,
    pub next_from_message_id: Option<i64>,
    pub items: Vec<TelegramMessage>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramMediaDownloadRequest {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub tdlib_file_id: i64,
    pub provider_attachment_id: Option<String>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub priority: Option<i32>,
}

impl TelegramMediaDownloadRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        if self.tdlib_file_id <= 0 {
            return Err(TelegramError::InvalidRequest(
                "tdlib_file_id must be a positive TDLib file id".to_owned(),
            ));
        }
        if let Some(priority) = self.priority
            && !(1..=32).contains(&priority)
        {
            return Err(TelegramError::InvalidRequest(
                "priority must be between 1 and 32".to_owned(),
            ));
        }
        if let Some(value) = &self.provider_attachment_id {
            validate_non_empty("provider_attachment_id", value)?;
        }
        if let Some(value) = &self.filename {
            validate_non_empty("filename", value)?;
        }
        if let Some(value) = &self.content_type {
            validate_non_empty("content_type", value)?;
        }
        Ok(())
    }

    pub(crate) fn provider_attachment_id(&self) -> String {
        self.provider_attachment_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("tdlib-file:{}", self.tdlib_file_id))
    }

    pub(crate) fn content_type(&self) -> String {
        self.content_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "application/octet-stream".to_owned())
    }

    pub(crate) fn filename(&self) -> Option<String> {
        self.filename
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMediaDownloadResponse {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub runtime_kind: String,
    pub status: String,
    pub tdlib_file_id: i64,
    pub local_path: Option<String>,
    pub size_bytes: Option<i64>,
    pub expected_size_bytes: Option<i64>,
    pub downloaded_size_bytes: Option<i64>,
    pub is_downloading_active: bool,
    pub is_downloading_completed: bool,
    pub attachment_id: Option<String>,
    pub blob_id: Option<String>,
    pub scan_status: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramMediaSendRequest {
    pub command_id: String,
    pub provider_chat_id: String,
    pub media_type: TdlibMediaKind,
    pub local_path: String,
    pub caption: Option<String>,
    pub filename: Option<String>,
}

impl TelegramMediaSendRequest {
    pub(super) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("command_id", &self.command_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("local_path", &self.local_path)?;
        if let Some(caption) = &self.caption {
            validate_non_empty("caption", caption)?;
        }
        if let Some(filename) = &self.filename {
            validate_non_empty("filename", filename)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramRuntimeStatus {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime_kind: String,
    pub status: String,
    pub fixture_runtime: bool,
    pub tdjson_path: Option<String>,
    pub tdjson_runtime_available: bool,
    pub tdjson_probe_error: Option<String>,
    pub telegram_api_id_configured: bool,
    pub telegram_api_hash_configured: bool,
    pub telegram_app_credentials_configured: bool,
    pub live_send_available: bool,
    pub runtime_blockers: Vec<String>,
    pub last_error: Option<String>,
    pub updated_at: DateTime<Utc>,
}
