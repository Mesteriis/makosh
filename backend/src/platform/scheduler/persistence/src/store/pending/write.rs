use makosh_scheduler_protocol::OverlapPolicyV1;
use sqlx::{Postgres, Transaction, query, query_scalar};

use super::request::{
    SchedulerPendingFireErrorV1, SchedulerPendingFireOutcomeV1, SchedulerPendingFireV1,
};
use crate::store::runs::SchedulerPostgresStoreV1;

impl SchedulerPostgresStoreV1 {
    /// Persists the explicit non-start outcome for a due occurrence.
    pub async fn record_pending_fire(
        &self,
        fire: &SchedulerPendingFireV1,
    ) -> Result<SchedulerPendingFireOutcomeV1, SchedulerPendingFireErrorV1> {
        let mut transaction = self.pool().begin().await.map_err(unavailable)?;
        verify_schedule(&mut transaction, fire).await?;
        let outcome = match fire.policy().overlap() {
            OverlapPolicyV1::Queue { max_pending_runs } => {
                record_queued(&mut transaction, fire, max_pending_runs).await?
            }
            OverlapPolicyV1::CoalesceLatest => record_coalesced(&mut transaction, fire).await?,
            OverlapPolicyV1::Forbid | OverlapPolicyV1::AllowBounded { .. } => {
                SchedulerPendingFireOutcomeV1::Dropped
            }
        };
        transaction.commit().await.map_err(unavailable)?;
        Ok(outcome)
    }
}

/// Records an overlap outcome while the caller already holds the schedule row lock.
pub(crate) async fn record_pending_locked(
    transaction: &mut Transaction<'_, Postgres>,
    fire: &SchedulerPendingFireV1,
) -> Result<SchedulerPendingFireOutcomeV1, SchedulerPendingFireErrorV1> {
    match fire.policy().overlap() {
        OverlapPolicyV1::Queue { max_pending_runs } => {
            record_queued(transaction, fire, max_pending_runs).await
        }
        OverlapPolicyV1::CoalesceLatest => record_coalesced(transaction, fire).await,
        OverlapPolicyV1::Forbid | OverlapPolicyV1::AllowBounded { .. } => {
            Ok(SchedulerPendingFireOutcomeV1::Dropped)
        }
    }
}

async fn verify_schedule(
    transaction: &mut Transaction<'_, Postgres>,
    fire: &SchedulerPendingFireV1,
) -> Result<(), SchedulerPendingFireErrorV1> {
    let found = query_scalar::<_, i32>(
        "SELECT 1 FROM makosh_platform.scheduler_schedules WHERE schedule_id = $1 AND schedule_revision = $2 AND enabled = TRUE AND concurrency_key = $3 AND max_parallelism = $4 AND policy_bytes = $5 FOR UPDATE",
    )
    .bind(fire.claim().schedule_id().bytes().to_vec())
    .bind(i64::try_from(fire.schedule_revision().value()).map_err(|_| SchedulerPendingFireErrorV1::Stale)?)
    .bind(fire.concurrency_key().value())
    .bind(i32::from(fire.policy().max_parallelism()))
    .bind(fire.policy().canonical_bytes())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(unavailable)?;
    found
        .is_some()
        .then_some(())
        .ok_or(SchedulerPendingFireErrorV1::Stale)
}

async fn record_queued(
    transaction: &mut Transaction<'_, Postgres>,
    fire: &SchedulerPendingFireV1,
    max_pending_runs: u16,
) -> Result<SchedulerPendingFireOutcomeV1, SchedulerPendingFireErrorV1> {
    let count = query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM makosh_platform.scheduler_pending_fires WHERE schedule_id = $1",
    )
    .bind(fire.claim().schedule_id().bytes().to_vec())
    .fetch_one(&mut **transaction)
    .await
    .map_err(unavailable)?;
    if count >= i64::from(max_pending_runs) {
        return Ok(SchedulerPendingFireOutcomeV1::Dropped);
    }
    insert_pending(transaction, fire).await
}

async fn record_coalesced(
    transaction: &mut Transaction<'_, Postgres>,
    fire: &SchedulerPendingFireV1,
) -> Result<SchedulerPendingFireOutcomeV1, SchedulerPendingFireErrorV1> {
    query("DELETE FROM makosh_platform.scheduler_pending_fires WHERE schedule_id = $1")
        .bind(fire.claim().schedule_id().bytes().to_vec())
        .execute(&mut **transaction)
        .await
        .map_err(unavailable)?;
    insert_pending(transaction, fire)
        .await
        .map(|_| SchedulerPendingFireOutcomeV1::Coalesced)
}

async fn insert_pending(
    transaction: &mut Transaction<'_, Postgres>,
    fire: &SchedulerPendingFireV1,
) -> Result<SchedulerPendingFireOutcomeV1, SchedulerPendingFireErrorV1> {
    let inserted = query(
        "INSERT INTO makosh_platform.scheduler_pending_fires (fire_key, schedule_id, schedule_revision, scheduled_for_unix_ms, concurrency_key, recorded_at_unix_ms) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (fire_key) DO NOTHING",
    )
    .bind(fire.claim().fire_key().to_vec())
    .bind(fire.claim().schedule_id().bytes().to_vec())
    .bind(i64::try_from(fire.schedule_revision().value()).map_err(|_| SchedulerPendingFireErrorV1::Stale)?)
    .bind(fire.claim().scheduled_for().value())
    .bind(fire.concurrency_key().value())
    .bind(fire.recorded_at().value())
    .execute(&mut **transaction)
    .await
    .map_err(unavailable)?;
    Ok(if inserted.rows_affected() == 1 {
        SchedulerPendingFireOutcomeV1::Queued
    } else {
        SchedulerPendingFireOutcomeV1::AlreadyQueued
    })
}

fn unavailable(_: sqlx::Error) -> SchedulerPendingFireErrorV1 {
    SchedulerPendingFireErrorV1::Unavailable
}
