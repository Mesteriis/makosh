use std::future::Future;

use makosh_runtime_protocol::v1::{VaultCiphertextResponseV1, VaultCiphertextRouteV1};

use crate::storage_runtime_vault::{StorageVaultRouteFailureV1, StorageVaultRoutePortV1};

pub(super) struct UnavailableVaultRoute;

impl StorageVaultRoutePortV1 for UnavailableVaultRoute {
    #[allow(clippy::manual_async_fn)] // The route port requires a Send future.
    fn route_vault_ciphertext(
        &mut self,
        _: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, StorageVaultRouteFailureV1>> + Send
    {
        async { Err(StorageVaultRouteFailureV1::Unavailable) }
    }
}
