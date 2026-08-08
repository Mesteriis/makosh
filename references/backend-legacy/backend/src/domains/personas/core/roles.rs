use chrono::{DateTime, Utc};
use makosh_events_api::NewEventEnvelope;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

use makosh_events_postgres::errors::EventStoreError;
use makosh_events_postgres::store::EventStore;

use super::errors::PersonaCoreError;
use super::evidence::link_persona_entity_in_transaction;

pub const PERSONA_ROLE_ASSIGNED_EVENT_TYPE: &str = "persona.role.assigned";
pub const PERSONA_ROLE_REMOVED_EVENT_TYPE: &str = "persona.role.removed";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonaRole {
    pub id: String,
    #[serde(alias = "person_id")]
    pub persona_id: String,
    pub role: String,
    pub assigned_by: Option<String>,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonaRoleStore {
    pool: PgPool,
}

impl PersonaRoleStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_person(
        &self,
        persona_id: &str,
    ) -> Result<Vec<PersonaRole>, PersonaCoreError> {
        let rows = sqlx::query(
            r#"SELECT id::text, persona_id, role, assigned_by, assigned_at
               FROM persona_roles WHERE persona_id = $1 ORDER BY assigned_at"#,
        )
        .bind(persona_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_role).collect()
    }

    pub async fn assign(
        &self,
        persona_id: &str,
        role: &str,
        assigned_by: Option<&str>,
    ) -> Result<PersonaRole, PersonaCoreError> {
        self.assign_with_observation(persona_id, role, assigned_by, None)
            .await
    }

    pub async fn assign_with_observation(
        &self,
        persona_id: &str,
        role: &str,
        assigned_by: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<PersonaRole, PersonaCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"INSERT INTO persona_roles (persona_id, role, assigned_by)
               VALUES ($1, $2, $3)
               ON CONFLICT (persona_id, role) DO UPDATE SET assigned_by = EXCLUDED.assigned_by
               RETURNING id::text, persona_id, role, assigned_by, assigned_at"#,
        )
        .bind(persona_id)
        .bind(role)
        .bind(assigned_by)
        .fetch_one(&mut *transaction)
        .await?;
        let role = row_to_role(row)?;

        if let Some(observation_id) = observation_id {
            link_persona_entity_in_transaction(
                &mut transaction,
                observation_id,
                "role",
                role.id.clone(),
                None,
                Some(json!({
                    "persona_id": persona_id,
                    "role": &role.role,
                    "action": "assign",
                })),
            )
            .await?;
        }
        append_role_assigned_event(&mut transaction, &role).await?;
        transaction.commit().await?;

        Ok(role)
    }

    pub async fn remove(&self, persona_id: &str, role: &str) -> Result<bool, PersonaCoreError> {
        self.remove_with_observation(persona_id, role, None).await
    }

    pub async fn remove_with_observation(
        &self,
        persona_id: &str,
        role: &str,
        observation_id: Option<&str>,
    ) -> Result<bool, PersonaCoreError> {
        let mut transaction = self.pool.begin().await?;
        let existing_role = sqlx::query(
            r#"SELECT id::text, persona_id, role, assigned_by, assigned_at
               FROM persona_roles
               WHERE persona_id = $1 AND role = $2
               FOR UPDATE"#,
        )
        .bind(persona_id)
        .bind(role)
        .fetch_optional(&mut *transaction)
        .await?
        .map(row_to_role)
        .transpose()?;

        let result = sqlx::query("DELETE FROM persona_roles WHERE persona_id = $1 AND role = $2")
            .bind(persona_id)
            .bind(role)
            .execute(&mut *transaction)
            .await?;
        let removed = result.rows_affected() > 0;

        if let Some(existing_role) = existing_role.as_ref()
            && removed
            && let Some(observation_id) = observation_id
        {
            link_persona_entity_in_transaction(
                &mut transaction,
                observation_id,
                "role",
                format!("{persona_id}:{role}"),
                None,
                Some(json!({
                    "persona_id": &existing_role.persona_id,
                    "role": &existing_role.role,
                    "action": "delete",
                    "deleted": removed,
                })),
            )
            .await?;
        }

        if let Some(existing_role) = existing_role.as_ref()
            && removed
        {
            append_role_removed_event(&mut transaction, existing_role).await?;
        }

        transaction.commit().await?;

        Ok(removed)
    }
}

fn row_to_role(row: PgRow) -> Result<PersonaRole, PersonaCoreError> {
    Ok(PersonaRole {
        id: row.try_get("id")?,
        persona_id: row.try_get("persona_id")?,
        role: row.try_get("role")?,
        assigned_by: row.try_get("assigned_by")?,
        assigned_at: row.try_get("assigned_at")?,
    })
}

pub(crate) fn persona_role_knowledge_id(role: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = false;

    for character in role.trim().to_ascii_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_was_separator = false;
        } else if !slug.is_empty() && !previous_was_separator {
            slug.push('_');
            previous_was_separator = true;
        }
    }

    while slug.ends_with('_') {
        slug.pop();
    }

    if slug.is_empty() {
        "persona_role:unspecified".to_owned()
    } else {
        format!("persona_role:{slug}")
    }
}

async fn append_role_assigned_event(
    transaction: &mut Transaction<'_, Postgres>,
    role: &PersonaRole,
) -> Result<(), PersonaCoreError> {
    let role_knowledge_id = persona_role_knowledge_id(&role.role);
    let event = NewEventEnvelope::builder(
        format!(
            "persona_role_assigned:{}:{role_knowledge_id}",
            role.persona_id
        ),
        PERSONA_ROLE_ASSIGNED_EVENT_TYPE,
        role.assigned_at,
        json!({
            "kind": "personas",
            "provider": "makosh",
            "source_id": &role.persona_id,
        }),
        json!({
            "kind": "persona",
            "persona_id": &role.persona_id,
        }),
    )
    .payload(json!({
        "persona_id": &role.persona_id,
        "role": &role.role,
        "assigned_by": &role.assigned_by,
        "role_knowledge_id": role_knowledge_id,
    }))
    .build()
    .map_err(EventStoreError::from)?;

    match EventStore::append_in_transaction(transaction, &event).await {
        Ok(_) => Ok(()),
        Err(error) if error.is_unique_violation() => Ok(()),
        Err(error) => Err(error.into()),
    }
}

async fn append_role_removed_event(
    transaction: &mut Transaction<'_, Postgres>,
    role: &PersonaRole,
) -> Result<(), PersonaCoreError> {
    let role_knowledge_id = persona_role_knowledge_id(&role.role);
    let event = NewEventEnvelope::builder(
        format!("persona_role_removed:{}", Uuid::now_v7()),
        PERSONA_ROLE_REMOVED_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "personas",
            "provider": "makosh",
            "source_id": &role.persona_id,
        }),
        json!({
            "kind": "persona",
            "persona_id": &role.persona_id,
        }),
    )
    .payload(json!({
        "persona_id": &role.persona_id,
        "role": &role.role,
        "assigned_by": &role.assigned_by,
        "role_knowledge_id": role_knowledge_id,
    }))
    .build()
    .map_err(EventStoreError::from)?;

    EventStore::append_in_transaction(transaction, &event).await?;
    Ok(())
}
