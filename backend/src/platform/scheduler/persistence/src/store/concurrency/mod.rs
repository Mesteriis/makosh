use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::{ConcurrencyKeyV1, SchedulePolicyV1};
use sqlx::{Postgres, Transaction, query};

use super::runs::{SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1};

impl SchedulerPostgresStoreV1 {
    /// Creates a bounded coordination slot or updates its inactive limit.
    pub async fn ensure_concurrency_slot(
        &self,
        key: &ConcurrencyKeyV1,
        policy: &SchedulePolicyV1,
        now: UtcMillisV1,
    ) -> Result<(), SchedulerRunClaimErrorV1> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
        ensure_slot(&mut transaction, key, policy, now).await?;
        transaction
            .commit()
            .await
            .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)
    }
}

pub(super) async fn ensure_slot(
    transaction: &mut Transaction<'_, Postgres>,
    key: &ConcurrencyKeyV1,
    policy: &SchedulePolicyV1,
    now: UtcMillisV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let configured = query(
        "INSERT INTO makosh_platform.scheduler_concurrency (concurrency_key, active_runs, max_parallelism, updated_at_unix_ms) VALUES ($1, 0, $2, $3) ON CONFLICT (concurrency_key) DO UPDATE SET max_parallelism = EXCLUDED.max_parallelism, updated_at_unix_ms = EXCLUDED.updated_at_unix_ms WHERE makosh_platform.scheduler_concurrency.active_runs = 0 OR makosh_platform.scheduler_concurrency.max_parallelism = EXCLUDED.max_parallelism",
    )
    .bind(key.value())
    .bind(i32::from(policy.max_parallelism()))
    .bind(now.value())
    .execute(&mut **transaction)
    .await
    .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
    (configured.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::ConcurrencyBusy)
}
