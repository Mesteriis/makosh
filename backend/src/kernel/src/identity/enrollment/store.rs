//! Pristine-instance Control Store preparation shared by first-owner ceremonies.

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::control_store::lifecycle::bootstrap_control_store;
use crate::infrastructure::filesystem::{
    acquire_runtime_directory_lock, ensure_not_symlink, ensure_owner_private_directory,
    resolve_data_directory, resolve_runtime_directory,
};

pub fn prepare_pristine(
    data_dir_override: Option<PathBuf>,
) -> Result<(PathBuf, std::fs::File, SqliteControlStore), String> {
    let data_dir = resolve_data_directory(data_dir_override)?;
    let data_dir_existed = data_dir.exists();
    ensure_not_symlink(&data_dir, "data directory")?;
    std::fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
    ensure_not_symlink(&data_dir, "data directory")?;
    if data_dir_existed {
        ensure_owner_private_directory(&data_dir)?;
    } else {
        std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    let runtime_dir_existed = runtime_dir.exists();
    ensure_not_symlink(&runtime_dir, "runtime directory")?;
    std::fs::create_dir_all(&runtime_dir).map_err(|error| error.to_string())?;
    if runtime_dir_existed {
        ensure_owner_private_directory(&runtime_dir)?;
    } else {
        std::fs::set_permissions(&runtime_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    let lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let store = bootstrap_control_store(&data_dir, &data_dir.join("kernel-control-store.sqlite"))?;
    if store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .is_some()
    {
        return Err("initial owner is already enrolled for this instance".to_owned());
    }
    Ok((data_dir, lock, store))
}
