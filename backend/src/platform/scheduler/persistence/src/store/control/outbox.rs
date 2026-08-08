use makosh_events_protocol::delivery::{
    OutboxEntryV1, OutboxPublishReceiptV1, OutboxRecordV1, OutboxRelayErrorV1,
    OwnerOutboxStorePortV1,
};
use sqlx::{query, query_as};

use crate::SchedulerPostgresStoreV1;

/// Result-only relay view, kept distinct from the Scheduler job-dispatch outbox.
pub struct SchedulerScheduleControlResultOutboxV1<'a> {
    store: &'a SchedulerPostgresStoreV1,
}

impl<'a> SchedulerScheduleControlResultOutboxV1<'a> {
    #[must_use]
    pub const fn new(store: &'a SchedulerPostgresStoreV1) -> Self {
        Self { store }
    }

    async fn next_result(&self) -> Result<Option<OutboxEntryV1>, OutboxRelayErrorV1> {
        let row = query_as::<_, (Vec<u8>, Vec<u8>, Vec<u8>)>(
            "SELECT message_id, envelope_sha256, exact_envelope_bytes FROM makosh_platform.scheduler_schedule_control_results WHERE state = 'pending' ORDER BY created_at_unix_ms, message_id LIMIT 1",
        )
        .fetch_optional(self.store.pool())
        .await
        .map_err(persistence)?;
        row.map(result_entry).transpose()
    }

    async fn mark_result_published(
        &self,
        entry: &OutboxEntryV1,
        receipt: &OutboxPublishReceiptV1,
    ) -> Result<(), OutboxRelayErrorV1> {
        let updated = query(
            "UPDATE makosh_platform.scheduler_schedule_control_results SET state = 'published', published_stream = $2, published_sequence = $3 WHERE message_id = $1 AND state = 'pending'",
        )
        .bind(entry.record().message_id().to_vec())
        .bind(receipt.stream())
        .bind(i64::try_from(receipt.sequence()).map_err(|_| OutboxRelayErrorV1::Persistence)?)
        .execute(self.store.pool())
        .await
        .map_err(persistence)?;
        (updated.rows_affected() == 1)
            .then_some(())
            .ok_or(OutboxRelayErrorV1::Persistence)
    }
}

impl OwnerOutboxStorePortV1 for SchedulerScheduleControlResultOutboxV1<'_> {
    fn next_pending(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Option<OutboxEntryV1>, OutboxRelayErrorV1>> + Send
    {
        self.next_result()
    }

    fn mark_published(
        &mut self,
        entry: &OutboxEntryV1,
        receipt: &OutboxPublishReceiptV1,
    ) -> impl std::future::Future<Output = Result<(), OutboxRelayErrorV1>> + Send {
        self.mark_result_published(entry, receipt)
    }
}

fn result_entry(
    (message_id, envelope_sha256, exact_bytes): (Vec<u8>, Vec<u8>, Vec<u8>),
) -> Result<OutboxEntryV1, OutboxRelayErrorV1> {
    let record =
        OutboxRecordV1::accept(exact_bytes).map_err(|_| OutboxRelayErrorV1::Persistence)?;
    (message_id == record.message_id() && envelope_sha256 == record.envelope_sha256())
        .then_some(record)
        .ok_or(OutboxRelayErrorV1::Persistence)
        .and_then(|record| OutboxEntryV1::new(result_outbox_id(record.message_id()), record))
}

fn result_outbox_id(message_id: &[u8; 16]) -> String {
    let mut value = String::from("scheduler_schedule_result_");
    for byte in message_id {
        value.push_str(&format!("{byte:02x}"));
    }
    value
}

fn persistence(error: sqlx::Error) -> OutboxRelayErrorV1 {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_scheduler_schedule_control_persistence_error={error}");
    }
    OutboxRelayErrorV1::Persistence
}
