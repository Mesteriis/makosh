use std::os::unix::fs::PermissionsExt;

use makosh_runtime_protocol::v1::{
    GetVaultRuntimeStatusRequestV1, ManagedVaultRuntimeControlRequestV1, VaultRuntimeStateV1,
    managed_vault_runtime_control_request_v1::Operation,
    managed_vault_runtime_control_response_v1::Result as ResponseResult,
};
use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_store_sqlcipher::VaultStore;
use tempfile::TempDir;

use crate::control::runtime::response_for;
use crate::service::runtime::VaultService;
use crate::transport::keys::VaultTransportKeyPair;
use crate::transport::session::VaultTransportReplayGuard;

#[test]
fn managed_control_returns_only_the_current_typed_vault_status() {
    let (mut service, keys) = service_and_keys();
    let mut replay_guard = VaultTransportReplayGuard::new(7);
    let response = response_for(
        ManagedVaultRuntimeControlRequestV1 {
            operation: Some(Operation::GetStatus(GetVaultRuntimeStatusRequestV1 {})),
        },
        &mut service,
        &keys,
        &mut replay_guard,
        [9; 65],
    )
    .expect("managed status response");

    assert!(response.error_code.is_empty());
    match response.result {
        Some(ResponseResult::Status(status)) => {
            assert_eq!(status.state, VaultRuntimeStateV1::Ready as i32);
            assert_eq!(status.vault_runtime_generation, 7);
            assert_eq!(status.hpke_public_key_x25519, keys.public_key().as_bytes());
            assert!(status.blocker_code.is_empty());
        }
        _ => panic!("expected managed Vault status"),
    }
}

#[test]
fn managed_control_rejects_an_empty_operation_without_echoing_input() {
    let (mut service, keys) = service_and_keys();
    let mut replay_guard = VaultTransportReplayGuard::new(7);
    let response = response_for(
        ManagedVaultRuntimeControlRequestV1 { operation: None },
        &mut service,
        &keys,
        &mut replay_guard,
        [9; 65],
    )
    .expect("sanitized denial");

    assert!(response.result.is_none());
    assert_eq!(response.error_code, "operation_not_available");
}

fn service_and_keys() -> (VaultService, VaultTransportKeyPair) {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let key = FileWrappingKeyProvider::new(&temporary.path().join("platform-wrapping-key.bin"))
        .load_or_create()
        .expect("file wrapping key");
    let store = VaultStore::initialize(
        &temporary.path().join("vault.db"),
        &temporary.path().join("vault.anchor"),
        "vault-instance",
        &key,
    )
    .expect("Vault store");
    (
        VaultService::new(store, 7).expect("Vault service"),
        VaultTransportKeyPair::generate(),
    )
}
