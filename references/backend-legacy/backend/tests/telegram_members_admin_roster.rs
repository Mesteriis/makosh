use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::query;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "telegram-members-admin-test-secret";

#[tokio::test]
async fn members_route_returns_admin_only_provider_roster_rows() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database.clone(),
    );
    let pool = database.pool().expect("configured pool").clone();

    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": "acct-1",
            "provider_kind": "telegram_user",
            "display_name": "Telegram Member Admin Roster",
            "external_account_id": "telegram:12345",
            "tdlib_data_path": "docker/data/telegram/test-members-admin-roster",
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
            "chat_title": "Admin Roster Room",
            "sender_id": "sender-1",
            "sender_display_name": "Sender",
            "text": "seed chat",
            "import_batch_id": "seed-batch-1",
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let response = app
        .clone()
        .oneshot(get(
            "/api/v1/communications/conversations?account_id=acct-1&limit=10",
        ))
        .await
        .expect("chat list response");
    let body = json_body(response).await;
    let telegram_chat_id = body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned();

    query(
        r#"
        INSERT INTO telegram_chat_participants (
            participant_id, telegram_chat_id, account_id, provider_chat_id, provider_member_id,
            display_name, username, role, status, is_admin, is_owner, permissions, raw_payload,
            source, observed_at, created_at, updated_at
        )
        VALUES
            ('participant-1', $1, 'acct-1', 'provider-chat-1', 'user:1', 'Recent Member', NULL, 'member', 'member', false, false, '{"observed_via":"tdlib.getSupergroupMembers"}'::jsonb, '{}'::jsonb, 'tdlib', NOW(), NOW(), NOW()),
            ('participant-2', $1, 'acct-1', 'provider-chat-1', 'user:2', 'Admin Only', 'admin_only', 'admin', 'administrator', true, false, '{"observed_via":"tdlib.getSupergroupMembers.administrators","can_invite_users":true}'::jsonb, '{}'::jsonb, 'tdlib', NOW(), NOW(), NOW())
        "#,
    )
    .bind(&telegram_chat_id)
    .execute(&pool)
    .await
    .expect("insert participants");

    let members = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/conversations/{telegram_chat_id}/members?limit=10"
        )))
        .await
        .expect("members response");
    let body = json_body(members).await;
    let items = body["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["provider_member_id"], "user:2");
    assert_eq!(items[0]["role"], "admin");
    assert_eq!(items[0]["status"], "administrator");
    assert_eq!(items[0]["is_admin"], true);
    assert_eq!(
        items[0]["permissions"]["observed_via"],
        "tdlib.getSupergroupMembers.administrators"
    );
    assert_eq!(items[0]["permissions"]["can_invite_users"], true);
    assert_eq!(items[1]["provider_member_id"], "user:1");
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("X-Макошь-Secret", LOCAL_API_TOKEN)
        .body(Body::empty())
        .expect("request")
}

async fn post_ok<S>(app: S, uri: &str, body: Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-Макошь-Secret", LOCAL_API_TOKEN)
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json body")
}
