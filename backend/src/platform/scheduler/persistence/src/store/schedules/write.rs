use sqlx::{Postgres, Transaction, query};

use super::record::PersistedScheduleRowV1;
use super::request::{
    SchedulerScheduleStoreErrorV1, SchedulerScheduleUpsertOutcomeV1, SchedulerScheduleUpsertV1,
};
use crate::store::concurrency::ensure_slot;
use crate::store::runs::{SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1};

impl SchedulerPostgresStoreV1 {
    /// Atomically persists a revisioned schedule and its bounded coordination slot.
    pub async fn upsert_schedule(
        &self,
        change: &SchedulerScheduleUpsertV1,
    ) -> Result<SchedulerScheduleUpsertOutcomeV1, SchedulerScheduleStoreErrorV1> {
        let mut transaction = self.pool().begin().await.map_err(unavailable)?;
        ensure_slot(
            &mut transaction,
            change.spec().concurrency_key(),
            change.spec().policy(),
            change.updated_at(),
        )
        .await
        .map_err(map_slot_error)?;
        let outcome = upsert_locked(&mut transaction, change).await?;
        transaction.commit().await.map_err(unavailable)?;
        Ok(outcome)
    }
}

pub(crate) async fn upsert_locked(
    transaction: &mut Transaction<'_, Postgres>,
    change: &SchedulerScheduleUpsertV1,
) -> Result<SchedulerScheduleUpsertOutcomeV1, SchedulerScheduleStoreErrorV1> {
    if insert_schedule(transaction, change).await? {
        return Ok(SchedulerScheduleUpsertOutcomeV1::Inserted);
    }
    let current = lock_schedule(transaction, change).await?;
    let requested = change.spec().revision().value();
    let persisted = u64::try_from(current.revision())
        .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)?;
    if requested < persisted {
        return Err(SchedulerScheduleStoreErrorV1::StaleRevision);
    }
    if requested == persisted {
        return current
            .matches(change)
            .then_some(SchedulerScheduleUpsertOutcomeV1::Unchanged)
            .ok_or(SchedulerScheduleStoreErrorV1::RevisionConflict);
    }
    update_schedule(transaction, change).await?;
    Ok(SchedulerScheduleUpsertOutcomeV1::Updated)
}

async fn insert_schedule(
    transaction: &mut Transaction<'_, Postgres>,
    change: &SchedulerScheduleUpsertV1,
) -> Result<bool, SchedulerScheduleStoreErrorV1> {
    let changed = query(
        "INSERT INTO makosh_platform.scheduler_schedules (schedule_id, schedule_revision, job_owner, job_name, job_major, contract_name, contract_revision, contract_schema_sha256, scope_id, concurrency_key, max_parallelism, enabled, policy_bytes, next_due_at_unix_ms, updated_at_unix_ms) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) ON CONFLICT (schedule_id) DO NOTHING",
    )
    .bind(change.spec().schedule_id().bytes().to_vec())
    .bind(revision(change)?)
    .bind(change.spec().binding().job_kind().owner())
    .bind(change.spec().binding().job_kind().name())
    .bind(i32::from(change.spec().binding().job_kind().major()))
    .bind(change.spec().binding().contract_name())
    .bind(i32::try_from(change.spec().binding().contract_revision()).map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)?)
    .bind(change.spec().binding().schema_sha256().to_vec())
    .bind(change.spec().scope().value())
    .bind(change.spec().concurrency_key().value())
    .bind(i32::from(change.spec().policy().max_parallelism()))
    .bind(change.spec().enabled())
    .bind(change.spec().policy().canonical_bytes())
    .bind(change.next_due_at().value())
    .bind(change.updated_at().value())
    .execute(&mut **transaction)
    .await
    .map_err(unavailable)?;
    Ok(changed.rows_affected() == 1)
}

async fn lock_schedule(
    transaction: &mut Transaction<'_, Postgres>,
    change: &SchedulerScheduleUpsertV1,
) -> Result<PersistedScheduleRowV1, SchedulerScheduleStoreErrorV1> {
    query(
        "SELECT schedule_id, schedule_revision, job_owner, job_name, job_major, contract_name, contract_revision, contract_schema_sha256, scope_id, concurrency_key, max_parallelism, enabled, policy_bytes, next_due_at_unix_ms FROM makosh_platform.scheduler_schedules WHERE schedule_id = $1 FOR UPDATE",
    )
    .bind(change.spec().schedule_id().bytes().to_vec())
    .fetch_one(&mut **transaction)
    .await
    .map_err(unavailable)
    .and_then(PersistedScheduleRowV1::from_row)
}

async fn update_schedule(
    transaction: &mut Transaction<'_, Postgres>,
    change: &SchedulerScheduleUpsertV1,
) -> Result<(), SchedulerScheduleStoreErrorV1> {
    let updated = query(
        "UPDATE makosh_platform.scheduler_schedules SET schedule_revision = $2, job_owner = $3, job_name = $4, job_major = $5, contract_name = $6, contract_revision = $7, contract_schema_sha256 = $8, scope_id = $9, concurrency_key = $10, max_parallelism = $11, enabled = $12, policy_bytes = $13, next_due_at_unix_ms = $14, updated_at_unix_ms = $15 WHERE schedule_id = $1",
    )
    .bind(change.spec().schedule_id().bytes().to_vec())
    .bind(revision(change)?)
    .bind(change.spec().binding().job_kind().owner())
    .bind(change.spec().binding().job_kind().name())
    .bind(i32::from(change.spec().binding().job_kind().major()))
    .bind(change.spec().binding().contract_name())
    .bind(i32::try_from(change.spec().binding().contract_revision()).map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)?)
    .bind(change.spec().binding().schema_sha256().to_vec())
    .bind(change.spec().scope().value())
    .bind(change.spec().concurrency_key().value())
    .bind(i32::from(change.spec().policy().max_parallelism()))
    .bind(change.spec().enabled())
    .bind(change.spec().policy().canonical_bytes())
    .bind(change.next_due_at().value())
    .bind(change.updated_at().value())
    .execute(&mut **transaction)
    .await
    .map_err(unavailable)?;
    (updated.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerScheduleStoreErrorV1::Unavailable)
}

fn revision(change: &SchedulerScheduleUpsertV1) -> Result<i64, SchedulerScheduleStoreErrorV1> {
    i64::try_from(change.spec().revision().value())
        .map_err(|_| SchedulerScheduleStoreErrorV1::CorruptState)
}

fn unavailable(_: sqlx::Error) -> SchedulerScheduleStoreErrorV1 {
    SchedulerScheduleStoreErrorV1::Unavailable
}

fn map_slot_error(error: SchedulerRunClaimErrorV1) -> SchedulerScheduleStoreErrorV1 {
    match error {
        SchedulerRunClaimErrorV1::ConcurrencyBusy => SchedulerScheduleStoreErrorV1::ConcurrencyBusy,
        _ => SchedulerScheduleStoreErrorV1::Unavailable,
    }
}
