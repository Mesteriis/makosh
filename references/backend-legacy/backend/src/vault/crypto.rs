use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::Utc;
use hkdf::Hkdf;
use sha2::{Digest, Sha512};

use super::constants::{MASTER_KEY_LEN, MIN_ENTROPY_EVENTS, VAULT_VERSION};
use super::errors::HostVaultError;
use super::models::{EntropyEvent, SecretEntryContext};

pub(super) fn derive_master_key(
    os_random: &[u8],
    entropy: &[EntropyEvent],
) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
    let entropy_json = serde_json::to_vec(entropy)?;
    let mut hasher = Sha512::new();
    hasher.update(os_random);
    hasher.update(&entropy_json);
    hasher.update(
        Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_default()
            .to_be_bytes(),
    );
    let digest = hasher.finalize();
    let hkdf = Hkdf::<sha2::Sha256>::new(None, &digest);
    let mut key = [0_u8; MASTER_KEY_LEN];
    hkdf.expand(b"makosh-host-vault:master:v1", &mut key)
        .map_err(|_| HostVaultError::Crypto)?;
    Ok(key)
}

pub(super) fn derive_domain_key(
    master_key: &[u8; MASTER_KEY_LEN],
    label: &[u8],
) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
    let hkdf = Hkdf::<sha2::Sha256>::new(None, master_key);
    let mut key = [0_u8; MASTER_KEY_LEN];
    let mut info = b"makosh-host-vault:v1:".to_vec();
    info.extend_from_slice(label);
    hkdf.expand(&info, &mut key)
        .map_err(|_| HostVaultError::Crypto)?;
    Ok(key)
}

pub(super) fn entry_aad(secret_ref: &str, context: SecretEntryContext<'_>) -> String {
    format!(
        "v={VAULT_VERSION};ref={};kind={};account_id={};purpose={};secret_kind={}",
        secret_ref.trim(),
        context.entry_kind.trim(),
        context.account_id.trim(),
        context.purpose.trim(),
        context.secret_kind.trim()
    )
}

pub(super) fn recovery_phrase(master_key: &[u8; MASTER_KEY_LEN]) -> Result<String, HostVaultError> {
    Ok(master_key
        .chunks(2)
        .map(|chunk| format!("{:02x}{:02x}", chunk[0], chunk[1]))
        .collect::<Vec<_>>()
        .join(" "))
}

pub(super) fn master_key_from_recovery_phrase(
    phrase: &str,
) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
    let compact = phrase.split_whitespace().collect::<String>();
    if compact.len() != MASTER_KEY_LEN * 2 {
        return Err(HostVaultError::InvalidRecoveryPhrase);
    }
    let mut key = [0_u8; MASTER_KEY_LEN];
    for index in 0..MASTER_KEY_LEN {
        let byte = u8::from_str_radix(&compact[index * 2..index * 2 + 2], 16)
            .map_err(|_| HostVaultError::InvalidRecoveryPhrase)?;
        key[index] = byte;
    }
    Ok(key)
}

pub(super) fn decode_master_key(encoded: &str) -> Result<[u8; MASTER_KEY_LEN], HostVaultError> {
    let decoded = BASE64_STANDARD
        .decode(encoded.trim())
        .map_err(|_| HostVaultError::InvalidEncoding)?;
    decoded
        .try_into()
        .map_err(|_| HostVaultError::InvalidEncoding)
}

pub(super) fn entropy_progress(events: usize) -> u8 {
    ((events.min(MIN_ENTROPY_EVENTS) * 100) / MIN_ENTROPY_EVENTS) as u8
}

pub(super) fn validate_non_empty(field: &'static str, value: &str) -> Result<(), HostVaultError> {
    if value.trim().is_empty() {
        return Err(HostVaultError::EmptyField(field));
    }
    Ok(())
}
