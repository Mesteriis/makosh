use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::RetryPolicyV1;
use sqlx::{Postgres, Transaction, query, query_as};

use super::release::release_concurrency_slot;
use super::{SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1, SchedulerRunClaimV1};

pub(crate) async fn initialize_retry_state(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    query("INSERT INTO makosh_platform.scheduler_run_retries (run_id, retry_max_attempts, retry_base_backoff_millis) VALUES ($1, $2, $3)")
        .bind(claim.run_id().bytes().to_vec())
        .bind(i32::from(claim.policy().retry().max_attempts()))
        .bind(i64::try_from(claim.policy().retry().base_backoff_millis()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
        .execute(&mut **transaction)
        .await
        .map_err(unavailable)?;
    Ok(())
}

impl SchedulerPostgresStoreV1 {
    pub async fn fail_claim(
        &self,
        claim: &SchedulerRunClaimV1,
        failed_at: UtcMillisV1,
    ) -> Result<RetryFailureOutcomeV1, SchedulerRunClaimErrorV1> {
        let mut transaction = self.pool().begin().await.map_err(unavailable)?;
        let state = locked_state(&mut transaction, claim, failed_at).await?;
        let outcome = transition_failure(&mut transaction, claim, failed_at, state).await?;
        release_concurrency_slot(&mut transaction, claim, failed_at).await?;
        transaction.commit().await.map_err(unavailable)?;
        Ok(outcome)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryFailureOutcomeV1 {
    RetryAt(UtcMillisV1),
    Exhausted,
}

async fn locked_state(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
    failed_at: UtcMillisV1,
) -> Result<(i32, i32, i64), SchedulerRunClaimErrorV1> {
    query_as::<_, (i32, i32, i64)>("SELECT runs.attempt_count, retries.retry_max_attempts, retries.retry_base_backoff_millis FROM makosh_platform.scheduler_runs AS runs JOIN makosh_platform.scheduler_run_retries AS retries ON retries.run_id = runs.run_id WHERE runs.run_id = $1 AND runs.lease_epoch = $2 AND runs.concurrency_key = $3 AND runs.lease_expires_at_unix_ms > $4 AND runs.state IN ('pending_dispatch', 'dispatched', 'running') FOR UPDATE OF runs, retries")
        .bind(claim.run_id().bytes().to_vec()).bind(i64::try_from(claim.lease_epoch()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?).bind(claim.concurrency_key().value()).bind(failed_at.value())
        .fetch_optional(&mut **transaction).await.map_err(unavailable)?.ok_or(SchedulerRunClaimErrorV1::Denied)
}

async fn transition_failure(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
    failed_at: UtcMillisV1,
    state: (i32, i32, i64),
) -> Result<RetryFailureOutcomeV1, SchedulerRunClaimErrorV1> {
    let (attempt_count, max_attempts, base_backoff_millis) = state;
    let policy = RetryPolicyV1::new(
        u16::try_from(max_attempts).map_err(|_| SchedulerRunClaimErrorV1::Denied)?,
        u64::try_from(base_backoff_millis).map_err(|_| SchedulerRunClaimErrorV1::Denied)?,
    )
    .map_err(|_| SchedulerRunClaimErrorV1::Denied)?;
    let attempt = u16::try_from(attempt_count).map_err(|_| SchedulerRunClaimErrorV1::Denied)?;
    let Some(delay) = policy.delay_after_failure(attempt) else {
        mark_failure(transaction, claim, None).await?;
        return Ok(RetryFailureOutcomeV1::Exhausted);
    };
    let due = failed_at
        .value()
        .checked_add(i64::try_from(delay).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
        .map(UtcMillisV1::new)
        .ok_or(SchedulerRunClaimErrorV1::Denied)?;
    mark_failure(transaction, claim, Some(due)).await?;
    Ok(RetryFailureOutcomeV1::RetryAt(due))
}

async fn mark_failure(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
    retry_due: Option<UtcMillisV1>,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let state = if retry_due.is_some() {
        "retry_wait"
    } else {
        "failed"
    };
    query("UPDATE makosh_platform.scheduler_runs SET state = $3, attempt_count = attempt_count + 1 WHERE run_id = $1 AND lease_epoch = $2")
        .bind(claim.run_id().bytes().to_vec()).bind(i64::try_from(claim.lease_epoch()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?).bind(state).execute(&mut **transaction).await.map_err(unavailable)?;
    query("UPDATE makosh_platform.scheduler_run_retries SET next_attempt_at_unix_ms = $2 WHERE run_id = $1")
        .bind(claim.run_id().bytes().to_vec()).bind(retry_due.map(UtcMillisV1::value)).execute(&mut **transaction).await.map_err(unavailable)?;
    Ok(())
}

fn unavailable(_: sqlx::Error) -> SchedulerRunClaimErrorV1 {
    SchedulerRunClaimErrorV1::Unavailable
}
