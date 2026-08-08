//! Versioned authenticated Blob-file encoding.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use getrandom::fill;
use makosh_blob_protocol::{BlobBackupClassV1, BlobCustodyScopeV1, BlobRefV1};

use super::store::BlobStorageError;

const MAGIC: &[u8; 8] = b"HBLBENC2";
const NONCE_BYTES: usize = 24;

pub(super) fn is_current_magic(bytes: &[u8; 8]) -> bool {
    bytes == MAGIC
}

pub(super) fn encrypt(
    reference: &BlobRefV1,
    custody: &BlobCustodyScopeV1,
    key_revision: u64,
    key: &[u8; 32],
    plaintext: &[u8],
) -> Result<Vec<u8>, BlobStorageError> {
    let mut nonce = [0; NONCE_BYTES];
    fill(&mut nonce).map_err(|_| BlobStorageError::Randomness)?;
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| BlobStorageError::Crypto)?;
    let nonce = XNonce::try_from(nonce.as_slice()).map_err(|_| BlobStorageError::Crypto)?;
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad: &associated_data(reference, custody, key_revision),
            },
        )
        .map_err(|_| BlobStorageError::Crypto)?;
    Ok([MAGIC.as_slice(), nonce.as_slice(), ciphertext.as_slice()].concat())
}

pub(super) fn decrypt(
    reference: &BlobRefV1,
    custody: &BlobCustodyScopeV1,
    key_revision: u64,
    key: &[u8; 32],
    bytes: &[u8],
) -> Result<Vec<u8>, BlobStorageError> {
    if bytes.len() <= MAGIC.len() + NONCE_BYTES || bytes[..MAGIC.len()] != *MAGIC {
        return Err(BlobStorageError::MalformedCiphertext);
    }
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| BlobStorageError::Crypto)?;
    let nonce = XNonce::try_from(&bytes[MAGIC.len()..MAGIC.len() + NONCE_BYTES])
        .map_err(|_| BlobStorageError::MalformedCiphertext)?;
    cipher
        .decrypt(
            &nonce,
            Payload {
                msg: &bytes[MAGIC.len() + NONCE_BYTES..],
                aad: &associated_data(reference, custody, key_revision),
            },
        )
        .map_err(|_| BlobStorageError::AuthenticationFailed)
}

fn associated_data(
    reference: &BlobRefV1,
    custody: &BlobCustodyScopeV1,
    key_revision: u64,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(512);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(reference.reference_id());
    append_field(&mut bytes, reference.owner_id().as_bytes());
    bytes.extend_from_slice(&reference.declared_size().to_be_bytes());
    bytes.extend_from_slice(
        &reference
            .expires_at_unix_ms()
            .unwrap_or_default()
            .to_be_bytes(),
    );
    bytes.push(backup_class_code(reference.backup_class()));
    append_field(&mut bytes, custody.owner_id().as_bytes());
    append_field(&mut bytes, custody.custody_scope_id().as_bytes());
    bytes.extend_from_slice(&key_revision.to_be_bytes());
    bytes
}

fn append_field(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(
        &(u16::try_from(value.len()).expect("validated identifier fits u16")).to_be_bytes(),
    );
    target.extend_from_slice(value);
}

const fn backup_class_code(value: BlobBackupClassV1) -> u8 {
    match value {
        BlobBackupClassV1::Required => 1,
        BlobBackupClassV1::Rebuildable => 2,
        BlobBackupClassV1::Excluded => 3,
    }
}
