use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::platform::secrets::models::{SecretKind, SecretStoreKind};
use makosh_communications_api::accounts::CommunicationProviderKind;

use super::constants::{
    DEFAULT_GOOGLE_AUTHORIZATION_ENDPOINT, DEFAULT_GOOGLE_TOKEN_ENDPOINT,
    DEFAULT_GOOGLE_WORKSPACE_SCOPES,
};
use super::errors::EmailAccountSetupError;
use super::validation::validate_non_empty;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthSetupRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub app_return_url: Option<String>,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub scopes: Vec<String>,
}

impl GmailOAuthSetupRequest {
    pub fn new(
        account_id: impl Into<String>,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
        client_id: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            display_name: display_name.into(),
            external_account_id: external_account_id.into(),
            client_id: client_id.into(),
            client_secret: None,
            redirect_uri: redirect_uri.into(),
            app_return_url: None,
            authorization_endpoint: DEFAULT_GOOGLE_AUTHORIZATION_ENDPOINT.to_owned(),
            token_endpoint: DEFAULT_GOOGLE_TOKEN_ENDPOINT.to_owned(),
            scopes: DEFAULT_GOOGLE_WORKSPACE_SCOPES
                .iter()
                .map(|scope| (*scope).to_owned())
                .collect(),
        }
    }

    pub fn external_account_id(mut self, external_account_id: impl Into<String>) -> Self {
        self.external_account_id = external_account_id.into();
        self
    }

    pub fn client_secret(mut self, client_secret: impl Into<String>) -> Self {
        self.client_secret = Some(client_secret.into());
        self
    }

    pub fn app_return_url(mut self, app_return_url: impl Into<String>) -> Self {
        self.app_return_url = Some(app_return_url.into());
        self
    }

    pub fn authorization_endpoint(mut self, authorization_endpoint: impl Into<String>) -> Self {
        self.authorization_endpoint = authorization_endpoint.into();
        self
    }

    pub fn token_endpoint(mut self, token_endpoint: impl Into<String>) -> Self {
        self.token_endpoint = token_endpoint.into();
        self
    }

    pub fn scopes(mut self, scopes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.scopes = scopes.into_iter().map(Into::into).collect();
        self
    }

    pub(super) fn validate(&self) -> Result<(), EmailAccountSetupError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("client_id", &self.client_id)?;
        validate_non_empty("redirect_uri", &self.redirect_uri)?;
        validate_non_empty("authorization_endpoint", &self.authorization_endpoint)?;
        validate_non_empty("token_endpoint", &self.token_endpoint)?;
        if self.scopes.is_empty() {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "scopes",
                message: "must not be empty",
            });
        }
        for scope in &self.scopes {
            validate_non_empty("scope", scope)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthPendingGrant {
    pub setup_id: String,
    pub account_id: String,
    pub authorization_url: String,
    pub state: String,
    pub code_verifier: String,
    pub request: GmailOAuthSetupRequest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub mailbox: String,
    pub username: String,
    pub password: String,
    pub secret_kind: SecretKind,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_tls: Option<bool>,
    pub smtp_starttls: Option<bool>,
    pub smtp_username: Option<String>,
}

impl ImapAccountSetupRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account_id: impl Into<String>,
        provider_kind: CommunicationProviderKind,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        tls: bool,
        mailbox: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            provider_kind,
            display_name: display_name.into(),
            external_account_id: external_account_id.into(),
            host: host.into(),
            port,
            tls,
            mailbox: mailbox.into(),
            username: username.into(),
            password: password.into(),
            secret_kind: SecretKind::Password,
            smtp_host: None,
            smtp_port: None,
            smtp_tls: None,
            smtp_starttls: None,
            smtp_username: None,
        }
    }

    pub fn secret_kind(mut self, secret_kind: SecretKind) -> Self {
        self.secret_kind = secret_kind;
        self
    }

    pub fn smtp_host(mut self, smtp_host: impl Into<String>) -> Self {
        self.smtp_host = Some(smtp_host.into());
        self
    }

    pub fn smtp_port(mut self, smtp_port: u16) -> Self {
        self.smtp_port = Some(smtp_port);
        self
    }

    pub fn smtp_tls(mut self, smtp_tls: bool) -> Self {
        self.smtp_tls = Some(smtp_tls);
        self
    }

    pub fn smtp_starttls(mut self, smtp_starttls: bool) -> Self {
        self.smtp_starttls = Some(smtp_starttls);
        self
    }

    pub fn smtp_username(mut self, smtp_username: impl Into<String>) -> Self {
        self.smtp_username = Some(smtp_username.into());
        self
    }

    pub(super) fn smtp_config(&self) -> ImapAccountSmtpConfig {
        let default_host = match self.provider_kind {
            CommunicationProviderKind::Icloud => "smtp.mail.me.com".to_owned(),
            _ => self.host.clone(),
        };
        ImapAccountSmtpConfig {
            host: self.smtp_host.clone().unwrap_or(default_host),
            port: self.smtp_port.unwrap_or(587),
            tls: self.smtp_tls.unwrap_or(true),
            starttls: self.smtp_starttls.unwrap_or(true),
            username: self
                .smtp_username
                .clone()
                .unwrap_or_else(|| self.external_account_id.clone()),
        }
    }

    pub(super) fn validate(&self) -> Result<(), EmailAccountSetupError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_non_empty("host", &self.host)?;
        validate_non_empty("mailbox", &self.mailbox)?;
        validate_non_empty("username", &self.username)?;
        validate_non_empty("password", &self.password)?;
        if self.provider_kind == CommunicationProviderKind::Gmail {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "provider_kind",
                message: "must be icloud or imap",
            });
        }
        if self.port == 0 {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "port",
                message: "must be greater than zero",
            });
        }
        if !matches!(
            self.secret_kind,
            SecretKind::AppPassword | SecretKind::Password
        ) {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "secret_kind",
                message: "must be app_password or password",
            });
        }
        let smtp_config = self.smtp_config();
        validate_non_empty("smtp_host", &smtp_config.host)?;
        validate_non_empty("smtp_username", &smtp_config.username)?;
        if smtp_config.port == 0 {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "smtp_port",
                message: "must be greater than zero",
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ImapAccountSmtpConfig {
    pub(super) host: String,
    pub(super) port: u16,
    pub(super) tls: bool,
    pub(super) starttls: bool,
    pub(super) username: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailAccountSetupResult {
    pub account_id: String,
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct GmailOAuthTokenBundle {
    pub(super) token_url: String,
    pub(super) client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) client_secret: Option<String>,
    pub(super) access_token: String,
    pub(super) refresh_token: String,
    pub(super) expires_at: DateTime<Utc>,
    pub(super) token_type: Option<String>,
    pub(super) scope: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct OAuthTokenResponse {
    pub(super) access_token: String,
    pub(super) refresh_token: Option<String>,
    pub(super) expires_in: Option<i64>,
    pub(super) token_type: Option<String>,
    pub(super) scope: Option<String>,
}
