use std::path::Path;

use makosh_vault_key_provider::WrappingKey;

use crate::VaultRecoveryKeyV1;
use crate::database::connection::{configure, open_keyed_connection};
use crate::database::store::{
    VaultStore, VaultStoreError, canonical_store_path, validate_store_parent,
};
use crate::identity::anchor as vault_anchor;
use crate::recovery::{backup, root_rotation};

impl VaultStore {
    /// Exports a new encrypted Vault snapshot while no Vault process owns the store.
    pub fn export_backup_offline(
        path: &Path,
        anchor_path: &Path,
        destination: &Path,
        wrapping_key: &WrappingKey,
    ) -> Result<backup::VaultBackupManifestV1, VaultStoreError> {
        let path = canonical_store_path(path)?;
        validate_store_parent(&path)?;
        validate_store_parent(anchor_path)?;
        if root_rotation::rotation_pending(&path)? {
            return Err(VaultStoreError::RootRotationPending);
        }
        let (instance_id, root) = vault_anchor::open_anchor(anchor_path, wrapping_key)
            .map_err(|_| VaultStoreError::Anchor)?;
        let sqlcipher_key = root
            .derive_sqlcipher_key(&instance_id)
            .map_err(|_| VaultStoreError::Anchor)?;
        let manifest_key = root
            .derive_backup_manifest_key(&instance_id)
            .map_err(|_| VaultStoreError::Anchor)?;
        let paths = backup::create_destination(destination)?;
        backup::export_database_snapshot(&path, &paths.database(), &sqlcipher_key, &instance_id)?;
        vault_anchor::copy_private_anchor(anchor_path, &paths.anchor())
            .map_err(|_| VaultStoreError::Backup)?;
        let manifest = backup::create_manifest(
            &paths.manifest(),
            &instance_id,
            &paths.database(),
            &paths.anchor(),
            &manifest_key,
        )?;
        let verified = Self::verify_backup_offline(paths.directory(), wrapping_key)?;
        if verified != manifest {
            return Err(VaultStoreError::Backup);
        }
        backup::sync_directory(paths.directory())?;
        Ok(manifest)
    }

    /// Verifies an immutable offline Vault snapshot before any restore attempt.
    pub fn verify_backup_offline(
        destination: &Path,
        wrapping_key: &WrappingKey,
    ) -> Result<backup::VaultBackupManifestV1, VaultStoreError> {
        let paths = backup::open_destination(destination)?;
        let (instance_id, root) = vault_anchor::open_anchor(&paths.anchor(), wrapping_key)
            .map_err(|_| VaultStoreError::Backup)?;
        let sqlcipher_key = root
            .derive_sqlcipher_key(&instance_id)
            .map_err(|_| VaultStoreError::Backup)?;
        let manifest_key = root
            .derive_backup_manifest_key(&instance_id)
            .map_err(|_| VaultStoreError::Backup)?;
        let manifest = backup::verify_manifest(
            &paths.manifest(),
            &paths.database(),
            &paths.anchor(),
            &manifest_key,
        )?;
        if manifest.instance_id() != instance_id {
            return Err(VaultStoreError::Backup);
        }
        let database = open_keyed_connection(&paths.database(), &sqlcipher_key)?;
        configure(&database)?;
        backup::validate_database(&database, &instance_id)?;
        Ok(manifest)
    }

    /// Restores a verified snapshot only into a new empty Vault contour.
    pub fn restore_backup_offline(
        backup_directory: &Path,
        destination_database: &Path,
        destination_anchor: &Path,
        recovery_key: &VaultRecoveryKeyV1,
        wrapping_key: &WrappingKey,
    ) -> Result<backup::VaultBackupManifestV1, VaultStoreError> {
        let destination_database = canonical_store_path(destination_database)?;
        validate_store_parent(&destination_database)?;
        validate_store_parent(destination_anchor)?;
        if destination_database.exists() || destination_anchor.exists() {
            return Err(VaultStoreError::AlreadyInitialized);
        }
        let backup_paths = backup::open_destination(backup_directory)?;
        let (instance_id, root) =
            vault_anchor::open_anchor_with_recovery(&backup_paths.anchor(), recovery_key)
                .map_err(|_| VaultStoreError::Backup)?;
        let sqlcipher_key = root
            .derive_sqlcipher_key(&instance_id)
            .map_err(|_| VaultStoreError::Backup)?;
        let manifest_key = root
            .derive_backup_manifest_key(&instance_id)
            .map_err(|_| VaultStoreError::Backup)?;
        let manifest = backup::verify_manifest(
            &backup_paths.manifest(),
            &backup_paths.database(),
            &backup_paths.anchor(),
            &manifest_key,
        )?;
        if manifest.instance_id() != instance_id {
            return Err(VaultStoreError::Backup);
        }
        backup::export_database_snapshot(
            &backup_paths.database(),
            &destination_database,
            &sqlcipher_key,
            &instance_id,
        )?;
        let restored_instance = vault_anchor::create_restored_anchor(
            &backup_paths.anchor(),
            destination_anchor,
            recovery_key,
            wrapping_key,
        )
        .map_err(|_| VaultStoreError::Backup)?;
        if restored_instance != instance_id {
            return Err(VaultStoreError::Backup);
        }
        let restored = Self::open(&destination_database, destination_anchor, wrapping_key)?;
        if restored.instance_id() != instance_id {
            return Err(VaultStoreError::Backup);
        }
        drop(restored);
        Ok(manifest)
    }
}
