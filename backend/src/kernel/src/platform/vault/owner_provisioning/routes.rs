//! Opaque HPKE route construction for an owner-authorized provisioning session.

use std::path::Path;

use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    VaultCiphertextResponseV1, VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1,
};
use makosh_vault_protocol::VaultCiphertextFrameV1;

use crate::platform::vault::managed_route::relay_kernel_authorized_route;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelay;

pub(super) struct OwnerVaultRouteInputV1<'a> {
    pub(super) registration_id: &'a str,
    pub(super) runtime_instance_id: &'a str,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    pub(super) vault_runtime_generation: u64,
    pub(super) request_id: [u8; 16],
    pub(super) operation_digest_sha256: [u8; 32],
    pub(super) response_recipient_public_key: [u8; 32],
    pub(super) frame: VaultCiphertextFrameV1,
}

pub(super) fn relay(
    store: &SqliteControlStore,
    data_dir: &Path,
    relay: &dyn ManagedRuntimeRelay,
    input: OwnerVaultRouteInputV1<'_>,
) -> Result<VaultCiphertextResponseV1, String> {
    let route = VaultCiphertextRouteV1 {
        major: 1,
        registration_id: input.registration_id.to_owned(),
        runtime_instance_id: input.runtime_instance_id.to_owned(),
        vault_runtime_generation: input.vault_runtime_generation,
        grant_epoch: input.grant_epoch,
        request_id: input.request_id.to_vec(),
        operation_digest_sha256: input.operation_digest_sha256.to_vec(),
        direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
        hpke_encapped_key: input.frame.encapped_key().to_vec(),
        ciphertext: input.frame.ciphertext().to_vec(),
        hpke_authentication_tag: input.frame.tag().to_vec(),
        response_recipient_hpke_public_key_x25519: input.response_recipient_public_key.to_vec(),
        kernel_instance_id: String::new(),
        kernel_authorization_signature_raw: Vec::new(),
        caller_runtime_generation: input.runtime_generation,
        storage_role_epoch: 0,
        storage_credential_lease_revision: 0,
        storage_runtime_principal: String::new(),
        storage_owner_id: String::new(),
    };
    relay_kernel_authorized_route(store, data_dir, relay, route)
}
