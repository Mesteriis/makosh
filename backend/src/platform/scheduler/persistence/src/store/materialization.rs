//! Transactional conversion of due schedules into exact Scheduler dispatch records.

use makosh_clock_protocol::UtcMillisV1;
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_scheduler::{
    ScheduleContinuationV1, SchedulerDispatchIdentityV1, build_scheduled_job_envelope_v1, plan_due,
};
use makosh_scheduler_protocol::{JobRunIdV1, ScheduleRunLeaseV1, v1::JobTriggerKindV1};
use prost::Message;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction, query};

use super::{
    SchedulerDispatchClaimV1, SchedulerDueScheduleV1, SchedulerPendingFireV1,
    SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1, SchedulerRunClaimV1,
    dispatch::insert_dispatch,
    pending::record_pending_locked,
    runs::{
        clear_retry_due, reap_expired_in_transaction, reserve_concurrency_slot, reserve_run,
        resume_run,
    },
    schedules::PersistedScheduleRowV1,
};

use super::materialization_contract::{
    SchedulerDispatchAdmissionV1, SchedulerMaterializationErrorV1,
    SchedulerMaterializationOutcomeV1, SchedulerMaterializationSourceV1,
};

mod pending;

use pending::materialize_pending_occurrence;

const MAX_MATERIALIZATION_BATCH: u16 = 128;

impl SchedulerPostgresStoreV1 {
    /// Materializes bounded due occurrences only after exact subject admission.
    ///
    /// Every dispatch row is inserted in the same transaction that advances (or
    /// disarms) the source schedule. This makes replay after a process crash
    /// idempotent by the schedule fire key rather than by worker memory.
    pub async fn materialize_due(
        &self,
        now: UtcMillisV1,
        limit: u16,
        source: &SchedulerMaterializationSourceV1,
        admission: &SchedulerDispatchAdmissionV1,
    ) -> Result<SchedulerMaterializationOutcomeV1, SchedulerMaterializationErrorV1> {
        if limit == 0 || limit > MAX_MATERIALIZATION_BATCH {
            return Err(SchedulerMaterializationErrorV1::InvalidLimit);
        }
        let due = self
            .due_schedules(now, limit)
            .await
            .map_err(|_| SchedulerMaterializationErrorV1::Unavailable)?;
        let mut outcome = SchedulerMaterializationOutcomeV1::default();
        for candidate in due {
            outcome.add(
                self.materialize_candidate(candidate, now, source, admission)
                    .await?,
            );
        }
        Ok(outcome)
    }

    /// Reissues due retries with a new lease epoch and a new immutable dispatch.
    pub async fn materialize_retries(
        &self,
        now: UtcMillisV1,
        limit: u16,
        source: &SchedulerMaterializationSourceV1,
        admission: &SchedulerDispatchAdmissionV1,
    ) -> Result<SchedulerMaterializationOutcomeV1, SchedulerMaterializationErrorV1> {
        if limit == 0 || limit > MAX_MATERIALIZATION_BATCH {
            return Err(SchedulerMaterializationErrorV1::InvalidLimit);
        }
        let rows = query(
            "SELECT runs.run_id FROM makosh_platform.scheduler_runs AS runs JOIN makosh_platform.scheduler_run_retries AS retries ON retries.run_id = runs.run_id WHERE runs.state = 'retry_wait' AND retries.next_attempt_at_unix_ms <= $1 ORDER BY retries.next_attempt_at_unix_ms, runs.run_id LIMIT $2",
        )
        .bind(now.value())
        .bind(i64::from(limit))
        .fetch_all(self.pool())
        .await
        .map_err(unavailable)?;
        let mut outcome = SchedulerMaterializationOutcomeV1::default();
        for row in rows {
            let run_id: Vec<u8> = row
                .try_get("run_id")
                .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?;
            outcome.add(
                self.materialize_retry(run_id, now, source, admission)
                    .await?,
            );
        }
        Ok(outcome)
    }

    async fn materialize_candidate(
        &self,
        candidate: SchedulerDueScheduleV1,
        now: UtcMillisV1,
        source: &SchedulerMaterializationSourceV1,
        admission: &SchedulerDispatchAdmissionV1,
    ) -> Result<SchedulerMaterializationOutcomeV1, SchedulerMaterializationErrorV1> {
        let mut transaction = self.pool().begin().await.map_err(unavailable)?;
        reap_expired_in_transaction(&mut transaction, now)
            .await
            .map_err(map_claim)?;
        let Some(current) = lock_current_due(&mut transaction, &candidate).await? else {
            transaction.commit().await.map_err(unavailable)?;
            return Ok(SchedulerMaterializationOutcomeV1::default());
        };
        if !admission.admits(&current) {
            transaction.commit().await.map_err(unavailable)?;
            return Ok(SchedulerMaterializationOutcomeV1 {
                denied: 1,
                ..Default::default()
            });
        }
        let plan = plan_due(current.spec().policy(), current.next_due_at(), now)
            .map_err(|_| SchedulerMaterializationErrorV1::InvalidTime)?;
        let mut outcome = SchedulerMaterializationOutcomeV1::default();
        for scheduled_for in plan.occurrences() {
            outcome.add(
                materialize_occurrence(&mut transaction, &current, *scheduled_for, now, source)
                    .await?,
            );
        }
        advance_schedule(&mut transaction, &current, plan.continuation(), now).await?;
        transaction.commit().await.map_err(unavailable)?;
        Ok(outcome)
    }

    async fn materialize_retry(
        &self,
        run_id: Vec<u8>,
        now: UtcMillisV1,
        source: &SchedulerMaterializationSourceV1,
        admission: &SchedulerDispatchAdmissionV1,
    ) -> Result<SchedulerMaterializationOutcomeV1, SchedulerMaterializationErrorV1> {
        let mut transaction = self.pool().begin().await.map_err(unavailable)?;
        reap_expired_in_transaction(&mut transaction, now)
            .await
            .map_err(map_claim)?;
        let Some(candidate) = lock_due_retry(&mut transaction, &run_id, now).await? else {
            transaction.commit().await.map_err(unavailable)?;
            return Ok(SchedulerMaterializationOutcomeV1::default());
        };
        if !admission.admits(&candidate.schedule) {
            transaction.commit().await.map_err(unavailable)?;
            return Ok(SchedulerMaterializationOutcomeV1 {
                denied: 1,
                ..Default::default()
            });
        }
        let claim = retry_claim(&candidate, now)?;
        reserve_concurrency_slot(&mut transaction, &claim)
            .await
            .map_err(map_claim)?;
        resume_run(&mut transaction, &claim)
            .await
            .map_err(map_claim)?;
        clear_retry_due(&mut transaction, &claim.run_id().bytes())
            .await
            .map_err(map_claim)?;
        insert_dispatch(
            &mut transaction,
            &dispatch(
                &candidate.schedule,
                &claim,
                &source.dispatch_identity()?,
                JobTriggerKindV1::Retry,
            )?,
        )
        .await
        .map_err(|_| SchedulerMaterializationErrorV1::Unavailable)?;
        transaction.commit().await.map_err(unavailable)?;
        Ok(SchedulerMaterializationOutcomeV1 {
            dispatched: 1,
            ..Default::default()
        })
    }
}

async fn materialize_occurrence(
    transaction: &mut Transaction<'_, Postgres>,
    current: &SchedulerDueScheduleV1,
    scheduled_for: UtcMillisV1,
    now: UtcMillisV1,
    source: &SchedulerMaterializationSourceV1,
) -> Result<SchedulerMaterializationOutcomeV1, SchedulerMaterializationErrorV1> {
    let claim = initial_claim(current, scheduled_for, now)?;
    match reserve_concurrency_slot(transaction, &claim).await {
        Ok(()) => {
            reserve_run(transaction, &claim).await.map_err(map_claim)?;
            insert_dispatch(
                transaction,
                &dispatch(
                    current,
                    &claim,
                    &source.dispatch_identity()?,
                    JobTriggerKindV1::Time,
                )?,
            )
            .await
            .map_err(|_| SchedulerMaterializationErrorV1::Unavailable)?;
            Ok(SchedulerMaterializationOutcomeV1 {
                dispatched: 1,
                ..Default::default()
            })
        }
        Err(SchedulerRunClaimErrorV1::ConcurrencyExhausted) => {
            materialize_pending_occurrence(transaction, claim, now).await
        }
        Err(error) => Err(map_claim(error)),
    }
}

struct DueRetryV1 {
    run_id: JobRunIdV1,
    lease_epoch: u64,
    scheduled_for: UtcMillisV1,
    fire_key: [u8; 32],
    schedule: SchedulerDueScheduleV1,
}

async fn lock_due_retry(
    transaction: &mut Transaction<'_, Postgres>,
    run_id: &[u8],
    now: UtcMillisV1,
) -> Result<Option<DueRetryV1>, SchedulerMaterializationErrorV1> {
    let row = query(
        "SELECT runs.run_id AS run_id, runs.lease_epoch AS run_lease_epoch, runs.scheduled_for_unix_ms AS run_scheduled_for, runs.fire_key AS run_fire_key, schedules.schedule_id, schedules.schedule_revision, schedules.job_owner, schedules.job_name, schedules.job_major, schedules.contract_name, schedules.contract_revision, schedules.contract_schema_sha256, schedules.scope_id, schedules.concurrency_key, schedules.max_parallelism, schedules.enabled, schedules.policy_bytes, schedules.next_due_at_unix_ms FROM makosh_platform.scheduler_runs AS runs JOIN makosh_platform.scheduler_run_retries AS retries ON retries.run_id = runs.run_id JOIN makosh_platform.scheduler_schedules AS schedules ON schedules.schedule_id = runs.schedule_id AND schedules.schedule_revision = runs.schedule_revision WHERE runs.run_id = $1 AND runs.state = 'retry_wait' AND retries.next_attempt_at_unix_ms <= $2 AND schedules.enabled = TRUE FOR UPDATE OF runs, retries, schedules",
    )
    .bind(run_id)
    .bind(now.value())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(unavailable)?;
    let Some(row) = row else { return Ok(None) };
    let run_id = JobRunIdV1::new(fixed(
        row.try_get("run_id")
            .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?,
    )?)
    .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?;
    let lease_epoch = u64::try_from(
        row.try_get::<i64, _>("run_lease_epoch")
            .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?,
    )
    .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?;
    let scheduled_for = UtcMillisV1::new(
        row.try_get("run_scheduled_for")
            .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?,
    );
    let fire_key = fixed(
        row.try_get("run_fire_key")
            .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?,
    )?;
    let schedule = PersistedScheduleRowV1::from_row(row)
        .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?
        .into_due()
        .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?;
    Ok(Some(DueRetryV1 {
        run_id,
        lease_epoch,
        scheduled_for,
        fire_key,
        schedule,
    }))
}

async fn lock_current_due(
    transaction: &mut Transaction<'_, Postgres>,
    candidate: &SchedulerDueScheduleV1,
) -> Result<Option<SchedulerDueScheduleV1>, SchedulerMaterializationErrorV1> {
    let row = query(
        "SELECT schedule_id, schedule_revision, job_owner, job_name, job_major, contract_name, contract_revision, contract_schema_sha256, scope_id, concurrency_key, max_parallelism, enabled, policy_bytes, next_due_at_unix_ms FROM makosh_platform.scheduler_schedules WHERE schedule_id = $1 AND enabled = TRUE FOR UPDATE",
    )
    .bind(candidate.spec().schedule_id().bytes().to_vec())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(unavailable)?;
    row.map(PersistedScheduleRowV1::from_row)
        .transpose()
        .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)
        .and_then(|current| match current {
            Some(current) => {
                let current = current
                    .into_due()
                    .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?;
                (current.spec().revision() == candidate.spec().revision()
                    && current.next_due_at() == candidate.next_due_at())
                .then_some(current)
                .map(Some)
                .ok_or(SchedulerMaterializationErrorV1::CorruptState)
            }
            None => Ok(None),
        })
}

fn initial_claim(
    schedule: &SchedulerDueScheduleV1,
    scheduled_for: UtcMillisV1,
    now: UtcMillisV1,
) -> Result<SchedulerRunClaimV1, SchedulerMaterializationErrorV1> {
    let fire_key = fire_key(schedule, scheduled_for);
    let run_id = JobRunIdV1::new(first_16(fire_key))
        .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?;
    let expires_at = now
        .value()
        .checked_add(
            i64::try_from(schedule.spec().policy().deadline_millis())
                .map_err(|_| SchedulerMaterializationErrorV1::InvalidTime)?,
        )
        .map(UtcMillisV1::new)
        .ok_or(SchedulerMaterializationErrorV1::InvalidTime)?;
    let lease = ScheduleRunLeaseV1::new(
        run_id,
        schedule.spec().schedule_id(),
        schedule.spec().revision(),
        1,
        expires_at,
    )
    .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?;
    SchedulerRunClaimV1::new(
        lease,
        scheduled_for,
        now,
        now,
        schedule.spec().concurrency_key().clone(),
        schedule.spec().policy(),
        first_16(dispatch_key(run_id.bytes(), 1)),
        fire_key,
    )
    .map_err(|_| SchedulerMaterializationErrorV1::InvalidTime)
}

fn retry_claim(
    retry: &DueRetryV1,
    now: UtcMillisV1,
) -> Result<SchedulerRunClaimV1, SchedulerMaterializationErrorV1> {
    let epoch = retry
        .lease_epoch
        .checked_add(1)
        .ok_or(SchedulerMaterializationErrorV1::CorruptState)?;
    let expires_at = now
        .value()
        .checked_add(
            i64::try_from(retry.schedule.spec().policy().deadline_millis())
                .map_err(|_| SchedulerMaterializationErrorV1::InvalidTime)?,
        )
        .map(UtcMillisV1::new)
        .ok_or(SchedulerMaterializationErrorV1::InvalidTime)?;
    let lease = ScheduleRunLeaseV1::new(
        retry.run_id,
        retry.schedule.spec().schedule_id(),
        retry.schedule.spec().revision(),
        epoch,
        expires_at,
    )
    .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?;
    SchedulerRunClaimV1::new(
        lease,
        retry.scheduled_for,
        now,
        now,
        retry.schedule.spec().concurrency_key().clone(),
        retry.schedule.spec().policy(),
        first_16(dispatch_key(retry.run_id.bytes(), epoch)),
        retry.fire_key,
    )
    .map_err(|_| SchedulerMaterializationErrorV1::InvalidTime)
}

fn dispatch(
    schedule: &SchedulerDueScheduleV1,
    claim: &SchedulerRunClaimV1,
    source: &SchedulerDispatchIdentityV1,
    trigger: JobTriggerKindV1,
) -> Result<SchedulerDispatchClaimV1, SchedulerMaterializationErrorV1> {
    let lease = ScheduleRunLeaseV1::new(
        claim.run_id(),
        claim.schedule_id(),
        claim.schedule_revision(),
        claim.lease_epoch(),
        claim.lease_expires_at(),
    )
    .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?;
    let envelope = build_scheduled_job_envelope_v1(
        schedule.spec(),
        &lease,
        claim.scheduled_for(),
        trigger,
        claim.dispatch_message_id(),
        claim.fire_key(),
        source,
    )
    .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?;
    let record = OutboxRecordV1::accept(envelope.encode_to_vec())
        .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?;
    SchedulerDispatchClaimV1::new(claim.clone(), record)
        .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)
}

async fn advance_schedule(
    transaction: &mut Transaction<'_, Postgres>,
    schedule: &SchedulerDueScheduleV1,
    continuation: ScheduleContinuationV1,
    now: UtcMillisV1,
) -> Result<(), SchedulerMaterializationErrorV1> {
    let (enabled, next_due) = match continuation {
        ScheduleContinuationV1::At(next_due) => (true, next_due),
        ScheduleContinuationV1::Dormant => (false, schedule.next_due_at()),
        ScheduleContinuationV1::AfterTerminalDelay(_) => return Ok(()),
    };
    let updated = query(
        "UPDATE makosh_platform.scheduler_schedules SET enabled = $1, next_due_at_unix_ms = $2, updated_at_unix_ms = $3 WHERE schedule_id = $4 AND schedule_revision = $5 AND enabled = TRUE AND next_due_at_unix_ms = $6",
    )
    .bind(enabled)
    .bind(next_due.value())
    .bind(now.value())
    .bind(schedule.spec().schedule_id().bytes().to_vec())
    .bind(i64::try_from(schedule.spec().revision().value()).map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?)
    .bind(schedule.next_due_at().value())
    .execute(&mut **transaction)
    .await
    .map_err(unavailable)?;
    (updated.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerMaterializationErrorV1::CorruptState)
}

fn fire_key(schedule: &SchedulerDueScheduleV1, scheduled_for: UtcMillisV1) -> [u8; 32] {
    digest(
        b"makosh.scheduler.fire.v1",
        &[
            &schedule.spec().schedule_id().bytes(),
            &schedule.spec().revision().value().to_be_bytes(),
            &scheduled_for.value().to_be_bytes(),
        ],
    )
}

fn dispatch_key(run_id: [u8; 16], lease_epoch: u64) -> [u8; 32] {
    digest(
        b"makosh.scheduler.dispatch.v1",
        &[&run_id, &lease_epoch.to_be_bytes()],
    )
}

fn digest(domain: &[u8], fields: &[&[u8]]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(domain);
    for field in fields {
        digest.update(u64::try_from(field.len()).unwrap_or(u64::MAX).to_be_bytes());
        digest.update(field);
    }
    digest.finalize().into()
}

fn first_16(value: [u8; 32]) -> [u8; 16] {
    value[..16].try_into().expect("fixed digest prefix")
}

fn fixed<const N: usize>(value: Vec<u8>) -> Result<[u8; N], SchedulerMaterializationErrorV1> {
    value
        .try_into()
        .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)
}

fn map_claim(error: SchedulerRunClaimErrorV1) -> SchedulerMaterializationErrorV1 {
    match error {
        SchedulerRunClaimErrorV1::Unavailable => SchedulerMaterializationErrorV1::Unavailable,
        SchedulerRunClaimErrorV1::Denied
        | SchedulerRunClaimErrorV1::ConcurrencyBusy
        | SchedulerRunClaimErrorV1::AlreadyClaimed
        | SchedulerRunClaimErrorV1::PendingMissing => SchedulerMaterializationErrorV1::CorruptState,
        SchedulerRunClaimErrorV1::ConcurrencyExhausted => {
            SchedulerMaterializationErrorV1::Unavailable
        }
    }
}

fn unavailable(_: sqlx::Error) -> SchedulerMaterializationErrorV1 {
    SchedulerMaterializationErrorV1::Unavailable
}
