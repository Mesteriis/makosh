//! Owner-private regular-file adapter for a Vault platform wrapping key.

use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use makosh_secure_file::{SecureReadPolicy, read as read_secure_file};
use makosh_vault_key_provider::{WrappingKey, WrappingKeyProvider};

const KEY_BYTES: usize = 32;

pub struct FileWrappingKeyProvider {
    path: PathBuf,
}

impl FileWrappingKeyProvider {
    #[must_use]
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
        }
    }
}

impl WrappingKeyProvider for FileWrappingKeyProvider {
    type Error = FileWrappingKeyError;

    fn load_or_create(&self) -> Result<WrappingKey, Self::Error> {
        match std::fs::symlink_metadata(&self.path) {
            Ok(metadata) => load_existing(&self.path, &metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => create_new(&self.path),
            Err(error) => Err(FileWrappingKeyError::Io(error)),
        }
    }
}

#[derive(Debug)]
pub enum FileWrappingKeyError {
    InsecureKeyFile,
    MissingParent,
    Randomness,
    Io(std::io::Error),
}

impl std::fmt::Display for FileWrappingKeyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InsecureKeyFile => "Vault wrapping-key file is not a private regular file",
            Self::MissingParent => "Vault wrapping-key path has no parent directory",
            Self::Randomness => "OS randomness is unavailable for Vault wrapping key",
            Self::Io(_) => "Vault wrapping-key file operation failed",
        })
    }
}

impl std::error::Error for FileWrappingKeyError {}

fn load_existing(
    path: &Path,
    metadata: &std::fs::Metadata,
) -> Result<WrappingKey, FileWrappingKeyError> {
    if !is_private_regular_file(metadata) {
        return Err(FileWrappingKeyError::InsecureKeyFile);
    }
    let bytes = read_secure_file(path, SecureReadPolicy::owner_private(KEY_BYTES as u64)).map_err(
        |error| match error {
            makosh_secure_file::SecureFileError::Io(error) => FileWrappingKeyError::Io(error),
            makosh_secure_file::SecureFileError::InvalidFile
            | makosh_secure_file::SecureFileError::TooLarge => {
                FileWrappingKeyError::InsecureKeyFile
            }
        },
    )?;
    let bytes: [u8; KEY_BYTES] = bytes
        .try_into()
        .map_err(|_| FileWrappingKeyError::InsecureKeyFile)?;
    Ok(WrappingKey::from_bytes(bytes))
}

fn create_new(path: &Path) -> Result<WrappingKey, FileWrappingKeyError> {
    let parent = path.parent().ok_or(FileWrappingKeyError::MissingParent)?;
    let parent_metadata = std::fs::symlink_metadata(parent).map_err(FileWrappingKeyError::Io)?;
    if parent_metadata.file_type().is_symlink()
        || !parent_metadata.is_dir()
        || parent_metadata.mode() & 0o077 != 0
    {
        return Err(FileWrappingKeyError::InsecureKeyFile);
    }
    let mut bytes = [0; KEY_BYTES];
    getrandom::fill(&mut bytes).map_err(|_| FileWrappingKeyError::Randomness)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(FileWrappingKeyError::Io)?;
    if let Err(error) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
        let _ = std::fs::remove_file(path);
        return Err(FileWrappingKeyError::Io(error));
    }
    let metadata = std::fs::symlink_metadata(path).map_err(FileWrappingKeyError::Io)?;
    if !is_private_regular_file(&metadata) {
        return Err(FileWrappingKeyError::InsecureKeyFile);
    }
    Ok(WrappingKey::from_bytes(bytes))
}

fn is_private_regular_file(metadata: &std::fs::Metadata) -> bool {
    metadata.is_file()
        && !metadata.file_type().is_symlink()
        && metadata.len() == KEY_BYTES as u64
        && metadata.uid() == unsafe { libc::geteuid() }
        && metadata.mode() & 0o077 == 0
}
