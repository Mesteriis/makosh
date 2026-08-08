use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, header};
use chrono::Utc;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;

use makosh_hub_backend::platform::storage::database::Database;
use serde_json::{Value, json};

pub(crate) const T: &str = "v1comms-regression-test-token";

pub(crate) fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub(crate) fn delete(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn post_with_actor(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .header("x-makosh-actor-id", "makosh-frontend")
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub(crate) fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

pub(crate) async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

pub(crate) async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(T, database_url),
        database,
    )
}

pub(crate) async fn seed_projected_message(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "Seed Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw-{provider_record_id}"),
                account_id,
                "email_message",
                provider_record_id,
                format!("sha256:{provider_record_id}"),
                format!("batch-{provider_record_id}"),
                json!({
                    "subject": subject,
                    "from": "sender@example.com",
                    "to": ["recipient@example.com"],
                    "body_text": "Body for cursor pagination API"
                }),
            )
            .occurred_at(Utc::now()),
        )
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

pub(crate) async fn seed_projected_message_with_body(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "Thread Translate Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw-{provider_record_id}"),
                account_id,
                "email_message",
                provider_record_id,
                format!("sha256:{provider_record_id}"),
                format!("batch-{provider_record_id}"),
                json!({
                    "subject": subject,
                    "from": "sender@example.com",
                    "to": ["recipient@example.com"],
                    "body_text": body_text
                }),
            )
            .occurred_at(Utc::now()),
        )
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

pub(crate) async fn seed_projected_message_from_sender(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
    sender: &str,
    body_text: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "Paged Analytics Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw-{provider_record_id}"),
                account_id,
                "email_message",
                provider_record_id,
                format!("sha256:{provider_record_id}"),
                format!("batch-{provider_record_id}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": ["recipient@example.com"],
                    "body_text": body_text
                }),
            )
            .occurred_at(Utc::now()),
        )
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}
