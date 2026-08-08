//! Disposable signed app bundle and Kernel process for external-runtime tests.

use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{ModuleDescriptorV1, ModuleKindV1};
use prost::Message;

use super::super::common::unique_target_root;
use crate::platform::managed::signed_bundle::{InstalledSignedBundle, SignedRuntimeArtifact};

pub(super) struct RunningKernel {
    root: PathBuf,
    pub(super) data_dir: PathBuf,
    pub(super) registration_socket: PathBuf,
    pub(super) owner_socket: PathBuf,
    pub(super) runtime_socket: PathBuf,
    child: Child,
}

impl RunningKernel {
    pub(super) fn start() -> Result<Self, String> {
        let root = unique_target_root("makosh-external-storage-process");
        let data_dir = private_directory(root.join("data"))?;
        let kernel_binary = binary("MAKOSH_KERNEL_RUNTIME_BIN")?;
        let vault_binary = binary("MAKOSH_VAULT_RUNTIME_BIN")?;
        let release = InstalledSignedBundle::install(
            &root,
            &[SignedRuntimeArtifact::new(
                "platform.vault",
                vault_binary.clone(),
                vault_descriptor().encode_to_vec(),
            )],
        )?;
        copy_executable(&kernel_binary, release.kernel())?;
        enroll_initial_owner(release.kernel(), &data_dir)?;
        initialize_vault(&vault_binary, &data_dir, instance_id(&data_dir)?)?;
        let mut child = Command::new(release.kernel())
            .args(["--data-dir"])
            .arg(&data_dir)
            .arg("serve")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| error.to_string())?;
        let sockets = read_sockets(&mut child)?;
        Ok(Self {
            root,
            data_dir,
            registration_socket: sockets.0,
            owner_socket: sockets.1,
            runtime_socket: sockets.2,
            child,
        })
    }

    pub(super) fn stop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

impl Drop for RunningKernel {
    fn drop(&mut self) {
        self.stop();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn binary(name: &str) -> Result<PathBuf, String> {
    std::env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .ok_or_else(|| format!("{name} must name a regular binary"))
}

fn private_directory(path: PathBuf) -> Result<PathBuf, String> {
    std::fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    Ok(path)
}

fn copy_executable(source: &Path, destination: &Path) -> Result<(), String> {
    std::fs::copy(source, destination).map_err(|error| error.to_string())?;
    std::fs::set_permissions(destination, std::fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())
}

fn enroll_initial_owner(kernel: &Path, data_dir: &Path) -> Result<(), String> {
    let output = Command::new(kernel)
        .args(["--data-dir"])
        .arg(data_dir)
        .args([
            "initial-owner-enroll",
            "--owner-id",
            "owner_storage",
            "--device-id",
            "device_storage",
        ])
        .output()
        .map_err(|error| error.to_string())?;
    output
        .status
        .success()
        .then_some(())
        .ok_or_else(|| "initial owner enrollment failed".to_owned())
}

fn instance_id(data_dir: &Path) -> Result<String, String> {
    let store = SqliteControlStore::open(&data_dir.join("kernel-control-store.sqlite"))
        .map_err(|error| format!("{error:?}"))?;
    Ok(store.snapshot().instance_id().to_owned())
}

fn initialize_vault(binary: &Path, data_dir: &Path, instance_id: String) -> Result<(), String> {
    let output = Command::new(binary)
        .args(["initialize", "--data-dir"])
        .arg(data_dir.join("vault"))
        .args(["--instance-id", &instance_id])
        .output()
        .map_err(|error| error.to_string())?;
    output
        .status
        .success()
        .then_some(())
        .ok_or_else(|| "Vault initialization failed".to_owned())
}

fn read_sockets(child: &mut Child) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Kernel stdout is unavailable".to_owned())?;
    let mut registration = None;
    let mut owner = None;
    let mut runtime = None;
    for line in BufReader::new(stdout).lines().take(32) {
        let line = line.map_err(|error| error.to_string())?;
        registration = registration.or_else(|| socket_value(&line, "module_registration_socket="));
        owner = owner.or_else(|| socket_value(&line, "owner_control_socket="));
        runtime = runtime.or_else(|| socket_value(&line, "external_runtime_session_socket="));
        if let (Some(registration), Some(owner), Some(runtime)) = (&registration, &owner, &runtime)
        {
            return Ok((registration.clone(), owner.clone(), runtime.clone()));
        }
    }
    Err("Kernel IPC sockets did not start".to_owned())
}

fn socket_value(line: &str, prefix: &str) -> Option<PathBuf> {
    line.strip_prefix(prefix).map(PathBuf::from)
}

fn vault_descriptor() -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "vault".to_owned(),
        owner_id: "vault".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "external-storage-process".to_owned(),
        ..Default::default()
    }
}
