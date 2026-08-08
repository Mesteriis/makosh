//! Coordinates every revocation boundary before a binding can be replaced.

use super::{
    StorageFenceOutcomeV1, StoragePoolFenceCommandV1, StoragePoolFencePortV1,
    StoragePostgresFencePortV1, StorageRevocationErrorV1, StorageRevocationReportV1,
    StorageVaultLeasePortV1,
};
use crate::StorageLifecycleV1;

#[derive(Default)]
pub struct StorageRevokerV1;

impl StorageRevokerV1 {
    pub async fn revoke(
        &self,
        lifecycle: &mut StorageLifecycleV1,
        vault: &mut impl StorageVaultLeasePortV1,
        pool: &mut impl StoragePoolFencePortV1,
        postgres: &mut impl StoragePostgresFencePortV1,
    ) -> Result<StorageRevocationReportV1, StorageRevocationErrorV1> {
        let binding = lifecycle
            .begin_revocation()
            .map_err(StorageRevocationErrorV1::Lifecycle)?
            .clone();
        let report = attempt_all_fences(&binding, vault, pool, postgres).await;
        if !report.is_complete() {
            return Err(StorageRevocationErrorV1::Incomplete(report));
        }
        lifecycle
            .complete_revocation()
            .map_err(StorageRevocationErrorV1::Lifecycle)?;
        Ok(report)
    }
}

async fn attempt_all_fences(
    binding: &makosh_storage_protocol::StorageBindingV1,
    vault: &mut impl StorageVaultLeasePortV1,
    pool: &mut impl StoragePoolFencePortV1,
    postgres: &mut impl StoragePostgresFencePortV1,
) -> StorageRevocationReportV1 {
    let vault_lease_invalidated = applied(vault.invalidate_lease(binding).await);
    let pool_paused = applied(
        pool.apply_pool_fence(binding, StoragePoolFenceCommandV1::Pause)
            .await,
    );
    let pool_disabled = applied(
        pool.apply_pool_fence(binding, StoragePoolFenceCommandV1::Disable)
            .await,
    );
    let pool_killed = applied(
        pool.apply_pool_fence(binding, StoragePoolFenceCommandV1::Kill)
            .await,
    );
    let postgres_role_fenced = applied(postgres.fence_runtime_role(binding).await);
    StorageRevocationReportV1::new(
        vault_lease_invalidated,
        pool_paused,
        pool_disabled,
        pool_killed,
        postgres_role_fenced,
    )
}

fn applied(outcome: StorageFenceOutcomeV1) -> bool {
    matches!(outcome, StorageFenceOutcomeV1::Applied)
}
