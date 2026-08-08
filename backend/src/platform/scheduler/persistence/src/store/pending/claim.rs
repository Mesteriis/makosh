use sqlx::{Postgres, Transaction, query, query_scalar};

use super::super::runs::{SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1, SchedulerRunClaimV1};
use super::super::runs::{reap_expired_in_transaction, reserve_concurrency_slot, reserve_run};

impl SchedulerPostgresStoreV1 {
    /// Atomically turns one durable pending fire into its fenced run lease.
    pub async fn claim_pending_fire(
        &self,
        claim: &SchedulerRunClaimV1,
    ) -> Result<(), SchedulerRunClaimErrorV1> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
        reap_expired_in_transaction(&mut transaction, claim.claimed_at()).await?;
        reserve_concurrency_slot(&mut transaction, claim).await?;
        verify_current_schedule(&mut transaction, claim).await?;
        delete_pending(&mut transaction, claim).await?;
        reserve_run(&mut transaction, claim).await?;
        transaction
            .commit()
            .await
            .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)
    }
}

async fn verify_current_schedule(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let found = query_scalar::<_, i32>(
        "SELECT 1 FROM makosh_platform.scheduler_schedules WHERE schedule_id = $1 AND schedule_revision = $2 AND enabled = TRUE AND concurrency_key = $3 AND max_parallelism = $4 AND policy_bytes = $5 FOR UPDATE",
    )
    .bind(claim.schedule_id().bytes().to_vec())
    .bind(i64::try_from(claim.schedule_revision().value()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
    .bind(claim.concurrency_key().value())
    .bind(i32::from(claim.max_parallelism()))
    .bind(claim.policy().canonical_bytes())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
    found
        .is_some()
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::Denied)
}

async fn delete_pending(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let deleted = query(
        "DELETE FROM makosh_platform.scheduler_pending_fires WHERE fire_key = $1 AND schedule_id = $2 AND schedule_revision = $3 AND scheduled_for_unix_ms = $4 AND concurrency_key = $5",
    )
    .bind(claim.fire_key().to_vec())
    .bind(claim.schedule_id().bytes().to_vec())
    .bind(i64::try_from(claim.schedule_revision().value()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
    .bind(claim.scheduled_for().value())
    .bind(claim.concurrency_key().value())
    .execute(&mut **transaction)
    .await
    .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
    (deleted.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::PendingMissing)
}
