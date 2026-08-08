use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, TimeDelta, Utc};
use getrandom::getrandom;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZoomProtocolError {
    #[error("invalid Zoom request: {0}")]
    InvalidRequest(String),
}

pub const DEFAULT_ZOOM_AUTHORIZATION_ENDPOINT: &str = "https://zoom.us/oauth/authorize";
pub const DEFAULT_ZOOM_TOKEN_ENDPOINT: &str = "https://zoom.us/oauth/token";
pub const ZOOM_TOKEN_EXPIRY_SAFETY_MARGIN_SECONDS: i64 = 60;
pub const ZOOM_PROVIDER_KIND_STR: &str = "zoom_user";
pub const ZOOM_RUNTIME_KIND: &str = "zoom_fixture_runtime";
pub const ZOOM_LIVE_AUTHORIZED_RUNTIME_KIND: &str = "zoom_live_authorized_runtime";
pub const DEFAULT_ZOOM_API_BASE_URL: &str = "https://api.zoom.us/v2";
pub const ZOOM_EXPLICIT_TOKEN_REFRESH_THRESHOLD_SECONDS: i64 = 60;
pub const ZOOM_TOKEN_MAINTENANCE_REFRESH_THRESHOLD_SECONDS: i64 = 300;
pub const ZOOM_MAX_TOKEN_REFRESH_THRESHOLD_SECONDS: i64 = 86_400;
pub const ZOOM_TOKEN_ROTATION_REQUIRED_BLOCKER: &str = "zoom_token_rotation_required";
pub const ZOOM_PROVIDER_SYNC_DEFAULT_PAGE_SIZE: usize = 30;
pub const ZOOM_PROVIDER_SYNC_MAX_PAGE_SIZE: usize = 100;
pub const ZOOM_PROVIDER_SYNC_DEFAULT_MAX_MEETINGS: usize = 100;
pub const ZOOM_PROVIDER_SYNC_MAX_MEETINGS: usize = 500;
pub const ZOOM_MAX_RECORDING_MEDIA_DOWNLOAD_BYTES: usize = 268_435_456;
pub const ZOOM_DEFAULT_WEBHOOK_SUBSCRIPTION_NAME: &str = "Макошь Zoom Runtime";
pub const ZOOM_DEFAULT_WEBHOOK_EVENT_TYPES: &[&str] = &[
    "meeting.started",
    "meeting.ended",
    "meeting.participant_joined",
    "meeting.participant_left",
    "recording.completed",
];

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoomAuthShape {
    Fixture,
    #[serde(rename = "oauth_user")]
    #[default]
    OAuthUser,
    ServerToServer,
}

impl ZoomAuthShape {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::OAuthUser => "oauth_user",
            Self::ServerToServer => "server_to_server",
        }
    }
}

pub fn validate_non_empty(field: &'static str, value: &str) -> Result<String, ZoomProtocolError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ZoomProtocolError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

pub fn validate_object(field: &'static str, value: &Value) -> Result<(), ZoomProtocolError> {
    if !value.is_object() {
        return Err(ZoomProtocolError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}

pub fn validate_array(field: &'static str, value: &Value) -> Result<(), ZoomProtocolError> {
    if !value.is_array() {
        return Err(ZoomProtocolError::InvalidRequest(format!(
            "{field} must be a JSON array"
        )));
    }
    Ok(())
}

pub fn zoom_oauth_token_secret_ref(account_id: &str) -> String {
    format!(
        "secret:provider-account:{}:zoom_oauth_token",
        account_id.trim()
    )
}

pub fn zoom_client_secret_ref(account_id: &str) -> String {
    format!(
        "secret:provider-account:{}:zoom_client_secret",
        account_id.trim()
    )
}

pub fn zoom_oauth_expires_at(expires_in: Option<i64>) -> DateTime<Utc> {
    let seconds = expires_in
        .unwrap_or(3600)
        .saturating_sub(ZOOM_TOKEN_EXPIRY_SAFETY_MARGIN_SECONDS)
        .max(ZOOM_TOKEN_EXPIRY_SAFETY_MARGIN_SECONDS);
    Utc::now() + TimeDelta::seconds(seconds)
}

pub fn random_zoom_oauth_token() -> Result<String, ZoomProtocolError> {
    let mut bytes = [0_u8; 32];
    getrandom(&mut bytes).map_err(|_| {
        ZoomProtocolError::InvalidRequest("failed to generate Zoom OAuth state token".to_owned())
    })?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub fn zoom_authorization_url(
    authorization_endpoint: &str,
    client_id: &str,
    redirect_uri: &str,
    scopes: &[String],
    state: &str,
) -> Result<String, ZoomProtocolError> {
    validate_non_empty("authorization_endpoint", authorization_endpoint)?;
    validate_non_empty("client_id", client_id)?;
    validate_non_empty("redirect_uri", redirect_uri)?;
    validate_non_empty("state", state)?;
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    serializer
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id.trim())
        .append_pair("redirect_uri", redirect_uri.trim())
        .append_pair("state", state.trim());
    let scopes = normalized_scopes(scopes);
    if !scopes.is_empty() {
        serializer.append_pair("scope", &scopes.join(" "));
    }
    Ok(format!(
        "{}?{}",
        authorization_endpoint.trim(),
        serializer.finish()
    ))
}

pub fn normalized_scopes(scopes: &[String]) -> Vec<String> {
    scopes
        .iter()
        .map(|scope| scope.trim())
        .filter(|scope| !scope.is_empty())
        .map(str::to_owned)
        .collect()
}

pub fn sanitize_zoom_payload(mut payload: Value) -> Value {
    remove_secret_like_fields(&mut payload);
    payload
}

fn remove_secret_like_fields(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|key, _| !is_secret_like_key(key));
            for child in map.values_mut() {
                remove_secret_like_fields(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                remove_secret_like_fields(item);
            }
        }
        _ => {}
    }
}

fn is_secret_like_key(key: &str) -> bool {
    let normalized = key.trim().to_ascii_lowercase();
    normalized.contains("token")
        || normalized.contains("secret")
        || normalized.contains("password")
        || normalized == "authorization"
        || normalized == "api_key"
        || normalized == "apikey"
}
