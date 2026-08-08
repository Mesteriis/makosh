use makosh_clock_protocol::UtcMillisV1;
use sqlx::{Postgres, Transaction};

use super::{
    SchedulerMaterializationErrorV1, SchedulerMaterializationOutcomeV1, SchedulerPendingFireV1,
    SchedulerRunClaimV1, record_pending_locked,
};

pub(super) async fn materialize_pending_occurrence(
    transaction: &mut Transaction<'_, Postgres>,
    claim: SchedulerRunClaimV1,
    now: UtcMillisV1,
) -> Result<SchedulerMaterializationOutcomeV1, SchedulerMaterializationErrorV1> {
    let pending = SchedulerPendingFireV1::new(claim, now)
        .map_err(|_| SchedulerMaterializationErrorV1::CorruptState)?;
    match record_pending_locked(transaction, &pending)
        .await
        .map_err(|_| SchedulerMaterializationErrorV1::Unavailable)?
    {
        super::super::SchedulerPendingFireOutcomeV1::Queued
        | super::super::SchedulerPendingFireOutcomeV1::Coalesced
        | super::super::SchedulerPendingFireOutcomeV1::AlreadyQueued => {
            Ok(SchedulerMaterializationOutcomeV1 {
                queued: 1,
                ..Default::default()
            })
        }
        super::super::SchedulerPendingFireOutcomeV1::Dropped => {
            Ok(SchedulerMaterializationOutcomeV1 {
                dropped: 1,
                ..Default::default()
            })
        }
    }
}
