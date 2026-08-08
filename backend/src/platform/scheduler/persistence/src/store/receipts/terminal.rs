use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::{
    RetryPolicyV1, SchedulerReceiptValidationErrorV1,
    v1::{JobRunOutcomeV1, JobRunReceiptV1},
    validate_job_run_receipt_v1,
};
use sqlx::{Postgres, Transaction, query, query_as, query_scalar};

use crate::{SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1};

/// Fenced terminal outcome emitted by a worker after its durable acceptance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerRunTerminalResultV1 {
    run_id: [u8; 16],
    command_message_id: [u8; 16],
    lease_epoch: u64,
    observed_at: UtcMillisV1,
    state: &'static str,
}

impl TryFrom<&JobRunReceiptV1> for SchedulerRunTerminalResultV1 {
    type Error = SchedulerRunTerminalResultErrorV1;

    fn try_from(receipt: &JobRunReceiptV1) -> Result<Self, Self::Error> {
        validate_job_run_receipt_v1(receipt).map_err(map_validation)?;
        let state = terminal_state(receipt.outcome)?;
        let lease = receipt
            .lease
            .as_ref()
            .ok_or(SchedulerRunTerminalResultErrorV1::Invalid)?;
        Ok(Self {
            run_id: bytes(&receipt.job_run_id)?,
            command_message_id: bytes(&receipt.command_message_id)?,
            lease_epoch: lease.epoch,
            observed_at: UtcMillisV1::new(receipt.observed_at_unix_millis),
            state,
        })
    }
}

impl SchedulerPostgresStoreV1 {
    /// Applies a terminal result only after the exact dispatch was accepted.
    pub async fn finish_receipt(
        &self,
        result: &SchedulerRunTerminalResultV1,
    ) -> Result<SchedulerRunTerminalResultOutcomeV1, SchedulerRunClaimErrorV1> {
        let mut transaction = self.pool().begin().await.map_err(unavailable)?;
        let outcome = if result.state == "retryable_failed" {
            apply_retryable_failure(&mut transaction, result).await?
        } else {
            apply_result(&mut transaction, result).await?
        };
        transaction.commit().await.map_err(unavailable)?;
        Ok(outcome)
    }
}

async fn apply_result(
    transaction: &mut Transaction<'_, Postgres>,
    result: &SchedulerRunTerminalResultV1,
) -> Result<SchedulerRunTerminalResultOutcomeV1, SchedulerRunClaimErrorV1> {
    let (state, concurrency_key) = locked_run_state(transaction, result).await?;
    if state == "running" {
        insert_result(transaction, result).await?;
        update_run(transaction, result).await?;
        release_slot(transaction, &concurrency_key, result).await?;
        return Ok(SchedulerRunTerminalResultOutcomeV1::Applied);
    }
    duplicate_result(transaction, result, &state).await
}

/// A retryable worker attempt is terminal for its exact dispatch, but not
/// necessarily terminal for the durable run. The retry schedule is derived
/// from the state persisted at claim time, so a changed schedule cannot alter
/// an already-issued lease.
async fn apply_retryable_failure(
    transaction: &mut Transaction<'_, Postgres>,
    result: &SchedulerRunTerminalResultV1,
) -> Result<SchedulerRunTerminalResultOutcomeV1, SchedulerRunClaimErrorV1> {
    let (state, concurrency_key) = locked_run_state(transaction, result).await?;
    if state == "running" {
        insert_result(transaction, result).await?;
        let retry_due = retry_due(transaction, result).await?;
        transition_to_retry_wait(transaction, result, retry_due).await?;
        release_slot(transaction, &concurrency_key, result).await?;
        return Ok(SchedulerRunTerminalResultOutcomeV1::Applied);
    }
    duplicate_result(transaction, result, &state).await
}

async fn retry_due(
    transaction: &mut Transaction<'_, Postgres>,
    result: &SchedulerRunTerminalResultV1,
) -> Result<Option<UtcMillisV1>, SchedulerRunClaimErrorV1> {
    let (attempt_count, max_attempts, base_backoff_millis) = query_as::<_, (i32, i32, i64)>(
        "SELECT runs.attempt_count, retries.retry_max_attempts, retries.retry_base_backoff_millis FROM makosh_platform.scheduler_runs AS runs JOIN makosh_platform.scheduler_run_retries AS retries ON retries.run_id = runs.run_id WHERE runs.run_id = $1 AND runs.lease_epoch = $2 FOR UPDATE OF retries",
    )
    .bind(result.run_id.to_vec())
    .bind(i64::try_from(result.lease_epoch).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(unavailable)?
    .ok_or(SchedulerRunClaimErrorV1::Denied)?;
    let policy = RetryPolicyV1::new(
        u16::try_from(max_attempts).map_err(|_| SchedulerRunClaimErrorV1::Denied)?,
        u64::try_from(base_backoff_millis).map_err(|_| SchedulerRunClaimErrorV1::Denied)?,
    )
    .map_err(|_| SchedulerRunClaimErrorV1::Denied)?;
    let attempt = u16::try_from(attempt_count).map_err(|_| SchedulerRunClaimErrorV1::Denied)?;
    policy
        .delay_after_failure(attempt)
        .map(|delay| {
            result
                .observed_at
                .value()
                .checked_add(i64::try_from(delay).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
                .map(UtcMillisV1::new)
                .ok_or(SchedulerRunClaimErrorV1::Denied)
        })
        .transpose()
}

async fn transition_to_retry_wait(
    transaction: &mut Transaction<'_, Postgres>,
    result: &SchedulerRunTerminalResultV1,
    retry_due: Option<UtcMillisV1>,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let state = if retry_due.is_some() {
        "retry_wait"
    } else {
        "failed"
    };
    let updated = query(
        "UPDATE makosh_platform.scheduler_runs SET state = $4, attempt_count = attempt_count + 1 WHERE run_id = $1 AND lease_epoch = $2 AND dispatch_message_id = $3 AND state = 'running'",
    )
    .bind(result.run_id.to_vec())
    .bind(i64::try_from(result.lease_epoch).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
    .bind(result.command_message_id.to_vec())
    .bind(state)
    .execute(&mut **transaction)
    .await
    .map_err(unavailable)?;
    (updated.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::Denied)?;
    query(
        "UPDATE makosh_platform.scheduler_run_retries SET next_attempt_at_unix_ms = $2 WHERE run_id = $1",
    )
    .bind(result.run_id.to_vec())
    .bind(retry_due.map(UtcMillisV1::value))
    .execute(&mut **transaction)
    .await
    .map_err(unavailable)?;
    Ok(())
}

async fn locked_run_state(
    transaction: &mut Transaction<'_, Postgres>,
    result: &SchedulerRunTerminalResultV1,
) -> Result<(String, String), SchedulerRunClaimErrorV1> {
    query_as("SELECT runs.state, runs.concurrency_key FROM makosh_platform.scheduler_runs AS runs JOIN makosh_platform.scheduler_dispatches AS dispatch ON dispatch.run_id = runs.run_id AND dispatch.lease_epoch = runs.lease_epoch JOIN makosh_platform.scheduler_run_acceptances AS acceptance ON acceptance.command_message_id = dispatch.message_id AND acceptance.run_id = runs.run_id AND acceptance.lease_epoch = runs.lease_epoch WHERE runs.run_id = $1 AND runs.lease_epoch = $2 AND runs.dispatch_message_id = $3 AND dispatch.message_id = $3 AND dispatch.state = 'published' AND runs.lease_expires_at_unix_ms > $4 FOR UPDATE OF runs, dispatch, acceptance")
        .bind(result.run_id.to_vec()).bind(i64::try_from(result.lease_epoch).map_err(|_| SchedulerRunClaimErrorV1::Denied)?).bind(result.command_message_id.to_vec()).bind(result.observed_at.value())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(unavailable)?
        .ok_or(SchedulerRunClaimErrorV1::PendingMissing)
}

async fn insert_result(
    transaction: &mut Transaction<'_, Postgres>,
    result: &SchedulerRunTerminalResultV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let inserted = query("INSERT INTO makosh_platform.scheduler_run_results (command_message_id, run_id, lease_epoch, outcome, observed_at_unix_ms) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (command_message_id) DO NOTHING")
        .bind(result.command_message_id.to_vec()).bind(result.run_id.to_vec()).bind(i64::try_from(result.lease_epoch).map_err(|_| SchedulerRunClaimErrorV1::Denied)?).bind(result.state).bind(result.observed_at.value())
        .execute(&mut **transaction).await.map_err(unavailable)?;
    (inserted.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::Denied)
}

async fn update_run(
    transaction: &mut Transaction<'_, Postgres>,
    result: &SchedulerRunTerminalResultV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let updated = query("UPDATE makosh_platform.scheduler_runs SET state = $4 WHERE run_id = $1 AND lease_epoch = $2 AND dispatch_message_id = $3 AND state = 'running'")
        .bind(result.run_id.to_vec()).bind(i64::try_from(result.lease_epoch).map_err(|_| SchedulerRunClaimErrorV1::Denied)?).bind(result.command_message_id.to_vec()).bind(result.state)
        .execute(&mut **transaction).await.map_err(unavailable)?;
    (updated.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::Denied)
}

async fn release_slot(
    transaction: &mut Transaction<'_, Postgres>,
    concurrency_key: &str,
    result: &SchedulerRunTerminalResultV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let updated = query("UPDATE makosh_platform.scheduler_concurrency SET active_runs = active_runs - 1, updated_at_unix_ms = $2 WHERE concurrency_key = $1 AND active_runs > 0")
        .bind(concurrency_key).bind(result.observed_at.value())
        .execute(&mut **transaction).await.map_err(unavailable)?;
    (updated.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::Denied)
}

async fn duplicate_result(
    transaction: &mut Transaction<'_, Postgres>,
    result: &SchedulerRunTerminalResultV1,
    state: &str,
) -> Result<SchedulerRunTerminalResultOutcomeV1, SchedulerRunClaimErrorV1> {
    let matches = query_scalar("SELECT EXISTS (SELECT 1 FROM makosh_platform.scheduler_run_results WHERE command_message_id = $1 AND run_id = $2 AND lease_epoch = $3 AND outcome = $4)")
        .bind(result.command_message_id.to_vec()).bind(result.run_id.to_vec()).bind(i64::try_from(result.lease_epoch).map_err(|_| SchedulerRunClaimErrorV1::Denied)?).bind(result.state)
        .fetch_one(&mut **transaction).await.map_err(unavailable)?;
    (matches
        && (state == result.state || (result.state == "retryable_failed" && state == "retry_wait")))
        .then_some(SchedulerRunTerminalResultOutcomeV1::AlreadyApplied)
        .ok_or(SchedulerRunClaimErrorV1::Denied)
}

fn terminal_state(outcome: i32) -> Result<&'static str, SchedulerRunTerminalResultErrorV1> {
    match JobRunOutcomeV1::try_from(outcome).ok() {
        Some(JobRunOutcomeV1::Succeeded) => Ok("finished"),
        Some(JobRunOutcomeV1::RetryableFailed) => Ok("retryable_failed"),
        Some(JobRunOutcomeV1::Failed) => Ok("failed"),
        Some(JobRunOutcomeV1::Cancelled) => Ok("cancelled"),
        _ => Err(SchedulerRunTerminalResultErrorV1::NotTerminal),
    }
}

fn bytes(value: &[u8]) -> Result<[u8; 16], SchedulerRunTerminalResultErrorV1> {
    value
        .try_into()
        .map_err(|_| SchedulerRunTerminalResultErrorV1::Invalid)
}
fn map_validation(_: SchedulerReceiptValidationErrorV1) -> SchedulerRunTerminalResultErrorV1 {
    SchedulerRunTerminalResultErrorV1::Invalid
}
fn unavailable(_: sqlx::Error) -> SchedulerRunClaimErrorV1 {
    SchedulerRunClaimErrorV1::Unavailable
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerRunTerminalResultErrorV1 {
    Invalid,
    NotTerminal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerRunTerminalResultOutcomeV1 {
    Applied,
    AlreadyApplied,
}
