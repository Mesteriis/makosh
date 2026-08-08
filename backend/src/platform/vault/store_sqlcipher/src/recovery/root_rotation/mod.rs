//! Offline root-key rotation orchestration for the encrypted Vault store.

mod database;
mod journal;
mod paths;

use std::path::Path;

use makosh_vault_key_provider::WrappingKey;
use zeroize::Zeroizing;

use crate::database::store::VaultStoreError;
use crate::identity::VaultRecoveryKeyV1;
use crate::identity::anchor::{self as vault_anchor, VaultRootKey};
use crate::records::secret as secret_record;

pub(crate) fn rotate(
    database_path: &Path,
    anchor_path: &Path,
    instance_id: &str,
    current_root: &VaultRootKey,
    wrapping_key: &WrappingKey,
    recovery_key: Option<&VaultRecoveryKeyV1>,
) -> Result<(), VaultStoreError> {
    if finalize_pending(database_path, anchor_path)? {
        return Ok(());
    }
    let next_root = VaultRootKey::create().map_err(|_| VaultStoreError::Anchor)?;
    let current_sql_key = derive_sql_key(current_root, instance_id)?;
    let next_sql_key = derive_sql_key(&next_root, instance_id)?;
    let current_record_key = derive_record_key(current_root, instance_id)?;
    let next_record_key = derive_record_key(&next_root, instance_id)?;
    let staged_database = paths::staged_path(database_path, "database")?;
    let staged_anchor = paths::staged_path(anchor_path, "anchor")?;
    database::stage(
        database_path,
        &staged_database,
        &current_sql_key,
        &next_sql_key,
        &current_record_key,
        &next_record_key,
    )?;
    let result = stage_anchor_and_install(RotationInstallInput {
        database_path,
        anchor_path,
        current_root,
        next_root: &next_root,
        wrapping_key,
        recovery_key,
        staged_database: &staged_database,
        staged_anchor: &staged_anchor,
    });
    if result.is_err() && !journal::exists(database_path)? {
        let _ = std::fs::remove_file(&staged_database);
        let _ = std::fs::remove_file(&staged_anchor);
    }
    result
}

pub(crate) fn rotation_pending(database_path: &Path) -> Result<bool, VaultStoreError> {
    journal::exists(database_path)
}

struct RotationInstallInput<'a> {
    database_path: &'a Path,
    anchor_path: &'a Path,
    current_root: &'a VaultRootKey,
    next_root: &'a VaultRootKey,
    wrapping_key: &'a WrappingKey,
    recovery_key: Option<&'a VaultRecoveryKeyV1>,
    staged_database: &'a Path,
    staged_anchor: &'a Path,
}

fn stage_anchor_and_install(input: RotationInstallInput<'_>) -> Result<(), VaultStoreError> {
    let anchor = vault_anchor::encode_rotated_root_anchor(
        input.anchor_path,
        input.current_root,
        input.next_root,
        input.wrapping_key,
        input.recovery_key,
    )
    .map_err(|_| VaultStoreError::Anchor)?;
    vault_anchor::write_staged_anchor(input.staged_anchor, &anchor)
        .map_err(|_| VaultStoreError::Anchor)?;
    let reservation =
        journal::Reservation::from_staged(input.staged_database, input.staged_anchor)?;
    journal::write(input.database_path, &reservation)?;
    install_staged_pair(
        input.database_path,
        input.anchor_path,
        input.staged_database,
        input.staged_anchor,
    )
}

fn install_staged_pair(
    database_path: &Path,
    anchor_path: &Path,
    staged_database: &Path,
    staged_anchor: &Path,
) -> Result<(), VaultStoreError> {
    std::fs::rename(staged_database, database_path).map_err(|_| VaultStoreError::InsecurePath)?;
    paths::sync_parent(database_path)?;
    std::fs::rename(staged_anchor, anchor_path).map_err(|_| VaultStoreError::InsecurePath)?;
    paths::sync_parent(anchor_path)?;
    journal::remove(database_path)
}

fn finalize_pending(database_path: &Path, anchor_path: &Path) -> Result<bool, VaultStoreError> {
    if !journal::exists(database_path)? {
        return Ok(false);
    }
    let reservation = journal::read(database_path)?;
    if journal::digest(database_path)? != reservation.database_digest() {
        return Err(VaultStoreError::RootRotationPending);
    }
    if journal::digest(anchor_path)? == reservation.anchor_digest() {
        journal::remove(database_path)?;
        return Ok(true);
    }
    let staged_anchor = paths::staged_path(anchor_path, "anchor")?;
    if journal::digest(&staged_anchor)? != reservation.anchor_digest() {
        return Err(VaultStoreError::RootRotationPending);
    }
    std::fs::rename(staged_anchor, anchor_path).map_err(|_| VaultStoreError::InsecurePath)?;
    paths::sync_parent(anchor_path)?;
    journal::remove(database_path)?;
    Ok(true)
}

fn derive_sql_key(
    root: &VaultRootKey,
    instance_id: &str,
) -> Result<Zeroizing<[u8; 32]>, VaultStoreError> {
    root.derive_sqlcipher_key(instance_id)
        .map_err(|_| VaultStoreError::Anchor)
}

fn derive_record_key(
    root: &VaultRootKey,
    instance_id: &str,
) -> Result<Zeroizing<[u8; 32]>, VaultStoreError> {
    root.derive_record_key(instance_id, secret_record::CURRENT_KEY_EPOCH)
        .map_err(|_| VaultStoreError::Anchor)
}
