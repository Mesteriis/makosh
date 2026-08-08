use super::EventStore;
use crate::errors::EventStoreError;
use crate::rows::{row_to_event, row_to_stored_event};
use makosh_events_api::{EventEnvelope, EventLogQuery, StoredEventEnvelope};
use sqlx::{Postgres, QueryBuilder};

impl EventStore {
    pub async fn get_by_id(
        &self,
        event_id: &str,
    ) -> Result<Option<EventEnvelope>, EventStoreError> {
        let row = sqlx::query(
            r#"
            SELECT
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
            WHERE event_id = $1
            "#,
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_event).transpose()
    }

    pub async fn get_by_source_idempotency(
        &self,
        event_type: &str,
        source_kind: &str,
        provider: Option<&str>,
        source_id: &str,
    ) -> Result<Option<EventEnvelope>, EventStoreError> {
        let row = sqlx::query(
            r#"
            SELECT
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
            WHERE event_type = $1
              AND source->>'kind' = $2
              AND COALESCE(source->>'provider', '') = $3
              AND source->>'source_id' = $4
            "#,
        )
        .bind(event_type)
        .bind(source_kind)
        .bind(provider.unwrap_or_default())
        .bind(source_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_event).transpose()
    }

    pub async fn list_matching(
        &self,
        query: EventLogQuery,
    ) -> Result<Vec<StoredEventEnvelope>, EventStoreError> {
        let limit = i64::from(query.limit.unwrap_or(100).clamp(1, 1000));
        let mut builder = QueryBuilder::<Postgres>::new(
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
            WHERE TRUE
            "#,
        );

        if let Some(event_type) = query.event_type {
            builder.push(" AND event_type = ");
            builder.push_bind(event_type);
        }

        if let Some(source_code) = query.source_code {
            builder.push(" AND source ->> 'source_code' = ");
            builder.push_bind(source_code);
        }

        if let Some(subject_kind) = query.subject_kind {
            builder.push(" AND subject ->> 'kind' = ");
            builder.push_bind(subject_kind);
        }

        if let Some(subject_entity_id) = query.subject_entity_id {
            builder.push(" AND subject ->> 'entity_id' = ");
            builder.push_bind(subject_entity_id);
        }

        if let Some(correlation_id) = query.correlation_id {
            builder.push(" AND correlation_id = ");
            builder.push_bind(correlation_id);
        }

        if let Some(position_after) = query.position_after {
            builder.push(" AND position >= ");
            builder.push_bind(position_after);
        }

        if let Some(position_before) = query.position_before {
            builder.push(" AND position <= ");
            builder.push_bind(position_before);
        }

        if let Some(occurred_after) = query.occurred_after {
            builder.push(" AND occurred_at >= ");
            builder.push_bind(occurred_after);
        }

        if let Some(occurred_before) = query.occurred_before {
            builder.push(" AND occurred_at <= ");
            builder.push_bind(occurred_before);
        }

        builder.push(" ORDER BY occurred_at ASC, position ASC LIMIT ");
        builder.push_bind(limit);

        let rows = builder.build().fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_stored_event).collect()
    }
}
