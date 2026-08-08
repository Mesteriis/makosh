//! One-shot TLS listener for the Linux server bootstrap ceremony.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use makosh_kernel_control_store_sqlite::SqliteControlStore;
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use sha2::{Digest, Sha256};

use crate::identity::server_pairing::proof as server_pairing_proof;

const MAX_FAILED_REQUESTS: u8 = 8;
const MAX_REQUEST_HEADER_BYTES: usize = 16 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

pub struct TlsServer {
    configuration: Arc<ServerConfig>,
    certificate_sha256: [u8; 32],
}

impl TlsServer {
    pub fn generate() -> Result<Self, String> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let certified_key = generate_simple_self_signed(vec!["localhost".to_owned()])
            .map_err(|error| error.to_string())?;
        let certificate = certified_key.cert.der().clone();
        let certificate_sha256: [u8; 32] = Sha256::digest(certificate.as_ref()).into();
        let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
            certified_key.key_pair.serialize_der(),
        ));
        let configuration = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![certificate], private_key)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            configuration: Arc::new(configuration),
            certificate_sha256,
        })
    }

    #[must_use]
    pub fn certificate_sha256(&self) -> &[u8; 32] {
        &self.certificate_sha256
    }
}

pub struct ListenerConfig<'a> {
    pub store: &'a SqliteControlStore,
    pub tls_server: TlsServer,
    pub listen_address: SocketAddr,
    pub idle_timeout: Duration,
    pub token: [u8; 32],
    pub challenge: [u8; 32],
}

pub fn listen(config: ListenerConfig<'_>) -> Result<(), String> {
    let listener = TcpListener::bind(config.listen_address).map_err(|error| error.to_string())?;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let address = listener.local_addr().map_err(|error| error.to_string())?;
    println!("server_bootstrap_pairing_endpoint=https://{address}");
    println!(
        "server_bootstrap_pairing_tls_certificate_sha256={}",
        server_pairing_proof::hex(config.tls_server.certificate_sha256())
    );
    println!(
        "server_bootstrap_pairing_token={}",
        server_pairing_proof::hex(&config.token)
    );
    println!(
        "server_bootstrap_pairing_challenge={}",
        server_pairing_proof::hex(&config.challenge)
    );

    let deadline = Instant::now()
        .checked_add(config.idle_timeout)
        .ok_or_else(|| "server pairing listener timeout overflow".to_owned())?;
    let mut failures = 0_u8;
    while Instant::now() < deadline {
        match listener.accept() {
            Ok((stream, _)) => match handle_connection(
                stream,
                Arc::clone(&config.tls_server.configuration),
                config.store,
                &config.challenge,
            ) {
                Ok(ConnectionOutcome::EnrollmentCompleted) => {
                    println!("server_bootstrap_pairing_consumed=true");
                    return Ok(());
                }
                Ok(ConnectionOutcome::Rejected) | Err(_) => {
                    failures = failures.saturating_add(1);
                    if failures >= MAX_FAILED_REQUESTS {
                        return Err("server pairing listener rate limit exceeded".to_owned());
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("server pairing listener timed out".to_owned())
}

fn handle_connection(
    stream: TcpStream,
    tls_config: Arc<ServerConfig>,
    store: &SqliteControlStore,
    challenge: &[u8; 32],
) -> Result<ConnectionOutcome, String> {
    stream
        .set_nonblocking(false)
        .map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(REQUEST_TIMEOUT))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(REQUEST_TIMEOUT))
        .map_err(|error| error.to_string())?;
    let connection = ServerConnection::new(tls_config).map_err(|error| error.to_string())?;
    let mut tls = StreamOwned::new(connection, stream);
    let request = read_request(&mut tls)?;
    let outcome = match process_request(&request, store, challenge) {
        Ok(()) => ConnectionOutcome::EnrollmentCompleted,
        Err(_) => ConnectionOutcome::Rejected,
    };
    let (status, body) = match outcome {
        ConnectionOutcome::EnrollmentCompleted => ("201 Created", "paired\n"),
        ConnectionOutcome::Rejected => ("400 Bad Request", "pairing request rejected\n"),
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len(),
    );
    tls.write_all(response.as_bytes())
        .and_then(|_| tls.flush())
        .map_err(|error| error.to_string())?;
    tls.conn.send_close_notify();
    tls.flush().map_err(|error| error.to_string())?;
    Ok(outcome)
}

fn process_request(
    request: &Request,
    store: &SqliteControlStore,
    challenge: &[u8; 32],
) -> Result<(), String> {
    if request.method != "POST" || request.path != "/v1/initial-owner-enrollment" {
        return Err("unsupported pairing endpoint".to_owned());
    }
    let token = decode_hex::<32>(
        required_header(request, "authorization")?
            .strip_prefix("Bearer ")
            .ok_or_else(|| "missing pairing bearer token".to_owned())?,
        "pairing token",
    )?;
    let identity = server_pairing_proof::verify(
        challenge,
        required_header(request, "x-makosh-owner-id")?,
        required_header(request, "x-makosh-device-id")?,
        required_header(request, "x-makosh-device-public-key-sec1")?,
        required_header(request, "x-makosh-device-signature-raw")?,
    )?;
    let token_sha256: [u8; 32] = Sha256::digest(token).into();
    store
        .claim_initial_owner_from_server_bootstrap_pairing(
            &identity,
            &token_sha256,
            unix_time_ms()?,
        )
        .map_err(|_| "initial owner enrollment rejected".to_owned())
}

fn required_header<'a>(request: &'a Request, name: &str) -> Result<&'a str, String> {
    request
        .headers
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("missing {name}"))
}

fn read_request(tls: &mut StreamOwned<ServerConnection, TcpStream>) -> Result<Request, String> {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 1024];
    while bytes.len() < MAX_REQUEST_HEADER_BYTES {
        let count = tls.read(&mut chunk).map_err(|error| error.to_string())?;
        if count == 0 {
            return Err("pairing connection closed before request".to_owned());
        }
        bytes.extend_from_slice(&chunk[..count]);
        if bytes.windows(4).any(|window| window == b"\r\n\r\n") {
            return parse_request(&bytes);
        }
    }
    Err("pairing request header is too large".to_owned())
}

fn parse_request(bytes: &[u8]) -> Result<Request, String> {
    let end = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| "pairing request is incomplete".to_owned())?;
    let text = std::str::from_utf8(&bytes[..end])
        .map_err(|_| "pairing request is not UTF-8".to_owned())?;
    let mut lines = text.split("\r\n");
    let request_line = lines
        .next()
        .ok_or_else(|| "pairing request is missing a request line".to_owned())?;
    let mut parts = request_line.split(' ');
    let method = parts
        .next()
        .ok_or_else(|| "pairing request method is missing".to_owned())?;
    let path = parts
        .next()
        .ok_or_else(|| "pairing request path is missing".to_owned())?;
    if parts.next() != Some("HTTP/1.1") || parts.next().is_some() {
        return Err("pairing request has an invalid HTTP version".to_owned());
    }
    let mut headers = BTreeMap::new();
    for line in lines {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| "pairing request header is invalid".to_owned())?;
        let name = name.to_ascii_lowercase();
        let value = value.trim();
        if name.is_empty() || value.is_empty() || headers.insert(name, value.to_owned()).is_some() {
            return Err("pairing request headers are invalid".to_owned());
        }
    }
    Ok(Request {
        method: method.to_owned(),
        path: path.to_owned(),
        headers,
    })
}

fn decode_hex<const N: usize>(value: &str, field: &str) -> Result<[u8; N], String> {
    if value.len() != N * 2 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{field} has an invalid encoding"));
    }
    let mut bytes = [0_u8; N];
    for (index, output) in bytes.iter_mut().enumerate() {
        *output = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| format!("{field} has an invalid encoding"))?;
    }
    Ok(bytes)
}

fn unix_time_ms() -> Result<u64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| {
            u64::try_from(duration.as_millis()).map_err(|_| "clock overflow".to_owned())
        })
        .map_err(|_| "system clock is before Unix epoch".to_owned())?
}

struct Request {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
}

enum ConnectionOutcome {
    EnrollmentCompleted,
    Rejected,
}
