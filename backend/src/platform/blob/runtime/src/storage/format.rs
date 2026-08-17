//! Versioned authenticated Blob-file encoding.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use getrandom::fill;
use makosh_blob_protocol::{BlobBackupClassV1, BlobCustodyScopeV1, BlobRefV1};

use super::store::BlobStorageError;

const LEGACY_MAGIC: &[u8; 8] = b"HBLBENC2";
const CHUNKED_MAGIC: &[u8; 8] = b"HBLBENC3";
const NONCE_BYTES: usize = 24;
const TAG_BYTES: usize = 16;
pub(super) const CHUNK_PLAINTEXT_BYTES: usize = 1024 * 1024;
pub(super) const CHUNKED_HEADER_BYTES: usize = CHUNKED_MAGIC.len() + std::mem::size_of::<u64>();

pub(super) fn is_supported_magic(bytes: &[u8; 8]) -> bool {
    bytes == LEGACY_MAGIC || bytes == CHUNKED_MAGIC
}

pub(super) fn is_legacy_magic(bytes: &[u8; 8]) -> bool {
    bytes == LEGACY_MAGIC
}

pub(super) fn is_chunked_magic(bytes: &[u8; 8]) -> bool {
    bytes == CHUNKED_MAGIC
}

pub(super) fn chunked_header(key_revision: u64) -> [u8; CHUNKED_HEADER_BYTES] {
    let mut header = [0_u8; CHUNKED_HEADER_BYTES];
    header[..CHUNKED_MAGIC.len()].copy_from_slice(CHUNKED_MAGIC);
    header[CHUNKED_MAGIC.len()..].copy_from_slice(&key_revision.to_be_bytes());
    header
}

pub(super) fn chunked_key_revision(header: &[u8]) -> Result<u64, BlobStorageError> {
    if header.len() != CHUNKED_HEADER_BYTES || header[..CHUNKED_MAGIC.len()] != *CHUNKED_MAGIC {
        return Err(BlobStorageError::MalformedCiphertext);
    }
    let bytes: [u8; 8] = header[CHUNKED_MAGIC.len()..]
        .try_into()
        .map_err(|_| BlobStorageError::MalformedCiphertext)?;
    Ok(u64::from_be_bytes(bytes))
}

pub(super) const fn encrypted_chunk_bytes(plaintext_bytes: usize) -> usize {
    NONCE_BYTES + plaintext_bytes + TAG_BYTES
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
    Ok([
        LEGACY_MAGIC.as_slice(),
        nonce.as_slice(),
        ciphertext.as_slice(),
    ]
    .concat())
}

pub(super) fn decrypt(
    reference: &BlobRefV1,
    custody: &BlobCustodyScopeV1,
    key_revision: u64,
    key: &[u8; 32],
    bytes: &[u8],
) -> Result<Vec<u8>, BlobStorageError> {
    if bytes.len() <= LEGACY_MAGIC.len() + NONCE_BYTES
        || bytes[..LEGACY_MAGIC.len()] != *LEGACY_MAGIC
    {
        return Err(BlobStorageError::MalformedCiphertext);
    }
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| BlobStorageError::Crypto)?;
    let nonce = XNonce::try_from(&bytes[LEGACY_MAGIC.len()..LEGACY_MAGIC.len() + NONCE_BYTES])
        .map_err(|_| BlobStorageError::MalformedCiphertext)?;
    cipher
        .decrypt(
            &nonce,
            Payload {
                msg: &bytes[LEGACY_MAGIC.len() + NONCE_BYTES..],
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
    bytes.extend_from_slice(LEGACY_MAGIC);
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

pub(super) fn encrypt_chunk(
    reference: &BlobRefV1,
    custody: &BlobCustodyScopeV1,
    key_revision: u64,
    key: &[u8; 32],
    offset: u64,
    plaintext: &[u8],
) -> Result<Vec<u8>, BlobStorageError> {
    let mut nonce = [0; NONCE_BYTES];
    fill(&mut nonce).map_err(|_| BlobStorageError::Randomness)?;
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| BlobStorageError::Crypto)?;
    let nonce_ref = XNonce::try_from(nonce.as_slice()).map_err(|_| BlobStorageError::Crypto)?;
    let ciphertext = cipher
        .encrypt(
            &nonce_ref,
            Payload {
                msg: plaintext,
                aad: &chunk_associated_data(
                    reference,
                    custody,
                    key_revision,
                    offset,
                    plaintext.len(),
                )?,
            },
        )
        .map_err(|_| BlobStorageError::Crypto)?;
    Ok([nonce.as_slice(), ciphertext.as_slice()].concat())
}

pub(super) fn decrypt_chunk(
    reference: &BlobRefV1,
    custody: &BlobCustodyScopeV1,
    key_revision: u64,
    key: &[u8; 32],
    offset: u64,
    plaintext_len: usize,
    bytes: &[u8],
) -> Result<Vec<u8>, BlobStorageError> {
    if bytes.len() != encrypted_chunk_bytes(plaintext_len) {
        return Err(BlobStorageError::MalformedCiphertext);
    }
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| BlobStorageError::Crypto)?;
    let nonce = XNonce::try_from(&bytes[..NONCE_BYTES])
        .map_err(|_| BlobStorageError::MalformedCiphertext)?;
    cipher
        .decrypt(
            &nonce,
            Payload {
                msg: &bytes[NONCE_BYTES..],
                aad: &chunk_associated_data(
                    reference,
                    custody,
                    key_revision,
                    offset,
                    plaintext_len,
                )?,
            },
        )
        .map_err(|_| BlobStorageError::AuthenticationFailed)
}

fn chunk_associated_data(
    reference: &BlobRefV1,
    custody: &BlobCustodyScopeV1,
    key_revision: u64,
    offset: u64,
    plaintext_len: usize,
) -> Result<Vec<u8>, BlobStorageError> {
    let mut bytes = associated_data(reference, custody, key_revision);
    bytes[..LEGACY_MAGIC.len()].copy_from_slice(CHUNKED_MAGIC);
    bytes.extend_from_slice(&offset.to_be_bytes());
    bytes.extend_from_slice(
        &u64::try_from(plaintext_len)
            .map_err(|_| BlobStorageError::InvalidWrite)?
            .to_be_bytes(),
    );
    Ok(bytes)
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
