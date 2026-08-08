use makosh_communications_api::accounts::{
    CommunicationProviderKind, DeletedProviderAccount, NewProviderAccount,
    NewProviderAccountSecretBinding, ProviderAccount, ProviderAccountCommandPort,
    ProviderAccountLookupPort, ProviderAccountMutationOrigin, ProviderAccountPortError,
    ProviderAccountSecretBinding, ProviderAccountSecretPurpose, ProviderAccountUsage,
    ProviderSecretBindingCommandPort, ProviderSecretBindingLookupPort,
    ProviderSecretBindingPortError,
};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Transaction};

use crate::errors::CommunicationIngestionError;

use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;
use makosh_observations_postgres::store::ObservationStore;

#[path = "provider_store_support.rs"]
mod provider_store_support;
use provider_store_support::*;

#[derive(Clone)]
pub struct CommunicationProviderAccountStore {
    pool: PgPool,
}

impl CommunicationProviderAccountStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_runtime_account(
        &self,
        account_id: impl Into<String>,
        provider_kind: &str,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
        config: serde_json::Value,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        let provider_kind = CommunicationProviderKind::try_from(provider_kind)?;
        self.upsert(
            &NewProviderAccount::new(account_id, provider_kind, display_name, external_account_id)
                .config(config),
        )
        .await
    }

    pub async fn upsert(
        &self,
        account: &NewProviderAccount,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        self.upsert_with_origin(
            account,
            ProviderAccountMutationOrigin::LocalRuntime,
            "vault.communication_provider_accounts.upsert",
        )
        .await
    }

    pub async fn restore(
        &self,
        account: &NewProviderAccount,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        self.upsert_with_origin(
            account,
            ProviderAccountMutationOrigin::VaultSource,
            "vault_reconciliation.restore_provider_account",
        )
        .await
    }

    pub async fn upsert_with_origin(
        &self,
        account: &NewProviderAccount,
        origin: ProviderAccountMutationOrigin,
        actor: &str,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        validate_provider_account(account)?;
        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_PROVIDER_ACCOUNT",
                observation_origin_kind(origin),
                chrono::Utc::now(),
                json!({
                    "account_id": account.account_id.trim(),
                    "provider_kind": account.provider_kind.as_str(),
                    "display_name": account.display_name.trim(),
                    "external_account_id": account.external_account_id.trim(),
                    "config": account.config,
                    "action": "upsert_communication_provider_account",
                }),
                format!(
                    "communication-provider-account://{}",
                    account.account_id.trim()
                ),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": "upsert_communication_provider_account",
                "provider_kind": account.provider_kind.as_str(),
            })),
        )
        .await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_provider_accounts (
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                provider_kind = EXCLUDED.provider_kind,
                display_name = EXCLUDED.display_name,
                external_account_id = EXCLUDED.external_account_id,
                config = EXCLUDED.config,
                updated_at = now()
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account.account_id.trim())
        .bind(account.provider_kind.as_str())
        .bind(account.display_name.trim())
        .bind(account.external_account_id.trim())
        .bind(&account.config)
        .fetch_one(&mut *transaction)
        .await?;
        ensure_canonical_provider_account_in_transaction(
            &mut transaction,
            account.account_id.trim(),
        )
        .await?;
        link_vault_owned_entity_in_transaction(
            &mut transaction,
            VaultOwnedEntityLink {
                observation_id: observation.observation_id.clone(),
                domain: "vault",
                entity_kind: "communication_provider_account",
                entity_id: account.account_id.trim().to_owned(),
                relationship_kind: "upsert",
                base_metadata: json!({
                    "provider_kind": account.provider_kind.as_str(),
                    "external_account_id": account.external_account_id.trim(),
                }),
                extra_metadata: None,
            },
        )
        .await?;
        transaction.commit().await?;

        row_to_provider_account(row)
    }

    pub async fn get(
        &self,
        account_id: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            FROM communication_provider_accounts
            WHERE account_id = $1
            "#,
        )
        .bind(account_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_account).transpose()
    }

    pub async fn list(&self) -> Result<Vec<ProviderAccount>, CommunicationIngestionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            FROM communication_provider_accounts
            ORDER BY provider_kind ASC, display_name ASC, account_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_provider_account).collect()
    }

    pub async fn update_config(
        &self,
        account_id: &str,
        config: &serde_json::Value,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        self.update_config_with_origin(
            account_id,
            config,
            ProviderAccountMutationOrigin::LocalRuntime,
            "vault.communication_provider_accounts.update_config",
            "update_config",
        )
        .await
    }

    pub async fn update_display_name(
        &self,
        account_id: &str,
        display_name: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;
        validate_non_empty_field("display_name", display_name)?;
        let display_name = display_name.trim();

        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_PROVIDER_ACCOUNT_DISPLAY_NAME_MUTATION",
                ObservationOriginKind::LocalRuntime,
                chrono::Utc::now(),
                json!({
                    "account_id": account_id.trim(),
                    "display_name": display_name,
                    "action": "update_display_name",
                }),
                format!(
                    "communication-provider-account://{}/display-name",
                    account_id.trim()
                ),
            )
            .provenance(json!({
                "captured_by": "settings.provider_accounts.update_display_name",
                "action": "update_display_name",
            })),
        )
        .await?;

        let row = sqlx::query(
            r#"
            UPDATE communication_provider_accounts
            SET display_name = $2,
                updated_at = now()
            WHERE account_id = $1
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(display_name)
        .fetch_optional(&mut *transaction)
        .await?;

        if row.is_some() {
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                VaultOwnedEntityLink {
                    observation_id: observation.observation_id.clone(),
                    domain: "vault",
                    entity_kind: "communication_provider_account",
                    entity_id: account_id.trim().to_owned(),
                    relationship_kind: "display_name_update",
                    base_metadata: json!({
                        "account_id": account_id.trim(),
                    }),
                    extra_metadata: None,
                },
            )
            .await?;
            transaction.commit().await?;
        } else {
            transaction.rollback().await?;
        }

        row.map(row_to_provider_account).transpose()
    }

    pub async fn update_whatsapp_lifecycle_state(
        &self,
        account_id: &str,
        lifecycle_state: &str,
    ) -> Result<(), CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;
        validate_non_empty_field("lifecycle_state", lifecycle_state)?;
        sqlx::query(
            r#"
            UPDATE communication_provider_accounts
            SET config = jsonb_set(
                    COALESCE(config, '{}'::jsonb),
                    '{lifecycle_state}',
                    to_jsonb($2::text),
                    true
                ),
                updated_at = now()
            WHERE account_id = $1
              AND provider_kind IN ('whatsapp_web', 'whatsapp_business_cloud')
            "#,
        )
        .bind(account_id)
        .bind(lifecycle_state)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_logged_out(
        &self,
        account_id: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        let Some(current) = self.get(account_id).await? else {
            return Ok(None);
        };
        let mut config = current.config;
        let config_object = config
            .as_object_mut()
            .ok_or(CommunicationIngestionError::NonObjectJson("config"))?;
        config_object.insert("auth_state".to_owned(), json!("logged_out"));
        config_object.insert("logged_out_at".to_owned(), json!(chrono::Utc::now()));

        self.update_config_with_origin(
            account_id,
            &config,
            ProviderAccountMutationOrigin::LocalRuntime,
            "vault.communication_provider_accounts.mark_logged_out",
            "logout",
        )
        .await
    }

    pub async fn update_config_with_origin(
        &self,
        account_id: &str,
        config: &serde_json::Value,
        origin: ProviderAccountMutationOrigin,
        actor: &str,
        action: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;
        if !config.is_object() {
            return Err(CommunicationIngestionError::NonObjectJson("config"));
        }

        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_PROVIDER_ACCOUNT_CONFIG_MUTATION",
                observation_origin_kind(origin),
                chrono::Utc::now(),
                json!({
                    "account_id": account_id.trim(),
                    "config": config,
                    "action": action,
                }),
                format!(
                    "communication-provider-account://{}/config",
                    account_id.trim()
                ),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": action,
            })),
        )
        .await?;

        let row = sqlx::query(
            r#"
            UPDATE communication_provider_accounts
            SET config = $2,
                updated_at = now()
            WHERE account_id = $1
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(config)
        .fetch_optional(&mut *transaction)
        .await?;

        if row.is_some() {
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                VaultOwnedEntityLink {
                    observation_id: observation.observation_id.clone(),
                    domain: "vault",
                    entity_kind: "communication_provider_account",
                    entity_id: account_id.trim().to_owned(),
                    relationship_kind: "config_update",
                    base_metadata: json!({
                        "account_id": account_id.trim(),
                        "action": action,
                    }),
                    extra_metadata: None,
                },
            )
            .await?;
            transaction.commit().await?;
        } else {
            transaction.rollback().await?;
        }

        row.map(row_to_provider_account).transpose()
    }

    pub async fn usage(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccountUsage, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                (SELECT count(*) FROM communication_raw_records WHERE account_id = $1) AS raw_record_count,
                (SELECT count(*) FROM communication_messages WHERE account_id = $1) AS message_count,
                (SELECT count(*) FROM communication_ingestion_checkpoints WHERE account_id = $1) AS checkpoint_count
            "#,
        )
        .bind(account_id.trim())
        .fetch_one(&self.pool)
        .await?;

        Ok(ProviderAccountUsage {
            raw_record_count: row.try_get("raw_record_count")?,
            message_count: row.try_get("message_count")?,
            checkpoint_count: row.try_get("checkpoint_count")?,
        })
    }

    pub async fn delete_access_metadata(
        &self,
        account_id: &str,
    ) -> Result<DeletedProviderAccount, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;

        let mut transaction = self.pool.begin().await?;

        let binding_rows = sqlx::query(
            r#"
            DELETE FROM communication_provider_account_secret_refs
            WHERE account_id = $1
            RETURNING account_id, secret_purpose, secret_ref
            "#,
        )
        .bind(account_id.trim())
        .fetch_all(&mut *transaction)
        .await?;
        let mut removed_bindings = Vec::with_capacity(binding_rows.len());
        let unbound_secret_refs = binding_rows
            .into_iter()
            .map(|row| {
                let removed_account_id: String = row.try_get("account_id")?;
                let secret_purpose: String = row.try_get("secret_purpose")?;
                let secret_ref: String = row.try_get("secret_ref")?;
                removed_bindings.push((removed_account_id, secret_purpose, secret_ref.clone()));
                Ok(secret_ref)
            })
            .collect::<Result<Vec<String>, sqlx::Error>>()?;

        for (removed_account_id, secret_purpose, secret_ref) in &removed_bindings {
            let observation = ObservationStore::capture_in_transaction(
                &mut transaction,
                &NewObservation::new(
                    "COMMUNICATION_PROVIDER_SECRET_BINDING_REMOVED",
                    ObservationOriginKind::LocalRuntime,
                    chrono::Utc::now(),
                    json!({
                        "account_id": removed_account_id,
                        "secret_purpose": secret_purpose,
                        "secret_ref": secret_ref,
                        "action": "remove_provider_account_secret_binding",
                    }),
                    format!(
                        "communication-provider-account://{removed_account_id}/secret-binding/{secret_purpose}/delete"
                    ),
                )
                .provenance(json!({
                    "captured_by": "vault.communication_provider_accounts.delete_access_metadata",
                    "action": "remove_provider_account_secret_binding",
                })),
            )
            .await?;
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                VaultOwnedEntityLink {
                    observation_id: observation.observation_id.clone(),
                    domain: "vault",
                    entity_kind: "communication_provider_secret_binding",
                    entity_id: format!("{removed_account_id}:{secret_purpose}"),
                    relationship_kind: "remove",
                    base_metadata: json!({
                        "account_id": removed_account_id,
                        "secret_purpose": secret_purpose,
                        "secret_ref": secret_ref,
                    }),
                    extra_metadata: None,
                },
            )
            .await?;
        }

        sqlx::query(
            r#"
            DELETE FROM communication_ingestion_checkpoints
            WHERE account_id = $1
            "#,
        )
        .bind(account_id.trim())
        .execute(&mut *transaction)
        .await?;

        let existing_row = sqlx::query(
            r#"
            SELECT config
            FROM communication_provider_accounts
            WHERE account_id = $1
            "#,
        )
        .bind(account_id.trim())
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(existing_row) = existing_row else {
            transaction.commit().await?;
            return Ok(DeletedProviderAccount {
                account: None,
                unbound_secret_refs,
            });
        };
        let mut config: serde_json::Value = existing_row.try_get("config")?;
        let config_object = config
            .as_object_mut()
            .ok_or(CommunicationIngestionError::NonObjectJson("config"))?;
        config_object.insert("auth_state".to_owned(), json!("deleted"));
        config_object.insert("deleted_at".to_owned(), json!(chrono::Utc::now()));
        config_object.insert(
            "deleted_reason".to_owned(),
            json!("retained_evidence_credentials_purged"),
        );
        config_object.insert("sync_enabled".to_owned(), json!(false));

        let account_row = sqlx::query(
            r#"
            UPDATE communication_provider_accounts
            SET config = $2,
                updated_at = now()
            WHERE account_id = $1
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(&config)
        .fetch_optional(&mut *transaction)
        .await?;

        if let Some(account) = account_row.as_ref() {
            let deleted_account = row_ref_to_provider_account(account)?;
            let observation = ObservationStore::capture_in_transaction(
                &mut transaction,
                &NewObservation::new(
                    "COMMUNICATION_PROVIDER_ACCOUNT_DELETED",
                    ObservationOriginKind::LocalRuntime,
                    chrono::Utc::now(),
                    json!({
                        "account_id": deleted_account.account_id,
                        "provider_kind": deleted_account.provider_kind.as_str(),
                        "display_name": deleted_account.display_name,
                        "external_account_id": deleted_account.external_account_id,
                        "action": "delete_communication_provider_account_access",
                    }),
                    format!(
                        "communication-provider-account://{}/access/delete",
                        deleted_account.account_id
                    ),
                )
                .provenance(json!({
                    "captured_by": "vault.communication_provider_accounts.delete_access_metadata",
                    "action": "delete_communication_provider_account_access",
                    "provider_kind": deleted_account.provider_kind.as_str(),
                })),
            )
            .await?;
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                VaultOwnedEntityLink {
                    observation_id: observation.observation_id.clone(),
                    domain: "vault",
                    entity_kind: "communication_provider_account",
                    entity_id: deleted_account.account_id.clone(),
                    relationship_kind: "delete_access",
                    base_metadata: json!({
                        "provider_kind": deleted_account.provider_kind.as_str(),
                        "external_account_id": deleted_account.external_account_id,
                    }),
                    extra_metadata: None,
                },
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(DeletedProviderAccount {
            account: account_row.map(row_to_provider_account).transpose()?,
            unbound_secret_refs,
        })
    }

    pub async fn delete_metadata(
        &self,
        account_id: &str,
    ) -> Result<DeletedProviderAccount, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;

        let mut transaction = self.pool.begin().await?;

        let binding_rows = sqlx::query(
            r#"
            DELETE FROM communication_provider_account_secret_refs
            WHERE account_id = $1
            RETURNING account_id, secret_purpose, secret_ref
            "#,
        )
        .bind(account_id.trim())
        .fetch_all(&mut *transaction)
        .await?;
        let mut removed_bindings = Vec::with_capacity(binding_rows.len());
        let unbound_secret_refs = binding_rows
            .into_iter()
            .map(|row| {
                let removed_account_id: String = row.try_get("account_id")?;
                let secret_purpose: String = row.try_get("secret_purpose")?;
                let secret_ref: String = row.try_get("secret_ref")?;
                removed_bindings.push((removed_account_id, secret_purpose, secret_ref.clone()));
                Ok(secret_ref)
            })
            .collect::<Result<Vec<String>, sqlx::Error>>()?;

        for (removed_account_id, secret_purpose, secret_ref) in &removed_bindings {
            let observation = ObservationStore::capture_in_transaction(
                &mut transaction,
                &NewObservation::new(
                    "COMMUNICATION_PROVIDER_SECRET_BINDING_REMOVED",
                    ObservationOriginKind::LocalRuntime,
                    chrono::Utc::now(),
                    json!({
                        "account_id": removed_account_id,
                        "secret_purpose": secret_purpose,
                        "secret_ref": secret_ref,
                        "action": "remove_provider_account_secret_binding",
                    }),
                    format!(
                        "communication-provider-account://{removed_account_id}/secret-binding/{secret_purpose}/delete"
                    ),
                )
                .provenance(json!({
                    "captured_by": "vault.communication_provider_accounts.delete_metadata",
                    "action": "remove_provider_account_secret_binding",
                })),
            )
            .await?;
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                VaultOwnedEntityLink {
                    observation_id: observation.observation_id.clone(),
                    domain: "vault",
                    entity_kind: "communication_provider_secret_binding",
                    entity_id: format!("{removed_account_id}:{secret_purpose}"),
                    relationship_kind: "remove",
                    base_metadata: json!({
                        "account_id": removed_account_id,
                        "secret_purpose": secret_purpose,
                        "secret_ref": secret_ref,
                    }),
                    extra_metadata: None,
                },
            )
            .await?;
        }

        sqlx::query(
            r#"
            DELETE FROM communication_ingestion_checkpoints
            WHERE account_id = $1
            "#,
        )
        .bind(account_id.trim())
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            r#"
            DELETE FROM communication_address_book_sync_runs
            WHERE account_id = $1
            "#,
        )
        .bind(account_id.trim())
        .execute(&mut *transaction)
        .await?;

        let account_row = sqlx::query(
            r#"
            DELETE FROM communication_provider_accounts
            WHERE account_id = $1
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account_id.trim())
        .fetch_optional(&mut *transaction)
        .await?;

        if let Some(account) = account_row.as_ref() {
            let deleted_account = row_ref_to_provider_account(account)?;
            let observation = ObservationStore::capture_in_transaction(
                &mut transaction,
                &NewObservation::new(
                    "COMMUNICATION_PROVIDER_ACCOUNT_DELETED",
                    ObservationOriginKind::LocalRuntime,
                    chrono::Utc::now(),
                    json!({
                        "account_id": deleted_account.account_id,
                        "provider_kind": deleted_account.provider_kind.as_str(),
                        "display_name": deleted_account.display_name,
                        "external_account_id": deleted_account.external_account_id,
                        "config": deleted_account.config,
                        "action": "delete_communication_provider_account",
                    }),
                    format!(
                        "communication-provider-account://{}/delete",
                        deleted_account.account_id
                    ),
                )
                .provenance(json!({
                    "captured_by": "vault.communication_provider_accounts.delete_metadata",
                    "action": "delete_communication_provider_account",
                    "provider_kind": deleted_account.provider_kind.as_str(),
                })),
            )
            .await?;
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                VaultOwnedEntityLink {
                    observation_id: observation.observation_id.clone(),
                    domain: "vault",
                    entity_kind: "communication_provider_account",
                    entity_id: deleted_account.account_id.clone(),
                    relationship_kind: "delete",
                    base_metadata: json!({
                        "provider_kind": deleted_account.provider_kind.as_str(),
                        "external_account_id": deleted_account.external_account_id,
                    }),
                    extra_metadata: None,
                },
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(DeletedProviderAccount {
            account: account_row.map(row_to_provider_account).transpose()?,
            unbound_secret_refs,
        })
    }
}

#[derive(Clone)]
pub struct CommunicationProviderSecretBindingStore {
    pool: PgPool,
}

impl CommunicationProviderSecretBindingStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn bind(
        &self,
        binding: &NewProviderAccountSecretBinding,
    ) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
        self.bind_with_origin(
            binding,
            ObservationOriginKind::LocalRuntime,
            "vault.communication_provider_secret_bindings.bind",
        )
        .await
    }

    pub async fn restore(
        &self,
        binding: &NewProviderAccountSecretBinding,
    ) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
        self.bind_with_origin(
            binding,
            ObservationOriginKind::VaultSource,
            "vault_reconciliation.restore_provider_account_secret_binding",
        )
        .await
    }

    pub async fn bind_with_origin(
        &self,
        binding: &NewProviderAccountSecretBinding,
        origin: ObservationOriginKind,
        actor: &str,
    ) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
        validate_provider_secret_binding(binding)?;
        let binding_entity_id = format!(
            "{}:{}",
            binding.account_id.trim(),
            binding.secret_purpose.as_str()
        );
        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_PROVIDER_SECRET_BINDING",
                origin,
                chrono::Utc::now(),
                json!({
                    "account_id": binding.account_id.trim(),
                    "secret_purpose": binding.secret_purpose.as_str(),
                    "secret_ref": binding.secret_ref.trim(),
                    "action": "bind_provider_account_secret",
                }),
                format!(
                    "communication-provider-account://{}/secret-binding/{}",
                    binding.account_id.trim(),
                    binding.secret_purpose.as_str()
                ),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": "bind_provider_account_secret",
            })),
        )
        .await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_provider_account_secret_refs (
                account_id,
                secret_purpose,
                secret_ref,
                updated_at
            )
            VALUES ($1, $2, $3, now())
            ON CONFLICT (account_id, secret_purpose)
            DO UPDATE SET
                secret_ref = EXCLUDED.secret_ref,
                updated_at = now()
            RETURNING
                account_id,
                secret_purpose,
                secret_ref,
                created_at,
                updated_at
            "#,
        )
        .bind(binding.account_id.trim())
        .bind(binding.secret_purpose.as_str())
        .bind(binding.secret_ref.trim())
        .fetch_one(&mut *transaction)
        .await?;
        link_vault_owned_entity_in_transaction(
            &mut transaction,
            VaultOwnedEntityLink {
                observation_id: observation.observation_id.clone(),
                domain: "vault",
                entity_kind: "communication_provider_secret_binding",
                entity_id: binding_entity_id,
                relationship_kind: "bind",
                base_metadata: json!({
                    "account_id": binding.account_id.trim(),
                    "secret_purpose": binding.secret_purpose.as_str(),
                    "secret_ref": binding.secret_ref.trim(),
                }),
                extra_metadata: None,
            },
        )
        .await?;
        transaction.commit().await?;

        row_to_provider_secret_binding(row)
    }

    pub async fn list_for_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                account_id,
                secret_purpose,
                secret_ref,
                created_at,
                updated_at
            FROM communication_provider_account_secret_refs
            WHERE account_id = $1
            ORDER BY secret_purpose ASC
            "#,
        )
        .bind(account_id.trim())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_provider_secret_binding)
            .collect()
    }

    pub async fn get_for_account(
        &self,
        account_id: &str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Result<Option<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                account_id,
                secret_purpose,
                secret_ref,
                created_at,
                updated_at
            FROM communication_provider_account_secret_refs
            WHERE account_id = $1
              AND secret_purpose = $2
            "#,
        )
        .bind(account_id.trim())
        .bind(secret_purpose.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_secret_binding).transpose()
    }

    pub async fn account_has_host_vault_secret_refs(
        &self,
        account_id: &str,
    ) -> Result<bool, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;
        let count: i64 = sqlx::query_scalar(
            r#"SELECT count(*) FROM communication_provider_account_secret_refs refs
               JOIN secret_references secrets ON secrets.secret_ref = refs.secret_ref
               WHERE refs.account_id = $1 AND secrets.store_kind = 'host_vault'"#,
        )
        .bind(account_id.trim())
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    pub async fn is_secret_ref_bound(
        &self,
        secret_ref: &str,
    ) -> Result<bool, CommunicationIngestionError> {
        validate_non_empty_field("secret_ref", secret_ref)?;
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM communication_provider_account_secret_refs WHERE secret_ref = $1",
        )
        .bind(secret_ref.trim())
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    pub async fn unbind_for_account(
        &self,
        account_id: &str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Result<Option<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;
        let binding_entity_id = format!("{}:{}", account_id.trim(), secret_purpose.as_str());
        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_PROVIDER_SECRET_BINDING_REMOVED",
                ObservationOriginKind::LocalRuntime,
                chrono::Utc::now(),
                json!({
                    "account_id": account_id.trim(),
                    "secret_purpose": secret_purpose.as_str(),
                    "action": "remove_provider_account_secret_binding",
                }),
                format!(
                    "communication-provider-account://{}/secret-binding/{}/delete",
                    account_id.trim(),
                    secret_purpose.as_str()
                ),
            )
            .provenance(json!({
                "captured_by": "vault.communication_provider_secret_bindings.unbind",
                "action": "remove_provider_account_secret_binding",
            })),
        )
        .await?;
        let row = sqlx::query(
            r#"
            DELETE FROM communication_provider_account_secret_refs
            WHERE account_id = $1
              AND secret_purpose = $2
            RETURNING
                account_id,
                secret_purpose,
                secret_ref,
                created_at,
                updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(secret_purpose.as_str())
        .fetch_optional(&mut *transaction)
        .await?;

        if let Some(row) = row {
            let binding = row_to_provider_secret_binding(row)?;
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                VaultOwnedEntityLink {
                    observation_id: observation.observation_id.clone(),
                    domain: "vault",
                    entity_kind: "communication_provider_secret_binding",
                    entity_id: binding_entity_id,
                    relationship_kind: "remove",
                    base_metadata: json!({
                        "account_id": binding.account_id,
                        "secret_purpose": binding.secret_purpose.as_str(),
                        "secret_ref": binding.secret_ref,
                    }),
                    extra_metadata: None,
                },
            )
            .await?;
            transaction.commit().await?;
            return Ok(Some(binding));
        }

        transaction.rollback().await?;
        Ok(None)
    }
}

impl ProviderAccountLookupPort for CommunicationProviderAccountStore {
    fn get<'a>(
        &'a self,
        account_id: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Option<ProviderAccount>, ProviderAccountPortError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            CommunicationProviderAccountStore::get(self, account_id)
                .await
                .map_err(ProviderAccountPortError::new)
        })
    }

    fn list<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<ProviderAccount>, ProviderAccountPortError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            CommunicationProviderAccountStore::list(self)
                .await
                .map_err(ProviderAccountPortError::new)
        })
    }
}

impl ProviderAccountCommandPort for CommunicationProviderAccountStore {
    fn upsert<'a>(
        &'a self,
        account: &'a NewProviderAccount,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<ProviderAccount, ProviderAccountPortError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            CommunicationProviderAccountStore::upsert(self, account)
                .await
                .map_err(ProviderAccountPortError::new)
        })
    }

    fn upsert_runtime_account<'a>(
        &'a self,
        account_id: String,
        provider_kind: String,
        display_name: String,
        external_account_id: String,
        config: serde_json::Value,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<ProviderAccount, ProviderAccountPortError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            CommunicationProviderAccountStore::upsert_runtime_account(
                self,
                account_id,
                &provider_kind,
                display_name,
                external_account_id,
                config,
            )
            .await
            .map_err(ProviderAccountPortError::new)
        })
    }

    fn update_config<'a>(
        &'a self,
        account_id: &'a str,
        config: &'a serde_json::Value,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Option<ProviderAccount>, ProviderAccountPortError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            CommunicationProviderAccountStore::update_config(self, account_id, config)
                .await
                .map_err(ProviderAccountPortError::new)
        })
    }

    fn update_config_with_origin<'a>(
        &'a self,
        account_id: &'a str,
        config: &'a serde_json::Value,
        origin: ProviderAccountMutationOrigin,
        actor: &'a str,
        action: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Option<ProviderAccount>, ProviderAccountPortError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            CommunicationProviderAccountStore::update_config_with_origin(
                self, account_id, config, origin, actor, action,
            )
            .await
            .map_err(ProviderAccountPortError::new)
        })
    }

    fn mark_logged_out<'a>(
        &'a self,
        account_id: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Option<ProviderAccount>, ProviderAccountPortError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            CommunicationProviderAccountStore::mark_logged_out(self, account_id)
                .await
                .map_err(ProviderAccountPortError::new)
        })
    }

    fn delete_metadata<'a>(
        &'a self,
        account_id: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<DeletedProviderAccount, ProviderAccountPortError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            CommunicationProviderAccountStore::delete_metadata(self, account_id)
                .await
                .map_err(ProviderAccountPortError::new)
        })
    }
}

fn observation_origin_kind(origin: ProviderAccountMutationOrigin) -> ObservationOriginKind {
    match origin {
        ProviderAccountMutationOrigin::LocalRuntime => ObservationOriginKind::LocalRuntime,
        ProviderAccountMutationOrigin::VaultSource => ObservationOriginKind::VaultSource,
    }
}

impl ProviderSecretBindingLookupPort for CommunicationProviderSecretBindingStore {
    fn list_for_account<'a>(
        &'a self,
        account_id: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        Vec<ProviderAccountSecretBinding>,
                        ProviderSecretBindingPortError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            CommunicationProviderSecretBindingStore::list_for_account(self, account_id)
                .await
                .map_err(ProviderSecretBindingPortError::new)
        })
    }

    fn get_for_account<'a>(
        &'a self,
        account_id: &'a str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        Option<ProviderAccountSecretBinding>,
                        ProviderSecretBindingPortError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            CommunicationProviderSecretBindingStore::get_for_account(
                self,
                account_id,
                secret_purpose,
            )
            .await
            .map_err(ProviderSecretBindingPortError::new)
        })
    }
}

impl ProviderSecretBindingCommandPort for CommunicationProviderSecretBindingStore {
    fn bind<'a>(
        &'a self,
        binding: &'a NewProviderAccountSecretBinding,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<ProviderAccountSecretBinding, ProviderSecretBindingPortError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            CommunicationProviderSecretBindingStore::bind(self, binding)
                .await
                .map_err(ProviderSecretBindingPortError::new)
        })
    }

    fn unbind_for_account<'a>(
        &'a self,
        account_id: &'a str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        Option<ProviderAccountSecretBinding>,
                        ProviderSecretBindingPortError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            CommunicationProviderSecretBindingStore::unbind_for_account(
                self,
                account_id,
                secret_purpose,
            )
            .await
            .map_err(ProviderSecretBindingPortError::new)
        })
    }
}
