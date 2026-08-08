use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use sha2::{Digest, Sha256};

use crate::pairing::state as pairing_state;

const MAX_FAILED_REQUESTS: u8 = 8;
const MAX_REQUEST_HEADER_BYTES: usize = 16 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const PROOF_DOMAIN: &[u8] = b"makosh.development.remote-pairing.v1\0";

pub struct ListenerConfig<'a> {
    pub state_dir: &'a Path,
    pub listen_address: SocketAddr,
    pub idle_timeout: Duration,
}

pub fn listen(config: ListenerConfig<'_>) -> Result<(), String> {
    let tls_config = tls_config()?;
    let certificate_fingerprint = certificate_fingerprint(&tls_config.1);
    let listener = TcpListener::bind(config.listen_address).map_err(|error| error.to_string())?;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let address = listener.local_addr().map_err(|error| error.to_string())?;
    let mut challenge = [0_u8; 32];
    getrandom::fill(&mut challenge).map_err(|error| error.to_string())?;

    println!("development_pairing_endpoint=https://{address}");
    println!("development_pairing_tls_certificate_sha256={certificate_fingerprint}");
    let deadline = Instant::now()
        .checked_add(config.idle_timeout)
        .ok_or_else(|| "pairing listener timeout overflow".to_owned())?;
    let mut failures = 0_u8;
    while Instant::now() < deadline {
        match listener.accept() {
            Ok((stream, _)) => match handle_connection(
                stream,
                Arc::clone(&tls_config.0),
                config.state_dir,
                &challenge,
            ) {
                Ok(ConnectionOutcome::EnrollmentCompleted) => {
                    println!("development_remote_pairing_consumed=true");
                    return Ok(());
                }
                Ok(ConnectionOutcome::ChallengeIssued) => {}
                Ok(ConnectionOutcome::Rejected) => {
                    failures = failures.saturating_add(1);
                    if failures >= MAX_FAILED_REQUESTS {
                        return Err("development pairing listener rate limit exceeded".to_owned());
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Err(error) => {
                    let _ = error;
                    eprintln!("development pairing TLS connection failed");
                    failures = failures.saturating_add(1);
                    if failures >= MAX_FAILED_REQUESTS {
                        return Err("development pairing listener rate limit exceeded".to_owned());
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
    Err("development pairing listener timed out".to_owned())
}

pub fn proof_message(
    challenge: &[u8; 32],
    public_key: &[u8; 65],
    owner_id: &str,
    device_id: &str,
) -> Result<Vec<u8>, String> {
    validate_identity(owner_id, "owner_id")?;
    validate_identity(device_id, "device_id")?;
    let owner = owner_id.as_bytes();
    let device = device_id.as_bytes();
    let mut message = Vec::with_capacity(
        PROOF_DOMAIN.len() + challenge.len() + public_key.len() + owner.len() + device.len() + 4,
    );
    message.extend_from_slice(PROOF_DOMAIN);
    message.extend_from_slice(challenge);
    message.extend_from_slice(public_key);
    message.extend_from_slice(&(owner.len() as u16).to_be_bytes());
    message.extend_from_slice(owner);
    message.extend_from_slice(&(device.len() as u16).to_be_bytes());
    message.extend_from_slice(device);
    Ok(message)
}

pub fn decode_hex<const N: usize>(value: &str, field: &str) -> Result<[u8; N], String> {
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

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn tls_config() -> Result<(Arc<ServerConfig>, CertificateDer<'static>), String> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let certified_key = generate_simple_self_signed(vec!["localhost".to_owned()])
        .map_err(|error| error.to_string())?;
    let certificate = certified_key.cert.der().clone();
    let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
        certified_key.key_pair.serialize_der(),
    ));
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![certificate.clone()], private_key)
        .map_err(|error| error.to_string())?;
    Ok((Arc::new(config), certificate))
}

fn certificate_fingerprint(certificate: &CertificateDer<'_>) -> String {
    hex(&Sha256::digest(certificate.as_ref()))
}

fn handle_connection(
    stream: TcpStream,
    tls_config: Arc<ServerConfig>,
    state_dir: &Path,
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
    let (status, body, outcome) = match process_request(&request, state_dir, challenge) {
        Ok(true) => (
            "201 Created",
            "paired\n".to_owned(),
            ConnectionOutcome::EnrollmentCompleted,
        ),
        Ok(false) => (
            "200 OK",
            format!("challenge={}\n", hex(challenge)),
            ConnectionOutcome::ChallengeIssued,
        ),
        Err(_) => (
            "400 Bad Request",
            "pairing request rejected\n".to_owned(),
            ConnectionOutcome::Rejected,
        ),
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len(),
    );
    tls.write_all(response.as_bytes())
        .map_err(|error| error.to_string())?;
    tls.flush().map_err(|error| error.to_string())?;
    tls.conn.send_close_notify();
    tls.flush().map_err(|error| error.to_string())?;
    Ok(outcome)
}

fn process_request(
    request: &Request,
    state_dir: &Path,
    challenge: &[u8; 32],
) -> Result<bool, String> {
    let token = request
        .headers
        .get("authorization")
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| "missing pairing bearer token".to_owned())?;
    pairing_state::validate(state_dir, token)?;
    if request.method == "GET" && request.path == "/v1/pairing-challenge" {
        return Ok(false);
    }
    if request.method != "POST" || request.path != "/v1/initial-owner-enrollment" {
        return Err("unsupported pairing endpoint".to_owned());
    }
    let owner_id = required_header(request, "x-makosh-owner-id")?;
    let device_id = required_header(request, "x-makosh-device-id")?;
    let public_key = decode_hex::<65>(
        required_header(request, "x-makosh-device-public-key-sec1")?,
        "device public key",
    )?;
    let signature = Signature::from_slice(&decode_hex::<64>(
        required_header(request, "x-makosh-device-signature-raw")?,
        "device signature",
    )?)
    .map_err(|_| "device signature is invalid".to_owned())?;
    let verifying_key = VerifyingKey::from_sec1_bytes(&public_key)
        .map_err(|_| "device public key is invalid".to_owned())?;
    let message = proof_message(challenge, &public_key, owner_id, device_id)?;
    verifying_key
        .verify(&message, &signature)
        .map_err(|_| "device proof is invalid".to_owned())?;
    pairing_state::complete_with_receipt(
        state_dir,
        token,
        &pairing_state::RemotePairingReceipt {
            owner_id: owner_id.to_owned(),
            device_id: device_id.to_owned(),
            challenge: *challenge,
            device_public_key_sec1: public_key,
            signature_raw: signature.to_bytes().into(),
        },
    )?;
    Ok(true)
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

struct Request {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
}

enum ConnectionOutcome {
    ChallengeIssued,
    EnrollmentCompleted,
    Rejected,
}

fn validate_identity(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(format!("{field} must be an ASCII identifier"));
    }
    Ok(())
}
