use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::ProviderAccountLookupPort;
use makosh_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountCommandPort, ProviderAccountSecretPurpose,
};
use makosh_communications_api::accounts::{ProviderAccount, ProviderSecretBindingCommandPort};
use makosh_communications_api::commands::{
    CommunicationProviderCommand, ProviderCommandMirrorPort,
};
mod account_scope;
mod account_state;
mod command_conversion;
pub(crate) mod command_execution;
pub(crate) mod contracts;
mod identifiers;
mod provider_runtime;
mod reconciliation;
mod retry;
pub(crate) mod retry_execution;
mod secret_cleanup;
mod status;
mod validation;
mod web_companion;
use account_scope::live_whatsapp_account_ids;
use command_conversion::{
    communication_provider_command, provider_message_id_from_state,
    provider_request_id_matches_observed_media, provider_request_id_matches_observed_receipt,
    row_to_whatsapp_provider_write_command,
};
use command_execution::clamp_limit;
use identifiers::{
    session_secret_metadata, validated_or_generated_command_id, whatsapp_session_secret_ref,
    whatsapp_text_preview_hash,
};
use status::{
    account_provider_shape, account_runtime_kind, authorized_session_runtime_kind,
    media_transfer_available, provider_command_blockers, provider_shape_restorable_secret_purpose,
    runtime_health_status, runtime_runtime_available, runtime_status_blockers,
    whatsapp_account_lifecycle_state,
};
use validation::validate_non_empty;
pub(crate) async fn recover_stale_live_executing_commands(
    pool: &PgPool,
    account_lookup: &dyn ProviderAccountLookupPort,
    now: DateTime<Utc>,
    account_id: Option<&str>,
) -> Result<Vec<WhatsAppProviderWriteCommand>, WhatsappWebError> {
    let eligible_accounts = live_whatsapp_account_ids(account_lookup, account_id).await?;
    retry_execution::recover_stale_executing_commands_scoped(pool, now, &eligible_accounts).await
}

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::models::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebMessage, WhatsappWebObservedCall,
    WhatsappWebObservedDialog, WhatsappWebObservedMedia, WhatsappWebObservedMessage,
    WhatsappWebObservedMessageDelete, WhatsappWebObservedMessageUpdate,
    WhatsappWebObservedParticipant, WhatsappWebObservedPresence, WhatsappWebObservedReaction,
    WhatsappWebObservedReceipt, WhatsappWebObservedRuntimeEvent, WhatsappWebObservedStatus,
    WhatsappWebObservedStatusDelete, WhatsappWebObservedStatusView, WhatsappWebSession,
};
use crate::integrations::whatsapp::client::store::WhatsappWebStore;
use crate::platform::communications::ProviderChannelMessageLookupPort;
use crate::platform::secrets::models::{NewSecretReference, SecretStoreKind};
use crate::vault::HostVault;
use crate::vault::models::SecretEntryContext;
use contracts::*;

pub const WHATSAPP_OUTBOX_WORKER_ID: &str = "whatsapp-outbox-worker";
const RETRY_BASE_DELAY_SECONDS: i64 = 30;
const RETRY_MAX_DELAY_SECONDS: i64 = 15 * 60;
const STALE_EXECUTION_LOCK_SECONDS: i64 = 120;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WhatsAppProviderWriteCommand {
    pub(crate) command_id: String,
    pub(crate) account_id: String,
    pub(crate) command_kind: String,
    pub(crate) idempotency_key: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: Option<String>,
    pub(crate) capability_state: String,
    pub(crate) action_class: String,
    pub(crate) confirmation_decision: String,
    pub(crate) status: String,
    pub(crate) retry_count: i32,
    pub(crate) max_retries: i32,
    pub(crate) last_error: Option<String>,
    pub(crate) payload: Value,
    pub(crate) target_ref: Value,
    pub(crate) result_payload: Value,
    pub(crate) audit_metadata: Value,
    pub(crate) provider_state: Value,
    pub(crate) reconciliation_status: String,
    pub(crate) next_attempt_at: Option<DateTime<Utc>>,
    pub(crate) last_attempt_at: Option<DateTime<Utc>>,
    pub(crate) provider_observed_at: Option<DateTime<Utc>>,
    pub(crate) reconciled_at: Option<DateTime<Utc>>,
    pub(crate) dead_lettered_at: Option<DateTime<Utc>>,
    pub(crate) completed_at: Option<DateTime<Utc>>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

struct ProviderCommandInsert<'a> {
    command_id: String,
    account_id: &'a str,
    command_kind: &'a str,
    idempotency_key: String,
    provider_chat_id: &'a str,
    provider_message_id: Option<&'a str>,
    action_class: &'a str,
    confirmation_decision: &'a str,
    payload: Value,
    target_ref: Value,
    rendered_preview_hash: Option<String>,
    restored_session_secret_ref: Option<String>,
}

pub fn whatsapp_web_companion_runtime(
    pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
    provider_command_mirror: Arc<dyn ProviderCommandMirrorPort>,
) -> Arc<dyn WhatsAppProviderRuntime> {
    web_companion::build_runtime(
        pool,
        provider_account_store,
        provider_secret_binding_store,
        provider_channel_message_store,
        provider_command_mirror,
    )
}

impl WhatsappWebStore {
    async fn whatsapp_account(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, WhatsappWebError> {
        let account_id = validate_non_empty("account_id", account_id)?;
        let account = self
            .provider_account_store()
            .get(&account_id)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp account `{account_id}` is not configured"
                ))
            })?;
        if !account.provider_kind.is_whatsapp() {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "account `{}` is not a WhatsApp provider account",
                account.account_id
            )));
        }
        Ok(account)
    }

    fn status_from_account(
        &self,
        account: &ProviderAccount,
        status: &str,
        restored_session_secret_ref: Option<String>,
        last_error: Option<String>,
    ) -> WhatsAppRuntimeStatus {
        let runtime_kind = account_runtime_kind(account);
        let provider_shape = account_provider_shape(account, self.provider_shape());
        let lifecycle_state = whatsapp_account_lifecycle_state(account);
        let forced_link_required = matches!(status, "link_required" | "created");
        let session_restore_available = restored_session_secret_ref.is_some()
            && !forced_link_required
            && !matches!(lifecycle_state, "revoked" | "removed");
        let session_secret_ref = if session_restore_available {
            restored_session_secret_ref
        } else {
            None
        };
        let effective_status = match status {
            "available" | "linked" | "revoked" | "removed" | "blocked" | "degraded" | "created"
            | "link_required" | "qr_pending" | "pair_code_pending" => status.to_owned(),
            _ if session_restore_available && runtime_kind == "fixture" && status == "running" => {
                "available".to_owned()
            }
            _ if lifecycle_state == "revoked" || lifecycle_state == "removed" => {
                lifecycle_state.to_owned()
            }
            _ if lifecycle_state == "qr_pending" || lifecycle_state == "pair_code_pending" => {
                lifecycle_state.to_owned()
            }
            _ if matches!(
                lifecycle_state,
                "linked" | "available" | "syncing" | "degraded" | "blocked"
            ) =>
            {
                lifecycle_state.to_owned()
            }
            _ if session_restore_available => "linked".to_owned(),
            _ => "link_required".to_owned(),
        };
        let live_runtime_available =
            runtime_runtime_available(&runtime_kind, provider_shape, &effective_status);
        let live_send_available =
            live_runtime_available && matches!(effective_status.as_str(), "available" | "degraded");
        let media_transfer_available =
            media_transfer_available(&runtime_kind, provider_shape, &effective_status);
        WhatsAppRuntimeStatus {
            account_id: account.account_id.clone(),
            provider_kind: account.provider_kind.as_str().to_owned(),
            provider_shape: provider_shape.as_str().to_owned(),
            runtime_kind: runtime_kind.clone(),
            status: effective_status.clone(),
            fixture_runtime: runtime_kind == "fixture",
            live_runtime_available,
            live_send_available,
            media_download_available: media_transfer_available,
            media_upload_available: media_transfer_available,
            session_restore_available,
            session_secret_ref,
            runtime_blockers: runtime_status_blockers(
                &effective_status,
                provider_shape,
                &runtime_kind,
                session_restore_available,
                last_error.as_deref(),
            ),
            last_error,
            updated_at: Utc::now(),
        }
    }

    fn blocked_command_response(
        &self,
        account: &ProviderAccount,
        command: &WhatsAppProviderWriteCommand,
        rendered_preview_hash: Option<String>,
    ) -> WhatsAppProviderCommandResponse {
        let runtime_kind = account_runtime_kind(account);
        let provider_shape = account_provider_shape(account, self.provider_shape());
        let session_restore_available = command
            .audit_metadata
            .get("session_restore_available")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        WhatsAppProviderCommandResponse {
            command_id: command.command_id.clone(),
            idempotency_key: command.idempotency_key.clone(),
            command_kind: command.command_kind.clone(),
            account_id: account.account_id.clone(),
            provider_kind: account.provider_kind.as_str().to_owned(),
            provider_shape: provider_shape.as_str().to_owned(),
            runtime_kind,
            provider_chat_id: command.provider_chat_id.clone(),
            provider_message_id: command.provider_message_id.clone(),
            status: "blocked".to_owned(),
            durable_status: command.status.clone(),
            delivery_state: "not_attempted".to_owned(),
            session_restore_available,
            rendered_preview_hash,
            runtime_blockers: command
                .result_payload
                .get("runtime_blockers")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
            last_error: command.last_error.clone(),
            updated_at: Utc::now(),
        }
    }

    async fn insert_blocked_provider_command(
        &self,
        input: ProviderCommandInsert<'_>,
    ) -> Result<WhatsAppProviderWriteCommand, WhatsappWebError> {
        let session_restore_available = input.restored_session_secret_ref.is_some();
        let account = self.whatsapp_account(input.account_id).await?;
        let runtime_kind = account_runtime_kind(&account);
        let provider_shape = account_provider_shape(&account, self.provider_shape());
        let blockers =
            provider_command_blockers(&runtime_kind, provider_shape, session_restore_available);
        let last_error = blockers.first().cloned();
        let result_payload = json!({
            "status": "blocked",
            "delivery_state": "not_attempted",
            "runtime_kind": runtime_kind,
            "runtime_blockers": blockers,
        });
        let audit_metadata = json!({
            "provider": "whatsapp",
            "provider_shape": provider_shape.as_str(),
            "runtime_kind": runtime_kind,
            "session_restore_available": session_restore_available,
            "rendered_preview_hash": input.rendered_preview_hash,
        });

        let mut transaction = self.pool().begin().await?;
        sqlx::query(
            r#"
            INSERT INTO whatsapp_provider_write_commands
                (command_id, account_id, command_kind, idempotency_key, provider_chat_id,
                 provider_message_id, capability_state, action_class, confirmation_decision,
                 status, retry_count, max_retries, last_error, actor_id, payload, target_ref,
                 result_payload, audit_metadata, reconciliation_status)
            VALUES ($1, $2, $3, $4, $5, $6, 'blocked', $7, $8, 'cancelled', 0, 3, $9,
                    'makosh-frontend', $10, $11, $12, $13, 'not_required')
            ON CONFLICT (account_id, idempotency_key) DO NOTHING
            "#,
        )
        .bind(&input.command_id)
        .bind(input.account_id)
        .bind(input.command_kind)
        .bind(&input.idempotency_key)
        .bind(input.provider_chat_id)
        .bind(input.provider_message_id)
        .bind(input.action_class)
        .bind(input.confirmation_decision)
        .bind(last_error.as_deref())
        .bind(&input.payload)
        .bind(&input.target_ref)
        .bind(&result_payload)
        .bind(&audit_metadata)
        .execute(&mut *transaction)
        .await?;

        let command_row = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND idempotency_key = $2
            "#,
        )
        .bind(input.account_id)
        .bind(&input.idempotency_key)
        .fetch_optional(&mut *transaction)
        .await?;
        let command = command_row
            .map(row_to_whatsapp_provider_write_command)
            .transpose()?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp provider command `{}` was not persisted",
                    input.idempotency_key
                ))
            })?;

        transaction.commit().await?;

        self.mirror_canonical_provider_command(&command).await?;

        self.provider_command_by_idempotency(input.account_id, &input.idempotency_key)
            .await?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp provider command `{}` was not persisted",
                    input.idempotency_key
                ))
            })
    }

    async fn provider_command_by_idempotency(
        &self,
        account_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<WhatsAppProviderWriteCommand>, WhatsappWebError> {
        let row = sqlx::query(
            r#"
            SELECT *
            FROM whatsapp_provider_write_commands
            WHERE account_id = $1
              AND idempotency_key = $2
            "#,
        )
        .bind(account_id)
        .bind(idempotency_key)
        .fetch_optional(self.pool())
        .await?;

        row.map(row_to_whatsapp_provider_write_command).transpose()
    }

    async fn mirror_canonical_provider_command(
        &self,
        command: &WhatsAppProviderWriteCommand,
    ) -> Result<(), WhatsappWebError> {
        self.provider_command_mirror()
            .mirror(&communication_provider_command(command))
            .await
            .map_err(|error| WhatsappWebError::InvalidRequest(error.to_string()))
    }
}
