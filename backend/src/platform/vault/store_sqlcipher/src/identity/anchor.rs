//! Authenticated platform and recovery root-key slots beside the encrypted database.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use getrandom::fill;
use hkdf::Hkdf;
use makosh_vault_key_provider::WrappingKey;
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::identity::recovery_key::VaultRecoveryKeyV1;

const MAGIC_V1: [u8; 4] = *b"HVA1";
const MAGIC_V2: [u8; 4] = *b"HVA2";
const NONCE_BYTES: usize = 24;
const ROOT_KEY_BYTES: usize = 32;
const WRAPPED_ROOT_BYTES: usize = ROOT_KEY_BYTES + 16;
const MAX_INSTANCE_ID_BYTES: usize = 128;
const PLATFORM_SLOT: u8 = 1;
const RECOVERY_SLOT: u8 = 2;

pub struct VaultRootKey(Zeroizing<[u8; ROOT_KEY_BYTES]>);

impl VaultRootKey {
    pub fn create() -> Result<Self, VaultAnchorError> {
        let mut root = Zeroizing::new([0; ROOT_KEY_BYTES]);
        fill(root.as_mut()).map_err(|_| VaultAnchorError::Randomness)?;
        Ok(Self(root))
    }

    pub fn derive_sqlcipher_key(
        &self,
        instance_id: &str,
    ) -> Result<Zeroizing<[u8; 32]>, VaultAnchorError> {
        derive_key(instance_id, &self.0, b"makosh-vault/sqlcipher-key/v1")
    }

    pub(crate) fn derive_legacy_sqlcipher_key(
        &self,
        instance_id: &str,
    ) -> Result<Zeroizing<[u8; 32]>, VaultAnchorError> {
        derive_key(instance_id, &self.0, b"hermes-vault/sqlcipher-key/v1")
    }

    pub fn derive_record_key(
        &self,
        instance_id: &str,
        key_epoch: u32,
    ) -> Result<Zeroizing<[u8; 32]>, VaultAnchorError> {
        let mut info = *b"makosh-vault/record-key/v1/0000";
        let offset = info.len() - key_epoch.to_be_bytes().len();
        info[offset..].copy_from_slice(&key_epoch.to_be_bytes());
        derive_key(instance_id, &self.0, &info)
    }

    pub(crate) fn derive_legacy_record_key(
        &self,
        instance_id: &str,
        key_epoch: u32,
    ) -> Result<Zeroizing<[u8; 32]>, VaultAnchorError> {
        let mut info = *b"hermes-vault/record-key/v1/0000";
        let offset = info.len() - key_epoch.to_be_bytes().len();
        info[offset..].copy_from_slice(&key_epoch.to_be_bytes());
        derive_key(instance_id, &self.0, &info)
    }

    pub(crate) fn derive_backup_manifest_key(
        &self,
        instance_id: &str,
    ) -> Result<Zeroizing<[u8; 32]>, VaultAnchorError> {
        derive_key(instance_id, &self.0, b"makosh-vault/backup-manifest/v1")
    }
}

pub fn create_anchor(
    path: &Path,
    instance_id: &str,
    wrapping_key: &WrappingKey,
) -> Result<VaultRootKey, VaultAnchorError> {
    let root = VaultRootKey::create()?;
    let bytes = encode_v1_anchor(instance_id, &root, wrapping_key)?;
    write_new_private_file(path, &bytes)?;
    Ok(root)
}

pub fn open_anchor(
    path: &Path,
    wrapping_key: &WrappingKey,
) -> Result<(String, VaultRootKey), VaultAnchorError> {
    let bytes = read_private_regular_file(path)?;
    decode_anchor(&bytes, PLATFORM_SLOT, wrapping_key.as_bytes())
}

pub fn open_anchor_with_recovery(
    path: &Path,
    recovery_key: &VaultRecoveryKeyV1,
) -> Result<(String, VaultRootKey), VaultAnchorError> {
    let bytes = read_private_regular_file(path)?;
    decode_anchor(&bytes, RECOVERY_SLOT, recovery_key.as_bytes())
}

pub(crate) fn copy_private_anchor(
    source: &Path,
    destination: &Path,
) -> Result<(), VaultAnchorError> {
    let bytes = read_private_regular_file(source)?;
    write_new_private_file(destination, &bytes)
}

pub(crate) fn create_restored_anchor(
    source: &Path,
    destination: &Path,
    recovery_key: &VaultRecoveryKeyV1,
    wrapping_key: &WrappingKey,
) -> Result<String, VaultAnchorError> {
    let bytes = read_private_regular_file(source)?;
    let (instance_id, root) = decode_anchor(&bytes, RECOVERY_SLOT, recovery_key.as_bytes())?;
    let replacement = encode_v2_anchor(&instance_id, &root, wrapping_key, recovery_key)?;
    write_new_private_file(destination, &replacement)?;
    Ok(instance_id)
}

pub fn add_recovery_slot(
    path: &Path,
    wrapping_key: &WrappingKey,
    recovery_key: &VaultRecoveryKeyV1,
) -> Result<(), VaultAnchorError> {
    let bytes = read_private_regular_file(path)?;
    let (instance_id, root) = decode_anchor(&bytes, PLATFORM_SLOT, wrapping_key.as_bytes())?;
    if has_recovery_slot(&bytes)? {
        return Err(VaultAnchorError::RecoverySlotExists);
    }
    let replacement = encode_v2_anchor(&instance_id, &root, wrapping_key, recovery_key)?;
    replace_private_file(path, &replacement)
}

pub fn rotate_recovery_slot(
    path: &Path,
    wrapping_key: &WrappingKey,
    current_recovery_key: &VaultRecoveryKeyV1,
    next_recovery_key: &VaultRecoveryKeyV1,
) -> Result<(), VaultAnchorError> {
    let bytes = read_private_regular_file(path)?;
    let (instance_id, platform_root) =
        decode_anchor(&bytes, PLATFORM_SLOT, wrapping_key.as_bytes())?;
    let (_, recovery_root) = decode_anchor(&bytes, RECOVERY_SLOT, current_recovery_key.as_bytes())?;
    if platform_root.0.as_ref() != recovery_root.0.as_ref() {
        return Err(VaultAnchorError::MalformedAnchor);
    }
    let replacement = encode_v2_anchor(
        &instance_id,
        &platform_root,
        wrapping_key,
        next_recovery_key,
    )?;
    replace_private_file(path, &replacement)
}

pub(crate) fn encode_rotated_root_anchor(
    path: &Path,
    current_root: &VaultRootKey,
    next_root: &VaultRootKey,
    wrapping_key: &WrappingKey,
    recovery_key: Option<&VaultRecoveryKeyV1>,
) -> Result<Vec<u8>, VaultAnchorError> {
    let bytes = read_private_regular_file(path)?;
    let (instance_id, platform_root) =
        decode_anchor(&bytes, PLATFORM_SLOT, wrapping_key.as_bytes())?;
    if platform_root.0.as_ref() != current_root.0.as_ref() {
        return Err(VaultAnchorError::RootMismatch);
    }
    if bytes.starts_with(&MAGIC_V1) {
        return encode_v1_anchor(&instance_id, next_root, wrapping_key);
    }
    let recovery_key = recovery_key.ok_or(VaultAnchorError::RecoveryKeyRequired)?;
    let (_, recovery_root) = decode_anchor(&bytes, RECOVERY_SLOT, recovery_key.as_bytes())?;
    if recovery_root.0.as_ref() != current_root.0.as_ref() {
        return Err(VaultAnchorError::RootMismatch);
    }
    encode_v2_anchor(&instance_id, next_root, wrapping_key, recovery_key)
}

pub(crate) fn write_staged_anchor(path: &Path, bytes: &[u8]) -> Result<(), VaultAnchorError> {
    write_new_private_file(path, bytes)
}

fn derive_key(
    instance_id: &str,
    root: &[u8; ROOT_KEY_BYTES],
    info: &[u8],
) -> Result<Zeroizing<[u8; 32]>, VaultAnchorError> {
    let hkdf = Hkdf::<Sha256>::new(Some(instance_id.as_bytes()), root);
    let mut key = Zeroizing::new([0; 32]);
    hkdf.expand(info, key.as_mut())
        .map_err(|_| VaultAnchorError::KeyDerivation)?;
    Ok(key)
}

fn encode_v1_anchor(
    instance_id: &str,
    root: &VaultRootKey,
    wrapping_key: &WrappingKey,
) -> Result<Vec<u8>, VaultAnchorError> {
    let header = instance_header(MAGIC_V1, instance_id)?;
    let slot = encode_slot(&header, PLATFORM_SLOT, root, wrapping_key.as_bytes())?;
    Ok([header, slot].concat())
}

fn encode_v2_anchor(
    instance_id: &str,
    root: &VaultRootKey,
    wrapping_key: &WrappingKey,
    recovery_key: &VaultRecoveryKeyV1,
) -> Result<Vec<u8>, VaultAnchorError> {
    let mut header = instance_header(MAGIC_V2, instance_id)?;
    header.push(2);
    let platform = encode_slot(&header, PLATFORM_SLOT, root, wrapping_key.as_bytes())?;
    let recovery = encode_slot(&header, RECOVERY_SLOT, root, recovery_key.as_bytes())?;
    Ok([header, platform, recovery].concat())
}

fn instance_header(magic: [u8; 4], instance_id: &str) -> Result<Vec<u8>, VaultAnchorError> {
    if instance_id.is_empty() || instance_id.len() > MAX_INSTANCE_ID_BYTES {
        return Err(VaultAnchorError::InvalidInstance);
    }
    let length = u16::try_from(instance_id.len()).map_err(|_| VaultAnchorError::InvalidInstance)?;
    let mut header = Vec::with_capacity(7 + instance_id.len());
    header.extend_from_slice(&magic);
    header.extend_from_slice(&length.to_be_bytes());
    header.extend_from_slice(instance_id.as_bytes());
    Ok(header)
}

fn encode_slot(
    header: &[u8],
    kind: u8,
    root: &VaultRootKey,
    key: &[u8; ROOT_KEY_BYTES],
) -> Result<Vec<u8>, VaultAnchorError> {
    let mut nonce = [0; NONCE_BYTES];
    fill(&mut nonce).map_err(|_| VaultAnchorError::Randomness)?;
    let mut aad = header.to_vec();
    aad.push(kind);
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| VaultAnchorError::Cipher)?;
    let nonce_ref = XNonce::try_from(nonce.as_slice()).map_err(|_| VaultAnchorError::Cipher)?;
    let wrapped = cipher
        .encrypt(
            &nonce_ref,
            Payload {
                msg: root.0.as_ref(),
                aad: &aad,
            },
        )
        .map_err(|_| VaultAnchorError::Cipher)?;
    if wrapped.len() != WRAPPED_ROOT_BYTES {
        return Err(VaultAnchorError::Cipher);
    }
    let mut encoded = Vec::with_capacity(1 + NONCE_BYTES + WRAPPED_ROOT_BYTES);
    encoded.push(kind);
    encoded.extend_from_slice(&nonce);
    encoded.extend_from_slice(&wrapped);
    Ok(encoded)
}

fn decode_anchor(
    bytes: &[u8],
    requested_kind: u8,
    key: &[u8; ROOT_KEY_BYTES],
) -> Result<(String, VaultRootKey), VaultAnchorError> {
    let parsed = parse_anchor(bytes)?;
    let slot = parsed
        .slots
        .iter()
        .find(|slot| slot.kind == requested_kind)
        .ok_or(VaultAnchorError::RecoveryUnavailable)?;
    let mut aad = parsed.header;
    aad.push(slot.kind);
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| VaultAnchorError::Cipher)?;
    let nonce = XNonce::try_from(slot.nonce).map_err(|_| VaultAnchorError::MalformedAnchor)?;
    let root = cipher
        .decrypt(
            &nonce,
            Payload {
                msg: slot.wrapped,
                aad: &aad,
            },
        )
        .map_err(|_| VaultAnchorError::Cipher)?;
    let root: [u8; ROOT_KEY_BYTES] = root
        .try_into()
        .map_err(|_| VaultAnchorError::MalformedAnchor)?;
    Ok((parsed.instance_id, VaultRootKey(Zeroizing::new(root))))
}

fn has_recovery_slot(bytes: &[u8]) -> Result<bool, VaultAnchorError> {
    Ok(parse_anchor(bytes)?
        .slots
        .iter()
        .any(|slot| slot.kind == RECOVERY_SLOT))
}

struct ParsedAnchor<'a> {
    instance_id: String,
    header: Vec<u8>,
    slots: Vec<ParsedSlot<'a>>,
}

struct ParsedSlot<'a> {
    kind: u8,
    nonce: &'a [u8],
    wrapped: &'a [u8],
}

fn parse_anchor(bytes: &[u8]) -> Result<ParsedAnchor<'_>, VaultAnchorError> {
    let (header_end, instance_id) = parse_instance(bytes)?;
    if bytes.starts_with(&MAGIC_V1) {
        parse_v1_anchor(bytes, header_end, instance_id)
    } else if bytes.starts_with(&MAGIC_V2) {
        parse_v2_anchor(bytes, header_end, instance_id)
    } else {
        Err(VaultAnchorError::MalformedAnchor)
    }
}

fn parse_instance(bytes: &[u8]) -> Result<(usize, String), VaultAnchorError> {
    if bytes.len() < 6 {
        return Err(VaultAnchorError::MalformedAnchor);
    }
    let length = usize::from(u16::from_be_bytes([bytes[4], bytes[5]]));
    if length == 0 || length > MAX_INSTANCE_ID_BYTES {
        return Err(VaultAnchorError::MalformedAnchor);
    }
    let end = 6 + length;
    let instance_id =
        std::str::from_utf8(bytes.get(6..end).ok_or(VaultAnchorError::MalformedAnchor)?)
            .map_err(|_| VaultAnchorError::MalformedAnchor)?
            .to_owned();
    Ok((end, instance_id))
}

fn parse_v1_anchor(
    bytes: &[u8],
    header_end: usize,
    instance_id: String,
) -> Result<ParsedAnchor<'_>, VaultAnchorError> {
    let slot = parse_slot(bytes, header_end, PLATFORM_SLOT)?;
    if slot.next != bytes.len() {
        return Err(VaultAnchorError::MalformedAnchor);
    }
    Ok(ParsedAnchor {
        instance_id,
        header: bytes[..header_end].to_vec(),
        slots: vec![slot.slot],
    })
}

fn parse_v2_anchor(
    bytes: &[u8],
    header_end: usize,
    instance_id: String,
) -> Result<ParsedAnchor<'_>, VaultAnchorError> {
    let count = *bytes
        .get(header_end)
        .ok_or(VaultAnchorError::MalformedAnchor)?;
    if count != 2 {
        return Err(VaultAnchorError::MalformedAnchor);
    }
    let first = parse_slot(bytes, header_end + 1, PLATFORM_SLOT)?;
    let second = parse_slot(bytes, first.next, RECOVERY_SLOT)?;
    if second.next != bytes.len() {
        return Err(VaultAnchorError::MalformedAnchor);
    }
    Ok(ParsedAnchor {
        instance_id,
        header: bytes[..header_end + 1].to_vec(),
        slots: vec![first.slot, second.slot],
    })
}

struct SlotWithOffset<'a> {
    slot: ParsedSlot<'a>,
    next: usize,
}

fn parse_slot(
    bytes: &[u8],
    offset: usize,
    expected_kind: u8,
) -> Result<SlotWithOffset<'_>, VaultAnchorError> {
    let end = offset + 1 + NONCE_BYTES + WRAPPED_ROOT_BYTES;
    let segment = bytes
        .get(offset..end)
        .ok_or(VaultAnchorError::MalformedAnchor)?;
    if segment[0] != expected_kind {
        return Err(VaultAnchorError::MalformedAnchor);
    }
    Ok(SlotWithOffset {
        slot: ParsedSlot {
            kind: segment[0],
            nonce: &segment[1..1 + NONCE_BYTES],
            wrapped: &segment[1 + NONCE_BYTES..],
        },
        next: end,
    })
}

fn write_new_private_file(path: &Path, bytes: &[u8]) -> Result<(), VaultAnchorError> {
    let mut file = new_private_file(path)?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| VaultAnchorError::File)?;
    validate_private_file(path)
}

fn replace_private_file(path: &Path, bytes: &[u8]) -> Result<(), VaultAnchorError> {
    let temporary = temporary_path(path)?;
    let result = (|| {
        let mut file = new_private_file(&temporary)?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|_| VaultAnchorError::File)?;
        std::fs::rename(&temporary, path).map_err(|_| VaultAnchorError::File)?;
        File::open(path.parent().ok_or(VaultAnchorError::File)?)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| VaultAnchorError::File)?;
        validate_private_file(path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

fn new_private_file(path: &Path) -> Result<File, VaultAnchorError> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| VaultAnchorError::File)
}

fn temporary_path(path: &Path) -> Result<PathBuf, VaultAnchorError> {
    let parent = path.parent().ok_or(VaultAnchorError::File)?;
    let name = path.file_name().ok_or(VaultAnchorError::File)?;
    let mut suffix = [0_u8; 16];
    fill(&mut suffix).map_err(|_| VaultAnchorError::Randomness)?;
    let suffix = suffix
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(parent.join(format!(".{}.{}.tmp", name.to_string_lossy(), suffix)))
}

fn read_private_regular_file(path: &Path) -> Result<Vec<u8>, VaultAnchorError> {
    let before = std::fs::symlink_metadata(path).map_err(|_| VaultAnchorError::File)?;
    if !is_private_regular_file(&before) {
        return Err(VaultAnchorError::File);
    }
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| VaultAnchorError::File)?;
    let mut bytes = Vec::with_capacity(256);
    file.read_to_end(&mut bytes)
        .map_err(|_| VaultAnchorError::File)?;
    let after = file.metadata().map_err(|_| VaultAnchorError::File)?;
    let path_after = std::fs::symlink_metadata(path).map_err(|_| VaultAnchorError::File)?;
    if !same_file(&before, &after) || !same_file(&before, &path_after) {
        return Err(VaultAnchorError::File);
    }
    Ok(bytes)
}

fn validate_private_file(path: &Path) -> Result<(), VaultAnchorError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| VaultAnchorError::File)?;
    is_private_regular_file(&metadata)
        .then_some(())
        .ok_or(VaultAnchorError::File)
}

fn is_private_regular_file(metadata: &std::fs::Metadata) -> bool {
    metadata.is_file()
        && !metadata.file_type().is_symlink()
        && metadata.uid() == unsafe { libc::geteuid() }
        && metadata.mode() & 0o077 == 0
}

fn same_file(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
}

#[derive(Debug)]
pub enum VaultAnchorError {
    Randomness,
    InvalidInstance,
    KeyDerivation,
    Cipher,
    MalformedAnchor,
    RecoveryUnavailable,
    RecoverySlotExists,
    RecoveryKeyRequired,
    RootMismatch,
    File,
}
