//! One-shot `initdb --pwfile` material owned by Storage Control.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use getrandom::fill;
use zeroize::Zeroizing;

use crate::PostgresAdapterErrorV1;

const PASSWORD_FILE_PREFIX: &str = ".makosh-initdb-password-";
const PASSWORD_FILE_ATTEMPTS: u8 = 3;

pub struct InitdbPasswordFileV1 {
    path: PathBuf,
}

impl InitdbPasswordFileV1 {
    pub fn create(
        runtime_dir: &Path,
        password: &Zeroizing<Vec<u8>>,
    ) -> Result<Self, PostgresAdapterErrorV1> {
        validate_runtime_directory(runtime_dir)?;
        validate_password(password)?;
        for _ in 0..PASSWORD_FILE_ATTEMPTS {
            let path = runtime_dir.join(password_file_name()?);
            match create_file(&path, password) {
                Ok(()) => return Ok(Self { path }),
                Err(CreatePasswordFileError::Exists) => continue,
                Err(CreatePasswordFileError::Invalid) => {
                    return Err(PostgresAdapterErrorV1::Bootstrap);
                }
            }
        }
        Err(PostgresAdapterErrorV1::Bootstrap)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn remove(mut self) -> Result<(), PostgresAdapterErrorV1> {
        let path = std::mem::take(&mut self.path);
        std::fs::remove_file(path).map_err(|_| PostgresAdapterErrorV1::Bootstrap)
    }
}

impl Drop for InitdbPasswordFileV1 {
    fn drop(&mut self) {
        if !self.path.as_os_str().is_empty() {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

fn validate_runtime_directory(runtime_dir: &Path) -> Result<(), PostgresAdapterErrorV1> {
    let metadata =
        std::fs::symlink_metadata(runtime_dir).map_err(|_| PostgresAdapterErrorV1::Bootstrap)?;
    let owner_matches = metadata.uid() == current_uid();
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.mode() & 0o777 != 0o700
        || !owner_matches
    {
        return Err(PostgresAdapterErrorV1::Bootstrap);
    }
    Ok(())
}

fn validate_password(password: &[u8]) -> Result<(), PostgresAdapterErrorV1> {
    if password.is_empty()
        || password.len() > 4 * 1024
        || password
            .iter()
            .any(|byte| matches!(byte, b'\0' | b'\r' | b'\n'))
    {
        return Err(PostgresAdapterErrorV1::Bootstrap);
    }
    Ok(())
}

fn password_file_name() -> Result<String, PostgresAdapterErrorV1> {
    let mut entropy = [0_u8; 16];
    fill(&mut entropy).map_err(|_| PostgresAdapterErrorV1::Bootstrap)?;
    let mut name = String::with_capacity(PASSWORD_FILE_PREFIX.len() + entropy.len() * 2);
    name.push_str(PASSWORD_FILE_PREFIX);
    for byte in entropy {
        use std::fmt::Write as _;
        write!(&mut name, "{byte:02x}").map_err(|_| PostgresAdapterErrorV1::Bootstrap)?;
    }
    Ok(name)
}

fn create_file(path: &Path, password: &[u8]) -> Result<(), CreatePasswordFileError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(map_create_error)?;
    let write_result = file
        .write_all(password)
        .and_then(|()| file.write_all(b"\n"))
        .and_then(|()| file.sync_all());
    if write_result.is_err() || validate_password_file(&file).is_err() {
        let _ = std::fs::remove_file(path);
        return Err(CreatePasswordFileError::Invalid);
    }
    Ok(())
}

fn validate_password_file(file: &File) -> Result<(), CreatePasswordFileError> {
    let metadata = file
        .metadata()
        .map_err(|_| CreatePasswordFileError::Invalid)?;
    let owner_matches = metadata.uid() == current_uid();
    if !metadata.is_file() || metadata.mode() & 0o777 != 0o600 || !owner_matches {
        return Err(CreatePasswordFileError::Invalid);
    }
    Ok(())
}

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and returns only the effective UID.
    unsafe { libc::geteuid() }
}

fn map_create_error(error: std::io::Error) -> CreatePasswordFileError {
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        CreatePasswordFileError::Exists
    } else {
        CreatePasswordFileError::Invalid
    }
}

enum CreatePasswordFileError {
    Exists,
    Invalid,
}
