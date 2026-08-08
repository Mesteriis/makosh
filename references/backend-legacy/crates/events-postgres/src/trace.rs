use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use super::errors::EventStoreError;
use super::rows::row_to_stored_event;
use super::store::EventStore;
use makosh_events_api::{EventEnvelopeError, StoredEventEnvelope};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventTrace {
    pub correlation_id: String,
    pub root_event_ids: Vec<String>,
    pub events: Vec<StoredEventEnvelope>,
    pub edges: Vec<EventTraceEdge>,
    pub orphan_event_ids: Vec<String>,
    pub missing_parent_ids: Vec<String>,
    pub consumer_annotations: Vec<EventConsumerAnnotation>,
    pub dead_letters: Vec<EventDeadLetterAnnotation>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventTraceEdge {
    pub parent_event_id: String,
    pub child_event_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventConsumerAnnotation {
    pub event_id: String,
    pub consumer_name: String,
    pub status: String,
    pub processed_at: Option<DateTime<Utc>>,
    pub attempts: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventDeadLetterAnnotation {
    pub event_id: String,
    pub consumer_name: Option<String>,
    pub reason: String,
    pub failed_at: Option<DateTime<Utc>>,
}

impl EventStore {
    pub async fn first_trace_id_for_subject(
        &self,
        subject_key: &str,
        subject_value: &str,
    ) -> Result<Option<String>, EventStoreError> {
        validate_non_empty("subject_key", subject_key)?;
        validate_non_empty("subject_value", subject_value)?;
        sqlx::query_scalar(
            "SELECT COALESCE(correlation_id, event_id) FROM event_log WHERE subject ->> $1 = $2 ORDER BY position ASC LIMIT 1",
        )
        .bind(subject_key.trim())
        .bind(subject_value.trim())
        .fetch_optional(self.pool())
        .await
        .map_err(EventStoreError::from)
    }

    pub async fn list_by_correlation_id(
        &self,
        correlation_id: &str,
        limit: u32,
    ) -> Result<Vec<StoredEventEnvelope>, EventStoreError> {
        validate_non_empty("correlation_id", correlation_id)?;
        let sql =
            stored_event_select_sql("WHERE correlation_id = $1 ORDER BY position ASC LIMIT $2");
        let rows = sqlx::query(&sql)
            .bind(correlation_id.trim())
            .bind(trace_limit(limit))
            .fetch_all(self.pool())
            .await?;

        rows.into_iter().map(row_to_stored_event).collect()
    }

    pub async fn list_children(
        &self,
        parent_event_id: &str,
        limit: u32,
    ) -> Result<Vec<StoredEventEnvelope>, EventStoreError> {
        validate_non_empty("parent_event_id", parent_event_id)?;
        let sql = stored_event_select_sql("WHERE causation_id = $1 ORDER BY position ASC LIMIT $2");
        let rows = sqlx::query(&sql)
            .bind(parent_event_id.trim())
            .bind(trace_limit(limit))
            .fetch_all(self.pool())
            .await?;

        rows.into_iter().map(row_to_stored_event).collect()
    }

    pub async fn trace_by_event_id(
        &self,
        event_id: &str,
        limit: u32,
    ) -> Result<Option<EventTrace>, EventStoreError> {
        validate_non_empty("event_id", event_id)?;
        let Some(anchor) = self.get_stored_by_id(event_id.trim()).await? else {
            return Ok(None);
        };
        let correlation_id = resolved_correlation_id(&anchor);
        let mut events = self.list_by_correlation_id(&correlation_id, limit).await?;

        if !events
            .iter()
            .any(|event| event.event.event_id == anchor.event.event_id)
        {
            events.push(anchor);
            events.sort_by_key(|event| event.position);
        }

        self.build_trace(correlation_id, events).await.map(Some)
    }

    pub async fn trace_by_correlation_id(
        &self,
        correlation_id: &str,
        limit: u32,
    ) -> Result<EventTrace, EventStoreError> {
        validate_non_empty("correlation_id", correlation_id)?;
        let correlation_id = correlation_id.trim().to_owned();
        let events = self.list_by_correlation_id(&correlation_id, limit).await?;

        self.build_trace(correlation_id, events).await
    }

    async fn get_stored_by_id(
        &self,
        event_id: &str,
    ) -> Result<Option<StoredEventEnvelope>, EventStoreError> {
        let sql = stored_event_select_sql("WHERE event_id = $1");
        let row = sqlx::query(&sql)
            .bind(event_id)
            .fetch_optional(self.pool())
            .await?;

        row.map(row_to_stored_event).transpose()
    }

    async fn build_trace(
        &self,
        correlation_id: String,
        events: Vec<StoredEventEnvelope>,
    ) -> Result<EventTrace, EventStoreError> {
        let positions = events
            .iter()
            .map(|event| event.position)
            .collect::<Vec<_>>();
        let consumer_annotations = self.consumer_annotations_for_positions(&positions).await?;
        let dead_letters = self.dead_letters_for_positions(&positions).await?;
        let event_ids = events
            .iter()
            .map(|event| event.event.event_id.as_str())
            .collect::<HashSet<_>>();

        let mut root_event_ids = Vec::new();
        let mut edges = Vec::new();
        let mut orphan_event_ids = Vec::new();
        let mut missing_parent_ids = Vec::new();
        let mut seen_missing_parent_ids = HashSet::new();

        for stored in &events {
            match stored.event.causation_id.as_deref() {
                None => {
                    root_event_ids.push(stored.event.event_id.clone());
                    if stored.event.correlation_id.is_none() {
                        orphan_event_ids.push(stored.event.event_id.clone());
                    }
                }
                Some(parent_event_id) if event_ids.contains(parent_event_id) => {
                    edges.push(EventTraceEdge {
                        parent_event_id: parent_event_id.to_owned(),
                        child_event_id: stored.event.event_id.clone(),
                    });
                }
                Some(parent_event_id) => {
                    orphan_event_ids.push(stored.event.event_id.clone());
                    if seen_missing_parent_ids.insert(parent_event_id.to_owned()) {
                        missing_parent_ids.push(parent_event_id.to_owned());
                    }
                }
            }
        }

        Ok(EventTrace {
            correlation_id,
            root_event_ids,
            events,
            edges,
            orphan_event_ids,
            missing_parent_ids,
            consumer_annotations,
            dead_letters,
        })
    }

    async fn consumer_annotations_for_positions(
        &self,
        positions: &[i64],
    ) -> Result<Vec<EventConsumerAnnotation>, EventStoreError> {
        if positions.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            SELECT
                event_id,
                consumer_name,
                'processed'::text AS status,
                processed_at,
                NULL::integer AS attempts
            FROM event_consumer_processed_events
            WHERE event_position = ANY($1)
            UNION ALL
            SELECT
                event_id,
                consumer_name,
                'failed'::text AS status,
                NULL::timestamptz AS processed_at,
                attempt_count AS attempts
            FROM event_consumer_failures
            WHERE event_position = ANY($1)
            ORDER BY event_id ASC, consumer_name ASC, status ASC
            "#,
        )
        .bind(positions)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(EventConsumerAnnotation {
                    event_id: row.try_get("event_id")?,
                    consumer_name: row.try_get("consumer_name")?,
                    status: row.try_get("status")?,
                    processed_at: row.try_get("processed_at")?,
                    attempts: row.try_get("attempts")?,
                })
            })
            .collect()
    }

    async fn dead_letters_for_positions(
        &self,
        positions: &[i64],
    ) -> Result<Vec<EventDeadLetterAnnotation>, EventStoreError> {
        if positions.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            SELECT
                event_id,
                consumer_name,
                last_error AS reason,
                created_at AS failed_at
            FROM event_dead_letters
            WHERE event_position = ANY($1)
            ORDER BY event_id ASC, consumer_name ASC
            "#,
        )
        .bind(positions)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(EventDeadLetterAnnotation {
                    event_id: row.try_get("event_id")?,
                    consumer_name: Some(row.try_get("consumer_name")?),
                    reason: row.try_get("reason")?,
                    failed_at: row.try_get("failed_at")?,
                })
            })
            .collect()
    }
}

fn resolved_correlation_id(event: &StoredEventEnvelope) -> String {
    event
        .event
        .correlation_id
        .clone()
        .unwrap_or_else(|| event.event.event_id.clone())
}

fn trace_limit(limit: u32) -> i64 {
    i64::from(limit.clamp(1, 5000))
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), EventStoreError> {
    if value.trim().is_empty() {
        return Err(EventStoreError::Envelope(EventEnvelopeError::EmptyField(
            field_name,
        )));
    }

    Ok(())
}

fn stored_event_select_sql(where_clause: &str) -> String {
    format!(
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
        {where_clause}
        "#
    )
}
