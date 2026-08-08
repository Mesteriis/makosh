use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::messages::models::NewProjectedMessage;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::storage::database::Database;

const TOKEN: &str = "message-flags-api-test-token";

async fn app(ctx: &TestContext) -> axum::Router {
    let database = Database::connect(Some(&ctx.connection_string()))
        .await
        .expect("database");
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            TOKEN,
            ctx.connection_string().as_str(),
        ),
        database,
    )
}

fn request(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("x-makosh-secret", TOKEN)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("response body"),
    )
    .expect("json")
}

#[tokio::test]
async fn message_important_endpoint_toggles_metadata_flag() {
    let ctx = TestContext::new().await;
    let communication_store = CommunicationIngestionStore::new(ctx.pool().clone());
    let message_store = MessageProjectionStore::new(ctx.pool().clone());
    let suffix = unique_suffix();
    let account_id = format!("acct-important-{suffix}");
    let raw_record_id = format!("raw-important-{suffix}");
    let provider_record_id = format!("provider-important-{suffix}");

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Important IMAP",
            format!("important-{suffix}@example.com"),
        ))
        .await
        .expect("account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:{raw_record_id}"),
                format!("batch_{raw_record_id}"),
                json!({
                    "subject": "Important subject",
                    "from": "alice@example.com",
                    "to": ["bob@example.com"],
                    "body_text": "Important body"
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"message_flags_api_test"})),
        )
        .await
        .expect("raw record");

    let projected = message_store
        .upsert_message(&NewProjectedMessage {
            message_id: format!("message-important-{suffix}"),
            raw_record_id: raw.raw_record_id,
            account_id: account_id.clone(),
            provider_record_id,
            subject: "Important subject".to_owned(),
            sender: "alice@example.com".to_owned(),
            recipients: vec!["bob@example.com".to_owned()],
            body_text: "Important body".to_owned(),
            occurred_at: raw.occurred_at,
            channel_kind: "email".to_owned(),
            conversation_id: None,
            sender_display_name: Some("alice@example.com".to_owned()),
            delivery_state: "received".to_owned(),
            message_metadata: json!({}),
        })
        .await
        .expect("message");
    let message_id = projected.message_id;
    message_store
        .set_read_state(&message_id, true, "local_user")
        .await
        .expect("mark message read before metadata mutations");

    let app = app(&ctx).await;
    let uri = format!("/api/v1/communications/messages/{message_id}/important");

    let response = app
        .clone()
        .oneshot(request(Method::POST, &uri))
        .await
        .expect("important response");
    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["message_id"], message_id);
    assert_eq!(body["important"], true);
    let stored = message_store
        .message(&message_id)
        .await
        .expect("stored message")
        .expect("message exists");
    assert_eq!(stored.message_metadata["important"], true);
    assert!(stored.is_read);
    let first_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'communication_message'
           AND entity_id = $1
           AND relationship_kind = 'message_flag_update'
         ORDER BY created_at ASC
         LIMIT 1",
    )
    .bind(&message_id)
    .fetch_one(ctx.pool())
    .await
    .expect("first observation link");
    let first_observation_id: String = first_link
        .try_get("observation_id")
        .expect("first observation id");
    let first_metadata: Value = first_link.try_get("metadata").expect("first metadata");
    assert_eq!(first_metadata["important"], true);
    let first_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&first_observation_id)
    .fetch_one(ctx.pool())
    .await
    .expect("first observation");
    let first_origin_kind: String = first_observation
        .try_get("origin_kind")
        .expect("first origin kind");
    let first_payload: Value = first_observation.try_get("payload").expect("first payload");
    assert_eq!(first_origin_kind, "manual");
    assert_eq!(first_payload["operation"], "message_important_toggle");
    assert_eq!(first_payload["message_id"], message_id);

    let response = app
        .oneshot(request(Method::POST, &uri))
        .await
        .expect("second important response");
    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["message_id"], message_id);
    assert_eq!(body["important"], false);
    let stored = message_store
        .message(&message_id)
        .await
        .expect("stored message")
        .expect("message exists");
    assert_eq!(stored.message_metadata["important"], false);
    assert!(stored.is_read);
    let links_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)
             FROM observation_links
             WHERE domain = 'communications'
               AND entity_kind = 'communication_message'
               AND entity_id = $1
               AND relationship_kind = 'message_flag_update'",
    )
    .bind(&message_id)
    .fetch_one(ctx.pool())
    .await
    .expect("message flag observation count");
    assert_eq!(links_count, 2);
    let provider_command_kinds = sqlx::query_scalar::<_, String>(
        "SELECT command_kind
         FROM communication_provider_commands
         WHERE account_id = $1 AND channel_kind = 'mail'
         ORDER BY created_at ASC, command_id ASC",
    )
    .bind(&account_id)
    .fetch_all(ctx.pool())
    .await
    .expect("important provider commands");
    assert_eq!(provider_command_kinds, vec!["important", "not_important"]);
}

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
        .to_string()
}
