//! Narrow Kernel-mediated route port; Storage never contacts Vault directly.

use std::future::Future;

use makosh_runtime_protocol::v1::{VaultCiphertextResponseV1, VaultCiphertextRouteV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageVaultRouteFailureV1 {
    Rejected,
    Unavailable,
}

pub trait StorageVaultRoutePortV1 {
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1>> + Send;
}
