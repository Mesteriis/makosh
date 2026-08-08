use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "telegram-members-private-sync-secret";

#[tokio::test]
async fn telegram_private_members_sync_uses_tdlib_chat_metadata_and_records_audit() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-private-members-{suffix}");
    let provider_chat_id = format!("private-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::TelegramUser,
                format!("Telegram Private Members {suffix}"),
                "telegram:777".to_owned(),
            )
            .config(json!({"runtime": "tdlib_qr_authorized"})),
        )
        .await
        .expect("provider account");

    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("private-message-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Alice",
            "sender_id": "user:888",
            "sender_display_name": "Alice",
            "text": "hello",
            "import_batch_id": format!("telegram-private-members-seed-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let telegram_chat_id = first_chat_id(app.clone(), &account_id).await;
    sqlx::query(
        r#"
        UPDATE telegram_chats
        SET metadata = metadata || $2::jsonb
        WHERE telegram_chat_id = $1
        "#,
    )
    .bind(&telegram_chat_id)
    .bind(json!({
        "tdlib_private_user_id": "888",
        "tdlib_chat_type": "chatTypePrivate"
    }))
    .execute(&pool)
    .await
    .expect("update chat metadata");

    let sync_response = app
        .clone()
        .oneshot(json_post(
            &format!(
                "/api/v1/integrations/telegram/provider-sync/conversations/{telegram_chat_id}/members"
            ),
            json!({}),
        ))
        .await
        .expect("sync members response");
    assert_eq!(sync_response.status(), StatusCode::OK);
    let sync_body = json_body(sync_response).await;
    assert_eq!(sync_body["telegram_chat_id"], json!(telegram_chat_id));
    assert_eq!(sync_body["synced_count"], json!(1));
    assert_eq!(sync_body["items"][0]["source"], json!("tdlib"));
    assert_eq!(
        sync_body["items"][0]["provider_member_id"],
        json!("user:888")
    );
    assert_eq!(sync_body["items"][0]["sender_display_name"], json!("Alice"));
    assert_eq!(sync_body["items"][0]["role"], json!("member"));
    assert_eq!(sync_body["items"][0]["status"], json!("member"));
    assert_eq!(
        sync_body["items"][0]["permissions"]["observed_via"],
        json!("tdlib.chat.metadata")
    );

    let members_response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/conversations/{telegram_chat_id}/members?limit=10"
        )))
        .await
        .expect("members response");
    assert_eq!(members_response.status(), StatusCode::OK);
    let members_body = json_body(members_response).await;
    assert_eq!(members_body["items"][0]["source"], json!("tdlib"));
    assert_eq!(
        members_body["items"][0]["provider_member_id"],
        json!("user:888")
    );

    let audit_metadata: Value = sqlx::query_scalar(
        r#"
        SELECT metadata
        FROM api_audit_log
        WHERE operation = 'telegram.participants.sync'
          AND actor_id = 'makosh-frontend'
          AND target_id = $1
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind(&telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("participants sync audit metadata");
    assert_eq!(audit_metadata["action_class"], json!("read"));
    assert_eq!(
        audit_metadata["capability"],
        json!("telegram.participants.sync")
    );
    assert_eq!(audit_metadata["decision"], json!("allowed"));
    assert_eq!(
        audit_metadata["reason"],
        json!("explicit_user_confirmation")
    );
    assert_eq!(audit_metadata["account_id"], json!(account_id));
    assert_eq!(audit_metadata["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(audit_metadata["synced_count"], json!("1"));

    let participant_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'telegram.participant.updated'
          AND subject->>'kind' = 'telegram_chat_participant'
          AND subject->>'telegram_chat_id' = $1
        "#,
    )
    .bind(&telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("participant update event count");
    assert_eq!(participant_event_count, 1);

    let started_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'telegram.sync.started'
          AND payload->>'scope' = 'members'
          AND payload->>'provider_chat_id' = $2
          AND subject->>'id' = $1
        "#,
    )
    .bind(&telegram_chat_id)
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("members sync started count");
    assert_eq!(started_count, 1);

    let progress_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'telegram.sync.progress'
          AND payload->>'scope' = 'members'
          AND payload->>'provider_chat_id' = $2
          AND payload->>'status' = 'completed'
          AND subject->>'id' = $1
        "#,
    )
    .bind(&telegram_chat_id)
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("members sync progress count");
    assert_eq!(progress_count, 1);

    let completed_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'telegram.sync.completed'
          AND payload->>'scope' = 'members'
          AND payload->>'provider_chat_id' = $2
          AND payload->>'status' = 'completed'
          AND subject->>'id' = $1
        "#,
    )
    .bind(&telegram_chat_id)
    .bind(&provider_chat_id)
    .fetch_one(&pool)
    .await
    .expect("members sync completed count");
    assert_eq!(completed_count, 1);
}

async fn first_chat_id<S>(app: S, account_id: &str) -> String
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/conversations?account_id={account_id}&limit=10"
        )))
        .await
        .expect("chat list response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned()
}

async fn post_ok<S>(app: S, path: &str, body: Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post(path, body))
        .await
        .expect("post response");
    assert_eq!(response.status(), StatusCode::OK);
}

fn get(path: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::empty())
        .expect("request")
}

fn json_post(path: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1_000_000)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json body")
}

fn unique_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
        .to_string()
}
