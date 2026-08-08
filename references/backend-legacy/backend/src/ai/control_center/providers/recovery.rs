use serde_json::{Value, json};

use makosh_observations_api::models::ObservationOriginKind;

use super::super::errors::AiControlCenterError;
use super::super::evidence::{
    capture_provider_account_observation_with_origin,
    capture_provider_secret_binding_observation_with_origin,
};
use super::super::models::{AiProviderAccount, AiProviderVaultRestore};
use super::super::rows::row_to_provider;
use super::super::store::AiControlCenterStore;
use super::super::validation::{
    object_value, reject_secret_like_json, string_array_value, validate_non_empty,
    validate_provider_kind,
};

impl AiControlCenterStore {
    pub(crate) async fn restore_provider_from_vault(
        &self,
        restore: &AiProviderVaultRestore,
    ) -> Result<AiProviderAccount, AiControlCenterError> {
        validate_non_empty("provider_id", &restore.provider_id)?;
        validate_provider_kind(&restore.provider_kind)?;
        validate_non_empty("provider_key", &restore.provider_key)?;
        validate_non_empty("display_name", &restore.display_name)?;
        validate_non_empty("secret_ref", &restore.secret_ref)?;
        validate_non_empty("secret_purpose", &restore.secret_purpose)?;
        validate_ai_provider_status(&restore.status)?;
        validate_ai_provider_consent_state(&restore.consent_state)?;
        let config = Value::Object(object_value(restore.config.clone(), "config")?);
        reject_secret_like_json(&config)?;
        let capabilities = string_array_value(restore.capabilities.clone(), "capabilities")?;

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO ai_provider_accounts (
                provider_id,
                provider_kind,
                provider_key,
                display_name,
                status,
                consent_state,
                consented_at,
                config,
                capabilities,
                updated_at
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                CASE WHEN $6 = 'granted' THEN now() ELSE NULL END,
                $7,
                $8,
                now()
            )
            ON CONFLICT (provider_id)
            DO UPDATE SET
                provider_kind = EXCLUDED.provider_kind,
                provider_key = EXCLUDED.provider_key,
                display_name = EXCLUDED.display_name,
                status = EXCLUDED.status,
                consent_state = EXCLUDED.consent_state,
                consented_at = EXCLUDED.consented_at,
                config = EXCLUDED.config,
                capabilities = EXCLUDED.capabilities,
                updated_at = now()
            RETURNING
                provider_id,
                provider_kind,
                provider_key,
                display_name,
                status,
                consent_state,
                consented_at,
                config,
                capabilities,
                created_at,
                updated_at
            "#,
        )
        .bind(restore.provider_id.trim())
        .bind(restore.provider_kind.trim())
        .bind(restore.provider_key.trim())
        .bind(restore.display_name.trim())
        .bind(restore.status.trim())
        .bind(restore.consent_state.trim())
        .bind(config)
        .bind(json!(capabilities))
        .fetch_one(&mut *transaction)
        .await?;

        let provider = row_to_provider(row)?;
        capture_provider_account_observation_with_origin(
            &mut transaction,
            &provider,
            "vault_restore",
            "vault_reconciliation.restore_ai_provider",
            ObservationOriginKind::VaultSource,
        )
        .await?;
        sqlx::query(
            r#"
            INSERT INTO ai_provider_secret_refs (provider_id, secret_purpose, secret_ref, updated_at)
            VALUES ($1, $2, $3, now())
            ON CONFLICT (provider_id, secret_purpose)
            DO UPDATE SET
                secret_ref = EXCLUDED.secret_ref,
                updated_at = now()
            "#,
        )
        .bind(provider.provider_id.trim())
        .bind(restore.secret_purpose.trim())
        .bind(restore.secret_ref.trim())
        .execute(&mut *transaction)
        .await?;
        capture_provider_secret_binding_observation_with_origin(
            &mut transaction,
            &provider.provider_id,
            restore.secret_purpose.trim(),
            restore.secret_ref.trim(),
            "vault_reconciliation.restore_ai_provider_secret_binding",
            ObservationOriginKind::VaultSource,
        )
        .await?;
        transaction.commit().await?;
        self.seed_models_for_provider(&provider, "vault_reconciliation.restore_ai_provider")
            .await?;
        Ok(provider)
    }
}

fn validate_ai_provider_status(status: &str) -> Result<(), AiControlCenterError> {
    match status.trim() {
        "ready" | "disabled" | "needs_setup" | "error" => Ok(()),
        other => Err(AiControlCenterError::InvalidRequest(format!(
            "unsupported AI provider status `{other}`"
        ))),
    }
}

fn validate_ai_provider_consent_state(state: &str) -> Result<(), AiControlCenterError> {
    match state.trim() {
        "not_required" | "required" | "granted" | "revoked" => Ok(()),
        other => Err(AiControlCenterError::InvalidRequest(format!(
            "unsupported AI provider consent_state `{other}`"
        ))),
    }
}
