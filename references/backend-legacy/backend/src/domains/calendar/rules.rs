use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use makosh_observations_postgres::errors::ObservationStoreError;

use super::evidence::link_calendar_entity_in_transaction;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarRule {
    pub rule_id: String,
    pub name: String,
    pub natural_language_description: Option<String>,
    pub compiled_dsl: Value,
    pub enabled: bool,
    pub approval_mode: String,
    pub last_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CalendarRuleStore {
    pool: PgPool,
}

impl CalendarRuleStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<CalendarRule>, CalendarRuleError> {
        let rows = sqlx::query("SELECT rule_id, name, natural_language_description, compiled_dsl, enabled, approval_mode, last_run_at, created_at, updated_at FROM calendar_rules ORDER BY name")
            .fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(CalendarRule {
                    rule_id: r.try_get("rule_id")?,
                    name: r.try_get("name")?,
                    natural_language_description: r.try_get("natural_language_description")?,
                    compiled_dsl: r.try_get("compiled_dsl")?,
                    enabled: r.try_get("enabled")?,
                    approval_mode: r.try_get("approval_mode")?,
                    last_run_at: r.try_get("last_run_at")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn create(
        &self,
        name: &str,
        description: Option<&str>,
        dsl: Value,
        approval_mode: Option<&str>,
    ) -> Result<CalendarRule, CalendarRuleError> {
        self.create_with_observation(name, description, dsl, approval_mode, None, "create", None)
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_with_observation(
        &self,
        name: &str,
        description: Option<&str>,
        dsl: Value,
        approval_mode: Option<&str>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<CalendarRule, CalendarRuleError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let rule_id = format!("rule:v1:{ts:x}");
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("INSERT INTO calendar_rules (rule_id, name, natural_language_description, compiled_dsl, approval_mode) VALUES ($1,$2,$3,$4,$5) RETURNING rule_id, name, natural_language_description, compiled_dsl, enabled, approval_mode, last_run_at, created_at, updated_at")
            .bind(&rule_id).bind(name).bind(description).bind(&dsl).bind(approval_mode.unwrap_or("suggest_only")).fetch_one(&mut *transaction).await?;
        let rule = CalendarRule {
            rule_id: row.try_get("rule_id")?,
            name: row.try_get("name")?,
            natural_language_description: row.try_get("natural_language_description")?,
            compiled_dsl: row.try_get("compiled_dsl")?,
            enabled: row.try_get("enabled")?,
            approval_mode: row.try_get("approval_mode")?,
            last_run_at: row.try_get("last_run_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                &mut transaction,
                observation_id,
                "calendar_rule",
                rule.rule_id.clone(),
                Some(relationship_kind),
                serde_json::json!({
                    "rule_id": rule.rule_id,
                    "approval_mode": rule.approval_mode,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(rule)
    }

    pub async fn update(
        &self,
        rule_id: &str,
        update: &RuleUpdate,
    ) -> Result<CalendarRule, CalendarRuleError> {
        self.update_with_observation(rule_id, update, None, "update", None)
            .await
    }

    pub async fn update_with_observation(
        &self,
        rule_id: &str,
        update: &RuleUpdate,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<CalendarRule, CalendarRuleError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("UPDATE calendar_rules SET name=COALESCE($2,name), natural_language_description=COALESCE($3,natural_language_description), compiled_dsl=COALESCE($4,compiled_dsl), enabled=COALESCE($5,enabled), approval_mode=COALESCE($6,approval_mode), updated_at=now() WHERE rule_id=$1 RETURNING rule_id, name, natural_language_description, compiled_dsl, enabled, approval_mode, last_run_at, created_at, updated_at")
            .bind(rule_id).bind(update.name.as_deref()).bind(update.description.as_deref()).bind(update.dsl.as_ref()).bind(update.enabled).bind(update.approval_mode.as_deref()).fetch_one(&mut *transaction).await?;
        let rule = CalendarRule {
            rule_id: row.try_get("rule_id")?,
            name: row.try_get("name")?,
            natural_language_description: row.try_get("natural_language_description")?,
            compiled_dsl: row.try_get("compiled_dsl")?,
            enabled: row.try_get("enabled")?,
            approval_mode: row.try_get("approval_mode")?,
            last_run_at: row.try_get("last_run_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                &mut transaction,
                observation_id,
                "calendar_rule",
                rule.rule_id.clone(),
                Some(relationship_kind),
                serde_json::json!({
                    "rule_id": rule.rule_id,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(rule)
    }

    pub async fn delete(&self, rule_id: &str) -> Result<(), CalendarRuleError> {
        self.delete_with_observation(rule_id, None, "delete", None)
            .await
    }

    pub async fn delete_with_observation(
        &self,
        rule_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<(), CalendarRuleError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query("DELETE FROM calendar_rules WHERE rule_id=$1")
            .bind(rule_id)
            .execute(&mut *transaction)
            .await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                &mut transaction,
                observation_id,
                "calendar_rule",
                rule_id.to_owned(),
                Some(relationship_kind),
                serde_json::json!({
                    "rule_id": rule_id,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RuleUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub dsl: Option<Value>,
    pub enabled: Option<bool>,
    pub approval_mode: Option<String>,
}

#[derive(Debug, Error)]
pub enum CalendarRuleError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("not found")]
    NotFound,
}
