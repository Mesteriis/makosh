//! Kernel-inherited ciphertext-only Vault route for managed module runtimes.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, RejectManagedControlRequestsV2,
};
use makosh_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeVaultRouteRequestV1, VaultCiphertextResponseV1, VaultCiphertextRouteV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;

use super::{StorageVaultRouteFailureV1, StorageVaultRoutePortV1};

const MAX_FRAME_BYTES: usize = 512 * 1024;

pub struct InheritedKernelVaultRouteV1 {
    channel: UnixStream,
}

/// The V2 counterpart owns the correlated inherited control channel and keeps
/// Vault ciphertext routing typed; it cannot transport an opaque relay frame.
pub struct InheritedKernelVaultRouteV2 {
    channel: ManagedControlChannelV2<UnixStream>,
}

impl InheritedKernelVaultRouteV2 {
    #[must_use]
    pub fn new(channel: ManagedControlChannelV2<UnixStream>) -> Self {
        Self { channel }
    }

    pub fn into_channel(self) -> ManagedControlChannelV2<UnixStream> {
        self.channel
    }

    pub fn route(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1> {
        let mut bootstrap_dispatcher = RejectManagedControlRequestsV2;
        let response = self
            .channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::RouteVaultCiphertext(
                        ManagedRuntimeVaultRouteRequestV1 { route: Some(route) },
                    )),
                },
                &mut bootstrap_dispatcher,
            )
            .map_err(|_| StorageVaultRouteFailureV1::Unavailable)?;
        let response = response
            .result
            .and_then(|result| match result {
                ControlResult::VaultRoute(response) => Some(response),
                _ => None,
            })
            .ok_or(StorageVaultRouteFailureV1::Rejected)?;
        response
            .response
            .filter(|_| response.error_code.is_empty())
            .ok_or(StorageVaultRouteFailureV1::Rejected)
    }
}

impl InheritedKernelVaultRouteV1 {
    #[must_use]
    pub fn new(channel: UnixStream) -> Self {
        Self { channel }
    }

    pub fn route(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1> {
        write_frame(
            &mut self.channel,
            &ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::RouteVaultCiphertext(
                    ManagedRuntimeVaultRouteRequestV1 { route: Some(route) },
                )),
            }
            .encode_to_vec(),
        )?;
        let response =
            ManagedRuntimeControlResponseV1::decode(read_frame(&mut self.channel)?.as_slice())
                .map_err(|_| StorageVaultRouteFailureV1::Rejected)?
                .result
                .and_then(|result| match result {
                    ControlResult::VaultRoute(response) => Some(response),
                    _ => None,
                })
                .ok_or(StorageVaultRouteFailureV1::Rejected)?;
        response
            .response
            .filter(|_| response.error_code.is_empty())
            .ok_or(StorageVaultRouteFailureV1::Rejected)
    }
}

impl StorageVaultRoutePortV1 for InheritedKernelVaultRouteV1 {
    #[allow(clippy::manual_async_fn)]
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl std::future::Future<
        Output = Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1>,
    > + Send {
        async move { self.route(route) }
    }
}

impl StorageVaultRoutePortV1 for InheritedKernelVaultRouteV2 {
    #[allow(clippy::manual_async_fn)]
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl std::future::Future<
        Output = Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1>,
    > + Send {
        async move { self.route(route) }
    }
}

fn write_frame(channel: &mut UnixStream, bytes: &[u8]) -> Result<(), StorageVaultRouteFailureV1> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err(StorageVaultRouteFailureV1::Rejected);
    }
    let mut length =
        u32::try_from(bytes.len()).map_err(|_| StorageVaultRouteFailureV1::Rejected)?;
    let mut prefix = Vec::with_capacity(5);
    while length >= 0x80 {
        prefix.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    prefix.push(length as u8);
    channel
        .write_all(&prefix)
        .and_then(|_| channel.write_all(bytes))
        .and_then(|_| channel.flush())
        .map_err(|_| StorageVaultRouteFailureV1::Unavailable)
}

fn read_frame(channel: &mut UnixStream) -> Result<Vec<u8>, StorageVaultRouteFailureV1> {
    let length =
        usize::try_from(read_varint(channel)?).map_err(|_| StorageVaultRouteFailureV1::Rejected)?;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(StorageVaultRouteFailureV1::Rejected);
    }
    let mut bytes = vec![0_u8; length];
    channel
        .read_exact(&mut bytes)
        .map_err(|_| StorageVaultRouteFailureV1::Unavailable)?;
    Ok(bytes)
}

fn read_varint(channel: &mut UnixStream) -> Result<u64, StorageVaultRouteFailureV1> {
    let mut value = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        channel
            .read_exact(&mut byte)
            .map_err(|_| StorageVaultRouteFailureV1::Unavailable)?;
        value |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(StorageVaultRouteFailureV1::Rejected)
}
