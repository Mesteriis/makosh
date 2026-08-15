use std::fs::File;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

pub fn resolve_data_directory(override_path: Option<PathBuf>) -> Result<PathBuf, String> {
    match override_path {
        Some(path) if path.is_absolute() => Ok(path),
        Some(_) => Err("data directory must be an absolute path".to_owned()),
        None => directories::ProjectDirs::from("dev", "Макошь", "Макошь")
            .map(|directories| directories.data_local_dir().to_owned())
            .ok_or_else(|| "OS-standard local data directory is unavailable".to_owned()),
    }
}

pub fn resolve_runtime_directory(data_dir: &Path) -> Result<PathBuf, String> {
    let directories = directories::ProjectDirs::from("dev", "makosh", "makosh")
        .ok_or_else(|| "OS-standard local runtime directory is unavailable".to_owned())?;
    let canonical_data_dir = data_dir.canonicalize().map_err(|error| error.to_string())?;
    let instance_key = Sha256::digest(canonical_data_dir.as_os_str().as_encoded_bytes())
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(directories.cache_dir().join("runtime").join(instance_key))
}

pub fn acquire_runtime_directory_lock(runtime_dir: &Path) -> Result<File, String> {
    let lock_path = runtime_dir.join("kernel.lock");
    ensure_regular_file_or_absent(&lock_path, "runtime lock")?;
    let lock = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)
        .map_err(|error| error.to_string())?;
    lock.try_lock()
        .map_err(|_| "data directory is already in use".to_owned())?;
    Ok(lock)
}

pub fn ensure_owner_private_directory(directory: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(directory).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("data directory must be a real directory".to_owned());
    }
    if metadata.uid() != current_uid() {
        return Err("data directory must be owned by the current user".to_owned());
    }
    let mode = metadata.permissions().mode();
    if mode & 0o077 != 0 {
        return Err("data directory must be owner-private".to_owned());
    }
    Ok(())
}

pub fn remove_stale_owner_unix_socket(path: &Path, label: &str) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(format!("{label} must not be a symlink"))
        }
        Ok(metadata) if !metadata.file_type().is_socket() => {
            Err(format!("{label} must be a Unix socket"))
        }
        Ok(metadata) if metadata.uid() != current_uid() => {
            Err(format!("{label} must be owned by the current user"))
        }
        Ok(_) => std::fs::remove_file(path).map_err(|error| error.to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub fn ensure_not_symlink(path: &Path, label: &str) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(format!("{label} must not be a symlink"))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub fn ensure_regular_file_or_absent(path: &Path, label: &str) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(format!("{label} must not be a symlink"))
        }
        Ok(metadata) if metadata.file_type().is_file() => Ok(()),
        Ok(_) => Err(format!("{label} must be a regular file")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub fn prepare_owner_private_directory(directory: &Path) -> Result<(), String> {
    let existed = directory.exists();
    ensure_not_symlink(directory, "recovery export directory")?;
    std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    if existed {
        ensure_owner_private_directory(directory)
    } else {
        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())
    }
}

pub fn new_instance_id() -> Result<String, String> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|error| error.to_string())?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and only reads process credentials.
    unsafe { libc::geteuid() }
}
