//! Exact Control Store, installation authority and high-water fence media.

use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use makosh_kernel_control_store_sqlite::{SqliteControlStore, StagedControlStoreRestore};
use makosh_secure_file::{SecureReadPolicy, read as read_secure_file};

use crate::control_store::lifecycle::{
    installation_anchor_path, open_validated_control_store, read_installation_anchor,
    write_installation_anchor,
};
use crate::infrastructure::filesystem::ensure_owner_private_directory;

use super::fence as recovery_fence;

const STORE_FILE: &str = "control-store.sqlite";
const RUNTIME_STORE_FILE: &str = "kernel-control-store.sqlite";
const ANCHOR_FILE: &str = ".makosh-installation-anchor-v1";
const FENCE_FILE: &str = ".makosh-recovery-fence-v1";

pub(crate) fn capture(data_dir: &Path, destination: &Path) -> Result<(), String> {
    validate_empty_private_directory(destination, "Control Store media destination")?;
    let store = open_validated_control_store(&data_dir.join(RUNTIME_STORE_FILE))?;
    let export = store
        .export_to(&destination.join(STORE_FILE))
        .map_err(|_| unavailable())?;
    let exported =
        SqliteControlStore::open(&destination.join(STORE_FILE)).map_err(|_| unavailable())?;
    if exported.snapshot().instance_id() != store.snapshot().instance_id()
        || exported.snapshot().generation() != store.snapshot().generation()
        || exported.snapshot().identity_epoch() != store.snapshot().identity_epoch()
        || exported.snapshot().grant_epoch() != store.snapshot().grant_epoch()
        || export.instance_id() != store.snapshot().instance_id()
    {
        return Err(unavailable());
    }
    copy_private_file(
        &installation_anchor_path(data_dir),
        &destination.join(ANCHOR_FILE),
        256,
    )?;
    copy_private_file(
        &recovery_fence::path(data_dir),
        &destination.join(FENCE_FILE),
        1024,
    )?;
    sync_directory(destination)
}

pub(crate) fn restore_to_empty_target(source: &Path, target: &Path) -> Result<(), String> {
    let (instance_id, source_store, record) = open_source(source)?;
    validate_empty_private_directory(target, "Control Store restore target")?;
    let fences = recovery_fence::next(&record, None, source_store.snapshot())?;
    install_target(source, target, &instance_id, fences)
}

pub(crate) fn open_source(
    source: &Path,
) -> Result<
    (
        String,
        SqliteControlStore,
        recovery_fence::RecoveryFenceRecord,
    ),
    String,
> {
    validate_source_layout(source)?;
    let instance_id = read_installation_anchor(&source.join(ANCHOR_FILE))?;
    let source_store =
        SqliteControlStore::open(&source.join(STORE_FILE)).map_err(|_| unavailable())?;
    if source_store.snapshot().instance_id() != instance_id {
        return Err("Control Store recovery authority does not match the snapshot".to_owned());
    }
    let record = recovery_fence::read(source).map_err(|_| unavailable())?;
    recovery_fence::verify_committed_source(&record, source_store.snapshot())
        .map_err(|_| unavailable())?;
    Ok((instance_id, source_store, record))
}

fn install_target(
    source: &Path,
    target: &Path,
    instance_id: &str,
    fences: makosh_kernel_control_store::RecoveryFences,
) -> Result<(), String> {
    let staged_path = target.join(".kernel-control-store.sqlite.restore.tmp");
    let result = StagedControlStoreRestore::prepare(
        &source.join(STORE_FILE),
        &staged_path,
        instance_id,
        fences,
    )
    .map_err(|_| unavailable())
    .and_then(|_| write_installation_anchor(&installation_anchor_path(target), instance_id))
    .and_then(|()| recovery_fence::initialize(target, instance_id, fences))
    .and_then(|()| {
        std::fs::rename(&staged_path, target.join(RUNTIME_STORE_FILE)).map_err(|_| unavailable())
    })
    .and_then(|()| sync_directory(target));
    if result.is_err() {
        let _ = std::fs::remove_file(staged_path);
    }
    result
}

fn validate_source_layout(source: &Path) -> Result<(), String> {
    validate_private_directory(source, "Control Store recovery source")?;
    let actual = std::fs::read_dir(source)
        .map_err(|_| unavailable())?
        .map(|entry| {
            entry
                .map_err(|_| unavailable())?
                .file_name()
                .into_string()
                .map_err(|_| unavailable())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let expected = [STORE_FILE, ANCHOR_FILE, FENCE_FILE]
        .into_iter()
        .map(str::to_owned)
        .collect();
    (actual == expected)
        .then_some(())
        .ok_or_else(|| "Control Store recovery layout is invalid".to_owned())
}

fn validate_empty_private_directory(path: &Path, label: &str) -> Result<(), String> {
    validate_private_directory(path, label)?;
    std::fs::read_dir(path)
        .map_err(|_| unavailable())?
        .next()
        .transpose()
        .map_err(|_| unavailable())?
        .is_none()
        .then_some(())
        .ok_or_else(|| format!("{label} must be empty"))
}

fn validate_private_directory(path: &Path, label: &str) -> Result<(), String> {
    if !path.is_absolute() {
        return Err(format!("{label} is invalid"));
    }
    ensure_owner_private_directory(path).map_err(|_| format!("{label} is invalid"))
}

fn copy_private_file(source: &Path, destination: &Path, maximum: u64) -> Result<(), String> {
    let bytes = read_secure_file(source, SecureReadPolicy::owner_private(maximum))
        .map_err(|_| unavailable())?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(destination)
        .map_err(|_| unavailable())?;
    output.write_all(&bytes).map_err(|_| unavailable())?;
    output.sync_all().map_err(|_| unavailable())
}

fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| unavailable())
}

fn unavailable() -> String {
    "Control Store recovery media is unavailable".to_owned()
}
