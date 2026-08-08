//! Ephemeral content-key lease received from the Vault route.

use makosh_blob_protocol::{BlobAccessFenceV1, BlobCustodyScopeV1, BlobRefV1};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

pub struct BlobKeyLeaseV1 {
    fence: BlobAccessFenceV1,
    custody: BlobCustodyScopeV1,
    key_revision: u64,
    reference_id: [u8; 16],
    expires_at_unix_ms: u64,
    key: Zeroizing<[u8; 32]>,
}

impl BlobKeyLeaseV1 {
    pub fn from_vault_response(
        reference: &BlobRefV1,
        fence: BlobAccessFenceV1,
        custody: BlobCustodyScopeV1,
        key_revision: u64,
        expires_at_unix_ms: u64,
        now_unix_ms: u64,
        response: Zeroizing<Vec<u8>>,
    ) -> Result<Self, BlobLeaseError> {
        if response.is_empty() || response.len() > MAX_VAULT_RESPONSE_BYTES || key_revision == 0 {
            return Err(BlobLeaseError::InvalidVaultResponse);
        }
        if expires_at_unix_ms <= now_unix_ms {
            return Err(BlobLeaseError::Expired);
        }
        Ok(Self {
            fence,
            custody,
            key_revision,
            reference_id: *reference.reference_id(),
            expires_at_unix_ms,
            key: Zeroizing::new(derive_key(reference, &response)),
        })
    }

    pub(crate) fn key_for(
        &self,
        reference: &BlobRefV1,
        expected_fence: &BlobAccessFenceV1,
        expected_custody: &BlobCustodyScopeV1,
        now_unix_ms: u64,
    ) -> Result<&[u8; 32], BlobLeaseError> {
        if self.expires_at_unix_ms <= now_unix_ms {
            return Err(BlobLeaseError::Expired);
        }
        if &self.fence != expected_fence {
            return Err(BlobLeaseError::FenceMismatch);
        }
        if &self.custody != expected_custody {
            return Err(BlobLeaseError::CustodyMismatch);
        }
        if self.reference_id != *reference.reference_id() {
            return Err(BlobLeaseError::ReferenceMismatch);
        }
        Ok(&self.key)
    }

    #[must_use]
    pub(crate) const fn key_revision(&self) -> u64 {
        self.key_revision
    }
}

const MAX_VAULT_RESPONSE_BYTES: usize = 16 * 1024;

fn derive_key(reference: &BlobRefV1, response: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.blob.content-key.v1\0");
    hash.update(reference.reference_id());
    hash.update(response);
    hash.finalize().into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobLeaseError {
    InvalidVaultResponse,
    Expired,
    FenceMismatch,
    CustodyMismatch,
    ReferenceMismatch,
}
