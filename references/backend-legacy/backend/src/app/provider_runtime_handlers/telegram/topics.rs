use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::app::api_support::{
    messaging_integrations::*,
    stores::{domain_stores::*, integration_stores::*},
};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::application::telegram_runtime;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::topics::{
    TelegramTopic, TelegramTopicCloseRequest, TelegramTopicCreateRequest,
    TelegramTopicLifecycleResponse,
};
use crate::platform::audit::models::NewApiAuditRecord;

use crate::platform::events::bus::telegram_event_types;

use super::helpers::{
    AUDIT_ACTOR_ID, ensure_telegram_account_operation_allowed, publish_telegram_event,
};

#[derive(Deserialize)]
pub(crate) struct TelegramTopicsQuery {
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramTopicSearchQuery {
    pub(crate) q: String,
    pub(crate) telegram_chat_id: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct TelegramTopicListApiResponse {
    pub(crate) telegram_chat_id: String,
    pub(crate) items: Vec<TelegramTopic>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramTopicMessagesQuery {
    pub(crate) limit: Option<i64>,
}

fn build_command_event(
    account_id: &str,
    command_id: &str,
    provider_chat_id: &str,
    action: &str,
    status: &str,
    extra: serde_json::Value,
) -> NewEventEnvelope {
    let now = Utc::now();
    let mut payload = json!({
        "command_id": command_id,
        "account_id": account_id,
        "provider_chat_id": provider_chat_id,
        "action": action,
        "status": status,
    });
    if let (Some(payload_obj), Some(extra_obj)) = (payload.as_object_mut(), extra.as_object()) {
        for (key, value) in extra_obj {
            payload_obj.insert(key.clone(), value.clone());
        }
    }
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_topic_command_{}",
            now.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::COMMAND_STATUS_CHANGED,
        now,
        json!({"channel": "telegram", "account_id": account_id}),
        json!({"id": command_id, "kind": "telegram_command"}),
    )
    .payload(payload)
    .build()
    .expect("event envelope must be valid")
}

/// GET /api/v1/communications/conversations/{conversation_id}/topics
///
/// Attempts a live TDLib fetch to refresh the topic projection before serving DB rows.
/// Falls back to the DB projection if TDLib is unavailable or the account is in fixture mode.
pub(crate) async fn get_telegram_topics(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Query(query): Query<TelegramTopicsQuery>,
) -> Result<Json<TelegramTopicListApiResponse>, ApiError> {
    let store = telegram_provider_runtime_service(&state)?;
    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let chat = store
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(format!(
                "telegram chat `{telegram_chat_id}` was not found"
            )))
        })?;
    ensure_telegram_account_operation_allowed(&state, &chat.account_id, "topics.list").await?;

    let runtime_context = telegram_runtime_use_case_context(&state)?;
    if let Err(error) =
        telegram_runtime::refresh_forum_topics(&runtime_context, &telegram_chat_id).await
    {
        tracing::debug!(
            error = %error,
            telegram_chat_id = %telegram_chat_id,
            "get_telegram_topics: TDLib live sync failed, serving DB projection"
        );
    }

    let items = store.list_topics(&telegram_chat_id, limit).await?.items;

    Ok(Json(TelegramTopicListApiResponse {
        telegram_chat_id,
        items,
    }))
}

/// POST /api/v1/communications/conversations/{conversation_id}/topics
pub(crate) async fn post_telegram_topic_create(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramTopicCreateRequest>,
) -> Result<Json<TelegramTopicLifecycleResponse>, ApiError> {
    request.validate()?;
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "topics.create").await?;
    let store = telegram_provider_runtime_service(&state)?;
    let chat = store
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
        .ok_or_else(|| {
            ApiError::Telegram(TelegramError::InvalidRequest(format!(
                "telegram chat `{telegram_chat_id}` was not found"
            )))
        })?;
    let command_id = request.command_id.clone();

    store
        .insert_command(
            &command_id,
            &request.account_id,
            "topic_create",
            &format!(
            "topic_create:{}:{}",
            request.provider_chat_id,
            Utc::now().timestamp_millis()
            ),
            &request.provider_chat_id,
            None,
            "available",
            "provider_write",
            "confirmed",
            AUDIT_ACTOR_ID,
            json!({"title": request.title.trim()}),
            json!({"telegram_chat_id": telegram_chat_id, "provider_chat_id": request.provider_chat_id}),
            json!({"source": "telegram_topic_create"}),
        )
        .await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_topic_create(
            AUDIT_ACTOR_ID,
            &chat.telegram_chat_id,
            &request.account_id,
            &request.provider_chat_id,
        ))
        .await?;

    publish_telegram_event(
        &state,
        build_command_event(
            &request.account_id,
            &command_id,
            &request.provider_chat_id,
            "topic_create",
            "queued",
            json!({
                "telegram_chat_id": chat.telegram_chat_id,
                "title": request.title.trim(),
            }),
        ),
    )
    .await?;

    Ok(Json(TelegramTopicLifecycleResponse {
        operation: "topic_create".to_owned(),
        topic_id: None,
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        provider_topic_id: None,
        status: "queued".to_owned(),
        timestamp: Utc::now(),
        command_id,
    }))
}

/// GET /api/v1/communications/topics/{topic_id}
pub(crate) async fn get_telegram_topic_detail(
    State(state): State<AppState>,
    Path(topic_id): Path<String>,
) -> Result<Json<TelegramTopic>, ApiError> {
    let store = telegram_provider_runtime_service(&state)?;
    let topic = store
        .get_topic(&topic_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(topic))
}

/// POST /api/v1/communications/topics/{topic_id}/close
pub(crate) async fn post_telegram_topic_close(
    State(state): State<AppState>,
    Path(topic_id): Path<String>,
    Json(request): Json<TelegramTopicCloseRequest>,
) -> Result<Json<TelegramTopicLifecycleResponse>, ApiError> {
    request.validate()?;
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "topics.close").await?;
    let store = telegram_provider_runtime_service(&state)?;
    let topic = store
        .get_topic(&topic_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let command_kind = if request.is_closed {
        "topic_close"
    } else {
        "topic_reopen"
    };
    let command_id = request.command_id.clone();

    store
        .insert_command(
            &command_id,
            &request.account_id,
            command_kind,
            &format!(
                "{command_kind}:{}:{}",
                topic.provider_topic_id,
                Utc::now().timestamp_millis()
            ),
            &request.provider_chat_id,
            None,
            "available",
            "provider_write",
            "confirmed",
            AUDIT_ACTOR_ID,
            json!({
            "provider_topic_id": topic.provider_topic_id,
            "is_closed": request.is_closed,
            }),
            json!({
            "topic_id": topic.topic_id,
            "telegram_chat_id": topic.telegram_chat_id,
            "provider_chat_id": topic.provider_chat_id,
            "provider_topic_id": topic.provider_topic_id,
            }),
            json!({"source": "telegram_topic_close"}),
        )
        .await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::telegram_topic_close(
            AUDIT_ACTOR_ID,
            &topic.topic_id,
            &request.account_id,
            &request.provider_chat_id,
            request.is_closed,
        ))
        .await?;

    publish_telegram_event(
        &state,
        build_command_event(
            &request.account_id,
            &command_id,
            &request.provider_chat_id,
            command_kind,
            "queued",
            json!({
                "topic_id": topic.topic_id,
                "telegram_chat_id": topic.telegram_chat_id,
                "provider_topic_id": topic.provider_topic_id,
                "is_closed": request.is_closed,
            }),
        ),
    )
    .await?;

    Ok(Json(TelegramTopicLifecycleResponse {
        operation: command_kind.to_owned(),
        topic_id: Some(topic.topic_id),
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        provider_topic_id: Some(topic.provider_topic_id),
        status: "queued".to_owned(),
        timestamp: Utc::now(),
        command_id,
    }))
}

/// GET /api/v1/communications/topics/{topic_id}/messages
/// Returns messages whose metadata.forum_topic_id matches topic_id.
pub(crate) async fn get_telegram_topic_messages(
    State(state): State<AppState>,
    Path(topic_id): Path<String>,
    Query(query): Query<TelegramTopicMessagesQuery>,
) -> Result<Json<TelegramMessageListResponse>, ApiError> {
    let store = telegram_provider_runtime_service(&state)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);

    let message_ids = store.list_topic_message_ids(&topic_id, limit).await?;

    if message_ids.is_empty() {
        return Ok(Json(TelegramMessageListResponse { items: vec![] }));
    }

    let items = store.messages_by_ids(&message_ids).await?;

    Ok(Json(TelegramMessageListResponse { items }))
}

/// GET /api/v1/communications/topics/search?q=&telegram_chat_id=&limit=
pub(crate) async fn search_telegram_topics(
    State(state): State<AppState>,
    Query(query): Query<TelegramTopicSearchQuery>,
) -> Result<Json<TelegramTopicListApiResponse>, ApiError> {
    let store = telegram_provider_runtime_service(&state)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let search_q = query.q.trim().to_owned();
    let telegram_chat_id = query.telegram_chat_id.trim().to_owned();

    if search_q.is_empty() {
        return Err(ApiError::Telegram(TelegramError::InvalidRequest(
            "search query `q` is required".to_owned(),
        )));
    }

    if telegram_chat_id.is_empty() {
        return Err(ApiError::Telegram(TelegramError::InvalidRequest(
            "search query `telegram_chat_id` is required".to_owned(),
        )));
    }

    let items = store
        .search_topics(&telegram_chat_id, &search_q, limit)
        .await?;

    Ok(Json(TelegramTopicListApiResponse {
        telegram_chat_id,
        items,
    }))
}
