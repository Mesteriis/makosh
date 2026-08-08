//! File-backed Vault setup and verified managed launch for integration tests.

use std::path::{Path, PathBuf};

use makosh_runtime_protocol::v1::{ModuleDescriptorV1, ModuleKindV1};
use prost::Message;

use super::super::common::*;
use crate::identity::device::signer::FileDeviceSigner;
use crate::platform::vault::managed_route::KernelManagedVaultRouteHandler;
use crate::platform::vault::{binding as vault_binding, launch as vault_launch};

pub(crate) fn vault_binary() -> PathBuf {
    binary("MAKOSH_VAULT_RUNTIME_BIN")
}

pub(crate) fn vault_descriptor() -> Vec<u8> {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "vault".to_owned(),
        owner_id: "vault".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "managed-vault-live-test".to_owned(),
        ..Default::default()
    }
    .encode_to_vec()
}

pub(crate) fn private_directory(path: PathBuf) -> PathBuf {
    std::fs::create_dir_all(&path).expect("private directory");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
        .expect("private directory mode");
    path
}

pub(crate) fn write_private(path: &Path, value: &[u8]) {
    std::fs::write(path, value).expect("private credential");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .expect("private credential mode");
}

pub(crate) fn initialize_vault(data: &Path) {
    let output = std::process::Command::new(vault_binary())
        .args(["initialize", "--data-dir"])
        .arg(data.join("vault"))
        .args(["--instance-id", "kernel-main"])
        .output()
        .expect("Vault initializer");
    assert!(output.status.success(), "Vault initialization failed");
}

pub(crate) fn bind_and_start(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    data: &Path,
    kernel: &Path,
) {
    let _ = FileDeviceSigner::open_or_create_for_instance(data).expect("Kernel signer");
    vault_binding::bind_installed_release(store, kernel).expect("bind signed Vault release");
    supervisor
        .configure_vault_route_handler(Arc::new(KernelManagedVaultRouteHandler::new(
            Arc::clone(store),
            data,
            Arc::new(supervisor.relay_port()),
        )))
        .expect("configure Vault route handler");
    vault_launch::start_from_kernel(supervisor, store, data, kernel, &data.join("runtime"))
        .expect("start signed Vault");
}

fn binary(name: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .unwrap_or_else(|| panic!("{name} must name a regular binary"))
}
