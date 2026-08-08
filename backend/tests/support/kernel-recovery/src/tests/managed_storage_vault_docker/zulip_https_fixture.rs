//! Loopback HTTPS Zulip API fixture with an explicit conformance-only CA.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, ExtendedKeyUsagePurpose, IsCa, KeyPair,
    KeyUsagePurpose,
};

use super::*;

pub(super) struct ZulipHttpsFixture {
    realm_url: String,
    ca_certificate_path: PathBuf,
    state: Arc<ZulipHttpsFixtureState>,
    shutdown: Arc<AtomicBool>,
    server: Option<std::thread::JoinHandle<()>>,
}

struct ZulipHttpsFixtureState {
    accepted_connections: AtomicU64,
    released_events: AtomicU64,
    served_events: AtomicU64,
    message_commands: AtomicU64,
    history_pages: AtomicU64,
    credential_v2_requests: AtomicU64,
}

impl ZulipHttpsFixture {
    pub(super) fn start(root: &Path) -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (ca_certificate, server_certificate, server_key) = certificate_chain();
        let certificate = server_certificate.der().clone();
        let key = rustls::pki_types::PrivateKeyDer::Pkcs8(
            rustls::pki_types::PrivatePkcs8KeyDer::from(server_key.serialize_der()),
        );
        let config = Arc::new(
            rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(vec![certificate], key)
                .expect("Zulip fixture TLS configuration"),
        );
        let ca_certificate_path = root.join("zulip-conformance-ca.pem");
        std::fs::write(&ca_certificate_path, ca_certificate.pem())
            .expect("write Zulip conformance CA");
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind Zulip HTTPS fixture");
        listener
            .set_nonblocking(true)
            .expect("configure Zulip HTTPS fixture");
        let port = listener.local_addr().expect("Zulip fixture address").port();
        let state = Arc::new(ZulipHttpsFixtureState {
            accepted_connections: AtomicU64::new(0),
            released_events: AtomicU64::new(0),
            served_events: AtomicU64::new(0),
            message_commands: AtomicU64::new(0),
            history_pages: AtomicU64::new(0),
            credential_v2_requests: AtomicU64::new(0),
        });
        let server_state = Arc::clone(&state);
        let shutdown = Arc::new(AtomicBool::new(false));
        let server_shutdown = Arc::clone(&shutdown);
        let server = std::thread::spawn(move || {
            serve(listener, config, &server_shutdown, server_state);
        });
        Self {
            realm_url: format!("https://localhost:{port}"),
            ca_certificate_path,
            state,
            shutdown,
            server: Some(server),
        }
    }

    pub(super) fn realm_url(&self) -> &str {
        &self.realm_url
    }

    pub(super) fn ca_certificate_path(&self) -> &Path {
        &self.ca_certificate_path
    }

    pub(super) fn accepted_connections(&self) -> u64 {
        self.state.accepted_connections.load(Ordering::Relaxed)
    }

    pub(super) fn release_next_event(&self) -> u64 {
        self.state.released_events.fetch_add(1, Ordering::Release) + 1
    }

    pub(super) fn served_events(&self) -> u64 {
        self.state.served_events.load(Ordering::Acquire)
    }

    pub(super) fn message_commands(&self) -> u64 {
        self.state.message_commands.load(Ordering::Acquire)
    }

    pub(super) fn history_pages(&self) -> u64 {
        self.state.history_pages.load(Ordering::Acquire)
    }

    pub(super) fn credential_v2_requests(&self) -> u64 {
        self.state.credential_v2_requests.load(Ordering::Acquire)
    }
}

impl Drop for ZulipHttpsFixture {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(server) = self.server.take() {
            server.join().expect("join Zulip HTTPS fixture");
        }
    }
}

fn certificate_chain() -> (Certificate, Certificate, KeyPair) {
    let mut ca_parameters =
        CertificateParams::new(Vec::<String>::new()).expect("empty CA subject alternative names");
    ca_parameters.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_parameters
        .key_usages
        .push(KeyUsagePurpose::DigitalSignature);
    ca_parameters.key_usages.push(KeyUsagePurpose::KeyCertSign);
    let ca_key = KeyPair::generate().expect("Zulip fixture CA key");
    let ca_certificate = ca_parameters
        .self_signed(&ca_key)
        .expect("self-signed Zulip fixture CA");

    let mut server_parameters =
        CertificateParams::new(vec!["localhost".to_owned()]).expect("localhost certificate");
    server_parameters
        .key_usages
        .push(KeyUsagePurpose::DigitalSignature);
    server_parameters
        .extended_key_usages
        .push(ExtendedKeyUsagePurpose::ServerAuth);
    let server_key = KeyPair::generate().expect("Zulip fixture server key");
    let server_certificate = server_parameters
        .signed_by(&server_key, &ca_certificate, &ca_key)
        .expect("CA-signed Zulip fixture certificate");
    (ca_certificate, server_certificate, server_key)
}

fn serve(
    listener: TcpListener,
    config: Arc<rustls::ServerConfig>,
    shutdown: &AtomicBool,
    state: Arc<ZulipHttpsFixtureState>,
) {
    let mut connections = Vec::new();
    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((tcp, _)) => {
                let connection_config = Arc::clone(&config);
                let connection_state = Arc::clone(&state);
                connections.push(std::thread::spawn(move || {
                    match serve_connection(tcp, connection_config, &connection_state) {
                        Ok(()) => {
                            connection_state
                                .accepted_connections
                                .fetch_add(1, Ordering::Relaxed);
                        }
                        Err(error) if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() => {
                            fixture_diagnostic(&format!(
                                "developer_zulip_fixture_connection_error={error}"
                            ));
                        }
                        Err(_) => {}
                    }
                }));
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("Zulip HTTPS fixture accept failed: {error}"),
        }
    }
    for connection in connections {
        connection.join().expect("join Zulip HTTPS connection");
    }
}

fn fixture_diagnostic(message: &str) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        let diagnostic = format!("{message}\n");
        let mut stderr = std::io::stderr().lock();
        let _ = std::io::Write::write_all(&mut stderr, diagnostic.as_bytes());
    }
}

fn serve_connection(
    tcp: TcpStream,
    config: Arc<rustls::ServerConfig>,
    state: &ZulipHttpsFixtureState,
) -> Result<(), std::io::Error> {
    tcp.set_nonblocking(false)?;
    tcp.set_read_timeout(Some(Duration::from_secs(2)))?;
    tcp.set_write_timeout(Some(Duration::from_secs(2)))?;
    let connection = rustls::ServerConnection::new(config).map_err(std::io::Error::other)?;
    let mut stream = rustls::StreamOwned::new(connection, tcp);
    let request = read_request(&mut stream)?;
    let credential_v2 = STANDARD.encode(b"managed-account@example.test:managed-zulip-api-key-v2");
    if request
        .windows(credential_v2.len())
        .any(|window| window == credential_v2.as_bytes())
    {
        state.credential_v2_requests.fetch_add(1, Ordering::Release);
    }
    let request_line = request
        .split(|byte| *byte == b'\n')
        .next()
        .and_then(|line| std::str::from_utf8(line).ok())
        .unwrap_or_default();
    let (status, body) = if request_line.starts_with("POST /api/v1/register ") {
        (
            "200 OK",
            r#"{"result":"success","msg":"","queue_id":"managed-zulip-queue","last_event_id":0}"#
                .to_owned(),
        )
    } else if request_line.starts_with("GET /api/v1/events?") {
        ("200 OK", next_event_response(state))
    } else if request_line.starts_with("GET /api/v1/messages?") {
        state.history_pages.fetch_add(1, Ordering::Release);
        ("200 OK", history_response(request_line))
    } else if request_line.starts_with("POST /api/v1/messages ") {
        state.message_commands.fetch_add(1, Ordering::Release);
        (
            "200 OK",
            r#"{"result":"success","msg":"","id":4242}"#.to_owned(),
        )
    } else {
        (
            "404 Not Found",
            r#"{"result":"error","msg":"unknown route"}"#.to_owned(),
        )
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn history_response(request_line: &str) -> String {
    if request_line.contains("anchor=9002") {
        return r#"{"result":"success","msg":"","found_oldest":true,"found_newest":false,"messages":[{"id":9001,"sender_id":71,"sender_email":"history@example.test","stream_id":44,"display_recipient":"operations","subject":"history","content":"oldest managed history","timestamp":100,"reactions":[]}]}"#.to_owned();
    }
    r#"{"result":"success","msg":"","found_oldest":false,"found_newest":true,"messages":[{"id":9002,"sender_id":72,"sender_email":"history@example.test","stream_id":44,"display_recipient":"operations","subject":"history","content":"searchable managed history","timestamp":101,"reactions":[{"user_id":73,"emoji_name":"thumbs_up","emoji_code":"1f44d","reaction_type":"unicode_emoji"}]},{"id":9003,"sender_id":73,"sender_email":"account@example.test","recipient_id":55,"content":"direct managed history","timestamp":102,"reactions":[]}]}"#.to_owned()
}

fn next_event_response(state: &ZulipHttpsFixtureState) -> String {
    let released = state.released_events.load(Ordering::Acquire);
    let event_id = state
        .served_events
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |served| {
            (served < released).then_some(served + 1)
        })
        .ok()
        .map(|served| served + 1);
    let Some(event_id) = event_id else {
        return r#"{"result":"success","msg":"","events":[]}"#.to_owned();
    };
    let provider_message_id = 9_100 + event_id;
    format!(
        r#"{{"result":"success","msg":"","events":[{{"id":{event_id},"type":"message","message":{{"id":{provider_message_id},"sender_id":73,"sender_email":"sender@example.test","stream_id":44,"subject":"managed","content":"managed Zulip observation {event_id}"}}}}]}}"#
    )
}

fn read_request(
    stream: &mut rustls::StreamOwned<rustls::ServerConnection, TcpStream>,
) -> Result<Vec<u8>, std::io::Error> {
    const MAX_REQUEST_BYTES: usize = 65_536;
    let mut request = Vec::new();
    let mut chunk = [0_u8; 4_096];
    loop {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..read]);
        if request.len() > MAX_REQUEST_BYTES {
            return Err(std::io::Error::other("Zulip fixture request is too large"));
        }
        if request_is_complete(&request) {
            break;
        }
    }
    (!request.is_empty())
        .then_some(request)
        .ok_or_else(|| std::io::Error::other("Zulip fixture request is empty"))
}

fn request_is_complete(request: &[u8]) -> bool {
    let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
        return false;
    };
    let headers = match std::str::from_utf8(&request[..header_end]) {
        Ok(headers) => headers,
        Err(_) => return false,
    };
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    request.len() >= header_end + 4 + content_length
}
