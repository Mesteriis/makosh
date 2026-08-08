use serde_json::json;

use crate::integrations::telegram::client::commands::insert_command;
use crate::integrations::telegram::client::models::chats::TelegramChat;
use crate::integrations::telegram::tdjson::parsing::events::TelegramTdlibChatPositionSnapshot;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::telegram_event_types;

use super::super::publish_chat_position_event;
use super::seed_chat;

#[tokio::test]
async fn publish_chat_position_event_reconciles_archive_command_when_provider_chat_is_archived() {
    let ctx = makosh_backend_testkit::context::TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-archive-reconcile-1";
    let provider_chat_id = "chat-archive-reconcile-1";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed archive chat");
    let command_id = "cmd-archive-reconcile-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "archive",
        "archive:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_archived": true,
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
    .expect("seed archive command");

    let event_bus = InMemoryEventBus::new();
    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "archive".to_owned(),
            provider_folder_id: None,
            order: 42,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
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
    .expect("archive command status");
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
    .expect("archive command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}

#[tokio::test]
async fn publish_chat_position_event_reconciles_unarchive_command_when_provider_chat_is_unarchived()
{
    let ctx = makosh_backend_testkit::context::TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-unarchive-reconcile-1";
    let provider_chat_id = "chat-unarchive-reconcile-1";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed unarchive chat");
    let command_id = "cmd-unarchive-reconcile-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "unarchive",
        "unarchive:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_archived": false,
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
    .expect("seed unarchive command");

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
    .expect("unarchive command status");
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
    .expect("unarchive command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}

#[tokio::test]
async fn publish_chat_position_event_marks_archive_command_as_mismatch_when_provider_disagrees() {
    let ctx = makosh_backend_testkit::context::TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-archive-mismatch-1";
    let provider_chat_id = "chat-archive-mismatch-1";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed archive mismatch chat");
    let command_id = "cmd-archive-mismatch-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "archive",
        "archive-mismatch:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_archived": true,
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
    .expect("seed archive mismatch command");

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
    .expect("archive mismatch command status");
    assert_eq!(row.0, "failed");
    assert_eq!(row.1, "mismatch");
    assert_eq!(
        row.2,
        Some("Provider observed a different archive state than requested".to_owned())
    );
    assert_eq!(row.3["expected_is_archived"], json!(true));
    assert_eq!(row.3["observed_is_archived"], json!(false));
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
    .expect("archive mismatch command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}

#[tokio::test]
async fn publish_chat_position_event_marks_unarchive_command_as_mismatch_when_provider_disagrees() {
    let ctx = makosh_backend_testkit::context::TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-unarchive-mismatch-1";
    let provider_chat_id = "chat-unarchive-mismatch-1";
    let chat: TelegramChat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed unarchive mismatch chat");
    let command_id = "cmd-unarchive-mismatch-1";
    insert_command(
        &pool,
        command_id,
        account_id,
        "unarchive",
        "unarchive-mismatch:manual",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "is_archived": false,
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
    .expect("seed unarchive mismatch command");

    let event_bus = InMemoryEventBus::new();
    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "archive".to_owned(),
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
    .expect("unarchive mismatch command status");
    assert_eq!(row.0, "failed");
    assert_eq!(row.1, "mismatch");
    assert_eq!(
        row.2,
        Some("Provider observed a different archive state than requested".to_owned())
    );
    assert_eq!(row.3["expected_is_archived"], json!(false));
    assert_eq!(row.3["observed_is_archived"], json!(true));
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
    .expect("unarchive mismatch command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}
