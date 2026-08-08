//! Kernel-to-Storage Control admission for one already durable binding.

use makosh_kernel_control_store::PlatformStorageBindingV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_storage_protocol::{
    v1::{
        ApplyStorageBindingRequestV1, StorageRuntimeControlRequestV1,
        StorageRuntimeControlResponseV1, storage_runtime_control_request_v1::Operation,
        storage_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::validate_storage_runtime_control_response,
};
use prost::Message;
use std::time::Duration;

use super::{binding::STORAGE_PROCESS_ID, topology};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

/// Storage remains the sole role/migration/Vault/pool owner. Kernel transports
/// only the exact durable binding and its canonical bundle over the authenticated
/// inherited channel, then requires the matching active acknowledgement.
pub(crate) fn apply_reserved_binding(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    binding: &PlatformStorageBindingV1,
) -> Result<(), String> {
    supervisor
        .is_active(STORAGE_PROCESS_ID)?
        .then_some(())
        .ok_or_else(|| "Storage runtime is unavailable for Scheduler binding".to_owned())?;
    let runtime_topology = topology::to_runtime(&topology::current(store)?)?;
    let expected = topology::to_runtime_binding(&runtime_topology, binding)?;
    let bundle = store
        .platform_storage_bundle(binding.owner_id(), binding.storage_bundle_revision())
        .map_err(|_| "Scheduler Storage bundle is unavailable".to_owned())?
        .ok_or_else(|| "Scheduler Storage bundle is unavailable".to_owned())?;
    let request = StorageRuntimeControlRequestV1 {
        operation: Some(Operation::ApplyBinding(ApplyStorageBindingRequestV1 {
            binding: Some(expected.clone()),
            bundle: Some(topology::to_runtime_bundle(&bundle)?),
        })),
    };
    let response = supervisor.relay_with_timeout(
        STORAGE_PROCESS_ID,
        request.encode_to_vec(),
        Duration::from_secs(30),
    )?;
    let response = StorageRuntimeControlResponseV1::decode(response.as_slice())
        .map_err(|_| "managed Storage binding response is invalid".to_owned())?;
    validate_storage_runtime_control_response(&response)
        .map_err(|_| "managed Storage binding response is invalid".to_owned())?;
    let error_code = response.error_code;
    match response.result {
        Some(ResponseResult::ActiveBinding(actual))
            if error_code.is_empty() && actual == expected =>
        {
            Ok(())
        }
        _ if !error_code.is_empty() => Err(format!(
            "Storage runtime rejected Scheduler binding: {error_code}"
        )),
        _ => Err("Storage runtime did not provision Scheduler binding".to_owned()),
    }
}
