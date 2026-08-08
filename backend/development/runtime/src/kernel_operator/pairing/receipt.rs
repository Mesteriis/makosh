use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use makosh_kernel_control_store::InitialOwnerIdentity;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};
use sha2::{Digest, Sha256};

const STATE_FILE: &str = "development-remote-pairing-v1.state";
const RECEIPT_FILE: &str = "development-remote-pairing-v1.receipt";
const RECEIPT_HEADER: &str = "development_remote_pairing_receipt_v1";
const PROOF_DOMAIN: &[u8] = b"makosh.development.remote-pairing.v1\0";
const MAX_RECEIPT_BYTES: u64 = 1024;

pub fn load_verified_identity(state_dir: &Path) -> Result<InitialOwnerIdentity, String> {
    ensure_private_directory(state_dir)?;
    let expected_receipt_sha256 = read_consumed_receipt_digest(state_dir)?;
    let receipt_path = state_dir.join(RECEIPT_FILE);
    let metadata = std::fs::symlink_metadata(&receipt_path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("development pairing receipt must be a regular file".to_owned());
    }
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err("development pairing receipt must be owner-private".to_owned());
    }
    if metadata.len() > MAX_RECEIPT_BYTES {
        return Err("development pairing receipt is too large".to_owned());
    }
    let bytes = std::fs::read(&receipt_path).map_err(|error| error.to_string())?;
    let observed: [u8; 32] = Sha256::digest(&bytes).into();
    if !constant_time_equal(&expected_receipt_sha256, &observed) {
        return Err("development pairing receipt does not match consumed state".to_owned());
    }
    parse_and_verify_receipt(&bytes)
}

fn ensure_private_directory(directory: &Path) -> Result<(), String> {
    if !directory.is_absolute() {
        return Err("development pairing state directory must be absolute".to_owned());
    }
    let metadata = std::fs::symlink_metadata(directory).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("development pairing state directory must be a directory".to_owned());
    }
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err("development pairing state directory must be owner-private".to_owned());
    }
    Ok(())
}

fn read_consumed_receipt_digest(state_dir: &Path) -> Result<[u8; 32], String> {
    let path = state_dir.join(STATE_FILE);
    let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("development pairing state must be a regular file".to_owned());
    }
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err("development pairing state must be owner-private".to_owned());
    }
    let content = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut lines = content.lines();
    if lines.next() != Some("development_remote_pairing_v1")
        || lines.next() != Some("consumed")
        || lines
            .next()
            .and_then(|value| value.parse::<u128>().ok())
            .is_none()
        || lines
            .next()
            .and_then(|value| decode_hex::<32>(value).ok())
            .is_none()
    {
        return Err("development pairing state is invalid".to_owned());
    }
    let digest = lines
        .next()
        .filter(|value| *value != "-")
        .ok_or_else(|| "development pairing state has no enrollment receipt".to_owned())
        .and_then(decode_hex::<32>)?;
    if lines.next().is_some() {
        return Err("development pairing state is invalid".to_owned());
    }
    Ok(digest)
}

fn parse_and_verify_receipt(bytes: &[u8]) -> Result<InitialOwnerIdentity, String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| "development pairing receipt is not UTF-8".to_owned())?;
    let mut lines = text.lines();
    if lines.next() != Some(RECEIPT_HEADER) {
        return Err("development pairing receipt is invalid".to_owned());
    }
    let owner_id = lines
        .next()
        .filter(valid_identity)
        .ok_or_else(|| "development pairing receipt owner is invalid".to_owned())?;
    let device_id = lines
        .next()
        .filter(valid_identity)
        .ok_or_else(|| "development pairing receipt device is invalid".to_owned())?;
    let challenge = lines
        .next()
        .ok_or_else(|| "development pairing receipt challenge is missing".to_owned())
        .and_then(decode_hex::<32>)?;
    let public_key = lines
        .next()
        .ok_or_else(|| "development pairing receipt public key is missing".to_owned())
        .and_then(decode_hex::<65>)?;
    let signature = lines
        .next()
        .ok_or_else(|| "development pairing receipt signature is missing".to_owned())
        .and_then(decode_hex::<64>)?;
    if lines.next().is_some() {
        return Err("development pairing receipt is invalid".to_owned());
    }
    let verifying_key = VerifyingKey::from_sec1_bytes(&public_key)
        .map_err(|_| "development pairing receipt public key is invalid".to_owned())?;
    let signature = Signature::from_slice(&signature)
        .map_err(|_| "development pairing receipt signature is invalid".to_owned())?;
    verifying_key
        .verify(
            &proof_message(&challenge, &public_key, owner_id, device_id),
            &signature,
        )
        .map_err(|_| "development pairing receipt proof verification failed".to_owned())?;
    Ok(InitialOwnerIdentity::new(owner_id, device_id, public_key))
}

fn proof_message(
    challenge: &[u8; 32],
    public_key: &[u8; 65],
    owner_id: &str,
    device_id: &str,
) -> Vec<u8> {
    let mut message = Vec::with_capacity(
        PROOF_DOMAIN.len()
            + challenge.len()
            + public_key.len()
            + owner_id.len()
            + device_id.len()
            + 4,
    );
    message.extend_from_slice(PROOF_DOMAIN);
    message.extend_from_slice(challenge);
    message.extend_from_slice(public_key);
    message.extend_from_slice(&(owner_id.len() as u16).to_be_bytes());
    message.extend_from_slice(owner_id.as_bytes());
    message.extend_from_slice(&(device_id.len() as u16).to_be_bytes());
    message.extend_from_slice(device_id.as_bytes());
    message
}

fn decode_hex<const N: usize>(value: &str) -> Result<[u8; N], String> {
    if value.len() != N * 2 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("development pairing receipt contains invalid hexadecimal data".to_owned());
    }
    let mut bytes = [0_u8; N];
    for (index, output) in bytes.iter_mut().enumerate() {
        *output = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).map_err(|_| {
            "development pairing receipt contains invalid hexadecimal data".to_owned()
        })?;
    }
    Ok(bytes)
}

fn valid_identity(value: &&str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn constant_time_equal(left: &[u8; 32], right: &[u8; 32]) -> bool {
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}
