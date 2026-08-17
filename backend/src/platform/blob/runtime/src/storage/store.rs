//! Owner-fenced encrypted Blob reads and atomic writes.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use makosh_blob_protocol::{BlobAccessFenceV1, BlobCustodyScopeV1, BlobRangeV1, BlobRefV1};

use crate::lease::{BlobKeyLeaseV1, BlobLeaseError};
use sha2::{Digest, Sha256};

use super::{format, root};

const MAX_BLOB_BYTES: u64 = 4 * 1024 * 1024 * 1024;

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
        remove_staged_content(&content_root)?;
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
        let mut file = File::open(target).map_err(|_| {
            developer_storage_stage("read_file");
            BlobStorageError::Filesystem
        })?;
        let mut magic = [0_u8; 8];
        file.read_exact(&mut magic)
            .map_err(|_| BlobStorageError::MalformedCiphertext)?;
        if format::is_legacy_magic(&magic) {
            file.seek(SeekFrom::Start(0))
                .map_err(|_| BlobStorageError::Filesystem)?;
            let mut encrypted = Vec::new();
            file.read_to_end(&mut encrypted)
                .map_err(|_| BlobStorageError::Filesystem)?;
            let plaintext =
                format::decrypt(reference, custody, lease.key_revision(), key, &encrypted)
                    .inspect_err(|_| developer_storage_stage("decrypt"))?;
            if u64::try_from(plaintext.len()) != Ok(reference.declared_size()) {
                return Err(BlobStorageError::MalformedCiphertext);
            }
            let start =
                usize::try_from(range.start()).map_err(|_| BlobStorageError::InvalidRange)?;
            let end = usize::try_from(range.end_exclusive())
                .map_err(|_| BlobStorageError::InvalidRange)?;
            return plaintext
                .get(start..end)
                .map(ToOwned::to_owned)
                .ok_or(BlobStorageError::InvalidRange);
        }
        if !format::is_chunked_magic(&magic) {
            return Err(BlobStorageError::MalformedCiphertext);
        }
        self.read_chunked_range(
            &mut file,
            reference,
            custody,
            lease.key_revision(),
            key,
            range,
        )
    }

    pub fn write_chunk(
        &self,
        reference: &BlobRefV1,
        fence: &BlobAccessFenceV1,
        custody: &BlobCustodyScopeV1,
        lease: &BlobKeyLeaseV1,
        offset: u64,
        plaintext: &[u8],
        complete: bool,
        expected_plaintext_sha256: &[u8; 32],
        now_unix_ms: u64,
    ) -> Result<bool, BlobStorageError> {
        self.validate_chunk(reference, fence, offset, plaintext, complete, now_unix_ms)?;
        if custody.owner_id() != reference.owner_id() {
            return Err(BlobStorageError::FenceMismatch);
        }
        let key = lease
            .key_for(reference, fence, custody, now_unix_ms)
            .map_err(BlobStorageError::Lease)?;
        let target = root::blob_path(&self.content_root, reference);
        reject_existing_path(&target)?;
        let staged = target.with_extension("staged");
        let mut file = if offset == 0 {
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW)
                .open(&staged)
                .map_err(|_| BlobStorageError::AlreadyExists)?;
            file.write_all(&format::chunked_header(lease.key_revision()))
                .map_err(|_| BlobStorageError::Filesystem)?;
            file
        } else {
            root::validate_private_regular_file(&staged)?;
            OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(libc::O_NOFOLLOW)
                .open(&staged)
                .map_err(|_| BlobStorageError::Filesystem)?
        };
        self.validate_staged_position(&mut file, lease.key_revision(), offset)?;
        let encrypted = format::encrypt_chunk(
            reference,
            custody,
            lease.key_revision(),
            key,
            offset,
            plaintext,
        )?;
        file.write_all(&encrypted)
            .and_then(|_| file.sync_all())
            .map_err(|_| BlobStorageError::Filesystem)?;
        if !complete {
            return Ok(false);
        }
        self.verify_chunked_receipt(
            &mut file,
            reference,
            custody,
            lease.key_revision(),
            key,
            expected_plaintext_sha256,
        )?;
        root::validate_private_regular_file(&staged)?;
        reject_existing_path(&target)?;
        fs::rename(&staged, &target).map_err(|_| BlobStorageError::Filesystem)?;
        self.sync_content_root()?;
        Ok(true)
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
        let staged = target.with_extension("staged");
        for path in [&target, &staged] {
            match fs::symlink_metadata(path) {
                Ok(_) => {
                    root::validate_private_regular_file(path)?;
                    fs::remove_file(path).map_err(|_| BlobStorageError::Filesystem)?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(BlobStorageError::Filesystem),
            }
        }
        self.sync_content_root()
    }

    fn validate_chunk(
        &self,
        reference: &BlobRefV1,
        fence: &BlobAccessFenceV1,
        offset: u64,
        plaintext: &[u8],
        complete: bool,
        now_unix_ms: u64,
    ) -> Result<(), BlobStorageError> {
        let chunk_bytes = u64::try_from(format::CHUNK_PLAINTEXT_BYTES)
            .map_err(|_| BlobStorageError::InvalidWrite)?;
        let plaintext_bytes =
            u64::try_from(plaintext.len()).map_err(|_| BlobStorageError::InvalidWrite)?;
        let remaining = reference
            .declared_size()
            .checked_sub(offset)
            .ok_or(BlobStorageError::InvalidWrite)?;
        let expected = remaining.min(chunk_bytes);
        if reference.is_expired_at(now_unix_ms)
            || fence.owner_id() != reference.owner_id()
            || reference.declared_size() > self.maximum_blob_bytes
            || offset % chunk_bytes != 0
            || plaintext_bytes != expected
            || expected == 0
            || complete != (offset + plaintext_bytes == reference.declared_size())
        {
            return Err(BlobStorageError::InvalidWrite);
        }
        Ok(())
    }

    fn validate_staged_position(
        &self,
        file: &mut File,
        key_revision: u64,
        offset: u64,
    ) -> Result<(), BlobStorageError> {
        let mut header = [0_u8; format::CHUNKED_HEADER_BYTES];
        file.seek(SeekFrom::Start(0))
            .and_then(|_| file.read_exact(&mut header))
            .map_err(|_| BlobStorageError::MalformedCiphertext)?;
        if format::chunked_key_revision(&header)? != key_revision {
            return Err(BlobStorageError::AuthenticationFailed);
        }
        let full_record =
            u64::try_from(format::encrypted_chunk_bytes(format::CHUNK_PLAINTEXT_BYTES))
                .map_err(|_| BlobStorageError::InvalidWrite)?;
        let chunk_bytes = u64::try_from(format::CHUNK_PLAINTEXT_BYTES)
            .map_err(|_| BlobStorageError::InvalidWrite)?;
        let expected_physical = u64::try_from(format::CHUNKED_HEADER_BYTES)
            .map_err(|_| BlobStorageError::InvalidWrite)?
            + (offset / chunk_bytes) * full_record;
        if file
            .metadata()
            .map_err(|_| BlobStorageError::Filesystem)?
            .len()
            != expected_physical
        {
            return Err(BlobStorageError::InvalidWrite);
        }
        file.seek(SeekFrom::End(0))
            .map(|_| ())
            .map_err(|_| BlobStorageError::Filesystem)
    }

    fn verify_chunked_receipt(
        &self,
        file: &mut File,
        reference: &BlobRefV1,
        custody: &BlobCustodyScopeV1,
        key_revision: u64,
        key: &[u8; 32],
        expected: &[u8; 32],
    ) -> Result<(), BlobStorageError> {
        file.seek(SeekFrom::Start(
            u64::try_from(format::CHUNKED_HEADER_BYTES)
                .map_err(|_| BlobStorageError::MalformedCiphertext)?,
        ))
        .map_err(|_| BlobStorageError::Filesystem)?;
        let mut hasher = Sha256::new();
        let mut offset = 0_u64;
        while offset < reference.declared_size() {
            let remaining = reference.declared_size() - offset;
            let plaintext_len = usize::try_from(
                remaining.min(
                    u64::try_from(format::CHUNK_PLAINTEXT_BYTES)
                        .map_err(|_| BlobStorageError::MalformedCiphertext)?,
                ),
            )
            .map_err(|_| BlobStorageError::MalformedCiphertext)?;
            let mut encrypted = vec![0_u8; format::encrypted_chunk_bytes(plaintext_len)];
            file.read_exact(&mut encrypted)
                .map_err(|_| BlobStorageError::MalformedCiphertext)?;
            let plaintext = format::decrypt_chunk(
                reference,
                custody,
                key_revision,
                key,
                offset,
                plaintext_len,
                &encrypted,
            )?;
            hasher.update(&plaintext);
            offset +=
                u64::try_from(plaintext_len).map_err(|_| BlobStorageError::MalformedCiphertext)?;
        }
        let mut trailing = [0_u8; 1];
        if file
            .read(&mut trailing)
            .map_err(|_| BlobStorageError::Filesystem)?
            != 0
            || hasher.finalize().as_slice() != expected
        {
            return Err(BlobStorageError::AuthenticationFailed);
        }
        Ok(())
    }

    fn read_chunked_range(
        &self,
        file: &mut File,
        reference: &BlobRefV1,
        custody: &BlobCustodyScopeV1,
        key_revision: u64,
        key: &[u8; 32],
        range: BlobRangeV1,
    ) -> Result<Vec<u8>, BlobStorageError> {
        let mut revision = [0_u8; 8];
        file.read_exact(&mut revision)
            .map_err(|_| BlobStorageError::MalformedCiphertext)?;
        if u64::from_be_bytes(revision) != key_revision {
            return Err(BlobStorageError::AuthenticationFailed);
        }
        let chunk_bytes = u64::try_from(format::CHUNK_PLAINTEXT_BYTES)
            .map_err(|_| BlobStorageError::InvalidRange)?;
        let first = range.start() / chunk_bytes;
        let last = (range.end_exclusive() - 1) / chunk_bytes;
        let full_record =
            u64::try_from(format::encrypted_chunk_bytes(format::CHUNK_PLAINTEXT_BYTES))
                .map_err(|_| BlobStorageError::InvalidRange)?;
        let mut output = Vec::with_capacity(
            usize::try_from(range.end_exclusive() - range.start())
                .map_err(|_| BlobStorageError::InvalidRange)?,
        );
        for chunk_index in first..=last {
            let offset = chunk_index * chunk_bytes;
            let plaintext_len =
                usize::try_from((reference.declared_size() - offset).min(chunk_bytes))
                    .map_err(|_| BlobStorageError::InvalidRange)?;
            let physical = u64::try_from(format::CHUNKED_HEADER_BYTES)
                .map_err(|_| BlobStorageError::InvalidRange)?
                + chunk_index * full_record;
            file.seek(SeekFrom::Start(physical))
                .map_err(|_| BlobStorageError::Filesystem)?;
            let mut encrypted = vec![0_u8; format::encrypted_chunk_bytes(plaintext_len)];
            file.read_exact(&mut encrypted)
                .map_err(|_| BlobStorageError::MalformedCiphertext)?;
            let plaintext = format::decrypt_chunk(
                reference,
                custody,
                key_revision,
                key,
                offset,
                plaintext_len,
                &encrypted,
            )?;
            let take_start = usize::try_from(range.start().saturating_sub(offset))
                .map_err(|_| BlobStorageError::InvalidRange)?;
            let take_end =
                usize::try_from(range.end_exclusive().min(offset + chunk_bytes) - offset)
                    .map_err(|_| BlobStorageError::InvalidRange)?;
            output.extend_from_slice(
                plaintext
                    .get(take_start..take_end)
                    .ok_or(BlobStorageError::InvalidRange)?,
            );
        }
        Ok(output)
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
        if !format::is_supported_magic(&magic) {
            return Err(BlobStorageError::MalformedCiphertext);
        }
    }
    Ok(())
}

fn remove_staged_content(content_root: &Path) -> Result<(), BlobStorageError> {
    for entry in fs::read_dir(content_root).map_err(|_| BlobStorageError::Filesystem)? {
        let path = entry.map_err(|_| BlobStorageError::Filesystem)?.path();
        let Some(filename) = path.file_name().and_then(|value| value.to_str()) else {
            return Err(BlobStorageError::UnsafePath);
        };
        if filename.len() == 39
            && filename.ends_with(".staged")
            && filename.as_bytes().get(..32).is_some_and(|prefix| {
                prefix
                    .iter()
                    .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
            })
        {
            root::validate_private_regular_file(&path)?;
            fs::remove_file(path).map_err(|_| BlobStorageError::Filesystem)?;
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
