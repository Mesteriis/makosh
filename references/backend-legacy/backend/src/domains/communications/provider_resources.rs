use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use makosh_communications_api::mail_resources::{
    DiscoveredMailProviderResource, MailProviderResourceCommandPort, MailProviderResourceKind,
    MailProviderResourcePortError, MailProviderSemanticRole,
};

const RESOURCE_COLUMNS: &str = r#"
    mapping_id,
    account_id,
    resource_kind,
    provider_resource_id,
    display_name,
    semantic_role,
    local_folder_id,
    selectable,
    writable,
    mapping_source,
    capabilities,
    observed_at,
    created_at,
    updated_at
"#;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MailProviderResource {
    pub mapping_id: String,
    pub account_id: String,
    pub resource_kind: MailProviderResourceKind,
    pub provider_resource_id: String,
    pub display_name: String,
    pub semantic_role: Option<MailProviderSemanticRole>,
    pub local_folder_id: Option<String>,
    pub selectable: bool,
    pub writable: bool,
    pub mapping_source: String,
    pub capabilities: Value,
    pub observed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct NewMailProviderResource {
    pub account_id: String,
    pub resource_kind: MailProviderResourceKind,
    pub provider_resource_id: String,
    pub display_name: String,
    pub semantic_role: Option<MailProviderSemanticRole>,
    pub local_folder_id: Option<String>,
    pub selectable: bool,
    pub writable: bool,
    pub capabilities: Value,
    pub observed_at: DateTime<Utc>,
}

impl NewMailProviderResource {
    pub fn new(
        account_id: impl Into<String>,
        resource_kind: MailProviderResourceKind,
        provider_resource_id: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            resource_kind,
            provider_resource_id: provider_resource_id.into(),
            display_name: display_name.into(),
            semantic_role: None,
            local_folder_id: None,
            selectable: true,
            writable: true,
            capabilities: json!({}),
            observed_at: Utc::now(),
        }
    }

    pub fn semantic_role(mut self, semantic_role: MailProviderSemanticRole) -> Self {
        self.semantic_role = Some(semantic_role);
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    pub fn capabilities(mut self, capabilities: Value) -> Self {
        self.capabilities = capabilities;
        self
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MailProviderResourceMappingUpdate {
    pub semantic_role: Option<MailProviderSemanticRole>,
    pub local_folder_id: Option<String>,
}

#[derive(Debug, Error)]
pub enum MailProviderResourceError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("capabilities must be a JSON object")]
    InvalidCapabilities,

    #[error("provider account was not found: {0}")]
    AccountNotFound(String),

    #[error("local folder does not belong to provider account: {0}")]
    LocalFolderAccountMismatch(String),

    #[error(transparent)]
    Parse(#[from] makosh_communications_api::mail_resources::MailProviderResourceParseError),
}

#[derive(Clone)]
pub struct MailProviderResourceStore {
    pool: PgPool,
}

impl MailProviderResourceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_discovered(
        &self,
        resource: &NewMailProviderResource,
    ) -> Result<MailProviderResource, MailProviderResourceError> {
        validate_resource(resource)?;
        let account_id = resource.account_id.trim();
        let provider_resource_id = resource.provider_resource_id.trim();
        let display_name = resource.display_name.trim();
        let mapping_id =
            resource_mapping_id(account_id, resource.resource_kind, provider_resource_id);
        let mut transaction = self.pool.begin().await?;
        lock_account(&mut transaction, account_id).await?;

        let existing = sqlx::query(
            r#"
            SELECT mapping_id, mapping_source, semantic_role
            FROM communication_mail_provider_resources
            WHERE account_id = $1
              AND resource_kind = $2
              AND provider_resource_id = $3
            "#,
        )
        .bind(account_id)
        .bind(resource.resource_kind.as_str())
        .bind(provider_resource_id)
        .fetch_optional(&mut *transaction)
        .await?;
        let existing_mapping_id = existing
            .as_ref()
            .map(|row| row.try_get::<String, _>("mapping_id"))
            .transpose()?;
        let existing_is_manual = existing
            .as_ref()
            .map(|row| row.try_get::<String, _>("mapping_source"))
            .transpose()?
            .is_some_and(|source| source == "manual");
        let mut semantic_role: Option<MailProviderSemanticRole> = if existing_is_manual {
            existing
                .as_ref()
                .and_then(|row| row.try_get::<Option<String>, _>("semantic_role").ok())
                .flatten()
                .as_deref()
                .map(MailProviderSemanticRole::try_from)
                .transpose()?
        } else {
            resource.semantic_role
        };

        if let Some(role) = semantic_role {
            let manual_conflict = sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM communication_mail_provider_resources
                    WHERE account_id = $1
                      AND resource_kind = $2
                      AND semantic_role = $3
                      AND mapping_source = 'manual'
                      AND ($4::text IS NULL OR mapping_id <> $4)
                )
                "#,
            )
            .bind(account_id)
            .bind(resource.resource_kind.as_str())
            .bind(role.as_str())
            .bind(existing_mapping_id.as_deref())
            .fetch_one(&mut *transaction)
            .await?;
            if manual_conflict {
                semantic_role = None;
            } else {
                clear_discovered_role(
                    &mut transaction,
                    account_id,
                    resource.resource_kind,
                    role,
                    existing_mapping_id.as_deref(),
                )
                .await?;
            }
        }

        let sql = format!(
            r#"
            INSERT INTO communication_mail_provider_resources (
                mapping_id,
                account_id,
                resource_kind,
                provider_resource_id,
                display_name,
                semantic_role,
                local_folder_id,
                selectable,
                writable,
                mapping_source,
                capabilities,
                observed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'discovered', $10, $11)
            ON CONFLICT (account_id, resource_kind, provider_resource_id)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                semantic_role = CASE
                    WHEN communication_mail_provider_resources.mapping_source = 'manual'
                    THEN communication_mail_provider_resources.semantic_role
                    ELSE EXCLUDED.semantic_role
                END,
                local_folder_id = CASE
                    WHEN communication_mail_provider_resources.mapping_source = 'manual'
                    THEN communication_mail_provider_resources.local_folder_id
                    ELSE EXCLUDED.local_folder_id
                END,
                selectable = EXCLUDED.selectable,
                writable = EXCLUDED.writable,
                capabilities = EXCLUDED.capabilities,
                observed_at = EXCLUDED.observed_at,
                updated_at = now()
            RETURNING {RESOURCE_COLUMNS}
            "#,
        );
        let row = sqlx::query(&sql)
            .bind(&mapping_id)
            .bind(account_id)
            .bind(resource.resource_kind.as_str())
            .bind(provider_resource_id)
            .bind(display_name)
            .bind(semantic_role.map(MailProviderSemanticRole::as_str))
            .bind(resource.local_folder_id.as_deref())
            .bind(resource.selectable)
            .bind(resource.writable)
            .bind(&resource.capabilities)
            .bind(resource.observed_at)
            .fetch_one(&mut *transaction)
            .await?;
        if resource.resource_kind == MailProviderResourceKind::Folder
            && semantic_role == Some(MailProviderSemanticRole::Sent)
        {
            reconcile_imap_sent_delivery_states(&mut transaction, account_id).await?;
        }
        transaction.commit().await?;
        row_to_resource(row)
    }

    pub async fn set_manual_mapping(
        &self,
        mapping_id: &str,
        update: &MailProviderResourceMappingUpdate,
    ) -> Result<Option<MailProviderResource>, MailProviderResourceError> {
        validate_non_empty("mapping_id", mapping_id)?;
        let account_id = sqlx::query_scalar::<_, String>(
            "SELECT account_id FROM communication_mail_provider_resources WHERE mapping_id = $1",
        )
        .bind(mapping_id.trim())
        .fetch_optional(&self.pool)
        .await?;
        let Some(account_id) = account_id else {
            return Ok(None);
        };

        let mut transaction = self.pool.begin().await?;
        lock_account(&mut transaction, &account_id).await?;
        let current = sqlx::query(
            "SELECT resource_kind FROM communication_mail_provider_resources WHERE mapping_id = $1 FOR UPDATE",
        )
        .bind(mapping_id.trim())
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(current) = current else {
            transaction.commit().await?;
            return Ok(None);
        };
        let resource_kind = MailProviderResourceKind::try_from(
            current.try_get::<String, _>("resource_kind")?.as_str(),
        )?;
        validate_local_folder(
            &mut transaction,
            &account_id,
            update.local_folder_id.as_deref(),
        )
        .await?;
        let update_role: Option<MailProviderSemanticRole> = update
            .semantic_role
            .filter(|role| *role != MailProviderSemanticRole::User);
        if let Some(role) = update_role {
            sqlx::query(
                r#"
                UPDATE communication_mail_provider_resources
                SET semantic_role = NULL, updated_at = now()
                WHERE account_id = $1
                  AND resource_kind = $2
                  AND semantic_role = $3
                  AND mapping_id <> $4
                "#,
            )
            .bind(&account_id)
            .bind(resource_kind.as_str())
            .bind(role.as_str())
            .bind(mapping_id.trim())
            .execute(&mut *transaction)
            .await?;
        }
        let sql = format!(
            r#"
            UPDATE communication_mail_provider_resources
            SET semantic_role = $2,
                local_folder_id = $3,
                mapping_source = 'manual',
                updated_at = now()
            WHERE mapping_id = $1
            RETURNING {RESOURCE_COLUMNS}
            "#,
        );
        let row = sqlx::query(&sql)
            .bind(mapping_id.trim())
            .bind(update.semantic_role.map(MailProviderSemanticRole::as_str))
            .bind(update.local_folder_id.as_deref())
            .fetch_optional(&mut *transaction)
            .await?;
        let resource = row.map(row_to_resource).transpose()?;
        if let Some(resource) = resource.as_ref() {
            reconcile_provider_folder_memberships_for_mapping(&mut transaction, resource).await?;
        }
        if resource_kind == MailProviderResourceKind::Folder {
            reconcile_imap_sent_delivery_states(&mut transaction, &account_id).await?;
        }
        transaction.commit().await?;
        Ok(resource)
    }

    pub async fn resource(
        &self,
        mapping_id: &str,
    ) -> Result<Option<MailProviderResource>, MailProviderResourceError> {
        validate_non_empty("mapping_id", mapping_id)?;
        let sql = format!(
            "SELECT {RESOURCE_COLUMNS} FROM communication_mail_provider_resources WHERE mapping_id = $1"
        );
        let row = sqlx::query(&sql)
            .bind(mapping_id.trim())
            .fetch_optional(&self.pool)
            .await?;
        row.map(row_to_resource).transpose()
    }

    pub async fn resource_for_role(
        &self,
        account_id: &str,
        resource_kind: MailProviderResourceKind,
        semantic_role: MailProviderSemanticRole,
    ) -> Result<Option<MailProviderResource>, MailProviderResourceError> {
        validate_non_empty("account_id", account_id)?;
        let sql = format!(
            r#"
            SELECT {RESOURCE_COLUMNS}
            FROM communication_mail_provider_resources
            WHERE account_id = $1 AND resource_kind = $2 AND semantic_role = $3
            ORDER BY (mapping_source = 'manual') DESC, updated_at DESC, mapping_id ASC
            LIMIT 1
            "#,
        );
        let row = sqlx::query(&sql)
            .bind(account_id.trim())
            .bind(resource_kind.as_str())
            .bind(semantic_role.as_str())
            .fetch_optional(&self.pool)
            .await?;
        row.map(row_to_resource).transpose()
    }

    pub async fn resource_for_display_name(
        &self,
        account_id: &str,
        resource_kind: MailProviderResourceKind,
        display_name: &str,
    ) -> Result<Option<MailProviderResource>, MailProviderResourceError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("display_name", display_name)?;
        let sql = format!(
            r#"
            SELECT {RESOURCE_COLUMNS}
            FROM communication_mail_provider_resources
            WHERE account_id = $1
              AND resource_kind = $2
              AND lower(display_name) = lower($3)
            ORDER BY (mapping_source = 'manual') DESC, writable DESC, updated_at DESC, mapping_id ASC
            LIMIT 1
            "#,
        );
        let row = sqlx::query(&sql)
            .bind(account_id.trim())
            .bind(resource_kind.as_str())
            .bind(display_name.trim())
            .fetch_optional(&self.pool)
            .await?;
        row.map(row_to_resource).transpose()
    }

    pub async fn resource_for_local_folder(
        &self,
        account_id: &str,
        local_folder_id: &str,
    ) -> Result<Option<MailProviderResource>, MailProviderResourceError> {
        validate_non_empty("account_id", account_id)?;
        validate_non_empty("local_folder_id", local_folder_id)?;
        let sql = format!(
            r#"
            SELECT {RESOURCE_COLUMNS}
            FROM communication_mail_provider_resources
            WHERE account_id = $1
              AND local_folder_id = $2
            ORDER BY (mapping_source = 'manual') DESC, writable DESC, updated_at DESC, mapping_id ASC
            LIMIT 1
            "#,
        );
        let row = sqlx::query(&sql)
            .bind(account_id.trim())
            .bind(local_folder_id.trim())
            .fetch_optional(&self.pool)
            .await?;
        row.map(row_to_resource).transpose()
    }

    pub async fn list_for_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<MailProviderResource>, MailProviderResourceError> {
        validate_non_empty("account_id", account_id)?;
        let sql = format!(
            r#"
            SELECT {RESOURCE_COLUMNS}
            FROM communication_mail_provider_resources
            WHERE account_id = $1
            ORDER BY resource_kind ASC, lower(display_name) ASC, provider_resource_id ASC
            "#,
        );
        let rows = sqlx::query(&sql)
            .bind(account_id.trim())
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(row_to_resource).collect()
    }
}

impl MailProviderResourceCommandPort for MailProviderResourceStore {
    fn record_discovered_resources<'a>(
        &'a self,
        account_id: &'a str,
        resources: &'a [DiscoveredMailProviderResource],
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<(), MailProviderResourcePortError>> + Send + 'a,
        >,
    > {
        Box::pin(async move {
            for resource in resources {
                self.upsert_discovered(&NewMailProviderResource {
                    account_id: account_id.to_owned(),
                    resource_kind: resource.resource_kind,
                    provider_resource_id: resource.provider_resource_id.clone(),
                    display_name: resource.display_name.clone(),
                    semantic_role: resource.semantic_role,
                    local_folder_id: None,
                    selectable: resource.selectable,
                    writable: resource.writable,
                    capabilities: resource.capabilities.clone(),
                    observed_at: Utc::now(),
                })
                .await
                .map_err(MailProviderResourcePortError::new)?;
            }
            Ok(())
        })
    }
}

fn validate_resource(resource: &NewMailProviderResource) -> Result<(), MailProviderResourceError> {
    validate_non_empty("account_id", &resource.account_id)?;
    validate_non_empty("provider_resource_id", &resource.provider_resource_id)?;
    validate_non_empty("display_name", &resource.display_name)?;
    if !resource.capabilities.is_object() {
        return Err(MailProviderResourceError::InvalidCapabilities);
    }
    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), MailProviderResourceError> {
    if value.trim().is_empty() {
        return Err(MailProviderResourceError::EmptyField(field));
    }
    Ok(())
}

fn resource_mapping_id(
    account_id: &str,
    resource_kind: MailProviderResourceKind,
    provider_resource_id: &str,
) -> String {
    let digest = Sha256::digest(
        format!(
            "{account_id}\u{1f}{}\u{1f}{provider_resource_id}",
            resource_kind.as_str()
        )
        .as_bytes(),
    );
    format!("mail-provider-resource:v1:{digest:x}")
}

fn row_to_resource(row: PgRow) -> Result<MailProviderResource, MailProviderResourceError> {
    let resource_kind = row.try_get::<String, _>("resource_kind")?;
    let semantic_role = row.try_get::<Option<String>, _>("semantic_role")?;
    Ok(MailProviderResource {
        mapping_id: row.try_get("mapping_id")?,
        account_id: row.try_get("account_id")?,
        resource_kind: MailProviderResourceKind::try_from(resource_kind.as_str())?,
        provider_resource_id: row.try_get("provider_resource_id")?,
        display_name: row.try_get("display_name")?,
        semantic_role: semantic_role
            .as_deref()
            .map(MailProviderSemanticRole::try_from)
            .transpose()?,
        local_folder_id: row.try_get("local_folder_id")?,
        selectable: row.try_get("selectable")?,
        writable: row.try_get("writable")?,
        mapping_source: row.try_get("mapping_source")?,
        capabilities: row.try_get("capabilities")?,
        observed_at: row.try_get("observed_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
mod reconciliation;
use reconciliation::*;
