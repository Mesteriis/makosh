//! Bounded local clamd INSTREAM adapter.

mod endpoint;
mod instream;

pub use endpoint::{
    ClamAvInstreamLimitsV1, ClamAvLoopbackEndpointV1, ClamAvTimeoutsV1, scan_clamav_loopback_v1,
};
pub use instream::{ClamAvScanErrorV1, scan_clamav_instream_v1};

pub const PACKAGE: &str = "makosh-attachment-security-clamav";
