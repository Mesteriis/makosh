//! Credential-free ports for infrastructure-specific revoke adapters.

use makosh_storage_protocol::StorageBindingV1;
use std::future::Future;

use super::StoragePoolFenceCommandV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageFenceOutcomeV1 {
    Applied,
    Rejected,
    Unavailable,
}

pub trait StorageVaultLeasePortV1 {
    fn invalidate_lease(
        &mut self,
        binding: &StorageBindingV1,
    ) -> impl Future<Output = StorageFenceOutcomeV1> + Send;
}

pub trait StoragePoolFencePortV1 {
    fn apply_pool_fence(
        &mut self,
        binding: &StorageBindingV1,
        command: StoragePoolFenceCommandV1,
    ) -> impl Future<Output = StorageFenceOutcomeV1> + Send;
}

pub trait StoragePostgresFencePortV1 {
    fn fence_runtime_role(
        &mut self,
        binding: &StorageBindingV1,
    ) -> impl Future<Output = StorageFenceOutcomeV1> + Send;
}
