use chrono::{DateTime, Utc};
use makosh_events_api::NewEventEnvelope;
use serde_json::json;
use sqlx::Postgres;
use sqlx::postgres::PgPool;

use makosh_events_postgres::errors::EventStoreError;
use makosh_events_postgres::store::EventStore;

use super::errors::PersonaTrustError;
use super::models::PersonaPromise;
use super::rows::row_to_promise;

pub const PERSONA_PROMISE_CREATED_EVENT_TYPE: &str = "persona.promise.created";

#[derive(Clone)]
pub struct PersonaPromiseStore {
    pool: PgPool,
}

impl PersonaPromiseStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, persona_id: &str) -> Result<Vec<PersonaPromise>, PersonaTrustError> {
        let rows = sqlx::query(
            "SELECT id::text, persona_id, description, source_message_id, promised_at,
             due_at, fulfilled_at, status, created_at, updated_at
             FROM persona_promises WHERE persona_id = $1 ORDER BY promised_at DESC",
        )
        .bind(persona_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_promise).collect()
    }

    pub async fn create(
        &self,
        persona_id: &str,
        description: &str,
        due_at: Option<DateTime<Utc>>,
    ) -> Result<PersonaPromise, PersonaTrustError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "INSERT INTO persona_promises (persona_id, description, due_at)
             VALUES ($1, $2, $3)
             RETURNING id::text, persona_id, description, source_message_id, promised_at,
                       due_at, fulfilled_at, status, created_at, updated_at",
        )
        .bind(persona_id)
        .bind(description)
        .bind(due_at)
        .fetch_one(&mut *transaction)
        .await?;
        let promise = row_to_promise(row)?;
        append_promise_created_event(&mut transaction, &promise).await?;
        transaction.commit().await?;

        Ok(promise)
    }

    pub async fn fulfill(&self, id: &str) -> Result<(), PersonaTrustError> {
        sqlx::query(
            "UPDATE persona_promises
             SET status = 'fulfilled', fulfilled_at = now(), updated_at = now()
             WHERE id::text = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_broken(&self, id: &str) -> Result<(), PersonaTrustError> {
        sqlx::query(
            "UPDATE persona_promises SET status = 'broken', updated_at = now() WHERE id::text = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

async fn append_promise_created_event(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    promise: &PersonaPromise,
) -> Result<(), PersonaTrustError> {
    let event = NewEventEnvelope::builder(
        format!("persona_promise_created:{}", promise.id),
        PERSONA_PROMISE_CREATED_EVENT_TYPE,
        promise.promised_at,
        json!({
            "kind": "persona_promise",
            "provider": "makosh",
            "source_id": promise.id,
        }),
        json!({
            "kind": "persona",
            "persona_id": &promise.persona_id,
        }),
    )
    .payload(json!({
        "promise_id": &promise.id,
        "persona_id": &promise.persona_id,
        "description": &promise.description,
        "due_at": promise.due_at,
    }))
    .build()
    .map_err(EventStoreError::from)?;

    match EventStore::append_in_transaction(transaction, &event).await {
        Ok(_) => Ok(()),
        Err(error) if error.is_unique_violation() => Ok(()),
        Err(error) => Err(error.into()),
    }
}
