use std::sync::{Arc, Mutex};

use base64::Engine;
use bytes::Bytes;
use http_body_util::{BodyExt, Limited};
use hyper::body::Body;
use hyper::header::{CACHE_CONTROL, CONTENT_TYPE, ORIGIN};
use hyper::{Method, Request, Response, StatusCode};
use makosh_gateway_session::{
    BrowserPairingManager, BrowserSameOriginSessionV1, BrowserWebauthnVerifier,
};
use makosh_gateway_session_contract::BrowserEnrollmentAuthority;
use serde::{Deserialize, Serialize};
use webauthn_rs_core::proto::{CreationChallengeResponse, RegisterPublicKeyCredential};

use crate::{GatewayHttpResponse, full_gateway_body};

const PAIRING_PREFIX: &str = "/browser/v1/pairing/";
const REGISTRATION_SUFFIX: &str = "/registration";
const REGISTRATION_FINISH_SUFFIX: &str = "/registration/finish";
const MAX_REGISTRATION_BODY_BYTES: usize = 64 * 1024;

pub type SharedBrowserPairingManager = Arc<Mutex<BrowserPairingManager>>;

/// Narrow public adapter for an owner-approved, short-lived browser WebAuthn
/// registration. The pairing ID is opaque and cannot create a pairing on its
/// own; only the private owner-control path inserts a ceremony into the shared
/// manager.
pub struct BrowserPairingRouter<A> {
    pairings: SharedBrowserPairingManager,
    authority: Arc<A>,
    verifier: Arc<BrowserWebauthnVerifier>,
    exact_https_origin: Arc<str>,
}

impl<A> Clone for BrowserPairingRouter<A> {
    fn clone(&self) -> Self {
        Self {
            pairings: Arc::clone(&self.pairings),
            authority: Arc::clone(&self.authority),
            verifier: Arc::clone(&self.verifier),
            exact_https_origin: Arc::clone(&self.exact_https_origin),
        }
    }
}

impl<A> BrowserPairingRouter<A>
where
    A: BrowserEnrollmentAuthority,
{
    #[must_use]
    pub fn new(
        pairings: SharedBrowserPairingManager,
        authority: A,
        verifier: BrowserWebauthnVerifier,
        exact_https_origin: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            pairings,
            authority: Arc::new(authority),
            verifier: Arc::new(verifier),
            exact_https_origin: exact_https_origin.into(),
        }
    }

    pub async fn route<B>(&self, request: Request<B>) -> GatewayHttpResponse
    where
        B: Body<Data = Bytes>,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let (parts, body) = request.into_parts();
        let Some((pairing_id, finish)) = registration_route(parts.uri.path()) else {
            return response(StatusCode::NOT_FOUND, "not found\n");
        };
        match (parts.method, finish) {
            (Method::GET, false) => self.options(pairing_id),
            (Method::POST, true) => {
                let origin = match parts
                    .headers
                    .get(ORIGIN)
                    .and_then(|value| value.to_str().ok())
                {
                    Some(origin) => origin,
                    None => return response(StatusCode::FORBIDDEN, "browser origin is required\n"),
                };
                let body = match Limited::new(body, MAX_REGISTRATION_BODY_BYTES)
                    .collect()
                    .await
                {
                    Ok(body) => body.to_bytes(),
                    Err(_) => {
                        return response(StatusCode::BAD_REQUEST, "browser request is invalid\n");
                    }
                };
                self.finish(origin, &parts.headers, pairing_id, &body)
            }
            _ => response(StatusCode::NOT_FOUND, "not found\n"),
        }
    }

    fn options(&self, pairing_id: &str) -> GatewayHttpResponse {
        let options = self
            .pairings
            .lock()
            .map_err(|_| ())
            .and_then(|mut pairings| pairings.registration_options(pairing_id).map_err(|_| ()));
        match options {
            Ok(options) => json_response(
                StatusCode::OK,
                RegistrationOptionsResponse {
                    public_key: options,
                },
            ),
            Err(()) => response(StatusCode::UNAUTHORIZED, "browser pairing is unavailable\n"),
        }
    }

    fn finish(
        &self,
        origin: &str,
        headers: &hyper::HeaderMap,
        pairing_id: &str,
        body: &[u8],
    ) -> GatewayHttpResponse {
        if BrowserSameOriginSessionV1::require_mutation_origin(
            origin,
            self.exact_https_origin.as_ref(),
        )
        .is_err()
        {
            return response(StatusCode::FORBIDDEN, "browser origin is invalid\n");
        }
        if !is_json(headers) {
            return response(StatusCode::BAD_REQUEST, "browser request is invalid\n");
        }
        let request = match serde_json::from_slice::<FinishRegistrationRequest>(body) {
            Ok(request) => request,
            Err(_) => return response(StatusCode::BAD_REQUEST, "browser request is invalid\n"),
        };
        let browser_key_public_key =
            match decode_browser_key_public_key(&request.browser_key_public_key) {
                Ok(key) => key,
                Err(()) => {
                    return response(StatusCode::BAD_REQUEST, "browser request is invalid\n");
                }
            };
        let result = self
            .pairings
            .lock()
            .map_err(|_| "browser pairing is unavailable".to_owned())
            .and_then(|mut pairings| {
                pairings.finish_webauthn_and_admit(
                    self.authority.as_ref(),
                    self.verifier.as_ref(),
                    pairing_id,
                    &request.credential,
                    &browser_key_public_key,
                )
            });
        match result {
            Ok(_) => json_response(StatusCode::OK, EnrollmentResponse { enrolled: true }),
            Err(_) => response(StatusCode::UNAUTHORIZED, "browser pairing is unavailable\n"),
        }
    }
}

#[derive(Serialize)]
struct RegistrationOptionsResponse {
    public_key: CreationChallengeResponse,
}

#[derive(Serialize)]
struct EnrollmentResponse {
    enrolled: bool,
}

#[derive(Deserialize)]
struct FinishRegistrationRequest {
    credential: RegisterPublicKeyCredential,
    browser_key_public_key: String,
}

fn decode_browser_key_public_key(value: &str) -> Result<Vec<u8>, ()> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(value)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(value))
        .map_err(|_| ())?;
    (bytes.len() == 65 && bytes.first() == Some(&4))
        .then_some(bytes)
        .ok_or(())
}

fn registration_route(path: &str) -> Option<(&str, bool)> {
    let value = path.strip_prefix(PAIRING_PREFIX)?;
    if let Some(pairing_id) = value.strip_suffix(REGISTRATION_SUFFIX) {
        return valid_pairing_id(pairing_id).then_some((pairing_id, false));
    }
    if let Some(pairing_id) = value.strip_suffix(REGISTRATION_FINISH_SUFFIX) {
        return valid_pairing_id(pairing_id).then_some((pairing_id, true));
    }
    None
}

fn valid_pairing_id(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_json(headers: &hyper::HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json"))
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
            "browser pairing is unavailable\n",
        ),
    }
}

fn response(status: StatusCode, body: &'static str) -> GatewayHttpResponse {
    Response::builder()
        .status(status)
        .header(CACHE_CONTROL, "no-store")
        .body(full_gateway_body(body))
        .expect("Gateway browser response is valid")
}
