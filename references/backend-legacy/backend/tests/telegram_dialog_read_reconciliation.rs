mod telegram_support;

use axum::http::StatusCode;
use chrono::Utc;
use serde_json::json;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::integrations::telegram::client::chat_state::{
    reconcile_archive_commands_from_provider_state,
    reconcile_mark_read_commands_from_provider_state,
    reconcile_marked_as_unread_commands_from_provider_state,
    reconcile_mute_commands_from_provider_state, reconcile_pin_commands_from_provider_state,
};
use makosh_hub_backend::integrations::telegram::client::commands::{
    insert_command, new_command_id,
};
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, json_body, unique_suffix,
};

#[tokio::test]
async fn mark_read_reconciliation_completes_targeted_read_commands_from_chat_read_inbox() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-reconcile-{suffix}");
    let provider_chat_id = format!("dialog-reconcile-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Dialog Read Reconcile",
            "external_account_id": format!("tg-dialog-reconcile-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("{provider_chat_id}:700"),
            "chat_kind": "private",
            "chat_title": "Dialog Reconcile Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Irina Volkova",
            "text": "Targeted mark-read commands should reconcile from updateChatReadInbox state.",
            "import_batch_id": format!("telegram-dialog-reconcile-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let chats_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations?account_id={account_id}&limit=10"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chats response");
    assert_eq!(chats_response.status(), StatusCode::OK);
    let chats_body = json_body(chats_response).await;
    let telegram_chat_id = chats_body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned();

    let target_message_id = format!("{provider_chat_id}:777");
    let command_id = new_command_id();
    insert_command(
        &pool,
        &command_id,
        &account_id,
        "mark_read",
        &format!("mark_read:{telegram_chat_id}:manual"),
        &provider_chat_id,
        Some(&target_message_id),
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_dialog_read_reconciliation",
            "last_read_inbox_provider_message_id": target_message_id,
        }),
        json!({
            "telegram_chat_id": telegram_chat_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": target_message_id,
        }),
        json!({
            "source": "telegram_dialog_read_reconciliation",
            "last_read_inbox_provider_message_id": target_message_id,
        }),
    )
    .await
    .expect("mark_read command row");

    let reconciled = reconcile_mark_read_commands_from_provider_state(
        &pool,
        &account_id,
        &provider_chat_id,
        &format!("{provider_chat_id}:778"),
        Utc::now(),
        "tdlib.updateChatReadInbox",
    )
    .await
    .expect("mark read reconciliation");
    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_id, command_id);
    assert_eq!(reconciled[0].command_kind, "mark_read");
    assert_eq!(
        reconciled[0].provider_message_id.as_deref(),
        Some(target_message_id.as_str())
    );
    assert_eq!(reconciled[0].status, "completed");
    assert_eq!(reconciled[0].reconciliation_status, "observed");
}

#[tokio::test]
async fn dialog_pin_reconciliation_marks_mismatched_unpin_command_failed() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-pin-reconcile-{suffix}");
    let provider_chat_id = format!("dialog-pin-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Dialog Pin Reconcile",
            "external_account_id": format!("tg-dialog-pin-reconcile-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let command_id = new_command_id();
    insert_command(
        &pool,
        &command_id,
        &account_id,
        "unpin",
        &format!("unpin:{provider_chat_id}:manual"),
        &provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_dialog_read_reconciliation",
            "is_pinned": false,
        }),
        json!({
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_dialog_read_reconciliation",
        }),
    )
    .await
    .expect("unpin command row");

    let reconciled = reconcile_pin_commands_from_provider_state(
        &pool,
        &account_id,
        &provider_chat_id,
        true,
        Utc::now(),
        "tdlib.updateChatPosition",
    )
    .await
    .expect("dialog pin reconciliation");
    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_id, command_id);
    assert_eq!(reconciled[0].command_kind, "unpin");
    assert_eq!(reconciled[0].status, "failed");
    assert_eq!(reconciled[0].reconciliation_status, "mismatch");
    assert_eq!(
        reconciled[0].last_error.as_deref(),
        Some("Provider observed a different dialog pin state than requested")
    );
    assert_eq!(
        reconciled[0].provider_state["expected_is_pinned"],
        json!(false)
    );
    assert_eq!(
        reconciled[0].provider_state["observed_is_pinned"],
        json!(true)
    );
}

#[tokio::test]
async fn dialog_archive_reconciliation_marks_mismatched_unarchive_command_failed() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-archive-reconcile-{suffix}");
    let provider_chat_id = format!("dialog-archive-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Dialog Archive Reconcile",
            "external_account_id": format!("tg-dialog-archive-reconcile-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    insert_command(
        &pool,
        &new_command_id(),
        &account_id,
        "unarchive",
        &format!("unarchive:{provider_chat_id}:manual"),
        &provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_dialog_read_reconciliation",
            "is_archived": false,
        }),
        json!({
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_dialog_read_reconciliation",
        }),
    )
    .await
    .expect("unarchive command row");

    let reconciled = reconcile_archive_commands_from_provider_state(
        &pool,
        &account_id,
        &provider_chat_id,
        true,
        Utc::now(),
        "tdlib.updateChatPosition",
    )
    .await
    .expect("dialog archive reconciliation");

    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_kind, "unarchive");
    assert_eq!(reconciled[0].status, "failed");
    assert_eq!(reconciled[0].reconciliation_status, "mismatch");
    assert_eq!(
        reconciled[0].last_error.as_deref(),
        Some("Provider observed a different archive state than requested")
    );
    assert_eq!(
        reconciled[0].provider_state["expected_is_archived"],
        json!(false)
    );
    assert_eq!(
        reconciled[0].provider_state["observed_is_archived"],
        json!(true)
    );
}

#[tokio::test]
async fn dialog_mute_reconciliation_marks_mismatched_unmute_command_failed() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-mute-reconcile-{suffix}");
    let provider_chat_id = format!("dialog-mute-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Dialog Mute Reconcile",
            "external_account_id": format!("tg-dialog-mute-reconcile-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    insert_command(
        &pool,
        &new_command_id(),
        &account_id,
        "unmute",
        &format!("unmute:{provider_chat_id}:manual"),
        &provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_dialog_read_reconciliation",
            "use_default_mute_for": true,
            "mute_for": 0,
        }),
        json!({
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_dialog_read_reconciliation",
        }),
    )
    .await
    .expect("unmute command row");

    let reconciled = reconcile_mute_commands_from_provider_state(
        &pool,
        &account_id,
        &provider_chat_id,
        false,
        31_708_800,
        Utc::now(),
        "tdlib.updateChatNotificationSettings",
    )
    .await
    .expect("dialog mute reconciliation");

    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_kind, "unmute");
    assert_eq!(reconciled[0].status, "failed");
    assert_eq!(reconciled[0].reconciliation_status, "mismatch");
    assert_eq!(
        reconciled[0].last_error.as_deref(),
        Some("Provider observed a different mute state than requested")
    );
    assert_eq!(
        reconciled[0].provider_state["expected_is_muted"],
        json!(false)
    );
    assert_eq!(
        reconciled[0].provider_state["observed_is_muted"],
        json!(true)
    );
}

#[tokio::test]
async fn dialog_mark_unread_reconciliation_completes_matching_command() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-unread-reconcile-{suffix}");
    let provider_chat_id = format!("dialog-unread-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Dialog Unread Reconcile",
            "external_account_id": format!("tg-dialog-unread-reconcile-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    insert_command(
        &pool,
        &new_command_id(),
        &account_id,
        "mark_unread",
        &format!("mark_unread:{provider_chat_id}:manual"),
        &provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_dialog_read_reconciliation",
            "is_marked_as_unread": true,
        }),
        json!({
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_dialog_read_reconciliation",
        }),
    )
    .await
    .expect("mark_unread command row");

    let reconciled = reconcile_marked_as_unread_commands_from_provider_state(
        &pool,
        &account_id,
        &provider_chat_id,
        true,
        Utc::now(),
        "tdlib.updateChatIsMarkedAsUnread",
    )
    .await
    .expect("dialog unread reconciliation");

    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_kind, "mark_unread");
    assert_eq!(reconciled[0].status, "completed");
    assert_eq!(reconciled[0].reconciliation_status, "observed");
}

#[tokio::test]
async fn dialog_mark_unread_reconciliation_marks_provider_read_state_as_mismatch() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-unread-mismatch-{suffix}");
    let provider_chat_id = format!("dialog-unread-mismatch-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Dialog Unread Mismatch",
            "external_account_id": format!("tg-dialog-unread-mismatch-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    insert_command(
        &pool,
        &new_command_id(),
        &account_id,
        "mark_unread",
        &format!("mark_unread:{provider_chat_id}:manual"),
        &provider_chat_id,
        None,
        "available",
        "provider_write",
        "confirmed",
        "makosh-frontend",
        json!({
            "source": "telegram_dialog_read_reconciliation",
            "is_marked_as_unread": true,
        }),
        json!({
            "provider_chat_id": provider_chat_id,
        }),
        json!({
            "source": "telegram_dialog_read_reconciliation",
        }),
    )
    .await
    .expect("mark_unread command row");

    let reconciled = reconcile_marked_as_unread_commands_from_provider_state(
        &pool,
        &account_id,
        &provider_chat_id,
        false,
        Utc::now(),
        "tdlib.updateChatIsMarkedAsUnread",
    )
    .await
    .expect("dialog unread mismatch");

    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].command_kind, "mark_unread");
    assert_eq!(reconciled[0].status, "failed");
    assert_eq!(reconciled[0].reconciliation_status, "mismatch");
    assert_eq!(
        reconciled[0].last_error.as_deref(),
        Some("Provider observed a different unread state than requested")
    );
    assert_eq!(
        reconciled[0].provider_state["expected_is_marked_as_unread"],
        json!(true)
    );
    assert_eq!(
        reconciled[0].provider_state["observed_is_marked_as_unread"],
        json!(false)
    );
}
