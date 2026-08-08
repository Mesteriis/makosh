use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use makosh_communications_api::conversations::ConversationReadPort;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;

use super::helpers::{
    AUDIT_ACTOR_ID, ensure_telegram_account_operation_allowed, publish_telegram_event,
};
use crate::app::api_support::{
    messaging_integrations::*,
    stores::{domain_stores::*, integration_stores::*},
};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::application::telegram_runtime;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::chats::{
    TelegramChat, TelegramChatGroupFilterListResponse, TelegramChatMember,
};
use crate::integrations::telegram::runtime::models::{
    TelegramChatSyncRequest, TelegramChatSyncResponse, TelegramHistorySyncRequest,
    TelegramHistorySyncResponse,
};
use crate::platform::audit::models::NewApiAuditRecord;

use crate::platform::events::bus::telegram_event_types;

const COMMUNICATION_CONVERSATION_CHANNEL_KINDS: &[&str] =
    &["telegram_user", "telegram_bot", "whatsapp_web"];

fn build_event(
    event_type: &str,
    account_id: &str,
    subject_id: &str,
    payload: serde_json::Value,
) -> NewEventEnvelope {
    let now = Utc::now();
    NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": subject_id, "kind": "telegram_sync"}),
    )
    .payload(payload)
    .build()
    .expect("event envelope must be valid")
}

pub(crate) async fn get_telegram_chats(
    State(state): State<AppState>,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<TelegramChatListResponse>, ApiError> {
    let channel_kind = normalized_channel_kind(query.channel_kind.as_deref());
    let limit = query.limit.unwrap_or(50);
    let mut items = if includes_telegram_channel_kind(channel_kind) {
        telegram_provider_runtime_service(&state)?
            .list_chats(query.account_id.as_deref(), limit)
            .await?
    } else {
        Vec::new()
    };
    if includes_whatsapp_channel_kind(channel_kind) {
        items.extend(
            list_canonical_communication_conversations(
                &state,
                query.account_id.as_deref(),
                channel_kind,
                None,
                limit,
            )
            .await?,
        );
    }
    dedupe_and_sort_chats(&mut items, limit);

    Ok(Json(TelegramChatListResponse { items }))
}

pub(crate) async fn get_telegram_folders(
    State(state): State<AppState>,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<TelegramChatGroupFilterListResponse>, ApiError> {
    let items = telegram_provider_runtime_service(&state)?
        .list_chat_group_filters(query.account_id.as_deref())
        .await?;

    Ok(Json(TelegramChatGroupFilterListResponse { items }))
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatDetailResponse {
    pub(crate) item: TelegramChat,
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatMemberListResponse {
    pub(crate) items: Vec<TelegramChatMember>,
    pub(crate) next_cursor: Option<String>,
}

#[derive(serde::Deserialize)]
pub(crate) struct TelegramChatMembersQuery {
    pub(crate) query: Option<String>,
    pub(crate) role: Option<String>,
    pub(crate) limit: Option<i64>,
    pub(crate) cursor: Option<String>,
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatMembersSyncResponse {
    pub(crate) telegram_chat_id: String,
    pub(crate) synced_count: usize,
    pub(crate) items: Vec<TelegramChatMember>,
}

pub(crate) async fn get_telegram_chat_detail(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
) -> Result<Json<TelegramChatDetailResponse>, ApiError> {
    let item = if let Some(item) = telegram_provider_runtime_service(&state)?
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
    {
        item
    } else {
        canonical_communication_conversation(&state, &telegram_chat_id)
            .await?
            .ok_or_else(|| {
                ApiError::Telegram(TelegramError::InvalidRequest(format!(
                    "Telegram chat `{telegram_chat_id}` was not found"
                )))
            })?
    };

    Ok(Json(TelegramChatDetailResponse { item }))
}

pub(crate) async fn get_telegram_chat_members(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Query(query): Query<TelegramChatMembersQuery>,
) -> Result<Json<TelegramChatMemberListResponse>, ApiError> {
    let limit = query.limit.unwrap_or(50);
    let items = match telegram_provider_runtime_service(&state)?
        .list_chat_members(
            &telegram_chat_id,
            query.query.as_deref(),
            query.role.as_deref(),
            limit,
            query.cursor.as_deref(),
        )
        .await
    {
        Ok(items) => items,
        Err(TelegramError::InvalidRequest(_)) => {
            list_canonical_conversation_members(
                &state,
                &telegram_chat_id,
                query.query.as_deref(),
                query.role.as_deref(),
                limit,
                query.cursor.as_deref(),
            )
            .await?
        }
        Err(error) => return Err(error.into()),
    };
    let next_cursor = if items.len() >= limit as usize {
        let offset = query
            .cursor
            .as_deref()
            .unwrap_or("0")
            .parse::<i64>()
            .unwrap_or(0)
            .max(0)
            + limit;
        Some(offset.to_string())
    } else {
        None
    };

    Ok(Json(TelegramChatMemberListResponse { items, next_cursor }))
}

pub(crate) async fn post_telegram_chat_members_sync(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
) -> Result<Json<TelegramChatMembersSyncResponse>, ApiError> {
    let telegram_provider_runtime_service = telegram_provider_runtime_service(&state)?;
    let chat = telegram_provider_runtime_service
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(format!(
                "Telegram chat `{telegram_chat_id}` was not found"
            )))
        })?;
    ensure_telegram_account_operation_allowed(&state, &chat.account_id, "participants.sync")
        .await?;
    let started = build_event(
        telegram_event_types::SYNC_STARTED,
        &chat.account_id,
        &telegram_chat_id,
        json!({
            "scope": "members",
            "provider_chat_id": &chat.provider_chat_id,
        }),
    );
    publish_telegram_event(&state, started).await?;

    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let items = match telegram_runtime::sync_chat_members(&runtime_context, &telegram_chat_id).await
    {
        Ok(items) => items,
        Err(error) => {
            let failed = build_event(
                telegram_event_types::SYNC_FAILED,
                &chat.account_id,
                &telegram_chat_id,
                json!({
                    "scope": "members",
                    "provider_chat_id": &chat.provider_chat_id,
                    "status": "failed",
                }),
            );
            publish_telegram_event(&state, failed).await?;
            return Err(error.into());
        }
    };

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_participants_sync(
            AUDIT_ACTOR_ID,
            &telegram_chat_id,
            &chat.account_id,
            &chat.provider_chat_id,
            items.len() as i64,
        ))
        .await?;

    let progress = build_event(
        telegram_event_types::SYNC_PROGRESS,
        &chat.account_id,
        &telegram_chat_id,
        json!({
            "scope": "members",
            "provider_chat_id": &chat.provider_chat_id,
            "synced_count": items.len(),
            "status": "completed",
        }),
    );
    publish_telegram_event(&state, progress).await?;

    let completed = build_event(
        telegram_event_types::SYNC_COMPLETED,
        &chat.account_id,
        &telegram_chat_id,
        json!({
            "scope": "members",
            "provider_chat_id": &chat.provider_chat_id,
            "synced_count": items.len(),
            "status": "completed",
        }),
    );
    publish_telegram_event(&state, completed).await?;

    Ok(Json(TelegramChatMembersSyncResponse {
        telegram_chat_id,
        synced_count: items.len(),
        items,
    }))
}

pub(crate) async fn list_canonical_communication_conversations(
    state: &AppState,
    account_id: Option<&str>,
    channel_kind: Option<&str>,
    query: Option<&str>,
    limit: i64,
) -> Result<Vec<TelegramChat>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let limit = limit.clamp(1, 200);
    let like_pattern = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{value}%"));
    let canonical_channel_kinds = canonical_conversation_channel_kinds(channel_kind);
    let rows = makosh_communications_postgres::conversations::ConversationReadStore::new(pool)
        .list_conversations(
            account_id,
            canonical_channel_kinds,
            like_pattern.as_deref(),
            limit,
        )
        .await
        .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?;

    rows.into_iter()
        .filter_map(|row| canonical_conversation_to_chat(row).transpose())
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub(crate) async fn canonical_communication_conversation(
    state: &AppState,
    conversation_id: &str,
) -> Result<Option<TelegramChat>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let row = makosh_communications_postgres::conversations::ConversationReadStore::new(pool)
        .get_conversation(conversation_id, COMMUNICATION_CONVERSATION_CHANNEL_KINDS)
        .await
        .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?;

    match row {
        Some(row) => Ok(canonical_conversation_to_chat(row)?),
        None => canonical_message_row_to_chat(state, conversation_id)
            .await
            .map_err(Into::into),
    }
}

async fn canonical_message_row_to_chat(
    state: &AppState,
    conversation_id: &str,
) -> Result<Option<TelegramChat>, TelegramError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let row = makosh_communications_postgres::conversations::ConversationReadStore::new(pool)
        .get_conversation_from_message_projection(
            conversation_id,
            COMMUNICATION_CONVERSATION_CHANNEL_KINDS,
        )
        .await
        .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?;
    let Some(row) = row else {
        return Ok(None);
    };
    if !matches!(row.channel_kind.as_str(), "whatsapp_web") {
        return Ok(None);
    }
    let provider_chat_id = row.provider_conversation_id.clone();
    Ok(Some(TelegramChat {
        telegram_chat_id: provider_chat_id.clone(),
        account_id: row.account_id,
        provider_chat_id,
        chat_kind: "group".to_owned(),
        title: row.conversation_id,
        username: None,
        sync_state: "fixture".to_owned(),
        last_message_at: row.last_message_at,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }))
}

async fn list_canonical_conversation_members(
    state: &AppState,
    conversation_id: &str,
    query: Option<&str>,
    role: Option<&str>,
    limit: i64,
    cursor: Option<&str>,
) -> Result<Vec<TelegramChatMember>, ApiError> {
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let limit = limit.clamp(1, 200);
    let offset = cursor.unwrap_or("0").parse::<i64>().unwrap_or(0).max(0);
    let like_pattern = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{value}%"));
    let rows = makosh_communications_postgres::conversations::ConversationReadStore::new(pool)
        .list_conversation_members(
            conversation_id,
            COMMUNICATION_CONVERSATION_CHANNEL_KINDS,
            like_pattern.as_deref(),
            role.map(str::trim).filter(|value| !value.is_empty()),
            offset,
            limit,
        )
        .await
        .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?;

    rows.into_iter()
        .map(canonical_member_to_chat_member)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

fn canonical_conversation_to_chat(
    row: makosh_communications_api::conversations::CanonicalConversationRecord,
) -> Result<Option<TelegramChat>, TelegramError> {
    if !matches!(row.channel_kind.as_str(), "whatsapp_web") {
        return Ok(None);
    }
    Ok(Some(TelegramChat {
        telegram_chat_id: row.conversation_id,
        account_id: row.account_id,
        provider_chat_id: row.provider_conversation_id,
        chat_kind: row
            .metadata
            .get("chat_kind")
            .and_then(|value| value.as_str())
            .unwrap_or("group")
            .to_owned(),
        title: row.title,
        username: None,
        sync_state: "fixture".to_owned(),
        last_message_at: row.last_message_at,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }))
}

pub(crate) fn normalized_channel_kind(channel_kind: Option<&str>) -> Option<&str> {
    channel_kind
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(crate) fn includes_telegram_channel_kind(channel_kind: Option<&str>) -> bool {
    matches!(
        channel_kind,
        None | Some("telegram") | Some("telegram_user") | Some("telegram_bot")
    )
}

pub(crate) fn includes_whatsapp_channel_kind(channel_kind: Option<&str>) -> bool {
    matches!(channel_kind, None | Some("whatsapp") | Some("whatsapp_web"))
}

fn canonical_conversation_channel_kinds(channel_kind: Option<&str>) -> &'static [&'static str] {
    match channel_kind {
        Some("whatsapp") => &["whatsapp_web"],
        Some("whatsapp_web") => &["whatsapp_web"],
        Some("telegram") => &["telegram_user", "telegram_bot"],
        Some("telegram_user") => &["telegram_user"],
        Some("telegram_bot") => &["telegram_bot"],
        _ => COMMUNICATION_CONVERSATION_CHANNEL_KINDS,
    }
}

fn canonical_member_to_chat_member(
    row: makosh_communications_api::conversations::CanonicalConversationMemberRecord,
) -> Result<TelegramChatMember, TelegramError> {
    let participant_metadata = row.participant_metadata;
    let identity_metadata = row.identity_metadata;
    let provider_identity_id = row.provider_identity_id;
    let provider_member_id = provider_identity_id
        .clone()
        .unwrap_or_else(|| row.participant_id.clone());
    let is_admin = participant_metadata
        .get("is_admin")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let is_owner = participant_metadata
        .get("is_owner")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    Ok(TelegramChatMember {
        sender_id: provider_member_id.clone(),
        sender_display_name: Some(row.display_name),
        message_count: 0,
        last_message_at: row.last_message_at,
        source: "canonical_communications".to_owned(),
        provider_member_id,
        username: None,
        role: Some(row.role),
        status: participant_metadata
            .get("status")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned),
        is_admin,
        is_owner,
        permissions: json!({
            "identity_kind": row.identity_kind,
            "address": row.address,
            "participant_metadata": participant_metadata,
            "identity_metadata": identity_metadata,
        }),
        observed_at: None,
    })
}

pub(crate) fn dedupe_and_sort_chats(items: &mut Vec<TelegramChat>, limit: i64) {
    let mut by_id = std::collections::BTreeMap::new();
    for item in items.drain(..) {
        by_id.entry(item.telegram_chat_id.clone()).or_insert(item);
    }
    let mut merged = by_id.into_values().collect::<Vec<_>>();
    merged.sort_by(|left, right| {
        right
            .last_message_at
            .cmp(&left.last_message_at)
            .then_with(|| left.telegram_chat_id.cmp(&right.telegram_chat_id))
    });
    merged.truncate(limit.clamp(1, 200) as usize);
    *items = merged;
}

pub(crate) async fn post_telegram_sync_chats(
    State(state): State<AppState>,
    Json(request): Json<TelegramChatSyncRequest>,
) -> Result<Json<TelegramChatSyncResponse>, ApiError> {
    let started = build_event(
        telegram_event_types::SYNC_STARTED,
        &request.account_id,
        &request.account_id,
        json!({
            "scope": "chats",
        }),
    );
    publish_telegram_event(&state, started).await?;

    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = match telegram_runtime::sync_chats(&runtime_context, &request).await {
        Ok(response) => response,
        Err(error) => {
            let failed = build_event(
                telegram_event_types::SYNC_FAILED,
                &request.account_id,
                &request.account_id,
                json!({
                    "scope": "chats",
                    "status": "failed",
                }),
            );
            publish_telegram_event(&state, failed).await?;
            return Err(error.into());
        }
    };

    let progress = build_event(
        telegram_event_types::SYNC_PROGRESS,
        &request.account_id,
        &request.account_id,
        json!({
            "scope": "chats",
            "synced_count": response.synced_count,
            "status": &response.status,
        }),
    );
    publish_telegram_event(&state, progress).await?;

    let completed = build_event(
        telegram_event_types::SYNC_COMPLETED,
        &request.account_id,
        &request.account_id,
        json!({
            "scope": "chats",
            "synced_count": response.synced_count,
            "status": &response.status,
        }),
    );
    publish_telegram_event(&state, completed).await?;

    Ok(Json(response))
}

pub(crate) async fn post_telegram_sync_history(
    State(state): State<AppState>,
    Json(request): Json<TelegramHistorySyncRequest>,
) -> Result<Json<TelegramHistorySyncResponse>, ApiError> {
    let started = build_event(
        telegram_event_types::SYNC_STARTED,
        &request.account_id,
        &request.provider_chat_id,
        json!({
            "scope": "history",
            "provider_chat_id": &request.provider_chat_id,
            "mode": &request.mode,
        }),
    );
    publish_telegram_event(&state, started).await?;

    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let response = match telegram_runtime::sync_history(&runtime_context, &request).await {
        Ok(response) => response,
        Err(error) => {
            let failed = build_event(
                telegram_event_types::SYNC_FAILED,
                &request.account_id,
                &request.provider_chat_id,
                json!({
                    "scope": "history",
                    "provider_chat_id": &request.provider_chat_id,
                    "mode": &request.mode,
                    "status": "failed",
                }),
            );
            publish_telegram_event(&state, failed).await?;
            return Err(error.into());
        }
    };

    let progress = build_event(
        telegram_event_types::SYNC_PROGRESS,
        &request.account_id,
        &request.provider_chat_id,
        json!({
            "scope": "history",
            "provider_chat_id": &request.provider_chat_id,
            "mode": &request.mode,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
            "status": &response.status,
        }),
    );
    publish_telegram_event(&state, progress).await?;

    let completed = build_event(
        telegram_event_types::SYNC_COMPLETED,
        &request.account_id,
        &request.provider_chat_id,
        json!({
            "scope": "history",
            "provider_chat_id": &request.provider_chat_id,
            "mode": &request.mode,
            "synced_count": response.synced_count,
            "has_more": response.has_more,
            "status": &response.status,
        }),
    );
    publish_telegram_event(&state, completed).await?;

    Ok(Json(response))
}
