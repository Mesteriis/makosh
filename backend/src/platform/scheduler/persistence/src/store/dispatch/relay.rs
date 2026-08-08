use makosh_events_protocol::delivery::{
    OutboxEntryV1, OutboxPublishReceiptV1, OutboxRelayErrorV1, OwnerOutboxStorePortV1,
};
use sqlx::{Postgres, Transaction, query};

use crate::SchedulerPostgresStoreV1;

impl SchedulerPostgresStoreV1 {
    /// Marks a dispatch published only after the broker has acknowledged its exact bytes.
    pub async fn mark_dispatch_published(
        &self,
        entry: &OutboxEntryV1,
        receipt: &OutboxPublishReceiptV1,
    ) -> Result<(), OutboxRelayErrorV1> {
        let mut transaction = self.pool().begin().await.map_err(persistence)?;
        mark_run_dispatched(&mut transaction, entry).await?;
        mark_dispatch_published(&mut transaction, entry, receipt).await?;
        transaction.commit().await.map_err(persistence)
    }
}

impl OwnerOutboxStorePortV1 for SchedulerPostgresStoreV1 {
    fn next_pending(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Option<OutboxEntryV1>, OutboxRelayErrorV1>> + Send
    {
        self.next_pending_dispatch()
    }

    fn mark_published(
        &mut self,
        entry: &OutboxEntryV1,
        receipt: &OutboxPublishReceiptV1,
    ) -> impl std::future::Future<Output = Result<(), OutboxRelayErrorV1>> + Send {
        self.mark_dispatch_published(entry, receipt)
    }
}

async fn mark_run_dispatched(
    transaction: &mut Transaction<'_, Postgres>,
    entry: &OutboxEntryV1,
) -> Result<(), OutboxRelayErrorV1> {
    let updated = query(
        "UPDATE makosh_platform.scheduler_runs AS runs SET state = 'dispatched' FROM makosh_platform.scheduler_dispatches AS dispatch WHERE dispatch.message_id = $1 AND dispatch.state = 'pending' AND runs.run_id = dispatch.run_id AND runs.lease_epoch = dispatch.lease_epoch AND runs.state = 'pending_dispatch'",
    )
    .bind(entry.record().message_id().to_vec())
    .execute(&mut **transaction)
    .await
    .map_err(persistence)?;
    (updated.rows_affected() == 1)
        .then_some(())
        .ok_or(OutboxRelayErrorV1::Persistence)
}

async fn mark_dispatch_published(
    transaction: &mut Transaction<'_, Postgres>,
    entry: &OutboxEntryV1,
    receipt: &OutboxPublishReceiptV1,
) -> Result<(), OutboxRelayErrorV1> {
    let updated = query(
        "UPDATE makosh_platform.scheduler_dispatches SET state = 'published', published_stream = $2, published_sequence = $3 WHERE message_id = $1 AND state = 'pending'",
    )
    .bind(entry.record().message_id().to_vec())
    .bind(receipt.stream())
    .bind(i64::try_from(receipt.sequence()).map_err(|_| OutboxRelayErrorV1::Persistence)?)
    .execute(&mut **transaction)
    .await
    .map_err(persistence)?;
    (updated.rows_affected() == 1)
        .then_some(())
        .ok_or(OutboxRelayErrorV1::Persistence)
}

fn persistence(_: sqlx::Error) -> OutboxRelayErrorV1 {
    OutboxRelayErrorV1::Persistence
}
