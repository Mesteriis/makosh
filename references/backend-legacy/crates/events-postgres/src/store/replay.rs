use super::EventStore;
use crate::errors::EventStoreError;
use crate::rows::row_to_stored_event;
use makosh_events_api::StoredEventEnvelope;

impl EventStore {
    pub async fn latest_position(&self) -> Result<i64, EventStoreError> {
        let position = sqlx::query_scalar::<_, Option<i64>>(
            r#"
            SELECT MAX(position)
            FROM event_log
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(position.unwrap_or(0))
    }

    pub async fn list_after_position(
        &self,
        after_position: i64,
        limit: u32,
    ) -> Result<Vec<StoredEventEnvelope>, EventStoreError> {
        if after_position < 0 {
            return Err(EventStoreError::InvalidReplayPosition(after_position));
        }

        let limit = i64::from(limit.clamp(1, 1000));
        let rows = sqlx::query(
            r#"
            SELECT
                position,
                event_id,
                event_type,
                schema_version,
                occurred_at,
                recorded_at,
                source,
                actor,
                subject,
                payload,
                provenance,
                causation_id,
                correlation_id
            FROM event_log
            WHERE position > $1
            ORDER BY position ASC
            LIMIT $2
            "#,
        )
        .bind(after_position)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_stored_event).collect()
    }
}
