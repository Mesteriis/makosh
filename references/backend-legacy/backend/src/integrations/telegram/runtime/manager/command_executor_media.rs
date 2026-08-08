use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::platform::events::bus::InMemoryEventBus;
use makosh_events_postgres::store::EventStore;

pub(super) async fn emit_media_upload_event(
    event_bus: &InMemoryEventBus,
    pool: &PgPool,
    command: &TelegramProviderWriteCommand,
    event_type: &str,
    extra_payload: Value,
) {
    let now = Utc::now();
    let mut payload = json!({
        "command_id": command.command_id,
        "account_id": command.account_id,
        "provider_chat_id": command.provider_chat_id,
        "attachment_id": payload_optional_string(command, "attachment_id"),
        "blob_id": payload_optional_string(command, "blob_id"),
        "media_type": payload_optional_string(command, "media_type"),
        "caption_present": payload_optional_string(command, "caption").is_some(),
    });
    if let (Some(payload_obj), Some(extra_obj)) =
        (payload.as_object_mut(), extra_payload.as_object())
    {
        for (key, value) in extra_obj {
            payload_obj.insert(key.clone(), value.clone());
        }
    }
    let event = NewEventEnvelope::builder(
        format!("evt_{}", now.timestamp_nanos_opt().unwrap_or(0)),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": command.account_id}),
        json!({"id": command.command_id, "kind": "telegram_media_upload"}),
    )
    .payload(payload)
    .build();

    let Ok(event) = event else {
        return;
    };

    let event_store = EventStore::new(pool.clone());
    if let Err(error) = event_store.append(&event).await {
        tracing::warn!(error = %error, "command executor: failed to append media upload event");
    }

    let _ = event_bus.broadcast(event);
}

pub(super) fn media_upload_progress_payload(
    command: &TelegramProviderWriteCommand,
    phase: &str,
    detail: &str,
) -> Value {
    let mut provider_state = command.provider_state.clone();
    if let Some(provider_state_obj) = provider_state.as_object_mut() {
        provider_state_obj.insert("upload_phase".to_owned(), Value::String(phase.to_owned()));
        provider_state_obj.insert(
            "progress_detail".to_owned(),
            Value::String(detail.to_owned()),
        );
    }
    json!({
        "status": command.status,
        "retry_count": command.retry_count,
        "max_retries": command.max_retries,
        "last_error": command.last_error,
        "next_attempt_at": command.next_attempt_at,
        "last_attempt_at": command.last_attempt_at,
        "provider_observed_at": command.provider_observed_at,
        "provider_state": provider_state,
        "reconciliation_status": command.reconciliation_status,
        "reconciled_at": command.reconciled_at,
        "dead_lettered_at": command.dead_lettered_at,
        "completed_at": command.completed_at,
        "progress_phase": phase,
        "progress_detail": detail,
    })
}

fn payload_optional_string(command: &TelegramProviderWriteCommand, key: &str) -> Option<String> {
    command
        .payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::media_upload_progress_payload;
    use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;

    fn sample_command() -> TelegramProviderWriteCommand {
        TelegramProviderWriteCommand {
            command_id: "cmd-1".to_owned(),
            account_id: "account-1".to_owned(),
            command_kind: "send_media".to_owned(),
            idempotency_key: "idem-1".to_owned(),
            provider_chat_id: "chat-1".to_owned(),
            provider_message_id: None,
            target_ref: json!({}),
            payload: json!({"attachment_id": "att-1", "blob_id": "blob-1"}),
            capability_state: "available".to_owned(),
            action_class: "provider_write".to_owned(),
            confirmation_decision: "confirmed".to_owned(),
            status: "executing".to_owned(),
            retry_count: 1,
            max_retries: 3,
            last_error: None,
            result_payload: json!({}),
            audit_metadata: json!({}),
            actor_id: "makosh-frontend".to_owned(),
            happened_at: Utc::now(),
            next_attempt_at: None,
            last_attempt_at: None,
            locked_at: None,
            locked_by: None,
            provider_observed_at: None,
            provider_state: json!({"dispatch": "claimed"}),
            reconciliation_status: "not_observed".to_owned(),
            reconciled_at: None,
            dead_lettered_at: None,
            completed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn media_upload_progress_payload_carries_phase_detail_in_provider_state() {
        let payload = media_upload_progress_payload(
            &sample_command(),
            "dispatching_to_provider",
            "Uploading local media to Telegram",
        );

        assert_eq!(payload["status"], "executing");
        assert_eq!(payload["progress_phase"], "dispatching_to_provider");
        assert_eq!(
            payload["progress_detail"],
            "Uploading local media to Telegram"
        );
        assert_eq!(payload["provider_state"]["dispatch"], "claimed");
        assert_eq!(
            payload["provider_state"]["upload_phase"],
            "dispatching_to_provider"
        );
        assert_eq!(
            payload["provider_state"]["progress_detail"],
            "Uploading local media to Telegram"
        );
    }
}
