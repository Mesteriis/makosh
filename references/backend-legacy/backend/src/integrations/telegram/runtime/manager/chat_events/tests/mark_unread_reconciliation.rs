use makosh_backend_testkit::context::TestContext;
use serde_json::json;

use crate::integrations::telegram::client::commands::insert_command;
use crate::integrations::telegram::client::models::chats::TelegramChat;
use crate::integrations::telegram::tdjson::parsing::events::TelegramTdlibChatMarkedAsUnreadSnapshot;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::telegram_event_types;

use super::super::publish_chat_marked_as_unread_event;
use super::seed_chat;

#[tokio::test]
async fn publish_chat_marked_as_unread_event_reconciles_mark_unread_command_and_emits_events() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-mark-unread-reconcile";
    let provider_chat_id = "chat-mark-unread-reconcile";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let command_id = "cmd-mark-unread-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "mark_unread",
        "mark_unread:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_marked_as_unread": true,
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
    .expect("seed mark_unread command");

    let event_bus = InMemoryEventBus::new();
    publish_chat_marked_as_unread_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatMarkedAsUnreadSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            is_marked_as_unread: true,
            source_event: "updateChatIsMarkedAsUnread".to_owned(),
        },
    )
    .await;

    let row: (String, String) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("mark_unread command status");
    assert_eq!(row, ("completed".to_owned(), "observed".to_owned()));

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
    .expect("mark_unread command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}

#[tokio::test]
async fn publish_chat_marked_as_unread_event_marks_mark_unread_as_mismatch_when_provider_disagrees()
{
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-mark-unread-mismatch";
    let provider_chat_id = "chat-mark-unread-mismatch";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let command_id = "cmd-mark-unread-mismatch-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "mark_unread",
        "mark_unread:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_marked_as_unread": true,
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
    .expect("seed mark_unread command");

    let event_bus = InMemoryEventBus::new();
    publish_chat_marked_as_unread_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatMarkedAsUnreadSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            is_marked_as_unread: false,
            source_event: "updateChatIsMarkedAsUnread".to_owned(),
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
    .expect("mark_unread mismatch command status");
    assert_eq!(row.0, "failed");
    assert_eq!(row.1, "mismatch");
    assert_eq!(
        row.2,
        Some("Provider observed a different unread state than requested".to_owned())
    );
    assert_eq!(row.3["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(row.3["expected_is_marked_as_unread"], json!(true));
    assert_eq!(row.3["observed_is_marked_as_unread"], json!(false));
    assert_eq!(
        row.3["observed_via"],
        json!("tdlib.updateChatIsMarkedAsUnread")
    );
    assert_eq!(row.4["mismatch"], json!(true));

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
    .expect("mark_unread mismatch command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["result_payload"]["mismatch"], json!(true));
}
