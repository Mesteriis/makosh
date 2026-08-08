use axum::Json;
use axum::extract::{Path, State};
use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;

use super::helpers::{
    AUDIT_ACTOR_ID, ensure_telegram_account_operation_allowed, publish_telegram_event,
};
use crate::app::api_support::stores::{domain_stores::*, integration_stores::*};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::integrations::telegram::client::commands;
use crate::integrations::telegram::client::models::chats::TelegramChat;
use crate::platform::audit::models::NewApiAuditRecord;

use crate::platform::events::bus::telegram_event_types;
use crate::platform::settings::store::ApplicationSettingsStore;

const TELEGRAM_READ_RECEIPT_REPORTS_ENABLED_SETTING_KEY: &str =
    "communications.telegram.read_receipt_reports_enabled";

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

#[allow(clippy::too_many_arguments)]
fn build_command_event(
    account_id: &str,
    command_id: &str,
    provider_chat_id: &str,
    telegram_chat_id: Option<&str>,
    provider_message_id: Option<&str>,
    action: &str,
    status: &str,
    chat: Option<&TelegramChat>,
) -> NewEventEnvelope {
    build_event(
        telegram_event_types::COMMAND_STATUS_CHANGED,
        account_id,
        command_id,
        json!({
            "command_id": command_id,
            "action": action,
            "provider_chat_id": provider_chat_id,
            "telegram_chat_id": telegram_chat_id,
            "message_id": provider_message_id,
            "status": status,
            "chat": chat,
        }),
    )
}

fn build_chat_flag_event(
    event_type: &str,
    request: &TelegramChatActionRequest,
    telegram_chat_id: &str,
    flag_key: &str,
    flag_value: bool,
    chat: Option<&TelegramChat>,
) -> NewEventEnvelope {
    build_event(
        event_type,
        &request.account_id,
        telegram_chat_id,
        json!({
            "provider_chat_id": &request.provider_chat_id,
            "telegram_chat_id": telegram_chat_id,
            flag_key: flag_value,
            "chat": chat,
        }),
    )
}

fn build_chat_updated_event(
    request: &TelegramChatActionRequest,
    telegram_chat_id: &str,
    action: &str,
    chat: Option<&TelegramChat>,
) -> NewEventEnvelope {
    build_event(
        telegram_event_types::CHAT_UPDATED,
        &request.account_id,
        telegram_chat_id,
        json!({
            "provider_chat_id": &request.provider_chat_id,
            "telegram_chat_id": telegram_chat_id,
            "action": action,
            "chat": chat,
        }),
    )
}

#[derive(serde::Deserialize)]
pub(crate) struct TelegramChatActionRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    #[serde(default)]
    pub(crate) last_read_inbox_provider_message_id: Option<String>,
}

#[derive(serde::Deserialize)]
pub(crate) struct TelegramChatHistoryPolicyRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) full_history_sync_enabled: bool,
}

#[derive(serde::Deserialize)]
pub(crate) struct TelegramChatReadReceiptPolicyRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) read_receipt_reports_enabled: bool,
}

#[derive(serde::Deserialize)]
pub(crate) struct TelegramChatUnreadCounterPolicyRequest {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) hide_unread_counter: bool,
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatActionResponse {
    pub(crate) telegram_chat_id: String,
    pub(crate) action: String,
    pub(crate) status: String,
    pub(crate) metadata: serde_json::Value,
}

#[derive(serde::Serialize)]
pub(crate) struct TelegramChatLifecycleCommandResponse {
    pub(crate) telegram_chat_id: Option<String>,
    pub(crate) provider_chat_id: String,
    pub(crate) action: String,
    pub(crate) status: String,
    pub(crate) command_id: String,
}

pub(crate) async fn put_telegram_chat_history_policy(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatHistoryPolicyRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    let store = telegram_provider_runtime_service(&state)?;
    let chat = store.telegram_chat_by_id(&telegram_chat_id).await?.ok_or(
        ApiError::InvalidCommunicationQuery("Telegram chat was not found"),
    )?;
    if chat.account_id != request.account_id.trim()
        || chat.provider_chat_id != request.provider_chat_id.trim()
    {
        return Err(ApiError::InvalidCommunicationQuery(
            "Telegram chat does not belong to the supplied account and provider chat",
        ));
    }

    let metadata = store
        .set_chat_metadata_bool(
            &telegram_chat_id,
            "full_history_sync_enabled",
            request.full_history_sync_enabled,
        )
        .await?;
    let action_request = TelegramChatActionRequest {
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        last_read_inbox_provider_message_id: None,
    };
    publish_chat_updated_event(
        &state,
        &action_request,
        &telegram_chat_id,
        "history_policy_updated",
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "history_policy_updated".to_owned(),
        status: "applied".to_owned(),
        metadata,
    }))
}

pub(crate) async fn put_telegram_chat_read_receipt_policy(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatReadReceiptPolicyRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    let store = telegram_provider_runtime_service(&state)?;
    let chat = store.telegram_chat_by_id(&telegram_chat_id).await?.ok_or(
        ApiError::InvalidCommunicationQuery("Telegram chat was not found"),
    )?;
    if chat.account_id != request.account_id.trim()
        || chat.provider_chat_id != request.provider_chat_id.trim()
    {
        return Err(ApiError::InvalidCommunicationQuery(
            "Telegram chat does not belong to the supplied account and provider chat",
        ));
    }

    let metadata = store
        .set_chat_metadata_bool(
            &telegram_chat_id,
            "read_receipt_reports_enabled",
            request.read_receipt_reports_enabled,
        )
        .await?;
    let action_request = TelegramChatActionRequest {
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        last_read_inbox_provider_message_id: None,
    };
    publish_chat_updated_event(
        &state,
        &action_request,
        &telegram_chat_id,
        "read_receipt_policy_updated",
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "read_receipt_policy_updated".to_owned(),
        status: "applied".to_owned(),
        metadata,
    }))
}

pub(crate) async fn put_telegram_chat_unread_counter_policy(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatUnreadCounterPolicyRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    let store = telegram_provider_runtime_service(&state)?;
    let chat = store.telegram_chat_by_id(&telegram_chat_id).await?.ok_or(
        ApiError::InvalidCommunicationQuery("Telegram chat was not found"),
    )?;
    if chat.account_id != request.account_id.trim()
        || chat.provider_chat_id != request.provider_chat_id.trim()
    {
        return Err(ApiError::InvalidCommunicationQuery(
            "Telegram chat does not belong to the supplied account and provider chat",
        ));
    }

    let metadata = store
        .set_chat_metadata_bool(
            &telegram_chat_id,
            "hide_unread_counter",
            request.hide_unread_counter,
        )
        .await?;
    let action_request = TelegramChatActionRequest {
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        last_read_inbox_provider_message_id: None,
    };
    publish_chat_updated_event(
        &state,
        &action_request,
        &telegram_chat_id,
        "unread_counter_policy_updated",
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "unread_counter_policy_updated".to_owned(),
        status: "applied".to_owned(),
        metadata,
    }))
}

async fn record_dialog_command(
    state: &AppState,
    telegram_chat_id: &str,
    request: &TelegramChatActionRequest,
    command_kind: &str,
    action_class: &str,
) -> Result<String, ApiError> {
    record_chat_lifecycle_command(
        state,
        Some(telegram_chat_id),
        request,
        command_kind,
        action_class,
        if command_kind == "mark_read" {
            request.last_read_inbox_provider_message_id.as_deref()
        } else {
            None
        },
    )
    .await
}

async fn record_chat_lifecycle_command(
    state: &AppState,
    telegram_chat_id: Option<&str>,
    request: &TelegramChatActionRequest,
    command_kind: &str,
    action_class: &str,
    provider_message_id: Option<&str>,
) -> Result<String, ApiError> {
    record_chat_lifecycle_command_with_payload(
        state,
        telegram_chat_id,
        request,
        command_kind,
        action_class,
        provider_message_id,
        false,
        json!({
            "source": "telegram_chat_lifecycle",
            "last_read_inbox_provider_message_id": provider_message_id,
        }),
        json!({
            "source": "telegram_chat_lifecycle",
            "last_read_inbox_provider_message_id": provider_message_id,
        }),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn record_chat_lifecycle_command_with_payload(
    state: &AppState,
    telegram_chat_id: Option<&str>,
    request: &TelegramChatActionRequest,
    command_kind: &str,
    action_class: &str,
    provider_message_id: Option<&str>,
    skip_default_audit: bool,
    payload: serde_json::Value,
    audit_metadata: serde_json::Value,
) -> Result<String, ApiError> {
    let service = telegram_provider_runtime_service(state)?;
    let command_id = commands::new_command_id();
    let target_subject = telegram_chat_id.unwrap_or(request.provider_chat_id.trim());
    let target_ref = if let Some(telegram_chat_id) = telegram_chat_id {
        json!({
            "telegram_chat_id": telegram_chat_id,
            "provider_chat_id": &request.provider_chat_id,
            "provider_message_id": provider_message_id
        })
    } else {
        json!({
            "provider_chat_id": &request.provider_chat_id,
            "provider_message_id": provider_message_id
        })
    };

    let _cmd = service
        .insert_command(
            &command_id,
            &request.account_id,
            command_kind,
            &format!(
                "{command_kind}:{}:{}",
                target_subject,
                Utc::now().timestamp_millis()
            ),
            &request.provider_chat_id,
            provider_message_id,
            "available",
            action_class,
            "confirmed",
            AUDIT_ACTOR_ID,
            payload,
            target_ref,
            audit_metadata,
        )
        .await?;

    if !skip_default_audit {
        api_audit_log(state)?
            .record(&NewApiAuditRecord::telegram_chat_action(
                AUDIT_ACTOR_ID,
                telegram_chat_id,
                &request.account_id,
                &request.provider_chat_id,
                provider_message_id,
                command_kind,
            ))
            .await?;
    }

    let chat = if let Some(telegram_chat_id) = telegram_chat_id {
        service.telegram_chat_by_id(telegram_chat_id).await?
    } else {
        None
    };
    let command_event = build_command_event(
        &request.account_id,
        &command_id,
        &request.provider_chat_id,
        telegram_chat_id,
        provider_message_id,
        command_kind,
        "queued",
        chat.as_ref(),
    );
    publish_telegram_event(state, command_event).await?;

    Ok(command_id)
}

pub(crate) async fn post_telegram_chat_join(
    State(state): State<AppState>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatLifecycleCommandResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "participants.join")
        .await?;
    let command_id =
        record_chat_lifecycle_command(&state, None, &request, "join", "provider_write", None)
            .await?;

    Ok(Json(TelegramChatLifecycleCommandResponse {
        telegram_chat_id: None,
        provider_chat_id: request.provider_chat_id,
        action: "join".to_owned(),
        status: "queued".to_owned(),
        command_id,
    }))
}

pub(crate) async fn post_telegram_chat_leave(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatLifecycleCommandResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "participants.leave")
        .await?;
    let command_id = record_chat_lifecycle_command(
        &state,
        Some(&telegram_chat_id),
        &request,
        "leave",
        "provider_write",
        None,
    )
    .await?;

    Ok(Json(TelegramChatLifecycleCommandResponse {
        telegram_chat_id: Some(telegram_chat_id),
        provider_chat_id: request.provider_chat_id,
        action: "leave".to_owned(),
        status: "queued".to_owned(),
        command_id,
    }))
}

async fn publish_chat_flag_event(
    state: &AppState,
    request: &TelegramChatActionRequest,
    telegram_chat_id: &str,
    event_type: &str,
    flag_key: &str,
    flag_value: bool,
) -> Result<(), ApiError> {
    let service = telegram_provider_runtime_service(state)?;
    let chat = service.telegram_chat_by_id(telegram_chat_id).await?;
    let event = build_chat_flag_event(
        event_type,
        request,
        telegram_chat_id,
        flag_key,
        flag_value,
        chat.as_ref(),
    );
    publish_telegram_event(state, event).await
}

async fn publish_chat_updated_event(
    state: &AppState,
    request: &TelegramChatActionRequest,
    telegram_chat_id: &str,
    action: &str,
) -> Result<(), ApiError> {
    let service = telegram_provider_runtime_service(state)?;
    let chat = service.telegram_chat_by_id(telegram_chat_id).await?;
    let event = build_chat_updated_event(request, telegram_chat_id, action, chat.as_ref());
    publish_telegram_event(state, event).await
}

pub(crate) async fn post_telegram_chat_pin(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.pin").await?;
    let metadata = telegram_provider_runtime_service(&state)?
        .set_chat_metadata_bool(&telegram_chat_id, "is_pinned", true)
        .await?;
    let _command_id =
        record_dialog_command(&state, &telegram_chat_id, &request, "pin", "provider_write").await?;
    publish_chat_flag_event(
        &state,
        &request,
        &telegram_chat_id,
        telegram_event_types::CHAT_PINNED,
        "is_pinned",
        true,
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "pin".to_owned(),
        status: "pinned".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_unpin(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.pin").await?;
    let metadata = telegram_provider_runtime_service(&state)?
        .set_chat_metadata_bool(&telegram_chat_id, "is_pinned", false)
        .await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "unpin",
        "provider_write",
    )
    .await?;
    publish_chat_flag_event(
        &state,
        &request,
        &telegram_chat_id,
        telegram_event_types::CHAT_PINNED,
        "is_pinned",
        false,
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "unpin".to_owned(),
        status: "unpinned".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_archive(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.archive")
        .await?;
    let metadata = telegram_provider_runtime_service(&state)?
        .set_chat_metadata_bool(&telegram_chat_id, "is_archived", true)
        .await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "archive",
        "provider_write",
    )
    .await?;
    publish_chat_flag_event(
        &state,
        &request,
        &telegram_chat_id,
        telegram_event_types::CHAT_ARCHIVED,
        "is_archived",
        true,
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "archive".to_owned(),
        status: "archived".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_unarchive(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.archive")
        .await?;
    let metadata = telegram_provider_runtime_service(&state)?
        .set_chat_metadata_bool(&telegram_chat_id, "is_archived", false)
        .await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "unarchive",
        "provider_write",
    )
    .await?;
    publish_chat_flag_event(
        &state,
        &request,
        &telegram_chat_id,
        telegram_event_types::CHAT_ARCHIVED,
        "is_archived",
        false,
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "unarchive".to_owned(),
        status: "unarchived".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_mute(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.mute").await?;
    let metadata = telegram_provider_runtime_service(&state)?
        .set_chat_metadata_bool(&telegram_chat_id, "is_muted", true)
        .await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "mute",
        "provider_write",
    )
    .await?;
    publish_chat_flag_event(
        &state,
        &request,
        &telegram_chat_id,
        telegram_event_types::CHAT_MUTED,
        "is_muted",
        true,
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "mute".to_owned(),
        status: "muted".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_unmute(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.mute").await?;
    let metadata = telegram_provider_runtime_service(&state)?
        .set_chat_metadata_bool(&telegram_chat_id, "is_muted", false)
        .await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "unmute",
        "provider_write",
    )
    .await?;
    publish_chat_flag_event(
        &state,
        &request,
        &telegram_chat_id,
        telegram_event_types::CHAT_MUTED,
        "is_muted",
        false,
    )
    .await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "unmute".to_owned(),
        status: "unmuted".to_owned(),
        metadata,
    }))
}

pub(crate) async fn post_telegram_chat_mark_read(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    let service = telegram_provider_runtime_service(&state)?;
    let chat = service
        .telegram_chat_by_id(&telegram_chat_id)
        .await?
        .ok_or(ApiError::InvalidCommunicationQuery(
            "Telegram chat was not found",
        ))?;
    if chat.account_id != request.account_id.trim()
        || chat.provider_chat_id != request.provider_chat_id.trim()
    {
        return Err(ApiError::InvalidCommunicationQuery(
            "Telegram chat does not belong to the supplied account and provider chat",
        ));
    }
    let report_to_provider = telegram_read_receipt_reports_enabled(&state, &chat).await?;
    if report_to_provider {
        ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.mark_read")
            .await?;
    }
    service
        .set_chat_last_read_at(&telegram_chat_id, Some(Utc::now()))
        .await?;
    let metadata = service
        .recompute_chat_unread_count(&telegram_chat_id)
        .await?;
    if report_to_provider {
        let _command_id = record_dialog_command(
            &state,
            &telegram_chat_id,
            &request,
            "mark_read",
            "provider_write",
        )
        .await?;
    }
    let action = if report_to_provider {
        "mark_read"
    } else {
        "mark_read_local_only"
    };
    publish_chat_updated_event(&state, &request, &telegram_chat_id, action).await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: action.to_owned(),
        status: if report_to_provider {
            "read".to_owned()
        } else {
            "read_local_only".to_owned()
        },
        metadata,
    }))
}

async fn telegram_read_receipt_reports_enabled(
    state: &AppState,
    chat: &TelegramChat,
) -> Result<bool, ApiError> {
    if let Some(enabled) = chat
        .metadata
        .get("read_receipt_reports_enabled")
        .and_then(serde_json::Value::as_bool)
    {
        return Ok(enabled);
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(ApplicationSettingsStore::new(pool)
        .setting(TELEGRAM_READ_RECEIPT_REPORTS_ENABLED_SETTING_KEY)
        .await?
        .and_then(|setting| setting.value.as_bool())
        .unwrap_or(true))
}

pub(crate) async fn post_telegram_chat_mark_unread(
    State(state): State<AppState>,
    Path(telegram_chat_id): Path<String>,
    Json(request): Json<TelegramChatActionRequest>,
) -> Result<Json<TelegramChatActionResponse>, ApiError> {
    ensure_telegram_account_operation_allowed(&state, &request.account_id, "dialogs.mark_unread")
        .await?;
    let service = telegram_provider_runtime_service(&state)?;
    service
        .set_chat_last_read_at(&telegram_chat_id, None)
        .await?;
    let metadata = service
        .recompute_chat_unread_count(&telegram_chat_id)
        .await?;
    let _command_id = record_dialog_command(
        &state,
        &telegram_chat_id,
        &request,
        "mark_unread",
        "provider_write",
    )
    .await?;
    publish_chat_updated_event(&state, &request, &telegram_chat_id, "mark_unread").await?;

    Ok(Json(TelegramChatActionResponse {
        telegram_chat_id,
        action: "mark_unread".to_owned(),
        status: "unread".to_owned(),
        metadata,
    }))
}
