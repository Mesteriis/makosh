use makosh_backend_testkit::context::TestContext;
use serde_json::json;
use sqlx::PgPool;

use super::{
    publish_chat_folders_event, publish_chat_notification_settings_event,
    publish_chat_position_event, publish_chat_unread_event,
};
use crate::integrations::telegram::client::commands::insert_command;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::chats::TelegramChat;
use crate::integrations::telegram::client::models::chats::{
    NewTelegramChat, TelegramChatKind, TelegramSyncState,
};
use crate::integrations::telegram::tdjson::{
    parsing::events::TelegramTdlibChatNotificationSettingsSnapshot,
    parsing::events::TelegramTdlibChatPositionSnapshot,
    parsing::events::TelegramTdlibChatUnreadSnapshot, snapshots::TelegramTdlibChatFolderSnapshot,
};
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::events::bus::telegram_event_types;

#[cfg(test)]
mod archive_reconciliation;
#[cfg(test)]
mod mark_unread_reconciliation;
#[cfg(test)]
mod pin_mute_reconciliation;

async fn seed_chat(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
) -> Result<TelegramChat, TelegramError> {
    crate::test_support::upsert_telegram_runtime_account(
        pool,
        account_id,
        "Runtime Chat Account",
        &format!("telegram-ext-{account_id}"),
    )
    .await;
    crate::test_support::telegram_store(pool)
        .upsert_chat(&NewTelegramChat {
            account_id: account_id.to_owned(),
            provider_chat_id: provider_chat_id.to_owned(),
            chat_kind: TelegramChatKind::Private,
            title: "Runtime Chat".to_owned(),
            username: None,
            sync_state: TelegramSyncState::Synced,
            last_message_at: None,
            metadata: json!({}),
        })
        .await
}

#[tokio::test]
async fn publish_chat_notification_settings_event_appends_chat_updated_before_chat_muted() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-1";
    let provider_chat_id = "chat-1";
    let chat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let event_bus = InMemoryEventBus::new();
    let snapshot = TelegramTdlibChatNotificationSettingsSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        use_default_mute_for: false,
        mute_for: 3600,
        source_event: "updateChatNotificationSettings".to_owned(),
    };

    publish_chat_notification_settings_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    let rows: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.chat.updated', 'telegram.chat.muted')
        ORDER BY position ASC
        "#,
    )
    .bind(&chat.telegram_chat_id)
    .fetch_all(&pool)
    .await
    .expect("notification events");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, telegram_event_types::CHAT_UPDATED);
    assert_eq!(
        rows[0].1["action"],
        json!("provider_notification_settings_update")
    );
    assert_eq!(rows[0].1["chat"]["metadata"]["is_muted"], json!(true));
    assert_eq!(rows[1].0, telegram_event_types::CHAT_MUTED);
    assert_eq!(rows[1].1["is_muted"], json!(true));
}

#[tokio::test]
async fn publish_chat_position_event_appends_chat_updated_before_flag_events() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-1";
    let provider_chat_id = "chat-2";
    let chat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let event_bus = InMemoryEventBus::new();
    let snapshot = TelegramTdlibChatPositionSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        list_kind: "archive".to_owned(),
        provider_folder_id: Some(7),
        order: 42,
        is_pinned: true,
        source_event: "updateChatPosition".to_owned(),
    };

    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    let rows: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN (
            'telegram.chat.updated',
            'telegram.chat.pinned',
            'telegram.chat.archived'
          )
        ORDER BY position ASC
        "#,
    )
    .bind(&chat.telegram_chat_id)
    .fetch_all(&pool)
    .await
    .expect("position events");

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].0, telegram_event_types::CHAT_UPDATED);
    assert_eq!(rows[0].1["action"], json!("provider_chat_position_update"));
    assert_eq!(rows[0].1["chat"]["metadata"]["is_archived"], json!(true));
    assert_eq!(rows[0].1["chat"]["metadata"]["is_pinned"], json!(true));
    assert_eq!(rows[1].0, telegram_event_types::CHAT_PINNED);
    assert_eq!(rows[1].1["is_pinned"], json!(true));
    assert_eq!(rows[2].0, telegram_event_types::CHAT_ARCHIVED);
    assert_eq!(rows[2].1["is_archived"], json!(true));
}

#[tokio::test]
async fn publish_chat_position_event_emits_folder_filters_for_folder_membership_changes() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-folder";
    let provider_chat_id = "chat-folder";
    let _chat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let event_bus = InMemoryEventBus::new();
    let snapshot = TelegramTdlibChatPositionSnapshot {
        provider_chat_id: provider_chat_id.to_owned(),
        list_kind: "folder".to_owned(),
        provider_folder_id: Some(7),
        order: 42,
        is_pinned: false,
        source_event: "updateChatPosition".to_owned(),
    };

    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &snapshot,
    )
    .await;

    let row: (String, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE event_type = 'telegram.folders.updated'
          AND payload->>'account_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .expect("folders updated event");

    assert_eq!(row.0, telegram_event_types::FOLDERS_UPDATED);
    let items = row.1["items"].as_array().expect("folder items");
    assert!(items.iter().any(|item| item["id"] == json!("local:all")));
    assert!(items.iter().any(|item| {
        item["id"] == json!("folder:Unknown folder 7")
            && item["provider_folder_id"] == json!(7)
            && item["count"] == json!(1)
    }));
}

#[tokio::test]
async fn publish_chat_folders_event_emits_chat_updated_for_folder_label_projection_changes() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-folder-labels";
    let provider_chat_id = "chat-folder-labels";
    let chat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let event_bus = InMemoryEventBus::new();

    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "folder".to_owned(),
            provider_folder_id: Some(7),
            order: 42,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
        },
    )
    .await;

    publish_chat_folders_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &[TelegramTdlibChatFolderSnapshot {
            provider_folder_id: 7,
            title: "Projects".to_owned(),
            icon_name: None,
            color_id: None,
            raw: json!({
                "@type": "chatFolder",
                "id": 7,
                "name": { "@type": "formattedText", "text": "Projects" },
            }),
        }],
    )
    .await;

    let row: (String, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type = 'telegram.chat.updated'
          AND payload->>'action' = 'provider_chat_folder_labels_update'
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&chat.telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("folder label chat update event");

    assert_eq!(row.0, telegram_event_types::CHAT_UPDATED);
    assert_eq!(row.1["provider_folder_id"], json!(7));
    assert_eq!(row.1["folder_labels"], json!(["Projects"]));
    assert_eq!(row.1["chat"]["metadata"]["folder_name"], json!("Projects"));
    assert_eq!(row.1["chat"]["metadata"]["provider_folder_id"], json!(7));
}

#[tokio::test]
async fn publish_chat_folders_event_refreshes_unknown_labels_when_folder_snapshot_disappears() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-folder-missing";
    let provider_chat_id = "chat-folder-missing";
    let chat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let event_bus = InMemoryEventBus::new();

    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "folder".to_owned(),
            provider_folder_id: Some(7),
            order: 42,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
        },
    )
    .await;

    publish_chat_folders_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &[TelegramTdlibChatFolderSnapshot {
            provider_folder_id: 7,
            title: "Projects".to_owned(),
            icon_name: None,
            color_id: None,
            raw: json!({
                "@type": "chatFolder",
                "id": 7,
                "name": { "@type": "formattedText", "text": "Projects" },
            }),
        }],
    )
    .await;

    publish_chat_folders_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &[],
    )
    .await;

    let row: (String, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type = 'telegram.chat.updated'
          AND payload->>'action' = 'provider_chat_folder_labels_update'
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&chat.telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("fallback folder label event");

    assert_eq!(row.0, telegram_event_types::CHAT_UPDATED);
    assert_eq!(row.1["folder_labels"], json!(["Unknown folder 7"]));
    assert_eq!(row.1["provider_folder_id"], json!(7));
    assert_eq!(
        row.1["chat"]["metadata"]["folder_name"],
        json!("Unknown folder 7")
    );
    assert_eq!(row.1["chat"]["metadata"]["provider_folder_id"], json!(7));
}

#[tokio::test]
async fn publish_chat_position_event_reconciles_folder_add_and_remove_commands() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-folder-reconcile";
    let provider_chat_id = "chat-folder-reconcile";
    let chat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let event_bus = InMemoryEventBus::new();

    let add_command_id = "cmd-folder-add-1";
    let remove_command_id = "cmd-folder-remove-1";
    insert_command(
        &pool,
        add_command_id,
        account_id,
        "folder_add",
        "folder-add-runtime",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({"provider_folder_id": 7}),
        json!({"telegram_chat_id": chat.telegram_chat_id, "provider_chat_id": provider_chat_id}),
        json!({"source": "runtime-test"}),
    )
    .await
    .expect("seed folder_add command");
    insert_command(
        &pool,
        remove_command_id,
        account_id,
        "folder_remove",
        "folder-remove-runtime",
        provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({"provider_folder_id": 7}),
        json!({"telegram_chat_id": chat.telegram_chat_id, "provider_chat_id": provider_chat_id}),
        json!({"source": "runtime-test"}),
    )
    .await
    .expect("seed folder_remove command");

    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "folder".to_owned(),
            provider_folder_id: Some(7),
            order: 42,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
        },
    )
    .await;

    let add_row: (String, String) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(add_command_id)
    .fetch_one(&pool)
    .await
    .expect("folder_add command status");
    let remove_row: (String, String) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(remove_command_id)
    .fetch_one(&pool)
    .await
    .expect("folder_remove command status");

    assert_eq!(add_row, ("completed".to_owned(), "observed".to_owned()));
    assert_eq!(remove_row, ("queued".to_owned(), "not_observed".to_owned()));

    let add_events: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.command.status_changed', 'telegram.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(add_command_id)
    .fetch_all(&pool)
    .await
    .expect("folder_add command events");
    assert_eq!(add_events.len(), 2);
    assert_eq!(
        add_events[0].0,
        telegram_event_types::COMMAND_STATUS_CHANGED
    );
    assert_eq!(add_events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(add_events[0].1["command_id"], json!(add_command_id));
    assert_eq!(add_events[1].1["command_id"], json!(add_command_id));

    publish_chat_position_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatPositionSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            list_kind: "folder".to_owned(),
            provider_folder_id: Some(7),
            order: 0,
            is_pinned: false,
            source_event: "updateChatPosition".to_owned(),
        },
    )
    .await;

    let remove_row: (String, String) = sqlx::query_as(
        r#"
        SELECT status, reconciliation_status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(remove_command_id)
    .fetch_one(&pool)
    .await
    .expect("folder_remove command status");
    assert_eq!(remove_row, ("completed".to_owned(), "observed".to_owned()));

    let remove_events: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type IN ('telegram.command.status_changed', 'telegram.command.reconciled')
        ORDER BY position ASC
        "#,
    )
    .bind(remove_command_id)
    .fetch_all(&pool)
    .await
    .expect("folder_remove command events");
    assert_eq!(remove_events.len(), 2);
    assert_eq!(
        remove_events[0].0,
        telegram_event_types::COMMAND_STATUS_CHANGED
    );
    assert_eq!(remove_events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(remove_events[0].1["command_id"], json!(remove_command_id));
    assert_eq!(remove_events[1].1["command_id"], json!(remove_command_id));
}

#[tokio::test]
async fn publish_chat_unread_event_reconciles_mark_read_command_and_emits_events() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let account_id = "acct-unread-reconcile";
    let provider_chat_id = "chat-unread-reconcile";
    let chat = seed_chat(&pool, account_id, provider_chat_id)
        .await
        .expect("seed chat");
    let command_id = "cmd-mark-read-1";
    let target_message_id = format!("{provider_chat_id}:777");
    insert_command(
        &pool,
        command_id,
        account_id,
        "mark_read",
        "mark_read:manual",
        provider_chat_id,
        Some(&target_message_id),
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_runtime",
            "last_read_inbox_provider_message_id": target_message_id,
        }),
        json!({
            "telegram_chat_id": chat.telegram_chat_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": target_message_id,
        }),
        json!({
            "source": "telegram_runtime",
        }),
    )
    .await
    .expect("seed mark_read command");

    let event_bus = InMemoryEventBus::new();
    publish_chat_unread_event(
        &Some(crate::test_support::telegram_store(&pool)),
        &event_bus,
        account_id,
        &TelegramTdlibChatUnreadSnapshot {
            provider_chat_id: provider_chat_id.to_owned(),
            unread_count: None,
            unread_mention_count: None,
            last_read_inbox_message_id: Some("778".to_owned()),
            source_event: "updateChatReadInbox".to_owned(),
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
    .expect("mark_read command status");
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
    .expect("mark_read command events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, telegram_event_types::COMMAND_STATUS_CHANGED);
    assert_eq!(events[1].0, telegram_event_types::COMMAND_RECONCILED);
    assert_eq!(events[0].1["command_id"], json!(command_id));
    assert_eq!(events[1].1["command_id"], json!(command_id));
}
