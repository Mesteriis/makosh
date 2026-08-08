//! Authenticated Connect adapter for platform-owned write-only Vault provisioning.

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Limited};
use hyper::body::Body;
use hyper::header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE, HeaderName, ORIGIN};
use hyper::{Method, Request, Response, StatusCode};
use makosh_gateway_protocol::v1::{
    AuthorizeOwnerVaultProvisioningRequestV1, AuthorizeOwnerVaultProvisioningResponseV1,
    CommitOwnerVaultProvisioningRequestV1, CommitOwnerVaultProvisioningResponseV1,
    PrepareOwnerVaultProvisioningRequestV1, PrepareOwnerVaultProvisioningResponseV1,
};
use makosh_gateway_session_contract::BrowserAuthenticationAuthority;
use prost::Message;
use tokio::task;
use tokio::time::{Instant, timeout_at};

use super::owner_principal::OwnerBrowserPrincipalV1;
use crate::{GatewayHttpResponse, SharedBrowserGatewaySessionService, full_gateway_body};

pub const OWNER_VAULT_PREPARE_PATH: &str =
    "/makosh.gateway.v1.OwnerVaultProvisioningService/Prepare";
pub const OWNER_VAULT_AUTHORIZE_PATH: &str =
    "/makosh.gateway.v1.OwnerVaultProvisioningService/Authorize";
pub const OWNER_VAULT_COMMIT_PATH: &str = "/makosh.gateway.v1.OwnerVaultProvisioningService/Commit";

const MAX_REQUEST_BYTES: usize = 128 * 1024;
const MAX_REQUEST_DEADLINE: Duration = Duration::from_secs(10);
const CONNECT_PROTOCOL_VERSION: HeaderName = HeaderName::from_static("connect-protocol-version");
const CONNECT_ERROR_CODE: HeaderName = HeaderName::from_static("connect-error-code");
const CONNECT_TIMEOUT_MS: HeaderName = HeaderName::from_static("connect-timeout-ms");

pub type OwnerVaultClientPrincipalV1 = OwnerBrowserPrincipalV1;

pub trait OwnerVaultProvisioningHandlerV1: Send + Sync {
    fn prepare(
        &self,
        principal: &OwnerVaultClientPrincipalV1,
        request: PrepareOwnerVaultProvisioningRequestV1,
    ) -> Result<PrepareOwnerVaultProvisioningResponseV1, OwnerVaultProvisioningRouteErrorV1>;

    fn authorize(
        &self,
        principal: &OwnerVaultClientPrincipalV1,
        request: AuthorizeOwnerVaultProvisioningRequestV1,
    ) -> Result<AuthorizeOwnerVaultProvisioningResponseV1, OwnerVaultProvisioningRouteErrorV1>;

    fn commit(
        &self,
        principal: &OwnerVaultClientPrincipalV1,
        request: CommitOwnerVaultProvisioningRequestV1,
    ) -> Result<CommitOwnerVaultProvisioningResponseV1, OwnerVaultProvisioningRouteErrorV1>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerVaultProvisioningRouteErrorV1 {
    InvalidArgument,
    PermissionDenied,
    NotFound,
    Conflict,
    Unavailable,
    Internal,
}

pub struct OwnerVaultProvisioningRouter<A> {
    service: SharedBrowserGatewaySessionService<A>,
    handler: Arc<dyn OwnerVaultProvisioningHandlerV1>,
}

impl<A> Clone for OwnerVaultProvisioningRouter<A> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            handler: Arc::clone(&self.handler),
        }
    }
}

impl<A> OwnerVaultProvisioningRouter<A>
where
    A: BrowserAuthenticationAuthority,
{
    #[must_use]
    pub fn new(
        service: SharedBrowserGatewaySessionService<A>,
        handler: Arc<dyn OwnerVaultProvisioningHandlerV1>,
    ) -> Self {
        Self { service, handler }
    }

    #[must_use]
    pub fn admits_path(path: &str) -> bool {
        matches!(
            path,
            OWNER_VAULT_PREPARE_PATH | OWNER_VAULT_AUTHORIZE_PATH | OWNER_VAULT_COMMIT_PATH
        )
    }

    pub async fn route<B>(&self, request: Request<B>) -> GatewayHttpResponse
    where
        B: Body<Data = Bytes>,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let (parts, body) = request.into_parts();
        if parts.method != Method::POST
            || !Self::admits_path(parts.uri.path())
            || parts.uri.query().is_some()
        {
            return not_found();
        }
        if self.service.is_lan_development() {
            return permission_denied();
        }
        if !is_protobuf(&parts.headers) {
            return invalid_argument();
        }
        let origin = parts
            .headers
            .get(ORIGIN)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        if self.service.require_mutation_origin(origin).is_err() {
            return permission_denied();
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
        if body.is_empty() {
            return invalid_argument();
        }
        let operation = match decode_operation(parts.uri.path(), &body) {
            Ok(operation) => operation,
            Err(()) => return invalid_argument(),
        };
        let principal = match OwnerVaultClientPrincipalV1::new(
            session.owner_id(),
            session.device_id(),
            session.session_id(),
        ) {
            Ok(principal) => principal,
            Err(_) => return unauthenticated(),
        };
        let handler = Arc::clone(&self.handler);
        let response_payload = match timeout_at(
            deadline,
            task::spawn_blocking(move || operation.execute(handler, &principal)),
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

enum OwnerVaultProvisioningOperationV1 {
    Prepare(PrepareOwnerVaultProvisioningRequestV1),
    Authorize(AuthorizeOwnerVaultProvisioningRequestV1),
    Commit(CommitOwnerVaultProvisioningRequestV1),
}

impl OwnerVaultProvisioningOperationV1 {
    fn execute(
        self,
        handler: Arc<dyn OwnerVaultProvisioningHandlerV1>,
        principal: &OwnerVaultClientPrincipalV1,
    ) -> Result<Vec<u8>, OwnerVaultProvisioningRouteErrorV1> {
        match self {
            Self::Prepare(request) => handler
                .prepare(principal, request)
                .map(|value| value.encode_to_vec()),
            Self::Authorize(request) => handler
                .authorize(principal, request)
                .map(|value| value.encode_to_vec()),
            Self::Commit(request) => handler
                .commit(principal, request)
                .map(|value| value.encode_to_vec()),
        }
    }
}

fn decode_operation(path: &str, bytes: &[u8]) -> Result<OwnerVaultProvisioningOperationV1, ()> {
    match path {
        OWNER_VAULT_PREPARE_PATH => PrepareOwnerVaultProvisioningRequestV1::decode(bytes)
            .map(OwnerVaultProvisioningOperationV1::Prepare)
            .map_err(|_| ()),
        OWNER_VAULT_AUTHORIZE_PATH => AuthorizeOwnerVaultProvisioningRequestV1::decode(bytes)
            .map(OwnerVaultProvisioningOperationV1::Authorize)
            .map_err(|_| ()),
        OWNER_VAULT_COMMIT_PATH => CommitOwnerVaultProvisioningRequestV1::decode(bytes)
            .map(OwnerVaultProvisioningOperationV1::Commit)
            .map_err(|_| ()),
        _ => Err(()),
    }
}

fn route_error(error: OwnerVaultProvisioningRouteErrorV1) -> GatewayHttpResponse {
    match error {
        OwnerVaultProvisioningRouteErrorV1::InvalidArgument => invalid_argument(),
        OwnerVaultProvisioningRouteErrorV1::PermissionDenied => permission_denied(),
        OwnerVaultProvisioningRouteErrorV1::NotFound => not_found(),
        OwnerVaultProvisioningRouteErrorV1::Conflict => conflict(),
        OwnerVaultProvisioningRouteErrorV1::Unavailable => unavailable(),
        OwnerVaultProvisioningRouteErrorV1::Internal => internal(),
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
        .expect("Gateway owner Vault response is valid")
}

fn invalid_argument() -> GatewayHttpResponse {
    connect_error(StatusCode::BAD_REQUEST, "invalid_argument")
}
fn unauthenticated() -> GatewayHttpResponse {
    connect_error(StatusCode::UNAUTHORIZED, "unauthenticated")
}
fn permission_denied() -> GatewayHttpResponse {
    connect_error(StatusCode::FORBIDDEN, "permission_denied")
}
fn not_found() -> GatewayHttpResponse {
    connect_error(StatusCode::NOT_FOUND, "unimplemented")
}
fn conflict() -> GatewayHttpResponse {
    connect_error(StatusCode::CONFLICT, "already_exists")
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
        .expect("Gateway owner Vault Connect error is valid")
}
