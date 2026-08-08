use makosh_storage_protocol::StorageBindingV1;
use makosh_storage_vault::{StorageVaultLeaseAdapterV1, StorageVaultRoutePortV1};

use crate::{StorageFenceOutcomeV1, StorageVaultLeasePortV1};

impl<T> StorageVaultLeasePortV1 for StorageVaultLeaseAdapterV1<T>
where
    T: StorageVaultRoutePortV1 + Send,
{
    #[allow(clippy::manual_async_fn)]
    fn invalidate_lease(
        &mut self,
        binding: &StorageBindingV1,
    ) -> impl std::future::Future<Output = StorageFenceOutcomeV1> + Send {
        async move {
            match self.revoke_runtime_credential(binding).await {
                Ok(()) => StorageFenceOutcomeV1::Applied,
                Err(makosh_storage_vault::StorageVaultRouteFailureV1::Rejected) => {
                    StorageFenceOutcomeV1::Rejected
                }
                Err(makosh_storage_vault::StorageVaultRouteFailureV1::Unavailable) => {
                    StorageFenceOutcomeV1::Unavailable
                }
            }
        }
    }
}
