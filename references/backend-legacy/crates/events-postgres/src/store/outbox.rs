use chrono::{DateTime, Duration, Utc};
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::EventStore;
use crate::errors::EventStoreError;
use makosh_events_api::{
    DispatchableEventOutboxItem, EventEnvelope, EventOutboxItem, NewEventEnvelope,
};

impl EventStore {
    pub async fn append_for_dispatch(
        &self,
        event: &NewEventEnvelope,
    ) -> Result<i64, EventStoreError> {
        let mut transaction = self.pool.begin().await?;
        let position = Self::append_in_transaction(&mut transaction, event).await?;

        sqlx::query(
            r#"
            INSERT INTO event_outbox (event_id, subject)
            VALUES ($1, $2)
            ON CONFLICT (event_id) DO NOTHING
            "#,
        )
        .bind(&event.event_id)
        .bind(&event.event_type)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(position)
    }

    pub async fn append_for_dispatch_idempotent(
        &self,
        event: &NewEventEnvelope,
    ) -> Result<Option<i64>, EventStoreError> {
        match self.append_for_dispatch(event).await {
            Ok(position) => Ok(Some(position)),
            Err(error) if error.is_unique_violation() => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub async fn pending_outbox_batch(
        &self,
        limit: u32,
    ) -> Result<Vec<EventOutboxItem>, EventStoreError> {
        let limit = i64::from(limit.clamp(1, 1000));
        let rows = sqlx::query(
            r#"
            SELECT event_id, subject, status, attempts
            FROM event_outbox
            WHERE status = 'pending'
              AND next_attempt_at <= now()
            ORDER BY created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_outbox_item).collect()
    }

    pub async fn recover_stale_outbox_items(
        &self,
        stale_dispatch_after: Duration,
    ) -> Result<u32, EventStoreError> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE event_outbox
            SET
                status = 'pending',
                next_attempt_at = now(),
                last_error_redacted = COALESCE(last_error_redacted, 'dispatcher recovered stale dispatch lease'),
                updated_at = now()
            WHERE status = 'dispatching'
              AND updated_at <= $1
            "#,
        )
        .bind(Utc::now() - stale_dispatch_after)
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(u32::try_from(rows_affected).unwrap_or(u32::MAX))
    }

    pub async fn claim_pending_outbox_batch(
        &self,
        limit: u32,
    ) -> Result<Vec<DispatchableEventOutboxItem>, EventStoreError> {
        let limit = i64::from(limit.clamp(1, 1000));
        let mut transaction = self.pool.begin().await?;
        let rows = sqlx::query(
            r#"
            WITH candidates AS (
                SELECT event_id
                FROM event_outbox
                WHERE status = 'pending'
                  AND next_attempt_at <= now()
                ORDER BY created_at ASC
                LIMIT $1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE event_outbox AS outbox
            SET
                status = 'dispatching',
                attempts = outbox.attempts + 1,
                last_error_redacted = NULL,
                updated_at = now()
            FROM candidates
            JOIN event_log ON event_log.event_id = candidates.event_id
            WHERE outbox.event_id = candidates.event_id
            RETURNING
                outbox.event_id,
                outbox.subject AS outbox_subject,
                outbox.attempts,
                event_log.event_id AS log_event_id,
                event_log.event_type,
                event_log.schema_version,
                event_log.occurred_at,
                event_log.recorded_at,
                event_log.source,
                event_log.actor,
                event_log.subject,
                event_log.payload,
                event_log.provenance,
                event_log.causation_id,
                event_log.correlation_id
            "#,
        )
        .bind(limit)
        .fetch_all(&mut *transaction)
        .await?;

        transaction.commit().await?;
        rows.into_iter()
            .map(row_to_dispatchable_outbox_item)
            .collect()
    }

    pub async fn mark_outbox_published(&self, event_id: &str) -> Result<(), EventStoreError> {
        sqlx::query(
            r#"
            UPDATE event_outbox
            SET
                status = 'published',
                published_at = now(),
                updated_at = now()
            WHERE event_id = $1
            "#,
        )
        .bind(event_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn mark_outbox_retry(
        &self,
        event_id: &str,
        error: &str,
        next_attempt_at: DateTime<Utc>,
    ) -> Result<(), EventStoreError> {
        sqlx::query(
            r#"
            UPDATE event_outbox
            SET
                status = 'pending',
                next_attempt_at = $2,
                last_error_redacted = $3,
                updated_at = now()
            WHERE event_id = $1
            "#,
        )
        .bind(event_id)
        .bind(next_attempt_at)
        .bind(truncate_redacted_error(error))
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

fn row_to_outbox_item(row: PgRow) -> Result<EventOutboxItem, EventStoreError> {
    Ok(EventOutboxItem {
        event_id: row.try_get("event_id")?,
        subject: row.try_get("subject")?,
        status: row.try_get("status")?,
        attempts: row.try_get("attempts")?,
    })
}

fn row_to_dispatchable_outbox_item(
    row: PgRow,
) -> Result<DispatchableEventOutboxItem, EventStoreError> {
    Ok(DispatchableEventOutboxItem {
        event_id: row.try_get("event_id")?,
        subject: row.try_get("outbox_subject")?,
        attempts: row.try_get("attempts")?,
        event: EventEnvelope {
            event_id: row.try_get("log_event_id")?,
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
        },
    })
}

fn truncate_redacted_error(error: &str) -> String {
    let trimmed = error.trim();
    if trimmed.chars().count() <= 500 {
        return trimmed.to_owned();
    }

    trimmed.chars().take(500).collect()
}
