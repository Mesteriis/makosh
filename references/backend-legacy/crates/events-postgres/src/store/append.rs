use sqlx::{Postgres, Transaction};

use super::EventStore;
use crate::errors::EventStoreError;
use makosh_events_api::NewEventEnvelope;

impl EventStore {
    pub async fn append(&self, event: &NewEventEnvelope) -> Result<i64, EventStoreError> {
        let mut transaction = self.pool.begin().await?;
        let position = Self::append_in_transaction(&mut transaction, event).await?;
        transaction.commit().await?;

        Ok(position)
    }

    pub async fn append_idempotent(
        &self,
        event: &NewEventEnvelope,
    ) -> Result<Option<i64>, EventStoreError> {
        match self.append(event).await {
            Ok(position) => Ok(Some(position)),
            Err(error) if error.is_unique_violation() => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub async fn append_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event: &NewEventEnvelope,
    ) -> Result<i64, EventStoreError> {
        let position = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO event_log (
                event_id,
                event_type,
                schema_version,
                occurred_at,
                source,
                actor,
                subject,
                payload,
                provenance,
                causation_id,
                correlation_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING position
            "#,
        )
        .bind(&event.event_id)
        .bind(&event.event_type)
        .bind(event.schema_version)
        .bind(event.occurred_at)
        .bind(&event.source)
        .bind(&event.actor)
        .bind(&event.subject)
        .bind(&event.payload)
        .bind(&event.provenance)
        .bind(&event.causation_id)
        .bind(&event.correlation_id)
        .fetch_one(&mut **transaction)
        .await?;

        Ok(position)
    }
}
