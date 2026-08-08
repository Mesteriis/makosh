use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::ScheduleTriggerV1;
use sqlx::{Postgres, Transaction, query};

use super::claim::{SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1};
use super::request::SchedulerRunClaimV1;

impl SchedulerPostgresStoreV1 {
    /// Releases one fenced run after its worker reached a terminal outcome.
    pub async fn finish_claim(
        &self,
        claim: &SchedulerRunClaimV1,
        finished_at: UtcMillisV1,
    ) -> Result<(), SchedulerRunClaimErrorV1> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
        mark_run_finished(&mut transaction, claim, finished_at).await?;
        release_concurrency_slot(&mut transaction, claim, finished_at).await?;
        transaction
            .commit()
            .await
            .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)
    }

    /// Completes a fixed-delay run and rearms only its still-current schedule revision.
    pub async fn finish_fixed_delay_claim(
        &self,
        claim: &SchedulerRunClaimV1,
        finished_at: UtcMillisV1,
    ) -> Result<FixedDelayCompletionOutcomeV1, SchedulerRunClaimErrorV1> {
        let next_due = fixed_delay_next_due(claim, finished_at)?;
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
        mark_run_finished(&mut transaction, claim, finished_at).await?;
        let outcome = rearm_fixed_delay(&mut transaction, claim, next_due, finished_at).await?;
        release_concurrency_slot(&mut transaction, claim, finished_at).await?;
        transaction
            .commit()
            .await
            .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
        Ok(outcome)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixedDelayCompletionOutcomeV1 {
    Rearmed,
    ConfigurationReplaced,
}

fn fixed_delay_next_due(
    claim: &SchedulerRunClaimV1,
    finished_at: UtcMillisV1,
) -> Result<UtcMillisV1, SchedulerRunClaimErrorV1> {
    let ScheduleTriggerV1::FixedDelay { delay_millis } = claim.policy().trigger() else {
        return Err(SchedulerRunClaimErrorV1::Denied);
    };
    let delay = i64::try_from(*delay_millis).map_err(|_| SchedulerRunClaimErrorV1::Denied)?;
    finished_at
        .value()
        .checked_add(delay)
        .map(UtcMillisV1::new)
        .ok_or(SchedulerRunClaimErrorV1::Denied)
}

async fn rearm_fixed_delay(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
    next_due: UtcMillisV1,
    finished_at: UtcMillisV1,
) -> Result<FixedDelayCompletionOutcomeV1, SchedulerRunClaimErrorV1> {
    let updated = query(
        "UPDATE makosh_platform.scheduler_schedules SET next_due_at_unix_ms = $1, updated_at_unix_ms = $2 WHERE schedule_id = $3 AND schedule_revision = $4 AND enabled = TRUE AND concurrency_key = $5 AND max_parallelism = $6 AND policy_bytes = $7",
    )
    .bind(next_due.value())
    .bind(finished_at.value())
    .bind(claim.schedule_id().bytes().to_vec())
    .bind(i64::try_from(claim.schedule_revision().value()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
    .bind(claim.concurrency_key().value())
    .bind(i32::from(claim.max_parallelism()))
    .bind(claim.policy().canonical_bytes())
    .execute(&mut **transaction)
    .await
    .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
    Ok(if updated.rows_affected() == 1 {
        FixedDelayCompletionOutcomeV1::Rearmed
    } else {
        FixedDelayCompletionOutcomeV1::ConfigurationReplaced
    })
}

async fn mark_run_finished(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
    finished_at: UtcMillisV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let updated = query(
        "UPDATE makosh_platform.scheduler_runs SET state = 'finished' WHERE run_id = $1 AND lease_epoch = $2 AND concurrency_key = $3 AND lease_expires_at_unix_ms > $4 AND state IN ('pending_dispatch', 'dispatched', 'running')",
    )
    .bind(claim.run_id().bytes().to_vec())
    .bind(i64::try_from(claim.lease_epoch()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
    .bind(claim.concurrency_key().value())
    .bind(finished_at.value())
    .execute(&mut **transaction)
    .await
    .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
    (updated.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::Denied)
}

pub(crate) async fn release_concurrency_slot(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
    finished_at: UtcMillisV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let updated = query(
        "UPDATE makosh_platform.scheduler_concurrency SET active_runs = active_runs - 1, updated_at_unix_ms = $2 WHERE concurrency_key = $1 AND active_runs > 0",
    )
    .bind(claim.concurrency_key().value())
    .bind(finished_at.value())
    .execute(&mut **transaction)
    .await
    .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
    (updated.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::Denied)
}

pub(crate) async fn reap_expired_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    now: UtcMillisV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    query(
        "WITH expired AS (UPDATE makosh_platform.scheduler_runs SET state = 'expired' WHERE state IN ('pending_dispatch', 'dispatched', 'running') AND lease_expires_at_unix_ms <= $1 RETURNING concurrency_key), released AS (SELECT concurrency_key, COUNT(*)::INTEGER AS released_count FROM expired GROUP BY concurrency_key) UPDATE makosh_platform.scheduler_concurrency AS slots SET active_runs = GREATEST(slots.active_runs - released.released_count, 0), updated_at_unix_ms = $1 FROM released WHERE slots.concurrency_key = released.concurrency_key",
    )
    .bind(now.value())
    .execute(&mut **transaction)
    .await
    .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
    Ok(())
}
