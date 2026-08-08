use axum::Json;
use axum::extract::{Path, State};
use chrono::Utc;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use makosh_communications_api::commands::{
    CommunicationProviderCommand, NewCommunicationProviderCommand,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use url::Url;
use uuid::Uuid;

use crate::app::api_support::stores::{domain_stores::*, integration_stores::*};
use crate::app::error::types::ApiError;
use crate::app::signal_hub_support::{
    provider_account_or_not_found, sync_provider_account_signal_connection,
};
use crate::app::state::AppState;
use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;

use crate::platform::secrets::models::{NewSecretReference, SecretKind, SecretStoreKind};
use crate::vault::errors::HostVaultError;
use crate::vault::models::{SecretEntryContext, VaultMode};

#[derive(Debug, Deserialize)]
pub(crate) struct ZulipAccountSetupRequest {
    pub(crate) account_id: String,
    pub(crate) display_name: String,
    pub(crate) external_account_id: String,
    #[serde(alias = "realm_url")]
    pub(crate) base_url: String,
    pub(crate) api_key: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ZulipAccountSetupResponse {
    pub(crate) account_id: String,
    pub(crate) provider_kind: String,
    pub(crate) display_name: String,
    pub(crate) external_account_id: String,
    pub(crate) base_url: String,
    pub(crate) credential_binding: ZulipCredentialBinding,
}

#[derive(Debug, Serialize)]
pub(crate) struct ZulipCredentialBinding {
    pub(crate) secret_purpose: String,
    pub(crate) secret_ref: String,
    pub(crate) secret_kind: String,
    pub(crate) store_kind: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ZulipStreamUploadCommandRequest {
    pub(crate) command_id: Option<String>,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) stream: String,
    pub(crate) topic: String,
    pub(crate) content: String,
    pub(crate) attachment_id: Option<String>,
    pub(crate) blob_id: Option<String>,
    pub(crate) filename: Option<String>,
    pub(crate) actor_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ZulipDirectUploadCommandRequest {
    pub(crate) command_id: Option<String>,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) recipients: Vec<String>,
    pub(crate) content: String,
    pub(crate) attachment_id: Option<String>,
    pub(crate) blob_id: Option<String>,
    pub(crate) filename: Option<String>,
    pub(crate) actor_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ZulipUploadCommandRequest {
    pub(crate) command_id: Option<String>,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) attachment_id: Option<String>,
    pub(crate) blob_id: Option<String>,
    pub(crate) filename: Option<String>,
    pub(crate) actor_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ZulipCommandEnqueueResponse {
    pub(crate) command_id: String,
    pub(crate) account_id: String,
    pub(crate) channel_kind: String,
    pub(crate) command_kind: String,
    pub(crate) idempotency_key: String,
    pub(crate) status: String,
    pub(crate) reconciliation_status: String,
    pub(crate) provider_conversation_id: Option<String>,
    pub(crate) payload: Value,
}

pub(crate) async fn post_zulip_account(
    State(state): State<AppState>,
    Json(request): Json<ZulipAccountSetupRequest>,
) -> Result<Json<ZulipAccountSetupResponse>, ApiError> {
    require_unlocked_host_vault(&state)?;
    let account_id = required_trimmed("account_id", &request.account_id)?;
    let display_name = required_trimmed("display_name", &request.display_name)?;
    let external_account_id =
        required_trimmed("external_account_id", &request.external_account_id)?;
    let base_url = normalized_base_url(&request.base_url)?;
    let api_key = required_trimmed("api_key", &request.api_key)?;
    let secret_ref = zulip_api_key_secret_ref(account_id);
    let secret_purpose = ProviderAccountSecretPurpose::ZulipApiKey;
    let metadata =
        zulip_secret_metadata(account_id, external_account_id, &base_url, secret_purpose);

    zulip_secret_reference_store(&state)?
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret_ref,
                SecretKind::ApiToken,
                SecretStoreKind::HostVault,
                format!("Zulip API key for {account_id}"),
            )
            .metadata(metadata.clone()),
        )
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "Zulip secret reference upsert failed");
            ApiError::FailedPrecondition("Zulip secret reference could not be persisted".to_owned())
        })?;

    state.vault.store_secret(
        &secret_ref,
        api_key,
        SecretEntryContext {
            entry_kind: "provider_api_token",
            account_id,
            purpose: secret_purpose.as_str(),
            secret_kind: SecretKind::ApiToken.as_str(),
            label: "Zulip API key",
            metadata: &metadata,
        },
    )?;

    let account = NewProviderAccount::new(
        account_id,
        CommunicationProviderKind::ZulipBot,
        display_name,
        external_account_id,
    )
    .config(json!({
        "base_url": base_url,
        "runtime": "bot_api",
        "lifecycle_state": "linked",
        "credentials": {
            "api_key_bound": true,
            "secret_purpose": secret_purpose.as_str(),
            "secret_kind": SecretKind::ApiToken.as_str(),
            "store_kind": SecretStoreKind::HostVault.as_str(),
            "secret_material": "excluded",
            "updated_at": Utc::now(),
        },
    }));
    communication_provider_account_store(&state)?
        .upsert(&account)
        .await?;
    communication_provider_secret_binding_store(&state)?
        .bind(&NewProviderAccountSecretBinding::new(
            account_id,
            secret_purpose,
            &secret_ref,
        ))
        .await?;

    let account = provider_account_or_not_found(&state, account_id).await?;
    sync_provider_account_signal_connection(&state, &account, Some(&secret_ref)).await?;

    Ok(Json(ZulipAccountSetupResponse {
        account_id: account.account_id,
        provider_kind: account.provider_kind.as_str().to_owned(),
        display_name: account.display_name,
        external_account_id: account.external_account_id,
        base_url,
        credential_binding: ZulipCredentialBinding {
            secret_purpose: secret_purpose.as_str().to_owned(),
            secret_ref,
            secret_kind: SecretKind::ApiToken.as_str().to_owned(),
            store_kind: SecretStoreKind::HostVault.as_str().to_owned(),
        },
    }))
}

pub(crate) async fn post_zulip_stream_upload_command(
    Path(account_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<ZulipStreamUploadCommandRequest>,
) -> Result<Json<ZulipCommandEnqueueResponse>, ApiError> {
    let account_id = validate_zulip_account(&state, &account_id).await?;
    let stream = required_trimmed("stream", &request.stream)?;
    let topic = required_trimmed("topic", &request.topic)?;
    let content = required_trimmed("content", &request.content)?;
    let attachment =
        upload_attachment_ref(request.attachment_id.as_deref(), request.blob_id.as_deref())?;
    let command_id = command_id_or_new(request.command_id.as_deref());
    let idempotency_key =
        optional_trimmed(request.idempotency_key.as_deref()).unwrap_or_else(|| {
            stream_upload_idempotency_key(account_id, stream, topic, &attachment, content)
        });
    let actor_id = actor_id_or_default(request.actor_id.as_deref());

    let mut payload = json!({
        "stream": stream,
        "topic": topic,
        "content": content,
        "attachment_id": attachment.attachment_id,
        "blob_id": attachment.blob_id,
    });
    if let Some(filename) = optional_trimmed(request.filename.as_deref()) {
        payload["filename"] = json!(filename);
    }

    let command = NewCommunicationProviderCommand::new(
        command_id,
        account_id,
        "zulip",
        "send_stream_message_with_upload",
        idempotency_key,
        actor_id,
    )
    .provider_conversation_id(format!("{stream}/{topic}"))
    .target_ref(json!({
        "stream": stream,
        "topic": topic,
    }))
    .payload(payload);

    let command = enqueue_zulip_command(&state, &command).await?;
    Ok(Json(zulip_command_enqueue_response(command)))
}

pub(crate) async fn post_zulip_direct_upload_command(
    Path(account_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<ZulipDirectUploadCommandRequest>,
) -> Result<Json<ZulipCommandEnqueueResponse>, ApiError> {
    let account_id = validate_zulip_account(&state, &account_id).await?;
    let recipients = validate_recipients(&request.recipients)?;
    let content = required_trimmed("content", &request.content)?;
    let attachment =
        upload_attachment_ref(request.attachment_id.as_deref(), request.blob_id.as_deref())?;
    let command_id = command_id_or_new(request.command_id.as_deref());
    let idempotency_key =
        optional_trimmed(request.idempotency_key.as_deref()).unwrap_or_else(|| {
            direct_upload_idempotency_key(account_id, &recipients, &attachment, content)
        });
    let actor_id = actor_id_or_default(request.actor_id.as_deref());
    let mut payload = json!({
        "recipients": recipients,
        "content": content,
        "attachment_id": attachment.attachment_id,
        "blob_id": attachment.blob_id,
    });
    if let Some(filename) = optional_trimmed(request.filename.as_deref()) {
        payload["filename"] = json!(filename);
    }

    let command = NewCommunicationProviderCommand::new(
        command_id,
        account_id,
        "zulip",
        "send_direct_message_with_upload",
        idempotency_key,
        actor_id,
    )
    .target_ref(json!({
        "recipients": payload["recipients"].clone(),
    }))
    .payload(payload);

    let command = enqueue_zulip_command(&state, &command).await?;
    Ok(Json(zulip_command_enqueue_response(command)))
}

pub(crate) async fn post_zulip_upload_command(
    Path(account_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<ZulipUploadCommandRequest>,
) -> Result<Json<ZulipCommandEnqueueResponse>, ApiError> {
    let account_id = validate_zulip_account(&state, &account_id).await?;
    let attachment =
        upload_attachment_ref(request.attachment_id.as_deref(), request.blob_id.as_deref())?;
    let command_id = command_id_or_new(request.command_id.as_deref());
    let idempotency_key = optional_trimmed(request.idempotency_key.as_deref())
        .unwrap_or_else(|| upload_idempotency_key(account_id, &attachment));
    let actor_id = actor_id_or_default(request.actor_id.as_deref());
    let mut payload = json!({
        "attachment_id": attachment.attachment_id,
        "blob_id": attachment.blob_id,
    });
    if let Some(filename) = optional_trimmed(request.filename.as_deref()) {
        payload["filename"] = json!(filename);
    }

    let command = NewCommunicationProviderCommand::new(
        command_id,
        account_id,
        "zulip",
        "upload_file",
        idempotency_key,
        actor_id,
    )
    .target_ref(json!({
        "attachment_id": payload["attachment_id"].clone(),
        "blob_id": payload["blob_id"].clone(),
    }))
    .payload(payload);

    let command = enqueue_zulip_command(&state, &command).await?;
    Ok(Json(zulip_command_enqueue_response(command)))
}

fn require_unlocked_host_vault(state: &AppState) -> Result<(), ApiError> {
    match state.vault.status()?.state {
        VaultMode::Unlocked => Ok(()),
        VaultMode::Locked => Err(ApiError::HostVault(HostVaultError::Locked)),
        VaultMode::Uninitialized => Err(ApiError::HostVault(HostVaultError::Uninitialized)),
    }
}

fn required_trimmed<'a>(field: &'static str, value: &'a str) -> Result<&'a str, ApiError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ApiError::FailedPrecondition(format!("{field} is required")));
    }
    Ok(value)
}

fn optional_trimmed(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

async fn validate_zulip_account<'a>(
    state: &AppState,
    account_id: &'a str,
) -> Result<&'a str, ApiError> {
    let account_id = required_trimmed("account_id", account_id)?;
    let account = provider_account_or_not_found(state, account_id).await?;
    if account.provider_kind != CommunicationProviderKind::ZulipBot {
        return Err(ApiError::FailedPrecondition(
            "account is not a zulip_bot provider account".to_owned(),
        ));
    }
    Ok(account_id)
}

fn command_id_or_new(command_id: Option<&str>) -> String {
    optional_trimmed(command_id).unwrap_or_else(|| format!("zulip-command-{}", Uuid::now_v7()))
}

fn actor_id_or_default(actor_id: Option<&str>) -> String {
    optional_trimmed(actor_id).unwrap_or_else(|| "makosh-frontend".to_owned())
}

fn normalized_base_url(value: &str) -> Result<String, ApiError> {
    let value = required_trimmed("base_url", value)?;
    let parsed = Url::parse(value)
        .map_err(|_| ApiError::FailedPrecondition("base_url must be a valid URL".to_owned()))?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed.as_str().trim_end_matches('/').to_owned()),
        _ => Err(ApiError::FailedPrecondition(
            "base_url must use http or https".to_owned(),
        )),
    }
}

struct UploadAttachmentRef {
    attachment_id: Option<String>,
    blob_id: Option<String>,
}

fn upload_attachment_ref(
    attachment_id: Option<&str>,
    blob_id: Option<&str>,
) -> Result<UploadAttachmentRef, ApiError> {
    let attachment_id = optional_trimmed(attachment_id);
    let blob_id = optional_trimmed(blob_id);
    if attachment_id.is_none() && blob_id.is_none() {
        return Err(ApiError::FailedPrecondition(
            "attachment_id or blob_id is required".to_owned(),
        ));
    }
    Ok(UploadAttachmentRef {
        attachment_id,
        blob_id,
    })
}

fn validate_recipients(values: &[String]) -> Result<Vec<String>, ApiError> {
    let recipients = values
        .iter()
        .filter_map(|value| optional_trimmed(Some(value)))
        .collect::<Vec<_>>();
    if recipients.is_empty() {
        return Err(ApiError::FailedPrecondition(
            "recipients must not be empty".to_owned(),
        ));
    }
    Ok(recipients)
}

async fn enqueue_zulip_command(
    state: &AppState,
    command: &NewCommunicationProviderCommand,
) -> Result<CommunicationProviderCommand, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CommunicationProviderCommandStore::new(pool)
        .enqueue(command)
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))
}

fn stream_upload_idempotency_key(
    account_id: &str,
    stream: &str,
    topic: &str,
    attachment: &UploadAttachmentRef,
    content: &str,
) -> String {
    format!(
        "zulip:send-stream-upload:{account_id}:{stream}:{topic}:{}:{}:{}",
        attachment.attachment_id.as_deref().unwrap_or(""),
        attachment.blob_id.as_deref().unwrap_or(""),
        content
    )
}

fn direct_upload_idempotency_key(
    account_id: &str,
    recipients: &[String],
    attachment: &UploadAttachmentRef,
    content: &str,
) -> String {
    format!(
        "zulip:send-direct-upload:{account_id}:{}:{}:{}:{}",
        recipients.join(","),
        attachment.attachment_id.as_deref().unwrap_or(""),
        attachment.blob_id.as_deref().unwrap_or(""),
        content
    )
}

fn upload_idempotency_key(account_id: &str, attachment: &UploadAttachmentRef) -> String {
    format!(
        "zulip:upload:{account_id}:{}:{}",
        attachment.attachment_id.as_deref().unwrap_or(""),
        attachment.blob_id.as_deref().unwrap_or("")
    )
}

fn zulip_command_enqueue_response(
    command: CommunicationProviderCommand,
) -> ZulipCommandEnqueueResponse {
    ZulipCommandEnqueueResponse {
        command_id: command.command_id,
        account_id: command.account_id,
        channel_kind: command.channel_kind,
        command_kind: command.command_kind,
        idempotency_key: command.idempotency_key,
        status: command.status,
        reconciliation_status: command.reconciliation_status,
        provider_conversation_id: command.provider_conversation_id,
        payload: command.payload,
    }
}

fn zulip_api_key_secret_ref(account_id: &str) -> String {
    format!("secret:provider-account:{account_id}:zulip_api_key")
}

fn zulip_secret_metadata(
    account_id: &str,
    external_account_id: &str,
    base_url: &str,
    purpose: ProviderAccountSecretPurpose,
) -> Value {
    json!({
        "provider": CommunicationProviderKind::ZulipBot.as_str(),
        "account_id": account_id,
        "external_account_id": external_account_id,
        "base_url": base_url,
        "secret_purpose": purpose.as_str(),
        "credential_kind": "zulip_api_key",
        "secret_material": "excluded",
        "api_key": "excluded",
    })
}
