use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
mod telegram_support;

use axum::http::StatusCode;
use chrono::Utc;
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::messages::provider_observation_projection::consume_accepted_signal_event;
use makosh_hub_backend::domains::signal_hub::telegram::dispatch_telegram_raw_signal;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, json_body, json_post_request_with_actor, unique_suffix,
};
#[tokio::test]
async fn telegram_tdlib_projection_accepts_media_message_without_text() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("telegram-empty-media-{suffix}");
    let provider_chat_id = format!("-100{suffix}");
    let provider_message_id = format!("{provider_chat_id}:87403003904");

    communication_store
        .upsert_provider_account(
            &NewProviderAccount::new(
                &account_id,
                CommunicationProviderKind::TelegramUser,
                "Telegram Empty Media",
                format!("tg-empty-media-{suffix}"),
            )
            .config(json!({"runtime": "tdlib_qr_authorized"})),
        )
        .await
        .expect("provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw:telegram-empty-media:{suffix}"),
                &account_id,
                "telegram_message",
                &provider_message_id,
                format!("sha256:{suffix}"),
                format!("telegram-tdlib-history:{account_id}:{provider_chat_id}"),
                json!({
                    "provider_chat_id": provider_chat_id,
                    "chat_title": "Media Channel",
                    "chat_kind": "channel",
                    "sender_id": format!("chat:{provider_chat_id}"),
                    "sender_display_name": "Media Channel",
                    "text": "",
                    "delivery_state": "received",
                    "tdlib_raw": {
                        "@type": "message",
                        "id": 87403003904_i64,
                        "chat_id": provider_chat_id,
                        "content": {"@type": "messagePhoto"}
                    }
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "runtime": "tdlib",
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
            })),
        )
        .await
        .expect("raw source");

    let accepted_event = dispatch_telegram_raw_signal(pool.clone(), &raw)
        .await
        .expect("dispatch empty media telegram signal")
        .expect("accepted empty media telegram signal");
    let projected = consume_accepted_signal_event(pool.clone(), &accepted_event)
        .await
        .expect("project accepted empty media signal")
        .expect("projected empty media message");

    assert_eq!(projected.provider_record_id, provider_message_id);
    assert_eq!(projected.conversation_id, Some(provider_chat_id));
    assert_eq!(projected.body_text, "");
    assert_eq!(
        projected.message_metadata["tdlib_raw"]["content"]["@type"],
        json!("messagePhoto")
    );
}

#[tokio::test]
async fn telegram_fixture_media_download_fails_closed_without_live_runtime() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-media-{suffix}");
    let chat_id = format!("media-chat-{suffix}");
    let provider_message_id = format!("media-message-{suffix}");
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
            "display_name": "Telegram Media",
            "external_account_id": format!("tg-media-{suffix}"),
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
            "provider_chat_id": chat_id,
            "provider_message_id": provider_message_id,
            "chat_kind": "private",
            "chat_title": "Media Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Document metadata only.",
            "import_batch_id": format!("telegram-fixture-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let download_response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/provider-media/download",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "tdlib_file_id": 42,
                "provider_attachment_id": "tdlib-file:42",
                "filename": "document.pdf",
                "content_type": "application/pdf"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("download response");

    assert_eq!(download_response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(download_response).await;
    assert_eq!(body["error"], json!("invalid_telegram_request"));
    assert!(
        body["message"]
            .as_str()
            .expect("message")
            .contains("Telegram media downloads require an enabled TDLib actor")
    );

    let media_events: Vec<(String, Value)> = sqlx::query_as(
        r#"
        SELECT event_type, payload
        FROM event_log
        WHERE event_type IN (
            'telegram.media.download.started',
            'telegram.media.download.failed'
        )
          AND payload->>'provider_message_id' = $1
        ORDER BY position ASC
        "#,
    )
    .bind(&provider_message_id)
    .fetch_all(&pool)
    .await
    .expect("media download lifecycle events");
    assert_eq!(media_events.len(), 2);
    assert_eq!(media_events[0].0, "telegram.media.download.started");
    assert_eq!(media_events[0].1["download_state"], json!("requested"));
    assert_eq!(media_events[0].1["tdlib_file_id"], json!(42));
    assert_eq!(media_events[1].0, "telegram.media.download.failed");
    assert_eq!(media_events[1].1["download_state"], json!("failed"));
    assert!(
        media_events[1].1["error"]
            .as_str()
            .expect("failed event error")
            .contains("Telegram media downloads require an enabled TDLib actor")
    );
}
