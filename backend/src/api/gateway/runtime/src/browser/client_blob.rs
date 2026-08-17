//! Authenticated descriptor-declared client Blob transport.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Limited};
use hyper::body::Body;
use hyper::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, COOKIE, RANGE};
use hyper::{Method, Request, Response, StatusCode};
use makosh_gateway_session_contract::BrowserAuthenticationAuthority;
use tokio::task;
use tokio::time::{Instant, timeout_at};

use crate::{GatewayHttpResponse, SharedBrowserGatewaySessionService, full_gateway_body};

const MAX_REQUEST_BYTES: usize = 4_096;
const MAX_REQUEST_DEADLINE: Duration = Duration::from_secs(10);
const MAX_CLIENT_BLOB_RESPONSE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_INLINE_BLOB_RESPONSE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_STREAM_RANGE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_ACTIVE_STREAM_LEASES: usize = 4_096;
const STREAM_LEASE_TTL: Duration = Duration::from_secs(60 * 60);
const STREAM_MODE_HEADER: &str = "x-makosh-blob-mode";
const STREAM_CONTENT_TYPE_HEADER: &str = "x-makosh-blob-content-type";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientBlobRouteV1 {
    registration_id: String,
    capability_id: String,
    owner: String,
    contract_name: String,
    contract_major: u32,
    contract_revision: u32,
    contract_schema_sha256: [u8; 32],
    path: String,
    max_response_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientBlobContractVersionV1 {
    pub major: u32,
    pub revision: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientBlobTransportV1 {
    pub path: String,
    pub max_response_bytes: u64,
}

impl ClientBlobRouteV1 {
    pub fn new(
        registration_id: impl Into<String>,
        capability_id: impl Into<String>,
        owner: impl Into<String>,
        contract_name: impl Into<String>,
        contract_version: ClientBlobContractVersionV1,
        contract_schema_sha256: [u8; 32],
        transport: ClientBlobTransportV1,
    ) -> Result<Self, &'static str> {
        let route = Self {
            registration_id: registration_id.into(),
            capability_id: capability_id.into(),
            owner: owner.into(),
            contract_name: contract_name.into(),
            contract_major: contract_version.major,
            contract_revision: contract_version.revision,
            contract_schema_sha256,
            path: transport.path,
            max_response_bytes: transport.max_response_bytes,
        };
        if route.registration_id.is_empty()
            || route.capability_id.is_empty()
            || route.owner.is_empty()
            || route.contract_name.is_empty()
            || route.contract_major == 0
            || route.contract_revision == 0
            || !valid_client_blob_path(&route.path)
            || !(1..=MAX_CLIENT_BLOB_RESPONSE_BYTES).contains(&route.max_response_bytes)
        {
            return Err("client Blob route is invalid");
        }
        Ok(route)
    }

    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn contract_name(&self) -> &str {
        &self.contract_name
    }

    pub const fn contract_major(&self) -> u32 {
        self.contract_major
    }

    pub const fn contract_revision(&self) -> u32 {
        self.contract_revision
    }

    pub const fn contract_schema_sha256(&self) -> &[u8; 32] {
        &self.contract_schema_sha256
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub const fn max_response_bytes(&self) -> u64 {
        self.max_response_bytes
    }
}

pub struct ClientBlobRouter<A> {
    service: SharedBrowserGatewaySessionService<A>,
    route: ClientBlobRouteV1,
    handler: ClientBlobRouteHandler,
    stream_leases: Arc<Mutex<HashMap<String, ClientBlobStreamLeaseV1>>>,
}

pub struct ClientBlobReadV1 {
    pub content: Vec<u8>,
    pub declared_size: u64,
}

pub type ClientBlobRouteHandler = Arc<
    dyn Fn(
            &ClientBlobRouteV1,
            &str,
            &str,
            &str,
            &[u8],
            Option<(u64, u64)>,
        ) -> Result<ClientBlobReadV1, ClientBlobRouteErrorV1>
        + Send
        + Sync,
>;

struct ClientBlobStreamLeaseV1 {
    owner_id: String,
    device_id: String,
    session_id: String,
    request_payload: Vec<u8>,
    media_type: String,
    declared_size: u64,
    expires_at: Instant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientBlobRouteErrorV1 {
    InvalidArgument,
    NotFound,
    Unavailable,
    Internal,
}

impl<A> ClientBlobRouter<A>
where
    A: BrowserAuthenticationAuthority,
{
    #[must_use]
    pub fn new(
        service: SharedBrowserGatewaySessionService<A>,
        route: ClientBlobRouteV1,
        handler: ClientBlobRouteHandler,
    ) -> Self {
        Self {
            service,
            route,
            handler,
            stream_leases: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn path(&self) -> &str {
        self.route.path()
    }

    pub fn admits_path(&self, path: &str) -> bool {
        path == self.route.path()
            || path
                .strip_prefix(self.route.path())
                .is_some_and(|suffix| suffix.starts_with("/stream/") && suffix.len() == 72)
    }

    pub async fn route<B>(&self, request: Request<B>) -> GatewayHttpResponse
    where
        B: Body<Data = Bytes>,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let (parts, body) = request.into_parts();
        if parts.uri.query().is_some() || !self.admits_path(parts.uri.path()) {
            return error_response(StatusCode::NOT_FOUND, "not_found");
        }
        let cookie = parts
            .headers
            .get(COOKIE)
            .and_then(|value| value.to_str().ok());
        let session = match self.service.authorize_request(cookie) {
            Ok(session) => session,
            Err(_) => return error_response(StatusCode::UNAUTHORIZED, "unauthenticated"),
        };
        if parts.method == Method::GET {
            return self
                .route_stream_read(
                    parts.uri.path(),
                    &parts.headers,
                    session.owner_id(),
                    session.device_id(),
                    session.session_id(),
                )
                .await;
        }
        if parts.method != Method::POST || parts.uri.path() != self.route.path() {
            return error_response(StatusCode::NOT_FOUND, "not_found");
        }
        if !is_protobuf(&parts.headers) {
            return error_response(StatusCode::BAD_REQUEST, "invalid_argument");
        }
        let deadline = Instant::now() + MAX_REQUEST_DEADLINE;
        let body = match timeout_at(deadline, Limited::new(body, MAX_REQUEST_BYTES).collect()).await
        {
            Ok(Ok(collected)) => collected.to_bytes(),
            Ok(Err(_)) => return error_response(StatusCode::BAD_REQUEST, "invalid_argument"),
            Err(_) => return error_response(StatusCode::GATEWAY_TIMEOUT, "deadline_exceeded"),
        };
        if body.is_empty() {
            return error_response(StatusCode::BAD_REQUEST, "invalid_argument");
        }
        let owner_id = session.owner_id().to_owned();
        let device_id = session.device_id().to_owned();
        let session_id = session.session_id().to_owned();
        let route = self.route.clone();
        let handler = Arc::clone(&self.handler);
        let request_payload = body.to_vec();
        let handler_payload = request_payload.clone();
        let stream_mode = parts
            .headers
            .get(STREAM_MODE_HEADER)
            .and_then(|value| value.to_str().ok())
            == Some("range-v1");
        let media_type = parts
            .headers
            .get(STREAM_CONTENT_TYPE_HEADER)
            .and_then(|value| value.to_str().ok())
            .filter(|value| valid_media_type(value))
            .unwrap_or("application/octet-stream")
            .to_owned();
        // Authorize and probe one authenticated byte before deciding whether
        // the caller may use the bounded inline path or needs a range lease.
        let read_range = Some((0, 1));
        let content = match timeout_at(
            deadline,
            task::spawn_blocking(move || {
                handler(
                    &route,
                    &owner_id,
                    &device_id,
                    &session_id,
                    &handler_payload,
                    read_range,
                )
            }),
        )
        .await
        {
            Ok(Ok(Ok(content))) => content,
            Ok(Ok(Err(error))) => return route_error(error),
            Ok(Err(_)) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal"),
            Err(_) => return error_response(StatusCode::GATEWAY_TIMEOUT, "deadline_exceeded"),
        };
        if content.content.len() != 1
            || content.declared_size == 0
            || content.declared_size > self.route.max_response_bytes()
        {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal");
        }
        if stream_mode {
            return self.issue_stream_lease(
                session.owner_id(),
                session.device_id(),
                session.session_id(),
                request_payload,
                media_type,
                content.declared_size,
            );
        }
        if content.declared_size > MAX_INLINE_BLOB_RESPONSE_BYTES {
            return error_response(StatusCode::PAYLOAD_TOO_LARGE, "stream_required");
        }
        let route = self.route.clone();
        let handler = Arc::clone(&self.handler);
        let owner_id = session.owner_id().to_owned();
        let device_id = session.device_id().to_owned();
        let session_id = session.session_id().to_owned();
        let content = match timeout_at(
            deadline,
            task::spawn_blocking(move || {
                handler(
                    &route,
                    &owner_id,
                    &device_id,
                    &session_id,
                    &request_payload,
                    None,
                )
            }),
        )
        .await
        {
            Ok(Ok(Ok(content))) => content,
            Ok(Ok(Err(error))) => return route_error(error),
            Ok(Err(_)) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal"),
            Err(_) => return error_response(StatusCode::GATEWAY_TIMEOUT, "deadline_exceeded"),
        };
        if content.declared_size > MAX_INLINE_BLOB_RESPONSE_BYTES
            || u64::try_from(content.content.len()).ok() != Some(content.declared_size)
        {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal");
        }
        binary_response(content.content)
    }

    fn issue_stream_lease(
        &self,
        owner_id: &str,
        device_id: &str,
        session_id: &str,
        request_payload: Vec<u8>,
        media_type: String,
        declared_size: u64,
    ) -> GatewayHttpResponse {
        let mut token_bytes = [0_u8; 32];
        if getrandom::fill(&mut token_bytes).is_err() || token_bytes.iter().all(|byte| *byte == 0) {
            return error_response(StatusCode::SERVICE_UNAVAILABLE, "unavailable");
        }
        let token = token_bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let mut leases = match self.stream_leases.lock() {
            Ok(leases) => leases,
            Err(_) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        };
        let now = Instant::now();
        leases.retain(|_, lease| lease.expires_at > now);
        if leases.len() >= MAX_ACTIVE_STREAM_LEASES {
            return error_response(StatusCode::SERVICE_UNAVAILABLE, "unavailable");
        }
        leases.insert(
            token.clone(),
            ClientBlobStreamLeaseV1 {
                owner_id: owner_id.to_owned(),
                device_id: device_id.to_owned(),
                session_id: session_id.to_owned(),
                request_payload,
                media_type,
                declared_size,
                expires_at: now + STREAM_LEASE_TTL,
            },
        );
        text_response(format!("{}/stream/{token}", self.route.path()))
    }

    async fn route_stream_read(
        &self,
        path: &str,
        headers: &hyper::HeaderMap,
        owner_id: &str,
        device_id: &str,
        session_id: &str,
    ) -> GatewayHttpResponse {
        let Some(token) = path.strip_prefix(&format!("{}/stream/", self.route.path())) else {
            return error_response(StatusCode::NOT_FOUND, "not_found");
        };
        let (payload, media_type, declared_size) = {
            let mut leases = match self.stream_leases.lock() {
                Ok(leases) => leases,
                Err(_) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal"),
            };
            let now = Instant::now();
            leases.retain(|_, lease| lease.expires_at > now);
            let Some(lease) = leases.get_mut(token) else {
                return error_response(StatusCode::NOT_FOUND, "not_found");
            };
            if lease.owner_id != owner_id
                || lease.device_id != device_id
                || lease.session_id != session_id
            {
                return error_response(StatusCode::NOT_FOUND, "not_found");
            }
            lease.expires_at = now + STREAM_LEASE_TTL;
            (
                lease.request_payload.clone(),
                lease.media_type.clone(),
                lease.declared_size,
            )
        };
        let (start, end_exclusive) = match bounded_range(headers, declared_size) {
            Some(range) => range,
            None => return range_not_satisfiable(declared_size),
        };
        let route = self.route.clone();
        let handler = Arc::clone(&self.handler);
        let owner_id = owner_id.to_owned();
        let device_id = device_id.to_owned();
        let session_id = session_id.to_owned();
        let deadline = Instant::now() + MAX_REQUEST_DEADLINE;
        let read = match timeout_at(
            deadline,
            task::spawn_blocking(move || {
                handler(
                    &route,
                    &owner_id,
                    &device_id,
                    &session_id,
                    &payload,
                    Some((start, end_exclusive)),
                )
            }),
        )
        .await
        {
            Ok(Ok(Ok(read))) => read,
            Ok(Ok(Err(error))) => return route_error(error),
            Ok(Err(_)) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal"),
            Err(_) => return error_response(StatusCode::GATEWAY_TIMEOUT, "deadline_exceeded"),
        };
        if read.declared_size != declared_size
            || u64::try_from(read.content.len()).ok() != Some(end_exclusive - start)
        {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal");
        }
        partial_binary_response(
            read.content,
            &media_type,
            start,
            end_exclusive,
            declared_size,
        )
    }
}

impl<A> Clone for ClientBlobRouter<A> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            route: self.route.clone(),
            handler: Arc::clone(&self.handler),
            stream_leases: Arc::clone(&self.stream_leases),
        }
    }
}

fn route_error(error: ClientBlobRouteErrorV1) -> GatewayHttpResponse {
    match error {
        ClientBlobRouteErrorV1::InvalidArgument => {
            error_response(StatusCode::BAD_REQUEST, "invalid_argument")
        }
        ClientBlobRouteErrorV1::NotFound => error_response(StatusCode::NOT_FOUND, "not_found"),
        ClientBlobRouteErrorV1::Unavailable => {
            error_response(StatusCode::SERVICE_UNAVAILABLE, "unavailable")
        }
        ClientBlobRouteErrorV1::Internal => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal")
        }
    }
}

fn is_protobuf(headers: &hyper::HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| matches!(value.trim(), "application/proto" | "application/protobuf"))
}

fn binary_response(content: Vec<u8>) -> GatewayHttpResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/octet-stream")
        .header(CONTENT_LENGTH, content.len().to_string())
        .header("cache-control", "no-store")
        .header("x-content-type-options", "nosniff")
        .body(full_gateway_body(Bytes::from(content)))
        .expect("client Blob response is valid")
}

fn text_response(content: String) -> GatewayHttpResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(CONTENT_LENGTH, content.len().to_string())
        .header("cache-control", "no-store")
        .header("x-content-type-options", "nosniff")
        .body(full_gateway_body(Bytes::from(content)))
        .expect("client Blob stream lease response is valid")
}

fn partial_binary_response(
    content: Vec<u8>,
    media_type: &str,
    start: u64,
    end_exclusive: u64,
    declared_size: u64,
) -> GatewayHttpResponse {
    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(CONTENT_TYPE, media_type)
        .header(CONTENT_LENGTH, content.len().to_string())
        .header(ACCEPT_RANGES, "bytes")
        .header(
            CONTENT_RANGE,
            format!("bytes {start}-{}/{declared_size}", end_exclusive - 1),
        )
        .header("cache-control", "private, max-age=300")
        .header("x-content-type-options", "nosniff")
        .body(full_gateway_body(Bytes::from(content)))
        .expect("client Blob range response is valid")
}

fn bounded_range(headers: &hyper::HeaderMap, declared_size: u64) -> Option<(u64, u64)> {
    let value = headers.get(RANGE).and_then(|value| value.to_str().ok());
    let (start, requested_end) = match value {
        Some(value) => {
            let range = value.strip_prefix("bytes=")?;
            if range.contains(',') {
                return None;
            }
            let (start, end) = range.split_once('-')?;
            if start.is_empty() {
                let suffix = end.parse::<u64>().ok()?;
                if suffix == 0 {
                    return None;
                }
                let bounded_suffix = suffix.min(declared_size).min(MAX_STREAM_RANGE_BYTES);
                (declared_size - bounded_suffix, declared_size)
            } else {
                let start = start.parse::<u64>().ok()?;
                let end = if end.is_empty() {
                    declared_size
                } else {
                    end.parse::<u64>().ok()?.checked_add(1)?
                };
                (start, end)
            }
        }
        None => (0, declared_size),
    };
    if start >= declared_size || requested_end <= start {
        return None;
    }
    Some((
        start,
        requested_end
            .min(declared_size)
            .min(start.saturating_add(MAX_STREAM_RANGE_BYTES)),
    ))
}

fn range_not_satisfiable(declared_size: u64) -> GatewayHttpResponse {
    Response::builder()
        .status(StatusCode::RANGE_NOT_SATISFIABLE)
        .header(CONTENT_RANGE, format!("bytes */{declared_size}"))
        .header("cache-control", "no-store")
        .body(full_gateway_body(Bytes::new()))
        .expect("client Blob range error response is valid")
}

fn valid_media_type(value: &str) -> bool {
    value.len() <= 128
        && value.split_once('/').is_some_and(|(kind, subtype)| {
            matches!(kind, "image" | "video" | "audio" | "application")
                && !subtype.is_empty()
                && subtype
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'+' | b'-'))
        })
}

fn error_response(status: StatusCode, code: &'static str) -> GatewayHttpResponse {
    Response::builder()
        .status(status)
        .header("cache-control", "no-store")
        .header("x-content-type-options", "nosniff")
        .header("x-makosh-error-code", code)
        .body(full_gateway_body(Bytes::new()))
        .expect("client Blob error response is valid")
}

fn valid_client_blob_path(path: &str) -> bool {
    path.starts_with("/api/blobs/")
        && path.len() <= 512
        && path
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-'))
        && !path.contains("//")
        && !path.ends_with('/')
}

#[cfg(test)]
mod tests {
    use http_body_util::Full;
    use makosh_gateway_session::BrowserGatewaySessionService;
    use makosh_gateway_session_contract::{
        BrowserAssertionAuthority, BrowserAuthenticationAuthority, BrowserDeviceAuthority,
        BrowserDeviceCredentialV1, BrowserDevicePrincipalV1, GatewayIdentityFenceV1,
    };

    use super::*;

    struct TestBrowserAuthority;

    impl BrowserDeviceAuthority for TestBrowserAuthority {
        fn current_identity_fence(&self) -> Result<GatewayIdentityFenceV1, String> {
            Err("unused test authority".to_owned())
        }
        fn active_browser_device(
            &self,
            _device_id: &str,
        ) -> Result<BrowserDevicePrincipalV1, String> {
            Err("unused test authority".to_owned())
        }
        fn active_browser_device_by_credential(
            &self,
            _credential_id: &[u8],
        ) -> Result<BrowserDevicePrincipalV1, String> {
            Err("unused test authority".to_owned())
        }
    }

    impl BrowserAssertionAuthority for TestBrowserAuthority {
        fn accept_verified_browser_assertion(
            &self,
            _credential_id: &[u8],
            _sign_count: u32,
            _backup_eligible: bool,
            _backup_state: bool,
        ) -> Result<BrowserDevicePrincipalV1, String> {
            Err("unused test authority".to_owned())
        }
    }

    impl BrowserAuthenticationAuthority for TestBrowserAuthority {
        fn active_browser_credential(
            &self,
            _credential_id: &[u8],
        ) -> Result<BrowserDeviceCredentialV1, String> {
            Err("unused test authority".to_owned())
        }
    }

    #[test]
    fn client_blob_route_requires_a_bounded_blob_path() {
        let route = ClientBlobRouteV1::new(
            "registration",
            "owner.content.v1",
            "owner",
            "owner.content-read",
            ClientBlobContractVersionV1 {
                major: 1,
                revision: 1,
            },
            [7; 32],
            ClientBlobTransportV1 {
                path: "/api/blobs/owner/v1/content".to_owned(),
                max_response_bytes: 256 * 1024,
            },
        )
        .expect("route");
        assert_eq!(route.max_response_bytes(), 256 * 1024);
        assert!(
            ClientBlobRouteV1::new(
                "registration",
                "owner.content.v1",
                "owner",
                "owner.content-read",
                ClientBlobContractVersionV1 {
                    major: 1,
                    revision: 1,
                },
                [7; 32],
                ClientBlobTransportV1 {
                    path: "/api/blobs/owner/v1/content".to_owned(),
                    max_response_bytes: MAX_CLIENT_BLOB_RESPONSE_BYTES,
                },
            )
            .is_ok()
        );
        assert!(
            ClientBlobRouteV1::new(
                "registration",
                "owner.content.v1",
                "owner",
                "owner.content-read",
                ClientBlobContractVersionV1 {
                    major: 1,
                    revision: 1,
                },
                [7; 32],
                ClientBlobTransportV1 {
                    path: "/api/blobs/owner/v1/content".to_owned(),
                    max_response_bytes: MAX_CLIENT_BLOB_RESPONSE_BYTES + 1,
                },
            )
            .is_err()
        );
        assert!(
            ClientBlobRouteV1::new(
                "registration",
                "owner.content.v1",
                "owner",
                "owner.content-read",
                ClientBlobContractVersionV1 {
                    major: 1,
                    revision: 1,
                },
                [7; 32],
                ClientBlobTransportV1 {
                    path: "/api/owner/content".to_owned(),
                    max_response_bytes: 1,
                },
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn client_blob_response_is_opaque_exact_and_non_cacheable() {
        let response = binary_response(vec![0, 1, 2, 255]);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE),
            Some(&hyper::header::HeaderValue::from_static(
                "application/octet-stream"
            ))
        );
        assert_eq!(
            response.headers().get("cache-control"),
            Some(&hyper::header::HeaderValue::from_static("no-store"))
        );
        assert_eq!(
            response.headers().get("x-content-type-options"),
            Some(&hyper::header::HeaderValue::from_static("nosniff"))
        );
        assert_eq!(
            response
                .into_body()
                .collect()
                .await
                .expect("response body")
                .to_bytes()
                .as_ref(),
            &[0, 1, 2, 255]
        );
    }

    #[test]
    fn stream_ranges_are_single_bounded_and_media_types_are_sanitized() {
        let mut headers = hyper::HeaderMap::new();
        headers.insert(RANGE, hyper::header::HeaderValue::from_static("bytes=5-"));
        assert_eq!(
            bounded_range(&headers, 10 * 1024 * 1024),
            Some((5, 5 + MAX_STREAM_RANGE_BYTES)),
        );
        headers.insert(
            RANGE,
            hyper::header::HeaderValue::from_static("bytes=10-19"),
        );
        assert_eq!(bounded_range(&headers, 100), Some((10, 20)));
        headers.insert(RANGE, hyper::header::HeaderValue::from_static("bytes=-25"));
        assert_eq!(bounded_range(&headers, 100), Some((75, 100)));
        headers.insert(RANGE, hyper::header::HeaderValue::from_static("bytes=-0"));
        assert_eq!(bounded_range(&headers, 100), None);
        headers.insert(
            RANGE,
            hyper::header::HeaderValue::from_static("bytes=10-19,30-39"),
        );
        assert_eq!(bounded_range(&headers, 100), None);
        assert!(valid_media_type("video/mp4"));
        assert!(valid_media_type("audio/ogg"));
        assert!(!valid_media_type("text/html"));
        assert!(!valid_media_type("video/mp4; charset=utf-8"));
    }

    #[tokio::test]
    async fn stream_lease_returns_only_the_authenticated_requested_range() {
        let service = Arc::new(
            BrowserGatewaySessionService::new_loopback_development(
                TestBrowserAuthority,
                "http://127.0.0.1:5173",
                "owner",
                "device",
            )
            .expect("loopback session"),
        );
        let route = ClientBlobRouteV1::new(
            "registration",
            "owner.content.v1",
            "owner",
            "owner.content-read",
            ClientBlobContractVersionV1 {
                major: 1,
                revision: 1,
            },
            [7; 32],
            ClientBlobTransportV1 {
                path: "/api/blobs/owner/v1/content".to_owned(),
                max_response_bytes: 10,
            },
        )
        .expect("route");
        let handler: ClientBlobRouteHandler =
            Arc::new(|_, owner_id, device_id, session_id, payload, range| {
                assert_eq!(owner_id, "owner");
                assert_eq!(device_id, "device");
                assert!(!session_id.is_empty());
                assert_eq!(payload, &[1]);
                let bytes = b"abcdefghij";
                let content = range.map_or_else(
                    || bytes.to_vec(),
                    |(start, end)| bytes[start as usize..end as usize].to_vec(),
                );
                Ok(ClientBlobReadV1 {
                    content,
                    declared_size: 10,
                })
            });
        let router = ClientBlobRouter::new(service, route, handler);
        let response = router
            .route(
                Request::post("/api/blobs/owner/v1/content")
                    .header(CONTENT_TYPE, "application/protobuf")
                    .header(STREAM_MODE_HEADER, "range-v1")
                    .header(STREAM_CONTENT_TYPE_HEADER, "video/mp4")
                    .body(Full::new(Bytes::from_static(&[1])))
                    .expect("stream request"),
            )
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        let stream_path = String::from_utf8(
            response
                .into_body()
                .collect()
                .await
                .expect("lease response")
                .to_bytes()
                .to_vec(),
        )
        .expect("stream path");
        assert!(router.admits_path(&stream_path));

        let response = router
            .route(
                Request::get(stream_path.clone())
                    .header(RANGE, "bytes=2-5")
                    .body(Full::new(Bytes::new()))
                    .expect("range request"),
            )
            .await;
        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            response
                .headers()
                .get(CONTENT_RANGE)
                .and_then(|value| value.to_str().ok()),
            Some("bytes 2-5/10"),
        );
        assert_eq!(
            response
                .into_body()
                .collect()
                .await
                .expect("range response")
                .to_bytes(),
            Bytes::from_static(b"cdef"),
        );

        let token = stream_path
            .rsplit('/')
            .next()
            .expect("stream lease token")
            .to_owned();
        let original_session_id = {
            let mut leases = router.stream_leases.lock().expect("stream leases");
            let lease = leases.get_mut(&token).expect("issued stream lease");
            let original = lease.session_id.clone();
            lease.session_id = "different-session".to_owned();
            original
        };
        let wrong_session = router
            .route(
                Request::get(stream_path.clone())
                    .header(RANGE, "bytes=0-0")
                    .body(Full::new(Bytes::new()))
                    .expect("wrong-session request"),
            )
            .await;
        assert_eq!(wrong_session.status(), StatusCode::NOT_FOUND);

        {
            let mut leases = router.stream_leases.lock().expect("stream leases");
            let lease = leases.get_mut(&token).expect("issued stream lease");
            lease.session_id = original_session_id;
            lease.expires_at = Instant::now()
                .checked_sub(Duration::from_secs(1))
                .expect("expired instant");
        }
        let expired = router
            .route(
                Request::get(stream_path)
                    .header(RANGE, "bytes=0-0")
                    .body(Full::new(Bytes::new()))
                    .expect("expired request"),
            )
            .await;
        assert_eq!(expired.status(), StatusCode::NOT_FOUND);
    }
}
