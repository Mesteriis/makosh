use makosh_events_protocol::delivery::{OutboxEntryV1, OutboxRecordV1, OutboxRelayErrorV1};
use sqlx::query_as;

use crate::SchedulerPostgresStoreV1;

impl SchedulerPostgresStoreV1 {
    /// Reads one pending exact dispatch without changing its durable state.
    pub async fn next_pending_dispatch(&self) -> Result<Option<OutboxEntryV1>, OutboxRelayErrorV1> {
        let row = query_as::<_, (Vec<u8>, Vec<u8>, Vec<u8>)>(
            "SELECT message_id, envelope_sha256, exact_envelope_bytes FROM makosh_platform.scheduler_dispatches WHERE state = 'pending' ORDER BY created_at_unix_ms, message_id LIMIT 1",
        )
        .fetch_optional(self.pool())
        .await
        .map_err(persistence)?;
        row.map(PendingDispatchRow::from_tuple)
            .map(PendingDispatchRow::into_entry)
            .transpose()
    }
}

struct PendingDispatchRow {
    message_id: Vec<u8>,
    envelope_sha256: Vec<u8>,
    exact_envelope_bytes: Vec<u8>,
}

impl PendingDispatchRow {
    fn from_tuple(value: (Vec<u8>, Vec<u8>, Vec<u8>)) -> Self {
        let (message_id, envelope_sha256, exact_envelope_bytes) = value;
        Self {
            message_id,
            envelope_sha256,
            exact_envelope_bytes,
        }
    }

    fn into_entry(self) -> Result<OutboxEntryV1, OutboxRelayErrorV1> {
        let record = OutboxRecordV1::accept(self.exact_envelope_bytes)
            .map_err(|_| OutboxRelayErrorV1::Persistence)?;
        (self.message_id == record.message_id() && self.envelope_sha256 == record.envelope_sha256())
            .then_some(record)
            .ok_or(OutboxRelayErrorV1::Persistence)
            .and_then(|record| OutboxEntryV1::new(dispatch_outbox_id(record.message_id()), record))
    }
}

fn dispatch_outbox_id(message_id: &[u8; 16]) -> String {
    let mut value = String::from("scheduler_dispatch_");
    for byte in message_id {
        value.push_str(&format!("{byte:02x}"));
    }
    value
}

fn persistence(_: sqlx::Error) -> OutboxRelayErrorV1 {
    OutboxRelayErrorV1::Persistence
}
