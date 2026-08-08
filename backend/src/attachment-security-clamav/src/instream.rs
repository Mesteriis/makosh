use std::io::{self, Read, Write};

use makosh_attachment_security_core::ScannerOutcomeV1;
use sha2::{Digest, Sha256};

use crate::ClamAvInstreamLimitsV1;

const INSTREAM_COMMAND: &[u8] = b"zINSTREAM\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClamAvScanErrorV1 {
    InvalidEndpoint,
    InvalidLimits,
    InvalidInput,
    SizeLimitExceeded,
    IntegrityMismatch,
    ConnectTimeout,
    ConnectFailed,
    IoTimeout,
    IoFailed,
    ResponseTooLarge,
    MalformedResponse,
    ScannerRejected,
}

impl ClamAvScanErrorV1 {
    pub(crate) fn from_connect_io(error: io::Error) -> Self {
        if matches!(
            error.kind(),
            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
        ) {
            Self::ConnectTimeout
        } else {
            Self::ConnectFailed
        }
    }

    pub(crate) fn from_io(error: io::Error) -> Self {
        if matches!(
            error.kind(),
            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
        ) {
            Self::IoTimeout
        } else {
            Self::IoFailed
        }
    }
}

pub fn scan_clamav_instream_v1(
    stream: &mut (impl Read + Write),
    bytes: &[u8],
    declared_size: u64,
    receipt_sha256: [u8; 32],
    limits: ClamAvInstreamLimitsV1,
) -> Result<ScannerOutcomeV1, ClamAvScanErrorV1> {
    let actual_size =
        u64::try_from(bytes.len()).map_err(|_| ClamAvScanErrorV1::SizeLimitExceeded)?;
    if actual_size != declared_size {
        return Err(ClamAvScanErrorV1::InvalidInput);
    }
    if actual_size > limits.max_scan_bytes() {
        return Err(ClamAvScanErrorV1::SizeLimitExceeded);
    }
    let actual_sha256: [u8; 32] = Sha256::digest(bytes).into();
    if actual_sha256 != receipt_sha256 {
        return Err(ClamAvScanErrorV1::IntegrityMismatch);
    }

    stream
        .write_all(INSTREAM_COMMAND)
        .map_err(ClamAvScanErrorV1::from_io)?;
    for chunk in bytes.chunks(limits.chunk_bytes() as usize) {
        let chunk_length =
            u32::try_from(chunk.len()).map_err(|_| ClamAvScanErrorV1::SizeLimitExceeded)?;
        stream
            .write_all(&chunk_length.to_be_bytes())
            .map_err(ClamAvScanErrorV1::from_io)?;
        stream
            .write_all(chunk)
            .map_err(ClamAvScanErrorV1::from_io)?;
    }
    stream
        .write_all(&0_u32.to_be_bytes())
        .map_err(ClamAvScanErrorV1::from_io)?;
    stream.flush().map_err(ClamAvScanErrorV1::from_io)?;

    let response = read_bounded_response(stream, limits.max_response_bytes())?;
    parse_response(&response)
}

fn read_bounded_response(
    stream: &mut impl Read,
    max_response_bytes: u32,
) -> Result<Vec<u8>, ClamAvScanErrorV1> {
    let capacity =
        usize::try_from(max_response_bytes).map_err(|_| ClamAvScanErrorV1::ResponseTooLarge)?;
    let mut response = Vec::with_capacity(capacity.min(256));
    loop {
        let mut byte = [0_u8; 1];
        match stream.read(&mut byte) {
            Ok(0) => return Err(ClamAvScanErrorV1::MalformedResponse),
            Ok(1) if byte[0] == 0 => return Ok(response),
            Ok(1) => {
                if response.len() >= capacity {
                    return Err(ClamAvScanErrorV1::ResponseTooLarge);
                }
                response.push(byte[0]);
            }
            Ok(_) => return Err(ClamAvScanErrorV1::MalformedResponse),
            Err(error) => return Err(ClamAvScanErrorV1::from_io(error)),
        }
    }
}

fn parse_response(response: &[u8]) -> Result<ScannerOutcomeV1, ClamAvScanErrorV1> {
    if response == b"stream: OK" {
        return Ok(ScannerOutcomeV1::Clean);
    }
    let Some(value) = response
        .strip_prefix(b"stream: ")
        .and_then(|value| value.strip_suffix(b" FOUND"))
    else {
        return if response
            .strip_prefix(b"stream: ")
            .is_some_and(|value| value.ends_with(b" ERROR"))
        {
            Err(ClamAvScanErrorV1::ScannerRejected)
        } else {
            Err(ClamAvScanErrorV1::MalformedResponse)
        };
    };
    if value.is_empty() || !value.iter().all(u8::is_ascii_graphic) {
        return Err(ClamAvScanErrorV1::MalformedResponse);
    }
    Ok(ScannerOutcomeV1::ThreatFound)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn exact_clean_response_requires_bounded_instream_framing_and_integrity() {
        let bytes = b"hello";
        let mut stream = ScriptedStream::new(b"stream: OK\0");
        let outcome = scan_clamav_instream_v1(
            &mut stream,
            bytes,
            bytes.len() as u64,
            Sha256::digest(bytes).into(),
            limits(),
        )
        .expect("clean");

        assert_eq!(outcome, ScannerOutcomeV1::Clean);
        let mut expected = INSTREAM_COMMAND.to_vec();
        expected.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        expected.extend_from_slice(bytes);
        expected.extend_from_slice(&0_u32.to_be_bytes());
        assert_eq!(stream.written, expected);
    }

    #[test]
    fn threat_response_returns_only_the_closed_outcome() {
        let bytes = b"eicar";
        let mut stream = ScriptedStream::new(b"stream: Eicar-Test-Signature FOUND\0");
        assert_eq!(
            scan_clamav_instream_v1(
                &mut stream,
                bytes,
                bytes.len() as u64,
                Sha256::digest(bytes).into(),
                limits(),
            ),
            Ok(ScannerOutcomeV1::ThreatFound)
        );
    }

    #[test]
    fn error_and_malformed_responses_never_become_clean() {
        let bytes = b"payload";
        for (response, expected) in [
            (
                b"stream: scan failed ERROR\0".as_slice(),
                ClamAvScanErrorV1::ScannerRejected,
            ),
            (
                b"stream: ALMOST_OK\0".as_slice(),
                ClamAvScanErrorV1::MalformedResponse,
            ),
            (
                b"stream: OK\n\0".as_slice(),
                ClamAvScanErrorV1::MalformedResponse,
            ),
        ] {
            let mut stream = ScriptedStream::new(response);
            assert_eq!(
                scan_clamav_instream_v1(
                    &mut stream,
                    bytes,
                    bytes.len() as u64,
                    Sha256::digest(bytes).into(),
                    limits(),
                )
                .expect_err("not clean"),
                expected
            );
        }
    }

    #[test]
    fn mismatched_size_or_receipt_is_rejected_before_scanner_io() {
        let bytes = b"payload";
        let mut wrong_size = ScriptedStream::new(b"stream: OK\0");
        assert_eq!(
            scan_clamav_instream_v1(
                &mut wrong_size,
                bytes,
                bytes.len() as u64 + 1,
                Sha256::digest(bytes).into(),
                limits(),
            ),
            Err(ClamAvScanErrorV1::InvalidInput)
        );
        assert!(wrong_size.written.is_empty());

        let mut wrong_receipt = ScriptedStream::new(b"stream: OK\0");
        assert_eq!(
            scan_clamav_instream_v1(
                &mut wrong_receipt,
                bytes,
                bytes.len() as u64,
                [9; 32],
                limits(),
            ),
            Err(ClamAvScanErrorV1::IntegrityMismatch)
        );
        assert!(wrong_receipt.written.is_empty());
    }

    fn limits() -> ClamAvInstreamLimitsV1 {
        ClamAvInstreamLimitsV1::new(1024, 1024, 256).expect("limits")
    }

    struct ScriptedStream {
        response: Cursor<Vec<u8>>,
        written: Vec<u8>,
    }

    impl ScriptedStream {
        fn new(response: &[u8]) -> Self {
            Self {
                response: Cursor::new(response.to_vec()),
                written: Vec::new(),
            }
        }
    }

    impl Read for ScriptedStream {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.response.read(buffer)
        }
    }

    impl Write for ScriptedStream {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.written.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
}
