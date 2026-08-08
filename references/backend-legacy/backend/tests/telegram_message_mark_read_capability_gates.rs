mod telegram_support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, json_body, json_post_request_with_actor,
    unique_suffix,
};

#[tokio::test]
async fn fixture_account_blocks_message_mark_read_before_side_effects() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-message-read-{suffix}");
    let provider_chat_id = format!("message-read-chat-{suffix}");
    let provider_message_id = format!("{provider_chat_id}:9001");
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
            "display_name": "Telegram Message Read",
            "external_account_id": format!("tg-message-read-{suffix}"),
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
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "chat_kind": "private",
                "chat_title": "Message Read Chat",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Maria Petrova",
                "text": "Mark this message as read through provider state.",
                "import_batch_id": format!("telegram-message-read-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_result = json_body(message_response).await;
    let message_id = message_result["message_id"]
        .as_str()
        .expect("message id")
        .to_owned();

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
    let initial_unread_count = chats_body["items"][0]["metadata"]["unread_count"].clone();
    let initial_mention_count = chats_body["items"][0]["metadata"]["mention_count"].clone();

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/telegram/provider-commands/messages/{message_id}/mark-read"
            ),
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message mark-read response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let command_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_provider_write_commands WHERE account_id = $1 AND command_kind = 'mark_read'",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("mark-read command count");
    assert_eq!(command_count, 0);

    let command_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'telegram.command.status_changed' AND payload->>'command_kind' = 'mark_read' AND payload->>'message_id' = $1",
    )
    .bind(&provider_message_id)
    .fetch_one(&pool)
    .await
    .expect("message mark-read command event count");
    assert_eq!(command_event_count, 0);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM api_audit_log WHERE operation = 'telegram.message.mark_read' AND target_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message mark-read audit count");
    assert_eq!(audit_count, 0);

    let chat_row: Value = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT jsonb_build_object(
            'last_read_inbox_provider_message_id', metadata->>'last_read_inbox_provider_message_id',
            'unread_count', metadata->'unread_count',
            'mention_count', metadata->'mention_count'
        )
        FROM telegram_chats
        WHERE telegram_chat_id = $1
        "#,
    )
    .bind(&telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("chat metadata after blocked mark-read");
    assert_eq!(chat_row["last_read_inbox_provider_message_id"], Value::Null);
    assert_eq!(chat_row["unread_count"], initial_unread_count);
    assert_eq!(chat_row["mention_count"], initial_mention_count);
}
