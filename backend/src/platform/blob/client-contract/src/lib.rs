//! Typed Blob data port. Implementations own transport; callers own grants.

use makosh_runtime_protocol::v1::BlobDataSessionGrantV1;

pub const PACKAGE: &str = "makosh-blob-client-contract";

pub trait BlobReadPort {
    fn read_range(
        &mut self,
        grant: BlobDataSessionGrantV1,
        channel_binding: Vec<u8>,
        start: u64,
        end_exclusive: u64,
    ) -> Result<Vec<u8>, BlobReadError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlobReadError {
    Unavailable,
    Rejected,
    InvalidResponse,
}
