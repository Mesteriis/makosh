//! Kernel relay for a binding already reserved in the durable Control Store.

use makosh_kernel_control_store::PlatformStorageBindingV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_storage_protocol::{
    v1::{
        RevokeStorageBindingRequestV1, StorageRuntimeControlRequestV1,
        StorageRuntimeControlResponseV1, storage_runtime_control_request_v1::Operation,
        storage_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::validate_storage_runtime_control_response,
};
use prost::Message;

use super::{binding::STORAGE_PROCESS_ID, topology};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

pub(crate) fn fence_registration_bindings(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<(), String> {
    let bindings = store
        .begin_registration_storage_bindings_revocation(registration_id)
        .map_err(|_| "managed Storage revocation cannot be reserved".to_owned())?;
    if !bindings.is_empty() && !supervisor.is_active(STORAGE_PROCESS_ID)? {
        return Err("managed Storage revocation is incomplete".to_owned());
    }
    for binding in bindings {
        if fence_reserved_binding(supervisor, store, &binding).is_err() {
            return Err("managed Storage revocation is incomplete".to_owned());
        }
    }
    Ok(())
}

pub(crate) fn fence_reserved_binding(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    binding: &PlatformStorageBindingV1,
) -> Result<(), String> {
    if !supervisor.is_active(STORAGE_PROCESS_ID)? {
        return Ok(());
    }
    let topology = topology::to_runtime(&topology::current(store)?)?;
    let expected = topology::to_runtime_binding(&topology, binding)?;
    let request = StorageRuntimeControlRequestV1 {
        operation: Some(Operation::RevokeBinding(RevokeStorageBindingRequestV1 {
            binding: Some(expected.clone()),
        })),
    };
    let response = supervisor.relay(STORAGE_PROCESS_ID, request.encode_to_vec())?;
    let response = StorageRuntimeControlResponseV1::decode(response.as_slice())
        .map_err(|_| "managed Storage revocation response is invalid".to_owned())?;
    validate_storage_runtime_control_response(&response)
        .map_err(|_| "managed Storage revocation response is invalid".to_owned())?;
    if !response.error_code.is_empty() {
        return Err(format!(
            "managed Storage revocation rejected: {}",
            response.error_code
        ));
    }
    match response.result {
        Some(ResponseResult::RevokedBinding(actual)) if actual == expected => Ok(()),
        _ => Err("managed Storage revocation is incomplete".to_owned()),
    }
}
