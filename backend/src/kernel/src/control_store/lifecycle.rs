use std::fs::File;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use makosh_kernel_control_store::ControlStore;
use makosh_kernel_control_store::RecoveryFences;
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::infrastructure::filesystem::{ensure_regular_file_or_absent, new_instance_id};
use crate::recovery::fence as recovery_fence;

const INSTALLATION_ANCHOR_FILE: &str = ".makosh-installation-anchor-v1";
const INSTALLATION_ANCHOR_PREFIX: &str = "makosh-installation-anchor-v1:";

pub fn installation_anchor_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join(INSTALLATION_ANCHOR_FILE)
}

pub fn bootstrap_control_store(
    data_dir: &Path,
    store_path: &Path,
) -> Result<SqliteControlStore, String> {
    let anchor_path = installation_anchor_path(data_dir);
    ensure_regular_file_or_absent(&anchor_path, "installation anchor")?;
    ensure_regular_file_or_absent(store_path, "control store")?;
    if anchor_path.exists() {
        return open_validated_control_store(store_path);
    }
    if store_path.exists() || data_directory_has_artifacts(data_dir)? {
        return Err(
            "installation anchor is missing; explicit offline recovery is required".to_owned(),
        );
    }

    let instance_id = new_instance_id()?;
    let store = SqliteControlStore::create(store_path, &instance_id, 1)
        .map_err(|error| format!("{error:?}"))?;
    recovery_fence::initialize(data_dir, &instance_id, RecoveryFences::new(1, 1, 1))?;
    write_installation_anchor(&anchor_path, &instance_id)?;
    Ok(store)
}

pub fn open_validated_control_store(store_path: &Path) -> Result<SqliteControlStore, String> {
    let data_dir = store_path
        .parent()
        .ok_or_else(|| "control store path has no data directory".to_owned())?;
    let anchor_path = installation_anchor_path(data_dir);
    let instance_id = read_installation_anchor(&anchor_path)?;
    ensure_regular_file_or_absent(store_path, "control store")?;
    if !store_path.exists() {
        return Err("control store is missing; explicit offline recovery is required".to_owned());
    }
    let store = SqliteControlStore::open(store_path).map_err(|error| format!("{error:?}"))?;
    if store.snapshot().instance_id() != instance_id {
        return Err("control store does not match installation anchor".to_owned());
    }
    recovery_fence::verify_or_finalize(data_dir, store_path, store.snapshot())?;
    Ok(store)
}

pub fn read_installation_anchor(anchor_path: &Path) -> Result<String, String> {
    ensure_regular_file_or_absent(anchor_path, "installation anchor")?;
    let bytes = std::fs::read(anchor_path).map_err(|error| error.to_string())?;
    let anchor = std::str::from_utf8(&bytes)
        .map_err(|_| "installation anchor is invalid".to_owned())?
        .strip_suffix('\n')
        .ok_or_else(|| "installation anchor is invalid".to_owned())?;
    let instance_id = anchor
        .strip_prefix(INSTALLATION_ANCHOR_PREFIX)
        .ok_or_else(|| "installation anchor is invalid".to_owned())?;
    if instance_id.len() != 32 || !instance_id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("installation anchor is invalid".to_owned());
    }
    Ok(instance_id.to_owned())
}

pub fn reset_untrusted_control_store(
    data_dir: &Path,
    store_path: &Path,
) -> Result<ControlStore, String> {
    let anchor_path = installation_anchor_path(data_dir);
    ensure_regular_file_or_absent(&anchor_path, "installation anchor")?;
    ensure_regular_file_or_absent(store_path, "control store")?;

    let instance_id = new_instance_id()?;
    let temporary_store = data_dir.join(format!(
        ".kernel-control-store.sqlite.{instance_id}.reset.tmp"
    ));
    let temporary_file = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary_store)
        .map_err(|error| error.to_string())?;
    drop(temporary_file);

    let created = match SqliteControlStore::create(&temporary_store, &instance_id, 1) {
        Ok(store) => store,
        Err(error) => {
            let _ = std::fs::remove_file(&temporary_store);
            return Err(format!("{error:?}"));
        }
    };
    if let Err(error) = File::open(&temporary_store).and_then(|file| file.sync_all()) {
        let _ = std::fs::remove_file(&temporary_store);
        return Err(error.to_string());
    }

    // A crash between these two replacements leaves an anchor/store mismatch,
    // which remains recovery-only. It can never look like a pristine install.
    if let Err(error) =
        recovery_fence::initialize(data_dir, &instance_id, RecoveryFences::new(1, 1, 1))
    {
        let _ = std::fs::remove_file(&temporary_store);
        return Err(error);
    }
    if let Err(error) = write_installation_anchor(&anchor_path, &instance_id) {
        let _ = std::fs::remove_file(&temporary_store);
        return Err(error);
    }
    if let Err(error) = std::fs::rename(&temporary_store, store_path) {
        let _ = std::fs::remove_file(&temporary_store);
        return Err(error.to_string());
    }
    File::open(data_dir)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| error.to_string())?;
    Ok(created.snapshot().clone())
}

fn data_directory_has_artifacts(data_dir: &Path) -> Result<bool, String> {
    let mut entries = std::fs::read_dir(data_dir).map_err(|error| error.to_string())?;
    Ok(entries
        .next()
        .transpose()
        .map_err(|error| error.to_string())?
        .is_some())
}

pub(crate) fn write_installation_anchor(
    anchor_path: &Path,
    instance_id: &str,
) -> Result<(), String> {
    let parent = anchor_path
        .parent()
        .ok_or_else(|| "installation anchor path has no parent".to_owned())?;
    let temporary = parent.join(format!(".{INSTALLATION_ANCHOR_FILE}.{instance_id}.tmp"));
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    let contents = format!("{INSTALLATION_ANCHOR_PREFIX}{instance_id}\n");
    let result = file
        .write_all(contents.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|error| error.to_string());
    drop(file);
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    std::fs::rename(&temporary, anchor_path).map_err(|error| error.to_string())?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| error.to_string())
}
