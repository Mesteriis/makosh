mod telegram_support;

use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, json_body, json_post_request_with_actor,
    unique_suffix,
};

#[tokio::test]
async fn removed_account_blocks_manual_send_before_message_audit_and_events() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-send-gates-{suffix}");
    let provider_chat_id = format!("send-gates-chat-{suffix}");
    let command_id = format!("send-{suffix}");
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
            "display_name": "Telegram Send Gates",
            "external_account_id": format!("tg-send-gates-{suffix}"),
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
            "provider_message_id": format!("incoming-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Manual Send Gate Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Can Макошь still send after account removal?",
            "import_batch_id": format!("telegram-send-gates-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let before_count = message_count(app.clone(), &account_id, &provider_chat_id).await;

    let remove_response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/integrations/telegram/accounts/{account_id}"
                ))
                .header("x-makosh-secret", LOCAL_API_TOKEN)
                .body(axum::body::Body::empty())
                .expect("delete request"),
        )
        .await
        .expect("remove account response");
    assert_eq!(remove_response.status(), StatusCode::OK);

    let send_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/provider-commands/messages/send",
            json!({
                "command_id": command_id,
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "text": "This send must be blocked by capability gate."
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("send response");
    assert_eq!(send_response.status(), StatusCode::BAD_REQUEST);

    let after_count = message_count(app.clone(), &account_id, &provider_chat_id).await;
    assert_eq!(after_count, before_count);

    let send_audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM api_audit_log WHERE operation = 'telegram.message.send' AND metadata->>'account_id' = $1 AND metadata->>'provider_chat_id' = $2",
    )
    .bind(&account_id)
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("send audit count");
    assert_eq!(send_audit_count, 0);

    let created_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'telegram.message.created' AND payload->>'provider_chat_id' = $1",
    )
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("created event count");
    assert_eq!(created_event_count, 1);

    let command_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'telegram.command.status_changed' AND payload->>'provider_chat_id' = $1",
    )
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("command event count");
    assert_eq!(command_event_count, 0);
}

async fn message_count<S>(app: S, account_id: &str, provider_chat_id: &str) -> usize
where
    S: tower::Service<axum::http::Request<axum::body::Body>, Response = axum::response::Response>
        + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/messages?account_id={account_id}&conversation_id={provider_chat_id}"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("messages response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    body["items"].as_array().expect("message items").len()
}
