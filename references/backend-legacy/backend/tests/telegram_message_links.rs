use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::messages::provider_channel_store::ProviderChannelMessageStore;
use makosh_hub_backend::integrations::telegram::client::models::chats::TelegramSyncState;
use makosh_hub_backend::integrations::telegram::client::models::chats::{
    NewTelegramChat, TelegramChatKind,
};
use makosh_hub_backend::integrations::telegram::client::store::TelegramStore;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "telegram-message-link-test-secret";

#[tokio::test]
async fn telegram_message_ingestion_projects_public_message_link_without_erasing_chat_username() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-link-{suffix}");
    let chat_id = format!("100{suffix}");
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
            "display_name": "Telegram Link Projection",
            "external_account_id": format!("tg-link-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/link-{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let store = telegram_store(&pool);
    let public_chat = store
        .upsert_chat(&NewTelegramChat {
            account_id: account_id.clone(),
            provider_chat_id: chat_id.clone(),
            chat_kind: TelegramChatKind::Channel,
            title: "Public Link Channel".to_owned(),
            username: Some("МакошьPublicChannel".to_owned()),
            sync_state: TelegramSyncState::Synced,
            last_message_at: None,
            metadata: json!({"runtime": "tdlib"}),
        })
        .await
        .expect("public chat");

    let result = assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id.clone(),
            "provider_chat_id": chat_id.clone(),
            "provider_message_id": format!("{chat_id}:4242"),
            "chat_kind": "channel",
            "chat_title": public_chat.title.clone(),
            "sender_id": "sender-link",
            "sender_display_name": "Link Sender",
            "text": "Public channel message with stable provider permalink.",
            "import_batch_id": format!("telegram-link-fixture-{suffix}"),
            "occurred_at": "2026-06-19T10:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let chats_after_ingest = store
        .list_chats(Some(&account_id), 10)
        .await
        .expect("chat lookup");
    let chat_after_ingest = chats_after_ingest
        .iter()
        .find(|chat| chat.provider_chat_id == chat_id)
        .expect("chat row");
    assert_eq!(
        chat_after_ingest.username.as_deref(),
        Some("МакошьPublicChannel")
    );
    let chat_observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'telegram'
          AND link.entity_kind = 'chat'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&public_chat.telegram_chat_id)
    .fetch_all(&pool)
    .await
    .expect("chat observations");
    assert!(
        chat_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_CHAT"
                && row.get::<String, _>("relationship_kind") == "upsert"
                && row.get::<Value, _>("payload")["username"] == json!("МакошьPublicChannel")
        }),
        "chat upsert observation must exist"
    );

    let message = store
        .message_by_id(result["message_id"].as_str().expect("message_id"))
        .await
        .expect("message lookup")
        .expect("projected message");
    assert_eq!(
        message.metadata["message_link"],
        json!("https://t.me/МакошьPublicChannel/4242")
    );
    assert_eq!(message.metadata["message_link_kind"], json!("public_t_me"));
}

async fn assert_ok<S>(app: S, path: &str, body: Value) -> Value
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post_request(path, body, LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&bytes).expect("json response")
}

fn json_post_request(path: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-makosh-secret", token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn unique_suffix() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    format!("{now}")
}

fn telegram_store(pool: &sqlx::PgPool) -> TelegramStore {
    TelegramStore::new(
        pool.clone(),
        Arc::new(CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(CommunicationProviderSecretBindingStore::new(pool.clone())),
        Arc::new(ProviderChannelMessageStore::new(pool.clone())),
        Arc::new(
            makosh_communications_postgres::store::CommunicationIngestionStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            makosh_hub_backend::platform::communications::EventStoreProviderMessageObservationEventPort::new(
                pool.clone(),
            ),
        ),
    )
}
