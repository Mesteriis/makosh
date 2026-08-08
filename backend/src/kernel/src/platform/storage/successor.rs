//! Fenced successor identity and Storage binding for a managed module runtime.

use makosh_kernel_control_store::{PlatformStorageBindingStateV1, PlatformStorageBindingV1};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::platform::macos::managed_launch::{self, ManagedLaunchReservation};
use crate::platform::storage::issuance::{StorageBindingIssueV1, issue_managed};
use crate::platform::storage::revocation;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

/// Fences a predecessor before reserving a fresh managed identity and Storage binding.
pub(crate) fn reserve(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    registration_id: &str,
    storage_capability_id: &str,
    issue: StorageBindingIssueV1,
) -> Result<(ManagedLaunchReservation, PlatformStorageBindingV1), String> {
    revoke_predecessor(supervisor, store, registration_id, storage_capability_id)?;
    let reservation = managed_launch::reserve(supervisor, store, registration_id)?;
    let binding = issue_managed(
        store,
        reservation.registration_id(),
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        storage_capability_id,
        issue,
    )?;
    Ok((reservation, binding))
}

pub(crate) fn issue_after(
    binding: &PlatformStorageBindingV1,
) -> Result<StorageBindingIssueV1, String> {
    issue_after_with_bundle(
        binding,
        binding.storage_bundle_revision(),
        *binding.storage_bundle_digest(),
    )
}

pub(crate) fn issue_after_with_bundle(
    binding: &PlatformStorageBindingV1,
    storage_bundle_revision: u64,
    storage_bundle_digest: [u8; 32],
) -> Result<StorageBindingIssueV1, String> {
    let role_epoch = binding
        .role_epoch()
        .checked_add(1)
        .ok_or_else(|| "Storage role epoch overflowed".to_owned())?;
    let credential_lease_revision = binding
        .credential_lease_revision()
        .checked_add(1)
        .ok_or_else(|| "Storage credential revision overflowed".to_owned())?;
    StorageBindingIssueV1::new(
        role_epoch,
        credential_lease_revision,
        storage_bundle_revision,
        storage_bundle_digest,
    )
}

fn revoke_predecessor(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    registration_id: &str,
    storage_capability_id: &str,
) -> Result<(), String> {
    let predecessor = store
        .platform_storage_binding(registration_id, storage_capability_id)
        .map_err(|_| "Storage binding is unavailable".to_owned())?;
    if let Some(predecessor) = predecessor {
        let revoking = match predecessor.state() {
            PlatformStorageBindingStateV1::Active => store
                .begin_platform_storage_binding_revocation(
                    registration_id,
                    storage_capability_id,
                    predecessor.binding_revision(),
                )
                .map_err(|_| "Storage binding cannot be reserved for revocation".to_owned())?,
            PlatformStorageBindingStateV1::Revoking => predecessor,
        };
        supervisor.request_stop_if_active(registration_id)?;
        revocation::fence_reserved_binding(supervisor, store, &revoking)?;
    }
    supervisor.stop_if_active(registration_id)?;
    Ok(())
}
