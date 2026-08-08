mod telegram_support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, ingest_fixture_telegram_message, json_body,
    json_post_request_with_actor, unique_suffix,
};

#[tokio::test]
async fn telegram_dialog_search_returns_projected_chat_matches() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-search-{suffix}");
    let matching_chat_id = format!("chat-alpha-{suffix}");
    let other_chat_id = format!("chat-beta-{suffix}");
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
            "display_name": "Telegram Dialog Search",
            "external_account_id": format!("tg-dialog-search-{suffix}"),
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
            "provider_chat_id": matching_chat_id,
            "provider_message_id": format!("dialog-search-message-1-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Project Alpha Ops",
            "sender_id": format!("sender-alpha-{suffix}"),
            "sender_display_name": "Alpha Sender",
            "text": "Alpha conversation seed",
            "import_batch_id": format!("telegram-dialog-search-seed-1-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": other_chat_id,
            "provider_message_id": format!("dialog-search-message-2-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Beta Support",
            "sender_id": format!("sender-beta-{suffix}"),
            "sender_display_name": "Beta Sender",
            "text": "Beta conversation seed",
            "import_batch_id": format!("telegram-dialog-search-seed-2-{suffix}"),
            "occurred_at": "2026-06-06T12:05:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations/search?q=Alpha&account_id={account_id}&limit=10"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dialog search response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("dialog search items");
    assert_eq!(body["query"], json!("Alpha"));
    assert_eq!(body["total"], json!(1));
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["provider_chat_id"], json!(matching_chat_id));
    assert_eq!(items[0]["title"], json!("Project Alpha Ops"));
}

#[tokio::test]
async fn telegram_media_search_filters_by_free_text_query() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-media-search-{suffix}");
    let chat_id = format!("chat-media-{suffix}");
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
            "display_name": "Telegram Media Search",
            "external_account_id": format!("tg-media-search-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("media-search-message-1-{suffix}"),
                "chat_kind": "private",
                "chat_title": "Media Search Chat",
                "sender_id": format!("sender-media-{suffix}"),
                "sender_display_name": "Media Sender",
                "text": "invoice attachment",
                "import_batch_id": format!("telegram-media-search-seed-1-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    let message_id = message_body["message_id"]
        .as_str()
        .expect("message id")
        .to_owned();

    sqlx::query(
        r#"
        UPDATE communication_messages
        SET message_metadata = $2::jsonb
        WHERE message_id = $1
        "#,
    )
    .bind(&message_id)
    .bind(json!({
        "attachments": [
            {
                "file_name": "invoice-2026.pdf",
                "kind": "document",
                "mime_type": "application/pdf",
                "size_bytes": 12345,
                "download_state": "downloaded",
                "attachment_id": "attachment-invoice-1",
                "tdlib_file_id": 4201,
                "local_path": "/tmp/makosh/invoice-2026.pdf"
            },
            {
                "file_name": "holiday-photo.jpg",
                "kind": "photo",
                "mime_type": "image/jpeg",
                "size_bytes": 45678,
                "download_state": "downloaded"
            }
        ]
    }))
    .execute(&pool)
    .await
    .expect("update message metadata");

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/search/media?q=invoice&account_id={account_id}&provider_chat_id={chat_id}&limit=20"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("media search response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("media search items");
    assert_eq!(body["query"], json!("invoice"));
    assert_eq!(body["source"], json!("projection"));
    assert_eq!(body["provider_search_attempted"], json!(false));
    assert_eq!(body["provider_search_error"], json!(null));
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["file_name"], json!("invoice-2026.pdf"));
    assert_eq!(
        items[0]["provider_attachment_id"],
        json!("attachment-invoice-1")
    );
    assert_eq!(items[0]["tdlib_file_id"], json!(4201));
    assert_eq!(
        items[0]["local_path"],
        json!("/tmp/makosh/invoice-2026.pdf")
    );
}

#[tokio::test]
async fn telegram_pinned_messages_route_returns_projection_backed_items() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-pinned-messages-{suffix}");
    let chat_id = format!("chat-pinned-{suffix}");
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
            "display_name": "Telegram Pinned Messages",
            "external_account_id": format!("tg-pinned-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let first_message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("pinned-message-1-{suffix}"),
        "Pinned root message",
        "2026-06-06T12:00:00Z",
    )
    .await;
    let second_message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("pinned-message-2-{suffix}"),
        "Newest pinned message",
        "2026-06-06T12:10:00Z",
    )
    .await;
    let unpinned_message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("pinned-message-3-{suffix}"),
        "Unpinned message",
        "2026-06-06T12:20:00Z",
    )
    .await;

    for message_id in [&first_message_id, &second_message_id] {
        sqlx::query(
            r#"
            UPDATE communication_messages
            SET message_metadata = $2::jsonb
            WHERE message_id = $1
            "#,
        )
        .bind(message_id)
        .bind(json!({ "is_pinned": true }))
        .execute(&pool)
        .await
        .expect("update pinned metadata");
    }
    sqlx::query(
        r#"
        UPDATE communication_messages
        SET message_metadata = $2::jsonb
        WHERE message_id = $1
        "#,
    )
    .bind(&unpinned_message_id)
    .bind(json!({ "is_pinned": false }))
    .execute(&pool)
    .await
    .expect("update unpinned metadata");

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

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/conversations/{telegram_chat_id}/pinned-messages?limit=10"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("pinned messages response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("pinned message items");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["message_id"], json!(second_message_id));
    assert_eq!(items[0]["text"], json!("Newest pinned message"));
    assert_eq!(items[1]["message_id"], json!(first_message_id));
    assert!(
        items
            .iter()
            .all(|item| item["message_id"] != json!(unpinned_message_id))
    );
}

#[tokio::test]
async fn telegram_message_created_event_includes_projected_chat_snapshot() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-created-event-{suffix}");
    let chat_id = format!("chat-created-event-{suffix}");
    let provider_message_id = format!("provider-created-event-{suffix}");
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
            "display_name": "Telegram Created Event",
            "external_account_id": format!("tg-created-event-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &chat_id,
        &provider_message_id,
        "@makosh newest message should patch dialog caches.",
        "2026-06-06T12:00:00Z",
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

    let realtime_payload: Value = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'telegram.message.created'
          AND subject->>'id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("telegram message created event payload");
    assert_eq!(realtime_payload["message"]["message_id"], json!(message_id));
    assert_eq!(
        realtime_payload["telegram_chat_id"],
        json!(telegram_chat_id)
    );
    assert_eq!(
        realtime_payload["chat"]["telegram_chat_id"],
        json!(telegram_chat_id)
    );
    assert_eq!(realtime_payload["chat"]["provider_chat_id"], json!(chat_id));
    assert_eq!(
        realtime_payload["chat"]["metadata"]["unread_count"],
        json!(1)
    );
    assert_eq!(
        realtime_payload["chat"]["metadata"]["mention_count"],
        json!(1)
    );
}

#[tokio::test]
async fn telegram_message_pin_route_records_local_projection_command_and_audit() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-message-pin-{suffix}");
    let chat_id = format!("chat-message-pin-{suffix}");
    let provider_message_id = format!("provider-message-pin-{suffix}");
    let command_id = format!("pin-message-{suffix}");
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
            "display_name": "Telegram Message Pin",
            "external_account_id": format!("tg-message-pin-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    let message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &chat_id,
        &provider_message_id,
        "Pin this Telegram message locally.",
        "2026-06-06T12:00:00Z",
    )
    .await;

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/provider-commands/messages/{message_id}/pin"),
            json!({
                "command_id": command_id,
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "is_pinned": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message pin response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["operation"], json!("pin"));
    assert_eq!(body["status"], json!("pinned"));

    let observation_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'signal.raw.telegram.message.pinned_state.observed'
          AND payload->>'message_id' = $1
        ORDER BY event_id DESC
        LIMIT 1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("pin observation event");
    assert_eq!(observation_payload["payload"]["is_pinned"], json!(true));
    assert_eq!(
        observation_payload["external_message_id"],
        json!(provider_message_id)
    );

    let command_row = sqlx::query(
        r#"
        SELECT command_kind, capability_state, action_class, payload
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("pin command row");
    let command_kind: String = command_row.try_get("command_kind").expect("command kind");
    let capability_state: String = command_row
        .try_get("capability_state")
        .expect("capability state");
    let action_class: String = command_row.try_get("action_class").expect("action class");
    let payload: Value = command_row.try_get("payload").expect("payload");
    assert_eq!(command_kind, "pin");
    assert_eq!(capability_state, "degraded");
    assert_eq!(action_class, "local_write");
    assert_eq!(payload["is_pinned"], json!(true));

    let audit_row = sqlx::query(
        r#"
        SELECT operation, metadata
        FROM api_audit_log
        WHERE target_id = $1
          AND operation = 'telegram.message.pin'
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("pin audit row");
    let audit_metadata: Value = audit_row.try_get("metadata").expect("audit metadata");
    assert_eq!(audit_metadata["action_class"], json!("local_write"));
    assert_eq!(audit_metadata["capability"], json!("telegram.message.pin"));
    assert_eq!(audit_metadata["operation"], json!("pin"));
    assert_eq!(audit_metadata["provider_chat_id"], json!(chat_id));

    let realtime_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'telegram.message.updated'
          AND subject->>'id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("telegram message updated event payload");
    assert_eq!(realtime_payload["is_pinned"], json!(true));
    assert_eq!(realtime_payload["message"]["message_id"], json!(message_id));
    assert_eq!(
        realtime_payload["message"]["provider_chat_id"],
        json!(chat_id)
    );
    assert!(
        realtime_payload["telegram_chat_id"]
            .as_str()
            .expect("telegram chat id")
            .starts_with("telegram_chat:v4:")
    );
}
