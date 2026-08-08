mod telegram_support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{LOCAL_API_TOKEN, json_body, json_post_request_with_actor, unique_suffix};

#[tokio::test]
async fn telegram_folder_add_action_records_provider_write_command() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-folder-action-{suffix}");
    let provider_chat_id = format!("folder-chat-{suffix}");
    let provider_folder_id = 7_i64;
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram Folder Action",
                "external_account_id": format!("tg-folder-{suffix}"),
                "api_id": 1,
                "api_hash": "test-api-hash",
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "qr_authorized": true,
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": format!("{provider_chat_id}:1"),
                "chat_kind": "private",
                "chat_title": "Folder Action Chat",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Folder Owner",
                "text": "Folder action should create a durable provider-write command.",
                "import_batch_id": format!("telegram-folder-action-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);

    let chats_response = app
        .clone()
        .oneshot(telegram_support::get_request_with_token(
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
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}"),
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("folder add response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["action"], json!("folder_add"));
    assert_eq!(body["status"], json!("queued"));

    let command_id = body["command_id"].as_str().expect("command id");
    let row = sqlx::query(
        r#"
        SELECT command_kind, provider_chat_id, payload, action_class, status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("folder command row");

    let command_kind: String = row.get("command_kind");
    let stored_provider_chat_id: String = row.get("provider_chat_id");
    let action_class: String = row.get("action_class");
    let status: String = row.get("status");
    let payload: serde_json::Value = row.get("payload");

    assert_eq!(command_kind, "folder_add");
    assert_eq!(stored_provider_chat_id, provider_chat_id);
    assert_eq!(action_class, "provider_write");
    assert_eq!(status, "queued");
    assert_eq!(payload["provider_folder_id"], json!(provider_folder_id));
}

#[tokio::test]
async fn telegram_folder_remove_action_records_provider_write_command() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-folder-remove-action-{suffix}");
    let provider_chat_id = format!("folder-remove-chat-{suffix}");
    let provider_folder_id = 11_i64;
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram Folder Remove Action",
                "external_account_id": format!("tg-folder-remove-{suffix}"),
                "api_id": 1,
                "api_hash": "test-api-hash",
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "qr_authorized": true,
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": format!("{provider_chat_id}:1"),
                "chat_kind": "private",
                "chat_title": "Folder Remove Action Chat",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Folder Owner",
                "text": "Folder remove should create a durable provider-write command.",
                "import_batch_id": format!("telegram-folder-remove-action-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);

    let chats_response = app
        .clone()
        .oneshot(telegram_support::get_request_with_token(
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
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/{provider_folder_id}/remove"
            ),
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("folder remove response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["action"], json!("folder_remove"));
    assert_eq!(body["status"], json!("queued"));

    let command_id = body["command_id"].as_str().expect("command id");
    let row = sqlx::query(
        r#"
        SELECT command_kind, provider_chat_id, payload, action_class, status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("folder remove command row");

    let command_kind: String = row.get("command_kind");
    let stored_provider_chat_id: String = row.get("provider_chat_id");
    let action_class: String = row.get("action_class");
    let status: String = row.get("status");
    let payload: serde_json::Value = row.get("payload");

    assert_eq!(command_kind, "folder_remove");
    assert_eq!(stored_provider_chat_id, provider_chat_id);
    assert_eq!(action_class, "provider_write");
    assert_eq!(status, "queued");
    assert_eq!(payload["provider_folder_id"], json!(provider_folder_id));
}

#[tokio::test]
async fn telegram_folder_reassign_action_queues_add_and_remove_commands_from_current_membership() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-folder-reassign-action-{suffix}");
    let provider_chat_id = format!("folder-reassign-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram Folder Reassign Action",
                "external_account_id": format!("tg-folder-reassign-{suffix}"),
                "api_id": 1,
                "api_hash": "test-api-hash",
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "qr_authorized": true,
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": format!("{provider_chat_id}:1"),
                "chat_kind": "private",
                "chat_title": "Folder Reassign Action Chat",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Folder Owner",
                "text": "Folder reassignment should queue add/remove commands from current membership.",
                "import_batch_id": format!("telegram-folder-reassign-action-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);

    let chats_response = app
        .clone()
        .oneshot(telegram_support::get_request_with_token(
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

    sqlx::query(
        r#"
        UPDATE telegram_chats
        SET metadata = jsonb_build_object(
            'provider_folder_id', 7,
            'folder_labels', jsonb_build_array('Team', 'Archive'),
            'tdlib_chat_positions', jsonb_build_object('folder_ids', jsonb_build_array(7, 9))
        )
        WHERE telegram_chat_id = $1
        "#,
    )
    .bind(&telegram_chat_id)
    .execute(&pool)
    .await
    .expect("seed chat folder metadata");

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/folders/reassign"
            ),
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "target_provider_folder_ids": [11]
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("folder reassign response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["action"], json!("folder_reassign"));
    assert_eq!(body["status"], json!("queued"));
    assert_eq!(body["added_provider_folder_ids"], json!([11]));
    assert_eq!(body["removed_provider_folder_ids"], json!([7, 9]));
    assert_eq!(body["command_ids"].as_array().map(Vec::len), Some(3));

    let command_rows = sqlx::query(
        r#"
        SELECT command_kind, payload
        FROM telegram_provider_write_commands
        WHERE command_id = ANY($1)
        ORDER BY created_at ASC
        "#,
    )
    .bind(
        body["command_ids"]
            .as_array()
            .expect("command ids")
            .iter()
            .map(|value| value.as_str().expect("command id").to_owned())
            .collect::<Vec<_>>(),
    )
    .fetch_all(&pool)
    .await
    .expect("folder reassign command rows");

    assert_eq!(command_rows.len(), 3);
    let command_kinds = command_rows
        .iter()
        .map(|row| row.get::<String, _>("command_kind"))
        .collect::<Vec<_>>();
    assert_eq!(
        command_kinds,
        vec!["folder_add", "folder_remove", "folder_remove"]
    );
    let payloads = command_rows
        .iter()
        .map(|row| row.get::<serde_json::Value, _>("payload"))
        .collect::<Vec<_>>();
    assert_eq!(payloads[0]["provider_folder_id"], json!(11));
    assert_eq!(payloads[1]["provider_folder_id"], json!(7));
    assert_eq!(payloads[2]["provider_folder_id"], json!(9));
    assert_eq!(
        payloads[0]["source"],
        json!("telegram_chat_folder_reassign")
    );
}
