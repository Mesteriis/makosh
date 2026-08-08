//! Real Vault binary smoke through the managed-child inherited-FD supervisor.

use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use makosh_runtime_protocol::v1::{
    GetVaultRuntimeStatusRequestV1, ManagedVaultRuntimeControlRequestV1,
    ManagedVaultRuntimeControlResponseV1, VaultRuntimeStateV1,
    managed_vault_runtime_control_request_v1::Operation,
    managed_vault_runtime_control_response_v1::Result as ResponseResult,
};
use prost::Message;

use super::common::*;

#[test]
#[ignore = "builds and launches the real Vault runtime binary"]
fn managed_vault_binary_serves_status_over_the_inherited_kernel_channel() {
    let root = unique_target_root("makosh-managed-vault-binary");
    let data_dir = private_directory(root.join("vault"));
    initialize_vault_binary(&data_dir);
    let descriptor = descriptor();
    let descriptor_bytes = descriptor.encode_to_vec();
    let contracts = StagedRuntimeContracts::stage(&root.join("contracts"), &descriptor_bytes, None)
        .expect("staged contracts");
    let staged = stage_binary(&root);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    supervisor
        .start_with_arguments_and_contracts(
            "vault".to_owned(),
            staged,
            inherited_arguments(&data_dir, &contracts),
            ManagedRuntimeExpectation::new(
                "vault",
                "vault-runtime",
                "vault",
                1,
                1,
                Sha256::digest(&descriptor_bytes).into(),
                None,
            ),
            ManagedChildExecutionPolicy::new(1, Duration::from_secs(30)).expect("execution policy"),
            contracts,
        )
        .expect("managed Vault start");

    let response = supervisor
        .relay(
            "vault",
            ManagedVaultRuntimeControlRequestV1 {
                operation: Some(Operation::GetStatus(GetVaultRuntimeStatusRequestV1 {})),
            }
            .encode_to_vec(),
        )
        .expect("managed Vault status");
    let response = ManagedVaultRuntimeControlResponseV1::decode(response.as_slice())
        .expect("typed Vault status");
    assert!(
        matches!(response.result, Some(ResponseResult::Status(status))
        if status.state == VaultRuntimeStateV1::Ready as i32
            && status.vault_runtime_generation == 1
            && status.hpke_public_key_x25519.len() == 32)
    );

    supervisor.shutdown().expect("stop managed Vault");
    std::fs::remove_dir_all(root).expect("remove fixture");
}

fn private_directory(path: std::path::PathBuf) -> std::path::PathBuf {
    std::fs::create_dir_all(&path).expect("private Vault directory");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
        .expect("private Vault directory mode");
    path
}

fn initialize_vault_binary(data_dir: &std::path::Path) {
    let output = std::process::Command::new(runtime_binary())
        .args(["initialize", "--data-dir"])
        .arg(data_dir)
        .args(["--instance-id", "kernel-main"])
        .output()
        .expect("Vault initialize process");
    assert!(output.status.success(), "Vault initialization failed");
}

fn stage_binary(root: &std::path::Path) -> staged_native_artifact::StagedNativeArtifact {
    let binary = runtime_binary();
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&binary).expect("Vault binary bytes")).into();
    staged_native_artifact::stage(&binary, &root.join("launch"), "vault-runtime", &digest)
        .expect("stage Vault runtime")
}

fn runtime_binary() -> std::path::PathBuf {
    std::env::var_os("MAKOSH_VAULT_RUNTIME_BIN")
        .map(std::path::PathBuf::from)
        .filter(|path| path.is_file())
        .expect("MAKOSH_VAULT_RUNTIME_BIN must name a regular Vault binary")
}

fn descriptor() -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "vault".to_owned(),
        owner_id: "vault".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "managed-binary-test".to_owned(),
        ..Default::default()
    }
}

fn inherited_arguments(
    data_dir: &std::path::Path,
    contracts: &StagedRuntimeContracts,
) -> Vec<String> {
    vec![
        "serve-inherited".to_owned(),
        "--data-dir".to_owned(),
        data_dir.display().to_string(),
        "--instance-id".to_owned(),
        "kernel-main".to_owned(),
        "--runtime-generation".to_owned(),
        "1".to_owned(),
        "--descriptor-path".to_owned(),
        contracts.descriptor_path().display().to_string(),
        "--kernel-authorization-key-sec1-hex".to_owned(),
        "04".repeat(65),
    ]
}
