//! Scheduler-owned stopped-instance preparation for exact Event Hub replay.

use makosh_scheduler_persistence::{
    SchedulerPostgresStoreV1, SchedulerRecoveryDatabaseV1, scheduler_storage_bundle_v1,
};
use makosh_secure_file::{SecureReadPolicy, read as read_secure_file};
use prost::Message;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;
use zeroize::{Zeroize, Zeroizing};

use crate::cli::RecoveryArguments;

const MAX_PASSWORD_BYTES: u64 = 4096;
const MAX_STORAGE_BUNDLE_BYTES: u64 = 512 * 1024;

pub(crate) fn export_storage_bundle(output: &Path) -> Result<(), String> {
    if !output.is_absolute() || std::fs::symlink_metadata(output).is_ok() {
        return Err(unavailable());
    }
    let parent = output.parent().ok_or_else(unavailable)?;
    let metadata = std::fs::symlink_metadata(parent).map_err(|_| unavailable())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != current_uid()
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(unavailable());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(output)
        .map_err(|_| unavailable())?;
    file.write_all(&scheduler_storage_bundle_v1().encode_to_vec())
        .and_then(|()| file.sync_all())
        .map_err(|_| unavailable())?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| unavailable())
}

pub(crate) fn prepare_event_hub_replay(arguments: &RecoveryArguments) -> Result<(), String> {
    validate_storage_bundle(arguments)?;
    let password = read_password(arguments)?;
    let database = SchedulerRecoveryDatabaseV1::new(
        arguments.host.clone(),
        arguments.port,
        arguments.database.clone(),
        arguments.username.clone(),
        &arguments.ssl_mode,
    )
    .map_err(|_| unavailable())?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| unavailable())?;
    let report = runtime.block_on(async {
        let store = SchedulerPostgresStoreV1::connect_recovery(&database, &password)
            .await
            .map_err(|_| unavailable())?;
        store
            .prepare_event_hub_replay()
            .await
            .map_err(|_| unavailable())
    })?;
    println!(
        "scheduler_requeued_dispatches={}",
        report.requeued_dispatches()
    );
    println!(
        "scheduler_preserved_acceptances={}",
        report.preserved_acceptances()
    );
    println!("scheduler_preserved_results={}", report.preserved_results());
    Ok(())
}

fn validate_storage_bundle(arguments: &RecoveryArguments) -> Result<(), String> {
    let bytes = read_secure_file(
        &arguments.storage_bundle,
        SecureReadPolicy::owner_private(MAX_STORAGE_BUNDLE_BYTES),
    )
    .map_err(|_| unavailable())?;
    (bytes == scheduler_storage_bundle_v1().encode_to_vec())
        .then_some(())
        .ok_or_else(unavailable)
}

fn read_password(arguments: &RecoveryArguments) -> Result<Zeroizing<String>, String> {
    let mut bytes = read_secure_file(
        &arguments.password_file,
        SecureReadPolicy::owner_private(MAX_PASSWORD_BYTES),
    )
    .map_err(|_| unavailable())?;
    let result = std::str::from_utf8(&bytes)
        .ok()
        .map(str::to_owned)
        .and_then(normalize_password);
    bytes.zeroize();
    result.map(Zeroizing::new).ok_or_else(unavailable)
}

fn normalize_password(mut password: String) -> Option<String> {
    if password.ends_with('\n') {
        let _ = password.pop();
        if password.ends_with('\r') {
            let _ = password.pop();
        }
    }
    (!password.is_empty()
        && password.len() <= 1024
        && !password
            .bytes()
            .any(|byte| matches!(byte, b'\r' | b'\n' | 0)))
    .then_some(password)
}

fn unavailable() -> String {
    "Scheduler offline recovery is unavailable".to_owned()
}

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and only reads process credentials.
    unsafe { libc::geteuid() }
}
