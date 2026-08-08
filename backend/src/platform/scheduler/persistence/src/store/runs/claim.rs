use sqlx::{PgPool, Postgres, Transaction, query, query_scalar};

use super::release::reap_expired_in_transaction;
use super::request::SchedulerRunClaimV1;
use super::retry::initialize_retry_state;

#[derive(Clone)]
pub struct SchedulerPostgresStoreV1 {
    pool: PgPool,
}

impl SchedulerPostgresStoreV1 {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) const fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn claim_due(
        &self,
        claim: &SchedulerRunClaimV1,
    ) -> Result<(), SchedulerRunClaimErrorV1> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
        self.claim_due_in_transaction(&mut transaction, claim)
            .await?;
        transaction
            .commit()
            .await
            .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)
    }

    pub(crate) async fn claim_due_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        claim: &SchedulerRunClaimV1,
    ) -> Result<(), SchedulerRunClaimErrorV1> {
        reap_expired_in_transaction(transaction, claim.claimed_at()).await?;
        reserve_concurrency_slot(transaction, claim).await?;
        reserve_schedule(transaction, claim).await?;
        reserve_run(transaction, claim).await
    }

    /// Extends an active worker lease without permitting it to outlive its deadline.
    pub async fn renew_claim_lease(
        &self,
        claim: &SchedulerRunClaimV1,
        renewed_at: makosh_clock_protocol::UtcMillisV1,
        renewed_until: makosh_clock_protocol::UtcMillisV1,
    ) -> Result<(), SchedulerRunClaimErrorV1> {
        claim.validate_renewal(renewed_at, renewed_until)?;
        let updated = query(
            "UPDATE makosh_platform.scheduler_runs SET lease_expires_at_unix_ms = $3 WHERE run_id = $1 AND lease_epoch = $2 AND state IN ('pending_dispatch', 'dispatched', 'running') AND lease_expires_at_unix_ms > $4 AND lease_expires_at_unix_ms < $3",
        )
        .bind(claim.run_id().bytes().to_vec())
        .bind(i64::try_from(claim.lease_epoch()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
        .bind(renewed_until.value())
        .bind(renewed_at.value())
        .execute(&self.pool)
        .await
        .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
        (updated.rows_affected() == 1)
            .then_some(())
            .ok_or(SchedulerRunClaimErrorV1::Denied)
    }
}

pub(crate) async fn reserve_concurrency_slot(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let updated = query(
        "UPDATE makosh_platform.scheduler_concurrency SET active_runs = active_runs + 1, updated_at_unix_ms = $3 WHERE concurrency_key = $1 AND max_parallelism = $2 AND active_runs < max_parallelism",
    )
    .bind(claim.concurrency_key().value())
    .bind(i32::from(claim.max_parallelism()))
    .bind(claim.claimed_at().value())
    .execute(&mut **transaction)
    .await
    .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
    (updated.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::ConcurrencyExhausted)
}

async fn reserve_schedule(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    if !claim.advances_due_on_claim() {
        return verify_fixed_delay_due(transaction, claim).await;
    }
    let updated = query(
        "UPDATE makosh_platform.scheduler_schedules SET next_due_at_unix_ms = $1, updated_at_unix_ms = $2 WHERE schedule_id = $3 AND schedule_revision = $4 AND enabled = TRUE AND next_due_at_unix_ms <= $5 AND concurrency_key = $6 AND max_parallelism = $7",
    )
    .bind(claim.next_due_at().value())
    .bind(claim.claimed_at().value())
    .bind(claim.schedule_id().bytes().to_vec())
    .bind(i64::try_from(claim.schedule_revision().value()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
    .bind(claim.scheduled_for().value())
    .bind(claim.concurrency_key().value())
    .bind(i32::from(claim.max_parallelism()))
    .execute(&mut **transaction)
    .await
    .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
    (updated.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::Denied)
}

async fn verify_fixed_delay_due(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let found = query_scalar::<_, i32>(
        "SELECT 1 FROM makosh_platform.scheduler_schedules WHERE schedule_id = $1 AND schedule_revision = $2 AND enabled = TRUE AND next_due_at_unix_ms <= $3 AND concurrency_key = $4 AND max_parallelism = $5 AND policy_bytes = $6 FOR UPDATE",
    )
    .bind(claim.schedule_id().bytes().to_vec())
    .bind(i64::try_from(claim.schedule_revision().value()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
    .bind(claim.scheduled_for().value())
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

pub(crate) async fn reserve_run(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &SchedulerRunClaimV1,
) -> Result<(), SchedulerRunClaimErrorV1> {
    let inserted = query(
        "INSERT INTO makosh_platform.scheduler_runs (run_id, schedule_id, schedule_revision, scheduled_for_unix_ms, lease_epoch, lease_expires_at_unix_ms, state, attempt_count, dispatch_message_id, fire_key, concurrency_key, created_at_unix_ms) VALUES ($1, $2, $3, $4, $5, $6, 'pending_dispatch', 1, $7, $8, $9, $10) ON CONFLICT (fire_key) DO NOTHING",
    )
    .bind(claim.run_id().bytes().to_vec())
    .bind(claim.schedule_id().bytes().to_vec())
    .bind(i64::try_from(claim.schedule_revision().value()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
    .bind(claim.scheduled_for().value())
    .bind(i64::try_from(claim.lease_epoch()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
    .bind(claim.lease_expires_at().value())
    .bind(claim.dispatch_message_id().to_vec())
    .bind(claim.fire_key().to_vec())
    .bind(claim.concurrency_key().value())
    .bind(claim.claimed_at().value())
    .execute(&mut **transaction)
    .await
    .map_err(|_| SchedulerRunClaimErrorV1::Unavailable)?;
    (inserted.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::AlreadyClaimed)?;
    initialize_retry_state(transaction, claim).await
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerRunClaimErrorV1 {
    Denied,
    ConcurrencyBusy,
    ConcurrencyExhausted,
    AlreadyClaimed,
    PendingMissing,
    Unavailable,
}
