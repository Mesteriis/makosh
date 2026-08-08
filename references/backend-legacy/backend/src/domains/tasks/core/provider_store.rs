use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::TaskCoreError;
use super::providers::TaskProviderAccount;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;
use makosh_observations_postgres::store::ObservationStore;

pub struct TaskProviderStore {
    pool: PgPool,
}

impl TaskProviderStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<TaskProviderAccount>, TaskCoreError> {
        let rows = sqlx::query(
            r#"
            SELECT account_id, provider, account_name, credentials_reference,
                   sync_mode, capabilities, created_at, updated_at
            FROM task_provider_accounts
            ORDER BY provider, account_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_task_provider_account).collect()
    }

    pub async fn create(
        &self,
        provider: &str,
        account_name: &str,
    ) -> Result<TaskProviderAccount, TaskCoreError> {
        self.create_with_origin(
            provider,
            account_name,
            ObservationOriginKind::LocalRuntime,
            "tasks_api.post_task_provider",
        )
        .await
    }

    pub async fn create_with_origin(
        &self,
        provider: &str,
        account_name: &str,
        origin_kind: ObservationOriginKind,
        actor: &str,
    ) -> Result<TaskProviderAccount, TaskCoreError> {
        let account_id = next_id("tprov");
        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "TASK_PROVIDER_ACCOUNT",
                origin_kind,
                chrono::Utc::now(),
                json!({
                    "account_id": account_id,
                    "provider": provider,
                    "account_name": account_name,
                    "action": "create_task_provider_account",
                }),
                format!("task-provider://{account_id}"),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": "create_task_provider_account",
            })),
        )
        .await?;
        let row = sqlx::query(
            r#"
            INSERT INTO task_provider_accounts (account_id, provider, account_name)
            VALUES ($1, $2, $3)
            RETURNING account_id, provider, account_name, credentials_reference,
                      sync_mode, capabilities, created_at, updated_at
            "#,
        )
        .bind(&account_id)
        .bind(provider)
        .bind(account_name)
        .fetch_one(&mut *transaction)
        .await?;
        link_domain_owned_entity_in_transaction(
            &mut transaction,
            &observation.observation_id,
            "tasks",
            "task_provider_account",
            account_id.clone(),
            "create",
            json!({
                "provider": provider,
                "account_name": account_name,
            }),
            None,
        )
        .await?;
        transaction.commit().await?;

        row_to_task_provider_account(row)
    }
}

fn row_to_task_provider_account(row: PgRow) -> Result<TaskProviderAccount, TaskCoreError> {
    Ok(TaskProviderAccount {
        account_id: row.try_get("account_id")?,
        provider: row.try_get("provider")?,
        account_name: row.try_get("account_name")?,
        credentials_reference: row.try_get("credentials_reference")?,
        sync_mode: row.try_get("sync_mode")?,
        capabilities: row.try_get("capabilities")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
async fn link_domain_owned_entity_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    observation_id: &str,
    domain: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: &str,
    base_metadata: serde_json::Value,
    extra_metadata: Option<serde_json::Value>,
) -> Result<(), makosh_observations_postgres::errors::ObservationStoreError> {
    let metadata = match extra_metadata {
        Some(extra) if base_metadata.is_object() && extra.is_object() => {
            let mut merged = base_metadata;
            if let (Some(base), Some(extra)) = (merged.as_object_mut(), extra.as_object()) {
                for (key, value) in extra {
                    base.insert(key.clone(), value.clone());
                }
            }
            merged
        }
        Some(extra) => extra,
        None => base_metadata,
    };

    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        domain,
        entity_kind,
        entity_id.into(),
        Some(relationship_kind),
        None,
        Some(metadata),
    )
    .await
}

fn next_id(prefix: &str) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}:v1:{ts:x}")
}
