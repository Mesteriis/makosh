use std::path::{Path, PathBuf};

pub fn default_vault_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".makosh").join("vault")
}

pub fn default_dev_key_path(home_dir: &Path) -> PathBuf {
    home_dir.join(".makosh").join("dev").join("master.key")
}
