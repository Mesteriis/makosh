use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Limited};
use hyper::body::Body;
use hyper::header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE, HeaderName};
use hyper::{Method, Request, Response, StatusCode};
use makosh_gateway_session_contract::BrowserAuthenticationAuthority;
use tokio::task;
use tokio::time::{Instant, timeout_at};

use crate::{GatewayHttpResponse, SharedBrowserGatewaySessionService, full_gateway_body};

const MAX_REQUEST_BYTES: usize = 16_384;
const MAX_REQUEST_DEADLINE: Duration = Duration::from_secs(10);
const CONNECT_PROTOCOL_VERSION: HeaderName = HeaderName::from_static("connect-protocol-version");
const CONNECT_ERROR_CODE: HeaderName = HeaderName::from_static("connect-error-code");
const CONNECT_TIMEOUT_MS: HeaderName = HeaderName::from_static("connect-timeout-ms");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientRpcRouteV1 {
    registration_id: String,
    capability_id: String,
    owner: String,
    contract_name: String,
    contract_major: u32,
    contract_revision: u32,
    contract_schema_sha256: [u8; 32],
    path: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientRpcContractVersionV1 {
    pub major: u32,
    pub revision: u32,
}

impl ClientRpcRouteV1 {
    #[must_use]
    pub fn new(
        registration_id: impl Into<String>,
        capability_id: impl Into<String>,
        owner: impl Into<String>,
        contract_name: impl Into<String>,
        contract_version: ClientRpcContractVersionV1,
        contract_schema_sha256: [u8; 32],
        path: impl Into<String>,
    ) -> Self {
        Self {
            registration_id: registration_id.into(),
            capability_id: capability_id.into(),
            owner: owner.into(),
            contract_name: contract_name.into(),
            contract_major: contract_version.major,
            contract_revision: contract_version.revision,
            contract_schema_sha256,
            path: path.into(),
        }
    }
    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }
    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }
    #[must_use]
    pub fn contract_name(&self) -> &str {
        &self.contract_name
    }
    #[must_use]
    pub const fn contract_major(&self) -> u32 {
        self.contract_major
    }
    #[must_use]
    pub const fn contract_revision(&self) -> u32 {
        self.contract_revision
    }
    #[must_use]
    pub const fn contract_schema_sha256(&self) -> &[u8; 32] {
        &self.contract_schema_sha256
    }
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
}

pub struct ClientRpcRouter<A> {
    service: SharedBrowserGatewaySessionService<A>,
    route: ClientRpcRouteV1,
    handler: ClientRpcRouteHandler,
}
pub type ClientRpcRouteHandler = Arc<
    dyn Fn(&ClientRpcRouteV1, &str, &str, &str, &[u8]) -> Result<Vec<u8>, ClientRpcRouteErrorV1>
        + Send
        + Sync,
>;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientRpcRouteErrorV1 {
    InvalidArgument,
    NotFound,
    Unavailable,
    Internal,
}

impl<A> ClientRpcRouter<A>
where
    A: BrowserAuthenticationAuthority,
{
    #[must_use]
    pub fn new(
        service: SharedBrowserGatewaySessionService<A>,
        route: ClientRpcRouteV1,
        handler: ClientRpcRouteHandler,
    ) -> Self {
        Self {
            service,
            route,
            handler,
        }
    }
    #[must_use]
    pub fn path(&self) -> &str {
        self.route.path()
    }
    pub async fn route<B>(&self, request: Request<B>) -> GatewayHttpResponse
    where
        B: Body<Data = Bytes>,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let (parts, body) = request.into_parts();
        if parts.method != Method::POST
            || parts.uri.path() != self.route.path()
            || parts.uri.query().is_some()
        {
            return not_found();
        }
        if !is_protobuf(&parts.headers) {
            return invalid_argument();
        }
        let cookie = parts
            .headers
            .get(COOKIE)
            .and_then(|value| value.to_str().ok());
        let session = match self.service.authorize_request(cookie) {
            Ok(session) => session,
            Err(_) => return unauthenticated(),
        };
        let timeout = match request_timeout(&parts.headers) {
            Ok(timeout) => timeout,
            Err(()) => return invalid_argument(),
        };
        let deadline = Instant::now() + timeout;
        let body = match timeout_at(deadline, Limited::new(body, MAX_REQUEST_BYTES).collect()).await
        {
            Ok(Ok(collected)) => collected.to_bytes(),
            Ok(Err(_)) => return invalid_argument(),
            Err(_) => return deadline_exceeded(),
        };
        let owner_id = session.owner_id().to_owned();
        let device_id = session.device_id().to_owned();
        let session_id = session.session_id().to_owned();
        let route = self.route.clone();
        let handler = Arc::clone(&self.handler);
        let response_payload = match timeout_at(
            deadline,
            task::spawn_blocking(move || {
                handler(&route, &owner_id, &device_id, &session_id, &body)
            }),
        )
        .await
        {
            Ok(Ok(Ok(response))) => response,
            Ok(Ok(Err(error))) => return route_error(error),
            Ok(Err(_)) => return internal(),
            Err(_) => return deadline_exceeded(),
        };
        protobuf_response(response_payload)
    }
}
impl<A> Clone for ClientRpcRouter<A> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            route: self.route.clone(),
            handler: Arc::clone(&self.handler),
        }
    }
}
fn route_error(error: ClientRpcRouteErrorV1) -> GatewayHttpResponse {
    match error {
        ClientRpcRouteErrorV1::InvalidArgument => invalid_argument(),
        ClientRpcRouteErrorV1::NotFound => not_found(),
        ClientRpcRouteErrorV1::Unavailable => unavailable(),
        ClientRpcRouteErrorV1::Internal => internal(),
    }
}
fn is_protobuf(headers: &hyper::HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| {
            matches!(
                value.trim(),
                "application/proto" | "application/connect+proto"
            )
        })
}
fn request_timeout(headers: &hyper::HeaderMap) -> Result<Duration, ()> {
    let Some(value) = headers.get(CONNECT_TIMEOUT_MS) else {
        return Ok(MAX_REQUEST_DEADLINE);
    };
    value
        .to_str()
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| (1..=MAX_REQUEST_DEADLINE.as_millis() as u64).contains(value))
        .map(Duration::from_millis)
        .ok_or(())
}
fn protobuf_response(response_payload: Vec<u8>) -> GatewayHttpResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/proto")
        .header(CACHE_CONTROL, "no-store")
        .header(CONNECT_PROTOCOL_VERSION, "1")
        .body(full_gateway_body(response_payload))
        .expect("Gateway ClientRpc response is valid")
}
fn invalid_argument() -> GatewayHttpResponse {
    connect_error(StatusCode::BAD_REQUEST, "invalid_argument")
}
fn unauthenticated() -> GatewayHttpResponse {
    connect_error(StatusCode::UNAUTHORIZED, "unauthenticated")
}
fn not_found() -> GatewayHttpResponse {
    connect_error(StatusCode::NOT_FOUND, "unimplemented")
}
fn unavailable() -> GatewayHttpResponse {
    connect_error(StatusCode::SERVICE_UNAVAILABLE, "unavailable")
}
fn internal() -> GatewayHttpResponse {
    connect_error(StatusCode::INTERNAL_SERVER_ERROR, "internal")
}
fn deadline_exceeded() -> GatewayHttpResponse {
    connect_error(StatusCode::GATEWAY_TIMEOUT, "deadline_exceeded")
}
fn connect_error(status: StatusCode, code: &'static str) -> GatewayHttpResponse {
    Response::builder()
        .status(status)
        .header(CACHE_CONTROL, "no-store")
        .header(CONNECT_PROTOCOL_VERSION, "1")
        .header(CONNECT_ERROR_CODE, code)
        .body(full_gateway_body(Bytes::new()))
        .expect("Gateway Connect error is valid")
}

#[cfg(test)]
mod tests {
    use http_body_util::{BodyExt, Full};
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

    #[tokio::test]
    async fn forwards_a_valid_empty_protobuf_request_to_the_owner_handler() {
        let service = Arc::new(
            BrowserGatewaySessionService::new_loopback_development(
                TestBrowserAuthority,
                "http://127.0.0.1:5173",
                "owner",
                "device",
            )
            .expect("loopback session"),
        );
        let route = ClientRpcRouteV1::new(
            "registration",
            "owner.catalog.query.v1",
            "owner",
            "owner.catalog.v1",
            ClientRpcContractVersionV1 {
                major: 1,
                revision: 1,
            },
            [7; 32],
            "/owner.catalog.v1.CatalogService/List",
        );
        let handler: ClientRpcRouteHandler =
            Arc::new(|_, owner_id, device_id, session_id, payload| {
                assert_eq!(owner_id, "owner");
                assert_eq!(device_id, "device");
                assert!(!session_id.is_empty());
                assert!(payload.is_empty());
                Ok(vec![1])
            });
        let router = ClientRpcRouter::new(service, route, handler);
        let request = Request::post("/owner.catalog.v1.CatalogService/List")
            .header(CONTENT_TYPE, "application/proto")
            .header(CONNECT_PROTOCOL_VERSION, "1")
            .body(Full::new(Bytes::new()))
            .expect("empty protobuf request");

        let response = router.route(request).await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn preserves_a_valid_empty_protobuf_response() {
        let response = protobuf_response(Vec::new());

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("application/proto")
        );
        assert!(
            response
                .into_body()
                .collect()
                .await
                .expect("response body")
                .to_bytes()
                .is_empty()
        );
    }
}
