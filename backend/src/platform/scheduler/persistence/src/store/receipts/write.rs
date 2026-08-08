use sqlx::{Postgres, Transaction, query, query_scalar};

use super::SchedulerRunAcceptanceV1;
use crate::{SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1};

impl SchedulerPostgresStoreV1 {
    /// Records a worker acknowledgement only for the current published dispatch lease.
    pub async fn accept_receipt(
        &self,
        acceptance: &SchedulerRunAcceptanceV1,
    ) -> Result<SchedulerRunAcceptanceOutcomeV1, SchedulerRunClaimErrorV1> {
        let mut transaction = self.pool().begin().await.map_err(unavailable)?;
        let outcome = apply_acceptance(&mut transaction, acceptance).await?;
        transaction.commit().await.map_err(unavailable)?;
        Ok(outcome)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerRunAcceptanceOutcomeV1 {
    Applied,
    AlreadyApplied,
}

async fn apply_acceptance(
    transaction: &mut Transaction<'_, Postgres>,
    acceptance: &SchedulerRunAcceptanceV1,
) -> Result<SchedulerRunAcceptanceOutcomeV1, SchedulerRunClaimErrorV1> {
    let state = locked_dispatch_state(transaction, acceptance).await?;
    match state.as_str() {
        "dispatched" => insert_and_start(transaction, acceptance).await,
        "running" if persisted_acceptance_matches(transaction, acceptance).await? => {
            Ok(SchedulerRunAcceptanceOutcomeV1::AlreadyApplied)
        }
        _ => Err(SchedulerRunClaimErrorV1::Denied),
    }
}

async fn locked_dispatch_state(
    transaction: &mut Transaction<'_, Postgres>,
    acceptance: &SchedulerRunAcceptanceV1,
) -> Result<String, SchedulerRunClaimErrorV1> {
    query_scalar("SELECT runs.state FROM makosh_platform.scheduler_runs AS runs JOIN makosh_platform.scheduler_dispatches AS dispatch ON dispatch.run_id = runs.run_id AND dispatch.lease_epoch = runs.lease_epoch WHERE runs.run_id = $1 AND runs.lease_epoch = $2 AND runs.dispatch_message_id = $3 AND runs.lease_expires_at_unix_ms > $4 AND dispatch.message_id = $3 AND dispatch.state = 'published' FOR UPDATE OF runs, dispatch")
        .bind(acceptance.run_id().to_vec())
        .bind(i64::try_from(acceptance.lease_epoch()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
        .bind(acceptance.command_message_id().to_vec())
        .bind(acceptance.observed_at().value())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(unavailable)?
        .ok_or(SchedulerRunClaimErrorV1::Denied)
}

async fn insert_and_start(
    transaction: &mut Transaction<'_, Postgres>,
    acceptance: &SchedulerRunAcceptanceV1,
) -> Result<SchedulerRunAcceptanceOutcomeV1, SchedulerRunClaimErrorV1> {
    let inserted = query("INSERT INTO makosh_platform.scheduler_run_acceptances (command_message_id, run_id, lease_epoch, observed_at_unix_ms) VALUES ($1, $2, $3, $4) ON CONFLICT (command_message_id) DO NOTHING")
        .bind(acceptance.command_message_id().to_vec())
        .bind(acceptance.run_id().to_vec())
        .bind(i64::try_from(acceptance.lease_epoch()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
        .bind(acceptance.observed_at().value())
        .execute(&mut **transaction)
        .await
        .map_err(unavailable)?;
    (inserted.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerRunClaimErrorV1::Denied)?;
    query("UPDATE makosh_platform.scheduler_runs SET state = 'running' WHERE run_id = $1 AND lease_epoch = $2 AND dispatch_message_id = $3 AND state = 'dispatched'")
        .bind(acceptance.run_id().to_vec())
        .bind(i64::try_from(acceptance.lease_epoch()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
        .bind(acceptance.command_message_id().to_vec())
        .execute(&mut **transaction)
        .await
        .map_err(unavailable)?;
    Ok(SchedulerRunAcceptanceOutcomeV1::Applied)
}

async fn persisted_acceptance_matches(
    transaction: &mut Transaction<'_, Postgres>,
    acceptance: &SchedulerRunAcceptanceV1,
) -> Result<bool, SchedulerRunClaimErrorV1> {
    query_scalar("SELECT EXISTS (SELECT 1 FROM makosh_platform.scheduler_run_acceptances WHERE command_message_id = $1 AND run_id = $2 AND lease_epoch = $3)")
        .bind(acceptance.command_message_id().to_vec())
        .bind(acceptance.run_id().to_vec())
        .bind(i64::try_from(acceptance.lease_epoch()).map_err(|_| SchedulerRunClaimErrorV1::Denied)?)
        .fetch_one(&mut **transaction)
        .await
        .map_err(unavailable)
}

fn unavailable(_: sqlx::Error) -> SchedulerRunClaimErrorV1 {
    SchedulerRunClaimErrorV1::Unavailable
}
