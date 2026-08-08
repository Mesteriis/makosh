mod telegram_support;

use axum::http::StatusCode;
use chrono::Utc;
use serde_json::json;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::integrations::telegram::client::references;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, json_body, json_post_request_with_actor, unique_suffix,
};

#[tokio::test]
async fn telegram_reference_inserts_return_existing_rows_on_conflict() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-reference-idempotent-{suffix}");
    let chat_id = format!("reference-idempotent-chat-{suffix}");
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
            "display_name": "Telegram Reference Idempotency",
            "external_account_id": format!("tg-reference-idempotent-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let root_message_id = create_reference_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("reference-idempotent-root-{suffix}"),
        "Root Sender",
        "Root body",
        &suffix,
    )
    .await;
    let reply_message_id = create_reference_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("reference-idempotent-reply-{suffix}"),
        "Reply Sender",
        "Reply body",
        &suffix,
    )
    .await;
    let forward_message_id = create_reference_message(
        app,
        &account_id,
        &chat_id,
        &format!("reference-idempotent-forward-{suffix}"),
        "Forward Sender",
        "Forward body",
        &suffix,
    )
    .await;

    let pool = ctx.pool();
    let first_reply = references::insert_reply_ref(
        pool,
        &reply_message_id,
        &root_message_id,
        &account_id,
        &chat_id,
        &format!("reference-idempotent-reply-{suffix}"),
        &format!("reference-idempotent-root-{suffix}"),
        false,
    )
    .await
    .expect("first reply ref");
    let second_reply = references::insert_reply_ref(
        pool,
        &reply_message_id,
        &root_message_id,
        &account_id,
        &chat_id,
        &format!("reference-idempotent-reply-{suffix}"),
        &format!("reference-idempotent-root-{suffix}"),
        false,
    )
    .await
    .expect("second reply ref returns existing row");
    assert_eq!(second_reply.reply_ref_id, first_reply.reply_ref_id);

    let forward_date = chrono::DateTime::parse_from_rfc3339("2026-06-05T11:00:00Z")
        .expect("timestamp")
        .with_timezone(&Utc);
    let first_forward = references::insert_forward_ref(
        pool,
        &forward_message_id,
        &account_id,
        &chat_id,
        &format!("reference-idempotent-forward-{suffix}"),
        Some("origin-chat-idempotent"),
        Some("origin-message-idempotent"),
        Some("origin-sender-idempotent"),
        Some("Original Author"),
        Some(forward_date),
    )
    .await
    .expect("first forward ref");
    let second_forward = references::insert_forward_ref(
        pool,
        &forward_message_id,
        &account_id,
        &chat_id,
        &format!("reference-idempotent-forward-{suffix}"),
        Some("origin-chat-idempotent"),
        Some("origin-message-idempotent"),
        Some("origin-sender-idempotent"),
        Some("Original Author"),
        Some(forward_date),
    )
    .await
    .expect("second forward ref returns existing row");
    assert_eq!(second_forward.forward_ref_id, first_forward.forward_ref_id);
}

async fn create_reference_message(
    app: axum::Router,
    account_id: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    sender_display_name: &str,
    text: &str,
    suffix: &str,
) -> String {
    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "chat_kind": "group",
                "chat_title": "Reference Idempotency Room",
                "sender_id": format!("sender-{sender_display_name}-{suffix}"),
                "sender_display_name": sender_display_name,
                "text": text,
                "import_batch_id": format!("telegram-reference-idempotent-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reference message response");
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await["message_id"]
        .as_str()
        .expect("message id")
        .to_owned()
}
