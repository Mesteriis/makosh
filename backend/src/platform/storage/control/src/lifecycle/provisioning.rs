//! Ordered, credential-free orchestration before a binding can become active.

use makosh_storage_protocol::StorageBindingV1;

use super::{StorageLifecycleErrorV1, StorageLifecycleV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageProvisioningErrorV1 {
    Role,
    Migration,
    Lease,
    Pool,
    Lifecycle(StorageLifecycleErrorV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageProvisioningFailureV1 {
    error: StorageProvisioningErrorV1,
    partial_fence_applied: bool,
}

impl StorageProvisioningFailureV1 {
    pub const fn error(self) -> StorageProvisioningErrorV1 {
        self.error
    }
    pub const fn partial_fence_applied(self) -> bool {
        self.partial_fence_applied
    }
}

pub trait StorageProvisioningPortV1 {
    fn ensure_role(&mut self, binding: &StorageBindingV1)
    -> Result<(), StorageProvisioningErrorV1>;
    fn apply_migrations_and_privileges(
        &mut self,
        binding: &StorageBindingV1,
    ) -> Result<(), StorageProvisioningErrorV1>;
    fn issue_credential_lease(
        &mut self,
        binding: &StorageBindingV1,
    ) -> Result<(), StorageProvisioningErrorV1>;
    fn publish_pool(
        &mut self,
        binding: &StorageBindingV1,
    ) -> Result<(), StorageProvisioningErrorV1>;
    fn fence_partial_binding(&mut self, binding: &StorageBindingV1) -> bool;
}

#[derive(Default)]
pub struct StorageProvisionerV1;

impl StorageProvisionerV1 {
    pub fn provision(
        &self,
        port: &mut impl StorageProvisioningPortV1,
        lifecycle: &mut StorageLifecycleV1,
        binding: &StorageBindingV1,
    ) -> Result<(), StorageProvisioningFailureV1> {
        let result = provision_steps(port, lifecycle, binding);
        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(StorageProvisioningFailureV1 {
                error,
                partial_fence_applied: port.fence_partial_binding(binding),
            }),
        }
    }
}

fn provision_steps(
    port: &mut impl StorageProvisioningPortV1,
    lifecycle: &mut StorageLifecycleV1,
    binding: &StorageBindingV1,
) -> Result<(), StorageProvisioningErrorV1> {
    port.ensure_role(binding)?;
    port.apply_migrations_and_privileges(binding)?;
    port.issue_credential_lease(binding)?;
    port.publish_pool(binding)?;
    lifecycle
        .activate(binding.clone())
        .map_err(StorageProvisioningErrorV1::Lifecycle)
}
