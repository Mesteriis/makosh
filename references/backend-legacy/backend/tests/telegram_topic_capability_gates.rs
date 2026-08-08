use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::query;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "telegram-topic-capability-gates-secret";

#[tokio::test]
async fn fixture_account_allows_topic_list_but_blocks_topic_writes() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": "acct-1",
            "provider_kind": "telegram_user",
            "display_name": "Telegram Fixture User",
            "external_account_id": "telegram:12345",
            "tdlib_data_path": "docker/data/telegram/test-topic-capability-gates",
            "transcription_enabled": false
        }),
    )
    .await;
    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": "acct-1",
            "provider_chat_id": "provider-chat-1",
            "provider_message_id": "seed-message-1",
            "chat_kind": "group",
            "chat_title": "Capability Room",
            "sender_id": "sender-1",
            "sender_display_name": "Sender",
            "text": "seed chat",
            "import_batch_id": "seed-batch-1",
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let telegram_chat_id = first_chat_id(app.clone()).await;
    query(
        r#"
        INSERT INTO telegram_topics (
            topic_id, telegram_chat_id, account_id, provider_topic_id, provider_chat_id,
            title, icon_emoji, is_pinned, is_closed, unread_count, metadata, created_at, updated_at
        )
        VALUES (
            'topic-1', $1, 'acct-1', 101, 'provider-chat-1',
            'General', NULL, false, false, 0, '{}'::jsonb, NOW(), NOW()
        )
        "#,
    )
    .bind(&telegram_chat_id)
    .execute(&pool)
    .await
    .expect("insert topic");

    let list_response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/conversations/{telegram_chat_id}/topics?limit=10"
        )))
        .await
        .expect("topics list response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = json_body(list_response).await;
    assert_eq!(list_body["items"].as_array().expect("items").len(), 1);
    assert_eq!(list_body["items"][0]["topic_id"], "topic-1");

    let create_response = app
        .clone()
        .oneshot(json_post(
            &format!("/api/v1/communications/conversations/{telegram_chat_id}/topics"),
            json!({
                "account_id": "acct-1",
                "provider_chat_id": "provider-chat-1",
                "title": "New Topic",
                "command_id": "cmd-topic-create-1"
            }),
        ))
        .await
        .expect("topic create response");
    assert_eq!(create_response.status(), StatusCode::BAD_REQUEST);

    let close_response = app
        .clone()
        .oneshot(json_post(
            "/api/v1/integrations/telegram/provider-commands/topics/topic-1/close",
            json!({
                "account_id": "acct-1",
                "provider_chat_id": "provider-chat-1",
                "is_closed": true,
                "command_id": "cmd-topic-close-1"
            }),
        ))
        .await
        .expect("topic close response");
    assert_eq!(close_response.status(), StatusCode::BAD_REQUEST);

    let command_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_provider_write_commands WHERE account_id = 'acct-1' AND command_kind IN ('topic_create', 'topic_close', 'topic_reopen')",
    )
    .fetch_one(&pool)
    .await
    .expect("command count");
    assert_eq!(command_count, 0);
}

async fn first_chat_id<S>(app: S) -> String
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(get(
            "/api/v1/communications/conversations?account_id=acct-1&limit=10",
        ))
        .await
        .expect("chat list response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned()
}

async fn post_ok<S>(app: S, uri: &str, body: Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app.oneshot(json_post(uri, body)).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("X-Макошь-Secret", LOCAL_API_TOKEN)
        .body(Body::empty())
        .expect("request")
}

fn json_post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-Макошь-Secret", LOCAL_API_TOKEN)
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json body")
}
