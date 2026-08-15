//! Vault runtime loop served exclusively over the inherited Kernel channel.

use prost::Message;

use makosh_runtime_protocol::v1::{
    GetVaultRuntimeStatusRequestV1, ManagedVaultRuntimeControlRequestV1,
    ManagedVaultRuntimeControlResponseV1, VaultRuntimeStateV1, VaultRuntimeStatusV1,
    managed_vault_runtime_control_request_v1::Operation,
    managed_vault_runtime_control_response_v1::Result as ResponseResult,
};
use makosh_runtime_protocol::validation::vault::VAULT_SECRET_UNAVAILABLE_ERROR_CODE;
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
    let span = tracing::info_span!(
        "vault.control",
        runtime.generation = service.runtime_generation(),
    );
    let _entered = span.enter();
    tracing::info!(event = "vault.control.ready");
    let mut replay_guard = VaultTransportReplayGuard::new(service.runtime_generation());
    loop {
        let frame = read_frame(&mut channel)?;
        let request = ManagedVaultRuntimeControlRequestV1::decode(frame.as_slice())
            .map_err(|_| "Vault inherited control frame is invalid".to_owned())?;
        let is_status_probe = matches!(request.operation, Some(Operation::GetStatus(_)));
        if is_status_probe {
            tracing::trace!(
                event = "vault.control.status_probe.received",
                control.operation = operation_name(&request),
                payload.frame_bytes = frame.len(),
            );
        } else {
            tracing::debug!(
                event = "vault.control.request.received",
                control.operation = operation_name(&request),
                payload.frame_bytes = frame.len(),
            );
        }
        let response = response_for(
            request,
            service,
            keys,
            &mut replay_guard,
            authorization_key_sec1,
        )
        .unwrap_or_else(|error| {
            if error == VAULT_SECRET_UNAVAILABLE_ERROR_CODE {
                tracing::debug!(
                    event = "vault.control.secret_unavailable",
                    error.code = VAULT_SECRET_UNAVAILABLE_ERROR_CODE,
                );
                return error_response(VAULT_SECRET_UNAVAILABLE_ERROR_CODE);
            }
            tracing::warn!(
                event = "vault.control.operation.denied",
                error.class = "vault_operation_denied",
                error.message = %error,
            );
            if tracing::enabled!(tracing::Level::DEBUG) {
                return error_response(&format!("developer_denied_{error}"));
            }
            error_response("operation_denied")
        });
        if is_status_probe {
            tracing::trace!(
                event = "vault.control.status_probe.ready",
                response.error_code = %response.error_code,
                response.has_result = response.result.is_some(),
            );
        } else {
            tracing::debug!(
                event = "vault.control.response.ready",
                response.error_code = %response.error_code,
                response.has_result = response.result.is_some(),
            );
        }
        write_frame(&mut channel, &response.encode_to_vec())?;
    }
}

fn operation_name(request: &ManagedVaultRuntimeControlRequestV1) -> &'static str {
    match request.operation {
        Some(Operation::GetStatus(_)) => "get_status",
        Some(Operation::CiphertextRoute(_)) => "ciphertext_route",
        None => "unavailable",
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
