use std::time::Duration;

use async_native_tls::TlsConnector;
use async_std::{
    future,
    io::{ReadExt, WriteExt},
    net::TcpStream,
};
use serde_json::Value;

use crate::{ZulipHttpConfigV1, command::ZulipHttpRequestV1};

const MAX_RESPONSE_BYTES: u64 = 1_048_576;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipHttpErrorV1 {
    InvalidConfiguration,
    InvalidCommand,
    Unavailable,
    Protocol,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipHttpResponseV1 {
    pub status: u16,
    pub provider_message_id: Option<i64>,
}

pub async fn execute(
    config: &ZulipHttpConfigV1,
    request: ZulipHttpRequestV1,
) -> Result<ZulipHttpResponseV1, ZulipHttpErrorV1> {
    let (status, value) = execute_value(config, request).await?;
    Ok(ZulipHttpResponseV1 {
        status,
        provider_message_id: value.get("id").and_then(Value::as_i64),
    })
}

pub(crate) async fn execute_value(
    config: &ZulipHttpConfigV1,
    request: ZulipHttpRequestV1,
) -> Result<(u16, Value), ZulipHttpErrorV1> {
    let bytes = future::timeout(REQUEST_TIMEOUT, execute_once(config, request))
        .await
        .map_err(|_| ZulipHttpErrorV1::Unavailable)??;
    decode_response(&bytes)
}

pub(crate) async fn execute_binary(
    config: &ZulipHttpConfigV1,
    request: ZulipHttpRequestV1,
) -> Result<(Vec<u8>, Option<String>), ZulipHttpErrorV1> {
    let bytes = future::timeout(REQUEST_TIMEOUT, execute_once(config, request))
        .await
        .map_err(|_| ZulipHttpErrorV1::Unavailable)??;
    let split = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or(ZulipHttpErrorV1::Protocol)?;
    let header = std::str::from_utf8(&bytes[..split]).map_err(|_| ZulipHttpErrorV1::Protocol)?;
    let status = header
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or(ZulipHttpErrorV1::Protocol)?;
    (200..300)
        .contains(&status)
        .then_some(())
        .ok_or(ZulipHttpErrorV1::Rejected)?;
    let content_type = header
        .lines()
        .find_map(|line| {
            line.split_once(':')
                .filter(|(name, _)| name.eq_ignore_ascii_case("content-type"))
                .map(|(_, value)| value.trim().to_owned())
        })
        .filter(|value| !value.is_empty());
    let body = &bytes[split + 4..];
    let body = if header
        .lines()
        .any(|line| line.eq_ignore_ascii_case("transfer-encoding: chunked"))
    {
        decode_chunked(body)?
    } else {
        body.to_vec()
    };
    Ok((body, content_type))
}

async fn execute_once(
    config: &ZulipHttpConfigV1,
    request: ZulipHttpRequestV1,
) -> Result<Vec<u8>, ZulipHttpErrorV1> {
    let endpoint = endpoint(&config.account.realm_url)?;
    let tcp = TcpStream::connect((endpoint.host.as_str(), endpoint.port))
        .await
        .map_err(|_| unavailable("tcp_connect"))?;
    let mut stream = tls_connector(&endpoint.host)?
        .connect(&endpoint.host, tcp)
        .await
        .map_err(|_| unavailable("tls_connect"))?;
    let authorization = basic_authorization(&config.account.account_email, &config.api_key);
    let mut request_bytes = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nAuthorization: Basic {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        request.method,
        request.path,
        endpoint.authority,
        authorization,
        request.content_type,
        request.body.len(),
    ).into_bytes();
    request_bytes.extend_from_slice(&request.body);
    stream
        .write_all(&request_bytes)
        .await
        .map_err(|_| unavailable("write_request"))?;
    stream
        .flush()
        .await
        .map_err(|_| unavailable("flush_request"))?;
    let mut bytes = Vec::new();
    stream
        .take(MAX_RESPONSE_BYTES)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| unavailable("read_response"))?;
    (bytes.len() < usize::try_from(MAX_RESPONSE_BYTES).unwrap_or(usize::MAX))
        .then_some(())
        .ok_or(ZulipHttpErrorV1::Protocol)?;
    Ok(bytes)
}

fn unavailable(stage: &str) -> ZulipHttpErrorV1 {
    #[cfg(feature = "conformance-test-support")]
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        let diagnostic = format!("developer_zulip_http_unavailable stage={stage}\n");
        let mut stderr = std::io::stderr().lock();
        let _ = std::io::Write::write_all(&mut stderr, diagnostic.as_bytes());
    }
    #[cfg(not(feature = "conformance-test-support"))]
    let _ = stage;
    ZulipHttpErrorV1::Unavailable
}

#[cfg(not(feature = "conformance-test-support"))]
fn tls_connector(_host: &str) -> Result<TlsConnector, ZulipHttpErrorV1> {
    Ok(TlsConnector::new())
}

#[cfg(feature = "conformance-test-support")]
fn tls_connector(host: &str) -> Result<TlsConnector, ZulipHttpErrorV1> {
    use async_native_tls::Certificate;

    if host != "localhost" && host != "127.0.0.1" && host != "::1" {
        return Ok(TlsConnector::new());
    }
    let certificate_path = std::env::var_os("MAKOSH_MANAGED_RUNTIME_CONFORMANCE_CA_CERT_FILE")
        .ok_or(ZulipHttpErrorV1::InvalidConfiguration)?;
    let certificate_bytes =
        std::fs::read(certificate_path).map_err(|_| ZulipHttpErrorV1::InvalidConfiguration)?;
    let certificate = Certificate::from_pem(&certificate_bytes)
        .map_err(|_| ZulipHttpErrorV1::InvalidConfiguration)?;
    Ok(TlsConnector::new().add_root_certificate(certificate))
}

struct Endpoint {
    authority: String,
    host: String,
    port: u16,
}

fn endpoint(realm_url: &str) -> Result<Endpoint, ZulipHttpErrorV1> {
    let authority = realm_url
        .strip_prefix("https://")
        .and_then(|value| value.split('/').next())
        .filter(|value| !value.is_empty())
        .ok_or(ZulipHttpErrorV1::InvalidConfiguration)?;
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) if !host.is_empty() && !port.is_empty() => (
            host.to_owned(),
            port.parse()
                .map_err(|_| ZulipHttpErrorV1::InvalidConfiguration)?,
        ),
        _ => (authority.to_owned(), 443),
    };
    (!host.contains(['[', ']', '@', ':']))
        .then_some(())
        .ok_or(ZulipHttpErrorV1::InvalidConfiguration)?;
    Ok(Endpoint {
        authority: authority.to_owned(),
        host,
        port,
    })
}

fn decode_response(bytes: &[u8]) -> Result<(u16, Value), ZulipHttpErrorV1> {
    let split = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or(ZulipHttpErrorV1::Protocol)?;
    let header = std::str::from_utf8(&bytes[..split]).map_err(|_| ZulipHttpErrorV1::Protocol)?;
    let status = header
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse().ok())
        .ok_or(ZulipHttpErrorV1::Protocol)?;
    (200..300)
        .contains(&status)
        .then_some(())
        .ok_or(ZulipHttpErrorV1::Rejected)?;
    let body = &bytes[split + 4..];
    let body = if header
        .lines()
        .any(|line| line.eq_ignore_ascii_case("transfer-encoding: chunked"))
    {
        decode_chunked(body)?
    } else {
        body.to_vec()
    };
    let value: Value = serde_json::from_slice(&body).map_err(|_| ZulipHttpErrorV1::Protocol)?;
    (value.get("result").and_then(Value::as_str) == Some("success"))
        .then_some(())
        .ok_or(ZulipHttpErrorV1::Rejected)?;
    Ok((status, value))
}

fn decode_chunked(mut bytes: &[u8]) -> Result<Vec<u8>, ZulipHttpErrorV1> {
    let mut decoded = Vec::new();
    loop {
        let line_end = bytes
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or(ZulipHttpErrorV1::Protocol)?;
        let size = std::str::from_utf8(&bytes[..line_end])
            .ok()
            .and_then(|line| line.split(';').next())
            .and_then(|value| usize::from_str_radix(value, 16).ok())
            .ok_or(ZulipHttpErrorV1::Protocol)?;
        bytes = &bytes[line_end + 2..];
        if size == 0 {
            return Ok(decoded);
        }
        (bytes.len() >= size + 2 && &bytes[size..size + 2] == b"\r\n")
            .then_some(())
            .ok_or(ZulipHttpErrorV1::Protocol)?;
        decoded.extend_from_slice(&bytes[..size]);
        bytes = &bytes[size + 2..];
    }
}

fn basic_authorization(username: &str, password: &str) -> String {
    base64_encode(format!("{username}:{password}").as_bytes())
}

fn base64_encode(value: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(value.len().div_ceil(3) * 4);
    for chunk in value.chunks(3) {
        let packed = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        encoded.push(char::from(ALPHABET[((packed >> 18) & 0x3f) as usize]));
        encoded.push(char::from(ALPHABET[((packed >> 12) & 0x3f) as usize]));
        encoded.push(if chunk.len() > 1 {
            char::from(ALPHABET[((packed >> 6) & 0x3f) as usize])
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            char::from(ALPHABET[(packed & 0x3f) as usize])
        } else {
            '='
        });
    }
    encoded
}
