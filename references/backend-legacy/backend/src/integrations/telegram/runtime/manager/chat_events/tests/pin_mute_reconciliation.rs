use serde_json::json;

use crate::integrations::telegram::client::commands::insert_command;
use crate::integrations::telegram::client::models::chats::TelegramChat;
use crate::integrations::telegram::tdjson::{
    parsing::events::TelegramTdlibChatNotificationSettingsSnapshot,
    parsing::events::TelegramTdlibChatPositionSnapshot,
};
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::telegram_event_types;

use super::super::publish_chat_notification_settings_event;
use super::super::publish_chat_position_event;
use super::seed_chat;

#[tokio::test]
async fn publish_chat_position_event_marks_pin_command_as_mismatch_when_provider_disagrees() {
    let ctx = makosh_backend_testkit::context::TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-pin-mismatch";
    let provider_chat_id = "chat-pin-mismatch";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let command_id = "cmd-pin-mismatch-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "pin",
        "pin:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_pinned": true,
        }),
        json!({
            "telegram_chat_id": chat.telegram_chat_id,
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_runtime",
        }),
    )
    .await
    .expect("seed pin command");

    let event_bus = InMemoryEventBus::new();
    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "main".to_owned(),
            provider_folder_id: None,
            order: 42,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
        },
    )
    .await;

    let row: (
        String,
        String,
        Option<String>,
        serde_json::Value,
        serde_json::Value,
    ) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status, last_error, provider_state, result_payload
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("pin mismatch command status");
    assert_eq!(row.0, "failed");
    assert_eq!(row.1, "mismatch");
    assert_eq!(
        row.2,
        Some("Provider observed a different dialog pin state than requested".to_owned())
    );
    assert_eq!(row.3["expected_is_pinned"], json!(true));
    assert_eq!(row.3["observed_is_pinned"], json!(false));
    assert_eq!(row.4["observed_via"], json!("tdlib.updateChatPosition"));

    let events: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.command.status_changed', 'telegram.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(command_id)
    .fetch_all(&pool)
    .await
    .expect("pin mismatch command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}

#[tokio::test]
async fn publish_chat_notification_settings_event_marks_unmute_command_as_mismatch_when_provider_disagrees()
 {
    let ctx = makosh_backend_testkit::context::TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-unmute-mismatch";
    let provider_chat_id = "chat-unmute-mismatch";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let command_id = "cmd-unmute-mismatch-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "unmute",
        "unmute:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_muted": false,
            "use_default_mute_for": true,
            "mute_for": 0,
        }),
        json!({
            "telegram_chat_id": chat.telegram_chat_id,
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_runtime",
        }),
    )
    .await
    .expect("seed unmute command");

    let event_bus = InMemoryEventBus::new();
    publish_chat_notification_settings_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatNotificationSettingsSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            use_default_mute_for: false,
            mute_for: 31_708_800,
            source_event: "updateChatNotificationSettings".to_owned(),
        },
    )
    .await;

    let row: (
        String,
        String,
        Option<String>,
        serde_json::Value,
        serde_json::Value,
    ) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status, last_error, provider_state, result_payload
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("unmute mismatch command status");
    assert_eq!(row.0, "failed");
    assert_eq!(row.1, "mismatch");
    assert_eq!(
        row.2,
        Some("Provider observed a different mute state than requested".to_owned())
    );
    assert_eq!(row.3["expected_is_muted"], json!(false));
    assert_eq!(row.3["observed_is_muted"], json!(true));
    assert_eq!(
        row.4["observed_via"],
        json!("tdlib.updateChatNotificationSettings")
    );

    let events: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.command.status_changed', 'telegram.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(command_id)
    .fetch_all(&pool)
    .await
    .expect("unmute mismatch command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}
