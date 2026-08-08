//! Restricts PgBouncer administration to commands derived from a fenced binding.

use std::future::Future;

use makosh_storage_control::{
    StorageFenceOutcomeV1, StoragePoolFenceCommandV1, StoragePoolFencePortV1,
};
use makosh_storage_protocol::StorageBindingV1;

use crate::{PoolAliasV1, PoolLifecycleCommandV1, PoolLifecycleOutcomeV1};

pub trait PgBouncerAdminPortV1 {
    fn execute_pool_command(
        &mut self,
        command: &str,
    ) -> impl Future<Output = PoolLifecycleOutcomeV1> + Send;
}

pub struct PgBouncerPoolFenceAdapterV1<T> {
    admin: T,
    pool_is_configured: bool,
}

impl<T> PgBouncerPoolFenceAdapterV1<T> {
    pub const fn new(admin: T) -> Self {
        Self {
            admin,
            pool_is_configured: true,
        }
    }

    pub const fn for_configuration_state(admin: T, pool_is_configured: bool) -> Self {
        Self {
            admin,
            pool_is_configured,
        }
    }

    pub fn into_inner(self) -> T {
        self.admin
    }
}

impl<T: PgBouncerAdminPortV1 + Send> StoragePoolFencePortV1 for PgBouncerPoolFenceAdapterV1<T> {
    fn apply_pool_fence(
        &mut self,
        binding: &StorageBindingV1,
        command: StoragePoolFenceCommandV1,
    ) -> impl Future<Output = StorageFenceOutcomeV1> + Send {
        let command = render_command(binding, command);
        async move {
            if !self.pool_is_configured {
                return StorageFenceOutcomeV1::Applied;
            }
            match command {
                Some(command) => map_outcome(self.admin.execute_pool_command(&command).await),
                None => StorageFenceOutcomeV1::Rejected,
            }
        }
    }
}

fn render_command(
    binding: &StorageBindingV1,
    command: StoragePoolFenceCommandV1,
) -> Option<String> {
    let alias = PoolAliasV1::from_binding(binding).ok()?;
    command_for(command).render(&alias).ok()
}

const fn command_for(command: StoragePoolFenceCommandV1) -> PoolLifecycleCommandV1 {
    match command {
        StoragePoolFenceCommandV1::Pause => PoolLifecycleCommandV1::Pause,
        StoragePoolFenceCommandV1::Disable => PoolLifecycleCommandV1::Disable,
        StoragePoolFenceCommandV1::Kill => PoolLifecycleCommandV1::Kill,
    }
}

const fn map_outcome(outcome: PoolLifecycleOutcomeV1) -> StorageFenceOutcomeV1 {
    match outcome {
        PoolLifecycleOutcomeV1::Applied => StorageFenceOutcomeV1::Applied,
        PoolLifecycleOutcomeV1::Rejected => StorageFenceOutcomeV1::Rejected,
        PoolLifecycleOutcomeV1::Unavailable => StorageFenceOutcomeV1::Unavailable,
    }
}
