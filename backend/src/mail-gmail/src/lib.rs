//! Gmail REST adapter owned by the Mail integration.
//!
//! It exposes provider operations only. Communications evidence mapping, durable
//! state and credential leasing stay in their respective owner packages.

use std::{collections::BTreeSet, fmt, time::Duration};

use async_native_tls::{Certificate, TlsConnector};
use async_std::net::TcpStream;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use futures_util::io::{AsyncReadExt, AsyncWriteExt};
use makosh_mail_api::{
    GMAIL_API_HOST, GMAIL_API_HTTPS_PORT, GmailOAuthAuthorityV1, GmailOAuthConfigurationV1,
    GmailOAuthEndpointV1, valid_ca_certificate_pem, valid_gmail_oauth_configuration,
};
use serde::Deserialize;

const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const MAX_MESSAGE_IDS: usize = 500;
const MAX_LABEL_IDS: usize = 512;
const MAX_BEARER_TOKEN_BYTES: usize = 16 * 1024;
const MAX_OAUTH_RESPONSE_BYTES: usize = 1024 * 1024;
const GMAIL_OPERATION_TIMEOUT: Duration = Duration::from_secs(15);
const GMAIL_OPERATIONAL_OAUTH_SCOPES: [&str; 5] = [
    "openid",
    "email",
    "https://www.googleapis.com/auth/gmail.modify",
    "https://www.googleapis.com/auth/gmail.send",
    "https://www.googleapis.com/auth/contacts",
];
const GMAIL_PERMANENT_DELETE_OAUTH_SCOPES: [&str; 3] =
    ["openid", "email", "https://mail.google.com/"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailApiClientV1 {
    host: String,
    port: u16,
    ca_certificate_pem: Option<String>,
    user_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GmailMutableMessageFlagV1 {
    Read,
    Starred,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailMessageLocationV1 {
    pub label_ids: Vec<String>,
}

pub fn decode_raw_rfc822(raw: &str) -> Result<Vec<u8>, GmailAdapterErrorV1> {
    if raw.is_empty() || raw.len() > MAX_RESPONSE_BYTES * 2 {
        return Err(GmailAdapterErrorV1::InvalidResponse);
    }
    URL_SAFE_NO_PAD
        .decode(raw)
        .map_err(|_| GmailAdapterErrorV1::InvalidResponse)
}

#[derive(Clone, Eq, PartialEq)]
pub struct GmailAuthorizationCodeExchangeV1 {
    pub configuration: GmailOAuthConfigurationV1,
    pub authorization_code: String,
    pub code_verifier: String,
}

#[derive(Clone, Eq, PartialEq)]
pub struct GmailRefreshTokenRequestV1 {
    pub configuration: GmailOAuthConfigurationV1,
    pub refresh_token: String,
}

#[derive(Clone, Eq, PartialEq, Deserialize)]
pub struct GmailOAuthTokenResponseV1 {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: Option<String>,
    pub scope: Option<String>,
}

impl fmt::Debug for GmailAuthorizationCodeExchangeV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GmailAuthorizationCodeExchangeV1")
            .field("configuration", &self.configuration)
            .field("authorization_code", &"[redacted]")
            .field("code_verifier", &"[redacted]")
            .finish()
    }
}

impl fmt::Debug for GmailRefreshTokenRequestV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GmailRefreshTokenRequestV1")
            .field("configuration", &self.configuration)
            .field("refresh_token", &"[redacted]")
            .finish()
    }
}

impl fmt::Debug for GmailOAuthTokenResponseV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GmailOAuthTokenResponseV1")
            .field("access_token", &"[redacted]")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[redacted]"),
            )
            .field("expires_in", &self.expires_in)
            .field("token_type", &self.token_type)
            .field("scope", &self.scope)
            .finish()
    }
}

pub async fn exchange_authorization_code(
    request: &GmailAuthorizationCodeExchangeV1,
) -> Result<GmailOAuthTokenResponseV1, GmailAdapterErrorV1> {
    if !valid_gmail_oauth_configuration(&request.configuration) {
        return Err(GmailAdapterErrorV1::InvalidRequest);
    }
    let form = vec![
        ("grant_type", "authorization_code".to_owned()),
        ("code", request.authorization_code.clone()),
        ("client_id", request.configuration.client_id.clone()),
        ("redirect_uri", request.configuration.redirect_uri.clone()),
        ("code_verifier", request.code_verifier.clone()),
    ];
    request_oauth_token(&request.configuration.token_endpoint, &form).await
}

pub async fn refresh_access_token(
    request: &GmailRefreshTokenRequestV1,
) -> Result<GmailOAuthTokenResponseV1, GmailAdapterErrorV1> {
    if !valid_gmail_oauth_configuration(&request.configuration)
        || !valid_bearer_token(&request.refresh_token)
    {
        return Err(GmailAdapterErrorV1::InvalidRequest);
    }
    let form = vec![
        ("grant_type", "refresh_token".to_owned()),
        ("refresh_token", request.refresh_token.clone()),
        ("client_id", request.configuration.client_id.clone()),
    ];
    request_oauth_token(&request.configuration.token_endpoint, &form).await
}

pub fn gmail_authorization_url(
    configuration: &GmailOAuthConfigurationV1,
    state: &str,
    code_challenge: &str,
    authority: GmailOAuthAuthorityV1,
) -> Result<String, GmailAdapterErrorV1> {
    if !valid_gmail_oauth_configuration(configuration)
        || !valid_oauth_carrier(state)
        || !valid_oauth_carrier(code_challenge)
    {
        return Err(GmailAdapterErrorV1::InvalidRequest);
    }
    let endpoint = &configuration.authorization_endpoint;
    let scopes = gmail_oauth_scopes(authority).join(" ");
    let query = [
        ("client_id", configuration.client_id.as_str()),
        ("redirect_uri", configuration.redirect_uri.as_str()),
        ("response_type", "code"),
        ("code_challenge", code_challenge),
        ("code_challenge_method", "S256"),
        ("access_type", "offline"),
        ("prompt", "consent"),
        ("scope", scopes.as_str()),
        ("state", state),
    ]
    .into_iter()
    .map(|(name, value)| {
        Ok(format!(
            "{}={}",
            percent_encode(name)?,
            percent_encode(value)?
        ))
    })
    .collect::<Result<Vec<_>, GmailAdapterErrorV1>>()?
    .join("&");
    Ok(format!(
        "https://{}:{}{}?{query}",
        endpoint.host, endpoint.port, endpoint.path
    ))
}

#[must_use]
pub fn gmail_scope_authorizes(authority: GmailOAuthAuthorityV1, granted_scope: &str) -> bool {
    let granted = granted_scope
        .split_ascii_whitespace()
        .collect::<BTreeSet<_>>();
    gmail_oauth_scopes(authority)
        .iter()
        .all(|scope| granted.contains(scope))
}

fn gmail_oauth_scopes(authority: GmailOAuthAuthorityV1) -> &'static [&'static str] {
    match authority {
        GmailOAuthAuthorityV1::Operational => &GMAIL_OPERATIONAL_OAUTH_SCOPES,
        GmailOAuthAuthorityV1::PermanentDelete => &GMAIL_PERMANENT_DELETE_OAUTH_SCOPES,
    }
}

impl GmailApiClientV1 {
    pub fn new(user_id: impl Into<String>) -> Result<Self, GmailAdapterErrorV1> {
        Self::for_endpoint(GMAIL_API_HOST, GMAIL_API_HTTPS_PORT, None, user_id)
    }

    #[cfg(any(test, feature = "conformance-test-support"))]
    pub fn for_conformance_endpoint(
        host: impl Into<String>,
        port: u16,
        ca_certificate_pem: Option<String>,
        user_id: impl Into<String>,
    ) -> Result<Self, GmailAdapterErrorV1> {
        let host = host.into();
        if !matches!(host.as_str(), "127.0.0.1" | "localhost") {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        Self::for_endpoint(host, port, ca_certificate_pem, user_id)
    }

    fn for_endpoint(
        host: impl Into<String>,
        port: u16,
        ca_certificate_pem: Option<String>,
        user_id: impl Into<String>,
    ) -> Result<Self, GmailAdapterErrorV1> {
        let host = host.into();
        let user_id = user_id.into();
        if !valid_host(&host)
            || port == 0
            || !valid_provider_id(&user_id)
            || ca_certificate_pem
                .as_deref()
                .is_some_and(|value| !valid_ca_certificate_pem(value))
        {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        Ok(Self {
            host,
            port,
            ca_certificate_pem,
            user_id,
        })
    }

    pub async fn list_labels(
        &self,
        access_token: &str,
    ) -> Result<Vec<GmailLabelV1>, GmailAdapterErrorV1> {
        let response: GmailLabelsResponse = self
            .get(
                access_token,
                &format!("/gmail/v1/users/{}/labels", self.user_id),
            )
            .await?;
        Ok(response.labels.unwrap_or_default())
    }

    pub async fn list_messages(
        &self,
        access_token: &str,
        request: &GmailListMessagesRequestV1,
    ) -> Result<GmailMessagePageV1, GmailAdapterErrorV1> {
        if request.max_results == 0 || request.max_results > 500 {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        let mut query = vec![format!("maxResults={}", request.max_results)];
        if let Some(page_token) = request.page_token.as_deref() {
            query.push(format!("pageToken={}", percent_encode(page_token)?));
        }
        if let Some(filter) = request.query.as_deref() {
            query.push(format!("q={}", percent_encode(filter)?));
        }
        for label_id in &request.label_ids {
            query.push(format!("labelIds={}", percent_encode(label_id)?));
        }
        let path = format!(
            "/gmail/v1/users/{}/messages?{}",
            self.user_id,
            query.join("&")
        );
        let response: GmailListResponse = self.get(access_token, &path).await?;
        Ok(GmailMessagePageV1 {
            messages: response.messages.unwrap_or_default(),
            next_page_token: response.next_page_token,
        })
    }

    pub async fn fetch_raw_message(
        &self,
        access_token: &str,
        message_id: &str,
    ) -> Result<GmailRawMessageV1, GmailAdapterErrorV1> {
        let message_id = provider_id(message_id)?;
        self.get(
            access_token,
            &format!(
                "/gmail/v1/users/{}/messages/{message_id}?format=raw",
                self.user_id
            ),
        )
        .await
    }

    pub async fn list_history(
        &self,
        access_token: &str,
        start_history_id: &str,
        page_token: Option<&str>,
    ) -> Result<GmailHistoryPageV1, GmailAdapterErrorV1> {
        let mut query = vec![format!("startHistoryId={}", provider_id(start_history_id)?)];
        query.push("historyTypes=messageAdded".to_owned());
        query.push("historyTypes=labelAdded".to_owned());
        query.push("historyTypes=labelRemoved".to_owned());
        if let Some(page_token) = page_token {
            query.push(format!("pageToken={}", percent_encode(page_token)?));
        }
        self.get(
            access_token,
            &format!(
                "/gmail/v1/users/{}/history?{}",
                self.user_id,
                query.join("&")
            ),
        )
        .await
    }

    pub async fn send_raw_message(
        &self,
        access_token: &str,
        rfc822: &[u8],
        thread_id: Option<&str>,
    ) -> Result<GmailSentMessageV1, GmailAdapterErrorV1> {
        if rfc822.is_empty() || rfc822.len() > MAX_RESPONSE_BYTES {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        let mut body = serde_json::json!({ "raw": URL_SAFE_NO_PAD.encode(rfc822) });
        if let Some(thread_id) = thread_id {
            body["threadId"] = serde_json::Value::String(provider_id(thread_id)?);
        }
        self.request_json(
            access_token,
            "POST",
            &format!("/gmail/v1/users/{}/messages/send", self.user_id),
            Some(body.to_string().as_bytes()),
        )
        .await
    }

    pub async fn batch_modify(
        &self,
        access_token: &str,
        request: &GmailBatchModifyRequestV1,
    ) -> Result<(), GmailAdapterErrorV1> {
        if request.message_ids.is_empty() || request.message_ids.len() > MAX_MESSAGE_IDS {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        let message_ids = request
            .message_ids
            .iter()
            .map(|id| provider_id(id))
            .collect::<Result<Vec<_>, _>>()?;
        let add_label_ids = request
            .add_label_ids
            .iter()
            .map(|id| provider_id(id))
            .collect::<Result<Vec<_>, _>>()?;
        let remove_label_ids = request
            .remove_label_ids
            .iter()
            .map(|id| provider_id(id))
            .collect::<Result<Vec<_>, _>>()?;
        if add_label_ids.is_empty() && remove_label_ids.is_empty() {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        let body = serde_json::json!({
            "ids": message_ids,
            "addLabelIds": add_label_ids,
            "removeLabelIds": remove_label_ids,
        });
        let _: serde_json::Value = self
            .request_json(
                access_token,
                "POST",
                &format!("/gmail/v1/users/{}/messages/batchModify", self.user_id),
                Some(body.to_string().as_bytes()),
            )
            .await?;
        Ok(())
    }

    pub async fn set_message_flag(
        &self,
        access_token: &str,
        provider_message_id: &str,
        flag: GmailMutableMessageFlagV1,
        target_value: bool,
    ) -> Result<(), GmailAdapterErrorV1> {
        let (add_label_ids, remove_label_ids) = gmail_labels_for_message_flag(flag, target_value);
        self.batch_modify(
            access_token,
            &GmailBatchModifyRequestV1 {
                message_ids: vec![provider_message_id.to_owned()],
                add_label_ids,
                remove_label_ids,
            },
        )
        .await
    }

    pub async fn archive_message(
        &self,
        access_token: &str,
        provider_message_id: &str,
    ) -> Result<GmailMessageLocationV1, GmailAdapterErrorV1> {
        self.batch_modify(
            access_token,
            &GmailBatchModifyRequestV1 {
                message_ids: vec![provider_message_id.to_owned()],
                add_label_ids: Vec::new(),
                remove_label_ids: vec!["INBOX".to_owned()],
            },
        )
        .await?;
        self.fetch_message_location(access_token, provider_message_id)
            .await
    }

    pub async fn trash_message(
        &self,
        access_token: &str,
        provider_message_id: &str,
    ) -> Result<GmailMessageLocationV1, GmailAdapterErrorV1> {
        self.post_message_action(access_token, provider_message_id, "trash")
            .await?;
        self.fetch_message_location(access_token, provider_message_id)
            .await
    }

    pub async fn restore_message(
        &self,
        access_token: &str,
        provider_message_id: &str,
    ) -> Result<GmailMessageLocationV1, GmailAdapterErrorV1> {
        self.post_message_action(access_token, provider_message_id, "untrash")
            .await?;
        self.fetch_message_location(access_token, provider_message_id)
            .await
    }

    pub async fn move_message(
        &self,
        access_token: &str,
        provider_message_id: &str,
        target_label_id: &str,
        target_is_inbox: bool,
    ) -> Result<GmailMessageLocationV1, GmailAdapterErrorV1> {
        let target_label_id = provider_id(target_label_id)?;
        self.batch_modify(
            access_token,
            &GmailBatchModifyRequestV1 {
                message_ids: vec![provider_message_id.to_owned()],
                add_label_ids: vec![target_label_id],
                remove_label_ids: if target_is_inbox {
                    Vec::new()
                } else {
                    vec!["INBOX".to_owned()]
                },
            },
        )
        .await?;
        self.fetch_message_location(access_token, provider_message_id)
            .await
    }

    pub async fn permanently_delete_message(
        &self,
        access_token: &str,
        provider_message_id: &str,
    ) -> Result<(), GmailAdapterErrorV1> {
        let provider_message_id = provider_id(provider_message_id)?;
        let status = async_std::future::timeout(
            GMAIL_OPERATION_TIMEOUT,
            self.request_status_inner(
                access_token,
                "DELETE",
                &format!(
                    "/gmail/v1/users/{}/messages/{provider_message_id}",
                    self.user_id
                ),
                None,
            ),
        )
        .await
        .map_err(|_| GmailAdapterErrorV1::Transport)??;
        match status {
            204 | 404 => Ok(()),
            status => Err(GmailAdapterErrorV1::ProviderStatus(status)),
        }
    }

    async fn post_message_action(
        &self,
        access_token: &str,
        provider_message_id: &str,
        action: &str,
    ) -> Result<(), GmailAdapterErrorV1> {
        let provider_message_id = provider_id(provider_message_id)?;
        if !matches!(action, "trash" | "untrash") {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        let _: serde_json::Value = self
            .request_json(
                access_token,
                "POST",
                &format!(
                    "/gmail/v1/users/{}/messages/{provider_message_id}/{action}",
                    self.user_id
                ),
                None,
            )
            .await?;
        Ok(())
    }

    async fn fetch_message_location(
        &self,
        access_token: &str,
        provider_message_id: &str,
    ) -> Result<GmailMessageLocationV1, GmailAdapterErrorV1> {
        let provider_message_id = provider_id(provider_message_id)?;
        let message: GmailRawMessageV1 = self
            .get(
                access_token,
                &format!(
                    "/gmail/v1/users/{}/messages/{provider_message_id}?format=minimal",
                    self.user_id
                ),
            )
            .await?;
        if message.id.as_deref() != Some(provider_message_id.as_str()) {
            return Err(GmailAdapterErrorV1::InvalidResponse);
        }
        let label_ids = message.label_ids.unwrap_or_default();
        if label_ids.len() > MAX_LABEL_IDS {
            return Err(GmailAdapterErrorV1::InvalidResponse);
        }
        let mut labels = BTreeSet::new();
        for label_id in label_ids {
            labels
                .insert(provider_id(&label_id).map_err(|_| GmailAdapterErrorV1::InvalidResponse)?);
        }
        Ok(GmailMessageLocationV1 {
            label_ids: labels.into_iter().collect(),
        })
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        access_token: &str,
        path: &str,
    ) -> Result<T, GmailAdapterErrorV1> {
        self.request_json(access_token, "GET", path, None).await
    }

    async fn request_json<T: for<'de> Deserialize<'de>>(
        &self,
        access_token: &str,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
    ) -> Result<T, GmailAdapterErrorV1> {
        async_std::future::timeout(
            GMAIL_OPERATION_TIMEOUT,
            self.request_json_inner(access_token, method, path, body),
        )
        .await
        .map_err(|_| GmailAdapterErrorV1::Transport)?
    }

    async fn request_json_inner<T: for<'de> Deserialize<'de>>(
        &self,
        access_token: &str,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
    ) -> Result<T, GmailAdapterErrorV1> {
        let response = self
            .request_raw_inner(access_token, method, path, body)
            .await?;
        parse_json_response(&response)
    }

    async fn request_status_inner(
        &self,
        access_token: &str,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
    ) -> Result<u16, GmailAdapterErrorV1> {
        let response = self
            .request_raw_inner(access_token, method, path, body)
            .await?;
        parse_response_status(&response)
    }

    async fn request_raw_inner(
        &self,
        access_token: &str,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
    ) -> Result<Vec<u8>, GmailAdapterErrorV1> {
        if !valid_bearer_token(access_token)
            || !path.starts_with('/')
            || path.contains('\r')
            || path.contains('\n')
        {
            return Err(GmailAdapterErrorV1::InvalidRequest);
        }
        let stream = TcpStream::connect((self.host.as_str(), self.port))
            .await
            .map_err(|_| GmailAdapterErrorV1::Transport)?;
        let connector = self
            .ca_certificate_pem
            .as_deref()
            .map(|pem| {
                Certificate::from_pem(pem.as_bytes())
                    .map(|certificate| TlsConnector::new().add_root_certificate(certificate))
                    .map_err(|_| GmailAdapterErrorV1::InvalidRequest)
            })
            .transpose()?
            .unwrap_or_default();
        let mut stream = connector
            .connect(self.host.as_str(), stream)
            .await
            .map_err(|_| GmailAdapterErrorV1::Transport)?;
        let body = body.unwrap_or_default();
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {access_token}\r\nAccept: application/json\r\nAccept-Encoding: identity\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.host,
            body.len(),
        );
        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|_| GmailAdapterErrorV1::Transport)?;
        if !body.is_empty() {
            stream
                .write_all(body)
                .await
                .map_err(|_| GmailAdapterErrorV1::Transport)?;
        }
        stream
            .flush()
            .await
            .map_err(|_| GmailAdapterErrorV1::Transport)?;
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .map_err(|_| GmailAdapterErrorV1::Transport)?;
        if response.len() > MAX_RESPONSE_BYTES {
            return Err(GmailAdapterErrorV1::InvalidResponse);
        }
        Ok(response)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailListMessagesRequestV1 {
    pub max_results: u16,
    pub page_token: Option<String>,
    pub query: Option<String>,
    pub label_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailListedMessageV1 {
    pub id: String,
    pub thread_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailMessagePageV1 {
    pub messages: Vec<GmailListedMessageV1>,
    pub next_page_token: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailRawMessageV1 {
    pub id: Option<String>,
    pub thread_id: Option<String>,
    pub label_ids: Option<Vec<String>>,
    pub history_id: Option<String>,
    pub internal_date: Option<String>,
    pub raw: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct GmailLabelV1 {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub label_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailBatchModifyRequestV1 {
    pub message_ids: Vec<String>,
    pub add_label_ids: Vec<String>,
    pub remove_label_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailSentMessageV1 {
    pub id: Option<String>,
    pub thread_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryPageV1 {
    pub history: Option<Vec<GmailHistoryItemV1>>,
    pub history_id: Option<String>,
    pub next_page_token: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmailHistoryItemV1 {
    pub messages_added: Option<Vec<GmailHistoryMessageAddedV1>>,
    pub labels_added: Option<Vec<GmailHistoryMessageAddedV1>>,
    pub labels_removed: Option<Vec<GmailHistoryMessageAddedV1>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct GmailHistoryMessageAddedV1 {
    pub message: GmailHistoryMessageV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct GmailHistoryMessageV1 {
    pub id: String,
}

pub fn history_message_ids(page: &GmailHistoryPageV1) -> Vec<String> {
    let mut message_ids = BTreeSet::new();
    for item in page.history.as_deref().unwrap_or_default() {
        for changes in [
            &item.messages_added,
            &item.labels_added,
            &item.labels_removed,
        ] {
            for change in changes.as_deref().unwrap_or_default() {
                if valid_provider_id(&change.message.id) {
                    message_ids.insert(change.message.id.clone());
                }
            }
        }
    }
    message_ids.into_iter().collect()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailLabelsResponse {
    labels: Option<Vec<GmailLabelV1>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GmailListResponse {
    messages: Option<Vec<GmailListedMessageV1>>,
    next_page_token: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GmailAdapterErrorV1 {
    InvalidRequest,
    Transport,
    ProviderStatus(u16),
    InvalidResponse,
}

impl fmt::Display for GmailAdapterErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for GmailAdapterErrorV1 {}

fn parse_json_response<T: for<'de> Deserialize<'de>>(
    response: &[u8],
) -> Result<T, GmailAdapterErrorV1> {
    if response.len() > MAX_RESPONSE_BYTES {
        return Err(GmailAdapterErrorV1::InvalidResponse);
    }
    let split = response
        .windows(4)
        .position(|value| value == b"\r\n\r\n")
        .ok_or(GmailAdapterErrorV1::InvalidResponse)?;
    let status = parse_response_status(response)?;
    if !(200..300).contains(&status) {
        return Err(GmailAdapterErrorV1::ProviderStatus(status));
    }
    serde_json::from_slice(&response[split + 4..]).map_err(|_| GmailAdapterErrorV1::InvalidResponse)
}

fn parse_response_status(response: &[u8]) -> Result<u16, GmailAdapterErrorV1> {
    if response.len() > MAX_RESPONSE_BYTES {
        return Err(GmailAdapterErrorV1::InvalidResponse);
    }
    let split = response
        .windows(4)
        .position(|value| value == b"\r\n\r\n")
        .ok_or(GmailAdapterErrorV1::InvalidResponse)?;
    let headers = std::str::from_utf8(&response[..split])
        .map_err(|_| GmailAdapterErrorV1::InvalidResponse)?;
    headers
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or(GmailAdapterErrorV1::InvalidResponse)
}

async fn request_oauth_token(
    token_endpoint: &GmailOAuthEndpointV1,
    form: &[(&str, String)],
) -> Result<GmailOAuthTokenResponseV1, GmailAdapterErrorV1> {
    async_std::future::timeout(
        GMAIL_OPERATION_TIMEOUT,
        request_oauth_token_inner(token_endpoint, form),
    )
    .await
    .map_err(|_| GmailAdapterErrorV1::Transport)?
}

async fn request_oauth_token_inner(
    token_endpoint: &GmailOAuthEndpointV1,
    form: &[(&str, String)],
) -> Result<GmailOAuthTokenResponseV1, GmailAdapterErrorV1> {
    if form
        .iter()
        .any(|(name, value)| name.is_empty() || value.trim().is_empty() || value.len() > 8192)
    {
        return Err(GmailAdapterErrorV1::InvalidRequest);
    }
    let body = form
        .iter()
        .map(|(name, value)| {
            let name = percent_encode(name)?;
            let value = percent_encode(value)?;
            Ok(format!("{name}={value}"))
        })
        .collect::<Result<Vec<_>, GmailAdapterErrorV1>>()?
        .join("&");
    let stream = TcpStream::connect((token_endpoint.host.as_str(), token_endpoint.port))
        .await
        .map_err(|_| GmailAdapterErrorV1::Transport)?;
    let connector = token_endpoint
        .ca_certificate_pem
        .as_deref()
        .map(|pem| {
            Certificate::from_pem(pem.as_bytes())
                .map(|certificate| TlsConnector::new().add_root_certificate(certificate))
                .map_err(|_| GmailAdapterErrorV1::InvalidRequest)
        })
        .transpose()?
        .unwrap_or_default();
    let mut stream = connector
        .connect(token_endpoint.host.as_str(), stream)
        .await
        .map_err(|_| GmailAdapterErrorV1::Transport)?;
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nAccept-Encoding: identity\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        token_endpoint.path,
        token_endpoint.host,
        body.len(),
    );
    stream
        .write_all(request.as_bytes())
        .await
        .map_err(|_| GmailAdapterErrorV1::Transport)?;
    stream
        .write_all(body.as_bytes())
        .await
        .map_err(|_| GmailAdapterErrorV1::Transport)?;
    stream
        .flush()
        .await
        .map_err(|_| GmailAdapterErrorV1::Transport)?;
    let mut response = Vec::new();
    stream
        .take(u64::try_from(MAX_OAUTH_RESPONSE_BYTES + 1).unwrap_or(u64::MAX))
        .read_to_end(&mut response)
        .await
        .map_err(|_| GmailAdapterErrorV1::Transport)?;
    if response.len() > MAX_OAUTH_RESPONSE_BYTES {
        return Err(GmailAdapterErrorV1::InvalidResponse);
    }
    let token: GmailOAuthTokenResponseV1 = parse_json_response(&response)?;
    if !valid_bearer_token(&token.access_token)
        || token
            .refresh_token
            .as_deref()
            .is_some_and(|value| !valid_bearer_token(value))
        || token.expires_in == 0
        || token
            .token_type
            .as_deref()
            .is_some_and(|value| value != "Bearer")
    {
        return Err(GmailAdapterErrorV1::InvalidResponse);
    }
    Ok(token)
}

fn valid_host(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && host
            .bytes()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, b'.' | b'-'))
}
fn valid_provider_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
}
fn valid_bearer_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_BEARER_TOKEN_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'+' | b'/' | b'=')
        })
}
fn valid_oauth_carrier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 8192
        && value.is_ascii()
        && !value.contains(['\r', '\n', '\0'])
}
fn provider_id(value: &str) -> Result<String, GmailAdapterErrorV1> {
    valid_provider_id(value)
        .then(|| value.to_owned())
        .ok_or(GmailAdapterErrorV1::InvalidRequest)
}

fn gmail_labels_for_message_flag(
    flag: GmailMutableMessageFlagV1,
    target_value: bool,
) -> (Vec<String>, Vec<String>) {
    match (flag, target_value) {
        (GmailMutableMessageFlagV1::Read, true) => (Vec::new(), vec!["UNREAD".to_owned()]),
        (GmailMutableMessageFlagV1::Read, false) => (vec!["UNREAD".to_owned()], Vec::new()),
        (GmailMutableMessageFlagV1::Starred, true) => (vec!["STARRED".to_owned()], Vec::new()),
        (GmailMutableMessageFlagV1::Starred, false) => (Vec::new(), vec!["STARRED".to_owned()]),
    }
}
fn percent_encode(value: &str) -> Result<String, GmailAdapterErrorV1> {
    if value.len() > 4096 || value.contains('\r') || value.contains('\n') {
        return Err(GmailAdapterErrorV1::InvalidRequest);
    }
    Ok(value
        .bytes()
        .flat_map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                vec![char::from(byte)]
            } else {
                format!("%{byte:02X}").chars().collect()
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oauth_configuration() -> GmailOAuthConfigurationV1 {
        GmailOAuthConfigurationV1 {
            client_id: "client-id.apps.googleusercontent.com".to_owned(),
            redirect_uri: "http://127.0.0.1:38123/oauth/callback".to_owned(),
            authorization_endpoint: GmailOAuthEndpointV1 {
                host: makosh_mail_api::GMAIL_OAUTH_AUTHORIZATION_HOST.to_owned(),
                port: makosh_mail_api::GMAIL_OAUTH_HTTPS_PORT,
                path: makosh_mail_api::GMAIL_OAUTH_AUTHORIZATION_PATH.to_owned(),
                ca_certificate_pem: None,
            },
            token_endpoint: GmailOAuthEndpointV1 {
                host: makosh_mail_api::GMAIL_OAUTH_TOKEN_HOST.to_owned(),
                port: makosh_mail_api::GMAIL_OAUTH_HTTPS_PORT,
                path: makosh_mail_api::GMAIL_OAUTH_TOKEN_PATH.to_owned(),
                ca_certificate_pem: None,
            },
        }
    }
    #[test]
    fn parses_a_bounded_success_response() {
        let value: GmailLabelsResponse =
            parse_json_response(b"HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\n{\"labels\":[]}")
                .expect("response");
        assert!(value.labels.unwrap_or_default().is_empty());
    }
    #[test]
    fn rejects_non_success_status() {
        let value: Result<serde_json::Value, _> =
            parse_json_response(b"HTTP/1.1 401 Unauthorized\r\n\r\n{}");
        assert_eq!(value, Err(GmailAdapterErrorV1::ProviderStatus(401)));
    }
    #[test]
    fn percent_encodes_query_values() {
        assert_eq!(
            percent_encode("label:inbox hello").expect("encoded"),
            "label%3Ainbox%20hello"
        );
    }
    #[test]
    fn authorization_url_uses_fixed_scopes_and_pkce() {
        let url = gmail_authorization_url(
            &oauth_configuration(),
            "state-value",
            "challenge-value",
            GmailOAuthAuthorityV1::Operational,
        )
        .expect("authorization URL");
        assert!(url.starts_with("https://accounts.google.com:443/o/oauth2/v2/auth?"));
        assert!(url.contains("code_challenge=challenge-value"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("state=state-value"));
        assert!(url.contains("gmail.modify"));
        assert!(url.contains("gmail.send"));
        assert!(url.contains("auth%2Fcontacts"));
        assert!(!url.contains("gmail.readonly"));
        assert!(!url.contains("client_secret"));
    }
    #[test]
    fn permanent_delete_requires_the_explicit_full_mail_scope() {
        let url = gmail_authorization_url(
            &oauth_configuration(),
            "state-value",
            "challenge-value",
            GmailOAuthAuthorityV1::PermanentDelete,
        )
        .expect("authorization URL");
        assert!(url.contains("mail.google.com"));
        assert!(!url.contains("gmail.modify"));
        assert!(gmail_scope_authorizes(
            GmailOAuthAuthorityV1::PermanentDelete,
            "openid email https://mail.google.com/"
        ));
        assert!(!gmail_scope_authorizes(
            GmailOAuthAuthorityV1::PermanentDelete,
            "openid email https://www.googleapis.com/auth/gmail.modify"
        ));
    }
    #[test]
    fn delete_status_accepts_only_success_or_replay_convergence() {
        for status in [204, 404] {
            assert_eq!(
                parse_response_status(
                    format!("HTTP/1.1 {status} Result\r\nContent-Length: 0\r\n\r\n").as_bytes()
                ),
                Ok(status)
            );
        }
        assert_eq!(
            parse_response_status(b"HTTP/1.1 403 Forbidden\r\n\r\n"),
            Ok(403)
        );
    }
    #[test]
    fn oauth_adapter_debug_redacts_every_credential_carrier() {
        let exchange = GmailAuthorizationCodeExchangeV1 {
            configuration: oauth_configuration(),
            authorization_code: "authorization-code-secret".to_owned(),
            code_verifier: "pkce-verifier-secret".to_owned(),
        };
        let refresh = GmailRefreshTokenRequestV1 {
            configuration: oauth_configuration(),
            refresh_token: "refresh-token-secret".to_owned(),
        };
        let response = GmailOAuthTokenResponseV1 {
            access_token: "access-token-secret".to_owned(),
            refresh_token: Some("rotated-refresh-secret".to_owned()),
            expires_in: 3600,
            token_type: Some("Bearer".to_owned()),
            scope: None,
        };
        let debug = format!("{exchange:?}\n{refresh:?}\n{response:?}");
        for secret in [
            "authorization-code-secret",
            "pkce-verifier-secret",
            "refresh-token-secret",
            "access-token-secret",
            "rotated-refresh-secret",
        ] {
            assert!(!debug.contains(secret));
        }
    }
    #[test]
    fn conformance_client_requires_a_bounded_tls_endpoint() {
        let client = GmailApiClientV1::for_conformance_endpoint("localhost", 19_443, None, "me")
            .expect("loopback Gmail endpoint");
        assert_eq!(client.host, "localhost");
        assert_eq!(client.port, 19_443);
        assert_eq!(
            GmailApiClientV1::for_conformance_endpoint("localhost", 0, None, "me"),
            Err(GmailAdapterErrorV1::InvalidRequest)
        );
        assert_eq!(
            GmailApiClientV1::for_conformance_endpoint("gmail.example.test", 19_443, None, "me",),
            Err(GmailAdapterErrorV1::InvalidRequest)
        );
        assert_eq!(
            GmailApiClientV1::for_conformance_endpoint(
                "localhost",
                19_443,
                Some("not a certificate".to_owned()),
                "me",
            ),
            Err(GmailAdapterErrorV1::InvalidRequest)
        );
    }
    #[test]
    fn bearer_tokens_are_header_safe_and_bounded() {
        assert!(valid_bearer_token("token-._~+/="));
        assert!(!valid_bearer_token(""));
        assert!(!valid_bearer_token("token\r\nInjected: value"));
        assert!(!valid_bearer_token(&"a".repeat(MAX_BEARER_TOKEN_BYTES + 1)));
    }
    #[test]
    fn history_collects_unique_valid_message_ids_from_supported_change_families() {
        let page = GmailHistoryPageV1 {
            history: Some(vec![GmailHistoryItemV1 {
                messages_added: Some(vec![GmailHistoryMessageAddedV1 {
                    message: GmailHistoryMessageV1 {
                        id: "message-2".into(),
                    },
                }]),
                labels_added: Some(vec![GmailHistoryMessageAddedV1 {
                    message: GmailHistoryMessageV1 {
                        id: "message-1".into(),
                    },
                }]),
                labels_removed: Some(vec![
                    GmailHistoryMessageAddedV1 {
                        message: GmailHistoryMessageV1 {
                            id: "message-2".into(),
                        },
                    },
                    GmailHistoryMessageAddedV1 {
                        message: GmailHistoryMessageV1 {
                            id: "invalid id".into(),
                        },
                    },
                ]),
            }]),
            history_id: Some("42".into()),
            next_page_token: None,
        };
        assert_eq!(history_message_ids(&page), vec!["message-1", "message-2"]);
    }

    #[test]
    fn message_flag_mapping_is_convergent_and_provider_owned() {
        assert_eq!(
            gmail_labels_for_message_flag(GmailMutableMessageFlagV1::Read, true),
            (Vec::new(), vec!["UNREAD".to_owned()])
        );
        assert_eq!(
            gmail_labels_for_message_flag(GmailMutableMessageFlagV1::Starred, false),
            (Vec::new(), vec!["STARRED".to_owned()])
        );
    }
}
