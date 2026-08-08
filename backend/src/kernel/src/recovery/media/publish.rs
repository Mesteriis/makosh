//! Private staging and atomic publication of signed recovery media.

use std::fs::{File, OpenOptions};
use std::io::Read;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::encryption::{RecoveryMediaEncryptionKey, encrypt_payload};
use super::format::{
    RecoveryMediaComponentV1, RecoveryMediaEntryV1, RecoveryMediaInventoryV1,
    RecoveryMediaManifestV1, RecoveryMediaProvenanceV1,
};
use super::layout::{MANIFEST_FILE, PAYLOAD_DIRECTORY, SIGNATURE_FILE};
use super::signature::SignedRecoveryMediaManifestV1;

pub(crate) trait RecoveryMediaSigner {
    fn key_id(&self) -> &str;
    fn sign(&self, manifest: &[u8]) -> Result<[u8; 64], String>;
}

pub(crate) struct RecoveryMediaPublisher {
    destination: PathBuf,
    staging: Option<PathBuf>,
    plaintext: PathBuf,
}

impl RecoveryMediaPublisher {
    pub(crate) fn create(destination: &Path) -> Result<Self, String> {
        if !destination.is_absolute() {
            return Err("recovery media destination must be absolute".to_owned());
        }
        ensure_absent(destination, "recovery media destination")?;
        let parent = destination
            .parent()
            .ok_or_else(|| "recovery media destination has no parent".to_owned())?;
        ensure_private_directory(parent, "recovery media parent")?;
        let staging = create_staging(parent)?;
        let plaintext = staging.join(".plaintext");
        if let Err(error) = create_private_directory(&plaintext) {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(error);
        }
        Ok(Self {
            destination: destination.to_owned(),
            staging: Some(staging),
            plaintext,
        })
    }

    pub(crate) fn payload_root(&self) -> &Path {
        &self.plaintext
    }

    pub(crate) fn publish(
        mut self,
        provenance: RecoveryMediaProvenanceV1,
        inventory: RecoveryMediaInventoryV1,
        signer: &impl RecoveryMediaSigner,
        encryption_key: &RecoveryMediaEncryptionKey,
    ) -> Result<PathBuf, String> {
        let staging = self
            .staging
            .as_ref()
            .ok_or_else(|| "recovery media staging is unavailable".to_owned())?;
        let payload = staging.join(PAYLOAD_DIRECTORY);
        encrypt_payload(&self.plaintext, &payload, encryption_key)?;
        std::fs::remove_dir_all(&self.plaintext)
            .map_err(|_| "recovery media plaintext cleanup failed".to_owned())?;
        let raw_manifest = RecoveryMediaManifestV1::encode(
            provenance.clone(),
            inventory,
            canonical_payload_entries(&payload)?,
        )?;
        let signature = signer.sign(&raw_manifest)?;
        let signed = SignedRecoveryMediaManifestV1::new(
            signer.key_id().to_owned(),
            raw_manifest,
            signature,
        )?;
        write_private_file(&staging.join(MANIFEST_FILE), signed.raw_manifest_bytes())?;
        write_private_file(&staging.join(SIGNATURE_FILE), &signed.signature_metadata())?;
        let rechecked_manifest = RecoveryMediaManifestV1::encode(
            provenance,
            inventory,
            canonical_payload_entries(&payload)?,
        )?;
        if rechecked_manifest != signed.raw_manifest_bytes() {
            return Err("recovery media payload changed before publication".to_owned());
        }
        sync_directory(&payload)?;
        sync_directory(staging)?;
        ensure_absent(&self.destination, "recovery media destination")?;
        std::fs::rename(staging, &self.destination)
            .map_err(|error| format!("publish recovery media: {error}"))?;
        self.staging = None;
        let parent = self
            .destination
            .parent()
            .ok_or_else(|| "recovery media destination has no parent".to_owned())?;
        if let Err(error) = sync_directory(parent) {
            let cleanup = std::fs::remove_dir_all(&self.destination);
            let _ = sync_directory(parent);
            return match cleanup {
                Ok(()) => Err(format!("publish recovery media: {error}")),
                Err(cleanup_error) => Err(format!(
                    "publish recovery media: {error}; cleanup failed: {cleanup_error}"
                )),
            };
        }
        Ok(self.destination.clone())
    }
}

impl Drop for RecoveryMediaPublisher {
    fn drop(&mut self) {
        if let Some(staging) = self.staging.take() {
            let _ = std::fs::remove_dir_all(staging);
        }
    }
}

fn collect_payload_entries(root: &Path) -> Result<Vec<RecoveryMediaEntryV1>, String> {
    let mut entries = Vec::new();
    collect_directory(root, root, &mut entries)?;
    if entries.is_empty() {
        return Err("recovery media payload is empty".to_owned());
    }
    Ok(entries)
}

fn canonical_payload_entries(root: &Path) -> Result<Vec<RecoveryMediaEntryV1>, String> {
    let mut entries = collect_payload_entries(root)?;
    entries.sort_by(|left, right| left.canonical_order().cmp(&right.canonical_order()));
    Ok(entries)
}

fn collect_directory(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<RecoveryMediaEntryV1>,
) -> Result<(), String> {
    restrict_private_directory(directory)?;
    let paths = std::fs::read_dir(directory)
        .map_err(|_| "recovery media payload cannot be read".to_owned())?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|_| "recovery media payload cannot be read".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if directory != root && paths.is_empty() {
        return Err("recovery media payload contains an empty directory".to_owned());
    }
    for path in paths {
        collect_path(root, &path, entries)?;
    }
    sync_directory(directory)
}

fn collect_path(
    root: &Path,
    path: &Path,
    entries: &mut Vec<RecoveryMediaEntryV1>,
) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "recovery media payload entry is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() {
        return Err("recovery media payload must not contain symlinks".to_owned());
    }
    if metadata.is_dir() {
        return collect_directory(root, path, entries);
    }
    if !metadata.is_file() {
        return Err("recovery media payload must contain regular files only".to_owned());
    }
    let relative = path
        .strip_prefix(root)
        .ok()
        .and_then(Path::to_str)
        .ok_or_else(|| "recovery media payload path is invalid".to_owned())?;
    let component = RecoveryMediaComponentV1::from_path(relative)?;
    let (size_bytes, sha256) = secure_file_digest(path)?;
    entries.push(RecoveryMediaEntryV1::new(
        component,
        component.inclusion(),
        relative.to_owned(),
        size_bytes,
        sha256,
    )?);
    Ok(())
}

fn secure_file_digest(path: &Path) -> Result<(u64, [u8; 32]), String> {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|_| "recovery media payload permissions cannot be restricted".to_owned())?;
    let before = std::fs::symlink_metadata(path)
        .map_err(|_| "recovery media payload file is unavailable".to_owned())?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| "recovery media payload file cannot be opened".to_owned())?;
    let opened = file
        .metadata()
        .map_err(|_| "recovery media payload file cannot be inspected".to_owned())?;
    if !same_file(&before, &opened) {
        return Err("recovery media payload file changed while opening".to_owned());
    }
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| "recovery media payload file cannot be read".to_owned())?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    file.sync_all()
        .map_err(|_| "recovery media payload file cannot be synced".to_owned())?;
    let after = file
        .metadata()
        .map_err(|_| "recovery media payload file cannot be inspected".to_owned())?;
    let path_after = std::fs::symlink_metadata(path)
        .map_err(|_| "recovery media payload file is unavailable".to_owned())?;
    if !same_file(&opened, &after) || !same_file(&opened, &path_after) {
        return Err("recovery media payload file changed while reading".to_owned());
    }
    Ok((opened.len(), digest.finalize().into()))
}

fn create_staging(parent: &Path) -> Result<PathBuf, String> {
    for _ in 0..16 {
        let mut suffix = [0_u8; 8];
        getrandom::fill(&mut suffix)
            .map_err(|_| "recovery media staging entropy is unavailable".to_owned())?;
        let name = suffix
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let path = parent.join(format!(".makosh-recovery-staging-{name}"));
        match std::fs::create_dir(&path) {
            Ok(()) => {
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
                    .map_err(|error| error.to_string())?;
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("recovery media staging path is unavailable".to_owned())
}

fn create_private_directory(path: &Path) -> Result<(), String> {
    std::fs::create_dir(path).map_err(|error| error.to_string())?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())
}

fn ensure_private_directory(path: &Path, label: &str) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != current_uid()
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(format!("{label} must be an owner-private real directory"));
    }
    Ok(())
}

fn restrict_private_directory(path: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "recovery media payload directory is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() || metadata.uid() != current_uid() {
        return Err("recovery media payload directory is invalid".to_owned());
    }
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .map_err(|_| "recovery media payload permissions cannot be restricted".to_owned())
}

fn ensure_absent(path: &Path, label: &str) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(format!("{label} already exists")),
        Err(error) => Err(error.to_string()),
    }
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|error| error.to_string())?;
    std::io::Write::write_all(&mut file, bytes).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())
}

fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| error.to_string())
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

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and only reads process credentials.
    unsafe { libc::geteuid() }
}
