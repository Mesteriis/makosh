//! Exclusive offline restore/reset operations.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use makosh_kernel_control_store_sqlite::{SqliteControlStore, StagedControlStoreRestore};

use crate::cli::OfflineControlStoreCommand;
use crate::control_store::lifecycle::{
    installation_anchor_path, open_validated_control_store, read_installation_anchor,
    reset_untrusted_control_store,
};
use crate::infrastructure::filesystem::ensure_regular_file_or_absent;
use crate::infrastructure::paths::prepare_offline_control_store;
use crate::recovery::fence as recovery_fence;

pub(crate) fn run(
    data_dir_override: Option<PathBuf>,
    operation: OfflineControlStoreCommand,
) -> Result<(), String> {
    let (data_dir, store_path, _lock) = prepare_offline_control_store(data_dir_override)?;
    match operation {
        OfflineControlStoreCommand::Restore { source } => restore(&data_dir, &store_path, &source),
        OfflineControlStoreCommand::Reset => reset(&data_dir, &store_path),
    }
}

fn restore(data_dir: &Path, store_path: &Path, source: &Path) -> Result<(), String> {
    if !source.is_absolute() {
        return Err("restore source must be an absolute path".to_owned());
    }
    ensure_regular_file_or_absent(source, "restore source")?;
    if !source.exists() {
        return Err("restore source does not exist".to_owned());
    }
    if source == store_path {
        return Err("restore source must differ from the target control store".to_owned());
    }
    confirm("restore", data_dir)?;
    let instance_id = read_installation_anchor(&installation_anchor_path(data_dir))?;
    let restored = install_fenced_restore(data_dir, source, store_path, &instance_id)?;
    println!("control_store_generation={}", restored.generation());
    Ok(())
}

fn reset(data_dir: &Path, store_path: &Path) -> Result<(), String> {
    confirm("reset", data_dir)?;
    match open_validated_control_store(store_path) {
        Ok(store) => {
            let reset = install_fenced_restore(
                data_dir,
                store_path,
                store_path,
                store.snapshot().instance_id(),
            )?;
            println!("reset_mode=fence_existing_instance");
            println!("control_store_generation={}", reset.generation());
        }
        Err(_) => {
            let reset = reset_untrusted_control_store(data_dir, store_path)?;
            println!("reset_mode=new_instance");
            println!("control_store_generation={}", reset.generation());
        }
    }
    Ok(())
}

fn install_fenced_restore(
    data_dir: &Path,
    source: &Path,
    destination: &Path,
    instance_id: &str,
) -> Result<makosh_kernel_control_store::ControlStore, String> {
    let anchor =
        recovery_fence::read(data_dir).map_err(|error| format!("read recovery fence: {error}"))?;
    let source_store = SqliteControlStore::open(source)
        .map_err(|error| format!("open restore source: {error:?}"))?;
    let current = SqliteControlStore::open(destination).ok();
    let fences = recovery_fence::next(
        &anchor,
        current.as_ref().map(SqliteControlStore::snapshot),
        source_store.snapshot(),
    )
    .map_err(|error| format!("calculate restore fences: {error}"))?;
    install_staged(data_dir, source, destination, instance_id, fences)
}

fn install_staged(
    data_dir: &Path,
    source: &Path,
    destination: &Path,
    instance_id: &str,
    fences: makosh_kernel_control_store::RecoveryFences,
) -> Result<makosh_kernel_control_store::ControlStore, String> {
    let staged_path = data_dir.join(".kernel-control-store.sqlite.restore.tmp");
    let _ = std::fs::remove_file(&staged_path);
    let staged = StagedControlStoreRestore::prepare(source, &staged_path, instance_id, fences)
        .map_err(|error| format!("prepare staged restore: {error:?}"))?;
    let reservation = recovery_fence::reserve(data_dir, instance_id, fences, *staged.sha256())
        .map_err(|error| format!("reserve recovery fence: {error}"))?;
    if let Err(error) = std::fs::rename(staged.path(), destination) {
        let _ = std::fs::remove_file(staged.path());
        return Err(format!("install staged restore: {error}"));
    }
    File::open(data_dir)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("sync restored Control Store directory: {error}"))?;
    recovery_fence::commit(data_dir, &reservation)
        .map_err(|error| format!("commit recovery fence: {error}"))?;
    Ok(staged.snapshot().clone())
}

fn confirm(operation: &str, data_dir: &Path) -> Result<(), String> {
    println!("offline_control_store_operation={operation}");
    println!("target_data_dir={}", data_dir.display());
    let expected = operation.to_ascii_uppercase();
    eprint!("Type {expected} to confirm: ");
    std::io::stderr()
        .flush()
        .map_err(|error| error.to_string())?;
    let mut confirmation = String::new();
    std::io::stdin()
        .read_line(&mut confirmation)
        .map_err(|error| error.to_string())?;
    if confirmation.trim() != expected {
        return Err("offline control-store operation was not confirmed".to_owned());
    }
    Ok(())
}
