//! Reads the sanitized status of the exact live managed Vault runtime.

use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    GetVaultRuntimeStatusRequestV1, ManagedVaultRuntimeControlRequestV1,
    ManagedVaultRuntimeControlResponseV1, VaultRuntimeStateV1, VaultRuntimeStatusV1,
    managed_vault_runtime_control_request_v1::Operation,
    managed_vault_runtime_control_response_v1::Result as ResponseResult,
};
use makosh_runtime_protocol::validation::vault::validate_vault_runtime_status_v1;
use prost::Message;

use crate::platform::vault::{binding::VAULT_PROCESS_ID, launch};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort;

pub struct ManagedVaultStatus {
    runtime_generation: u64,
    hpke_public_key_x25519: [u8; 32],
}

impl ManagedVaultStatus {
    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }

    #[must_use]
    pub const fn hpke_public_key_x25519(&self) -> &[u8; 32] {
        &self.hpke_public_key_x25519
    }
}

pub fn read_current(
    store: &SqliteControlStore,
    relay: &ManagedRuntimeRelayPort,
) -> Result<ManagedVaultStatus, String> {
    let launch = launch::current_launch(store)?;
    let request = ManagedVaultRuntimeControlRequestV1 {
        operation: Some(Operation::GetStatus(GetVaultRuntimeStatusRequestV1 {})),
    };
    let response = relay.relay(VAULT_PROCESS_ID, request.encode_to_vec())?;
    let response = ManagedVaultRuntimeControlResponseV1::decode(response.as_slice())
        .map_err(|_| "managed Vault status response is invalid".to_owned())?;
    parse_current(response, launch.runtime_generation())
}

pub(crate) fn parse_current(
    response: ManagedVaultRuntimeControlResponseV1,
    expected_generation: u64,
) -> Result<ManagedVaultStatus, String> {
    if !response.error_code.is_empty() {
        return Err("managed Vault status is unavailable".to_owned());
    }
    let status = match response.result {
        Some(ResponseResult::Status(status)) => status,
        _ => return Err("managed Vault status response is invalid".to_owned()),
    };
    validate_current_status(status, expected_generation)
}

fn validate_current_status(
    status: VaultRuntimeStatusV1,
    expected_generation: u64,
) -> Result<ManagedVaultStatus, String> {
    validate_vault_runtime_status_v1(&status)
        .map_err(|_| "managed Vault status response is invalid".to_owned())?;
    if VaultRuntimeStateV1::try_from(status.state).ok() != Some(VaultRuntimeStateV1::Ready)
        || status.vault_runtime_generation != expected_generation
    {
        return Err("managed Vault status is stale or unavailable".to_owned());
    }
    let hpke_public_key_x25519 = status
        .hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| "managed Vault status response is invalid".to_owned())?;
    Ok(ManagedVaultStatus {
        runtime_generation: status.vault_runtime_generation,
        hpke_public_key_x25519,
    })
}
