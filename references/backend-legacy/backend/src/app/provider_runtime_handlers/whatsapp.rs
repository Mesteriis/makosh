use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use makosh_communications_api::attachments::CanonicalMediaReadPort;
use makosh_communications_api::calls::CanonicalCallReadPort;
use makosh_communications_api::conversations::ConversationReadPort;
use makosh_events_api::NewEventEnvelope;
use makosh_personas_api::PersonaIdentityProjectionPort;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::app::api_support::{
    ensure_fixture_routes_enabled,
    stores::{domain_stores::*, integration_stores::*},
};
use crate::app::error::types::ApiError;
use crate::app::signal_hub_support::{
    provider_account_or_not_found, sync_provider_account_signal_connection,
    sync_whatsapp_runtime_signal_connection,
};
use crate::app::state::AppState;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::models::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsappWebAccountSetupRequest, WhatsappWebAccountSetupResponse,
    WhatsappWebCallIngestResult, WhatsappWebDialogIngestResult, WhatsappWebMediaIngestResult,
    WhatsappWebMessageDeleteIngestResult, WhatsappWebMessageIngestResult,
    WhatsappWebMessageUpdateIngestResult, WhatsappWebParticipantIngestResult,
    WhatsappWebPresenceIngestResult, WhatsappWebReactionIngestResult,
    WhatsappWebReceiptIngestResult, WhatsappWebRuntimeEventIngestResult,
    WhatsappWebStatusDeleteIngestResult, WhatsappWebStatusIngestResult,
    WhatsappWebStatusViewIngestResult,
};
use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppAuthorizedSessionCredentialWrite, WhatsAppCommandDeadLetterRequest,
    WhatsAppCredentialBinding, WhatsAppProviderCommand, WhatsAppProviderCommandListResponse,
    WhatsAppProviderCommandResponse, WhatsAppRuntimeStatus,
};
use crate::platform::events::bus::{sanitize_event_payload, whatsapp_event_types};

pub(crate) mod accounts;
pub(crate) mod conversations;
mod event_builders;
#[path = "whatsapp/event_types.rs"]
mod event_types;
#[path = "whatsapp/input_policy.rs"]
mod input_policy;
use input_policy::*;
mod lifecycle_projection;
pub(crate) mod media;
pub(crate) mod messages;
pub(crate) mod statuses;
pub(crate) mod sync_calls;
pub(crate) mod sync_chats;
pub(crate) mod sync_history;
pub(crate) mod sync_presence;
pub(crate) mod sync_statuses;
#[path = "whatsapp_support.rs"]
mod whatsapp_support;
use whatsapp_support::*;

const AUDIT_ACTOR_ID: &str = "makosh-frontend";

#[derive(Deserialize)]
pub(crate) struct WhatsAppCommandListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider_message_id: Option<String>,
    pub(crate) command_kinds: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppChatSyncRequest {
    pub(crate) account_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppChatSyncItem {
    pub(crate) conversation_id: String,
    pub(crate) account_id: String,
    pub(crate) channel_kind: String,
    pub(crate) provider_chat_id: String,
    pub(crate) title: String,
    pub(crate) chat_kind: Option<String>,
    pub(crate) is_archived: bool,
    pub(crate) is_pinned: bool,
    pub(crate) is_muted: bool,
    pub(crate) is_unread: bool,
    pub(crate) unread_count: Option<i64>,
    pub(crate) participant_count: Option<i64>,
    pub(crate) community_parent_chat_id: Option<String>,
    pub(crate) community_parent_title: Option<String>,
    pub(crate) invite_link: Option<String>,
    pub(crate) is_community_root: bool,
    pub(crate) is_broadcast: bool,
    pub(crate) is_newsletter: bool,
    pub(crate) avatar_metadata: Value,
    pub(crate) provider_labels: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppChatSyncResponse {
    pub(crate) account_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) items: Vec<WhatsAppChatSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppHistorySyncRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppHistorySyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<crate::integrations::whatsapp::client::models::WhatsappWebMessage>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppMembersSyncRequest {
    pub(crate) account_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppMembersSyncItem {
    pub(crate) participant_id: String,
    pub(crate) conversation_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_member_id: String,
    pub(crate) provider_identity_id: Option<String>,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) role: String,
    pub(crate) status: Option<String>,
    pub(crate) identity_kind: Option<String>,
    pub(crate) address: Option<String>,
    pub(crate) is_admin: bool,
    pub(crate) is_owner: bool,
    pub(crate) participant_metadata: Value,
    pub(crate) identity_metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppMembersSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<WhatsAppMembersSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppStatusSyncRequest {
    pub(crate) account_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppStatusSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<crate::integrations::whatsapp::client::models::WhatsappWebMessage>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppPresenceSyncRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppPresenceSyncItem {
    pub(crate) identity_id: String,
    pub(crate) account_id: String,
    pub(crate) channel_kind: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider_identity_id: String,
    pub(crate) identity_kind: String,
    pub(crate) display_name: Option<String>,
    pub(crate) address: Option<String>,
    pub(crate) presence_state: String,
    pub(crate) last_seen_at: Option<String>,
    pub(crate) observed_at: Option<String>,
    pub(crate) identity_metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppPresenceSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<WhatsAppPresenceSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppCallsSyncRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppCallsSyncItem {
    pub(crate) call_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_call_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) direction: String,
    pub(crate) call_state: String,
    pub(crate) started_at: Option<String>,
    pub(crate) ended_at: Option<String>,
    pub(crate) observed_at: Option<String>,
    pub(crate) metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppCallsSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<WhatsAppCallsSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppContactsSyncRequest {
    pub(crate) account_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppContactsSyncItem {
    pub(crate) identity_id: String,
    pub(crate) account_id: String,
    pub(crate) channel_kind: String,
    pub(crate) provider_identity_id: String,
    pub(crate) identity_kind: String,
    pub(crate) display_name: Option<String>,
    pub(crate) address: Option<String>,
    pub(crate) push_name: Option<String>,
    pub(crate) business_profile: Value,
    pub(crate) profile_photo_ref: Value,
    pub(crate) display_name_history: Vec<String>,
    pub(crate) identity_metadata: Value,
    pub(crate) whatsapp_trace_metadata: Value,
    pub(crate) phone_trace_metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppContactsSyncResponse {
    pub(crate) account_id: String,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<WhatsAppContactsSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppMediaSyncRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) content_type: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppMediaSyncItem {
    pub(crate) attachment_id: String,
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) account_id: String,
    pub(crate) channel_kind: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider_message_id: String,
    pub(crate) provider_attachment_id: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: String,
    pub(crate) size_bytes: i64,
    pub(crate) sha256: String,
    pub(crate) scan_status: String,
    pub(crate) storage_kind: String,
    pub(crate) storage_path: String,
    pub(crate) message_subject: String,
    pub(crate) sender: String,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) occurred_at: Option<String>,
    pub(crate) created_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppMediaSyncResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) content_type: Option<String>,
    pub(crate) runtime_kind: String,
    pub(crate) status: String,
    pub(crate) synced_count: usize,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<WhatsAppMediaSyncItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppRuntimeBridgeMediaLifecycleRequest {
    pub(crate) account_id: String,
    pub(crate) command_id: String,
    pub(crate) media_direction: String,
    pub(crate) lifecycle_phase: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) provider_message_id: Option<String>,
    pub(crate) provider_media_id: Option<String>,
    pub(crate) attachment_id: Option<String>,
    pub(crate) blob_id: Option<String>,
    pub(crate) progress_percent: Option<u8>,
    pub(crate) content_type: Option<String>,
    pub(crate) filename: Option<String>,
    pub(crate) error_code: Option<String>,
    pub(crate) error_message: Option<String>,
    pub(crate) runtime_blockers: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppRuntimeBridgeSyncLifecycleRequest {
    pub(crate) account_id: String,
    pub(crate) scope: String,
    pub(crate) phase: String,
    pub(crate) subject_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) synced_count: Option<i64>,
    pub(crate) has_more: Option<bool>,
    pub(crate) error_code: Option<String>,
    pub(crate) error_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppRuntimeBridgeClaimCommandsRequest {
    pub(crate) account_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WhatsAppRuntimeBridgeCommandFailedRequest {
    pub(crate) error_message: String,
    pub(crate) error_code: Option<String>,
    pub(crate) retry_after_seconds: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppRuntimeBridgeExecutableCommand {
    pub(crate) command_id: String,
    pub(crate) account_id: String,
    pub(crate) command_kind: String,
    pub(crate) provider_kind: String,
    pub(crate) provider_shape: String,
    pub(crate) runtime_kind: String,
    pub(crate) lifecycle_state: Option<String>,
    pub(crate) session_restore_available: bool,
    pub(crate) runtime_blockers: Vec<String>,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: Option<String>,
    pub(crate) idempotency_key: String,
    pub(crate) capability_state: String,
    pub(crate) action_class: String,
    pub(crate) confirmation_decision: String,
    pub(crate) status: String,
    pub(crate) retry_count: i32,
    pub(crate) max_retries: i32,
    pub(crate) payload: Value,
    pub(crate) target_ref: Value,
    pub(crate) audit_metadata: Value,
    pub(crate) provider_state: Value,
    pub(crate) result_payload: Value,
    pub(crate) next_attempt_at: Option<chrono::DateTime<Utc>>,
    pub(crate) last_attempt_at: Option<chrono::DateTime<Utc>>,
    pub(crate) created_at: chrono::DateTime<Utc>,
    pub(crate) updated_at: chrono::DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WhatsAppRuntimeBridgeClaimCommandsResponse {
    pub(crate) items: Vec<WhatsAppRuntimeBridgeExecutableCommand>,
}

pub(crate) async fn post_whatsapp_sync_members(
    State(state): State<AppState>,
    Path(provider_chat_id): Path<String>,
    Json(request): Json<WhatsAppMembersSyncRequest>,
) -> Result<Json<WhatsAppMembersSyncResponse>, ApiError> {
    let account_id = required_string("account_id", &request.account_id)?;
    ensure_whatsapp_sync_supported(&state, &account_id, "sync_members").await?;
    let provider_chat_id = required_string("provider_chat_id", &provider_chat_id)?;
    let limit = request.limit.unwrap_or(50).clamp(1, 200);
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &provider_chat_id,
        "members",
        "started",
        json!({"scope": "members", "provider_chat_id": provider_chat_id}),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_STARTED,
        &account_id,
        &provider_chat_id,
        json!({"scope": "members", "provider_chat_id": provider_chat_id}),
    )
    .await?;
    let runtime_kind = current_whatsapp_runtime_kind(&state, &account_id).await?;
    let items =
        match list_whatsapp_sync_members(&state, &account_id, &provider_chat_id, limit).await {
            Ok(items) => items,
            Err(error) => {
                capture_whatsapp_sync_runtime_signal(
                    &state,
                    &account_id,
                    &provider_chat_id,
                    "members",
                    "failed",
                    json!({
                        "scope": "members",
                        "provider_chat_id": provider_chat_id,
                        "status": "failed",
                    }),
                )
                .await?;
                publish_whatsapp_sync_event(
                    &state,
                    whatsapp_event_types::SYNC_FAILED,
                    &account_id,
                    &provider_chat_id,
                    json!({
                        "scope": "members",
                        "provider_chat_id": provider_chat_id,
                        "status": "failed",
                    }),
                )
                .await?;
                return Err(error);
            }
        };
    let response = WhatsAppMembersSyncResponse {
        account_id: account_id.clone(),
        provider_chat_id: provider_chat_id.clone(),
        runtime_kind,
        status: "synced".to_owned(),
        synced_count: items.len(),
        has_more: items.len() as i64 >= limit,
        items,
    };
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &provider_chat_id,
        "members",
        "progress",
        json!({
            "scope": "members",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_PROGRESS,
        &account_id,
        &provider_chat_id,
        json!({
            "scope": "members",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &provider_chat_id,
        "members",
        "completed",
        json!({
            "scope": "members",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_COMPLETED,
        &account_id,
        &provider_chat_id,
        json!({
            "scope": "members",
            "provider_chat_id": provider_chat_id,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_sync_contacts(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppContactsSyncRequest>,
) -> Result<Json<WhatsAppContactsSyncResponse>, ApiError> {
    let account_id = required_string("account_id", &request.account_id)?;
    ensure_whatsapp_sync_supported(&state, &account_id, "sync_contacts").await?;
    let limit = request.limit.unwrap_or(50).clamp(1, 200);
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &account_id,
        "contacts",
        "started",
        json!({
            "scope": "contacts",
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_STARTED,
        &account_id,
        &account_id,
        json!({
            "scope": "contacts",
        }),
    )
    .await?;
    let runtime_kind = current_whatsapp_runtime_kind(&state, &account_id).await?;
    let items = match list_whatsapp_sync_contacts_via_ports(&state, &account_id, limit).await {
        Ok(items) => items,
        Err(error) => {
            capture_whatsapp_sync_runtime_signal(
                &state,
                &account_id,
                &account_id,
                "contacts",
                "failed",
                json!({
                    "scope": "contacts",
                    "status": "failed",
                }),
            )
            .await?;
            publish_whatsapp_sync_event(
                &state,
                whatsapp_event_types::SYNC_FAILED,
                &account_id,
                &account_id,
                json!({
                    "scope": "contacts",
                    "status": "failed",
                }),
            )
            .await?;
            return Err(error);
        }
    };
    let response = WhatsAppContactsSyncResponse {
        account_id: account_id.clone(),
        runtime_kind,
        status: "synced".to_owned(),
        synced_count: items.len(),
        has_more: items.len() as i64 >= limit,
        items,
    };
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &account_id,
        "contacts",
        "progress",
        json!({
            "scope": "contacts",
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_PROGRESS,
        &account_id,
        &account_id,
        json!({
            "scope": "contacts",
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &account_id,
        "contacts",
        "completed",
        json!({
            "scope": "contacts",
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_COMPLETED,
        &account_id,
        &account_id,
        json!({
            "scope": "contacts",
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_sync_media(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppMediaSyncRequest>,
) -> Result<Json<WhatsAppMediaSyncResponse>, ApiError> {
    let account_id = required_string("account_id", &request.account_id)?;
    ensure_whatsapp_sync_supported(&state, &account_id, "sync_media").await?;
    let limit = request.limit.unwrap_or(50).clamp(1, 200);
    let provider_chat_id = request
        .provider_chat_id
        .as_deref()
        .map(|value| required_string("provider_chat_id", value))
        .transpose()?;
    let content_type = request
        .content_type
        .as_deref()
        .map(|value| required_string("content_type", value))
        .transpose()?;
    let subject_id = provider_chat_id
        .clone()
        .unwrap_or_else(|| account_id.clone());
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "media",
        "started",
        json!({
            "scope": "media",
            "provider_chat_id": provider_chat_id,
            "content_type": content_type,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_STARTED,
        &account_id,
        &subject_id,
        json!({
            "scope": "media",
            "provider_chat_id": provider_chat_id,
            "content_type": content_type,
        }),
    )
    .await?;
    let runtime_kind = current_whatsapp_runtime_kind(&state, &account_id).await?;
    let items = match list_whatsapp_sync_media(
        &state,
        &account_id,
        provider_chat_id.as_deref(),
        content_type.as_deref(),
        limit,
    )
    .await
    {
        Ok(items) => items,
        Err(error) => {
            capture_whatsapp_sync_runtime_signal(
                &state,
                &account_id,
                &subject_id,
                "media",
                "failed",
                json!({
                    "scope": "media",
                    "provider_chat_id": provider_chat_id,
                    "content_type": content_type,
                    "status": "failed",
                }),
            )
            .await?;
            publish_whatsapp_sync_event(
                &state,
                whatsapp_event_types::SYNC_FAILED,
                &account_id,
                &subject_id,
                json!({
                    "scope": "media",
                    "provider_chat_id": provider_chat_id,
                    "content_type": content_type,
                    "status": "failed",
                }),
            )
            .await?;
            return Err(error);
        }
    };
    let response = WhatsAppMediaSyncResponse {
        account_id: account_id.clone(),
        provider_chat_id: provider_chat_id.clone(),
        content_type: content_type.clone(),
        runtime_kind,
        status: "synced".to_owned(),
        synced_count: items.len(),
        has_more: items.len() as i64 >= limit,
        items,
    };
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "media",
        "progress",
        json!({
            "scope": "media",
            "provider_chat_id": provider_chat_id,
            "content_type": content_type,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_PROGRESS,
        &account_id,
        &subject_id,
        json!({
            "scope": "media",
            "provider_chat_id": provider_chat_id,
            "content_type": content_type,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &subject_id,
        "media",
        "completed",
        json!({
            "scope": "media",
            "provider_chat_id": provider_chat_id,
            "content_type": content_type,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_COMPLETED,
        &account_id,
        &subject_id,
        json!({
            "scope": "media",
            "provider_chat_id": provider_chat_id,
            "content_type": content_type,
            "status": response.status,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
        }),
    )
    .await?;
    Ok(Json(response))
}

pub(crate) async fn get_whatsapp_commands(
    State(state): State<AppState>,
    Query(query): Query<WhatsAppCommandListQuery>,
) -> Result<Json<WhatsAppProviderCommandListResponse>, ApiError> {
    let account_id = query.account_id.ok_or_else(|| {
        ApiError::WhatsappWeb(WhatsappWebError::InvalidRequest(
            "account_id is required".to_owned(),
        ))
    })?;
    let command_kinds = query
        .command_kinds
        .as_deref()
        .map(parse_command_kinds)
        .unwrap_or_default();
    Ok(Json(
        whatsapp_provider_runtime_service(&state)?
            .list_provider_commands(
                &account_id,
                query.provider_chat_id.as_deref(),
                query.provider_message_id.as_deref(),
                &command_kinds,
                query.limit.unwrap_or(50),
            )
            .await?,
    ))
}

pub(crate) async fn post_whatsapp_command_retry(
    State(state): State<AppState>,
    Path(command_id): Path<String>,
) -> Result<Json<WhatsAppProviderCommand>, ApiError> {
    let command = whatsapp_provider_runtime_service(&state)?
        .manual_retry_provider_command(&command_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    publish_whatsapp_command_record_event(&state, &command, "manual_retry").await?;
    Ok(Json(command))
}

pub(crate) async fn post_whatsapp_command_dead_letter(
    State(state): State<AppState>,
    Path(command_id): Path<String>,
    Json(request): Json<WhatsAppCommandDeadLetterRequest>,
) -> Result<Json<WhatsAppProviderCommand>, ApiError> {
    let command = whatsapp_provider_runtime_service(&state)?
        .dead_letter_provider_command(&command_id, &request.reason)
        .await?
        .ok_or(ApiError::NotFound)?;
    publish_whatsapp_command_record_event(&state, &command, "manual_dead_letter").await?;
    Ok(Json(command))
}

pub(crate) async fn post_whatsapp_fixture_account(
    State(state): State<AppState>,
    Json(request): Json<WhatsappWebAccountSetupRequest>,
) -> Result<Json<WhatsappWebAccountSetupResponse>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let response = whatsapp_provider_runtime_service(&state)?
        .setup_fixture_account(&request)
        .await?;
    let account = provider_account_or_not_found(&state, &response.account_id).await?;
    sync_provider_account_signal_connection(&state, &account, None).await?;
    Ok(Json(response))
}

pub(crate) async fn post_whatsapp_fixture_message(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessage>,
) -> Result<Json<WhatsappWebMessageIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_message(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::MESSAGE_CREATED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.occurred_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "delivery_state": request.delivery_state.as_str(),
            "sender_id": request.sender_id,
            "sender_display_name": request.sender_display_name,
            "occurred_at": request.occurred_at,
            "source": "fixture_message_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_reaction(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebReaction>,
) -> Result<Json<WhatsappWebReactionIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_reaction(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::REACTION_CHANGED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "reaction_id": result.reaction_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "provider_actor_id": request.provider_actor_id,
            "sender_display_name": request.sender_display_name,
            "reaction": request.reaction,
            "is_active": request.is_active,
            "observed_at": request.observed_at,
            "source": "fixture_reaction_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_message_update(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessageUpdate>,
) -> Result<Json<WhatsappWebMessageUpdateIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_message_update(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::MESSAGE_UPDATED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "version_id": result.version_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "observed_at": request.observed_at,
            "source": "fixture_message_update_ingest",
            "edited": true,
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_message_delete(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessageDelete>,
) -> Result<Json<WhatsappWebMessageDeleteIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_message_delete(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::MESSAGE_DELETED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "tombstone_id": result.tombstone_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "reason_class": request.reason_class,
            "actor_class": request.actor_class,
            "observed_at": request.observed_at,
            "source": "fixture_message_delete_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_receipt(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebReceipt>,
) -> Result<Json<WhatsappWebReceiptIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_receipt(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::RECEIPT_CHANGED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "delivery_state": result.delivery_state,
            "observed_at": request.observed_at,
            "source": "fixture_receipt_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_media(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMedia>,
) -> Result<Json<WhatsappWebMediaIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    Ok(Json(
        whatsapp_fixture_ingest_service(&state)?
            .ingest_media(&request)
            .await?,
    ))
}

pub(crate) async fn post_whatsapp_fixture_status(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebStatus>,
) -> Result<Json<WhatsappWebStatusIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_status(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::STATUS_UPDATED,
        "whatsapp_status",
        &result.message_id,
        Some(&format!("whatsapp_status_feed:{}", request.account_id)),
        Some(&request.provider_status_id),
        request.occurred_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_status_id": request.provider_status_id,
            "sender_id": request.sender_id,
            "sender_display_name": request.sender_display_name,
            "sender_identity_kind": request.sender_identity_kind,
            "sender_address": request.sender_address,
            "sender_push_name": request.sender_push_name,
            "occurred_at": request.occurred_at,
            "status_state": "posted",
            "source": "fixture_status_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_status_view(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebStatusView>,
) -> Result<Json<WhatsappWebStatusViewIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_status_view(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::STATUS_UPDATED,
        "whatsapp_status",
        &result.message_id,
        Some(&format!("whatsapp_status_feed:{}", request.account_id)),
        Some(&request.provider_status_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_status_id": request.provider_status_id,
            "viewer_id": request.viewer_id,
            "viewer_display_name": request.viewer_display_name,
            "observed_at": request.observed_at,
            "status_state": "viewed",
            "source": "fixture_status_view_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_status_delete(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebStatusDelete>,
) -> Result<Json<WhatsappWebStatusDeleteIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_status_delete(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::STATUS_DELETED,
        "whatsapp_status",
        &result.message_id,
        Some(&format!("whatsapp_status_feed:{}", request.account_id)),
        Some(&request.provider_status_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "tombstone_id": result.tombstone_id,
            "provider_status_id": request.provider_status_id,
            "actor_class": request.actor_class,
            "reason_class": request.reason_class,
            "observed_at": request.observed_at,
            "status_state": "deleted",
            "source": "fixture_status_delete_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_presence(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebPresence>,
) -> Result<Json<WhatsappWebPresenceIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_presence(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::PRESENCE_CHANGED,
        "whatsapp_identity",
        result
            .identity_id
            .as_deref()
            .unwrap_or(request.provider_identity_id.as_str()),
        Some(&request.provider_chat_id),
        None,
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "identity_id": result.identity_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_identity_id": request.provider_identity_id,
            "identity_kind": request.identity_kind,
            "display_name": request.display_name,
            "presence_state": request.presence_state,
            "last_seen_at": request.last_seen_at,
            "observed_at": request.observed_at,
            "source": "fixture_presence_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_call(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebCall>,
) -> Result<Json<WhatsappWebCallIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_call(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::CALL_UPDATED,
        "whatsapp_call",
        &result.call_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_call_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "call_id": result.call_id,
            "raw_record_id": result.raw_record_id,
            "provider_call_id": request.provider_call_id,
            "provider_chat_id": request.provider_chat_id,
            "direction": request.direction,
            "call_state": request.call_state,
            "started_at": request.started_at,
            "ended_at": request.ended_at,
            "source": "fixture_call_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_runtime_event(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebRuntimeEvent>,
) -> Result<Json<WhatsappWebRuntimeEventIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_event(&request)
        .await?;
    let mut metadata_keys = request
        .metadata
        .as_object()
        .map(|metadata| metadata.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    metadata_keys.sort();
    publish_whatsapp_runtime_event(
        &state,
        &request.account_id,
        &request.provider_event_id,
        &request.runtime_event_kind,
        request.effective_runtime_status(),
        request.effective_lifecycle_state(),
        request.effective_severity(),
        metadata_keys,
        request.observed_at,
    )
    .await?;
    lifecycle_projection::project_runtime_bridge_lifecycle_state(&state, &request).await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_dialog(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebDialog>,
) -> Result<Json<WhatsappWebDialogIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_dialog(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::DIALOG_UPDATED,
        "whatsapp_dialog",
        &result.conversation_id,
        Some(&request.provider_chat_id),
        None,
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "conversation_id": result.conversation_id,
            "channel_id": result.channel_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "chat_title": request.chat_title,
            "chat_kind": request.chat_kind,
            "is_pinned": request.is_pinned,
            "is_archived": request.is_archived,
            "is_muted": request.is_muted,
            "is_unread": request.is_unread,
            "unread_count": request.unread_count,
            "participant_count": request.participant_count,
            "community_parent_chat_id": request.community_parent_chat_id,
            "community_parent_title": request.community_parent_title,
            "invite_link": request.invite_link,
            "is_community_root": request.is_community_root,
            "is_broadcast": request.is_broadcast,
            "is_newsletter": request.is_newsletter,
            "avatar_metadata": request.avatar_metadata,
            "provider_labels": request.provider_labels,
            "observed_at": request.observed_at,
            "source": "fixture_dialog_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_participant(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebParticipant>,
) -> Result<Json<WhatsappWebParticipantIngestResult>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_participant(&request)
        .await?;
    let provider_member_id = request.effective_provider_member_id();
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::PARTICIPANT_CHANGED,
        "whatsapp_participant",
        &result.participant_id,
        Some(&request.provider_chat_id),
        Some(provider_member_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "conversation_id": result.conversation_id,
            "participant_id": result.participant_id,
            "identity_id": result.identity_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_member_id": provider_member_id,
            "provider_identity_id": request.provider_identity_id,
            "display_name": request.display_name,
            "role": result.current_role,
            "status": result.current_status,
            "previous_role": result.previous_role,
            "previous_status": result.previous_status,
            "role_changed": result.role_changed,
            "membership_changed": result.membership_changed,
            "observed_at": request.observed_at,
            "source": "fixture_participant_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_fixture_authorized_session(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppAuthorizedSessionCredentialWrite>,
) -> Result<Json<WhatsAppCredentialBinding>, ApiError> {
    ensure_fixture_routes_enabled(&state)?;
    let provider_account = provider_account_or_not_found(&state, &request.account_id).await?;
    let lifecycle_source =
        lifecycle_projection::authorized_session_lifecycle_source(&state, &request.account_id)
            .await?;
    let binding = whatsapp_provider_runtime_service(&state)?
        .store_authorized_session_credential(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    let status = whatsapp_provider_runtime_service(&state)?
        .runtime_status(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request.account_id,
        )
        .await?;
    sync_whatsapp_runtime_signal_connection(&state, &provider_account, &status).await?;
    capture_whatsapp_runtime_lifecycle_signal(&state, &status, lifecycle_source).await?;
    publish_whatsapp_runtime_status_event(&state, &status, lifecycle_source).await?;
    publish_whatsapp_session_link_state_event(
        &state,
        &status.account_id,
        &status.provider_shape,
        &status.runtime_kind,
        &status.status,
        lifecycle_source,
        status.updated_at,
    )
    .await?;
    Ok(Json(binding))
}

pub(crate) async fn post_whatsapp_runtime_bridge_message(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessage>,
) -> Result<Json<WhatsappWebMessageIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_message(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::MESSAGE_CREATED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.occurred_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "delivery_state": request.delivery_state.as_str(),
            "sender_id": request.sender_id,
            "sender_display_name": request.sender_display_name,
            "occurred_at": request.occurred_at,
            "source": "runtime_bridge_message_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_reaction(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebReaction>,
) -> Result<Json<WhatsappWebReactionIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_reaction(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::REACTION_CHANGED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "reaction_id": result.reaction_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "provider_actor_id": request.provider_actor_id,
            "sender_display_name": request.sender_display_name,
            "reaction": request.reaction,
            "is_active": request.is_active,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_reaction_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_message_update(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessageUpdate>,
) -> Result<Json<WhatsappWebMessageUpdateIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_message_update(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::MESSAGE_UPDATED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "version_id": result.version_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_message_update_ingest",
            "edited": true,
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_message_delete(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMessageDelete>,
) -> Result<Json<WhatsappWebMessageDeleteIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_message_delete(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::MESSAGE_DELETED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "tombstone_id": result.tombstone_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "reason_class": request.reason_class,
            "actor_class": request.actor_class,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_message_delete_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_receipt(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebReceipt>,
) -> Result<Json<WhatsappWebReceiptIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_receipt(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::RECEIPT_CHANGED,
        "whatsapp_message",
        &result.message_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_message_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_message_id": request.provider_message_id,
            "delivery_state": result.delivery_state,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_receipt_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_media(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebMedia>,
) -> Result<Json<WhatsappWebMediaIngestResult>, ApiError> {
    Ok(Json(
        whatsapp_fixture_ingest_service(&state)?
            .ingest_runtime_bridge_media(&request)
            .await?,
    ))
}

pub(crate) async fn post_whatsapp_runtime_bridge_media_lifecycle(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppRuntimeBridgeMediaLifecycleRequest>,
) -> Result<StatusCode, ApiError> {
    let event_type =
        event_types::runtime_bridge_media(&request.media_direction, &request.lifecycle_phase)?;
    let progress_percent = match request.lifecycle_phase.as_str() {
        "requested" => None,
        "failed" => request.progress_percent,
        "started" => Some(request.progress_percent.unwrap_or(0)),
        "progress" | "completed" => Some(request.progress_percent.unwrap_or(100)),
        _ => None,
    };
    let payload = json!({
        "account_id": request.account_id,
        "command_id": request.command_id,
        "provider_chat_id": request.provider_chat_id,
        "provider_message_id": request.provider_message_id,
        "provider_media_id": request.provider_media_id,
        "attachment_id": request.attachment_id,
        "blob_id": request.blob_id,
        "content_type": request.content_type,
        "filename": request.filename,
        "progress_percent": progress_percent,
        "error_code": request.error_code,
        "error_message": request.error_message,
        "runtime_blockers": request.runtime_blockers.unwrap_or_default(),
        "source": "runtime_bridge_media_lifecycle",
    });
    publish_whatsapp_media_event(&state, event_type, &request.command_id, payload).await?;
    Ok(StatusCode::ACCEPTED)
}

pub(crate) async fn post_whatsapp_runtime_bridge_sync_lifecycle(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppRuntimeBridgeSyncLifecycleRequest>,
) -> Result<StatusCode, ApiError> {
    let scope = match request.scope.as_str() {
        "chats" | "history" | "members" | "statuses" | "presence" | "calls" | "contacts"
        | "media" => request.scope.as_str(),
        _ => {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "unsupported runtime bridge sync scope `{}`",
                request.scope
            ))
            .into());
        }
    };
    let phase = match request.phase.as_str() {
        "started" | "progress" | "completed" | "failed" => request.phase.as_str(),
        _ => {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "unsupported runtime bridge sync phase `{}`",
                request.phase
            ))
            .into());
        }
    };
    let subject_id = request
        .subject_id
        .clone()
        .or_else(|| request.provider_chat_id.clone())
        .unwrap_or_else(|| request.account_id.clone());
    let payload = json!({
        "scope": scope,
        "status": phase,
        "subject_id": subject_id,
        "provider_chat_id": request.provider_chat_id,
        "synced_count": request.synced_count,
        "has_more": request.has_more,
        "error_code": request.error_code,
        "error_message": request.error_message,
        "source": "runtime_bridge_sync_lifecycle",
    });
    capture_whatsapp_sync_runtime_signal(
        &state,
        &request.account_id,
        &subject_id,
        scope,
        phase,
        payload.clone(),
    )
    .await?;
    let event_type = match phase {
        "started" => whatsapp_event_types::SYNC_STARTED,
        "progress" => whatsapp_event_types::SYNC_PROGRESS,
        "completed" => whatsapp_event_types::SYNC_COMPLETED,
        "failed" => whatsapp_event_types::SYNC_FAILED,
        _ => unreachable!(),
    };
    publish_whatsapp_sync_event(
        &state,
        event_type,
        &request.account_id,
        &subject_id,
        payload,
    )
    .await?;
    Ok(StatusCode::ACCEPTED)
}

pub(crate) async fn post_whatsapp_runtime_bridge_claim_commands(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppRuntimeBridgeClaimCommandsRequest>,
) -> Result<Json<WhatsAppRuntimeBridgeClaimCommandsResponse>, ApiError> {
    let limit = request.limit.unwrap_or(20).clamp(1, 100);
    let account_id = request
        .account_id
        .as_deref()
        .map(|value| required_string("account_id", value))
        .transpose()?;
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let now = Utc::now();
    let projection =
        makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore::new(
            pool.clone(),
        );
    let account_lookup =
        makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
            pool.clone(),
        );
    let imported = crate::integrations::whatsapp::runtime::command_execution::import_canonical_provider_commands(
        &pool,
        &projection,
        now,
        limit,
    )
    .await?;
    for command in &imported {
        publish_whatsapp_command_record_event(
            &state,
            &command.clone().into(),
            "canonical_provider_command_import",
        )
        .await?;
    }
    let recovered = crate::integrations::whatsapp::runtime::recover_stale_live_executing_commands(
        &pool,
        &account_lookup,
        now,
        account_id.as_deref(),
    )
    .await?;
    for command in &recovered {
        publish_whatsapp_command_record_event(&state, &command.clone().into(), "stale_recovery")
            .await?;
    }
    let claimed = crate::integrations::whatsapp::runtime::command_execution::claim_due_live_commands_for_execution(
        &pool,
        &account_lookup,
        now,
        limit,
        account_id.as_deref(),
    )
    .await?;
    let mut items = Vec::with_capacity(claimed.len());
    for command in claimed {
        let account = provider_account_or_not_found(&state, &command.account_id).await?;
        let runtime_status = whatsapp_provider_runtime_service(&state)?
            .runtime_status(
                &whatsapp_secret_reference_store(&state)?,
                &state.vault,
                &command.account_id,
            )
            .await?;
        publish_whatsapp_command_record_event(
            &state,
            &command.clone().into(),
            "runtime_bridge_claim",
        )
        .await?;
        items.push(WhatsAppRuntimeBridgeExecutableCommand {
            command_id: command.command_id,
            account_id: command.account_id,
            command_kind: command.command_kind,
            provider_kind: account.provider_kind.as_str().to_owned(),
            provider_shape: runtime_status.provider_shape,
            runtime_kind: runtime_status.runtime_kind,
            lifecycle_state: account
                .config
                .get("lifecycle_state")
                .and_then(Value::as_str)
                .map(str::to_owned),
            session_restore_available: runtime_status.session_restore_available,
            runtime_blockers: runtime_status.runtime_blockers,
            provider_chat_id: command.provider_chat_id,
            provider_message_id: command.provider_message_id,
            idempotency_key: command.idempotency_key,
            capability_state: command.capability_state,
            action_class: command.action_class,
            confirmation_decision: command.confirmation_decision,
            status: command.status,
            retry_count: command.retry_count,
            max_retries: command.max_retries,
            payload: command.payload,
            target_ref: command.target_ref,
            audit_metadata: command.audit_metadata,
            provider_state: command.provider_state,
            result_payload: command.result_payload,
            next_attempt_at: command.next_attempt_at,
            last_attempt_at: command.last_attempt_at,
            created_at: command.created_at,
            updated_at: command.updated_at,
        });
    }
    Ok(Json(WhatsAppRuntimeBridgeClaimCommandsResponse { items }))
}

pub(crate) async fn post_whatsapp_runtime_bridge_command_failed(
    State(state): State<AppState>,
    Path(command_id): Path<String>,
    Json(request): Json<WhatsAppRuntimeBridgeCommandFailedRequest>,
) -> Result<Json<WhatsAppProviderCommand>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let updated =
        crate::integrations::whatsapp::runtime::retry_execution::reschedule_failed_command(
            &pool,
            &command_id,
            Utc::now(),
            &required_string("error_message", &request.error_message)?,
            request
                .error_code
                .as_deref()
                .map(|value| required_string("error_code", value))
                .transpose()?
                .as_deref(),
            request.retry_after_seconds,
        )
        .await?
        .ok_or(ApiError::NotFound)?;
    let command: WhatsAppProviderCommand = updated.into();
    publish_whatsapp_command_record_event(&state, &command, "runtime_bridge_failed").await?;
    Ok(Json(command))
}

pub(crate) async fn post_whatsapp_runtime_bridge_status(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebStatus>,
) -> Result<Json<WhatsappWebStatusIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_status(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::STATUS_UPDATED,
        "whatsapp_status",
        &result.message_id,
        Some(&format!("whatsapp_status_feed:{}", request.account_id)),
        Some(&request.provider_status_id),
        request.occurred_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_status_id": request.provider_status_id,
            "sender_id": request.sender_id,
            "sender_display_name": request.sender_display_name,
            "sender_identity_kind": request.sender_identity_kind,
            "sender_address": request.sender_address,
            "sender_push_name": request.sender_push_name,
            "occurred_at": request.occurred_at,
            "status_state": "posted",
            "source": "runtime_bridge_status_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_status_view(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebStatusView>,
) -> Result<Json<WhatsappWebStatusViewIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_status_view(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::STATUS_UPDATED,
        "whatsapp_status",
        &result.message_id,
        Some(&format!("whatsapp_status_feed:{}", request.account_id)),
        Some(&request.provider_status_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "provider_status_id": request.provider_status_id,
            "viewer_id": request.viewer_id,
            "viewer_display_name": request.viewer_display_name,
            "observed_at": request.observed_at,
            "status_state": "viewed",
            "source": "runtime_bridge_status_view_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_status_delete(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebStatusDelete>,
) -> Result<Json<WhatsappWebStatusDeleteIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_status_delete(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::STATUS_DELETED,
        "whatsapp_status",
        &result.message_id,
        Some(&format!("whatsapp_status_feed:{}", request.account_id)),
        Some(&request.provider_status_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "message_id": result.message_id,
            "raw_record_id": result.raw_record_id,
            "tombstone_id": result.tombstone_id,
            "provider_status_id": request.provider_status_id,
            "actor_class": request.actor_class,
            "reason_class": request.reason_class,
            "observed_at": request.observed_at,
            "status_state": "deleted",
            "source": "runtime_bridge_status_delete_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_presence(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebPresence>,
) -> Result<Json<WhatsappWebPresenceIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_presence(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::PRESENCE_CHANGED,
        "whatsapp_identity",
        result
            .identity_id
            .as_deref()
            .unwrap_or(request.provider_identity_id.as_str()),
        Some(&request.provider_chat_id),
        None,
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "identity_id": result.identity_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_identity_id": request.provider_identity_id,
            "identity_kind": request.identity_kind,
            "display_name": request.display_name,
            "presence_state": request.presence_state,
            "last_seen_at": request.last_seen_at,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_presence_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_call(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebCall>,
) -> Result<Json<WhatsappWebCallIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_call(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::CALL_UPDATED,
        "whatsapp_call",
        &result.call_id,
        Some(&request.provider_chat_id),
        Some(&request.provider_call_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "call_id": result.call_id,
            "raw_record_id": result.raw_record_id,
            "provider_call_id": request.provider_call_id,
            "provider_chat_id": request.provider_chat_id,
            "direction": request.direction,
            "call_state": request.call_state,
            "started_at": request.started_at,
            "ended_at": request.ended_at,
            "source": "runtime_bridge_call_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_runtime_event(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebRuntimeEvent>,
) -> Result<Json<WhatsappWebRuntimeEventIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_runtime_event(&request)
        .await?;
    let mut metadata_keys = request
        .metadata
        .as_object()
        .map(|metadata| metadata.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    metadata_keys.sort();
    publish_whatsapp_runtime_event(
        &state,
        &request.account_id,
        &request.provider_event_id,
        &request.runtime_event_kind,
        request.effective_runtime_status(),
        request.effective_lifecycle_state(),
        request.effective_severity(),
        metadata_keys,
        request.observed_at,
    )
    .await?;
    lifecycle_projection::project_runtime_bridge_lifecycle_state(&state, &request).await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_dialog(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebDialog>,
) -> Result<Json<WhatsappWebDialogIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_dialog(&request)
        .await?;
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::DIALOG_UPDATED,
        "whatsapp_dialog",
        &result.conversation_id,
        Some(&request.provider_chat_id),
        None,
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "conversation_id": result.conversation_id,
            "channel_id": result.channel_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "chat_title": request.chat_title,
            "chat_kind": request.chat_kind,
            "is_pinned": request.is_pinned,
            "is_archived": request.is_archived,
            "is_muted": request.is_muted,
            "is_unread": request.is_unread,
            "unread_count": request.unread_count,
            "participant_count": request.participant_count,
            "community_parent_chat_id": request.community_parent_chat_id,
            "community_parent_title": request.community_parent_title,
            "invite_link": request.invite_link,
            "is_community_root": request.is_community_root,
            "is_broadcast": request.is_broadcast,
            "is_newsletter": request.is_newsletter,
            "avatar_metadata": request.avatar_metadata,
            "provider_labels": request.provider_labels,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_dialog_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_participant(
    State(state): State<AppState>,
    Json(request): Json<NewWhatsappWebParticipant>,
) -> Result<Json<WhatsappWebParticipantIngestResult>, ApiError> {
    let result = whatsapp_fixture_ingest_service(&state)?
        .ingest_runtime_bridge_participant(&request)
        .await?;
    let provider_member_id = request.effective_provider_member_id();
    publish_whatsapp_projection_event(
        &state,
        whatsapp_event_types::PARTICIPANT_CHANGED,
        "whatsapp_participant",
        &result.participant_id,
        Some(&request.provider_chat_id),
        Some(provider_member_id),
        request.observed_at,
        json!({
            "account_id": request.account_id,
            "conversation_id": result.conversation_id,
            "participant_id": result.participant_id,
            "identity_id": result.identity_id,
            "raw_record_id": result.raw_record_id,
            "provider_chat_id": request.provider_chat_id,
            "provider_member_id": provider_member_id,
            "provider_identity_id": request.provider_identity_id,
            "display_name": request.display_name,
            "role": result.current_role,
            "status": result.current_status,
            "previous_role": result.previous_role,
            "previous_status": result.previous_status,
            "role_changed": result.role_changed,
            "membership_changed": result.membership_changed,
            "observed_at": request.observed_at,
            "source": "runtime_bridge_participant_ingest",
        }),
    )
    .await?;
    Ok(Json(result))
}

pub(crate) async fn post_whatsapp_runtime_bridge_authorized_session(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppAuthorizedSessionCredentialWrite>,
) -> Result<Json<WhatsAppCredentialBinding>, ApiError> {
    let provider_account = provider_account_or_not_found(&state, &request.account_id).await?;
    let lifecycle_source =
        lifecycle_projection::authorized_session_lifecycle_source(&state, &request.account_id)
            .await?;
    let binding = whatsapp_provider_runtime_service(&state)?
        .store_authorized_session_credential(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request,
        )
        .await?;
    let status = whatsapp_provider_runtime_service(&state)?
        .runtime_status(
            &whatsapp_secret_reference_store(&state)?,
            &state.vault,
            &request.account_id,
        )
        .await?;
    sync_whatsapp_runtime_signal_connection(&state, &provider_account, &status).await?;
    capture_whatsapp_runtime_lifecycle_signal(&state, &status, lifecycle_source).await?;
    publish_whatsapp_runtime_status_event(&state, &status, lifecycle_source).await?;
    publish_whatsapp_session_link_state_event(
        &state,
        &status.account_id,
        &status.provider_shape,
        &status.runtime_kind,
        &status.status,
        lifecycle_source,
        status.updated_at,
    )
    .await?;
    Ok(Json(binding))
}
