use chrono::{DateTime, Utc};
use makosh_events_api::NewEventEnvelope;
use serde_json::json;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::client::store::TelegramStore;
use crate::integrations::telegram::tdjson::parsing::events::TelegramTdlibTypingSnapshot;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::telegram_event_types;
use makosh_events_postgres::store::EventStore;

use super::super::state::TelegramRuntimeEvent;
use super::chat_events::{
    publish_chat_folders_event, publish_chat_marked_as_unread_event,
    publish_chat_notification_settings_event, publish_chat_position_event,
    publish_chat_removed_from_list_event, publish_chat_unread_event,
};
use super::message_events::{
    publish_message_content_updated_event, publish_message_created_event,
    publish_message_deleted_event, publish_message_edited_event, publish_message_pinned_event,
    publish_message_send_failed_event, publish_message_send_succeeded_event,
    publish_reaction_changed_event,
};
use super::topic_events::publish_topic_event;

const TELEGRAM_RUNTIME_EVENT_BRIDGE_RUNTIME: &str = "telegram_runtime_event_bridge";

#[derive(Clone)]
pub struct TelegramRuntimeEventBridgeContext {
    pub(super) telegram_store: Option<TelegramStore>,
    pub(super) event_bus: InMemoryEventBus,
}

impl TelegramRuntimeEventBridgeContext {
    pub(crate) fn new(telegram_store: Option<TelegramStore>, event_bus: InMemoryEventBus) -> Self {
        Self {
            telegram_store,
            event_bus,
        }
    }
}

pub(super) fn spawn_telegram_runtime_event_bridge(
    telegram_store: Option<TelegramStore>,
    event_bus: InMemoryEventBus,
    account_id: String,
    mut runtime_events: UnboundedReceiver<TelegramRuntimeEvent>,
) {
    tokio::spawn(async move {
        while let Some(event) = runtime_events.recv().await {
            if !telegram_runtime_event_bridge_allows_processing(&telegram_store).await {
                continue;
            }
            match event {
                TelegramRuntimeEvent::MessageCreated(snapshot) => {
                    publish_message_created_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::MessageContentUpdated(snapshot) => {
                    publish_message_content_updated_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::MessageEdited(snapshot) => {
                    publish_message_edited_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::MessagePinnedUpdated(snapshot) => {
                    publish_message_pinned_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::MessageSendFailed(snapshot) => {
                    publish_message_send_failed_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::MessageSendSucceeded(snapshot) => {
                    publish_message_send_succeeded_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::MessageDeleted(snapshot) => {
                    publish_message_deleted_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::MessageInteractionInfoUpdated(snapshot) => {
                    publish_reaction_changed_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::TypingChanged(snapshot) => {
                    publish_typing_event(&telegram_store, &event_bus, &account_id, &snapshot).await;
                }
                TelegramRuntimeEvent::TopicUpdated(snapshot) => {
                    publish_topic_event(&telegram_store, &event_bus, &account_id, &snapshot).await;
                }
                TelegramRuntimeEvent::ChatUnreadUpdated(snapshot) => {
                    publish_chat_unread_event(&telegram_store, &event_bus, &account_id, &snapshot)
                        .await;
                }
                TelegramRuntimeEvent::ChatMarkedAsUnreadUpdated(snapshot) => {
                    publish_chat_marked_as_unread_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::ChatNotificationSettingsUpdated(snapshot) => {
                    publish_chat_notification_settings_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::ChatPositionUpdated(snapshot) => {
                    publish_chat_position_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::ChatRemovedFromList(snapshot) => {
                    publish_chat_removed_from_list_event(
                        &telegram_store,
                        &event_bus,
                        &account_id,
                        &snapshot,
                    )
                    .await;
                }
                TelegramRuntimeEvent::ChatFoldersUpdated(folders) => {
                    publish_chat_folders_event(&telegram_store, &event_bus, &account_id, &folders)
                        .await;
                }
            }
        }
    });
}

async fn telegram_runtime_event_bridge_allows_processing(
    telegram_store: &Option<TelegramStore>,
) -> bool {
    let Some(store) = telegram_store else {
        return true;
    };

    match crate::platform::events::runtime::runtime_allows_processing(
        store.pool(),
        "telegram",
        TELEGRAM_RUNTIME_EVENT_BRIDGE_RUNTIME,
        &json!({
            "label": "Telegram realtime event bridge",
            "scope": "subscription",
            "runtime": "tdlib",
        }),
    )
    .await
    {
        Ok(allowed) => allowed,
        Err(error) => {
            tracing::warn!(
                error = %error,
                runtime_kind = TELEGRAM_RUNTIME_EVENT_BRIDGE_RUNTIME,
                "telegram runtime event bridge gate check failed"
            );
            true
        }
    }
}

pub(super) async fn publish_command_reconciled_events(
    context: Option<&TelegramRuntimeEventBridgeContext>,
    command: &TelegramProviderWriteCommand,
    source: &str,
) {
    let Some(context) = context else {
        return;
    };
    let payload = json!({
        "source": source,
        "reconciliation_status": command.reconciliation_status,
        "provider_observed_at": command.provider_observed_at,
        "reconciled_at": command.reconciled_at,
        "provider_state": command.provider_state,
    });
    publish_command_event(
        context,
        command,
        telegram_event_types::COMMAND_STATUS_CHANGED,
        payload.clone(),
    )
    .await;
    publish_command_event(
        context,
        command,
        telegram_event_types::COMMAND_RECONCILED,
        payload,
    )
    .await;
}

pub(super) fn command_event_payload(
    command: &TelegramProviderWriteCommand,
    status: &str,
    extra_payload: serde_json::Value,
) -> serde_json::Value {
    let mut payload = json!({
        "command_id": command.command_id,
        "account_id": command.account_id,
        "command_kind": command.command_kind,
        "provider_chat_id": command.provider_chat_id,
        "message_id": command.provider_message_id,
        "status": status,
        "retry_count": command.retry_count,
        "max_retries": command.max_retries,
        "last_error": command.last_error,
        "result_payload": command.result_payload,
        "next_attempt_at": command.next_attempt_at,
        "last_attempt_at": command.last_attempt_at,
        "provider_observed_at": command.provider_observed_at,
        "provider_state": command.provider_state,
        "reconciliation_status": command.reconciliation_status,
        "reconciled_at": command.reconciled_at,
        "dead_lettered_at": command.dead_lettered_at,
        "completed_at": command.completed_at,
    });
    if let (Some(payload_obj), Some(extra_obj)) =
        (payload.as_object_mut(), extra_payload.as_object())
    {
        for (key, value) in extra_obj {
            payload_obj.insert(key.clone(), value.clone());
        }
    }
    if let Some(payload_obj) = payload.as_object_mut() {
        payload_obj.insert("payload".to_owned(), extra_payload);
    }
    payload
}

async fn publish_command_event(
    context: &TelegramRuntimeEventBridgeContext,
    command: &TelegramProviderWriteCommand,
    event_type: &str,
    extra_payload: serde_json::Value,
) {
    let now = Utc::now();
    let payload = command_event_payload(command, &command.status, extra_payload);

    let event = NewEventEnvelope::builder(
        format!(
            "evt_telegram_command_{}_{}_{}",
            event_type.replace('.', "_"),
            command.command_id,
            now.timestamp_nanos_opt().unwrap_or(0)
        ),
        event_type.to_owned(),
        now,
        json!({"channel": "telegram", "account_id": command.account_id}),
        json!({"id": command.command_id, "kind": "telegram_command"}),
    )
    .payload(payload)
    .build();

    let Ok(event) = event else {
        return;
    };

    if let Some(store) = &context.telegram_store {
        let pool = store.pool();
        let event_store = EventStore::new(pool.clone());
        if let Err(error) = event_store.append(&event).await {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append command reconciliation event");
        }
    }

    let _ = context.event_bus.broadcast(event);
}

#[cfg(test)]
mod typing_tests {
    use chrono::Utc;
    use makosh_backend_testkit::context::TestContext;
    use serde_json::json;

    use super::command_event_payload;
    use super::{TelegramRuntimeEventBridgeContext, publish_command_reconciled_events};
    use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
    use crate::platform::events::bus::InMemoryEventBus;

    fn sample_command() -> TelegramProviderWriteCommand {
        TelegramProviderWriteCommand {
            command_id: "cmd-1".to_owned(),
            account_id: "account-1".to_owned(),
            command_kind: "edit".to_owned(),
            idempotency_key: "idem-1".to_owned(),
            provider_chat_id: "chat-1".to_owned(),
            provider_message_id: Some("chat-1:42".to_owned()),
            target_ref: json!({"provider_message_id": "chat-1:42"}),
            payload: json!({"new_text": "after"}),
            capability_state: "available".to_owned(),
            action_class: "provider_write".to_owned(),
            confirmation_decision: "not_required".to_owned(),
            status: "completed".to_owned(),
            retry_count: 2,
            max_retries: 5,
            last_error: Some("temporary failure".to_owned()),
            result_payload: json!({"projection_message_id": "msg-1"}),
            audit_metadata: json!({"source": "test"}),
            actor_id: "makosh-frontend".to_owned(),
            happened_at: Utc::now(),
            next_attempt_at: None,
            last_attempt_at: Some(Utc::now()),
            locked_at: None,
            locked_by: None,
            provider_observed_at: Some(Utc::now()),
            provider_state: json!({"state": "observed"}),
            reconciliation_status: "observed".to_owned(),
            reconciled_at: Some(Utc::now()),
            dead_lettered_at: None,
            completed_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn command_event_payload_includes_durable_command_state() {
        let command = sample_command();
        let payload = command_event_payload(
            &command,
            "completed",
            json!({"source": "test", "phase": "provider_observed"}),
        );

        assert_eq!(payload["command_id"], json!("cmd-1"));
        assert_eq!(payload["command_kind"], json!("edit"));
        assert_eq!(payload["status"], json!("completed"));
        assert_eq!(payload["retry_count"], json!(2));
        assert_eq!(payload["max_retries"], json!(5));
        assert_eq!(payload["last_error"], json!("temporary failure"));
        assert_eq!(
            payload["result_payload"]["projection_message_id"],
            json!("msg-1")
        );
        assert_eq!(payload["provider_state"]["state"], json!("observed"));
        assert_eq!(payload["reconciliation_status"], json!("observed"));
        assert_eq!(payload["payload"]["source"], json!("test"));
        assert_eq!(payload["payload"]["phase"], json!("provider_observed"));
        assert!(payload["completed_at"].is_string());
    }

    #[tokio::test]
    async fn publish_command_reconciled_events_appends_status_and_reconciled_records() {
        let ctx = TestContext::new().await;
        let pool = ctx.pool().clone();
        let context = TelegramRuntimeEventBridgeContext::new(
            Some(crate::test_support::telegram_store(&pool)),
            InMemoryEventBus::new(),
        );
        let command = sample_command();

        publish_command_reconciled_events(Some(&context), &command, "tdlib.updateMessageContent")
            .await;

        let rows: Vec<(String, serde_json::Value)> = sqlx::query_as(
            r#"
            SELECT event_type, payload
            FROM event_log
            WHERE subject->>'id' = $1
              AND event_type IN ('telegram.command.status_changed', 'telegram.command.reconciled')
            ORDER BY position ASC
            "#,
        )
        .bind(&command.command_id)
        .fetch_all(&pool)
        .await
        .expect("command reconciliation events");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "telegram.command.status_changed");
        assert_eq!(rows[1].0, "telegram.command.reconciled");

        for (_, payload) in rows {
            assert_eq!(payload["command_id"], json!("cmd-1"));
            assert_eq!(payload["status"], json!("completed"));
            assert_eq!(payload["reconciliation_status"], json!("observed"));
            assert_eq!(payload["source"], json!("tdlib.updateMessageContent"));
            assert_eq!(
                payload["payload"]["source"],
                json!("tdlib.updateMessageContent")
            );
        }
    }
}

async fn publish_typing_event(
    telegram_store: &Option<TelegramStore>,
    event_bus: &InMemoryEventBus,
    account_id: &str,
    snapshot: &TelegramTdlibTypingSnapshot,
) {
    let Ok(event) = typing_changed_event(account_id, snapshot, Utc::now()) else {
        return;
    };

    if let Some(store) = telegram_store {
        let pool = store.pool();
        let event_store = EventStore::new(pool.clone());
        if let Err(error) = event_store.append(&event).await {
            tracing::warn!(error = %error, "Telegram runtime event bridge: failed to append typing event");
        }
    }

    let _ = event_bus.broadcast(event);
}

fn typing_changed_event(
    account_id: &str,
    snapshot: &TelegramTdlibTypingSnapshot,
    occurred_at: DateTime<Utc>,
) -> Result<NewEventEnvelope, makosh_events_api::EventEnvelopeError> {
    NewEventEnvelope::builder(
        format!(
            "evt_telegram_typing_{}_{}_{}",
            account_id,
            snapshot.provider_chat_id,
            occurred_at.timestamp_nanos_opt().unwrap_or(0)
        ),
        telegram_event_types::TYPING_CHANGED,
        occurred_at,
        json!({
            "channel": "telegram",
            "account_id": account_id,
            "runtime": "tdlib"
        }),
        json!({
            "kind": "telegram_chat",
            "provider_chat_id": snapshot.provider_chat_id
        }),
    )
    .payload(json!({
        "account_id": account_id,
        "provider_chat_id": snapshot.provider_chat_id,
        "provider_thread_id": snapshot.provider_thread_id,
        "sender_id": snapshot.sender_id,
        "action": snapshot.action,
        "is_active": snapshot.is_active,
        "source": "tdlib.updateUserChatAction"
    }))
    .provenance(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "tdlib_event": "updateUserChatAction"
    }))
    .build()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::TimeZone;
    use makosh_backend_testkit::context::TestContext;
    use serde_json::json;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio::time::timeout;

    use super::*;
    use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibMessageSnapshot;

    #[test]
    fn typing_changed_event_contains_sanitized_runtime_projection_payload() {
        let snapshot = TelegramTdlibTypingSnapshot {
            provider_chat_id: "-100123".to_owned(),
            provider_thread_id: Some("42".to_owned()),
            sender_id: "user:777".to_owned(),
            action: "chatActionTyping".to_owned(),
            is_active: true,
        };
        let occurred_at = Utc
            .with_ymd_and_hms(2026, 6, 17, 12, 0, 0)
            .single()
            .expect("valid test timestamp");

        let event = typing_changed_event("acct-1", &snapshot, occurred_at).expect("event");

        assert_eq!(event.event_type, telegram_event_types::TYPING_CHANGED);
        assert_eq!(event.source["account_id"], "acct-1");
        assert_eq!(event.subject["provider_chat_id"], "-100123");
        assert_eq!(event.payload["provider_thread_id"], "42");
        assert_eq!(event.payload["sender_id"], "user:777");
        assert_eq!(event.payload["is_active"], true);
        assert_eq!(event.provenance["tdlib_event"], "updateUserChatAction");
    }

    #[tokio::test]
    async fn telegram_runtime_event_bridge_skips_broadcast_when_runtime_paused() {
        let ctx = TestContext::new().await;
        let pool = ctx.pool().clone();
        crate::test_support::restore_signal_hub_system_sources(&pool).await;
        crate::test_support::set_signal_runtime_state(
            &pool,
            "telegram",
            TELEGRAM_RUNTIME_EVENT_BRIDGE_RUNTIME,
            "paused",
            json!({"scope": "subscription", "requested_by": "test"}),
        )
        .await;

        crate::test_support::upsert_telegram_runtime_account(
            &pool,
            "acct-runtime-bridge-paused",
            "Telegram Runtime Paused",
            "telegram-ext-runtime-paused",
        )
        .await;

        let event_bus = InMemoryEventBus::new();
        let mut events = event_bus.subscribe();
        let (tx, rx) = unbounded_channel();
        let telegram_store = crate::test_support::telegram_store(&pool);
        assert!(
            !telegram_runtime_event_bridge_allows_processing(&Some(telegram_store.clone())).await,
            "paused telegram runtime bridge must fail runtime gate"
        );
        spawn_telegram_runtime_event_bridge(
            Some(telegram_store),
            event_bus,
            "acct-runtime-bridge-paused".to_owned(),
            rx,
        );

        tx.send(TelegramRuntimeEvent::MessageCreated(
            TelegramTdlibMessageSnapshot {
                provider_chat_id: "-100bridge-paused".to_owned(),
                provider_message_id: "42".to_owned(),
                sender_id: "user:777".to_owned(),
                sender_display_name: "Alice".to_owned(),
                text: "skip while paused".to_owned(),
                occurred_at: Utc::now(),
                delivery_state:
                    crate::integrations::telegram::client::models::messages::TelegramDeliveryState::Received,
                raw: json!({
                    "@type": "message",
                    "chat_id": "-100bridge-paused",
                    "id": 42,
                }),
            },
        ))
        .expect("send runtime event");
        drop(tx);

        let no_event = timeout(Duration::from_millis(250), events.recv()).await;
        match no_event {
            Err(_) | Ok(Err(_)) => {}
            Ok(Ok(event)) => panic!(
                "paused bridge must not broadcast runtime event, got {}",
                event.event_type
            ),
        }

        let raw_signal_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM event_log WHERE event_type = 'signal.raw.telegram.message.observed'",
        )
        .fetch_one(&pool)
        .await
        .expect("raw signal count");
        assert_eq!(raw_signal_count, 0);
    }
}
