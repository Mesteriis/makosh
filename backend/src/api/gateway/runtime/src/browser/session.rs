use bytes::Bytes;
use http_body_util::{BodyExt, Limited};
use hyper::body::Body;
use hyper::header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE, HeaderName};
use hyper::{Method, Request, Response, StatusCode};
use makosh_gateway_protocol::v1::{
    BrowserGatewayAccessModeV1, BrowserSessionStatusRequestV1, BrowserSessionStatusResponseV1,
};
use makosh_gateway_session_contract::BrowserAuthenticationAuthority;
use prost::Message;

use crate::{GatewayHttpResponse, SharedBrowserGatewaySessionService, full_gateway_body};

const STATUS_PATH: &str = "/makosh.gateway.v1.BrowserSessionService/GetStatus";
const MAX_REQUEST_BYTES: usize = 1_024;
const CONNECT_PROTOCOL_VERSION: HeaderName = HeaderName::from_static("connect-protocol-version");
const CONNECT_ERROR_CODE: HeaderName = HeaderName::from_static("connect-error-code");

/// Owner-neutral ConnectRPC status route for a browser's already authenticated
/// Gateway session. It deliberately exposes no owner data or business method.
pub struct BrowserSessionStatusRouter<A> {
    service: SharedBrowserGatewaySessionService<A>,
}

impl<A> Clone for BrowserSessionStatusRouter<A> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
        }
    }
}

impl<A> BrowserSessionStatusRouter<A>
where
    A: BrowserAuthenticationAuthority,
{
    #[must_use]
    pub const fn from_shared(service: SharedBrowserGatewaySessionService<A>) -> Self {
        Self { service }
    }

    pub async fn route<B>(&self, request: Request<B>) -> GatewayHttpResponse
    where
        B: Body<Data = Bytes>,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let (parts, body) = request.into_parts();
        if parts.method != Method::POST
            || parts.uri.path() != STATUS_PATH
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
        let body = match Limited::new(body, MAX_REQUEST_BYTES).collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => return invalid_argument(),
        };
        if !body.is_empty() || BrowserSessionStatusRequestV1::decode(body).is_err() {
            return invalid_argument();
        }
        match self.service.authorize_request(cookie) {
            Ok(session) => protobuf_response(BrowserSessionStatusResponseV1 {
                major: 1,
                session_expires_at_unix_millis: session.expires_at_unix_millis(),
                access_mode: if self.service.is_loopback_development() {
                    BrowserGatewayAccessModeV1::LocalDevelopment as i32
                } else if self.service.is_lan_development() {
                    BrowserGatewayAccessModeV1::LanDevelopment as i32
                } else {
                    BrowserGatewayAccessModeV1::Paired as i32
                },
            }),
            Err(_) => unauthenticated(),
        }
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

fn protobuf_response(response: BrowserSessionStatusResponseV1) -> GatewayHttpResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/proto")
        .header(CACHE_CONTROL, "no-store")
        .header(CONNECT_PROTOCOL_VERSION, "1")
        .body(full_gateway_body(response.encode_to_vec()))
        .expect("Gateway Connect response is valid")
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

fn connect_error(status: StatusCode, code: &'static str) -> GatewayHttpResponse {
    Response::builder()
        .status(status)
        .header(CACHE_CONTROL, "no-store")
        .header(CONNECT_PROTOCOL_VERSION, "1")
        .header(CONNECT_ERROR_CODE, code)
        .body(full_gateway_body(Bytes::new()))
        .expect("Gateway Connect error is valid")
}
