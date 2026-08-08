//! Production capture port over exact staged component executables.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_secure_file::{SecureReadPolicy, read as read_secure_file};

use crate::distribution::staged_artifact::StagedNativeArtifact;
use crate::infrastructure::paths::prepare_offline_control_store;
use crate::infrastructure::{filesystem, paths};
use crate::platform::events::reconciliation;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelay;
use crate::runtime::managed::execution;

use super::capture_coordinator::WholeInstanceCapturePort;
use super::control_store_media;
use super::restore_coordinator::WholeInstanceRestorePort;

const MAX_RECOVERY_PROCESS_RUNTIME: Duration = Duration::from_secs(2 * 60 * 60);

pub(crate) struct RecoveryComponentExecutables<'a> {
    pub(crate) vault: &'a StagedNativeArtifact,
    pub(crate) storage: &'a StagedNativeArtifact,
    pub(crate) blob: Option<&'a StagedNativeArtifact>,
    pub(crate) scheduler: Option<&'a StagedNativeArtifact>,
}

pub(crate) struct PostgresRecoveryCommandV1 {
    pub(crate) pg_dump: PathBuf,
    pub(crate) pg_restore: PathBuf,
    pub(crate) psql: PathBuf,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) database: String,
    pub(crate) username: String,
    pub(crate) ssl_mode: String,
    pub(crate) password_file: PathBuf,
}

pub(crate) struct ProcessWholeInstanceCapturePort<'a> {
    data_dir: PathBuf,
    store: SqliteControlStore,
    executables: RecoveryComponentExecutables<'a>,
    postgres: PostgresRecoveryCommandV1,
    _exclusive_lock: File,
}

pub(crate) struct ProcessWholeInstanceRestorePort<'a> {
    target_data_dir: PathBuf,
    executables: RecoveryComponentExecutables<'a>,
    postgres: PostgresRecoveryCommandV1,
    vault_recovery_key_file: PathBuf,
    event_relay: &'a dyn ManagedRuntimeRelay,
    _exclusive_lock: File,
}

impl<'a> ProcessWholeInstanceCapturePort<'a> {
    pub(crate) fn open(
        data_dir: PathBuf,
        executables: RecoveryComponentExecutables<'a>,
        postgres: PostgresRecoveryCommandV1,
    ) -> Result<Self, String> {
        let (data_dir, store_path, exclusive_lock) = prepare_offline_control_store(Some(data_dir))?;
        let store = crate::control_store::lifecycle::open_validated_control_store(&store_path)?;
        Ok(Self {
            data_dir,
            store,
            executables,
            postgres,
            _exclusive_lock: exclusive_lock,
        })
    }

    pub(crate) fn authorize_owner(&self, operation_digest: [u8; 32]) -> Result<(), String> {
        crate::identity::owner::authorization::authorize(
            &self.data_dir,
            &self.store,
            "whole_instance_capture_v1",
            operation_digest,
        )
    }

    pub(crate) fn backup_generation(&self) -> u64 {
        self.store.snapshot().generation()
    }
}

impl<'a> ProcessWholeInstanceRestorePort<'a> {
    pub(crate) fn open(
        target_data_dir: PathBuf,
        executables: RecoveryComponentExecutables<'a>,
        postgres: PostgresRecoveryCommandV1,
        vault_recovery_key_file: PathBuf,
        event_relay: &'a dyn ManagedRuntimeRelay,
    ) -> Result<Self, String> {
        let target_data_dir = paths::prepare_runtime_directories(&target_data_dir)?;
        let runtime_dir = filesystem::resolve_runtime_directory(&target_data_dir)?;
        let exclusive_lock = filesystem::acquire_runtime_directory_lock(&runtime_dir)?;
        Ok(Self {
            target_data_dir,
            executables,
            postgres,
            vault_recovery_key_file,
            event_relay,
            _exclusive_lock: exclusive_lock,
        })
    }

    fn restored_store(&self) -> Result<SqliteControlStore, String> {
        crate::control_store::lifecycle::open_validated_control_store(
            &self.target_data_dir.join("kernel-control-store.sqlite"),
        )
    }
}

impl WholeInstanceCapturePort for ProcessWholeInstanceCapturePort<'_> {
    fn verify_quiesced(&mut self) -> Result<(), String> {
        let current = crate::control_store::lifecycle::open_validated_control_store(
            &self.data_dir.join("kernel-control-store.sqlite"),
        )?;
        (current.snapshot() == self.store.snapshot())
            .then_some(())
            .ok_or_else(|| "whole-instance capture authority changed after locking".to_owned())
    }

    fn capture_control_store(&mut self, destination: &Path) -> Result<(), String> {
        control_store_media::capture(&self.data_dir, destination)
    }

    fn capture_vault(&mut self, destination: &Path) -> Result<(), String> {
        run_component(
            self.executables.vault,
            &[
                "export-backup".to_owned(),
                "--data-dir".to_owned(),
                path_argument(&self.data_dir.join("vault"))?,
                "--destination".to_owned(),
                path_argument(&destination.join("snapshot"))?,
            ],
            "Vault backup export",
        )
    }

    fn capture_storage(&mut self, destination: &Path) -> Result<(), String> {
        let mut arguments = vec![
            "export-backup".to_owned(),
            "--pg-dump".to_owned(),
            path_argument(&self.postgres.pg_dump)?,
        ];
        arguments.extend(self.postgres.connection_arguments()?);
        arguments.extend([
            "--output".to_owned(),
            path_argument(&destination.join("postgres.dump"))?,
        ]);
        run_component(
            self.executables.storage,
            &arguments,
            "Storage backup export",
        )
    }

    fn capture_blob(&mut self, destination: &Path) -> Result<(), String> {
        let executable = self
            .executables
            .blob
            .ok_or_else(|| "Blob backup executable is unavailable".to_owned())?;
        run_component(
            executable,
            &[
                "export-backup".to_owned(),
                "--data-dir".to_owned(),
                path_argument(&self.data_dir.join("blob"))?,
                "--destination".to_owned(),
                path_argument(&destination.join("snapshot"))?,
            ],
            "Blob backup export",
        )
    }

    fn capture_event_topology(&mut self, destination: &Path) -> Result<(), String> {
        write_private_file(
            &destination.join("topology.pb"),
            &reconciliation::recovery_snapshot(&self.store)?,
        )
    }

    fn capture_scheduler(&mut self, destination: &Path) -> Result<(), String> {
        let executable = self
            .executables
            .scheduler
            .ok_or_else(|| "Scheduler recovery executable is unavailable".to_owned())?;
        run_component(
            executable,
            &[
                "export-recovery-bundle".to_owned(),
                "--output".to_owned(),
                path_argument(&destination.join("storage-bundle.pb"))?,
            ],
            "Scheduler recovery export",
        )
    }
}

impl WholeInstanceRestorePort for ProcessWholeInstanceRestorePort<'_> {
    fn verify_empty_target(&mut self) -> Result<(), String> {
        std::fs::read_dir(&self.target_data_dir)
            .map_err(|_| "whole-instance restore target is unavailable".to_owned())?
            .next()
            .transpose()
            .map_err(|_| "whole-instance restore target is unavailable".to_owned())?
            .is_none()
            .then_some(())
            .ok_or_else(|| "whole-instance restore target must be empty".to_owned())
    }

    fn restore_control_store(&mut self, source: &Path) -> Result<(), String> {
        control_store_media::restore_to_empty_target(source, &self.target_data_dir)
    }

    fn restore_vault(&mut self, source: &Path) -> Result<(), String> {
        let target = create_component_directory(&self.target_data_dir, "vault")?;
        run_component(
            self.executables.vault,
            &[
                "restore-backup".to_owned(),
                "--data-dir".to_owned(),
                path_argument(&target)?,
                "--source".to_owned(),
                path_argument(&source.join("snapshot"))?,
                "--recovery-key-file".to_owned(),
                path_argument(&self.vault_recovery_key_file)?,
            ],
            "Vault backup restore",
        )
    }

    fn restore_storage(&mut self, source: &Path) -> Result<(), String> {
        let mut arguments = vec![
            "restore-backup".to_owned(),
            "--pg-restore".to_owned(),
            path_argument(&self.postgres.pg_restore)?,
            "--psql".to_owned(),
            path_argument(&self.postgres.psql)?,
        ];
        arguments.extend(self.postgres.connection_arguments()?);
        arguments.extend([
            "--input".to_owned(),
            path_argument(&source.join("postgres.dump"))?,
        ]);
        run_component(
            self.executables.storage,
            &arguments,
            "Storage backup restore",
        )
    }

    fn restore_blob(&mut self, source: &Path) -> Result<(), String> {
        let executable = self
            .executables
            .blob
            .ok_or_else(|| "Blob recovery executable is unavailable".to_owned())?;
        let target = create_component_directory(&self.target_data_dir, "blob")?;
        run_component(
            executable,
            &[
                "restore-backup".to_owned(),
                "--source".to_owned(),
                path_argument(&source.join("snapshot"))?,
                "--data-dir".to_owned(),
                path_argument(&target)?,
            ],
            "Blob backup restore",
        )
    }

    fn recreate_event_topology(
        &mut self,
        source: &Path,
        control_store_source: &Path,
    ) -> Result<(), String> {
        let bytes = read_secure_file(
            &source.join("topology.pb"),
            SecureReadPolicy::owner_private(512 * 1024),
        )
        .map_err(|_| "Event Hub recovery topology is unavailable".to_owned())?;
        let (_, source_store, _) = control_store_media::open_source(control_store_source)
            .map_err(|_| "Event Hub recovery authority is unavailable".to_owned())?;
        reconciliation::validate_recovery_snapshot(&source_store, &bytes)?;
        let _ = reconciliation::apply_recovery_snapshot(&bytes, self.event_relay)?;
        Ok(())
    }

    fn prepare_outbox_inbox_replay(
        &mut self,
        scheduler_source: Option<&Path>,
    ) -> Result<(), String> {
        let Some(source) = scheduler_source else {
            return Ok(());
        };
        let executable = self
            .executables
            .scheduler
            .ok_or_else(|| "Scheduler recovery executable is unavailable".to_owned())?;
        let mut arguments = vec!["prepare-event-replay".to_owned()];
        arguments.extend(self.postgres.connection_arguments()?);
        arguments.extend([
            "--storage-bundle".to_owned(),
            path_argument(&source.join("storage-bundle.pb"))?,
        ]);
        run_component(executable, &arguments, "Scheduler replay preparation")
    }

    fn invalidate_stale_runtime_state(&mut self) -> Result<(), String> {
        let store = self.restored_store()?;
        store
            .approved_module_grant_snapshots()
            .map_err(|_| "restored runtime fences are unavailable".to_owned())?
            .is_empty()
            .then_some(())
            .ok_or_else(|| "restored runtime grants were not invalidated".to_owned())
    }
}

impl PostgresRecoveryCommandV1 {
    fn connection_arguments(&self) -> Result<Vec<String>, String> {
        Ok(vec![
            "--host".to_owned(),
            self.host.clone(),
            "--port".to_owned(),
            self.port.to_string(),
            "--database".to_owned(),
            self.database.clone(),
            "--username".to_owned(),
            self.username.clone(),
            "--ssl-mode".to_owned(),
            self.ssl_mode.clone(),
            "--password-file".to_owned(),
            path_argument(&self.password_file)?,
        ])
    }
}

fn run_component(
    executable: &StagedNativeArtifact,
    arguments: &[String],
    label: &str,
) -> Result<(), String> {
    let mut child = execution::spawn(executable, arguments, Stdio::null())?;
    let status = execution::wait(&mut child, MAX_RECOVERY_PROCESS_RUNTIME)?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("{label} failed"))
}

fn create_component_directory(root: &Path, component: &str) -> Result<PathBuf, String> {
    let path = root.join(component);
    std::fs::create_dir(&path).map_err(|_| format!("{component} restore target is unavailable"))?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
        .map_err(|_| format!("{component} restore target is unavailable"))?;
    Ok(path)
}

fn path_argument(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| "recovery path is not valid UTF-8".to_owned())
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| "Event Hub recovery snapshot is unavailable".to_owned())?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "Event Hub recovery snapshot is unavailable".to_owned())?;
    let parent = path
        .parent()
        .ok_or_else(|| "Event Hub recovery snapshot is unavailable".to_owned())?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| "Event Hub recovery snapshot is unavailable".to_owned())
}
