use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    time::Duration,
};

use makosh_attachment_security_contract::ATTACHMENT_SECURITY_MAX_SCAN_CANDIDATE_BYTES_V1;
use makosh_attachment_security_core::ScannerOutcomeV1;

use crate::{ClamAvScanErrorV1, scan_clamav_instream_v1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClamAvLoopbackEndpointV1 {
    port: u16,
}

impl ClamAvLoopbackEndpointV1 {
    pub fn new(port: u16) -> Result<Self, ClamAvScanErrorV1> {
        if port == 0 {
            return Err(ClamAvScanErrorV1::InvalidEndpoint);
        }
        Ok(Self { port })
    }

    #[must_use]
    pub const fn port(self) -> u16 {
        self.port
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClamAvTimeoutsV1 {
    connect: Duration,
    io: Duration,
}

impl ClamAvTimeoutsV1 {
    pub fn new(connect: Duration, io: Duration) -> Result<Self, ClamAvScanErrorV1> {
        if connect.is_zero()
            || io.is_zero()
            || connect > Duration::from_secs(30)
            || io > Duration::from_secs(120)
        {
            return Err(ClamAvScanErrorV1::InvalidLimits);
        }
        Ok(Self { connect, io })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClamAvInstreamLimitsV1 {
    max_scan_bytes: u64,
    chunk_bytes: u32,
    max_response_bytes: u32,
}

impl ClamAvInstreamLimitsV1 {
    pub fn new(
        max_scan_bytes: u64,
        chunk_bytes: u32,
        max_response_bytes: u32,
    ) -> Result<Self, ClamAvScanErrorV1> {
        if max_scan_bytes == 0
            || max_scan_bytes > ATTACHMENT_SECURITY_MAX_SCAN_CANDIDATE_BYTES_V1
            || !(1..=1024 * 1024).contains(&chunk_bytes)
            || !(16..=4096).contains(&max_response_bytes)
        {
            return Err(ClamAvScanErrorV1::InvalidLimits);
        }
        Ok(Self {
            max_scan_bytes,
            chunk_bytes,
            max_response_bytes,
        })
    }

    #[must_use]
    pub const fn max_scan_bytes(self) -> u64 {
        self.max_scan_bytes
    }

    #[must_use]
    pub const fn chunk_bytes(self) -> u32 {
        self.chunk_bytes
    }

    #[must_use]
    pub const fn max_response_bytes(self) -> u32 {
        self.max_response_bytes
    }
}

pub fn scan_clamav_loopback_v1(
    endpoint: ClamAvLoopbackEndpointV1,
    bytes: &[u8],
    declared_size: u64,
    receipt_sha256: [u8; 32],
    limits: ClamAvInstreamLimitsV1,
    timeouts: ClamAvTimeoutsV1,
) -> Result<ScannerOutcomeV1, ClamAvScanErrorV1> {
    let address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, endpoint.port);
    let mut stream = TcpStream::connect_timeout(&address.into(), timeouts.connect)
        .map_err(ClamAvScanErrorV1::from_connect_io)?;
    stream
        .set_read_timeout(Some(timeouts.io))
        .map_err(ClamAvScanErrorV1::from_io)?;
    stream
        .set_write_timeout(Some(timeouts.io))
        .map_err(ClamAvScanErrorV1::from_io)?;
    scan_clamav_instream_v1(&mut stream, bytes, declared_size, receipt_sha256, limits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_and_limits_are_local_and_bounded() {
        assert_eq!(
            ClamAvLoopbackEndpointV1::new(0),
            Err(ClamAvScanErrorV1::InvalidEndpoint)
        );
        assert_eq!(
            ClamAvInstreamLimitsV1::new(65 * 1024 * 1024, 1024, 256),
            Err(ClamAvScanErrorV1::InvalidLimits)
        );
        assert_eq!(
            ClamAvTimeoutsV1::new(Duration::ZERO, Duration::from_secs(1)),
            Err(ClamAvScanErrorV1::InvalidLimits)
        );
    }
}
