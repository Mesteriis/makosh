use super::super::*;
use super::helpers::trimmed_optional;
use makosh_communications_api::accounts::CommunicationProviderKind;

#[derive(Deserialize)]
pub(crate) struct GmailOAuthStartApiRequest {
    pub(super) account_id: String,
    pub(super) display_name: String,
    pub(super) external_account_id: Option<String>,
    pub(super) client_id: Option<String>,
    pub(super) client_secret: Option<String>,
    pub(super) redirect_uri: String,
    pub(super) app_return_url: Option<String>,
    pub(super) scopes: Option<Vec<String>>,
    pub(super) authorization_endpoint: Option<String>,
    pub(super) token_endpoint: Option<String>,
}

impl GmailOAuthStartApiRequest {
    pub(super) fn into_setup_request(
        self,
        config: &crate::platform::config::app_config::AppConfig,
    ) -> Result<GmailOAuthSetupRequest, EmailAccountSetupError> {
        let client_id = trimmed_optional(self.client_id)
            .or_else(|| config.google_oauth_client_id().map(str::to_owned))
            .ok_or(EmailAccountSetupError::InvalidRequest {
                field: "client_id",
                message: "must be configured as request client_id or MAKOSH_GOOGLE_OAUTH_CLIENT_ID",
            })?;
        let mut request = GmailOAuthSetupRequest::new(
            self.account_id,
            self.display_name,
            trimmed_optional(self.external_account_id).unwrap_or_default(),
            client_id,
            self.redirect_uri,
        );
        if let Some(app_return_url) = trimmed_optional(self.app_return_url) {
            request = request.app_return_url(app_return_url);
        }
        if let Some(client) = config.google_oauth_client() {
            request = request
                .authorization_endpoint(client.authorization_endpoint().to_owned())
                .token_endpoint(client.token_endpoint().to_owned());
        }
        if let Some(client_secret) = trimmed_optional(self.client_secret).or_else(|| {
            config
                .google_oauth_client_secret()
                .map(|secret| secret.expose_for_runtime().to_owned())
        }) {
            request = request.client_secret(client_secret);
        }
        if let Some(scopes) = self.scopes {
            request = request.scopes(scopes);
        }
        if let Some(authorization_endpoint) = self.authorization_endpoint {
            request = request.authorization_endpoint(authorization_endpoint);
        }
        if let Some(token_endpoint) = self.token_endpoint {
            request = request.token_endpoint(token_endpoint);
        }

        Ok(request)
    }
}

#[derive(Serialize)]
pub(crate) struct GmailOAuthStartApiResponse {
    pub(super) setup_id: String,
    pub(super) authorization_url: String,
    pub(super) state: String,
    pub(super) redirect_uri: String,
}

#[derive(Deserialize)]
pub(crate) struct GmailOAuthCompleteApiRequest {
    pub(super) setup_id: String,
    pub(super) state: String,
    pub(super) authorization_code: String,
    pub(super) external_account_id: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct GmailOAuthCallbackQuery {
    pub(super) code: Option<String>,
    pub(super) state: Option<String>,
    pub(super) error: Option<String>,
    pub(super) error_description: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ImapAccountSetupApiRequest {
    pub(super) account_id: String,
    pub(super) provider_kind: String,
    pub(super) display_name: String,
    pub(super) external_account_id: String,
    pub(super) host: String,
    pub(super) port: u16,
    pub(super) tls: bool,
    pub(super) mailbox: String,
    pub(super) username: String,
    pub(super) password: String,
    pub(super) secret_kind: Option<String>,
    pub(super) smtp_host: Option<String>,
    pub(super) smtp_port: Option<u16>,
    pub(super) smtp_tls: Option<bool>,
    pub(super) smtp_starttls: Option<bool>,
    pub(super) smtp_username: Option<String>,
}

impl ImapAccountSetupApiRequest {
    pub(super) fn into_setup_request(self) -> Result<ImapAccountSetupRequest, ApiError> {
        let Self {
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            host,
            port,
            tls,
            mailbox,
            username,
            password,
            secret_kind,
            smtp_host,
            smtp_port,
            smtp_tls,
            smtp_starttls,
            smtp_username,
        } = self;
        let provider_kind = match provider_kind.trim() {
            "icloud" => CommunicationProviderKind::Icloud,
            "imap" => CommunicationProviderKind::Imap,
            _ => {
                return Err(EmailAccountSetupError::InvalidRequest {
                    field: "provider_kind",
                    message: "must be icloud or imap",
                }
                .into());
            }
        };
        let secret_kind = match secret_kind.as_deref().unwrap_or("password").trim() {
            "app_password" => SecretKind::AppPassword,
            "password" => SecretKind::Password,
            _ => {
                return Err(EmailAccountSetupError::InvalidRequest {
                    field: "secret_kind",
                    message: "must be app_password or password",
                }
                .into());
            }
        };

        let mut request = ImapAccountSetupRequest::new(
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            host,
            port,
            tls,
            mailbox,
            username,
            password,
        )
        .secret_kind(secret_kind);
        if let Some(smtp_host) = trimmed_optional(smtp_host) {
            request = request.smtp_host(smtp_host);
        }
        if let Some(smtp_port) = smtp_port {
            request = request.smtp_port(smtp_port);
        }
        if let Some(smtp_tls) = smtp_tls {
            request = request.smtp_tls(smtp_tls);
        }
        if let Some(smtp_starttls) = smtp_starttls {
            request = request.smtp_starttls(smtp_starttls);
        }
        if let Some(smtp_username) = trimmed_optional(smtp_username) {
            request = request.smtp_username(smtp_username);
        }

        Ok(request)
    }
}

#[derive(Serialize)]
pub(crate) struct EmailAccountSetupApiResponse {
    pub(super) account_id: String,
    pub(super) secret_ref: String,
    pub(super) secret_kind: SecretKind,
    pub(super) store_kind: crate::platform::secrets::models::SecretStoreKind,
}

impl From<crate::integrations::mail::accounts::models::EmailAccountSetupResult>
    for EmailAccountSetupApiResponse
{
    fn from(result: crate::integrations::mail::accounts::models::EmailAccountSetupResult) -> Self {
        Self {
            account_id: result.account_id,
            secret_ref: result.secret_ref,
            secret_kind: result.secret_kind,
            store_kind: result.store_kind,
        }
    }
}
