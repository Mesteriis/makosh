//! Verifies the sanitized status of the exact live Blob managed child.

use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    v1::{
        BlobRuntimeControlRequestV1, BlobRuntimeControlResponseV1, BlobRuntimeStateV1,
        GetBlobRuntimeStatusRequestV1, blob_runtime_control_request_v1::Operation,
        blob_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::blob::validate_blob_runtime_control_response,
};
use prost::Message;

use crate::platform::blob::{binding::BLOB_PROCESS_ID, launch};
use crate::platform::vault::status as vault_status;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort;

pub(crate) struct ManagedBlobStatus {
    runtime_generation: u64,
}

impl ManagedBlobStatus {
    #[must_use]
    pub(crate) const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }
}

pub(crate) fn read_current(
    store: &SqliteControlStore,
    relay: &ManagedRuntimeRelayPort,
) -> Result<ManagedBlobStatus, String> {
    if !relay.is_ready(BLOB_PROCESS_ID)? {
        return Err("managed Blob is unavailable".to_owned());
    }
    let launch = launch::current_launch(store)?;
    let vault = vault_status::read_current(store, relay)?;
    let request = BlobRuntimeControlRequestV1 {
        operation: Some(Operation::GetStatus(GetBlobRuntimeStatusRequestV1 {})),
    };
    let response = relay.relay(BLOB_PROCESS_ID, request.encode_to_vec())?;
    let response = BlobRuntimeControlResponseV1::decode(response.as_slice())
        .map_err(|_| "managed Blob status response is invalid".to_owned())?;
    validate_blob_runtime_control_response(&response)
        .map_err(|_| "managed Blob status response is invalid".to_owned())?;
    let status = match response.result {
        Some(ResponseResult::Status(status)) if response.error_code.is_empty() => status,
        _ => return Err("managed Blob status is unavailable".to_owned()),
    };
    if BlobRuntimeStateV1::try_from(status.state).ok() != Some(BlobRuntimeStateV1::Ready)
        || status.runtime_generation != launch.runtime_generation()
        || status.vault_runtime_generation != vault.runtime_generation()
    {
        return Err("managed Blob status is stale or unavailable".to_owned());
    }
    Ok(ManagedBlobStatus {
        runtime_generation: status.runtime_generation,
    })
}
