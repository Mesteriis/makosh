//! Canonical private root and internal filename handling.

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use makosh_blob_protocol::BlobRefV1;

use super::store::BlobStorageError;

pub(super) fn prepare_content_root(path: &Path) -> Result<PathBuf, BlobStorageError> {
    prepare_private_child(path, "content")
}

pub(crate) fn prepare_metadata_root(path: &Path) -> Result<PathBuf, BlobStorageError> {
    prepare_private_child(path, "metadata")
}

pub(crate) fn prepare_release_root(path: &Path) -> Result<PathBuf, BlobStorageError> {
    prepare_private_child(path, "custody-releases-v1")
}

fn prepare_private_child(path: &Path, child: &str) -> Result<PathBuf, BlobStorageError> {
    fs::create_dir_all(path).map_err(|_| BlobStorageError::Filesystem)?;
    validate_private_directory(path)?;
    let root = fs::canonicalize(path).map_err(|_| BlobStorageError::Filesystem)?;
    let child = root.join(child);
    fs::create_dir(&child)
        .or_else(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(error)
            }
        })
        .map_err(|_| BlobStorageError::Filesystem)?;
    fs::set_permissions(&child, fs::Permissions::from_mode(0o700))
        .map_err(|_| BlobStorageError::Filesystem)?;
    validate_private_directory(&child)?;
    fs::canonicalize(child).map_err(|_| BlobStorageError::Filesystem)
}

pub(super) fn blob_path(content_root: &Path, reference: &BlobRefV1) -> PathBuf {
    let mut value = String::with_capacity(37);
    for byte in reference.reference_id() {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    value.push_str(".blob");
    content_root.join(value)
}

pub(crate) fn validate_private_directory(path: &Path) -> Result<(), BlobStorageError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| BlobStorageError::Filesystem)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(BlobStorageError::UnsafePath);
    }
    Ok(())
}

pub(crate) fn validate_private_regular_file(path: &Path) -> Result<(), BlobStorageError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| BlobStorageError::Filesystem)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(BlobStorageError::UnsafePath);
    }
    Ok(())
}
