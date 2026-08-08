//! Storage revoke command over the shared encrypted Vault session.

use makosh_storage_control::{StorageFenceOutcomeV1, StorageVaultLeasePortV1};
use makosh_storage_protocol::StorageBindingV1;
use makosh_vault_protocol::VaultTransportCommandV1;

use super::{StorageVaultLeaseAdapterV1, StorageVaultRoutePortV1, session};

impl<T> StorageVaultLeaseAdapterV1<T>
where
    T: StorageVaultRoutePortV1 + Send,
{
    pub async fn invalidate(&mut self, binding: &StorageBindingV1) -> StorageFenceOutcomeV1 {
        let prepared = match session::prepare_storage_credential(
            binding,
            &self.context,
            &VaultTransportCommandV1::RevokeAudience,
        ) {
            Ok(prepared) => prepared,
            Err(()) => return StorageFenceOutcomeV1::Rejected,
        };
        match session::execute(&mut self.route_port, prepared).await {
            Ok(value) if value.as_slice() == [1] => StorageFenceOutcomeV1::Applied,
            Ok(_) | Err(super::StorageVaultRouteFailureV1::Rejected) => {
                StorageFenceOutcomeV1::Rejected
            }
            Err(super::StorageVaultRouteFailureV1::Unavailable) => {
                StorageFenceOutcomeV1::Unavailable
            }
        }
    }
}

impl<T> StorageVaultLeasePortV1 for StorageVaultLeaseAdapterV1<T>
where
    T: StorageVaultRoutePortV1 + Send,
{
    fn invalidate_lease(
        &mut self,
        binding: &StorageBindingV1,
    ) -> impl std::future::Future<Output = StorageFenceOutcomeV1> + Send {
        self.invalidate(binding)
    }
}
