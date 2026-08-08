//! Streaming authenticated encryption for the recovery-media payload.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use super::format::RecoveryMediaEntryV1;

const ENCRYPTED_FILE_MAGIC: &[u8; 8] = b"HRENCV01";
const CHUNK_BYTES: usize = 1024 * 1024;
const TAG_BYTES: u64 = 16;
const HEADER_BYTES: u64 = 32;

pub(crate) struct RecoveryMediaEncryptionKey([u8; 32]);

pub(crate) struct DecryptedRecoveryPayload {
    root: PathBuf,
}

impl RecoveryMediaEncryptionKey {
    pub(crate) fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Drop for RecoveryMediaEncryptionKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl DecryptedRecoveryPayload {
    pub(super) fn create(
        encrypted_root: &Path,
        entries: &[RecoveryMediaEntryV1],
        workspace_parent: &Path,
        key: &RecoveryMediaEncryptionKey,
    ) -> Result<Self, String> {
        ensure_private_directory(workspace_parent, "recovery decryption workspace")?;
        let root = create_staging(workspace_parent)?;
        let result = decrypt_entries(encrypted_root, &root, entries, key);
        if result.is_err() {
            let _ = std::fs::remove_dir_all(&root);
        }
        result.map(|()| Self { root })
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for DecryptedRecoveryPayload {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

pub(super) fn encrypt_payload(
    plaintext_root: &Path,
    encrypted_root: &Path,
    key: &RecoveryMediaEncryptionKey,
) -> Result<(), String> {
    ensure_private_directory(plaintext_root, "recovery plaintext payload")?;
    create_private_directory(encrypted_root)?;
    let result = encrypt_directory(plaintext_root, plaintext_root, encrypted_root, key);
    if result.is_err() {
        let _ = std::fs::remove_dir_all(encrypted_root);
    }
    result
}

fn encrypt_directory(
    root: &Path,
    source: &Path,
    destination_root: &Path,
    key: &RecoveryMediaEncryptionKey,
) -> Result<(), String> {
    let entries = read_sorted_paths(source)?;
    if source != root && entries.is_empty() {
        return Err("recovery plaintext payload contains an empty directory".to_owned());
    }
    for path in entries {
        let relative = relative_path(root, &path)?;
        let destination = destination_root.join(&relative);
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|_| "recovery plaintext entry is unavailable".to_owned())?;
        if metadata.file_type().is_symlink() {
            return Err("recovery plaintext payload must not contain symlinks".to_owned());
        }
        if metadata.is_dir() {
            create_private_directory(&destination)?;
            encrypt_directory(root, &path, destination_root, key)?;
        } else if metadata.is_file() {
            encrypt_file(&path, &destination, &relative, key)?;
        } else {
            return Err("recovery plaintext payload must contain regular files only".to_owned());
        }
    }
    sync_directory(destination_root.join(relative_path(root, source)?))
}

fn encrypt_file(
    source: &Path,
    destination: &Path,
    relative: &str,
    key: &RecoveryMediaEncryptionKey,
) -> Result<(), String> {
    restrict_plaintext_file(source)?;
    let (mut input, metadata) = open_stable_private_file(source, "recovery plaintext file")?;
    let mut prefix = [0_u8; 16];
    getrandom::fill(&mut prefix)
        .map_err(|_| "recovery media encryption entropy is unavailable".to_owned())?;
    let mut output = create_private_file(destination)?;
    output
        .write_all(ENCRYPTED_FILE_MAGIC)
        .and_then(|()| output.write_all(&metadata.len().to_be_bytes()))
        .and_then(|()| output.write_all(&prefix))
        .map_err(|_| encryption_failed())?;
    encrypt_chunks(
        &mut input,
        &mut output,
        metadata.len(),
        prefix,
        relative,
        key,
    )?;
    verify_stable_path(source, &metadata, &input, "recovery plaintext file")?;
    output.sync_all().map_err(|_| encryption_failed())
}

fn restrict_plaintext_file(path: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "recovery plaintext file is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.uid() != current_uid() {
        return Err("recovery plaintext file is invalid".to_owned());
    }
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|_| "recovery plaintext permissions cannot be restricted".to_owned())
}

fn encrypt_chunks(
    input: &mut File,
    output: &mut File,
    plaintext_bytes: u64,
    prefix: [u8; 16],
    relative: &str,
    key: &RecoveryMediaEncryptionKey,
) -> Result<(), String> {
    let cipher = XChaCha20Poly1305::new_from_slice(key.bytes()).map_err(|_| encryption_failed())?;
    let mut remaining = plaintext_bytes;
    for index in 0..chunk_count(plaintext_bytes) {
        let length =
            usize::try_from(remaining.min(CHUNK_BYTES as u64)).map_err(|_| encryption_failed())?;
        let mut plaintext = vec![0_u8; length];
        input
            .read_exact(&mut plaintext)
            .map_err(|_| encryption_failed())?;
        let nonce_bytes = nonce(prefix, index);
        let nonce = XNonce::try_from(nonce_bytes.as_slice()).map_err(|_| encryption_failed())?;
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: &plaintext,
                    aad: &associated_data(plaintext_bytes, index, relative)?,
                },
            )
            .map_err(|_| encryption_failed())?;
        plaintext.zeroize();
        output
            .write_all(&ciphertext)
            .map_err(|_| encryption_failed())?;
        remaining = remaining.saturating_sub(length as u64);
    }
    Ok(())
}

fn decrypt_entries(
    encrypted_root: &Path,
    destination_root: &Path,
    entries: &[RecoveryMediaEntryV1],
    key: &RecoveryMediaEncryptionKey,
) -> Result<(), String> {
    for entry in entries {
        let destination = destination_root.join(entry.path());
        create_private_parents(destination_root, &destination)?;
        decrypt_file(&encrypted_root.join(entry.path()), &destination, entry, key)?;
    }
    sync_tree(destination_root)
}

fn decrypt_file(
    source: &Path,
    destination: &Path,
    entry: &RecoveryMediaEntryV1,
    key: &RecoveryMediaEncryptionKey,
) -> Result<(), String> {
    let (mut input, metadata) = open_stable_private_file(source, "recovery encrypted file")?;
    if metadata.len() != entry.size_bytes() {
        return Err(decryption_failed());
    }
    let mut header = [0_u8; HEADER_BYTES as usize];
    input
        .read_exact(&mut header)
        .map_err(|_| decryption_failed())?;
    if &header[..8] != ENCRYPTED_FILE_MAGIC {
        return Err(decryption_failed());
    }
    let plaintext_bytes =
        u64::from_be_bytes(header[8..16].try_into().map_err(|_| decryption_failed())?);
    let prefix: [u8; 16] = header[16..32].try_into().map_err(|_| decryption_failed())?;
    if metadata.len() != encrypted_size(plaintext_bytes)? {
        return Err(decryption_failed());
    }
    let mut digest = Sha256::new();
    digest.update(header);
    let mut output = create_private_file(destination)?;
    decrypt_chunks(
        &mut input,
        &mut output,
        plaintext_bytes,
        prefix,
        entry.path(),
        key,
        &mut digest,
    )?;
    verify_stable_path(source, &metadata, &input, "recovery encrypted file")?;
    if <[u8; 32]>::from(digest.finalize()) != *entry.sha256() {
        return Err(decryption_failed());
    }
    output.sync_all().map_err(|_| decryption_failed())
}

#[allow(clippy::too_many_arguments)]
fn decrypt_chunks(
    input: &mut File,
    output: &mut File,
    plaintext_bytes: u64,
    prefix: [u8; 16],
    relative: &str,
    key: &RecoveryMediaEncryptionKey,
    digest: &mut Sha256,
) -> Result<(), String> {
    let cipher = XChaCha20Poly1305::new_from_slice(key.bytes()).map_err(|_| decryption_failed())?;
    let mut remaining = plaintext_bytes;
    for index in 0..chunk_count(plaintext_bytes) {
        let plaintext_length = remaining.min(CHUNK_BYTES as u64);
        let cipher_length =
            usize::try_from(plaintext_length + TAG_BYTES).map_err(|_| decryption_failed())?;
        let mut ciphertext = vec![0_u8; cipher_length];
        input
            .read_exact(&mut ciphertext)
            .map_err(|_| decryption_failed())?;
        digest.update(&ciphertext);
        let nonce_bytes = nonce(prefix, index);
        let nonce = XNonce::try_from(nonce_bytes.as_slice()).map_err(|_| decryption_failed())?;
        let mut plaintext = cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: &ciphertext,
                    aad: &associated_data(plaintext_bytes, index, relative)?,
                },
            )
            .map_err(|_| decryption_failed())?;
        output
            .write_all(&plaintext)
            .map_err(|_| decryption_failed())?;
        plaintext.zeroize();
        remaining = remaining.saturating_sub(plaintext_length);
    }
    Ok(())
}

fn associated_data(plaintext_bytes: u64, index: u64, relative: &str) -> Result<Vec<u8>, String> {
    let path = relative.as_bytes();
    let path_length = u16::try_from(path.len()).map_err(|_| encryption_failed())?;
    let mut bytes = Vec::with_capacity(26 + path.len());
    bytes.extend_from_slice(ENCRYPTED_FILE_MAGIC);
    bytes.extend_from_slice(&plaintext_bytes.to_be_bytes());
    bytes.extend_from_slice(&index.to_be_bytes());
    bytes.extend_from_slice(&path_length.to_be_bytes());
    bytes.extend_from_slice(path);
    Ok(bytes)
}

fn nonce(prefix: [u8; 16], index: u64) -> [u8; 24] {
    let mut nonce = [0_u8; 24];
    nonce[..16].copy_from_slice(&prefix);
    nonce[16..].copy_from_slice(&index.to_be_bytes());
    nonce
}

fn chunk_count(plaintext_bytes: u64) -> u64 {
    plaintext_bytes.div_ceil(CHUNK_BYTES as u64).max(1)
}

fn encrypted_size(plaintext_bytes: u64) -> Result<u64, String> {
    HEADER_BYTES
        .checked_add(plaintext_bytes)
        .and_then(|size| size.checked_add(chunk_count(plaintext_bytes).checked_mul(TAG_BYTES)?))
        .ok_or_else(decryption_failed)
}

fn create_private_parents(root: &Path, destination: &Path) -> Result<(), String> {
    let parent = destination.parent().ok_or_else(decryption_failed)?;
    let relative = parent.strip_prefix(root).map_err(|_| decryption_failed())?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component);
        if !current.exists() {
            create_private_directory(&current)?;
        }
    }
    Ok(())
}

fn sync_tree(directory: &Path) -> Result<(), String> {
    for entry in read_sorted_paths(directory)? {
        if std::fs::symlink_metadata(&entry)
            .map_err(|_| decryption_failed())?
            .is_dir()
        {
            sync_tree(&entry)?;
        }
    }
    sync_directory(directory)
}

fn read_sorted_paths(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = std::fs::read_dir(directory)
        .map_err(|_| "recovery media directory cannot be read".to_owned())?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|_| encryption_failed())
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    Ok(paths)
}

fn relative_path(root: &Path, path: &Path) -> Result<String, String> {
    if root == path {
        return Ok(String::new());
    }
    path.strip_prefix(root)
        .ok()
        .and_then(Path::to_str)
        .map(str::to_owned)
        .ok_or_else(encryption_failed)
}

fn open_stable_private_file(path: &Path, label: &str) -> Result<(File, std::fs::Metadata), String> {
    let before = std::fs::symlink_metadata(path).map_err(|_| format!("{label} is unavailable"))?;
    if before.file_type().is_symlink()
        || !before.is_file()
        || before.uid() != current_uid()
        || before.permissions().mode() & 0o077 != 0
    {
        return Err(format!("{label} is invalid"));
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| format!("{label} cannot be opened"))?;
    let opened = file
        .metadata()
        .map_err(|_| format!("{label} cannot be inspected"))?;
    if !same_file(&before, &opened) {
        return Err(format!("{label} changed while opening"));
    }
    Ok((file, opened))
}

fn verify_stable_path(
    path: &Path,
    opened: &std::fs::Metadata,
    file: &File,
    label: &str,
) -> Result<(), String> {
    let after = file
        .metadata()
        .map_err(|_| format!("{label} cannot be inspected"))?;
    let path_after =
        std::fs::symlink_metadata(path).map_err(|_| format!("{label} is unavailable"))?;
    if !same_file(opened, &after) || !same_file(opened, &path_after) {
        return Err(format!("{label} changed while reading"));
    }
    Ok(())
}

fn create_private_file(path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| encryption_failed())
}

fn create_private_directory(path: &Path) -> Result<(), String> {
    std::fs::create_dir(path).map_err(|_| encryption_failed())?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .map_err(|_| encryption_failed())
}

fn ensure_private_directory(path: &Path, label: &str) -> Result<(), String> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| format!("{label} is unavailable"))?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != current_uid()
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(format!("{label} is invalid"));
    }
    Ok(())
}

fn create_staging(parent: &Path) -> Result<PathBuf, String> {
    for _ in 0..16 {
        let mut suffix = [0_u8; 8];
        getrandom::fill(&mut suffix).map_err(|_| decryption_failed())?;
        let name = suffix
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let path = parent.join(format!(".makosh-recovery-decrypt-{name}"));
        match std::fs::create_dir(&path) {
            Ok(()) => {
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
                    .map_err(|_| decryption_failed())?;
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err(decryption_failed()),
        }
    }
    Err(decryption_failed())
}

fn sync_directory(path: impl AsRef<Path>) -> Result<(), String> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| encryption_failed())
}

fn same_file(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}

fn encryption_failed() -> String {
    "recovery media encryption failed".to_owned()
}

fn decryption_failed() -> String {
    "recovery media decryption failed".to_owned()
}

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and only reads process credentials.
    unsafe { libc::geteuid() }
}
