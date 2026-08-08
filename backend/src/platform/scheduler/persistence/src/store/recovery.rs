//! Offline recovery preparation for Scheduler-owned durable delivery state.

use makosh_events_protocol::delivery::OutboxRecordV1;
use sqlx::{Postgres, Row, Transaction, query, query_scalar};

use crate::SchedulerPostgresStoreV1;

const RECOVERY_LOCK_KEY: i64 = 0x4845_524d_4553_5301;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerRecoveryReplayReportV1 {
    requeued_dispatches: u64,
    preserved_acceptances: u64,
    preserved_results: u64,
}

impl SchedulerRecoveryReplayReportV1 {
    #[must_use]
    pub const fn requeued_dispatches(self) -> u64 {
        self.requeued_dispatches
    }

    #[must_use]
    pub const fn preserved_acceptances(self) -> u64 {
        self.preserved_acceptances
    }

    #[must_use]
    pub const fn preserved_results(self) -> u64 {
        self.preserved_results
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerRecoveryErrorV1 {
    InvalidDurableState,
    Persistence,
}

impl SchedulerPostgresStoreV1 {
    /// Requeues only broker-acknowledged dispatches which have no durable
    /// acceptance. Exact envelope bytes and their digest are never rewritten.
    pub async fn prepare_event_hub_replay(
        &self,
    ) -> Result<SchedulerRecoveryReplayReportV1, SchedulerRecoveryErrorV1> {
        let mut transaction = self.pool().begin().await.map_err(persistence)?;
        acquire_recovery_lock(&mut transaction).await?;
        let replayable = read_replayable_dispatches(&mut transaction).await?;
        validate_replayable_dispatches(&replayable)?;
        let requeued = requeue_dispatches(&mut transaction).await?;
        if requeued != replayable.len() as u64 {
            return Err(SchedulerRecoveryErrorV1::InvalidDurableState);
        }
        let (acceptances, results) = inbox_counts(&mut transaction).await?;
        transaction.commit().await.map_err(persistence)?;
        Ok(SchedulerRecoveryReplayReportV1 {
            requeued_dispatches: requeued,
            preserved_acceptances: acceptances,
            preserved_results: results,
        })
    }
}

async fn acquire_recovery_lock(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), SchedulerRecoveryErrorV1> {
    query("SELECT pg_advisory_xact_lock($1)")
        .bind(RECOVERY_LOCK_KEY)
        .execute(&mut **transaction)
        .await
        .map_err(persistence)?;
    Ok(())
}

async fn read_replayable_dispatches(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>, SchedulerRecoveryErrorV1> {
    let rows = query(
        "SELECT dispatch.message_id, dispatch.envelope_sha256, dispatch.exact_envelope_bytes FROM makosh_platform.scheduler_dispatches AS dispatch JOIN makosh_platform.scheduler_runs AS runs ON runs.run_id = dispatch.run_id AND runs.lease_epoch = dispatch.lease_epoch LEFT JOIN makosh_platform.scheduler_run_acceptances AS acceptance ON acceptance.command_message_id = dispatch.message_id WHERE dispatch.state = 'published' AND runs.state = 'dispatched' AND acceptance.command_message_id IS NULL ORDER BY dispatch.created_at_unix_ms, dispatch.message_id FOR UPDATE OF dispatch, runs",
    )
    .fetch_all(&mut **transaction)
    .await
    .map_err(persistence)?;
    rows.into_iter()
        .map(|row| {
            Ok((
                row.try_get(0).map_err(persistence)?,
                row.try_get(1).map_err(persistence)?,
                row.try_get(2).map_err(persistence)?,
            ))
        })
        .collect()
}

fn validate_replayable_dispatches(
    rows: &[(Vec<u8>, Vec<u8>, Vec<u8>)],
) -> Result<(), SchedulerRecoveryErrorV1> {
    rows.iter().try_for_each(|(message_id, digest, bytes)| {
        let record = OutboxRecordV1::accept(bytes.clone())
            .map_err(|_| SchedulerRecoveryErrorV1::InvalidDurableState)?;
        (message_id.as_slice() == record.message_id()
            && digest.as_slice() == record.envelope_sha256())
        .then_some(())
        .ok_or(SchedulerRecoveryErrorV1::InvalidDurableState)
    })
}

async fn requeue_dispatches(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<u64, SchedulerRecoveryErrorV1> {
    let result = query(
        "WITH replayable AS (SELECT dispatch.message_id, dispatch.run_id, dispatch.lease_epoch FROM makosh_platform.scheduler_dispatches AS dispatch JOIN makosh_platform.scheduler_runs AS runs ON runs.run_id = dispatch.run_id AND runs.lease_epoch = dispatch.lease_epoch LEFT JOIN makosh_platform.scheduler_run_acceptances AS acceptance ON acceptance.command_message_id = dispatch.message_id WHERE dispatch.state = 'published' AND runs.state = 'dispatched' AND acceptance.command_message_id IS NULL), reset_runs AS (UPDATE makosh_platform.scheduler_runs AS runs SET state = 'pending_dispatch' FROM replayable WHERE runs.run_id = replayable.run_id AND runs.lease_epoch = replayable.lease_epoch AND runs.state = 'dispatched' RETURNING replayable.message_id) UPDATE makosh_platform.scheduler_dispatches AS dispatch SET state = 'pending', published_stream = NULL, published_sequence = NULL FROM reset_runs WHERE dispatch.message_id = reset_runs.message_id AND dispatch.state = 'published'",
    )
    .execute(&mut **transaction)
    .await
    .map_err(persistence)?;
    Ok(result.rows_affected())
}

async fn inbox_counts(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(u64, u64), SchedulerRecoveryErrorV1> {
    let acceptances: i64 =
        query_scalar("SELECT count(*) FROM makosh_platform.scheduler_run_acceptances")
            .fetch_one(&mut **transaction)
            .await
            .map_err(persistence)?;
    let results: i64 = query_scalar("SELECT count(*) FROM makosh_platform.scheduler_run_results")
        .fetch_one(&mut **transaction)
        .await
        .map_err(persistence)?;
    Ok((
        u64::try_from(acceptances).map_err(|_| SchedulerRecoveryErrorV1::InvalidDurableState)?,
        u64::try_from(results).map_err(|_| SchedulerRecoveryErrorV1::InvalidDurableState)?,
    ))
}

fn persistence(_: sqlx::Error) -> SchedulerRecoveryErrorV1 {
    SchedulerRecoveryErrorV1::Persistence
}
