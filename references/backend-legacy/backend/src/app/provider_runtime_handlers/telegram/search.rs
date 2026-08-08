use axum::Json;
use axum::extract::{Path, Query, State};
use makosh_communications_api::attachments::CanonicalMessageAttachmentReadPort;
use serde::{Deserialize, Serialize};

use super::chats::{
    dedupe_and_sort_chats, includes_telegram_channel_kind, includes_whatsapp_channel_kind,
    list_canonical_communication_conversations, normalized_channel_kind,
};
use crate::app::api_support::stores::integration_stores::*;
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::application::telegram_runtime;
use crate::domains::communications::messages::provider_channel_store::ProviderChannelMessageStore;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::chats::TelegramChat;
use makosh_communications_api::conversations::ConversationReadPort;
use makosh_communications_api::provider_messages::ProviderChannelMessage;

const COMMUNICATION_SEARCH_CHANNEL_KINDS: &[&str] =
    &["telegram_user", "telegram_bot", "whatsapp_web"];

#[derive(Deserialize)]
pub(crate) struct TelegramMessageSearchQuery {
    pub(crate) q: String,
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramProviderSearchCommand {
    pub(crate) account_id: String,
    pub(crate) q: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramMediaSearchQuery {
    pub(crate) q: Option<String>,
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) kind: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramChatSearchQuery {
    pub(crate) q: String,
    pub(crate) account_id: Option<String>,
    pub(crate) channel_kind: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct TelegramPinnedMessagesQuery {
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct TelegramSearchResponse {
    pub(crate) query: String,
    pub(crate) items: Vec<crate::integrations::telegram::client::models::messages::TelegramMessage>,
    pub(crate) total: usize,
}

#[derive(Serialize)]
pub(crate) struct TelegramProviderSearchResponse {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) query: String,
    pub(crate) limit: i32,
    pub(crate) status: String,
    pub(crate) error: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct TelegramChatSearchResponse {
    pub(crate) query: String,
    pub(crate) items: Vec<TelegramChat>,
    pub(crate) total: usize,
}

#[derive(Serialize)]
pub(crate) struct TelegramMediaItem {
    pub(crate) attachment_id: Option<String>,
    pub(crate) message_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) file_name: String,
    pub(crate) kind: String,
    pub(crate) mime_type: Option<String>,
    pub(crate) size_bytes: Option<i64>,
    pub(crate) occurred_at: Option<String>,
    pub(crate) download_state: String,
    pub(crate) tdlib_file_id: Option<i64>,
    pub(crate) provider_attachment_id: Option<String>,
    pub(crate) local_path: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct TelegramMediaSearchResponse {
    pub(crate) query: Option<String>,
    pub(crate) source: String,
    pub(crate) provider_search_attempted: bool,
    pub(crate) provider_search_error: Option<String>,
    pub(crate) items: Vec<TelegramMediaItem>,
}

/// GET /api/v1/communications/search/messages?q=&account_id=&provider_chat_id=&limit=
pub(crate) async fn search_telegram_messages(
    State(state): State<AppState>,
    Query(query): Query<TelegramMessageSearchQuery>,
) -> Result<Json<TelegramSearchResponse>, ApiError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let search_q = query.q.trim().to_owned();
    let channel_kinds = search_channel_kinds(query.channel_kind.as_deref());

    if search_q.is_empty() {
        return Err(ApiError::Telegram(TelegramError::InvalidRequest(
            "search query `q` is required".to_owned(),
        )));
    }

    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let items: Vec<crate::integrations::telegram::client::models::messages::TelegramMessage> =
        ProviderChannelMessageStore::new(pool)
            .search_messages(
                query.account_id.as_deref(),
                query.provider_chat_id.as_deref(),
                &search_q,
                channel_kinds,
                limit,
            )
            .await
            .map_err(TelegramError::from)?
            .into_iter()
            .map(provider_channel_message_to_search_message)
            .collect();

    Ok(Json(TelegramSearchResponse {
        query: search_q,
        total: items.len(),
        items,
    }))
}

/// POST /api/v1/integrations/telegram/provider-search
pub(crate) async fn post_telegram_provider_search(
    State(state): State<AppState>,
    Json(payload): Json<TelegramProviderSearchCommand>,
) -> Result<Json<TelegramProviderSearchResponse>, ApiError> {
    let limit = payload.limit.unwrap_or(50).clamp(1, 200);
    let search_q = payload.q.trim().to_owned();
    let account_id = payload.account_id.trim();

    if account_id.is_empty() {
        return Err(ApiError::Telegram(TelegramError::InvalidRequest(
            "search payload account_id is required".to_owned(),
        )));
    }

    if search_q.is_empty() {
        return Err(ApiError::Telegram(TelegramError::InvalidRequest(
            "search query `q` is required".to_owned(),
        )));
    }

    let runtime_context = telegram_runtime_use_case_context(&state)?;
    let result = telegram_runtime::refresh_provider_search(
        &runtime_context,
        account_id.to_owned(),
        payload.provider_chat_id.clone(),
        search_q.clone(),
        limit as i32,
    )
    .await;

    let (status, error) = match result {
        Ok(()) => ("queued".to_owned(), None),
        Err(error) => {
            tracing::debug!(
                error = %error,
                account_id = %account_id,
                "post_telegram_provider_search: TDLib provider search failed"
            );
            ("failed".to_owned(), Some(error.to_string()))
        }
    };

    Ok(Json(TelegramProviderSearchResponse {
        account_id: account_id.to_owned(),
        provider_chat_id: payload.provider_chat_id,
        query: search_q,
        limit: limit as i32,
        status,
        error,
    }))
}

/// GET /api/v1/communications/conversations/search?q=&account_id=&limit=
pub(crate) async fn search_telegram_chats(
    State(state): State<AppState>,
    Query(query): Query<TelegramChatSearchQuery>,
) -> Result<Json<TelegramChatSearchResponse>, ApiError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let search_q = query.q.trim().to_owned();
    let channel_kind = normalized_channel_kind(query.channel_kind.as_deref());

    if search_q.is_empty() {
        return Err(ApiError::Telegram(TelegramError::InvalidRequest(
            "search query `q` is required".to_owned(),
        )));
    }

    let mut items = if includes_telegram_channel_kind(channel_kind) {
        telegram_provider_runtime_service(&state)?
            .search_chats(query.account_id.as_deref(), &search_q, limit)
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
                Some(&search_q),
                limit,
            )
            .await?,
        );
    }
    dedupe_and_sort_chats(&mut items, limit);

    Ok(Json(TelegramChatSearchResponse {
        query: search_q,
        total: items.len(),
        items,
    }))
}

/// GET /api/v1/communications/conversations/{conversation_id}/pinned-messages?limit=
pub(crate) async fn get_telegram_pinned_messages(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Query(query): Query<TelegramPinnedMessagesQuery>,
) -> Result<
    Json<crate::app::api_support::messaging_integrations::TelegramMessageListResponse>,
    ApiError,
> {
    let limit = query.limit.unwrap_or(100).clamp(1, 200);
    let items = match telegram_provider_runtime_service(&state)?
        .pinned_messages(&conversation_id, limit)
        .await
    {
        Ok(items) => items,
        Err(TelegramError::InvalidRequest(_)) => {
            canonical_pinned_messages(&state, &conversation_id, limit).await?
        }
        Err(error) => return Err(error.into()),
    };

    Ok(Json(
        crate::app::api_support::messaging_integrations::TelegramMessageListResponse { items },
    ))
}

/// GET /api/v1/communications/search/media?account_id=&provider_chat_id=&kind=&limit=
pub(crate) async fn search_telegram_media(
    State(state): State<AppState>,
    Query(query): Query<TelegramMediaSearchQuery>,
) -> Result<Json<TelegramMediaSearchResponse>, ApiError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let channel_kinds = search_channel_kinds(query.channel_kind.as_deref());
    let search_q = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let messages = ProviderChannelMessageStore::new(pool.clone())
        .recent_messages(
            query.account_id.as_deref(),
            query.provider_chat_id.as_deref(),
            channel_kinds,
            limit,
        )
        .await
        .map_err(TelegramError::from)?
        .into_iter()
        .map(provider_channel_message_to_search_message)
        .collect::<Vec<_>>();

    let mut items = Vec::new();
    for msg in &messages {
        if let Some(arr) = msg
            .metadata
            .get("attachments")
            .or(msg.metadata.get("files"))
            .and_then(|v| v.as_array())
        {
            for att in arr {
                let kind = att
                    .get("attachment_type")
                    .or(att.get("kind"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("file");
                if query.kind.as_deref().is_some_and(|fk| kind != fk) {
                    continue;
                }
                let file_name = att
                    .get("filename")
                    .or(att.get("file_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_owned();
                let mime_type = att
                    .get("content_type")
                    .or(att.get("mime_type"))
                    .and_then(|v| v.as_str())
                    .map(ToOwned::to_owned);
                if let Some(search_q) = search_q {
                    let search_q = search_q.to_lowercase();
                    let mut haystack = vec![
                        file_name.to_lowercase(),
                        kind.to_lowercase(),
                        msg.provider_message_id.to_lowercase(),
                    ];
                    if let Some(mime) = mime_type.as_deref() {
                        haystack.push(mime.to_lowercase());
                    }
                    if !haystack.into_iter().any(|value| value.contains(&search_q)) {
                        continue;
                    }
                }
                items.push(TelegramMediaItem {
                    attachment_id: att
                        .get("attachment_id")
                        .and_then(|v| v.as_str())
                        .map(ToOwned::to_owned),
                    message_id: msg.message_id.clone(),
                    provider_message_id: msg.provider_message_id.clone(),
                    provider_chat_id: msg.provider_chat_id.clone().unwrap_or_default(),
                    file_name,
                    kind: kind.to_owned(),
                    mime_type,
                    size_bytes: att
                        .get("size")
                        .or(att.get("size_bytes"))
                        .and_then(|v| v.as_i64()),
                    occurred_at: msg.occurred_at.map(|t| t.to_rfc3339()),
                    download_state: att
                        .get("download_state")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_owned(),
                    tdlib_file_id: att
                        .get("tdlib_file_id")
                        .or_else(|| {
                            att.get("metadata")
                                .and_then(|value| value.get("tdlib_file_id"))
                        })
                        .and_then(|v| v.as_i64()),
                    provider_attachment_id: att
                        .get("provider_attachment_id")
                        .or_else(|| att.get("attachment_id"))
                        .and_then(|v| v.as_str())
                        .map(ToOwned::to_owned),
                    local_path: att
                        .get("local_path")
                        .and_then(|v| v.as_str())
                        .map(ToOwned::to_owned),
                });
            }
        }
    }

    let message_ids = messages
        .iter()
        .map(|message| message.message_id.clone())
        .collect::<Vec<_>>();
    if !message_ids.is_empty() {
        let attachment_rows =
            makosh_communications_postgres::attachments::CanonicalMessageAttachmentReadStore::new(
                pool.clone(),
            )
            .list_for_messages(&message_ids)
            .await
            .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?;

        for row in attachment_rows {
            let file_name = row.filename.unwrap_or_else(|| "unknown".to_owned());
            let mime_type = row.content_type;
            let provider_message_id = row.provider_message_id;
            let provider_attachment_id = row.provider_attachment_id;
            let kind = media_kind_from_mime_type(&mime_type);
            if query
                .kind
                .as_deref()
                .is_some_and(|filter_kind| filter_kind != kind)
            {
                continue;
            }
            if let Some(search_q) = search_q {
                let needle = search_q.to_lowercase();
                let haystacks = [
                    file_name.to_lowercase(),
                    mime_type.to_lowercase(),
                    kind.to_owned(),
                    provider_message_id.to_lowercase(),
                    provider_attachment_id.to_lowercase(),
                ];
                if !haystacks.into_iter().any(|value| value.contains(&needle)) {
                    continue;
                }
            }
            items.push(TelegramMediaItem {
                attachment_id: Some(row.attachment_id),
                message_id: row.message_id,
                provider_message_id,
                provider_chat_id: row.provider_chat_id,
                file_name,
                kind: kind.to_owned(),
                mime_type: Some(mime_type),
                size_bytes: Some(row.size_bytes),
                occurred_at: row.occurred_at.map(|timestamp| timestamp.to_rfc3339()),
                download_state: "available".to_owned(),
                tdlib_file_id: None,
                provider_attachment_id: Some(provider_attachment_id),
                local_path: row.storage_path,
            });
        }
    }
    dedupe_media_items(&mut items);

    Ok(Json(TelegramMediaSearchResponse {
        query: search_q.map(ToOwned::to_owned),
        source: "projection".to_owned(),
        provider_search_attempted: false,
        provider_search_error: None,
        items,
    }))
}

fn provider_channel_message_to_search_message(
    message: ProviderChannelMessage,
) -> crate::integrations::telegram::client::models::messages::TelegramMessage {
    crate::integrations::telegram::client::models::messages::TelegramMessage {
        message_id: message.message_id,
        raw_record_id: message.raw_record_id,
        account_id: message.account_id,
        provider_message_id: message.provider_record_id,
        provider_chat_id: Some(message.conversation_id),
        chat_title: message.subject,
        sender: message.sender,
        sender_display_name: message.sender_display_name,
        text: message.body_text,
        occurred_at: message.occurred_at,
        projected_at: message.projected_at,
        channel_kind: message.channel_kind,
        delivery_state: message.delivery_state,
        metadata: message.message_metadata,
    }
}

fn media_kind_from_mime_type(content_type: &str) -> &'static str {
    if content_type.starts_with("image/") {
        "photo"
    } else if content_type.starts_with("video/") {
        "video"
    } else if content_type.starts_with("audio/") {
        "audio"
    } else {
        "document"
    }
}

fn dedupe_media_items(items: &mut Vec<TelegramMediaItem>) {
    let mut seen = std::collections::HashSet::new();
    items.retain(|item| {
        seen.insert(format!(
            "{}:{}:{}:{}",
            item.message_id,
            item.provider_attachment_id.as_deref().unwrap_or(""),
            item.local_path.as_deref().unwrap_or(""),
            item.file_name
        ))
    });
}

fn search_channel_kinds(channel_kind: Option<&str>) -> &'static [&'static str] {
    match normalized_channel_kind(channel_kind) {
        Some("telegram") => &["telegram_user", "telegram_bot"],
        Some("telegram_user") => &["telegram_user"],
        Some("telegram_bot") => &["telegram_bot"],
        Some("whatsapp") => &["whatsapp_web"],
        Some("whatsapp_web") => &["whatsapp_web"],
        _ => COMMUNICATION_SEARCH_CHANNEL_KINDS,
    }
}

async fn canonical_pinned_messages(
    state: &AppState,
    conversation_id: &str,
    limit: i64,
) -> Result<Vec<crate::integrations::telegram::client::models::messages::TelegramMessage>, ApiError>
{
    let pool = state
        .database
        .pool()
        .expect("database pool configured")
        .clone();
    let row =
        makosh_communications_postgres::conversations::ConversationReadStore::new(pool.clone())
            .get_conversation(conversation_id, &["whatsapp_web"])
            .await
            .map_err(|error| TelegramError::InvalidRequest(error.to_string()))?;
    let Some(row) = row else {
        return Err(ApiError::NotFound);
    };
    let account_id = row.account_id;
    let provider_conversation_id = row.provider_conversation_id;
    Ok(ProviderChannelMessageStore::new(pool)
        .pinned_messages(
            &account_id,
            &provider_conversation_id,
            &["whatsapp_web"],
            limit,
        )
        .await
        .map_err(TelegramError::from)?
        .into_iter()
        .map(provider_channel_message_to_search_message)
        .collect())
}
