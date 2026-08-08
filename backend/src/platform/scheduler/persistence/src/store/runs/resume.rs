use sqlx::{Postgres, Transaction, query};

use super::claim::reserve_concurrency_slot;
use super::release::reap_expired_in_transaction;
use super::{SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1, SchedulerRunClaimV1};

impl SchedulerPostgresStoreV1 {
    /// Reacquires a retry-waiting run under a strictly newer lease epoch.
    pub async fn claim_retry(
        &self,
        claim: &SchedulerRunClaimV1,
    ) -> Result<(), SchedulerRunClaimErrorV1> {
        let mut transaction = self.pool().begin().await.map_err(unavailable)?;
        reap_expired_in_transaction(&mut transaction, claim.claimed_at()).await?;
        reserve_concurrency_slot(&mut transaction, claim).await?;
        resume_run(&mut transaction, claim).await?;
        clear_retry_due(&mut transaction, &claim.run_id().bytes()).await?;
        transaction.commit().await.map_err(unavailable)
    }
}

pub(crate) async fn resume_run(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let resumed = query(
        "UPDATE makosh_platform.scheduler_runs SET lease_epoch = $2, lease_expires_at_unix_ms = $3, state = 'pending_dispatch', dispatch_message_id = $4 WHERE run_id = $1 AND state = 'retry_wait' AND lease_epoch < $2 AND concurrency_key = $5 AND EXISTS (SELECT 1 FROM makosh_platform.scheduler_run_retries WHERE run_id = $1 AND next_attempt_at_unix_ms <= $6)",
    )
    .bind(claim.run_id().bytes().to_vec())
    .bind(i64::try_from(claim.lease_epoch()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
    .bind(claim.lease_expires_at().value())
    .bind(claim.dispatch_message_id().to_vec())
    .bind(claim.concurrency_key().value())
    .bind(claim.claimed_at().value())
    .execute(&mut **transaction)
    .await
    .map_err(unavailable)?;
    (resumed.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::Denied)
}

pub(crate) async fn clear_retry_due(
    transaction: &mut Transaction<'_, Postgres>,
    run_id: &[u8; 16],
) -> Result<(), SchedulerRunClaimErrorV1> {
    query("UPDATE makosh_platform.scheduler_run_retries SET next_attempt_at_unix_ms = NULL WHERE run_id = $1")
        .bind(run_id.to_vec())
        .execute(&mut **transaction)
        .await
        .map_err(unavailable)?;
    Ok(())
}

fn unavailable(_: sqlx::Error) -> SchedulerRunClaimErrorV1 {
    SchedulerRunClaimErrorV1::Unavailable
}
