use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::PgPool;
use sqlx::postgres::Postgres;
use thiserror::Error;

use crate::domains::organizations::core::errors::OrgCoreError;
use crate::domains::organizations::core::evidence::link_entity_in_transaction;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Clone, Debug, Serialize)]
pub struct OrgHealth {
    pub organization_id: String,
    pub display_name: String,
    pub health_status: String,
    pub last_health_check: Option<DateTime<Utc>>,
    pub watchlist: bool,
    pub interaction_count: i32,
    pub trust_score: Option<i16>,
    pub open_risks: i64,
    pub overdue_contracts: i64,
}

#[derive(Clone)]
pub struct OrgHealthStore {
    pool: PgPool,
}
impl OrgHealthStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn get(&self, org_id: &str) -> Result<Option<OrgHealth>, OrgHealthError> {
        let row = sqlx::query("SELECT o.organization_id, o.display_name, o.health_status, o.last_health_check, o.watchlist, o.interaction_count, o.trust_score, (SELECT count(*) FROM organization_risks r WHERE r.organization_id=o.organization_id AND r.resolved_at IS NULL) as open_risks, (SELECT count(*) FROM organization_contracts c WHERE c.organization_id=o.organization_id AND c.expires_at < now() AND c.status='active') as overdue_contracts FROM organizations o WHERE o.organization_id=$1")
            .bind(org_id).fetch_optional(&self.pool).await?;
        row.map(|r| {
            Ok(OrgHealth {
                organization_id: r.try_get("organization_id").unwrap_or_default(),
                display_name: r.try_get("display_name").unwrap_or_default(),
                health_status: r
                    .try_get("health_status")
                    .unwrap_or_else(|_| "healthy".into()),
                last_health_check: r.try_get("last_health_check").ok(),
                watchlist: r.try_get("watchlist").unwrap_or(false),
                interaction_count: r.try_get("interaction_count").unwrap_or(0),
                trust_score: r.try_get("trust_score").ok(),
                open_risks: r.try_get("open_risks").unwrap_or(0),
                overdue_contracts: r.try_get("overdue_contracts").unwrap_or(0),
            })
        })
        .transpose()
    }
    pub async fn list_unhealthy(&self) -> Result<Vec<OrgHealth>, OrgHealthError> {
        let rows = sqlx::query("SELECT organization_id, display_name, health_status, last_health_check, watchlist, interaction_count, trust_score FROM organizations WHERE health_status IS NOT NULL AND health_status != 'healthy' ORDER BY interaction_count DESC LIMIT 50")
            .fetch_all(&self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|r| OrgHealth {
                organization_id: r.try_get("organization_id").unwrap_or_default(),
                display_name: r.try_get("display_name").unwrap_or_default(),
                health_status: r
                    .try_get("health_status")
                    .unwrap_or_else(|_| "healthy".into()),
                last_health_check: r.try_get("last_health_check").ok(),
                watchlist: r.try_get("watchlist").unwrap_or(false),
                interaction_count: r.try_get("interaction_count").unwrap_or(0),
                trust_score: r.try_get("trust_score").ok(),
                open_risks: 0,
                overdue_contracts: 0,
            })
            .collect())
    }
    pub async fn toggle_watchlist(&self, org_id: &str) -> Result<bool, OrgHealthError> {
        self.toggle_watchlist_with_source(org_id, &organization_watchlist_source(org_id))
            .await
    }

    pub async fn toggle_watchlist_with_source(
        &self,
        org_id: &str,
        source: &str,
    ) -> Result<bool, OrgHealthError> {
        let mut transaction = self.pool.begin().await?;
        let watchlist =
            Self::toggle_watchlist_in_transaction(&mut transaction, org_id, source).await?;
        transaction.commit().await?;
        Ok(watchlist)
    }

    pub async fn toggle_watchlist_with_observation(
        &self,
        org_id: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<bool, OrgHealthError> {
        let mut transaction = self.pool.begin().await?;
        let watchlist =
            Self::toggle_watchlist_in_transaction(&mut transaction, org_id, source).await?;
        link_entity_in_transaction(
            &mut transaction,
            observation_id,
            "watchlist_toggle",
            org_id,
            json!({
                "watchlist": watchlist
            }),
        )
        .await?;
        transaction.commit().await?;
        Ok(watchlist)
    }

    async fn toggle_watchlist_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        org_id: &str,
        source: &str,
    ) -> Result<bool, OrgHealthError> {
        let row = sqlx::query("UPDATE organizations SET watchlist = NOT watchlist WHERE organization_id=$1 RETURNING watchlist").bind(org_id).fetch_optional(&mut **transaction).await?;
        let Some(row) = row else {
            return Ok(false);
        };
        let watchlist = row.try_get("watchlist").unwrap_or(false);
        sqlx::query(
            "INSERT INTO organization_preferences (organization_id, preference_type, value, source)
             VALUES ($1, 'ui:watchlist', $2, $3)
             ON CONFLICT (organization_id, preference_type)
             DO UPDATE SET value = $2, source = $3, updated_at = now()",
        )
        .bind(org_id)
        .bind(if watchlist { "true" } else { "false" })
        .bind(source)
        .execute(&mut **transaction)
        .await?;
        Ok(watchlist)
    }
}

fn organization_watchlist_source(org_id: &str) -> String {
    format!("organizations.watchlist:{org_id}")
}

#[derive(Clone, Debug, Serialize)]
pub struct OrgRisk {
    pub id: String,
    pub organization_id: String,
    pub risk_type: String,
    pub description: String,
    pub severity: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
}

#[derive(Clone)]
pub struct OrgRiskStore {
    pool: PgPool,
}
impl OrgRiskStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgRisk>, OrgHealthError> {
        let rows = sqlx::query("SELECT id::text, organization_id, risk_type, description, severity, source, confidence::float8 AS confidence, created_at, resolved_at, resolution FROM organization_risks WHERE organization_id=$1 ORDER BY created_at DESC")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgRisk {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    risk_type: r.try_get("risk_type")?,
                    description: r.try_get("description")?,
                    severity: r.try_get("severity")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    created_at: r.try_get("created_at")?,
                    resolved_at: r.try_get("resolved_at")?,
                    resolution: r.try_get("resolution")?,
                })
            })
            .collect()
    }
    pub async fn add(
        &self,
        org_id: &str,
        risk_type: &str,
        desc: &str,
        severity: &str,
        source: &str,
    ) -> Result<OrgRisk, OrgHealthError> {
        let row = sqlx::query("INSERT INTO organization_risks (organization_id, risk_type, description, severity, source) VALUES ($1,$2,$3,$4,$5) RETURNING id::text, organization_id, risk_type, description, severity, source, confidence::float8 AS confidence, created_at, resolved_at, resolution")
            .bind(org_id).bind(risk_type).bind(desc).bind(severity).bind(source).fetch_one(&self.pool).await?;
        Ok(OrgRisk {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            risk_type: row.try_get("risk_type")?,
            description: row.try_get("description")?,
            severity: row.try_get("severity")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            created_at: row.try_get("created_at")?,
            resolved_at: row.try_get("resolved_at")?,
            resolution: row.try_get("resolution")?,
        })
    }
}

#[derive(Debug, Error)]
pub enum OrgHealthError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Core(#[from] OrgCoreError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
}
