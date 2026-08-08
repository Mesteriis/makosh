use std::time::Duration;

use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

const CLAMAV_ADDRESS_ENV: &str = "MAKOSH_CLAMAV_ADDR";
const CLAMAV_TIMEOUT_SECONDS_ENV: &str = "MAKOSH_CLAMAV_TIMEOUT_SECONDS";
const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
const STREAM_CHUNK_BYTES: usize = 16 * 1024;
const MAX_RESPONSE_BYTES: usize = 4096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClamAvClient {
    address: String,
    timeout: Duration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClamAvVerdict {
    Clean,
    Malicious { signature: String },
}

#[derive(Debug, Error)]
pub enum ClamAvError {
    #[error("invalid {name}: {value}")]
    InvalidConfiguration { name: &'static str, value: String },
    #[error("ClamAV scan timed out after {0} seconds")]
    Timeout(u64),
    #[error("ClamAV I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("ClamAV returned an invalid response: {0}")]
    InvalidResponse(String),
    #[error("ClamAV rejected the scan: {0}")]
    Scanner(String),
}

impl ClamAvClient {
    pub fn from_env() -> Result<Option<Self>, ClamAvError> {
        let Ok(address) = std::env::var(CLAMAV_ADDRESS_ENV) else {
            return Ok(None);
        };
        let address = address.trim();
        if address.is_empty() {
            return Err(ClamAvError::InvalidConfiguration {
                name: CLAMAV_ADDRESS_ENV,
                value: address.to_owned(),
            });
        }
        let timeout_seconds = match std::env::var(CLAMAV_TIMEOUT_SECONDS_ENV) {
            Ok(value) => value
                .trim()
                .parse::<u64>()
                .ok()
                .filter(|value| *value > 0)
                .ok_or(ClamAvError::InvalidConfiguration {
                    name: CLAMAV_TIMEOUT_SECONDS_ENV,
                    value,
                })?,
            Err(_) => DEFAULT_TIMEOUT_SECONDS,
        };
        Ok(Some(Self::new(
            address,
            Duration::from_secs(timeout_seconds),
        )?))
    }

    pub fn new(address: impl Into<String>, timeout: Duration) -> Result<Self, ClamAvError> {
        let address = address.into();
        if address.trim().is_empty() || timeout.is_zero() {
            return Err(ClamAvError::InvalidConfiguration {
                name: CLAMAV_ADDRESS_ENV,
                value: address,
            });
        }
        Ok(Self { address, timeout })
    }

    pub async fn scan(&self, bytes: &[u8]) -> Result<ClamAvVerdict, ClamAvError> {
        match timeout(self.timeout, self.scan_inner(bytes)).await {
            Ok(result) => result,
            Err(_) => Err(ClamAvError::Timeout(self.timeout.as_secs())),
        }
    }

    async fn scan_inner(&self, bytes: &[u8]) -> Result<ClamAvVerdict, ClamAvError> {
        let mut stream = TcpStream::connect(&self.address).await?;
        stream.write_all(b"zINSTREAM\0").await?;
        for chunk in bytes.chunks(STREAM_CHUNK_BYTES) {
            let length = u32::try_from(chunk.len()).map_err(|_| {
                ClamAvError::InvalidResponse("attachment chunk is too large".to_owned())
            })?;
            stream.write_all(&length.to_be_bytes()).await?;
            stream.write_all(chunk).await?;
        }
        stream.write_all(&0_u32.to_be_bytes()).await?;
        stream.flush().await?;

        let mut response = Vec::new();
        loop {
            let byte = stream.read_u8().await?;
            if byte == 0 {
                break;
            }
            if response.len() >= MAX_RESPONSE_BYTES {
                return Err(ClamAvError::InvalidResponse(
                    "response exceeds 4096 bytes".to_owned(),
                ));
            }
            response.push(byte);
        }
        parse_clamav_response(&response)
    }
}

fn parse_clamav_response(response: &[u8]) -> Result<ClamAvVerdict, ClamAvError> {
    let response = std::str::from_utf8(response)
        .map_err(|_| ClamAvError::InvalidResponse("response is not UTF-8".to_owned()))?;
    let result = response
        .strip_prefix("stream: ")
        .ok_or_else(|| ClamAvError::InvalidResponse(response.to_owned()))?;
    if result == "OK" {
        return Ok(ClamAvVerdict::Clean);
    }
    if let Some(signature) = result.strip_suffix(" FOUND") {
        let signature = signature.trim();
        if !signature.is_empty() {
            return Ok(ClamAvVerdict::Malicious {
                signature: signature.to_owned(),
            });
        }
    }
    if let Some(message) = result.strip_suffix(" ERROR") {
        return Err(ClamAvError::Scanner(message.trim().to_owned()));
    }
    Err(ClamAvError::InvalidResponse(response.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    async fn fake_clamd(response: &'static [u8]) -> (String, tokio::task::JoinHandle<Vec<u8>>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind fake clamd");
        let address = listener
            .local_addr()
            .expect("fake clamd address")
            .to_string();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept scan");
            let mut command = [0_u8; 10];
            socket.read_exact(&mut command).await.expect("read command");
            assert_eq!(&command, b"zINSTREAM\0");
            let mut body = Vec::new();
            loop {
                let length = socket.read_u32().await.expect("read chunk length") as usize;
                if length == 0 {
                    break;
                }
                let offset = body.len();
                body.resize(offset + length, 0);
                socket
                    .read_exact(&mut body[offset..])
                    .await
                    .expect("read chunk");
            }
            socket.write_all(response).await.expect("write verdict");
            body
        });
        (address, server)
    }

    #[tokio::test]
    async fn streams_bytes_and_accepts_clean_verdict() {
        let (address, server) = fake_clamd(b"stream: OK\0").await;
        let client = ClamAvClient::new(address, Duration::from_secs(1)).expect("client");

        assert_eq!(
            client.scan(b"safe attachment").await.expect("scan"),
            ClamAvVerdict::Clean
        );
        assert_eq!(server.await.expect("server task"), b"safe attachment");
    }

    #[tokio::test]
    async fn preserves_malware_signature() {
        let (address, server) = fake_clamd(b"stream: Win.Test.EICAR_HDB-1 FOUND\0").await;
        let client = ClamAvClient::new(address, Duration::from_secs(1)).expect("client");

        assert_eq!(
            client.scan(b"unsafe attachment").await.expect("scan"),
            ClamAvVerdict::Malicious {
                signature: "Win.Test.EICAR_HDB-1".to_owned()
            }
        );
        server.await.expect("server task");
    }

    #[test]
    fn rejects_scanner_errors_instead_of_marking_clean() {
        let error = parse_clamav_response(b"stream: INSTREAM size limit exceeded. ERROR")
            .expect_err("scanner error");
        assert!(matches!(error, ClamAvError::Scanner(_)));
    }

    #[tokio::test]
    #[ignore = "requires an explicitly configured live ClamAV service"]
    async fn live_clamav_accepts_synthetic_safe_payload() {
        let client = ClamAvClient::from_env()
            .expect("valid ClamAV config")
            .expect("MAKOSH_CLAMAV_ADDR must be configured");
        assert_eq!(
            client
                .scan(b"makosh ClamAV live smoke payload")
                .await
                .expect("live scan"),
            ClamAvVerdict::Clean
        );
    }
}
