//! Vault runtime loop served exclusively over the inherited Kernel channel.

use prost::Message;

use makosh_runtime_protocol::v1::{
    GetVaultRuntimeStatusRequestV1, ManagedVaultRuntimeControlRequestV1,
    ManagedVaultRuntimeControlResponseV1, VaultRuntimeStateV1, VaultRuntimeStatusV1,
    managed_vault_runtime_control_request_v1::Operation,
    managed_vault_runtime_control_response_v1::Result as ResponseResult,
};
use makosh_runtime_protocol::validation::vault::validate_vault_runtime_status_v1;

use crate::control::inherited::{open_and_describe, read_frame, write_frame};
use crate::service::runtime::VaultService;
use crate::transport::keys::VaultTransportKeyPair;
use crate::transport::route::execute_route;
use crate::transport::session::VaultTransportReplayGuard;

#[allow(dead_code)] // Used by the inherited-channel composition harness.
pub fn serve(
    service: &mut VaultService,
    keys: &VaultTransportKeyPair,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    authorization_key_sec1: [u8; 65],
) -> Result<(), String> {
    let channel = open_and_describe(descriptor_bytes, settings_schema_bytes)?;
    serve_on_channel(channel, service, keys, authorization_key_sec1)
}

#[allow(dead_code)] // Used by the inherited-channel composition harness.
pub(crate) fn serve_on_channel(
    mut channel: std::os::unix::net::UnixStream,
    service: &mut VaultService,
    keys: &VaultTransportKeyPair,
    authorization_key_sec1: [u8; 65],
) -> Result<(), String> {
    let mut replay_guard = VaultTransportReplayGuard::new(service.runtime_generation());
    loop {
        let request =
            ManagedVaultRuntimeControlRequestV1::decode(read_frame(&mut channel)?.as_slice())
                .map_err(|_| "Vault inherited control frame is invalid".to_owned())?;
        let response = response_for(
            request,
            service,
            keys,
            &mut replay_guard,
            authorization_key_sec1,
        )
        .unwrap_or_else(|error| {
            if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_vault_operation_denied={error}");
                return error_response(&format!("developer_denied_{error}"));
            }
            error_response("operation_denied")
        });
        write_frame(&mut channel, &response.encode_to_vec())?;
    }
}

pub(crate) fn response_for(
    request: ManagedVaultRuntimeControlRequestV1,
    service: &mut VaultService,
    keys: &VaultTransportKeyPair,
    replay_guard: &mut VaultTransportReplayGuard,
    authorization_key_sec1: [u8; 65],
) -> Result<ManagedVaultRuntimeControlResponseV1, String> {
    match request.operation {
        Some(Operation::GetStatus(GetVaultRuntimeStatusRequestV1 {})) => {
            let status = VaultRuntimeStatusV1 {
                state: VaultRuntimeStateV1::Ready as i32,
                vault_runtime_generation: service.runtime_generation(),
                hpke_public_key_x25519: keys.public_key().as_bytes().to_vec(),
                blocker_code: String::new(),
            };
            validate_vault_runtime_status_v1(&status)
                .map_err(|_| "Vault inherited status is invalid".to_owned())?;
            Ok(ManagedVaultRuntimeControlResponseV1 {
                result: Some(ResponseResult::Status(status)),
                error_code: String::new(),
            })
        }
        Some(Operation::CiphertextRoute(route)) => execute_route(
            service,
            keys,
            replay_guard,
            authorization_key_sec1,
            route,
            unix_seconds()?,
        )
        .map(|response| ManagedVaultRuntimeControlResponseV1 {
            result: Some(ResponseResult::CiphertextResponse(response)),
            error_code: String::new(),
        }),
        None => Ok(error_response("operation_not_available")),
    }
}

fn error_response(error_code: &str) -> ManagedVaultRuntimeControlResponseV1 {
    ManagedVaultRuntimeControlResponseV1 {
        result: None,
        error_code: error_code.to_owned(),
    }
}

fn unix_seconds() -> Result<u64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "Vault clock is unavailable".to_owned())
}
