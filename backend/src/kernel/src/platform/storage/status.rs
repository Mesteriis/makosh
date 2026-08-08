//! Reads a status that is bound to the exact live Storage child generation.

use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_storage_protocol::{
    v1::{
        GetStorageRuntimeStatusRequestV1, StorageRuntimeControlRequestV1,
        StorageRuntimeControlResponseV1, StorageRuntimeStateV1, StorageRuntimeStatusV1,
        storage_runtime_control_request_v1::Operation,
        storage_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::validate_storage_runtime_control_response,
};
use prost::Message;

use crate::platform::storage::{binding::STORAGE_PROCESS_ID, launch, topology};
use crate::platform::vault::status as vault_status;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort;

const STARTUP_STATUS_ATTEMPTS: u8 = 40;
const STARTUP_STATUS_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(50);

pub struct ManagedStorageStatus {
    runtime_generation: u64,
    state: StorageRuntimeStateV1,
    topology_revision: u64,
}

struct CurrentStorageExpectation {
    runtime_generation: u64,
    topology_revision: u64,
    storage_generation: u64,
    vault_runtime_generation: u64,
}

impl ManagedStorageStatus {
    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }

    #[must_use]
    pub const fn state(&self) -> StorageRuntimeStateV1 {
        self.state
    }

    #[must_use]
    pub const fn topology_revision(&self) -> u64 {
        self.topology_revision
    }
}

pub fn read_current(
    store: &SqliteControlStore,
    relay: &ManagedRuntimeRelayPort,
) -> Result<ManagedStorageStatus, String> {
    let expectation = current_expectation(store, relay)?;
    read_bound_status(relay, &expectation)
}

pub(crate) fn wait_current(
    store: &SqliteControlStore,
    relay: &ManagedRuntimeRelayPort,
) -> Result<ManagedStorageStatus, String> {
    let expectation = current_expectation(store, relay)?;
    for attempt in 0..STARTUP_STATUS_ATTEMPTS {
        match read_bound_status(relay, &expectation) {
            Ok(status) => return Ok(status),
            Err(error)
                if attempt + 1 < STARTUP_STATUS_ATTEMPTS && retryable_startup_error(&error) =>
            {
                std::thread::sleep(STARTUP_STATUS_RETRY_DELAY);
            }
            Err(error) => return Err(error),
        }
    }
    Err("Storage runtime status is unavailable".to_owned())
}

fn current_expectation(
    store: &SqliteControlStore,
    relay: &ManagedRuntimeRelayPort,
) -> Result<CurrentStorageExpectation, String> {
    let launch = launch::current_launch(store)?;
    let topology = topology::current(store)?;
    let vault = vault_status::read_current(store, relay)?;
    Ok(CurrentStorageExpectation {
        runtime_generation: launch.runtime_generation(),
        topology_revision: topology.revision(),
        storage_generation: topology.storage_generation(),
        vault_runtime_generation: vault.runtime_generation(),
    })
}

fn read_bound_status(
    relay: &ManagedRuntimeRelayPort,
    expectation: &CurrentStorageExpectation,
) -> Result<ManagedStorageStatus, String> {
    let request = StorageRuntimeControlRequestV1 {
        operation: Some(Operation::GetStatus(GetStorageRuntimeStatusRequestV1 {})),
    };
    let response = relay.relay(STORAGE_PROCESS_ID, request.encode_to_vec())?;
    let response = StorageRuntimeControlResponseV1::decode(response.as_slice())
        .map_err(|_| "managed Storage status response is invalid".to_owned())?;
    parse_current(
        response,
        expectation.runtime_generation,
        expectation.topology_revision,
        expectation.storage_generation,
        expectation.vault_runtime_generation,
    )
}

fn retryable_startup_error(error: &str) -> bool {
    matches!(
        error,
        "managed runtime relay is unavailable" | "managed runtime relay timed out"
    )
}

pub(crate) fn parse_current(
    response: StorageRuntimeControlResponseV1,
    expected_generation: u64,
    expected_topology_revision: u64,
    expected_storage_generation: u64,
    expected_vault_runtime_generation: u64,
) -> Result<ManagedStorageStatus, String> {
    validate_storage_runtime_control_response(&response)
        .map_err(|_| "managed Storage status response is invalid".to_owned())?;
    let status = match response.result {
        Some(ResponseResult::Status(status)) if response.error_code.is_empty() => status,
        _ => return Err("managed Storage status is unavailable".to_owned()),
    };
    validate_current_status(
        status,
        expected_generation,
        expected_topology_revision,
        expected_storage_generation,
        expected_vault_runtime_generation,
    )
}

fn validate_current_status(
    status: StorageRuntimeStatusV1,
    expected_generation: u64,
    expected_topology_revision: u64,
    expected_storage_generation: u64,
    expected_vault_runtime_generation: u64,
) -> Result<ManagedStorageStatus, String> {
    let state = StorageRuntimeStateV1::try_from(status.state)
        .map_err(|_| "managed Storage status response is invalid".to_owned())?;
    if status.runtime_generation != expected_generation
        || status.topology_revision != expected_topology_revision
        || status.storage_generation != expected_storage_generation
        || status.vault_runtime_generation != expected_vault_runtime_generation
        || !launchable_state(state)
    {
        return Err("managed Storage status is stale or unavailable".to_owned());
    }
    Ok(ManagedStorageStatus {
        runtime_generation: status.runtime_generation,
        state,
        topology_revision: status.topology_revision,
    })
}

const fn launchable_state(state: StorageRuntimeStateV1) -> bool {
    matches!(
        state,
        StorageRuntimeStateV1::Reconciling | StorageRuntimeStateV1::Ready
    )
}
