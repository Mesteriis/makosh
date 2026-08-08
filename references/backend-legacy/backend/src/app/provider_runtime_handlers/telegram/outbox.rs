use axum::Json;
use axum::extract::{Path, State};
use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;

use super::helpers::publish_telegram_event;
use crate::app::api_support::stores::integration_stores::telegram_provider_runtime_service;
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;

use crate::platform::events::bus::telegram_event_types;

pub(crate) async fn post_telegram_command_retry(
    State(state): State<AppState>,
    Path(command_id): Path<String>,
) -> Result<Json<TelegramProviderWriteCommand>, ApiError> {
    let now = Utc::now();
    let command = telegram_provider_runtime_service(&state)?
        .manual_retry_command(&command_id, now)
        .await?
        .ok_or(ApiError::NotFound)?;

    let event = NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        telegram_event_types::COMMAND_STATUS_CHANGED.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": command.account_id}),
        json!({"id": command.command_id, "kind": "telegram_command"}),
    )
    .payload(json!({
        "command_id": command.command_id,
        "account_id": command.account_id,
        "provider_chat_id": command.provider_chat_id,
        "message_id": command.provider_message_id,
        "status": command.status,
        "retry_count": command.retry_count,
        "max_retries": command.max_retries,
        "last_error": command.last_error,
        "result_payload": command.result_payload,
        "source": "manual_retry",
        "next_attempt_at": command.next_attempt_at,
        "last_attempt_at": command.last_attempt_at,
        "provider_observed_at": command.provider_observed_at,
        "provider_state": command.provider_state,
        "reconciliation_status": command.reconciliation_status,
        "reconciled_at": command.reconciled_at,
        "dead_lettered_at": command.dead_lettered_at,
        "completed_at": command.completed_at,
        "payload": {
            "source": "manual_retry",
            "next_attempt_at": command.next_attempt_at,
        },
    }))
    .build()
    .expect("telegram command retry event must be valid");
    publish_telegram_event(&state, event).await?;

    Ok(Json(command))
}
