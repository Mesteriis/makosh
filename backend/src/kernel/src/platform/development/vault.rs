use std::path::Path;
use std::process::Command;

use makosh_kernel_control_store::PlatformManagedProcessBinding;

use crate::distribution::staged_artifact::StagedNativeArtifact;
use crate::platform::macos::native_launch;

pub(super) fn ensure_initialized_vault(
    data_dir: &Path,
    runtime_dir: &Path,
    binding: &PlatformManagedProcessBinding,
    instance_id: &str,
) -> Result<(), String> {
    let vault_dir = data_dir.join("vault");
    let state = match initialization_state(&vault_dir)? {
        VaultInitializationState::Initialized => VaultInitializationState::Initialized,
        VaultInitializationState::Partial => return Err("Vault recovery is required".to_owned()),
        VaultInitializationState::Missing => VaultInitializationState::Missing,
    };
    let kernel =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    let prepared = native_launch::prepare_bound_platform_process(
        &kernel,
        binding,
        &runtime_dir
            .join("developer-bootstrap")
            .join("vault-initialize"),
    )?;
    let artifact = prepared.into_staged_executable();
    let credential_dir = data_dir.join("developer-platform-credentials");
    match state {
        VaultInitializationState::Missing => {
            initialize_from_artifact(artifact, &vault_dir, instance_id, &credential_dir)
        }
        VaultInitializationState::Initialized => {
            reconcile_from_artifact(artifact, &vault_dir, &credential_dir)
        }
        VaultInitializationState::Partial => unreachable!("partial Vault state returned early"),
    }
}

fn initialize_from_artifact(
    artifact: StagedNativeArtifact,
    vault_dir: &Path,
    instance_id: &str,
    platform_credential_dir: &Path,
) -> Result<(), String> {
    let status = Command::new(artifact.path())
        .args(["initialize", "--data-dir"])
        .arg(vault_dir)
        .args(["--instance-id", instance_id])
        .args(["--platform-credential-dir"])
        .arg(platform_credential_dir)
        .status()
        .map_err(|_| "Vault initialization is unavailable".to_owned());
    let _ = artifact.remove();
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => Err("Vault initialization failed".to_owned()),
        Err(error) => Err(error),
    }
}

fn reconcile_from_artifact(
    artifact: StagedNativeArtifact,
    vault_dir: &Path,
    platform_credential_dir: &Path,
) -> Result<(), String> {
    let status = Command::new(artifact.path())
        .args(["import-platform-credentials", "--data-dir"])
        .arg(vault_dir)
        .args(["--platform-credential-dir"])
        .arg(platform_credential_dir)
        .status()
        .map_err(|_| "Vault platform credential reconciliation is unavailable".to_owned());
    let _ = artifact.remove();
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => Err("Vault platform credential reconciliation failed".to_owned()),
        Err(error) => Err(error),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VaultInitializationState {
    Missing,
    Initialized,
    Partial,
}

fn initialization_state(vault_dir: &Path) -> Result<VaultInitializationState, String> {
    let files = [
        vault_dir.join("platform-wrapping-key.bin"),
        vault_dir.join("vault.db"),
        vault_dir.join("vault.anchor"),
    ];
    match files.iter().filter(|path| path.exists()).count() {
        0 => Ok(VaultInitializationState::Missing),
        3 => Ok(VaultInitializationState::Initialized),
        _ => Ok(VaultInitializationState::Partial),
    }
}
