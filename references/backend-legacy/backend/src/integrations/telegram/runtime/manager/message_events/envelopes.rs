use chrono::{DateTime, Utc};
use makosh_events_api::{EventEnvelopeError, NewEventEnvelope};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::integrations::telegram::client::models::messages::TelegramMessage;
use crate::integrations::telegram::client::models::messages::TelegramMessageTombstone;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::telegram_event_types;
use makosh_events_postgres::store::EventStore;

pub(super) async fn append_and_broadcast(
    pool: Option<PgPool>,
    event_bus: &InMemoryEventBus,
    event: NewEventEnvelope,
) {
    if let Some(pool) = pool {
        let event_store = EventStore::new(pool);
        if let Err(error) = event_store.append(&event).await {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append message event");
        }
    }
    let _ = event_bus.broadcast(event);
}

pub(super) fn message_created_event(
    account_id: &str,
    message: &TelegramMessage,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_message_created_{}_{}_{}",
            account_id,
            message.message_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::MESSAGE_CREATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_message",
            "id": message.message_id,
            "provider_chat_id": message.provider_chat_id,
            "provider_message_id": message.provider_message_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "message_id": message.message_id,
        "provider_chat_id": message.provider_chat_id,
        "provider_message_id": message.provider_message_id,
        "message": message,
        "source": "tdlib.updateNewMessage"
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": "updateNewMessage"
    }))
    .build()
}

pub(super) fn message_deleted_event(
    account_id: &str,
    message: &TelegramMessage,
    tombstone: &TelegramMessageTombstone,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_message_deleted_{}_{}_{}",
            account_id,
            message.message_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::MESSAGE_DELETED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_message",
            "id": message.message_id,
            "provider_chat_id": message.provider_chat_id,
            "provider_message_id": message.provider_message_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "message_id": message.message_id,
        "provider_chat_id": message.provider_chat_id,
        "provider_message_id": message.provider_message_id,
        "message": message,
        "tombstone": tombstone,
        "source": "tdlib.updateDeleteMessages"
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": "updateDeleteMessages"
    }))
    .build()
}

pub(super) fn message_updated_event(
    account_id: &str,
    message: &TelegramMessage,
    extra_payload: Value,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, EventEnvelopeError> {
    let mut payload = json!({
        "account_id": account_id,
        "message_id": message.message_id,
        "provider_chat_id": message.provider_chat_id,
        "provider_message_id": message.provider_message_id,
        "message": message,
    });
    if let (Some(payload_obj), Some(extra_obj)) =
        (payload.as_object_mut(), extra_payload.as_object())
    {
        for (key, value) in extra_obj {
            payload_obj.insert(key.clone(), value.clone());
        }
    }

    NewEventEnvelope::builder(
        format!(
            "evt_telegram_message_updated_{}_{}_{}",
            account_id,
            message.message_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::MESSAGE_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_message",
            "id": message.message_id,
            "provider_chat_id": message.provider_chat_id,
            "provider_message_id": message.provider_message_id
        }),
    )
    .payload(payload)
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
    }))
    .build()
}

pub(super) fn reaction_changed_event(
    account_id: &str,
    message: &TelegramMessage,
    reaction_summary: Option<Value>,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_reaction_changed_{}_{}_{}",
            account_id,
            message.message_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::REACTION_CHANGED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_message",
            "id": message.message_id,
            "provider_chat_id": message.provider_chat_id,
            "provider_message_id": message.provider_message_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "message_id": message.message_id,
        "provider_chat_id": message.provider_chat_id,
        "provider_message_id": message.provider_message_id,
        "message": message,
        "reaction_summary": reaction_summary,
        "source": "tdlib.updateMessageInteractionInfo"
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": "updateMessageInteractionInfo"
    }))
    .build()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn sample_message(occurred_at: DateTime<Utc>) -> TelegramMessage {
        TelegramMessage {
            message_id: "message:v4:telegram:test".to_owned(),
            raw_record_id: "raw:v4:telegram:test".to_owned(),
            account_id: "acct-1".to_owned(),
            provider_message_id: "-100123:42".to_owned(),
            provider_chat_id: Some("-100123".to_owned()),
            chat_title: "Chat".to_owned(),
            sender: "Telegram User 777".to_owned(),
            sender_display_name: Some("Alice".to_owned()),
            text: "hello".to_owned(),
            occurred_at: Some(occurred_at),
            projected_at: occurred_at,
            channel_kind: "telegram_user".to_owned(),
            delivery_state: "received".to_owned(),
            metadata: json!({}),
        }
    }

    #[test]
    fn message_deleted_event_contains_tombstone_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 13, 0, 0)
            .single()
            .expect("valid timestamp");
        let message = sample_message(occurred_at);
        let tombstone = TelegramMessageTombstone {
            tombstone_id: "tomb-1".to_owned(),
            message_id: message.message_id.clone(),
            account_id: "acct-1".to_owned(),
            provider_message_id: "-100123:42".to_owned(),
            provider_chat_id: "-100123".to_owned(),
            reason_class: "deleted_by_provider".to_owned(),
            actor_class: "provider".to_owned(),
            observed_at: occurred_at,
            source_event: Some("updateDeleteMessages".to_owned()),
            is_provider_delete: true,
            is_local_visible: false,
            metadata: json!({"from_cache": false}),
            provenance: json!({"provider": "telegram"}),
            created_at: occurred_at,
        };

        let event = message_deleted_event("acct-1", &message, &tombstone, occurred_at)
            .expect("message deleted event");

        assert_eq!(event.event_type, telegram_event_types::MESSAGE_DELETED);
        assert_eq!(
            event.payload["tombstone"]["reason_class"],
            "deleted_by_provider"
        );
        assert_eq!(event.payload["provider_message_id"], "-100123:42");
    }

    #[test]
    fn reaction_changed_event_contains_summary_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 13, 0, 0)
            .single()
            .expect("valid timestamp");
        let message = sample_message(occurred_at);

        let event = reaction_changed_event(
            "acct-1",
            &message,
            Some(json!({"total_reactions": 2})),
            occurred_at,
        )
        .expect("reaction changed event");

        assert_eq!(event.event_type, telegram_event_types::REACTION_CHANGED);
        assert_eq!(event.payload["reaction_summary"]["total_reactions"], 2);
        assert_eq!(
            event.payload["message"]["message_id"],
            "message:v4:telegram:test"
        );
    }

    #[test]
    fn message_updated_event_contains_extra_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 13, 0, 0)
            .single()
            .expect("valid timestamp");
        let message = sample_message(occurred_at);

        let event = message_updated_event(
            "acct-1",
            &message,
            json!({"text_changed": true, "source": "tdlib.updateMessageContent"}),
            occurred_at,
        )
        .expect("message updated event");

        assert_eq!(event.event_type, telegram_event_types::MESSAGE_UPDATED);
        assert_eq!(event.payload["text_changed"], true);
        assert_eq!(event.payload["source"], "tdlib.updateMessageContent");
    }
}
