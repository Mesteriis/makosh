use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{
    Engine,
    engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD},
};
use bytes::Bytes;
use http_body_util::{BodyExt, Limited};
use hyper::body::Body;
use hyper::header::{CACHE_CONTROL, CONTENT_TYPE, ORIGIN, SET_COOKIE};
use hyper::{Method, Request, Response, StatusCode};
use makosh_gateway_session::{
    BrowserGatewaySessionService, BrowserWebauthnAuthenticationCeremonyV1,
};
use makosh_gateway_session_contract::BrowserAuthenticationAuthority;
use serde::{Deserialize, Serialize};
use webauthn_rs_core::proto::{PublicKeyCredential, PublicKeyCredentialRequestOptions};

use crate::{GatewayHttpResponse, full_gateway_body};

const AUTHENTICATION_BEGIN_PATH: &str = "/browser/v1/authentication/begin";
const AUTHENTICATION_FINISH_PREFIX: &str = "/browser/v1/authentication/";
const AUTHENTICATION_FINISH_SUFFIX: &str = "/finish";
const MAX_AUTHENTICATION_BODY_BYTES: usize = 64 * 1024;

/// Detached ordinary-HTTP browser-auth adapter. It exposes no business API and
/// must be composed into a Kernel-managed listener only after admission.
pub struct BrowserAuthenticationRouter<A> {
    service: SharedBrowserGatewaySessionService<A>,
}

impl<A> Clone for BrowserAuthenticationRouter<A> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
        }
    }
}

pub type SharedBrowserGatewaySessionService<A> = Arc<BrowserGatewaySessionService<A>>;

impl<A> BrowserAuthenticationRouter<A>
where
    A: BrowserAuthenticationAuthority,
{
    #[must_use]
    pub fn new(service: BrowserGatewaySessionService<A>) -> Self {
        Self::from_shared(Arc::new(service))
    }

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
        if parts.method != Method::POST {
            return response(StatusCode::NOT_FOUND, "not found\n");
        }
        let authentication_id = if parts.uri.path() == AUTHENTICATION_BEGIN_PATH {
            None
        } else {
            match authentication_id(parts.uri.path()) {
                Some(authentication_id) => Some(authentication_id.to_owned()),
                None => return response(StatusCode::NOT_FOUND, "not found\n"),
            }
        };
        let origin = match parts
            .headers
            .get(ORIGIN)
            .and_then(|value| value.to_str().ok())
        {
            Some(origin) => origin,
            None => return response(StatusCode::FORBIDDEN, "browser origin is required\n"),
        };
        let body = match Limited::new(body, MAX_AUTHENTICATION_BODY_BYTES)
            .collect()
            .await
        {
            Ok(body) => body.to_bytes(),
            Err(_) => return response(StatusCode::BAD_REQUEST, "browser request is invalid\n"),
        };
        match authentication_id {
            None => self.begin(origin, &parts.headers, &body),
            Some(authentication_id) => {
                self.finish(origin, &parts.headers, &authentication_id, &body)
            }
        }
    }

    fn begin(&self, origin: &str, headers: &hyper::HeaderMap, body: &[u8]) -> GatewayHttpResponse {
        if !is_json(headers) {
            return response(StatusCode::BAD_REQUEST, "browser request is invalid\n");
        }
        let request = match serde_json::from_slice::<BeginAuthenticationRequest>(body) {
            Ok(request) => request,
            Err(_) => return response(StatusCode::BAD_REQUEST, "browser request is invalid\n"),
        };
        let credential_id = match decode_credential_id(&request.credential_id) {
            Ok(credential_id) => credential_id,
            Err(()) => return response(StatusCode::BAD_REQUEST, "browser request is invalid\n"),
        };
        match self.service.begin_authentication(origin, &credential_id) {
            Ok(ceremony) => {
                json_response(StatusCode::OK, BeginAuthenticationResponse::from(&ceremony))
            }
            Err(error) if error == "browser mutation origin is invalid" => {
                response(StatusCode::FORBIDDEN, "browser origin is invalid\n")
            }
            Err(_) => response(
                StatusCode::UNAUTHORIZED,
                "browser authentication is unavailable\n",
            ),
        }
    }

    fn finish(
        &self,
        origin: &str,
        headers: &hyper::HeaderMap,
        authentication_id: &str,
        body: &[u8],
    ) -> GatewayHttpResponse {
        if !is_json(headers) {
            return response(StatusCode::BAD_REQUEST, "browser request is invalid\n");
        }
        let request = match serde_json::from_slice::<FinishAuthenticationRequest>(body) {
            Ok(request) => request,
            Err(_) => return response(StatusCode::BAD_REQUEST, "browser request is invalid\n"),
        };
        let browser_key_signature =
            match decode_browser_key_signature(&request.browser_key_signature) {
                Ok(signature) => signature,
                Err(()) => {
                    return response(StatusCode::BAD_REQUEST, "browser request is invalid\n");
                }
            };
        let now = match now_unix_millis() {
            Ok(now) => now,
            Err(()) => {
                return response(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "browser authentication is unavailable\n",
                );
            }
        };
        match self.service.finish_authentication(
            origin,
            authentication_id,
            &request.credential,
            &browser_key_signature,
            now,
        ) {
            Ok(cookie) => authenticated_response(&cookie),
            Err(error) if error == "browser mutation origin is invalid" => {
                response(StatusCode::FORBIDDEN, "browser origin is invalid\n")
            }
            Err(error) => {
                #[cfg(debug_assertions)]
                eprintln!("browser authentication rejected: {error}");
                response(
                    StatusCode::UNAUTHORIZED,
                    "browser authentication is unavailable\n",
                )
            }
        }
    }
}

#[derive(Deserialize)]
struct BeginAuthenticationRequest {
    credential_id: String,
}

#[derive(Serialize)]
struct BeginAuthenticationResponse<'a> {
    authentication_id: &'a str,
    public_key: &'a PublicKeyCredentialRequestOptions,
    browser_key_challenge: String,
}

impl<'a> From<&'a BrowserWebauthnAuthenticationCeremonyV1> for BeginAuthenticationResponse<'a> {
    fn from(ceremony: &'a BrowserWebauthnAuthenticationCeremonyV1) -> Self {
        Self {
            authentication_id: ceremony.authentication_id(),
            public_key: &ceremony.options().public_key,
            browser_key_challenge: URL_SAFE_NO_PAD.encode(ceremony.browser_key_challenge()),
        }
    }
}

#[derive(Deserialize)]
struct FinishAuthenticationRequest {
    credential: PublicKeyCredential,
    browser_key_signature: String,
}

fn authentication_id(path: &str) -> Option<&str> {
    let value = path
        .strip_prefix(AUTHENTICATION_FINISH_PREFIX)?
        .strip_suffix(AUTHENTICATION_FINISH_SUFFIX)?;
    (value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())).then_some(value)
}

fn decode_credential_id(value: &str) -> Result<Vec<u8>, ()> {
    if value.is_empty() || value.len() > 2_048 {
        return Err(());
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .or_else(|_| URL_SAFE.decode(value))
        .map_err(|_| ())?;
    (!bytes.is_empty() && bytes.len() <= 1024)
        .then_some(bytes)
        .ok_or(())
}

fn decode_browser_key_signature(value: &str) -> Result<Vec<u8>, ()> {
    let bytes = decode_credential_id(value)?;
    (bytes.len() == 64).then_some(bytes).ok_or(())
}

fn is_json(headers: &hyper::HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json"))
}

fn now_unix_millis() -> Result<u64, ()> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .map_err(|_| ())
}

fn json_response<T: Serialize>(status: StatusCode, value: T) -> GatewayHttpResponse {
    match serde_json::to_vec(&value) {
        Ok(body) => Response::builder()
            .status(status)
            .header(CONTENT_TYPE, "application/json")
            .header(CACHE_CONTROL, "no-store")
            .body(full_gateway_body(Bytes::from(body)))
            .expect("Gateway browser response is valid"),
        Err(_) => response(
            StatusCode::SERVICE_UNAVAILABLE,
            "browser authentication is unavailable\n",
        ),
    }
}

fn authenticated_response(cookie: &str) -> GatewayHttpResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .header(CACHE_CONTROL, "no-store")
        .header(SET_COOKIE, cookie)
        .body(full_gateway_body(Bytes::from_static(
            b"{\"authenticated\":true}",
        )))
        .expect("Gateway browser response is valid")
}

fn response(status: StatusCode, body: &'static str) -> GatewayHttpResponse {
    Response::builder()
        .status(status)
        .header(CACHE_CONTROL, "no-store")
        .body(full_gateway_body(body))
        .expect("Gateway browser response is valid")
}
