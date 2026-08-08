use chrono::{DateTime, Utc};
use makosh_events_api::NewEventEnvelope;
use serde_json::json;

use crate::integrations::telegram::client::models::chats::TelegramChatMember;
use crate::platform::events::bus::telegram_event_types;
use makosh_events_postgres::store::EventStore;

use super::realtime_events::TelegramRuntimeEventBridgeContext;

pub(super) async fn publish_participant_updated_event(
    context: Option<&TelegramRuntimeEventBridgeContext>,
    account_id: &str,
    telegram_chat_id: &str,
    provider_chat_id: &str,
    participant: &TelegramChatMember,
    source: &str,
) {
    let Some(context) = context else {
        return;
    };

    let occurred_at = Utc::now();
    let Ok(event) = participant_updated_event(
        account_id,
        telegram_chat_id,
        provider_chat_id,
        participant,
        source,
        occurred_at,
    ) else {
        return;
    };

    if let Some(store) = &context.telegram_store {
        let pool = store.pool();
        let event_store = EventStore::new(pool.clone());
        if let Err(error) = event_store.append(&event).await {
            tracing::warn!(
                error = %error,
                "Telegram runtime event bridge: failed to append participant update event"
            );
        }
    }

    let _ = context.event_bus.broadcast(event);
}

fn participant_updated_event(
    account_id: &str,
    telegram_chat_id: &str,
    provider_chat_id: &str,
    participant: &TelegramChatMember,
    source: &str,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, makosh_events_api::EventEnvelopeError> {
    let tdlib_event = participant_event_source(source);
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_participant_{}_{}_{}",
            telegram_chat_id,
            participant.provider_member_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::PARTICIPANT_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat_participant",
            "telegram_chat_id": telegram_chat_id,
            "provider_chat_id": provider_chat_id,
            "provider_member_id": participant.provider_member_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": telegram_chat_id,
        "provider_chat_id": provider_chat_id,
        "participant": participant,
        "source": source
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": tdlib_event
    }))
    .build()
}

fn participant_event_source(source: &str) -> &str {
    source.strip_prefix("tdlib.").unwrap_or(source)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use serde_json::json;

    use super::*;

    #[test]
    fn participant_updated_event_contains_projection_member_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let participant = TelegramChatMember {
            sender_id: "user:42".to_owned(),
            sender_display_name: Some("Owner User".to_owned()),
            message_count: 0,
            last_message_at: None,
            source: "tdlib".to_owned(),
            provider_member_id: "user:42".to_owned(),
            username: Some("owner".to_owned()),
            role: Some("owner".to_owned()),
            status: Some("creator".to_owned()),
            is_admin: true,
            is_owner: true,
            permissions: json!({"can_invite_users": true}),
            observed_at: Some(occurred_at),
        };

        let event = participant_updated_event(
            "acct-1",
            "telegram_chat:v1:test",
            "-100123",
            &participant,
            "tdlib.getSupergroupMembers",
            occurred_at,
        )
        .expect("event");

        assert_eq!(event.event_type, telegram_event_types::PARTICIPANT_UPDATED);
        assert_eq!(event.subject["kind"], "telegram_chat_participant");
        assert_eq!(event.subject["provider_member_id"], "user:42");
        assert_eq!(event.payload["telegram_chat_id"], "telegram_chat:v1:test");
        assert_eq!(event.payload["participant"]["role"], "owner");
        assert_eq!(
            event.payload["participant"]["permissions"]["can_invite_users"],
            true
        );
        assert_eq!(event.payload["source"], "tdlib.getSupergroupMembers");
        assert_eq!(event.provenance["tdlib_event"], "getSupergroupMembers");
    }

    #[test]
    fn participant_updated_event_uses_runtime_source_as_provenance() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let participant = TelegramChatMember {
            sender_id: "user:42".to_owned(),
            sender_display_name: Some("Owner User".to_owned()),
            message_count: 0,
            last_message_at: None,
            source: "tdlib".to_owned(),
            provider_member_id: "user:42".to_owned(),
            username: Some("owner".to_owned()),
            role: Some("member".to_owned()),
            status: Some("absent_exhaustive".to_owned()),
            is_admin: false,
            is_owner: false,
            permissions: json!({
                "membership_state": "absent_exhaustive"
            }),
            observed_at: Some(occurred_at),
        };

        let event = participant_updated_event(
            "acct-1",
            "telegram_chat:v1:test",
            "-100123",
            &participant,
            "tdlib.getSupergroupMembers.exhaustive_absence",
            occurred_at,
        )
        .expect("event");

        assert_eq!(
            event.provenance["tdlib_event"],
            "getSupergroupMembers.exhaustive_absence"
        );
        assert_eq!(
            event.payload["source"],
            "tdlib.getSupergroupMembers.exhaustive_absence"
        );
    }
}
