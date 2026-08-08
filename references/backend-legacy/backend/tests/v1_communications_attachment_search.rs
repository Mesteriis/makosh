use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::communications::storage::models::{
    CommunicationAttachmentDisposition, NewCommunicationAttachment, NewCommunicationBlob,
};
use makosh_hub_backend::domains::communications::storage::scanner::{
    AttachmentSafetyScanReport, AttachmentSafetyScanStatus,
};
use makosh_hub_backend::domains::communications::storage::store::CommunicationStorageStore;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::storage::database::Database;

const T: &str = "v1comms-attachment-search-test-token";

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(T, database_url),
        database,
    )
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("request")
}

#[tokio::test]
async fn v1_attachment_search_filters_and_paginates_metadata_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-attachment-search-{suffix}");
    let first_message_id = seed_message_with_attachment(
        pool.clone(),
        SeedAttachmentMessage {
            account_id: account_id.clone(),
            provider_record_id: format!("provider-attachment-search-{suffix}-1"),
            subject: "Invoice Q1".to_owned(),
            filename: "invoice-q1.pdf".to_owned(),
            content_type: "application/pdf".to_owned(),
            hex_digit: "a".to_owned(),
            scan_status: AttachmentSafetyScanStatus::NotScanned,
        },
    )
    .await;
    let second_message_id = seed_message_with_attachment(
        pool,
        SeedAttachmentMessage {
            account_id: account_id.clone(),
            provider_record_id: format!("provider-attachment-search-{suffix}-2"),
            subject: "Invoice Q2".to_owned(),
            filename: "invoice-q2.xlsx".to_owned(),
            content_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                .to_owned(),
            hex_digit: "b".to_owned(),
            scan_status: AttachmentSafetyScanStatus::Failed,
        },
    )
    .await;

    let app = router(&context.connection_string()).await;
    let response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/search?account_id={account_id}&q=invoice&limit=1"
        )))
        .await
        .expect("search response");
    assert_eq!(response.status(), StatusCode::OK);
    let first_page = response_json(response).await;
    assert_eq!(first_page["items"].as_array().expect("items").len(), 1);
    assert_eq!(first_page["has_more"], true);
    let next_cursor = first_page["next_cursor"]
        .as_str()
        .expect("next cursor")
        .to_owned();
    assert_eq!(first_page["items"][0]["filename"], "invoice-q2.xlsx");
    assert_eq!(first_page["items"][0]["message_id"], second_message_id);
    assert_eq!(first_page["items"][0]["message_subject"], "Invoice Q2");
    assert_eq!(first_page["items"][0]["storage_kind"], "local_fs");
    assert!(
        first_page["items"][0]["storage_path"]
            .as_str()
            .expect("storage path")
            .contains("invoice-q2")
    );

    let response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/search?account_id={account_id}&q=invoice&limit=1&cursor={next_cursor}"
        )))
        .await
        .expect("second search response");
    assert_eq!(response.status(), StatusCode::OK);
    let second_page = response_json(response).await;
    assert_eq!(second_page["items"].as_array().expect("items").len(), 1);
    assert_eq!(second_page["has_more"], false);
    assert_eq!(second_page["next_cursor"], Value::Null);
    assert_eq!(second_page["items"][0]["filename"], "invoice-q1.pdf");
    assert_eq!(second_page["items"][0]["message_id"], first_message_id);

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/search?account_id={account_id}&content_type=pdf&scan_status=not_scanned"
        )))
        .await
        .expect("filtered search response");
    assert_eq!(response.status(), StatusCode::OK);
    let filtered = response_json(response).await;
    assert_eq!(filtered["items"].as_array().expect("items").len(), 1);
    assert_eq!(filtered["items"][0]["filename"], "invoice-q1.pdf");
}

struct SeedAttachmentMessage {
    account_id: String,
    provider_record_id: String,
    subject: String,
    filename: String,
    content_type: String,
    hex_digit: String,
    scan_status: AttachmentSafetyScanStatus,
}

async fn seed_message_with_attachment(pool: sqlx::PgPool, seed: SeedAttachmentMessage) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let storage_store = CommunicationStorageStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &seed.account_id,
            CommunicationProviderKind::Gmail,
            "Attachment Search Gmail",
            format!("{}@example.com", seed.account_id),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{}", seed.provider_record_id),
            &seed.account_id,
            "email_message",
            &seed.provider_record_id,
            format!("sha256:{:0>64}", seed.hex_digit),
            format!("batch-{}", seed.provider_record_id),
            json!({
                "subject": seed.subject,
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Body for attachment search API"
            }),
        ))
        .await
        .expect("record raw source");
    let message_id = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id;
    let sha256 = format!("sha256:{:0>64}", seed.hex_digit);
    let blob = storage_store
        .upsert_blob(
            &NewCommunicationBlob::new(
                "local_fs",
                format!("attachments/{}/{}", seed.provider_record_id, seed.filename),
                &sha256,
                1024,
            )
            .content_type(&seed.content_type),
        )
        .await
        .expect("store blob");
    storage_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message_id,
                &raw.raw_record_id,
                blob.blob_id,
                format!("part-{}", seed.filename),
                &seed.content_type,
                1024,
                sha256,
            )
            .filename(&seed.filename)
            .disposition(CommunicationAttachmentDisposition::Attachment)
            .scan_report(AttachmentSafetyScanReport {
                status: seed.scan_status,
                engine: None,
                checked_at: None,
                summary: None,
                metadata: json!({}),
            }),
        )
        .await
        .expect("store attachment");
    message_id
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
