use super::common::*;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::identity::browser_gateway::ControlStoreBrowserAuthority;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, StatusCode};
use makosh_gateway_protocol::{
    v1::{ClientRealtimeEventV1, ClientRealtimeFrameV1, client_realtime_frame_v1::Frame},
    validation::validate_client_realtime_frame,
};
use makosh_gateway_runtime::{
    BrowserAuthenticationRouter, BrowserPairingRouter, BrowserRealtimeSubscriptionSource,
    ClientRealtimeSubscriptionV1, GatewayApplicationRouter, InMemoryBrowserRealtimeSource,
};
use makosh_gateway_session::{
    BrowserCredentialMaterialV1, BrowserGatewaySessionService, BrowserPairingManager,
    BrowserSameOriginSessionV1, BrowserSession, BrowserWebauthnVerifier, OwnerPairingApprovalV1,
};
use makosh_gateway_session_contract::{
    BrowserAssertionAuthority, BrowserAuthenticationAuthority, BrowserDeviceAuthority,
    BrowserEnrollmentAuthority, BrowserEnrollmentV1, GatewayIdentityFenceV1,
};
use makosh_kernel_control_store::BrowserDeviceEnrollmentV1;
use p256::ecdsa::{Signature, SigningKey, signature::Signer};
use sha2::{Digest, Sha256};
use webauthn_rs_core::proto::{COSEAlgorithm, COSEEC2Key, COSEKey, COSEKeyType, ECDSACurve};

struct ClosedReplaySource;

fn browser_authority(store: Arc<SqliteControlStore>) -> ControlStoreBrowserAuthority {
    ControlStoreBrowserAuthority::new(
        store,
        ManagedRuntimeSupervisor::new(Arc::new(AtomicBool::new(false))),
    )
}

impl BrowserRealtimeSubscriptionSource for ClosedReplaySource {
    fn subscribe(
        &self,
        session: &BrowserSession,
        after_cursor: Option<&str>,
    ) -> Result<ClientRealtimeSubscriptionV1, String> {
        if session.owner_id() != "owner-1" || after_cursor.is_some() {
            return Err("unexpected browser session".to_owned());
        }
        let (sender, receiver) = tokio::sync::broadcast::channel(1);
        drop(sender);
        ClientRealtimeSubscriptionV1::new(
            vec![ClientRealtimeFrameV1 {
                frame: Some(Frame::Event(ClientRealtimeEventV1 {
                    event_id: vec![7; 16],
                    cursor: "cursor-1".to_owned(),
                    contract_name: "makosh.client.status".to_owned(),
                    contract_version: 1,
                    event_kind: "status_changed".to_owned(),
                    occurred_at_unix_millis: 1,
                    causation_id: String::new(),
                    correlation_id: String::new(),
                    trace_id: String::new(),
                    payload: b"client-safe".to_vec(),
                })),
            }],
            receiver,
        )
    }
}

#[path = "browser_gateway_session/credential.rs"]
mod credential;

#[test]
fn browser_authentication_http_flow_issues_a_cookie_once_and_persists_the_counter() {
    let fixture = authentication_http_fixture();
    let (authentication_id, challenge, browser_key_challenge) =
        begin_browser_authentication(&fixture);
    assert_invalid_browser_local_key_proof_is_rejected(
        &fixture,
        &authentication_id,
        &challenge,
        &browser_key_challenge,
    );
    let cookie = finish_browser_authentication(
        &fixture,
        &authentication_id,
        &challenge,
        &browser_key_challenge,
    );
    assert_realtime_session_flow(&fixture, &cookie);
    assert_authentication_replay_is_rejected(
        &fixture,
        &authentication_id,
        &challenge,
        &browser_key_challenge,
    );
    std::fs::remove_dir_all(fixture.root).expect("remove fixture directory");
}

struct AuthenticationHttpFixture {
    root: std::path::PathBuf,
    store: Arc<SqliteControlStore>,
    router: GatewayApplicationRouter<ControlStoreBrowserAuthority, ClosedReplaySource>,
    runtime: tokio::runtime::Runtime,
}

fn authentication_http_fixture() -> AuthenticationHttpFixture {
    let root = unique_target_root("makosh-browser-gateway-http-authentication");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let path = root.join("control.sqlite");
    let store =
        Arc::new(SqliteControlStore::create(&path, "instance-browser", 1).expect("create store"));
    store
        .claim_initial_owner(&InitialOwnerIdentity::new("owner-1", "desktop-1", [4; 65]))
        .expect("claim initial owner");
    store
        .admit_browser_device(
            &BrowserDeviceEnrollmentV1::new(
                makosh_kernel_control_store::BrowserDeviceEnrollmentInputV1 {
                    owner_id: "owner-1".to_owned(),
                    device_id: "browser-1".to_owned(),
                    credential_id: vec![1],
                    cose_public_key: valid_browser_cose_key(),
                    browser_key_public_key: valid_browser_local_key(),
                    rp_id: "hub.local".to_owned(),
                    sign_count: 0,
                    backup_eligible: false,
                    backup_state: false,
                },
            )
            .expect("valid browser enrollment"),
            1,
        )
        .expect("admit browser");

    let verifier =
        BrowserWebauthnVerifier::new("hub.local", "https://hub.local").expect("verifier");
    let service = BrowserGatewaySessionService::new(
        browser_authority(Arc::clone(&store)),
        verifier,
        "https://hub.local",
    )
    .expect("browser session service");
    let service = std::sync::Arc::new(service);
    let router = GatewayApplicationRouter::new(true, service, ClosedReplaySource);
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let health = runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/healthz")
                .body(Full::new(Bytes::new()))
                .expect("health request"),
        ),
    );
    assert_eq!(health.status(), StatusCode::OK);
    AuthenticationHttpFixture {
        root,
        store,
        router,
        runtime,
    }
}

pub(super) fn admit_browser_test_device(store: &Arc<SqliteControlStore>, logical_owner_id: &str) {
    admit_browser_test_device_with_identity(store, logical_owner_id, "browser-1", 1, 7);
}

pub(super) fn admit_secondary_browser_test_device(
    store: &Arc<SqliteControlStore>,
    logical_owner_id: &str,
) {
    admit_browser_test_device_with_identity(store, logical_owner_id, "browser-2", 2, 8);
}

fn admit_browser_test_device_with_identity(
    store: &Arc<SqliteControlStore>,
    logical_owner_id: &str,
    device_id: &str,
    credential_id: u8,
    signing_key_byte: u8,
) {
    store
        .admit_browser_device(
            &BrowserDeviceEnrollmentV1::new(
                makosh_kernel_control_store::BrowserDeviceEnrollmentInputV1 {
                    owner_id: logical_owner_id.to_owned(),
                    device_id: device_id.to_owned(),
                    credential_id: vec![credential_id],
                    cose_public_key: browser_cose_key(signing_key_byte),
                    browser_key_public_key: browser_local_key(signing_key_byte),
                    rp_id: "hub.local".to_owned(),
                    sign_count: 0,
                    backup_eligible: false,
                    backup_state: false,
                },
            )
            .expect("valid browser enrollment"),
            1,
        )
        .expect("admit browser device");
}

pub(super) fn authenticate_gateway_router(
    router: &GatewayApplicationRouter<ControlStoreBrowserAuthority, InMemoryBrowserRealtimeSource>,
    runtime: &tokio::runtime::Runtime,
) -> String {
    authenticate_gateway_router_with_sign_count(router, runtime, 1)
}

pub(super) fn authenticate_gateway_router_with_sign_count(
    router: &GatewayApplicationRouter<ControlStoreBrowserAuthority, InMemoryBrowserRealtimeSource>,
    runtime: &tokio::runtime::Runtime,
    sign_count: u32,
) -> String {
    authenticate_gateway_router_for_device(router, runtime, 1, 7, sign_count)
}

pub(super) fn authenticate_secondary_gateway_router(
    router: &GatewayApplicationRouter<ControlStoreBrowserAuthority, InMemoryBrowserRealtimeSource>,
    runtime: &tokio::runtime::Runtime,
) -> String {
    authenticate_gateway_router_for_device(router, runtime, 2, 8, 1)
}

fn authenticate_gateway_router_for_device(
    router: &GatewayApplicationRouter<ControlStoreBrowserAuthority, InMemoryBrowserRealtimeSource>,
    runtime: &tokio::runtime::Runtime,
    credential_id: u8,
    signing_key_byte: u8,
    sign_count: u32,
) -> String {
    let begin = runtime.block_on(router.route(begin_authentication_request_for_credential(
        "https://hub.local",
        credential_id,
    )));
    assert_eq!(begin.status(), StatusCode::OK);
    let begin_body = runtime
        .block_on(begin.into_body().collect())
        .expect("begin response body")
        .to_bytes();
    let ceremony: serde_json::Value = serde_json::from_slice(&begin_body).expect("begin JSON");
    let authentication_id = ceremony["authentication_id"]
        .as_str()
        .expect("authentication ID");
    let challenge = ceremony["public_key"]["challenge"]
        .as_str()
        .expect("WebAuthn challenge");
    let browser_key_challenge = ceremony["browser_key_challenge"]
        .as_str()
        .expect("browser key challenge");
    let response = runtime.block_on(router.route(
        finish_authentication_request_with_browser_key_signature(
            authentication_id,
            &signed_browser_assertion_for_key(
                challenge,
                sign_count,
                credential_id,
                signing_key_byte,
            ),
            &signed_browser_key_proof_for_key(browser_key_challenge, signing_key_byte),
        ),
    ));
    assert_eq!(response.status(), StatusCode::OK);
    response
        .headers()
        .get("set-cookie")
        .expect("secure browser cookie")
        .to_str()
        .expect("cookie header")
        .split(';')
        .next()
        .expect("session cookie pair")
        .to_owned()
}

fn begin_browser_authentication(fixture: &AuthenticationHttpFixture) -> (String, String, String) {
    let begin = fixture.runtime.block_on(
        fixture
            .router
            .route(begin_authentication_request("https://hub.local")),
    );
    assert_eq!(begin.status(), StatusCode::OK);
    let begin_body = fixture
        .runtime
        .block_on(begin.into_body().collect())
        .expect("begin response body")
        .to_bytes();
    let ceremony: serde_json::Value = serde_json::from_slice(&begin_body).expect("begin JSON");
    let authentication_id = ceremony["authentication_id"]
        .as_str()
        .expect("authentication ID");
    let challenge = ceremony["public_key"]["challenge"]
        .as_str()
        .expect("WebAuthn challenge");
    let browser_key_challenge = ceremony["browser_key_challenge"]
        .as_str()
        .expect("browser key challenge");
    (
        authentication_id.to_owned(),
        challenge.to_owned(),
        browser_key_challenge.to_owned(),
    )
}

fn finish_browser_authentication(
    fixture: &AuthenticationHttpFixture,
    authentication_id: &str,
    challenge: &str,
    browser_key_challenge: &str,
) -> String {
    let request = finish_authentication_request(
        authentication_id,
        &signed_browser_assertion(challenge, 1),
        browser_key_challenge,
    );
    let response = fixture.runtime.block_on(fixture.router.route(request));
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .expect("cache control"),
        "no-store"
    );
    let cookie = response
        .headers()
        .get("set-cookie")
        .expect("secure browser cookie")
        .to_str()
        .expect("cookie header");
    assert!(cookie.starts_with("__Host-makosh-session="));
    assert!(cookie.contains("Secure; HttpOnly; SameSite=Strict"));
    assert_eq!(
        fixture
            .store
            .browser_device_identity("browser-1")
            .expect("read browser identity")
            .expect("browser identity exists")
            .enrollment()
            .sign_count(),
        1
    );
    cookie
        .split(';')
        .next()
        .expect("session cookie pair")
        .to_owned()
}

fn assert_realtime_session_flow(fixture: &AuthenticationHttpFixture, session_cookie: &str) {
    let realtime = fixture.runtime.block_on(
        fixture.router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", session_cookie)
                .body(Full::new(Bytes::new()))
                .expect("realtime request"),
        ),
    );
    assert_eq!(realtime.status(), StatusCode::OK);
    assert_eq!(
        realtime
            .headers()
            .get("content-type")
            .expect("SSE content type"),
        "text/event-stream"
    );
    let realtime_body = fixture
        .runtime
        .block_on(realtime.into_body().collect())
        .expect("SSE response body")
        .to_bytes();
    let realtime_body = std::str::from_utf8(&realtime_body).expect("SSE UTF-8");
    assert!(realtime_body.starts_with("id: cursor-1\nevent: makosh.realtime.v1\ndata: "));
    assert!(realtime_body.ends_with("\n\n"));
    assert!(!realtime_body.contains("client-safe"));
    let invalid_replay = fixture.runtime.block_on(
        fixture.router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events?cursor=cursor-1")
                .header("cookie", session_cookie)
                .body(Full::new(Bytes::new()))
                .expect("realtime request"),
        ),
    );
    assert_eq!(invalid_replay.status(), StatusCode::NOT_FOUND);
}

fn assert_authentication_replay_is_rejected(
    fixture: &AuthenticationHttpFixture,
    authentication_id: &str,
    challenge: &str,
    browser_key_challenge: &str,
) {
    let replay = fixture
        .runtime
        .block_on(fixture.router.route(finish_authentication_request(
            authentication_id,
            &signed_browser_assertion(challenge, 2),
            browser_key_challenge,
        )));
    assert_eq!(replay.status(), StatusCode::UNAUTHORIZED);
}

fn assert_invalid_browser_local_key_proof_is_rejected(
    fixture: &AuthenticationHttpFixture,
    authentication_id: &str,
    challenge: &str,
    browser_key_challenge: &str,
) {
    let mut invalid_signature = signed_browser_key_proof(browser_key_challenge);
    let replacement = if invalid_signature.starts_with('A') {
        "B"
    } else {
        "A"
    };
    invalid_signature.replace_range(0..1, replacement);
    let response = fixture.runtime.block_on(fixture.router.route(
        finish_authentication_request_with_browser_key_signature(
            authentication_id,
            &signed_browser_assertion(challenge, 1),
            &invalid_signature,
        ),
    ));
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[path = "browser_gateway_session/realtime_contract.rs"]
mod realtime_contract;

#[path = "browser_gateway_session/connect_status.rs"]
mod connect_status;

fn begin_authentication_request(origin: &str) -> Request<Full<Bytes>> {
    begin_authentication_request_for_credential(origin, 1)
}

fn begin_authentication_request_for_credential(
    origin: &str,
    credential_id: u8,
) -> Request<Full<Bytes>> {
    Request::builder()
        .method("POST")
        .uri("/browser/v1/authentication/begin")
        .header("origin", origin)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(format!(
            r#"{{"credential_id":"{}"}}"#,
            base64_url_encode(&[credential_id]),
        ))))
        .expect("browser authentication request")
}

fn finish_authentication_request(
    authentication_id: &str,
    assertion: &str,
    browser_key_challenge: &str,
) -> Request<Full<Bytes>> {
    finish_authentication_request_with_browser_key_signature(
        authentication_id,
        assertion,
        &signed_browser_key_proof(browser_key_challenge),
    )
}

fn finish_authentication_request_with_browser_key_signature(
    authentication_id: &str,
    assertion: &str,
    browser_key_signature: &str,
) -> Request<Full<Bytes>> {
    Request::builder()
        .method("POST")
        .uri(format!(
            "/browser/v1/authentication/{authentication_id}/finish"
        ))
        .header("origin", "https://hub.local")
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(format!(
            r#"{{"credential":{assertion},"browser_key_signature":"{}"}}"#,
            browser_key_signature,
        ))))
        .expect("browser authentication finish request")
}

fn signed_browser_assertion(challenge: &str, sign_count: u32) -> String {
    signed_browser_assertion_for_key(challenge, sign_count, 1, 7)
}

fn signed_browser_assertion_for_key(
    challenge: &str,
    sign_count: u32,
    credential_id: u8,
    signing_key_byte: u8,
) -> String {
    let client_data = format!(
        r#"{{"type":"webauthn.get","challenge":"{challenge}","origin":"https://hub.local","crossOrigin":false}}"#
    );
    let client_data_hash = Sha256::digest(client_data.as_bytes());
    let mut authenticator_data = Sha256::digest(b"hub.local").to_vec();
    authenticator_data.push(0x05);
    authenticator_data.extend_from_slice(&sign_count.to_be_bytes());
    let mut signed_data = authenticator_data.clone();
    signed_data.extend_from_slice(&client_data_hash);
    let signature: Signature = browser_signing_key_for_byte(signing_key_byte).sign(&signed_data);
    let credential_id = base64_url_encode(&[credential_id]);
    format!(
        r#"{{"id":"{credential_id}","rawId":"{credential_id}","type":"public-key","response":{{"authenticatorData":"{}","clientDataJSON":"{}","signature":"{}","userHandle":null}},"clientExtensionResults":{{}}}}"#,
        base64_url_encode(&authenticator_data),
        base64_url_encode(client_data.as_bytes()),
        base64_url_encode(signature.to_der().as_bytes()),
    )
}

fn base64_url_encode(value: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut encoded = String::with_capacity((value.len() * 4).div_ceil(3));
    for chunk in value.chunks(3) {
        let first = chunk[0];
        encoded.push(char::from(ALPHABET[usize::from(first >> 2)]));
        encoded.push(char::from(
            ALPHABET
                [usize::from(((first & 0x03) << 4) | (chunk.get(1).copied().unwrap_or(0) >> 4))],
        ));
        if let Some(second) = chunk.get(1) {
            encoded.push(char::from(
                ALPHABET[usize::from(
                    ((second & 0x0f) << 2) | (chunk.get(2).copied().unwrap_or(0) >> 6),
                )],
            ));
        }
        if let Some(third) = chunk.get(2) {
            encoded.push(char::from(ALPHABET[usize::from(third & 0x3f)]));
        }
    }
    encoded
}

pub(super) fn browser_signing_key() -> SigningKey {
    browser_signing_key_for_byte(7)
}

fn browser_signing_key_for_byte(value: u8) -> SigningKey {
    SigningKey::from_bytes((&[value; 32]).into()).expect("test signing key")
}

fn valid_browser_local_key() -> Vec<u8> {
    browser_local_key(7)
}

fn browser_local_key(signing_key_byte: u8) -> Vec<u8> {
    browser_signing_key_for_byte(signing_key_byte)
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .to_vec()
}

fn signed_browser_key_proof(challenge: &str) -> String {
    signed_browser_key_proof_for_key(challenge, 7)
}

fn signed_browser_key_proof_for_key(challenge: &str, signing_key_byte: u8) -> String {
    let raw = decode_base64_url(challenge);
    let signature: Signature = browser_signing_key_for_byte(signing_key_byte).sign(&raw);
    base64_url_encode(&signature.to_bytes())
}

fn decode_base64_url(value: &str) -> Vec<u8> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = Vec::with_capacity(value.len() * 3 / 4);
    let mut accumulator = 0_u32;
    let mut bits = 0_u8;
    for byte in value.bytes() {
        let value = ALPHABET
            .iter()
            .position(|candidate| *candidate == byte)
            .expect("base64url character") as u32;
        accumulator = (accumulator << 6) | value;
        bits += 6;
        while bits >= 8 {
            bits -= 8;
            output.push((accumulator >> bits) as u8);
        }
    }
    output
}

fn valid_browser_cose_key() -> Vec<u8> {
    browser_cose_key(7)
}

fn browser_cose_key(signing_key_byte: u8) -> Vec<u8> {
    let signing_key = browser_signing_key_for_byte(signing_key_byte);
    let point = signing_key.verifying_key().to_sec1_point(false);
    let key = COSEKey {
        type_: COSEAlgorithm::ES256,
        key: COSEKeyType::EC_EC2(COSEEC2Key {
            curve: ECDSACurve::SECP256R1,
            x: point.x().expect("P-256 x coordinate").to_vec().into(),
            y: point.y().expect("P-256 y coordinate").to_vec().into(),
        }),
    };
    serde_cbor_2::to_vec(&key).expect("serialize COSE key")
}

#[path = "browser_gateway_session/authority.rs"]
mod authority;

#[path = "browser_gateway_session/enrollment.rs"]
mod enrollment;

#[path = "browser_gateway_session/pairing.rs"]
mod pairing;

#[path = "browser_gateway_session/webauthn.rs"]
mod webauthn;

#[path = "browser_gateway_session/pairing_http.rs"]
mod pairing_http;

#[path = "browser_gateway_session/session_policy.rs"]
mod session_policy;

#[path = "browser_gateway_session/owner_epoch.rs"]
mod owner_epoch;
