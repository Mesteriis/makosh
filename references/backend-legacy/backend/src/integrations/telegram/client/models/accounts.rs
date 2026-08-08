use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::platform::secrets::models::{SecretKind, SecretStoreKind};
use makosh_communications_api::accounts::CommunicationProviderKind;

use super::super::errors::TelegramError;
use super::super::validation::{required_optional_value, validate_non_empty};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub tdlib_data_path: Option<String>,
    #[serde(default)]
    pub transcription_enabled: bool,
}

impl TelegramAccountSetupRequest {
    pub(in crate::integrations::telegram::client) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramLiveAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub api_id: Option<i64>,
    pub api_hash: Option<String>,
    pub bot_token: Option<String>,
    pub session_encryption_key: Option<String>,
    pub tdlib_data_path: Option<String>,
    #[serde(default)]
    pub qr_authorized: bool,
    #[serde(default)]
    pub transcription_enabled: bool,
}

impl TelegramLiveAccountSetupRequest {
    pub(crate) fn with_inferred_qr_authorization(mut self) -> Self {
        if self.is_finalized_qr_user_account() {
            self.qr_authorized = true;
        }
        self
    }

    pub(crate) fn with_app_credentials(
        mut self,
        api_id: Option<i64>,
        api_hash: Option<String>,
    ) -> Self {
        if self.is_qr_authorized_user_account() {
            return self;
        }
        if self.api_id.is_none() {
            self.api_id = api_id;
        }
        if self
            .api_hash
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
        {
            self.api_hash = api_hash;
        }
        self
    }

    pub(in crate::integrations::telegram::client) fn is_qr_authorized_user_account(&self) -> bool {
        self.qr_authorized && self.provider_kind == CommunicationProviderKind::TelegramUser
    }

    fn is_finalized_qr_user_account(&self) -> bool {
        self.provider_kind == CommunicationProviderKind::TelegramUser
            && self
                .external_account_id
                .trim()
                .strip_prefix("telegram:")
                .is_some_and(|provider_user_id| !provider_user_id.trim().is_empty())
            && self
                .tdlib_data_path
                .as_deref()
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
    }

    pub(in crate::integrations::telegram::client) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        match self.provider_kind {
            CommunicationProviderKind::TelegramUser => {
                if self.is_qr_authorized_user_account() {
                    required_optional_value("tdlib_data_path", self.tdlib_data_path.as_deref())?;
                    return Ok(());
                }
                let api_id = self.api_id.ok_or_else(|| {
                    TelegramError::InvalidRequest("api_id must not be empty".to_owned())
                })?;
                if api_id <= 0 {
                    return Err(TelegramError::InvalidRequest(
                        "api_id must be greater than zero".to_owned(),
                    ));
                }
                required_optional_value("api_hash", self.api_hash.as_deref())?;
            }
            CommunicationProviderKind::TelegramBot => {
                if self.qr_authorized {
                    return Err(TelegramError::InvalidRequest(
                        "qr_authorized is only supported for telegram_user".to_owned(),
                    ));
                }
                required_optional_value("bot_token", self.bot_token.as_deref())?;
            }
            _ => {
                return Err(TelegramError::InvalidRequest(
                    "provider_kind must be telegram_user or telegram_bot".to_owned(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccountSetupResponse {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime: String,
    pub transcription_enabled: bool,
    pub credential_bindings: Vec<TelegramCredentialBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccount {
    pub account_id: String,
    pub provider_kind: String,
    pub display_name: String,
    pub external_account_id: String,
    pub runtime: String,
    pub lifecycle_state: String,
    pub transcription_enabled: bool,
    pub tdlib_data_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccountListResponse {
    pub items: Vec<TelegramAccount>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccountLifecycleResponse {
    pub account: TelegramAccount,
    pub stopped_runtime_actor: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramCredentialBinding {
    pub secret_purpose: String,
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
}
