//! Root-rotation path derivation and directory durability helpers.

use std::fs::File;
use std::path::{Path, PathBuf};

use crate::database::store::VaultStoreError;

pub(super) const JOURNAL_NAME: &str = ".makosh-vault-root-rotation-v1";

pub(super) fn journal_path(database_path: &Path) -> Result<PathBuf, VaultStoreError> {
    database_path
        .parent()
        .map(|parent| parent.join(JOURNAL_NAME))
        .ok_or(VaultStoreError::InsecurePath)
}

pub(super) fn staged_path(path: &Path, kind: &str) -> Result<PathBuf, VaultStoreError> {
    let parent = path.parent().ok_or(VaultStoreError::InsecurePath)?;
    let name = path.file_name().ok_or(VaultStoreError::InsecurePath)?;
    Ok(parent.join(format!(
        ".{}.root-rotation-{kind}.tmp",
        name.to_string_lossy()
    )))
}

pub(super) fn sync_parent(path: &Path) -> Result<(), VaultStoreError> {
    File::open(path.parent().ok_or(VaultStoreError::InsecurePath)?)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| VaultStoreError::InsecurePath)
}
