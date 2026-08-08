use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Transaction};

use super::errors::CalendarError;
use super::models::{CalendarAccount, CalendarAccountUpdate};
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;
use makosh_observations_postgres::store::ObservationStore;

#[derive(Clone)]
pub struct CalendarAccountStore {
    pool: PgPool,
}

impl CalendarAccountStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        provider: &str,
        account_name: &str,
        email: Option<&str>,
    ) -> Result<CalendarAccount, CalendarError> {
        self.create_with_observation(provider, account_name, email, None, "create", None)
            .await
    }

    pub async fn create_with_observation(
        &self,
        provider: &str,
        account_name: &str,
        email: Option<&str>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CalendarAccount, CalendarError> {
        let account_id = next_id("cal");
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "INSERT INTO calendar_accounts (account_id, provider, account_name, email) VALUES ($1,$2,$3,$4) RETURNING account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at",
        )
        .bind(&account_id)
        .bind(provider)
        .bind(account_name)
        .bind(email)
        .fetch_one(&mut *transaction)
        .await?;
        let account = row_to_calendar_account(row).map_err(CalendarError::from)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                observation_id,
                "calendar",
                "calendar_account",
                account.account_id.clone(),
                relationship_kind,
                json!({
                    "account_id": account.account_id,
                    "provider": account.provider,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(account)
    }

    pub async fn get(&self, account_id: &str) -> Result<Option<CalendarAccount>, CalendarError> {
        let row = sqlx::query("SELECT account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at FROM calendar_accounts WHERE account_id=$1")
            .bind(account_id).fetch_optional(&self.pool).await?;
        row.map(row_to_calendar_account)
            .transpose()
            .map_err(CalendarError::from)
    }

    pub async fn list(
        &self,
        provider: Option<&str>,
    ) -> Result<Vec<CalendarAccount>, CalendarError> {
        let rows = if let Some(provider) = provider {
            sqlx::query("SELECT account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at FROM calendar_accounts WHERE provider=$1 ORDER BY account_name")
                .bind(provider).fetch_all(&self.pool).await?
        } else {
            sqlx::query("SELECT account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at FROM calendar_accounts ORDER BY account_name")
                .fetch_all(&self.pool).await?
        };
        rows.into_iter()
            .map(row_to_calendar_account)
            .collect::<Result<Vec<_>, _>>()
            .map_err(CalendarError::from)
    }

    pub async fn update(
        &self,
        account_id: &str,
        update: &CalendarAccountUpdate,
    ) -> Result<CalendarAccount, CalendarError> {
        self.update_with_observation(account_id, update, None, "update", None)
            .await
    }

    pub async fn update_with_observation(
        &self,
        account_id: &str,
        update: &CalendarAccountUpdate,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CalendarAccount, CalendarError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "UPDATE calendar_accounts SET account_name=COALESCE($2,account_name), email=COALESCE($3,email), sync_status=COALESCE($4,sync_status), updated_at=now() WHERE account_id=$1 RETURNING account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at",
        )
        .bind(account_id)
        .bind(update.account_name.as_deref())
        .bind(update.email.as_deref())
        .bind(update.sync_status.as_deref())
        .fetch_one(&mut *transaction)
        .await?;
        let account = row_to_calendar_account(row).map_err(CalendarError::from)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                observation_id,
                "calendar",
                "calendar_account",
                account.account_id.clone(),
                relationship_kind,
                json!({
                    "account_id": account.account_id,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(account)
    }

    pub async fn upsert_google_workspace_account(
        &self,
        mail_account_id: &str,
        account_name: &str,
        email: Option<&str>,
        credentials_reference: &str,
    ) -> Result<CalendarAccount, CalendarError> {
        self.upsert_linked_provider_account(
            &format!("google-calendar:{mail_account_id}"),
            "google",
            mail_account_id,
            account_name,
            email,
            credentials_reference,
            "gmail",
            "google_calendar_api",
            ObservationOriginKind::LocalRuntime,
            "mail_account_setup.upsert_google_workspace_calendar_account",
        )
        .await
    }

    pub async fn upsert_apple_icloud_account(
        &self,
        mail_account_id: &str,
        account_name: &str,
        email: Option<&str>,
        credentials_reference: &str,
    ) -> Result<CalendarAccount, CalendarError> {
        self.upsert_linked_provider_account(
            &format!("icloud-calendar:{mail_account_id}"),
            "apple",
            mail_account_id,
            account_name,
            email,
            credentials_reference,
            "icloud",
            "apple_caldav",
            ObservationOriginKind::LocalRuntime,
            "mail_account_setup.upsert_apple_icloud_calendar_account",
        )
        .await
    }

    pub async fn restore_google_workspace_account(
        &self,
        mail_account_id: &str,
        account_name: &str,
        email: Option<&str>,
        credentials_reference: &str,
    ) -> Result<CalendarAccount, CalendarError> {
        self.upsert_linked_provider_account(
            &format!("google-calendar:{mail_account_id}"),
            "google",
            mail_account_id,
            account_name,
            email,
            credentials_reference,
            "gmail",
            "google_calendar_api",
            ObservationOriginKind::VaultSource,
            "vault_reconciliation.restore_linked_calendar_account",
        )
        .await
    }

    pub async fn restore_apple_icloud_account(
        &self,
        mail_account_id: &str,
        account_name: &str,
        email: Option<&str>,
        credentials_reference: &str,
    ) -> Result<CalendarAccount, CalendarError> {
        self.upsert_linked_provider_account(
            &format!("icloud-calendar:{mail_account_id}"),
            "apple",
            mail_account_id,
            account_name,
            email,
            credentials_reference,
            "icloud",
            "apple_caldav",
            ObservationOriginKind::VaultSource,
            "vault_reconciliation.restore_linked_calendar_account",
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn upsert_linked_provider_account(
        &self,
        account_id: &str,
        provider: &str,
        mail_account_id: &str,
        account_name: &str,
        email: Option<&str>,
        credentials_reference: &str,
        source_provider: &str,
        sync_mode: &str,
        origin_kind: ObservationOriginKind,
        actor: &str,
    ) -> Result<CalendarAccount, CalendarError> {
        let mut transaction = self.pool.begin().await?;
        let capabilities = json!({
            "mail_account_id": mail_account_id,
            "source_provider": source_provider,
            "connected_services": ["calendar"],
            "sync_mode": sync_mode
        });
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "CALENDAR_ACCOUNT_LINK",
                origin_kind,
                chrono::Utc::now(),
                json!({
                    "account_id": account_id,
                    "provider": provider,
                    "mail_account_id": mail_account_id,
                    "account_name": account_name,
                    "email": email,
                    "credentials_reference": credentials_reference,
                    "source_provider": source_provider,
                    "sync_mode": sync_mode,
                    "action": "upsert_linked_calendar_account",
                }),
                format!("calendar-account://{account_id}/linked-provider"),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": "upsert_linked_calendar_account",
                "source_provider": source_provider,
                "mail_account_id": mail_account_id,
            })),
        )
        .await?;
        let row = sqlx::query(
            r#"
            INSERT INTO calendar_accounts (
                account_id,
                provider,
                account_name,
                email,
                credentials_reference,
                capabilities,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                provider = EXCLUDED.provider,
                account_name = EXCLUDED.account_name,
                email = EXCLUDED.email,
                credentials_reference = EXCLUDED.credentials_reference,
                capabilities = EXCLUDED.capabilities,
                updated_at = now()
            RETURNING account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at
            "#,
        )
        .bind(account_id)
        .bind(provider)
        .bind(account_name)
        .bind(email)
        .bind(credentials_reference)
        .bind(&capabilities)
        .fetch_one(&mut *transaction)
        .await?;
        link_vault_owned_entity_in_transaction(
            &mut transaction,
            &observation.observation_id,
            "calendar",
            "calendar_account",
            account_id.to_owned(),
            "linked_provider_upsert",
            json!({
                "account_id": account_id,
                "provider": provider,
                "mail_account_id": mail_account_id,
                "source_provider": source_provider,
                "sync_mode": sync_mode,
            }),
            None,
        )
        .await?;
        transaction.commit().await?;
        row_to_calendar_account(row).map_err(CalendarError::from)
    }

    pub async fn delete(&self, account_id: &str) -> Result<(), CalendarError> {
        self.delete_with_observation(account_id, None, "delete", None)
            .await
    }

    pub async fn delete_with_observation(
        &self,
        account_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), CalendarError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query("DELETE FROM calendar_accounts WHERE account_id=$1")
            .bind(account_id)
            .execute(&mut *transaction)
            .await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                observation_id,
                "calendar",
                "calendar_account",
                account_id.to_owned(),
                relationship_kind,
                json!({
                    "account_id": account_id,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}
fn row_to_calendar_account(row: PgRow) -> Result<CalendarAccount, sqlx::Error> {
    Ok(CalendarAccount {
        account_id: row.try_get("account_id")?,
        provider: row.try_get("provider")?,
        account_name: row.try_get("account_name")?,
        email: row.try_get("email")?,
        credentials_reference: row.try_get("credentials_reference")?,
        sync_status: row.try_get("sync_status")?,
        capabilities: row.try_get("capabilities")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
async fn link_vault_owned_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
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
