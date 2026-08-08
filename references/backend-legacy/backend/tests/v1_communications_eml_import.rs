use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::storage::store::CommunicationStorageStore;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "v1comms-eml-import-test-token";

#[tokio::test]
async fn v1_eml_import_preserves_evidence_attachments_and_idempotency() {
    let context = TestContext::new().await;
    let account_id = "acct-eml-import";
    CommunicationIngestionStore::new(context.pool().clone())
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "EML Import",
            "eml-import@example.com",
        ))
        .await
        .expect("store provider account");
    let app = router(&context.connection_string()).await;
    let eml = concat!(
        "From: sender@example.com\r\n",
        "To: recipient@example.com\r\n",
        "Subject: Imported EML\r\n",
        "Message-ID: <imported-eml@example.com>\r\n",
        "MIME-Version: 1.0\r\n",
        "Content-Type: multipart/mixed; boundary=import-boundary\r\n",
        "\r\n",
        "--import-boundary\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "Imported message body.\r\n",
        "--import-boundary\r\n",
        "Content-Type: text/plain; name=note.txt\r\n",
        "Content-Disposition: attachment; filename=note.txt\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "\r\n",
        "aW1wb3J0ZWQgYXR0YWNobWVudA==\r\n",
        "--import-boundary--\r\n"
    );

    let first_response = app
        .clone()
        .oneshot(post_import(account_id, eml))
        .await
        .expect("first EML import response");
    assert_eq!(first_response.status(), StatusCode::OK);
    let first = response_json(first_response).await;
    assert_eq!(first["attachment_count"], 1);

    let second_response = app
        .oneshot(post_import(account_id, eml))
        .await
        .expect("second EML import response");
    assert_eq!(second_response.status(), StatusCode::OK);
    let second = response_json(second_response).await;
    assert_eq!(second["message_id"], first["message_id"]);
    assert_eq!(second["raw_record_id"], first["raw_record_id"]);

    let raw_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM communication_raw_records WHERE raw_record_id = $1",
    )
    .bind(first["raw_record_id"].as_str().expect("raw record id"))
    .fetch_one(context.pool())
    .await
    .expect("raw record count");
    assert_eq!(raw_count, 1);

    let attachments = CommunicationStorageStore::new(context.pool().clone())
        .attachments_for_message(first["message_id"].as_str().expect("message id"))
        .await
        .expect("imported attachments");
    assert_eq!(attachments.len(), 1);
    assert_eq!(
        attachments[0].attachment.filename.as_deref(),
        Some("note.txt")
    );
    assert_eq!(
        attachments[0].attachment.scan_status.as_str(),
        "not_scanned"
    );
}

#[tokio::test]
async fn v1_mbox_import_projects_each_message_and_is_idempotent() {
    let context = TestContext::new().await;
    let account_id = "acct-mbox-import";
    CommunicationIngestionStore::new(context.pool().clone())
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "MBOX Import",
            "mbox-import@example.com",
        ))
        .await
        .expect("store provider account");
    let app = router(&context.connection_string()).await;
    let mbox = concat!(
        "From sender@example.com Fri Jul 11 12:00:00 2026\n",
        "From: sender@example.com\n",
        "To: recipient@example.com\n",
        "Subject: First imported MBOX message\n",
        "Message-ID: <first-mbox-import@example.com>\n",
        "\n",
        "First import body.\n",
        "From sender@example.com Fri Jul 11 12:01:00 2026\n",
        "From: sender@example.com\n",
        "To: recipient@example.com\n",
        "Subject: Second imported MBOX message\n",
        "Message-ID: <second-mbox-import@example.com>\n",
        "\n",
        "Second import body.\n",
    );

    let first_response = app
        .clone()
        .oneshot(post_mbox_import(account_id, mbox))
        .await
        .expect("first MBOX import response");
    assert_eq!(first_response.status(), StatusCode::OK);
    let first = response_json(first_response).await;
    assert_eq!(first["imported_count"], 2);
    assert_eq!(first["message_ids"].as_array().map(Vec::len), Some(2));

    let second_response = app
        .oneshot(post_mbox_import(account_id, mbox))
        .await
        .expect("second MBOX import response");
    assert_eq!(second_response.status(), StatusCode::OK);
    let second = response_json(second_response).await;
    assert_eq!(second["message_ids"], first["message_ids"]);

    let raw_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM communication_raw_records WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_one(context.pool())
    .await
    .expect("raw record count");
    assert_eq!(raw_count, 2);
}

#[tokio::test]
async fn v1_mbox_import_reports_invalid_messages_without_rolling_back_valid_ones() {
    let context = TestContext::new().await;
    let account_id = "acct-mbox-import-recovery";
    CommunicationIngestionStore::new(context.pool().clone())
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "MBOX Recovery Import",
            "mbox-recovery@example.com",
        ))
        .await
        .expect("store provider account");
    let app = router(&context.connection_string()).await;
    let mbox = concat!(
        "From sender@example.com Fri Jul 11 12:00:00 2026\n",
        "From: sender@example.com\n",
        "To: recipient@example.com\n",
        "Subject: Valid imported MBOX message\n",
        "Message-ID: <valid-mbox-import@example.com>\n",
        "\n",
        "Valid import body.\n",
        "From sender@example.com Fri Jul 11 12:01:00 2026\n",
        "Subject: Missing RFC822 body separator\n",
    );

    let response = app
        .oneshot(post_mbox_import(account_id, mbox))
        .await
        .expect("MBOX recovery import response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["imported_count"], 1);
    assert_eq!(body["failed_count"], 1);
    assert_eq!(
        body["failures"],
        json!([{
            "message_index": 1,
            "reason": "invalid_message"
        }])
    );
}

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url,
        ),
        database,
    )
}

fn post_import(account_id: &str, eml: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/communications/import/eml")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::from(
            json!({
                "account_id": account_id,
                "eml_base64": BASE64_STANDARD.encode(eml),
            })
            .to_string(),
        ))
        .expect("EML import request")
}

fn post_mbox_import(account_id: &str, mbox: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/communications/import/mbox")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::from(
            json!({
                "account_id": account_id,
                "mbox_base64": BASE64_STANDARD.encode(mbox),
            })
            .to_string(),
        ))
        .expect("MBOX import request")
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("response body"),
    )
    .expect("response json")
}
