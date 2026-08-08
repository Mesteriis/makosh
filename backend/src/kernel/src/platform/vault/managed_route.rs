//! Fences and relays ciphertext from a verified managed platform runtime.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    ManagedVaultRuntimeControlRequestV1, ManagedVaultRuntimeControlResponseV1,
    VaultCiphertextResponseV1, VaultCiphertextRouteV1,
    managed_vault_runtime_control_request_v1::Operation,
    managed_vault_runtime_control_response_v1::Result as ResponseResult,
};
use makosh_runtime_protocol::validation::vault::{
    STORAGE_REVOKE_AUDIENCE_OPERATION_DIGEST_V1, validate_vault_ciphertext_route_v1,
};
use prost::Message;

use crate::platform::storage::binding::STORAGE_PROCESS_ID;
use crate::platform::vault::binding::VAULT_PROCESS_ID;
use crate::platform::vault::{ciphertext_route, launch};
use crate::runtime::lifecycle::control::{
    ManagedRuntimeExpectation, ManagedRuntimeVaultRouteHandler,
};
use crate::runtime::lifecycle::fence::{
    current_managed_runtime_matches, current_platform_managed_runtime_matches,
};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelay;

pub struct KernelManagedVaultRouteHandler {
    store: Arc<SqliteControlStore>,
    data_dir: PathBuf,
    relay: Arc<dyn ManagedRuntimeRelay>,
}

impl KernelManagedVaultRouteHandler {
    #[must_use]
    pub fn new(
        store: Arc<SqliteControlStore>,
        data_dir: &Path,
        relay: Arc<dyn ManagedRuntimeRelay>,
    ) -> Self {
        Self {
            store,
            data_dir: data_dir.to_path_buf(),
            relay,
        }
    }
}

impl ManagedRuntimeVaultRouteHandler for KernelManagedVaultRouteHandler {
    fn route_vault_ciphertext(
        &self,
        expectation: &ManagedRuntimeExpectation,
        route: VaultCiphertextRouteV1,
    ) -> Result<VaultCiphertextResponseV1, String> {
        validate_vault_ciphertext_route_v1(&route)
            .map_err(|_| "managed Vault ciphertext route is invalid".to_owned())?;
        if route.registration_id == expectation.registration_id() {
            let current_runtime = current_platform_managed_runtime_matches(
                &*self.store,
                expectation.registration_id(),
                expectation.runtime_instance_id(),
                expectation.runtime_generation(),
                expectation.grant_epoch(),
            )
            .map_err(|_| "managed Vault ciphertext route is unavailable".to_owned())?
            .map_or_else(
                || {
                    current_managed_runtime_matches(
                        &*self.store,
                        expectation.registration_id(),
                        expectation.runtime_instance_id(),
                        expectation.runtime_generation(),
                        expectation.grant_epoch(),
                    )
                },
                Ok,
            )
            .map_err(|_| "managed Vault ciphertext route is unavailable".to_owned())?;
            if route.caller_runtime_generation != expectation.runtime_generation()
                || route.grant_epoch != expectation.grant_epoch()
                || !current_runtime
            {
                return Err("managed Vault ciphertext route is stale".to_owned());
            }
        } else {
            self.authorize_storage_delegated_route(expectation, &route)?;
        }
        relay_kernel_authorized_route(&self.store, &self.data_dir, &*self.relay, route)
    }
}

impl KernelManagedVaultRouteHandler {
    fn authorize_storage_delegated_route(
        &self,
        expectation: &ManagedRuntimeExpectation,
        route: &VaultCiphertextRouteV1,
    ) -> Result<(), String> {
        authorize_storage_delegated_route(&self.store, expectation, route)
    }
}

pub(crate) fn authorize_storage_delegated_route(
    store: &SqliteControlStore,
    expectation: &ManagedRuntimeExpectation,
    route: &VaultCiphertextRouteV1,
) -> Result<(), String> {
    if expectation.registration_id() != STORAGE_PROCESS_ID
        || current_platform_managed_runtime_matches(
            store,
            expectation.registration_id(),
            expectation.runtime_instance_id(),
            expectation.runtime_generation(),
            expectation.grant_epoch(),
        )
        .map_err(|_| "managed Vault ciphertext route is unavailable".to_owned())?
            != Some(true)
    {
        return Err("managed Vault ciphertext route is stale".to_owned());
    }
    let bindings = store
        .platform_storage_bindings()
        .map_err(|_| "managed Vault ciphertext route is unavailable".to_owned())?
        .into_iter()
        .filter(|binding| {
            binding.registration_id() == route.registration_id
                && binding.runtime_instance_id() == route.runtime_instance_id
                && binding.runtime_generation() == route.caller_runtime_generation
                && binding.grant_epoch() == route.grant_epoch
                && binding.role_epoch() == route.storage_role_epoch
                && binding.credential_lease_revision() == route.storage_credential_lease_revision
                && binding.runtime_principal() == route.storage_runtime_principal
                && binding.owner_id() == route.storage_owner_id
        })
        .collect::<Vec<_>>();
    let [binding] = bindings.as_slice() else {
        return Err("managed Vault ciphertext route is stale".to_owned());
    };
    if binding.state() == PlatformStorageBindingStateV1::Revoking {
        return (route.operation_digest_sha256 == STORAGE_REVOKE_AUDIENCE_OPERATION_DIGEST_V1)
            .then_some(())
            .ok_or_else(|| "managed Vault ciphertext route is stale".to_owned());
    }
    if binding.state() != PlatformStorageBindingStateV1::Active {
        return Err("managed Vault ciphertext route is stale".to_owned());
    }
    current_managed_runtime_matches(
        store,
        &route.registration_id,
        &route.runtime_instance_id,
        route.caller_runtime_generation,
        route.grant_epoch,
    )
    .map_err(|_| "managed Vault ciphertext route is unavailable".to_owned())?
    .then_some(())
    .ok_or_else(|| "managed Vault ciphertext route is stale".to_owned())
}

pub(crate) fn relay_kernel_authorized_route(
    store: &SqliteControlStore,
    data_dir: &Path,
    relay: &dyn ManagedRuntimeRelay,
    mut route: VaultCiphertextRouteV1,
) -> Result<VaultCiphertextResponseV1, String> {
    validate_vault_ciphertext_route_v1(&route)
        .map_err(|_| "managed Vault ciphertext route is invalid".to_owned())?;
    let vault = launch::current_launch(store)?;
    if route.vault_runtime_generation != vault.runtime_generation() {
        return Err("managed Vault ciphertext route is stale".to_owned());
    }
    ciphertext_route::sign_for_kernel(data_dir, store.snapshot().instance_id(), &mut route)?;
    let request = ManagedVaultRuntimeControlRequestV1 {
        operation: Some(Operation::CiphertextRoute(route.clone())),
    };
    let response = relay.relay(VAULT_PROCESS_ID, request.encode_to_vec())?;
    let response = ManagedVaultRuntimeControlResponseV1::decode(response.as_slice())
        .map_err(|_| "managed Vault ciphertext response is invalid".to_owned())?;
    if !response.error_code.is_empty() {
        if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
            eprintln!(
                "developer_vault_ciphertext_response_error_code={}",
                response.error_code
            );
        }
        return Err("managed Vault ciphertext response is unavailable".to_owned());
    }
    let response = match response.result {
        Some(ResponseResult::CiphertextResponse(response)) => response,
        _ => return Err("managed Vault ciphertext response is invalid".to_owned()),
    };
    ciphertext_route::validate_response(&route, response)
}
