use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Transaction};

use super::errors::CalendarError;
use super::models::CalendarSource;
use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;

#[derive(Clone)]
pub struct CalendarSourceStore {
    pool: PgPool,
}

impl CalendarSourceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        account_id: &str,
        name: &str,
        provider_calendar_id: Option<&str>,
        color: Option<&str>,
        timezone: Option<&str>,
    ) -> Result<CalendarSource, CalendarError> {
        self.create_with_observation(
            account_id,
            name,
            provider_calendar_id,
            color,
            timezone,
            None,
            "create",
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_with_observation(
        &self,
        account_id: &str,
        name: &str,
        provider_calendar_id: Option<&str>,
        color: Option<&str>,
        timezone: Option<&str>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CalendarSource, CalendarError> {
        let source_id = next_id("src");
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "INSERT INTO calendar_sources (source_id, account_id, provider_calendar_id, name, color, timezone) VALUES ($1,$2,$3,$4,$5,$6) RETURNING source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at",
        )
        .bind(&source_id)
        .bind(account_id)
        .bind(provider_calendar_id)
        .bind(name)
        .bind(color)
        .bind(timezone)
        .fetch_one(&mut *transaction)
        .await?;
        let source = row_to_calendar_source(row).map_err(CalendarError::from)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                VaultOwnedEntityLink {
                    observation_id: observation_id.to_owned(),
                    domain: "calendar",
                    entity_kind: "calendar_source",
                    entity_id: source.source_id.clone(),
                    relationship_kind: relationship_kind.to_owned(),
                    base_metadata: json!({
                        "source_id": source.source_id,
                        "account_id": source.account_id,
                    }),
                    extra_metadata: metadata,
                },
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(source)
    }

    pub async fn list_by_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<CalendarSource>, CalendarError> {
        let rows = sqlx::query("SELECT source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at FROM calendar_sources WHERE account_id=$1 ORDER BY name")
            .bind(account_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(row_to_calendar_source)
            .collect::<Result<Vec<_>, _>>()
            .map_err(CalendarError::from)
    }

    pub async fn get(&self, source_id: &str) -> Result<Option<CalendarSource>, CalendarError> {
        let row = sqlx::query("SELECT source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at FROM calendar_sources WHERE source_id=$1")
            .bind(source_id).fetch_optional(&self.pool).await?;
        row.map(row_to_calendar_source)
            .transpose()
            .map_err(CalendarError::from)
    }
}
struct VaultOwnedEntityLink {
    observation_id: String,
    domain: &'static str,
    entity_kind: &'static str,
    entity_id: String,
    relationship_kind: String,
    base_metadata: serde_json::Value,
    extra_metadata: Option<serde_json::Value>,
}

async fn link_vault_owned_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    request: VaultOwnedEntityLink,
) -> Result<(), makosh_observations_postgres::errors::ObservationStoreError> {
    let metadata = match request.extra_metadata {
        Some(extra) if request.base_metadata.is_object() && extra.is_object() => {
            let mut merged = request.base_metadata;
            if let (Some(base), Some(extra)) = (merged.as_object_mut(), extra.as_object()) {
                for (key, value) in extra {
                    base.insert(key.clone(), value.clone());
                }
            }
            merged
        }
        Some(extra) => extra,
        None => request.base_metadata,
    };

    link_domain_entity_in_transaction(
        transaction,
        &request.observation_id,
        request.domain,
        request.entity_kind,
        request.entity_id,
        Some(&request.relationship_kind),
        None,
        Some(metadata),
    )
    .await
}
fn row_to_calendar_source(row: PgRow) -> Result<CalendarSource, sqlx::Error> {
    Ok(CalendarSource {
        source_id: row.try_get("source_id")?,
        account_id: row.try_get("account_id")?,
        provider_calendar_id: row.try_get("provider_calendar_id")?,
        name: row.try_get("name")?,
        color: row.try_get("color")?,
        timezone: row.try_get("timezone")?,
        visibility: row.try_get("visibility")?,
        read_only: row.try_get("read_only")?,
        sync_enabled: row.try_get("sync_enabled")?,
        capabilities: row.try_get("capabilities")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
fn next_id(prefix: &str) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}:v1:{ts:x}")
}
