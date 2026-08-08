//! Authenticated descriptor-declared client Blob transport.

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Limited};
use hyper::body::Body;
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE, COOKIE};
use hyper::{Method, Request, Response, StatusCode};
use makosh_gateway_session_contract::BrowserAuthenticationAuthority;
use tokio::task;
use tokio::time::{Instant, timeout_at};

use crate::{GatewayHttpResponse, SharedBrowserGatewaySessionService, full_gateway_body};

const MAX_REQUEST_BYTES: usize = 4_096;
const MAX_REQUEST_DEADLINE: Duration = Duration::from_secs(10);
const MAX_CLIENT_BLOB_RESPONSE_BYTES: u64 = 32 * 1024 * 1024;

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
}

pub type ClientBlobRouteHandler = Arc<
    dyn Fn(&ClientBlobRouteV1, &str, &str, &str, &[u8]) -> Result<Vec<u8>, ClientBlobRouteErrorV1>
        + Send
        + Sync,
>;

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
        }
    }

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
            return error_response(StatusCode::NOT_FOUND, "not_found");
        }
        if !is_protobuf(&parts.headers) {
            return error_response(StatusCode::BAD_REQUEST, "invalid_argument");
        }
        let cookie = parts
            .headers
            .get(COOKIE)
            .and_then(|value| value.to_str().ok());
        let session = match self.service.authorize_request(cookie) {
            Ok(session) => session,
            Err(_) => return error_response(StatusCode::UNAUTHORIZED, "unauthenticated"),
        };
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
        let content = match timeout_at(
            deadline,
            task::spawn_blocking(move || {
                handler(&route, &owner_id, &device_id, &session_id, &body)
            }),
        )
        .await
        {
            Ok(Ok(Ok(content))) => content,
            Ok(Ok(Err(error))) => return route_error(error),
            Ok(Err(_)) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal"),
            Err(_) => return error_response(StatusCode::GATEWAY_TIMEOUT, "deadline_exceeded"),
        };
        if content.is_empty()
            || u64::try_from(content.len()).ok() > Some(self.route.max_response_bytes())
        {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal");
        }
        binary_response(content)
    }
}

impl<A> Clone for ClientBlobRouter<A> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            route: self.route.clone(),
            handler: Arc::clone(&self.handler),
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
    use super::*;

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
}
