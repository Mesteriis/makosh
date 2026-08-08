use std::path::{Path, PathBuf};

pub fn default_vault_path(home_dir: &Path) -> PathBuf {
    home_dir
        .join(".config")
        .join("makosh")
        .join("secrets.vault.json")
}
