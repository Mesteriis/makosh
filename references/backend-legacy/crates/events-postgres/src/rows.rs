use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::EventStoreError;
use makosh_events_api::{EventEnvelope, StoredEventEnvelope};

pub(super) fn row_to_event(row: PgRow) -> Result<EventEnvelope, EventStoreError> {
    Ok(EventEnvelope {
        event_id: row.try_get("event_id")?,
        event_type: row.try_get("event_type")?,
        schema_version: row.try_get("schema_version")?,
        occurred_at: row.try_get("occurred_at")?,
        recorded_at: row.try_get("recorded_at")?,
        source: row.try_get("source")?,
        actor: row.try_get("actor")?,
        subject: row.try_get("subject")?,
        payload: row.try_get("payload")?,
        provenance: row.try_get("provenance")?,
        causation_id: row.try_get("causation_id")?,
        correlation_id: row.try_get("correlation_id")?,
    })
}

pub(super) fn row_to_stored_event(row: PgRow) -> Result<StoredEventEnvelope, EventStoreError> {
    Ok(StoredEventEnvelope {
        position: row.try_get("position")?,
        event: row_to_event(row)?,
    })
}
