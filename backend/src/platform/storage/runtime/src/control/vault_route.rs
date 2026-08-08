//! Concrete ciphertext-only Vault relay over the inherited managed channel.

use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeVaultRouteRequestV1, VaultCiphertextResponseV1, VaultCiphertextRouteV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;

use crate::vault::{StorageVaultRouteFailureV1, StorageVaultRoutePortV1};

use super::framing::{read_frame, write_frame};

pub struct InheritedVaultRoutePortV1 {
    channel: UnixStream,
}

impl InheritedVaultRoutePortV1 {
    #[must_use]
    pub fn new(channel: UnixStream) -> Self {
        Self { channel }
    }
}

impl StorageVaultRoutePortV1 for InheritedVaultRoutePortV1 {
    #[allow(clippy::manual_async_fn)] // The public port must guarantee its returned future is Send.
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl std::future::Future<
        Output = Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1>,
    > + Send {
        async move { route_once(&mut self.channel, route) }
    }
}

fn route_once(
    channel: &mut UnixStream,
    route: VaultCiphertextRouteV1,
) -> Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1> {
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(Operation::RouteVaultCiphertext(
            ManagedRuntimeVaultRouteRequestV1 { route: Some(route) },
        )),
    };
    write_frame(channel, &request.encode_to_vec())
        .map_err(|_| StorageVaultRouteFailureV1::Unavailable)?;
    let response = ManagedRuntimeControlResponseV1::decode(
        read_frame(channel)
            .map_err(|_| StorageVaultRouteFailureV1::Unavailable)?
            .as_slice(),
    )
    .map_err(|_| StorageVaultRouteFailureV1::Rejected)?
    .result
    .and_then(|result| match result {
        ControlResult::VaultRoute(response) => Some(response),
        _ => None,
    })
    .ok_or(StorageVaultRouteFailureV1::Rejected)?;
    if !response.error_code.is_empty() {
        return Err(StorageVaultRouteFailureV1::Rejected);
    }
    response
        .response
        .ok_or(StorageVaultRouteFailureV1::Rejected)
}
