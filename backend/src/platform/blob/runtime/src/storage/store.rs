//! Owner-fenced encrypted Blob reads and atomic writes.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use makosh_blob_protocol::{BlobAccessFenceV1, BlobCustodyScopeV1, BlobRangeV1, BlobRefV1};

use crate::lease::{BlobKeyLeaseV1, BlobLeaseError};

use super::{format, root};

const MAX_BLOB_BYTES: u64 = 64 * 1024 * 1024;

pub struct EncryptedBlobStore {
    content_root: PathBuf,
    maximum_blob_bytes: u64,
}

impl EncryptedBlobStore {
    pub fn open(data_dir: &Path, maximum_blob_bytes: u64) -> Result<Self, BlobStorageError> {
        if maximum_blob_bytes == 0 || maximum_blob_bytes > MAX_BLOB_BYTES {
            return Err(BlobStorageError::InvalidQuota);
        }
        let content_root = root::prepare_content_root(data_dir)?;
        validate_existing_content(&content_root)?;
        Ok(Self {
            content_root,
            maximum_blob_bytes,
        })
    }

    pub fn write_new(
        &self,
        reference: &BlobRefV1,
        fence: &BlobAccessFenceV1,
        custody: &BlobCustodyScopeV1,
        lease: &BlobKeyLeaseV1,
        plaintext: &[u8],
        now_unix_ms: u64,
    ) -> Result<(), BlobStorageError> {
        self.validate_write(reference, fence, plaintext, now_unix_ms)?;
        if custody.owner_id() != reference.owner_id() {
            return Err(BlobStorageError::FenceMismatch);
        }
        let key = lease
            .key_for(reference, fence, custody, now_unix_ms)
            .map_err(BlobStorageError::Lease)?;
        let target = root::blob_path(&self.content_root, reference);
        reject_existing_path(&target)?;
        let encrypted = format::encrypt(reference, custody, lease.key_revision(), key, plaintext)?;
        self.write_staged(&target, &encrypted)
    }

    pub fn read_range(
        &self,
        reference: &BlobRefV1,
        fence: &BlobAccessFenceV1,
        custody: &BlobCustodyScopeV1,
        lease: &BlobKeyLeaseV1,
        range: BlobRangeV1,
        now_unix_ms: u64,
    ) -> Result<Vec<u8>, BlobStorageError> {
        if reference.is_expired_at(now_unix_ms) {
            return Err(BlobStorageError::Expired);
        }
        if fence.owner_id() != reference.owner_id() || custody.owner_id() != reference.owner_id() {
            return Err(BlobStorageError::FenceMismatch);
        }
        if range.end_exclusive() > reference.declared_size() {
            return Err(BlobStorageError::InvalidRange);
        }
        let key = lease
            .key_for(reference, fence, custody, now_unix_ms)
            .map_err(BlobStorageError::Lease)?;
        let target = root::blob_path(&self.content_root, reference);
        root::validate_private_regular_file(&target)
            .inspect_err(|_| developer_storage_stage("validate_file"))?;
        let encrypted = fs::read(target).map_err(|_| {
            developer_storage_stage("read_file");
            BlobStorageError::Filesystem
        })?;
        let plaintext = format::decrypt(reference, custody, lease.key_revision(), key, &encrypted)
            .inspect_err(|_| developer_storage_stage("decrypt"))?;
        if u64::try_from(plaintext.len()) != Ok(reference.declared_size()) {
            return Err(BlobStorageError::MalformedCiphertext);
        }
        let start = usize::try_from(range.start()).map_err(|_| BlobStorageError::InvalidRange)?;
        let end =
            usize::try_from(range.end_exclusive()).map_err(|_| BlobStorageError::InvalidRange)?;
        plaintext
            .get(start..end)
            .map(ToOwned::to_owned)
            .ok_or(BlobStorageError::InvalidRange)
    }

    /// Removes one owner-authorized Blob and syncs the containing private directory.
    pub fn delete(
        &self,
        reference: &BlobRefV1,
        fence: &BlobAccessFenceV1,
        custody: &BlobCustodyScopeV1,
        lease: &BlobKeyLeaseV1,
        now_unix_ms: u64,
    ) -> Result<(), BlobStorageError> {
        if fence.owner_id() != reference.owner_id() || custody.owner_id() != reference.owner_id() {
            return Err(BlobStorageError::FenceMismatch);
        }
        lease
            .key_for(reference, fence, custody, now_unix_ms)
            .map_err(BlobStorageError::Lease)?;
        let target = root::blob_path(&self.content_root, reference);
        root::validate_private_regular_file(&target)?;
        fs::remove_file(target).map_err(|_| BlobStorageError::Filesystem)?;
        self.sync_content_root()
    }

    pub(crate) fn exists(&self, reference: &BlobRefV1) -> Result<bool, BlobStorageError> {
        let target = root::blob_path(&self.content_root, reference);
        match fs::symlink_metadata(&target) {
            Ok(_) => {
                root::validate_private_regular_file(&target)?;
                Ok(true)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(_) => Err(BlobStorageError::Filesystem),
        }
    }

    pub(crate) fn discard_uncommitted(
        &self,
        reference: &BlobRefV1,
    ) -> Result<(), BlobStorageError> {
        let target = root::blob_path(&self.content_root, reference);
        match fs::symlink_metadata(&target) {
            Ok(_) => {
                root::validate_private_regular_file(&target)?;
                fs::remove_file(target).map_err(|_| BlobStorageError::Filesystem)?;
                self.sync_content_root()
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(BlobStorageError::Filesystem),
        }
    }

    fn validate_write(
        &self,
        reference: &BlobRefV1,
        fence: &BlobAccessFenceV1,
        plaintext: &[u8],
        now_unix_ms: u64,
    ) -> Result<(), BlobStorageError> {
        if reference.is_expired_at(now_unix_ms) {
            return Err(BlobStorageError::Expired);
        }
        if fence.owner_id() != reference.owner_id()
            || reference.declared_size() > self.maximum_blob_bytes
            || u64::try_from(plaintext.len()) != Ok(reference.declared_size())
        {
            return Err(BlobStorageError::InvalidWrite);
        }
        Ok(())
    }

    fn write_staged(&self, target: &Path, encrypted: &[u8]) -> Result<(), BlobStorageError> {
        let staged = target.with_extension("staged");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&staged)
            .map_err(|_| BlobStorageError::AlreadyExists)?;
        let result = (|| {
            file.write_all(encrypted)
                .map_err(|_| BlobStorageError::Filesystem)?;
            file.sync_all().map_err(|_| BlobStorageError::Filesystem)?;
            root::validate_private_regular_file(&staged)?;
            reject_existing_path(target)?;
            fs::rename(&staged, target).map_err(|_| BlobStorageError::Filesystem)?;
            self.sync_content_root()
        })();
        if result.is_err() {
            let _ = fs::remove_file(&staged);
        }
        result
    }

    fn sync_content_root(&self) -> Result<(), BlobStorageError> {
        File::open(&self.content_root)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| BlobStorageError::Filesystem)
    }
}

fn developer_storage_stage(stage: &str) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_blob_storage_read_denied stage={stage}");
    }
}

fn validate_existing_content(content_root: &Path) -> Result<(), BlobStorageError> {
    for entry in fs::read_dir(content_root).map_err(|_| BlobStorageError::Filesystem)? {
        let path = entry.map_err(|_| BlobStorageError::Filesystem)?.path();
        if !valid_blob_filename(&path) {
            return Err(BlobStorageError::UnsafePath);
        }
        root::validate_private_regular_file(&path)?;
        let mut magic = [0_u8; 8];
        File::open(path)
            .and_then(|mut file| file.read_exact(&mut magic))
            .map_err(|_| BlobStorageError::MalformedCiphertext)?;
        if !format::is_current_magic(&magic) {
            return Err(BlobStorageError::MalformedCiphertext);
        }
    }
    Ok(())
}

fn valid_blob_filename(path: &Path) -> bool {
    let Some(filename) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    filename.len() == 37
        && filename.ends_with(".blob")
        && filename.as_bytes().get(..32).is_some_and(|prefix| {
            prefix
                .iter()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        })
}

fn reject_existing_path(path: &Path) -> Result<(), BlobStorageError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(BlobStorageError::AlreadyExists),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(BlobStorageError::Filesystem),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobStorageError {
    AlreadyExists,
    AuthenticationFailed,
    Crypto,
    Expired,
    FenceMismatch,
    Filesystem,
    InvalidQuota,
    InvalidRange,
    InvalidWrite,
    Lease(BlobLeaseError),
    MalformedCiphertext,
    Randomness,
    UnsafePath,
}
