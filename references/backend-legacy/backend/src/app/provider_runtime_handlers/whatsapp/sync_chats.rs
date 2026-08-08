//! Hidden WebView WhatsApp chat synchronization endpoint and query.

use axum::Json;
use axum::extract::State;
use makosh_communications_api::conversations::ConversationReadPort;
use serde_json::{Value, json};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::platform::events::bus::whatsapp_event_types;

use super::{
    WhatsAppChatSyncItem, WhatsAppChatSyncRequest, WhatsAppChatSyncResponse,
    capture_whatsapp_sync_runtime_signal, current_whatsapp_runtime_kind,
    ensure_whatsapp_sync_supported, publish_whatsapp_sync_event, required_string,
};

pub(crate) async fn post_whatsapp_sync_chats(
    State(state): State<AppState>,
    Json(request): Json<WhatsAppChatSyncRequest>,
) -> Result<Json<WhatsAppChatSyncResponse>, ApiError> {
    let account_id = required_string("account_id", &request.account_id)?;
    ensure_whatsapp_sync_supported(&state, &account_id, "sync_chats").await?;
    let limit = request.limit.unwrap_or(50).clamp(1, 200);
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &account_id,
        "chats",
        "started",
        json!({"scope": "chats"}),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_STARTED,
        &account_id,
        &account_id,
        json!({"scope": "chats"}),
    )
    .await?;
    let runtime_kind = current_whatsapp_runtime_kind(&state, &account_id).await?;
    let items = match list_whatsapp_sync_chats(&state, &account_id, limit).await {
        Ok(items) => items,
        Err(error) => {
            capture_whatsapp_sync_runtime_signal(
                &state,
                &account_id,
                &account_id,
                "chats",
                "failed",
                json!({"scope": "chats", "status": "failed"}),
            )
            .await?;
            publish_whatsapp_sync_event(
                &state,
                whatsapp_event_types::SYNC_FAILED,
                &account_id,
                &account_id,
                json!({"scope": "chats", "status": "failed"}),
            )
            .await?;
            return Err(error);
        }
    };
    let response = WhatsAppChatSyncResponse {
        account_id: account_id.clone(),
        runtime_kind,
        status: "synced".to_owned(),
        synced_count: items.len(),
        items,
    };
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &account_id,
        "chats",
        "progress",
        json!({
            "scope": "chats",
            "status": response.status,
            "synced_count": response.synced_count,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_PROGRESS,
        &account_id,
        &account_id,
        json!({
            "scope": "chats",
            "status": response.status,
            "synced_count": response.synced_count,
        }),
    )
    .await?;
    capture_whatsapp_sync_runtime_signal(
        &state,
        &account_id,
        &account_id,
        "chats",
        "completed",
        json!({
            "scope": "chats",
            "status": response.status,
            "synced_count": response.synced_count,
        }),
    )
    .await?;
    publish_whatsapp_sync_event(
        &state,
        whatsapp_event_types::SYNC_COMPLETED,
        &account_id,
        &account_id,
        json!({
            "scope": "chats",
            "status": response.status,
            "synced_count": response.synced_count,
        }),
    )
    .await?;
    Ok(Json(response))
}

async fn list_whatsapp_sync_chats(
    state: &AppState,
    account_id: &str,
    limit: i64,
) -> Result<Vec<WhatsAppChatSyncItem>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let rows = makosh_communications_postgres::conversations::ConversationReadStore::new(pool)
        .list_conversations(Some(account_id), &["whatsapp_web"], None, limit)
        .await
        .map_err(|error| WhatsappWebError::InvalidRequest(error.to_string()))?;

    rows.into_iter()
        .map(|row| {
            let metadata = row.metadata;
            Ok(WhatsAppChatSyncItem {
                conversation_id: row.conversation_id,
                account_id: row.account_id,
                channel_kind: row.channel_kind,
                provider_chat_id: row.provider_conversation_id,
                title: row.title,
                chat_kind: metadata
                    .get("chat_kind")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                is_archived: metadata
                    .get("is_archived")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                is_pinned: metadata
                    .get("is_pinned")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                is_muted: metadata
                    .get("is_muted")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                is_unread: metadata
                    .get("is_unread")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                unread_count: metadata.get("unread_count").and_then(Value::as_i64),
                participant_count: metadata.get("participant_count").and_then(Value::as_i64),
                community_parent_chat_id: metadata
                    .get("community_parent_chat_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                community_parent_title: metadata
                    .get("community_parent_title")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                invite_link: metadata
                    .get("invite_link")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                is_community_root: metadata
                    .get("is_community_root")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                is_broadcast: metadata
                    .get("is_broadcast")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                is_newsletter: metadata
                    .get("is_newsletter")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                avatar_metadata: metadata
                    .get("avatar_metadata")
                    .cloned()
                    .unwrap_or_else(|| json!({})),
                provider_labels: metadata
                    .get("provider_labels")
                    .and_then(Value::as_array)
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_owned)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<_>, WhatsappWebError>>()
        .map_err(Into::into)
}
