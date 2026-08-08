//! Narrow Kernel-mediated Vault route; Blob never contacts Vault directly.

use std::future::Future;

use makosh_runtime_protocol::v1::{VaultCiphertextResponseV1, VaultCiphertextRouteV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobVaultRouteFailureV1 {
    Rejected,
    Unavailable,
}

pub trait BlobVaultRoutePortV1 {
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, BlobVaultRouteFailureV1>> + Send;
}
