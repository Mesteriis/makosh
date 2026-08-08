use chrono::{DateTime, Utc};
use makosh_events_api::NewEventEnvelope;
use serde_json::json;

use crate::integrations::telegram::client::models::chats::TelegramChat;
use crate::integrations::telegram::tdjson::parsing::events::{
    TelegramTdlibChatMarkedAsUnreadSnapshot, TelegramTdlibChatNotificationSettingsSnapshot,
    TelegramTdlibChatPositionSnapshot, TelegramTdlibChatUnreadSnapshot,
};

use crate::platform::events::bus::telegram_event_types;

pub(super) fn chat_unread_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatUnreadSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, makosh_events_api::EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_unread_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "action": "provider_unread_update",
        "unread_count": snapshot.unread_count,
        "unread_mention_count": snapshot.unread_mention_count,
        "last_read_inbox_provider_message_id": snapshot.last_read_inbox_message_id,
        "chat": chat,
        "source": format!("tdlib.{}", snapshot.source_event)
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": snapshot.source_event
    }))
    .build()
}

pub(super) fn chat_marked_as_unread_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatMarkedAsUnreadSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, makosh_events_api::EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_marked_unread_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "action": "provider_marked_as_unread_update",
        "is_marked_as_unread": snapshot.is_marked_as_unread,
        "chat": chat,
        "source": format!("tdlib.{}", snapshot.source_event)
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": snapshot.source_event
    }))
    .build()
}

pub(super) fn chat_notification_settings_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatNotificationSettingsSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, makosh_events_api::EventEnvelopeError> {
    let is_muted = !snapshot.use_default_mute_for && snapshot.mute_for > 0;
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_notification_settings_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_MUTED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "is_muted": is_muted,
        "use_default_mute_for": snapshot.use_default_mute_for,
        "mute_for": snapshot.mute_for,
        "chat": chat,
        "source": format!("tdlib.{}", snapshot.source_event)
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": snapshot.source_event
    }))
    .build()
}

pub(super) fn chat_notification_settings_chat_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatNotificationSettingsSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, makosh_events_api::EventEnvelopeError> {
    let is_muted = !snapshot.use_default_mute_for && snapshot.mute_for > 0;
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_updated_notification_settings_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "action": "provider_notification_settings_update",
        "is_muted": is_muted,
        "use_default_mute_for": snapshot.use_default_mute_for,
        "mute_for": snapshot.mute_for,
        "chat": chat,
        "source": format!("tdlib.{}", snapshot.source_event)
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": snapshot.source_event
    }))
    .build()
}

pub(super) fn chat_archived_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatPositionSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, makosh_events_api::EventEnvelopeError> {
    let is_archived = chat
        .metadata
        .get("is_archived")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_archive_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_ARCHIVED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "is_archived": is_archived,
        "list_kind": snapshot.list_kind,
        "provider_folder_id": snapshot.provider_folder_id,
        "order": snapshot.order,
        "is_pinned": snapshot.is_pinned,
        "chat": chat,
        "source": format!("tdlib.{}", snapshot.source_event)
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": snapshot.source_event
    }))
    .build()
}

pub(super) fn chat_position_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatPositionSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, makosh_events_api::EventEnvelopeError> {
    let is_archived = chat
        .metadata
        .get("is_archived")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_updated_position_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "action": "provider_chat_position_update",
        "is_archived": is_archived,
        "list_kind": snapshot.list_kind,
        "provider_folder_id": snapshot.provider_folder_id,
        "order": snapshot.order,
        "is_pinned": snapshot.is_pinned,
        "chat": chat,
        "source": format!("tdlib.{}", snapshot.source_event)
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": snapshot.source_event
    }))
    .build()
}

pub(super) fn chat_folder_labels_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, makosh_events_api::EventEnvelopeError> {
    let folder_labels = chat
        .metadata
        .get("folder_labels")
        .cloned()
        .unwrap_or_else(|| json!([]));
    let provider_folder_id = chat.metadata.get("provider_folder_id").cloned();

    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_folder_labels_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_UPDATED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "action": "provider_chat_folder_labels_update",
        "folder_labels": folder_labels,
        "provider_folder_id": provider_folder_id,
        "chat": chat,
        "source": "tdlib.updateChatFolders"
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": "updateChatFolders"
    }))
    .build()
}

pub(super) fn chat_pinned_updated_event(
    account_id: &str,
    chat: &TelegramChat,
    snapshot: &TelegramTdlibChatPositionSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, makosh_events_api::EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_chat_pinned_{}_{}_{}",
            account_id,
            chat.telegram_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::CHAT_PINNED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "id": chat.telegram_chat_id,
            "provider_chat_id": chat.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "telegram_chat_id": chat.telegram_chat_id,
        "provider_chat_id": chat.provider_chat_id,
        "is_pinned": snapshot.is_pinned,
        "list_kind": snapshot.list_kind,
        "provider_folder_id": snapshot.provider_folder_id,
        "order": snapshot.order,
        "chat": chat,
        "source": format!("tdlib.{}", snapshot.source_event)
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": snapshot.source_event
    }))
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_unread_updated_event_contains_sanitized_projected_chat_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let chat = TelegramChat {
            telegram_chat_id: "telegram_chat:v1:test".to_owned(),
            account_id: "acct-1".to_owned(),
            provider_chat_id: "-100123".to_owned(),
            chat_kind: "group".to_owned(),
            title: "Release Chat".to_owned(),
            username: None,
            sync_state: "synced".to_owned(),
            last_message_at: None,
            metadata: json!({
                "unread_count": 3,
                "mention_count": 1,
                "provider_unread_count": 3,
                "provider_unread_mention_count": 1
            }),
            created_at: occurred_at,
            updated_at: occurred_at,
        };
        let snapshot = TelegramTdlibChatUnreadSnapshot {
            provider_chat_id: "-100123".to_owned(),
            unread_count: Some(3),
            unread_mention_count: Some(1),
            last_read_inbox_message_id: Some("777".to_owned()),
            source_event: "updateChatReadInbox".to_owned(),
        };

        let event =
            chat_unread_updated_event("acct-1", &chat, &snapshot, occurred_at).expect("event");

        assert_eq!(event.event_type, telegram_event_types::CHAT_UPDATED);
        assert_eq!(event.payload["action"], "provider_unread_update");
        assert_eq!(event.payload["chat"]["metadata"]["unread_count"], 3);
    }

    #[test]
    fn chat_marked_as_unread_updated_event_contains_provider_state_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let chat = TelegramChat {
            telegram_chat_id: "telegram_chat:v1:test".to_owned(),
            account_id: "acct-1".to_owned(),
            provider_chat_id: "-100123".to_owned(),
            chat_kind: "group".to_owned(),
            title: "Release Chat".to_owned(),
            username: None,
            sync_state: "synced".to_owned(),
            last_message_at: None,
            metadata: json!({
                "is_marked_as_unread": true
            }),
            created_at: occurred_at,
            updated_at: occurred_at,
        };
        let snapshot = TelegramTdlibChatMarkedAsUnreadSnapshot {
            provider_chat_id: "-100123".to_owned(),
            is_marked_as_unread: true,
            source_event: "updateChatIsMarkedAsUnread".to_owned(),
        };

        let event = chat_marked_as_unread_updated_event("acct-1", &chat, &snapshot, occurred_at)
            .expect("event");

        assert_eq!(event.payload["action"], "provider_marked_as_unread_update");
        assert_eq!(event.payload["is_marked_as_unread"], true);
    }

    #[test]
    fn chat_notification_settings_updated_event_contains_provider_state_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let chat = TelegramChat {
            telegram_chat_id: "telegram_chat:v1:test".to_owned(),
            account_id: "acct-1".to_owned(),
            provider_chat_id: "-100123".to_owned(),
            chat_kind: "group".to_owned(),
            title: "Release Chat".to_owned(),
            username: None,
            sync_state: "synced".to_owned(),
            last_message_at: None,
            metadata: json!({
                "is_muted": true
            }),
            created_at: occurred_at,
            updated_at: occurred_at,
        };
        let snapshot = TelegramTdlibChatNotificationSettingsSnapshot {
            provider_chat_id: "-100123".to_owned(),
            use_default_mute_for: false,
            mute_for: 31_708_800,
            source_event: "updateChatNotificationSettings".to_owned(),
        };

        let event =
            chat_notification_settings_updated_event("acct-1", &chat, &snapshot, occurred_at)
                .expect("event");

        assert_eq!(event.event_type, telegram_event_types::CHAT_MUTED);
        assert_eq!(event.payload["is_muted"], true);
        assert_eq!(event.payload["mute_for"], 31_708_800);
    }

    #[test]
    fn chat_notification_settings_chat_updated_event_contains_snapshot_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let chat = TelegramChat {
            telegram_chat_id: "telegram_chat:v1:test".to_owned(),
            account_id: "acct-1".to_owned(),
            provider_chat_id: "-100123".to_owned(),
            chat_kind: "group".to_owned(),
            title: "Release Chat".to_owned(),
            username: None,
            sync_state: "synced".to_owned(),
            last_message_at: None,
            metadata: json!({
                "is_muted": true
            }),
            created_at: occurred_at,
            updated_at: occurred_at,
        };
        let snapshot = TelegramTdlibChatNotificationSettingsSnapshot {
            provider_chat_id: "-100123".to_owned(),
            use_default_mute_for: false,
            mute_for: 31_708_800,
            source_event: "updateChatNotificationSettings".to_owned(),
        };

        let event =
            chat_notification_settings_chat_updated_event("acct-1", &chat, &snapshot, occurred_at)
                .expect("event");

        assert_eq!(event.event_type, telegram_event_types::CHAT_UPDATED);
        assert_eq!(
            event.payload["action"],
            "provider_notification_settings_update"
        );
        assert_eq!(event.payload["is_muted"], true);
        assert_eq!(event.payload["chat"]["metadata"]["is_muted"], true);
    }

    #[test]
    fn chat_archived_updated_event_contains_provider_state_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let chat = TelegramChat {
            telegram_chat_id: "telegram_chat:v1:test".to_owned(),
            account_id: "acct-1".to_owned(),
            provider_chat_id: "-100123".to_owned(),
            chat_kind: "group".to_owned(),
            title: "Release Chat".to_owned(),
            username: None,
            sync_state: "synced".to_owned(),
            last_message_at: None,
            metadata: json!({
                "is_archived": true
            }),
            created_at: occurred_at,
            updated_at: occurred_at,
        };
        let snapshot = TelegramTdlibChatPositionSnapshot {
            provider_chat_id: "-100123".to_owned(),
            list_kind: "archive".to_owned(),
            provider_folder_id: None,
            order: 42,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
        };

        let event =
            chat_archived_updated_event("acct-1", &chat, &snapshot, occurred_at).expect("event");

        assert_eq!(event.event_type, telegram_event_types::CHAT_ARCHIVED);
        assert_eq!(event.payload["is_archived"], true);
        assert_eq!(event.payload["list_kind"], "archive");
    }

    #[test]
    fn chat_position_updated_event_contains_snapshot_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let chat = TelegramChat {
            telegram_chat_id: "telegram_chat:v1:test".to_owned(),
            account_id: "acct-1".to_owned(),
            provider_chat_id: "-100123".to_owned(),
            chat_kind: "group".to_owned(),
            title: "Release Chat".to_owned(),
            username: None,
            sync_state: "synced".to_owned(),
            last_message_at: None,
            metadata: json!({
                "is_archived": true,
                "is_pinned": true
            }),
            created_at: occurred_at,
            updated_at: occurred_at,
        };
        let snapshot = TelegramTdlibChatPositionSnapshot {
            provider_chat_id: "-100123".to_owned(),
            list_kind: "archive".to_owned(),
            provider_folder_id: Some(7),
            order: 42,
            is_pinned: true,
            source_event: "updateChatPosition".to_owned(),
        };

        let event =
            chat_position_updated_event("acct-1", &chat, &snapshot, occurred_at).expect("event");

        assert_eq!(event.event_type, telegram_event_types::CHAT_UPDATED);
        assert_eq!(event.payload["action"], "provider_chat_position_update");
        assert_eq!(event.payload["is_archived"], true);
        assert_eq!(event.payload["is_pinned"], true);
        assert_eq!(event.payload["provider_folder_id"], 7);
        assert_eq!(event.payload["chat"]["metadata"]["is_archived"], true);
    }

    #[test]
    fn chat_pinned_updated_event_contains_provider_state_payload() {
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");
        let chat = TelegramChat {
            telegram_chat_id: "telegram_chat:v1:test".to_owned(),
            account_id: "acct-1".to_owned(),
            provider_chat_id: "-100123".to_owned(),
            chat_kind: "group".to_owned(),
            title: "Release Chat".to_owned(),
            username: None,
            sync_state: "synced".to_owned(),
            last_message_at: None,
            metadata: json!({
                "is_pinned": true
            }),
            created_at: occurred_at,
            updated_at: occurred_at,
        };
        let snapshot = TelegramTdlibChatPositionSnapshot {
            provider_chat_id: "-100123".to_owned(),
            list_kind: "main".to_owned(),
            provider_folder_id: None,
            order: 42,
            is_pinned: true,
            source_event: "updateChatPosition".to_owned(),
        };

        let event =
            chat_pinned_updated_event("acct-1", &chat, &snapshot, occurred_at).expect("event");

        assert_eq!(event.event_type, telegram_event_types::CHAT_PINNED);
        assert_eq!(event.payload["is_pinned"], true);
        assert_eq!(event.payload["list_kind"], "main");
    }
}
