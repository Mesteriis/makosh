//! Offline Vault snapshot commands; no live runtime or secret argv is allowed.

use std::path::Path;

use makosh_secure_file::{SecureReadPolicy, read as read_secure_file};
use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_store_sqlcipher::{VaultRecoveryKeyV1, VaultStore};
use zeroize::Zeroize;

const MAX_RECOVERY_KEY_FILE_BYTES: u64 = 4096;

pub(crate) fn export(data_dir: &Path, destination: &Path) -> Result<(), String> {
    crate::ensure_private_directory(data_dir)?;
    let wrapping_key = load_wrapping_key(data_dir)?;
    VaultStore::export_backup_offline(
        &data_dir.join("vault.db"),
        &data_dir.join("vault.anchor"),
        destination,
        &wrapping_key,
    )
    .map_err(|_| "Vault backup export failed".to_owned())?;
    println!("vault_backup=exported");
    Ok(())
}

pub(crate) fn verify(data_dir: &Path, source: &Path) -> Result<(), String> {
    crate::ensure_private_directory(data_dir)?;
    let wrapping_key = load_wrapping_key(data_dir)?;
    VaultStore::verify_backup_offline(source, &wrapping_key)
        .map_err(|_| "Vault backup verification failed".to_owned())?;
    println!("vault_backup=verified");
    Ok(())
}

pub(crate) fn restore(
    data_dir: &Path,
    source: &Path,
    recovery_key_file: &Path,
) -> Result<(), String> {
    ensure_empty_private_directory(data_dir)?;
    let recovery_key = load_recovery_key(recovery_key_file)?;
    let wrapping_key = load_wrapping_key(data_dir)?;
    VaultStore::restore_backup_offline(
        source,
        &data_dir.join("vault.db"),
        &data_dir.join("vault.anchor"),
        &recovery_key,
        &wrapping_key,
    )
    .map_err(|_| "Vault backup restore failed".to_owned())?;
    println!("vault_backup=restored");
    Ok(())
}

fn load_wrapping_key(data_dir: &Path) -> Result<makosh_vault_key_provider::WrappingKey, String> {
    FileWrappingKeyProvider::new(&data_dir.join("platform-wrapping-key.bin"))
        .load_or_create()
        .map_err(|_| "Vault file key is unavailable".to_owned())
}

fn load_recovery_key(path: &Path) -> Result<VaultRecoveryKeyV1, String> {
    let mut bytes = read_secure_file(
        path,
        SecureReadPolicy::owner_private(MAX_RECOVERY_KEY_FILE_BYTES),
    )
    .map_err(|_| "Vault recovery key is unavailable".to_owned())?;
    let result = std::str::from_utf8(&bytes)
        .ok()
        .and_then(|words| VaultRecoveryKeyV1::from_mnemonic(words.trim()).ok());
    bytes.zeroize();
    result.ok_or_else(|| "Vault recovery key is unavailable".to_owned())
}

fn ensure_empty_private_directory(path: &Path) -> Result<(), String> {
    crate::ensure_private_directory(path)?;
    std::fs::read_dir(path)
        .map_err(|_| "Vault restore target is unavailable".to_owned())?
        .next()
        .transpose()
        .map_err(|_| "Vault restore target is unavailable".to_owned())?
        .is_none()
        .then_some(())
        .ok_or_else(|| "Vault restore target must be empty".to_owned())
}
