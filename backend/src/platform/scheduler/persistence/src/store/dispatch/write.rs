use sqlx::{Postgres, Transaction, query};

use super::{SchedulerDispatchClaimErrorV1, SchedulerDispatchClaimV1};
use crate::{SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1};

impl SchedulerPostgresStoreV1 {
    /// Atomically reserves a due run and its immutable dispatch bytes.
    pub async fn claim_due_with_dispatch(
        &self,
        dispatch: &SchedulerDispatchClaimV1,
    ) -> Result<(), SchedulerDispatchClaimErrorV1> {
        let mut transaction = self.pool().begin().await.map_err(unavailable)?;
        self.claim_due_in_transaction(&mut transaction, dispatch.claim())
            .await
            .map_err(map_claim_error)?;
        insert_dispatch(&mut transaction, dispatch).await?;
        transaction.commit().await.map_err(unavailable)
    }
}

pub(crate) async fn insert_dispatch(
    transaction: &mut Transaction<'_, Postgres>,
    dispatch: &SchedulerDispatchClaimV1,
) -> Result<(), SchedulerDispatchClaimErrorV1> {
    let claim = dispatch.claim();
    let record = dispatch.record();
    let inserted = query("INSERT INTO makosh_platform.scheduler_dispatches (run_id, lease_epoch, message_id, envelope_sha256, exact_envelope_bytes, state, created_at_unix_ms) VALUES ($1, $2, $3, $4, $5, 'pending', $6)")
        .bind(claim.run_id().bytes().to_vec())
        .bind(i64::try_from(claim.lease_epoch()).map_err(|_| SchedulerDispatchClaimErrorV1::ClaimDenied)?)
        .bind(record.message_id().to_vec())
        .bind(record.envelope_sha256().to_vec())
        .bind(record.exact_bytes())
        .bind(claim.claimed_at().value())
        .execute(&mut **transaction)
        .await
        .map_err(unavailable)?;
    (inserted.rows_affected() == 1)
        .then_some(())
        .ok_or(SchedulerDispatchClaimErrorV1::ClaimDenied)
}

fn map_claim_error(_: SchedulerRunClaimErrorV1) -> SchedulerDispatchClaimErrorV1 {
    SchedulerDispatchClaimErrorV1::ClaimDenied
}

fn unavailable(_: sqlx::Error) -> SchedulerDispatchClaimErrorV1 {
    SchedulerDispatchClaimErrorV1::Unavailable
}
