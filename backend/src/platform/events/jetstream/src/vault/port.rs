//! Narrow Kernel-mediated route; the NATS adapter never contacts Vault directly.

use std::future::Future;

use makosh_runtime_protocol::v1::{VaultCiphertextResponseV1, VaultCiphertextRouteV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NatsVaultRouteFailureV1 {
    Rejected,
    Unavailable,
}

pub trait NatsVaultRoutePortV1 {
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, NatsVaultRouteFailureV1>> + Send;
}
