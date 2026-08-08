//! Durable Storage credential fencing for opaque external Vault routes.

use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::VaultCiphertextRouteV1;

pub(crate) fn validate_vault_credential_fence(
    store: &SqliteControlStore,
    registration_id: &str,
    runtime_id: &str,
    runtime_generation: u64,
    grant_epoch: u64,
    route: &VaultCiphertextRouteV1,
) -> Result<(), String> {
    let active = store
        .platform_storage_bindings()
        .map_err(|error| format!("{error:?}"))?
        .into_iter()
        .filter(|binding| {
            binding.state() == PlatformStorageBindingStateV1::Active
                && binding.registration_id() == registration_id
                && binding.runtime_instance_id() == runtime_id
                && binding.runtime_generation() == runtime_generation
                && binding.grant_epoch() == grant_epoch
        })
        .collect::<Vec<_>>();
    let carries_storage_fence = route.storage_role_epoch != 0
        || route.storage_credential_lease_revision != 0
        || !route.storage_runtime_principal.is_empty();
    if active.is_empty() {
        return (!carries_storage_fence)
            .then_some(())
            .ok_or_else(|| "Storage credential route is unauthorized".to_owned());
    }
    active
        .iter()
        .any(|binding| {
            binding.role_epoch() == route.storage_role_epoch
                && binding.credential_lease_revision() == route.storage_credential_lease_revision
                && binding.runtime_principal() == route.storage_runtime_principal
        })
        .then_some(())
        .ok_or_else(|| "Storage credential route is stale or unauthorized".to_owned())
}
