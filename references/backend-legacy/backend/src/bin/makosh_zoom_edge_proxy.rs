use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::{Body, Bytes};
use axum::extract::{RawQuery, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use color_eyre::eyre::{Context, Result, eyre};
use reqwest::Client;
use serde::Serialize;
use tokio::net::TcpListener;
use tracing::Instrument;
use url::Url;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8788";
const DEFAULT_MAKOSH_BASE_URL: &str = "http://127.0.0.1:8080";
const PUBLIC_WEBHOOK_PATH: &str = "/webhooks/zoom";
const PROTECTED_MAKOSH_WEBHOOK_PATH: &str = "/api/v1/integrations/zoom/runtime-bridge/webhooks";
const PROTECTED_MAKOSH_CAPABILITIES_PATH: &str = "/api/v1/integrations/zoom/capabilities";
const MAKOSH_SECRET_HEADER: &str = "X-Макошь-Secret";
const ZOOM_SIGNATURE_HEADER: &str = "x-zm-signature";
const ZOOM_TIMESTAMP_HEADER: &str = "x-zm-request-timestamp";

#[derive(Clone)]
struct EdgeState {
    config: Arc<EdgeConfig>,
    client: Client,
}

#[derive(Clone, Debug)]
struct EdgeConfig {
    bind_addr: SocketAddr,
    makosh_base_url: Url,
    makosh_secret: String,
    account_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Debug, Serialize)]
struct EdgeManifestResponse {
    service: &'static str,
    public_webhook_path: &'static str,
    protected_makosh_webhook_path: &'static str,
    protected_makosh_capabilities_path: &'static str,
    local_auth_header: &'static str,
    signature_header: &'static str,
    timestamp_header: &'static str,
    post_forwarding: &'static str,
    payload_policy: &'static str,
    secret_policy: &'static str,
    configured_account_id: bool,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    makosh_hub_backend::app::router::init_tracing();
    let flow_id = std::env::var("MAKOSH_FLOW_ID").unwrap_or_else(|_| "zoom-edge-proxy".to_owned());
    let runtime_span = tracing::info_span!("makosh_zoom_edge_proxy", flow_id = %flow_id);

    async move {
        let config = Arc::new(EdgeConfig::from_env()?);
        let listener = TcpListener::bind(config.bind_addr)
            .await
            .with_context(|| format!("binding Zoom edge proxy on {}", config.bind_addr))?;
        tracing::info!(
            bind_addr = %config.bind_addr,
            public_webhook_path = PUBLIC_WEBHOOK_PATH,
            makosh_base_url = %config.makosh_base_url,
            "starting Zoom edge proxy"
        );
        axum::serve(listener, router(config)).await?;
        Ok(())
    }
    .instrument(runtime_span)
    .await
}

fn router(config: Arc<EdgeConfig>) -> Router {
    let state = EdgeState {
        config,
        client: Client::new(),
    };
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/manifest", get(edge_manifest))
        .route(PUBLIC_WEBHOOK_PATH, post(forward_zoom_webhook_post))
        .with_state(state)
}

async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "makosh-zoom-edge-proxy",
    })
}

async fn readyz(State(state): State<EdgeState>) -> Response {
    let url = match state
        .config
        .makosh_url(PROTECTED_MAKOSH_CAPABILITIES_PATH, None, false)
    {
        Ok(url) => url,
        Err(error) => {
            tracing::warn!(error = %error, "Макошь Zoom capabilities URL is invalid");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "makosh_zoom_capabilities_url_invalid",
                }),
            )
                .into_response();
        }
    };

    match state
        .client
        .get(url)
        .header(MAKOSH_SECRET_HEADER, state.config.makosh_secret.as_str())
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => Json(HealthResponse {
            status: "ready",
            service: "makosh-zoom-edge-proxy",
        })
        .into_response(),
        Ok(response) => sanitized_error_response(
            StatusCode::BAD_GATEWAY,
            "makosh_zoom_capabilities_unavailable",
            response.status(),
        ),
        Err(error) => {
            tracing::warn!(error = %error, "Макошь Zoom capabilities readiness check failed");
            (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "makosh_zoom_capabilities_unavailable",
                }),
            )
                .into_response()
        }
    }
}

async fn edge_manifest(State(state): State<EdgeState>) -> Json<EdgeManifestResponse> {
    Json(EdgeManifestResponse {
        service: "makosh-zoom-edge-proxy",
        public_webhook_path: PUBLIC_WEBHOOK_PATH,
        protected_makosh_webhook_path: PROTECTED_MAKOSH_WEBHOOK_PATH,
        protected_makosh_capabilities_path: PROTECTED_MAKOSH_CAPABILITIES_PATH,
        local_auth_header: MAKOSH_SECRET_HEADER,
        signature_header: ZOOM_SIGNATURE_HEADER,
        timestamp_header: ZOOM_TIMESTAMP_HEADER,
        post_forwarding: "forward_exact_raw_body_x_zm_signature_x_zm_timestamp_and_optional_account_id_to_protected_makosh",
        payload_policy: "post_body_is_not_parsed_or_rewritten_by_edge_proxy",
        secret_policy: "local_api_secret_is_env_only_and_never_returned",
        configured_account_id: state.config.account_id.is_some(),
    })
}

async fn forward_zoom_webhook_post(
    State(state): State<EdgeState>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let url =
        match state
            .config
            .makosh_url(PROTECTED_MAKOSH_WEBHOOK_PATH, raw_query.as_deref(), true)
        {
            Ok(url) => url,
            Err(error) => {
                tracing::warn!(error = %error, "Макошь Zoom webhook URL is invalid");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "makosh_zoom_webhook_url_invalid",
                    }),
                )
                    .into_response();
            }
        };

    let mut request = state
        .client
        .post(url)
        .header(MAKOSH_SECRET_HEADER, state.config.makosh_secret.as_str())
        .body(body);
    request = copy_header(request, &headers, header::CONTENT_TYPE.as_str());
    request = copy_header(request, &headers, ZOOM_SIGNATURE_HEADER);
    request = copy_header(request, &headers, ZOOM_TIMESTAMP_HEADER);
    forward_upstream_response(request.send().await).await
}

fn copy_header(
    request: reqwest::RequestBuilder,
    headers: &HeaderMap,
    name: &'static str,
) -> reqwest::RequestBuilder {
    if let Some(value) = headers.get(name).and_then(|value| value.to_str().ok()) {
        return request.header(name, value);
    }
    request
}

async fn forward_upstream_response(
    response: std::result::Result<reqwest::Response, reqwest::Error>,
) -> Response {
    match response {
        Ok(response) if response.status().is_success() => {
            let status = response_status(response.status());
            let content_type = response
                .headers()
                .get(header::CONTENT_TYPE.as_str())
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned);
            let body = match response.bytes().await {
                Ok(body) => body,
                Err(error) => {
                    tracing::warn!(error = %error, "failed to read successful Макошь Zoom webhook response");
                    return (
                        StatusCode::BAD_GATEWAY,
                        Json(ErrorResponse {
                            error: "makosh_response_read_failed",
                        }),
                    )
                        .into_response();
                }
            };
            let mut builder = Response::builder().status(status);
            if let Some(content_type) = content_type {
                builder = builder.header(header::CONTENT_TYPE, content_type);
            }
            builder
                .body(Body::from(body))
                .unwrap_or_else(|_| StatusCode::BAD_GATEWAY.into_response())
        }
        Ok(response) => sanitized_error_response(
            StatusCode::BAD_GATEWAY,
            "makosh_zoom_webhook_rejected",
            response.status(),
        ),
        Err(error) => {
            tracing::warn!(error = %error, "Макошь Zoom webhook forwarding failed");
            (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "makosh_zoom_webhook_unreachable",
                }),
            )
                .into_response()
        }
    }
}

fn sanitized_error_response(
    status: StatusCode,
    error: &'static str,
    upstream_status: reqwest::StatusCode,
) -> Response {
    tracing::warn!(
        upstream_status = upstream_status.as_u16(),
        "Макошь Zoom webhook proxy rejected request"
    );
    (status, Json(ErrorResponse { error })).into_response()
}

fn response_status(status: reqwest::StatusCode) -> StatusCode {
    StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
}

impl EdgeConfig {
    fn from_env() -> Result<Self> {
        let bind_addr = env_or_default("MAKOSH_ZOOM_EDGE_BIND_ADDR", DEFAULT_BIND_ADDR)
            .parse::<SocketAddr>()
            .context("invalid MAKOSH_ZOOM_EDGE_BIND_ADDR")?;
        let makosh_base_url =
            env_or_default("MAKOSH_ZOOM_EDGE_MAKOSH_BASE_URL", DEFAULT_MAKOSH_BASE_URL);
        let makosh_base_url = parse_base_url(&makosh_base_url)?;
        let makosh_secret = optional_env("MAKOSH_ZOOM_EDGE_MAKOSH_SECRET")
            .or_else(|| optional_env("MAKOSH_LOCAL_API_SECRET"))
            .ok_or_else(|| {
                eyre!("MAKOSH_ZOOM_EDGE_MAKOSH_SECRET or MAKOSH_LOCAL_API_SECRET must be set")
            })?;
        let account_id = optional_env("MAKOSH_ZOOM_EDGE_ACCOUNT_ID");

        Ok(Self {
            bind_addr,
            makosh_base_url,
            makosh_secret,
            account_id,
        })
    }

    fn makosh_url(
        &self,
        protected_path: &str,
        raw_query: Option<&str>,
        include_account_id: bool,
    ) -> Result<Url> {
        let path = protected_path.trim_start_matches('/');
        let mut url = self
            .makosh_base_url
            .join(path)
            .with_context(|| format!("joining Макошь path `{protected_path}`"))?;
        url.set_query(raw_query.filter(|value| !value.trim().is_empty()));
        if include_account_id && let Some(account_id) = &self.account_id {
            url.query_pairs_mut().append_pair("account_id", account_id);
        }
        Ok(url)
    }
}

fn parse_base_url(raw: &str) -> Result<Url> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(eyre!("MAKOSH_ZOOM_EDGE_MAKOSH_BASE_URL must not be empty"));
    }
    Url::parse(&format!("{trimmed}/")).with_context(|| "invalid MAKOSH_ZOOM_EDGE_MAKOSH_BASE_URL")
}

fn env_or_default(name: &str, default: &str) -> String {
    optional_env(name).unwrap_or_else(|| default.to_owned())
}

fn optional_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::routing::get;
    use serde_json::json;
    use tokio::task::JoinHandle;

    #[derive(Clone, Default)]
    struct FakeМакошьState {
        captured: Arc<tokio::sync::Mutex<FakeМакошьCaptured>>,
    }

    #[derive(Clone, Debug, Default)]
    struct FakeМакошьCaptured {
        capabilities_secret: Option<String>,
        webhook_query: Option<String>,
        webhook_secret: Option<String>,
        webhook_signature: Option<String>,
        webhook_timestamp: Option<String>,
        webhook_content_type: Option<String>,
        webhook_body: Vec<u8>,
    }

    #[tokio::test]
    async fn readyz_checks_zoom_capabilities_without_account_scoping() {
        let (makosh_addr, makosh_state, makosh_handle) = spawn_fake_makosh().await;
        let (edge_addr, edge_handle) = spawn_edge_proxy(makosh_addr, Some("zoom-account-1")).await;

        let response = reqwest::Client::new()
            .get(format!("http://{edge_addr}/readyz"))
            .send()
            .await
            .expect("readyz request should reach edge proxy");

        assert_eq!(response.status().as_u16(), StatusCode::OK.as_u16());
        let captured = makosh_state.captured.lock().await.clone();
        assert_eq!(captured.capabilities_secret.as_deref(), Some("edge-secret"));

        edge_handle.abort();
        makosh_handle.abort();
    }

    #[tokio::test]
    async fn post_webhook_forwards_raw_body_zoom_headers_account_scope_and_local_secret() {
        let (makosh_addr, makosh_state, makosh_handle) = spawn_fake_makosh().await;
        let (edge_addr, edge_handle) = spawn_edge_proxy(makosh_addr, Some("zoom-account-1")).await;
        let body = br#"{"event":"meeting.started","payload":{"object":{"id":"123"}}}"#;

        let response = reqwest::Client::new()
            .post(format!("http://{edge_addr}{PUBLIC_WEBHOOK_PATH}"))
            .header(ZOOM_SIGNATURE_HEADER, "v0=test-signature")
            .header(ZOOM_TIMESTAMP_HEADER, "1782500000")
            .header(header::CONTENT_TYPE.as_str(), "application/json")
            .body(body.as_slice().to_vec())
            .send()
            .await
            .expect("POST webhook should reach edge proxy");

        assert_eq!(response.status().as_u16(), StatusCode::ACCEPTED.as_u16());
        assert_eq!(
            response
                .text()
                .await
                .expect("response body should be readable"),
            r#"{"accepted":true}"#
        );

        let captured = makosh_state.captured.lock().await.clone();
        assert_eq!(captured.webhook_secret.as_deref(), Some("edge-secret"));
        assert_eq!(
            captured.webhook_signature.as_deref(),
            Some("v0=test-signature")
        );
        assert_eq!(captured.webhook_timestamp.as_deref(), Some("1782500000"));
        assert_eq!(
            captured.webhook_content_type.as_deref(),
            Some("application/json")
        );
        assert_eq!(captured.webhook_body, body);
        assert_eq!(
            captured.webhook_query.as_deref(),
            Some("account_id=zoom-account-1")
        );

        edge_handle.abort();
        makosh_handle.abort();
    }

    #[tokio::test]
    async fn post_webhook_preserves_existing_query_and_appends_configured_account_scope() {
        let (makosh_addr, makosh_state, makosh_handle) = spawn_fake_makosh().await;
        let (edge_addr, edge_handle) = spawn_edge_proxy(makosh_addr, Some("zoom-account-1")).await;

        let response = reqwest::Client::new()
            .post(format!(
                "http://{edge_addr}{PUBLIC_WEBHOOK_PATH}?source=zoom"
            ))
            .body(r#"{"event":"endpoint.url_validation","payload":{"plainToken":"abc"}}"#)
            .send()
            .await
            .expect("POST validation webhook should reach edge proxy");

        assert_eq!(response.status().as_u16(), StatusCode::ACCEPTED.as_u16());
        let captured = makosh_state.captured.lock().await.clone();
        let query = captured.webhook_query.expect("query should be forwarded");
        assert!(query.contains("source=zoom"));
        assert!(query.contains("account_id=zoom-account-1"));

        edge_handle.abort();
        makosh_handle.abort();
    }

    async fn spawn_edge_proxy(
        makosh_addr: SocketAddr,
        account_id: Option<&str>,
    ) -> (SocketAddr, JoinHandle<()>) {
        const TEST_MAKOSH_SHARED_KEY: &str = concat!("edge", "-", "secret");
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("edge proxy test listener should bind");
        let addr = listener
            .local_addr()
            .expect("edge proxy test listener should have local addr");
        let config = Arc::new(EdgeConfig {
            bind_addr: addr,
            makosh_base_url: Url::parse(&format!("http://{makosh_addr}/"))
                .expect("Макошь test URL should parse"),
            makosh_secret: TEST_MAKOSH_SHARED_KEY.to_owned(),
            account_id: account_id.map(str::to_owned),
        });
        let app = router(config);
        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("edge proxy test server should serve")
        });
        (addr, handle)
    }

    async fn spawn_fake_makosh() -> (SocketAddr, FakeМакошьState, JoinHandle<()>) {
        let state = FakeМакошьState::default();
        let app = Router::new()
            .route(PROTECTED_MAKOSH_CAPABILITIES_PATH, get(fake_capabilities))
            .route(PROTECTED_MAKOSH_WEBHOOK_PATH, post(fake_webhook_post))
            .with_state(state.clone());
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("fake Макошь test listener should bind");
        let addr = listener
            .local_addr()
            .expect("fake Макошь test listener should have local addr");
        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("fake Макошь test server should serve")
        });
        (addr, state, handle)
    }

    async fn fake_capabilities(
        State(state): State<FakeМакошьState>,
        headers: HeaderMap,
    ) -> Response {
        let mut captured = state.captured.lock().await;
        captured.capabilities_secret = header_value(&headers, MAKOSH_SECRET_HEADER);
        drop(captured);

        Json(json!({ "runtime_mode": "fixture_plus_blocked_live" })).into_response()
    }

    async fn fake_webhook_post(
        State(state): State<FakeМакошьState>,
        RawQuery(raw_query): RawQuery,
        headers: HeaderMap,
        body: Bytes,
    ) -> Response {
        let mut captured = state.captured.lock().await;
        captured.webhook_query = raw_query;
        captured.webhook_secret = header_value(&headers, MAKOSH_SECRET_HEADER);
        captured.webhook_signature = header_value(&headers, ZOOM_SIGNATURE_HEADER);
        captured.webhook_timestamp = header_value(&headers, ZOOM_TIMESTAMP_HEADER);
        captured.webhook_content_type = header_value(&headers, header::CONTENT_TYPE.as_str());
        captured.webhook_body = body.to_vec();
        drop(captured);

        (
            StatusCode::ACCEPTED,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"accepted":true}"#,
        )
            .into_response()
    }

    fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
    }
}
