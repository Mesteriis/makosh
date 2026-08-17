//! Loopback TLS Gmail OAuth provider fixture with secret-safe request evidence.

use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rcgen::generate_simple_self_signed;
use rustls::{
    ServerConfig, ServerConnection, StreamOwned,
    pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer},
};
use sha2::{Digest, Sha256};

const MAX_HTTP_LINE_BYTES: usize = 8 * 1024;
const MAX_REQUEST_BODY_BYTES: usize = 64 * 1024;
const OAUTH_FIXTURE_HOST: &str = "127.0.0.1";
const GMAIL_OAUTH_SCOPE: &str = "openid email https://www.googleapis.com/auth/gmail.modify https://www.googleapis.com/auth/gmail.send https://www.googleapis.com/auth/contacts";
const GMAIL_PERMANENT_DELETE_OAUTH_SCOPE: &str = "openid email https://mail.google.com/";

struct GmailOAuthCapturedRequestV1 {
    path: String,
    content_type: String,
    form: BTreeMap<String, String>,
}

pub(super) struct GmailOAuthAuthorizationMaterialV1 {
    pub(super) state: String,
    pub(super) code_challenge: String,
}

pub(super) struct MailGmailOAuthFixture {
    port: u16,
    ca_certificate_pem: String,
    accepted_requests: Arc<AtomicUsize>,
    completed_responses: Arc<AtomicUsize>,
    requests: Arc<Mutex<Vec<GmailOAuthCapturedRequestV1>>>,
    shutdown: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl MailGmailOAuthFixture {
    pub(super) fn start_successful_rotation() -> Self {
        Self::start()
    }

    fn start() -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let certified = generate_simple_self_signed(vec![OAUTH_FIXTURE_HOST.to_owned()])
            .expect("generate Gmail OAuth fixture certificate");
        let ca_certificate_pem = certified.cert.pem();
        let certificate = certified.cert.der().clone();
        let key =
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(certified.key_pair.serialize_der()));
        let server = Arc::new(
            ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(vec![certificate], key)
                .expect("configure Gmail OAuth fixture TLS"),
        );
        let listener =
            TcpListener::bind(("127.0.0.1", 0)).expect("bind Gmail OAuth fixture listener");
        listener
            .set_nonblocking(true)
            .expect("configure Gmail OAuth fixture listener");
        let port = listener
            .local_addr()
            .expect("Gmail OAuth fixture address")
            .port();
        let accepted_requests = Arc::new(AtomicUsize::new(0));
        let completed_responses = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_requests = Arc::clone(&requests);
        let worker_accepted = Arc::clone(&accepted_requests);
        let worker_completed = Arc::clone(&completed_responses);
        let worker_shutdown = Arc::clone(&shutdown);
        let worker = thread::spawn(move || {
            while !worker_shutdown.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        configure_stream(&stream);
                        let connection = ServerConnection::new(Arc::clone(&server))
                            .expect("Gmail OAuth TLS session");
                        let mut stream = StreamOwned::new(connection, stream);
                        serve_connection(
                            &mut stream,
                            &worker_accepted,
                            &worker_completed,
                            &worker_requests,
                        );
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("Gmail OAuth fixture accept failed: {error}"),
                }
            }
        });
        Self {
            port,
            ca_certificate_pem,
            accepted_requests,
            completed_responses,
            requests,
            shutdown,
            worker: Some(worker),
        }
    }

    pub(super) fn port(&self) -> u16 {
        self.port
    }

    pub(super) fn host(&self) -> &'static str {
        OAUTH_FIXTURE_HOST
    }

    pub(super) fn ca_certificate_pem(&self) -> &str {
        &self.ca_certificate_pem
    }

    pub(super) fn request_count(&self) -> usize {
        self.accepted_requests.load(Ordering::SeqCst)
    }

    pub(super) fn wait_for_request_count(&self, expected: usize) {
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while self.request_count() < expected {
            assert!(
                std::time::Instant::now() < deadline,
                "Gmail OAuth fixture did not receive the expected request"
            );
            thread::sleep(Duration::from_millis(25));
        }
    }

    pub(super) fn wait_for_response_count(&self, expected: usize) {
        let deadline = std::time::Instant::now() + Duration::from_secs(15);
        while self.completed_responses.load(Ordering::SeqCst) < expected {
            assert!(
                std::time::Instant::now() < deadline,
                "Gmail OAuth fixture did not complete the expected response"
            );
            thread::sleep(Duration::from_millis(25));
        }
    }

    pub(super) fn authorization_material(
        &self,
        authorization_url: &str,
    ) -> GmailOAuthAuthorizationMaterialV1 {
        self.authorization_material_with_scope(authorization_url, GMAIL_OAUTH_SCOPE)
    }

    pub(super) fn permanent_delete_authorization_material(
        &self,
        authorization_url: &str,
    ) -> GmailOAuthAuthorizationMaterialV1 {
        self.authorization_material_with_scope(
            authorization_url,
            GMAIL_PERMANENT_DELETE_OAUTH_SCOPE,
        )
    }

    fn authorization_material_with_scope(
        &self,
        authorization_url: &str,
        expected_scope: &str,
    ) -> GmailOAuthAuthorizationMaterialV1 {
        let (base, query) = authorization_url
            .split_once('?')
            .expect("Gmail OAuth authorization URL query");
        assert!(
            base == format!("https://{}:{}/authorize", self.host(), self.port),
            "Gmail OAuth authorization endpoint differs from signed settings"
        );
        let query = parse_form(query.as_bytes());
        assert!(
            query.len() == 9,
            "Gmail OAuth authorization query is not exact"
        );
        assert_form_value(&query, "client_id", "managed-mail-gmail-client");
        assert_form_value(&query, "redirect_uri", "https://127.0.0.1/oauth/callback");
        assert_form_value(&query, "response_type", "code");
        assert_form_value(&query, "code_challenge_method", "S256");
        assert_form_value(&query, "access_type", "offline");
        assert_form_value(&query, "prompt", "consent");
        assert_form_value(&query, "scope", expected_scope);
        GmailOAuthAuthorizationMaterialV1 {
            state: query.get("state").expect("Gmail OAuth state").clone(),
            code_challenge: query
                .get("code_challenge")
                .expect("Gmail OAuth code challenge")
                .clone(),
        }
    }

    pub(super) fn assert_authorization_code_exchange(
        &self,
        request_index: usize,
        expected_authorization_code: &str,
        expected_code_challenge: &str,
    ) {
        let requests = self.requests.lock().expect("lock Gmail OAuth requests");
        let request = requests
            .get(request_index)
            .expect("Gmail OAuth authorization-code request");
        assert_request_shape(request, 5);
        assert_form_value(&request.form, "grant_type", "authorization_code");
        assert_form_value(&request.form, "client_id", "managed-mail-gmail-client");
        assert_form_value(
            &request.form,
            "redirect_uri",
            "https://127.0.0.1/oauth/callback",
        );
        assert_form_value(&request.form, "code", expected_authorization_code);
        let verifier = request
            .form
            .get("code_verifier")
            .expect("Gmail OAuth code verifier");
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        assert!(
            challenge == expected_code_challenge,
            "Gmail OAuth token exchange did not use the hidden PKCE verifier"
        );
    }

    pub(super) fn assert_refresh_exchange(&self, request_index: usize) {
        let requests = self.requests.lock().expect("lock Gmail OAuth requests");
        let request = requests
            .get(request_index)
            .expect("Gmail OAuth refresh request");
        assert_request_shape(request, 3);
        assert_form_value(&request.form, "grant_type", "refresh_token");
        assert_form_value(&request.form, "client_id", "managed-mail-gmail-client");
        assert_form_value(
            &request.form,
            "refresh_token",
            "managed-mail-gmail-refresh-v1",
        );
    }
}

impl Drop for MailGmailOAuthFixture {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            let result = worker.join();
            if !thread::panicking() {
                result.expect("join Gmail OAuth fixture");
            }
        }
    }
}

fn configure_stream(stream: &TcpStream) {
    stream
        .set_nonblocking(false)
        .expect("configure blocking Gmail OAuth fixture connection");
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("configure Gmail OAuth fixture read timeout");
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .expect("configure Gmail OAuth fixture write timeout");
}

fn serve_connection(
    stream: &mut StreamOwned<ServerConnection, TcpStream>,
    accepted_requests: &AtomicUsize,
    completed_responses: &AtomicUsize,
    requests: &Mutex<Vec<GmailOAuthCapturedRequestV1>>,
) {
    let (request_line, headers, body) = read_http_request(stream);
    let mut request_parts = request_line.split_whitespace();
    assert_eq!(request_parts.next(), Some("POST"));
    let path = request_parts
        .next()
        .expect("Gmail OAuth request path")
        .to_owned();
    assert_eq!(request_parts.next(), Some("HTTP/1.1"));
    assert_eq!(request_parts.next(), None);
    let content_type = headers
        .get("content-type")
        .expect("Gmail OAuth request content type")
        .clone();
    requests
        .lock()
        .expect("lock Gmail OAuth requests")
        .push(GmailOAuthCapturedRequestV1 {
            path,
            content_type,
            form: parse_form(&body),
        });
    let request_index = accepted_requests.fetch_add(1, Ordering::SeqCst);
    if request_index == 3 {
        return;
    }
    if request_index == 0 {
        thread::sleep(Duration::from_secs(3));
    }
    let response_body = match request_index {
        0 => br#"{"access_token":"managed-mail-gmail-access-v1","refresh_token":"managed-mail-gmail-refresh-v1","expires_in":3600,"token_type":"Bearer","scope":"openid email https://www.googleapis.com/auth/gmail.modify https://www.googleapis.com/auth/gmail.send https://www.googleapis.com/auth/contacts"}"#.as_slice(),
        1 => br#"{"access_token":"managed-mail-gmail-access-v2","refresh_token":"managed-mail-gmail-refresh-v2","expires_in":3600,"token_type":"Bearer","scope":"openid email https://www.googleapis.com/auth/gmail.modify https://www.googleapis.com/auth/gmail.send https://www.googleapis.com/auth/contacts"}"#.as_slice(),
        2 => br#"{"access_token":"managed-mail-gmail-access-under-scoped","refresh_token":"managed-mail-gmail-refresh-under-scoped","expires_in":3600,"token_type":"Bearer","scope":"openid email https://www.googleapis.com/auth/gmail.modify https://www.googleapis.com/auth/gmail.send"}"#.as_slice(),
        4 => br#"{"access_token":"managed-mail-gmail-access-v3","refresh_token":"managed-mail-gmail-refresh-v3","expires_in":3600,"token_type":"Bearer","scope":"openid email https://www.googleapis.com/auth/gmail.modify https://www.googleapis.com/auth/gmail.send https://www.googleapis.com/auth/contacts"}"#.as_slice(),
        _ => br#"{"error":"unexpected_request"}"#.as_slice(),
    };
    let status = if matches!(request_index, 0 | 1 | 2 | 4) {
        "200 OK"
    } else {
        "400 Bad Request"
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response_body.len()
    );
    stream
        .write_all(response.as_bytes())
        .and_then(|_| stream.write_all(response_body))
        .and_then(|_| stream.flush())
        .expect("write Gmail OAuth response");
    completed_responses.fetch_add(1, Ordering::SeqCst);
}

fn assert_request_shape(request: &GmailOAuthCapturedRequestV1, expected_fields: usize) {
    assert!(
        request.path == "/token",
        "Gmail OAuth request escaped the signed token path"
    );
    assert!(
        request.content_type == "application/x-www-form-urlencoded",
        "Gmail OAuth request content type is invalid"
    );
    assert!(
        request.form.len() == expected_fields,
        "Gmail OAuth form contains unexpected fields"
    );
}

fn assert_form_value(form: &BTreeMap<String, String>, name: &str, expected: &str) {
    assert!(
        form.get(name).is_some_and(|value| value == expected),
        "Gmail OAuth form field is absent or invalid"
    );
}

fn parse_form(body: &[u8]) -> BTreeMap<String, String> {
    let body = std::str::from_utf8(body).expect("Gmail OAuth form encoding");
    let mut form = BTreeMap::new();
    for pair in body.split('&') {
        let (name, value) = pair.split_once('=').expect("Gmail OAuth form field");
        assert!(
            form.insert(decode_form_component(name), decode_form_component(value))
                .is_none(),
            "duplicate Gmail OAuth form field"
        );
    }
    form
}

fn decode_form_component(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' => {
                assert!(index + 2 < bytes.len(), "truncated Gmail OAuth form escape");
                decoded.push((decode_hex(bytes[index + 1]) << 4) | decode_hex(bytes[index + 2]));
                index += 3;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded).expect("Gmail OAuth form UTF-8")
}

fn decode_hex(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        b'A'..=b'F' => value - b'A' + 10,
        _ => panic!("invalid Gmail OAuth form escape"),
    }
}

fn read_http_request(
    stream: &mut StreamOwned<ServerConnection, TcpStream>,
) -> (String, BTreeMap<String, String>, Vec<u8>) {
    let max_request_bytes = MAX_HTTP_LINE_BYTES + MAX_REQUEST_BODY_BYTES;
    let read_deadline = Instant::now() + Duration::from_secs(14);
    let mut request = Vec::new();
    let mut chunk = [0_u8; 4 * 1024];
    loop {
        let read = match stream.read(&mut chunk) {
            Ok(read) => read,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) && Instant::now() < read_deadline =>
            {
                continue;
            }
            Err(error) => panic!("read Gmail OAuth HTTP request: {error}"),
        };
        assert!(read > 0, "Gmail OAuth HTTP request ended early");
        request.extend_from_slice(&chunk[..read]);
        assert!(
            request.len() <= max_request_bytes,
            "Gmail OAuth HTTP request exceeded its bound"
        );
        let Some(header_end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") else {
            continue;
        };
        let head =
            std::str::from_utf8(&request[..header_end]).expect("Gmail OAuth request headers");
        let mut lines = head.split("\r\n");
        let request_line = lines.next().expect("Gmail OAuth request line").to_owned();
        let mut headers = BTreeMap::new();
        for line in lines {
            let (name, value) = line
                .split_once(':')
                .expect("Gmail OAuth request header shape");
            assert!(
                headers
                    .insert(name.to_ascii_lowercase(), value.trim().to_owned())
                    .is_none(),
                "duplicate Gmail OAuth request header"
            );
        }
        let content_length = headers
            .get("content-length")
            .expect("Gmail OAuth request content length")
            .parse::<usize>()
            .expect("Gmail OAuth request content length value");
        assert!(content_length <= MAX_REQUEST_BODY_BYTES);
        let body_start = header_end + 4;
        if request.len() < body_start + content_length {
            continue;
        }
        assert_eq!(
            request.len(),
            body_start + content_length,
            "Gmail OAuth request has trailing bytes"
        );
        return (request_line, headers, request[body_start..].to_vec());
    }
}
