use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::personas::core::evidence::link_persona_entity_in_transaction;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Clone, Debug, Serialize)]
pub struct PersonaHealth {
    #[serde(rename = "persona_id")]
    pub persona_id: String,
    pub health_status: String,
    pub last_health_check: Option<DateTime<Utc>>,
    pub communication_gap_days: i32,
    pub watchlist: bool,
    pub interaction_count: i32,
    pub last_interaction_at: Option<DateTime<Utc>>,
    pub trust_score: Option<i16>,
    pub open_promises: i64,
    pub open_risks: i64,
}

#[derive(Clone)]
pub struct PersonaHealthStore {
    pool: PgPool,
}

impl PersonaHealthStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, persona_id: &str) -> Result<Option<PersonaHealth>, PersonaHealthError> {
        let row = sqlx::query(
            r#"SELECT p.persona_id, p.health_status, p.last_health_check, p.communication_gap_days,
               p.watchlist, p.interaction_count, p.last_interaction_at, p.trust_score,
               (SELECT count(*) FROM persona_promises pp WHERE pp.persona_id = p.persona_id AND pp.status = 'pending') as open_promises,
               (SELECT count(*) FROM persona_risks pr WHERE pr.persona_id = p.persona_id AND pr.resolved_at IS NULL) as open_risks
               FROM personas p WHERE p.persona_id = $1"#
        ).bind(persona_id).fetch_optional(&self.pool).await?;
        row.map(|r| PersonaHealth {
            persona_id: r.try_get("persona_id").unwrap_or_default(),
            health_status: r
                .try_get("health_status")
                .unwrap_or_else(|_| "healthy".into()),
            last_health_check: r.try_get("last_health_check").ok(),
            communication_gap_days: r.try_get("communication_gap_days").unwrap_or(0),
            watchlist: r.try_get("watchlist").unwrap_or(false),
            interaction_count: r.try_get("interaction_count").unwrap_or(0),
            last_interaction_at: r.try_get("last_interaction_at").ok(),
            trust_score: r.try_get("trust_score").ok(),
            open_promises: r.try_get("open_promises").unwrap_or(0),
            open_risks: r.try_get("open_risks").unwrap_or(0),
        })
        .map_or(Ok(None), |h| Ok(Some(h)))
    }

    pub async fn list_health(&self) -> Result<Vec<PersonaHealth>, PersonaHealthError> {
        let rows = sqlx::query(
            r#"SELECT p.persona_id, p.health_status, p.last_health_check, p.communication_gap_days,
               p.watchlist, p.interaction_count, p.last_interaction_at, p.trust_score,
               0::bigint as open_promises, 0::bigint as open_risks
               FROM personas p WHERE p.health_status != 'healthy' ORDER BY p.last_interaction_at DESC NULLS LAST LIMIT 50"#
        ).fetch_all(&self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|r| PersonaHealth {
                persona_id: r.try_get("persona_id").unwrap_or_default(),
                health_status: r
                    .try_get("health_status")
                    .unwrap_or_else(|_| "healthy".into()),
                last_health_check: r.try_get("last_health_check").ok(),
                communication_gap_days: r.try_get("communication_gap_days").unwrap_or(0),
                watchlist: r.try_get("watchlist").unwrap_or(false),
                interaction_count: r.try_get("interaction_count").unwrap_or(0),
                last_interaction_at: r.try_get("last_interaction_at").ok(),
                trust_score: r.try_get("trust_score").ok(),
                open_promises: r.try_get("open_promises").unwrap_or(0),
                open_risks: r.try_get("open_risks").unwrap_or(0),
            })
            .collect())
    }

    pub async fn list_watchlist(&self) -> Result<Vec<PersonaHealth>, PersonaHealthError> {
        let rows = sqlx::query(
            r#"SELECT p.persona_id, p.health_status, p.last_health_check, p.communication_gap_days,
               p.watchlist, p.interaction_count, p.last_interaction_at, p.trust_score,
               0::bigint as open_promises, 0::bigint as open_risks
               FROM personas p WHERE p.watchlist = true ORDER BY p.trust_score DESC NULLS LAST"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| PersonaHealth {
                persona_id: r.try_get("persona_id").unwrap_or_default(),
                health_status: r
                    .try_get("health_status")
                    .unwrap_or_else(|_| "healthy".into()),
                last_health_check: r.try_get("last_health_check").ok(),
                communication_gap_days: r.try_get("communication_gap_days").unwrap_or(0),
                watchlist: r.try_get("watchlist").unwrap_or(false),
                interaction_count: r.try_get("interaction_count").unwrap_or(0),
                last_interaction_at: r.try_get("last_interaction_at").ok(),
                trust_score: r.try_get("trust_score").ok(),
                open_promises: r.try_get("open_promises").unwrap_or(0),
                open_risks: r.try_get("open_risks").unwrap_or(0),
            })
            .collect())
    }

    pub async fn toggle_watchlist(&self, persona_id: &str) -> Result<bool, PersonaHealthError> {
        self.toggle_watchlist_with_source(persona_id, &persona_watchlist_source(persona_id))
            .await
    }

    pub async fn toggle_watchlist_with_source(
        &self,
        persona_id: &str,
        source: &str,
    ) -> Result<bool, PersonaHealthError> {
        let mut transaction = self.pool.begin().await?;
        let watchlist =
            Self::toggle_watchlist_in_transaction(&mut transaction, persona_id, source).await?;
        transaction.commit().await?;
        Ok(watchlist)
    }

    pub async fn toggle_watchlist_with_observation(
        &self,
        persona_id: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<bool, PersonaHealthError> {
        let mut transaction = self.pool.begin().await?;
        let watchlist =
            Self::toggle_watchlist_in_transaction(&mut transaction, persona_id, source).await?;
        link_persona_entity_in_transaction(
            &mut transaction,
            observation_id,
            "watchlist_toggle",
            persona_id,
            None,
            Some(json!({
                "watchlist": watchlist
            })),
        )
        .await?;
        transaction.commit().await?;
        Ok(watchlist)
    }

    async fn toggle_watchlist_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        persona_id: &str,
        source: &str,
    ) -> Result<bool, PersonaHealthError> {
        let row = sqlx::query(
            "UPDATE personas SET watchlist = NOT watchlist WHERE persona_id = $1 RETURNING watchlist",
        )
        .bind(persona_id)
        .fetch_optional(&mut **transaction)
        .await?;
        let Some(row) = row else {
            return Ok(false);
        };
        let watchlist = row.try_get("watchlist").unwrap_or(false);
        sync_watchlist_preference_in_transaction(transaction, persona_id, watchlist, source)
            .await?;
        Ok(watchlist)
    }
}

async fn sync_watchlist_preference_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    persona_id: &str,
    watchlist: bool,
    source: &str,
) -> Result<(), PersonaHealthError> {
    if watchlist {
        sqlx::query(
            "INSERT INTO persona_preferences (persona_id, preference_type, value, source, confidence)
             VALUES ($1, 'ui:watchlist', 'true', $2, 1.0)
             ON CONFLICT (persona_id, preference_type)
             DO UPDATE SET value = 'true', source = $2, confidence = 1.0, updated_at = now()",
        )
        .bind(persona_id)
        .bind(source)
        .execute(&mut **transaction)
        .await?;
        return Ok(());
    }

    sqlx::query(
        "DELETE FROM persona_preferences WHERE persona_id = $1 AND preference_type = 'ui:watchlist'",
    )
    .bind(persona_id)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

fn persona_watchlist_source(persona_id: &str) -> String {
    format!("personas.watchlist:{persona_id}")
}

#[derive(Debug, Error)]
pub enum PersonaHealthError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
}
