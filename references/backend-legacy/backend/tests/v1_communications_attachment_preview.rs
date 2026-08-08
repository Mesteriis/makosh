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
use makosh_hub_backend::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use makosh_hub_backend::domains::communications::storage::models::{
    CommunicationAttachmentDisposition, NewCommunicationAttachment, NewCommunicationBlob,
};
use makosh_hub_backend::domains::communications::storage::scanner::{
    AttachmentSafetyScanReport, AttachmentSafetyScanStatus,
};
use makosh_hub_backend::domains::communications::storage::store::CommunicationStorageStore;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use makosh_hub_backend::platform::storage::database::Database;

const T: &str = "v1comms-attachment-preview-test-token";

#[tokio::test]
async fn v1_attachment_preview_reads_bounded_local_text_blob_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "notes.txt",
        "text/plain",
        AttachmentSafetyScanStatus::Clean,
        b"First line\nSecond line\n",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("attachment preview response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["message_id"], seeded.message_id);
    assert_eq!(body["filename"], "notes.txt");
    assert_eq!(body["content_type"], "text/plain");
    assert_eq!(body["scan_status"], "clean");
    assert_eq!(body["preview_kind"], "text");
    assert_eq!(body["text"], "First line\nSecond line\n");
    assert_eq!(body["truncated"], false);
    assert_eq!(body["byte_count"], 23);
    assert_eq!(body["max_preview_bytes"], 65536);
}

#[tokio::test]
async fn v1_attachment_content_disarm_download_requires_current_clean_source() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "source.pdf",
        "application/pdf",
        AttachmentSafetyScanStatus::Clean,
        b"%PDF-1.4\nsource\n%%EOF",
    )
    .await;
    store_completed_content_disarm(
        context.pool(),
        &seeded.attachment_id,
        b"%PDF-1.4\nsafe\n%%EOF",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/content-disarm",
            seeded.attachment_id
        )))
        .await
        .expect("CDR download response");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "application/pdf");
    assert_eq!(
        response.headers()["content-disposition"],
        "attachment; filename=\"attachment-cdr.pdf\""
    );
    assert_eq!(
        to_bytes(response.into_body(), 3 * 1024 * 1024)
            .await
            .expect("CDR bytes"),
        b"%PDF-1.4\nsafe\n%%EOF".as_slice()
    );

    sqlx::query("UPDATE communication_attachments SET sha256 = $1 WHERE attachment_id = $2")
        .bind("sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd")
        .bind(&seeded.attachment_id)
        .execute(context.pool())
        .await
        .expect("invalidate CDR source hash");
    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/content-disarm",
            seeded.attachment_id
        )))
        .await
        .expect("stale CDR response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn v1_attachment_text_extraction_persists_a_derived_blob_reference() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "extract.txt",
        "text/plain",
        AttachmentSafetyScanStatus::Clean,
        b"Searchable local attachment text.\r\n",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .clone()
        .oneshot(post(&format!(
            "/api/v1/communications/attachments/{}/extract-text",
            seeded.attachment_id
        )))
        .await
        .expect("attachment text extraction response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["status"], "completed");
    assert_eq!(body["extracted_size_bytes"], 34);

    let stored: (String, String, i64) = sqlx::query_as(
        "SELECT status, source_sha256, extracted_size_bytes FROM communication_attachment_extractions WHERE attachment_id = $1",
    )
    .bind(&seeded.attachment_id)
    .fetch_one(context.pool())
    .await
    .expect("derived extraction reference");
    assert_eq!(stored.0, "completed");
    assert!(stored.1.starts_with("sha256:"));
    assert_eq!(stored.2, 34);

    let extraction_events = sqlx::query_scalar::<_, Value>(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'communication.attachment.processing_changed.v1'
          AND payload->>'attachment_id' = $1
          AND payload->>'processing_kind' = 'text_extraction'
        ORDER BY position ASC
        "#,
    )
    .bind(&seeded.attachment_id)
    .fetch_all(context.pool())
    .await
    .expect("attachment extraction lifecycle events");
    assert_eq!(
        extraction_events
            .iter()
            .filter_map(|event| event["status"].as_str())
            .collect::<Vec<_>>(),
        vec!["executing", "completed"]
    );
    assert!(extraction_events.iter().all(|event| {
        event["extractor"] == "makosh.local_utf8.v1"
            && event.get("text").is_none()
            && event.get("storage_path").is_none()
    }));

    let response = app
        .clone()
        .oneshot(get(
            "/api/v1/communications/attachments/search?q=searchable%20local",
        ))
        .await
        .expect("derived text search response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("search items").len(), 1);
    assert_eq!(body["items"][0]["attachment_id"], seeded.attachment_id);
    assert_eq!(body["items"][0]["extracted_text_match"], true);

    let response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/extract-text",
            seeded.attachment_id
        )))
        .await
        .expect("attachment extracted text response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["text"], "Searchable local attachment text.\n");
    assert_eq!(body["truncated"], false);
    assert_eq!(body["extracted_size_bytes"], 34);

    sqlx::query("UPDATE communication_attachments SET sha256 = $1 WHERE attachment_id = $2")
        .bind("sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc")
        .bind(&seeded.attachment_id)
        .execute(context.pool())
        .await
        .expect("invalidate extracted source hash");

    let response = app
        .oneshot(get(
            "/api/v1/communications/attachments/search?q=searchable%20local",
        ))
        .await
        .expect("stale derived text search response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert!(
        body["items"]
            .as_array()
            .expect("stale search items")
            .is_empty()
    );
}

#[tokio::test]
async fn v1_attachment_preview_reads_bounded_local_image_blob_against_postgres() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "pixel.png",
        "image/png",
        AttachmentSafetyScanStatus::Clean,
        b"\x89PNG\r\n\x1a\n",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("attachment image preview response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["filename"], "pixel.png");
    assert_eq!(body["content_type"], "image/png");
    assert_eq!(body["preview_kind"], "image");
    assert_eq!(body["text"], "");
    assert_eq!(body["data_url"], "data:image/png;base64,iVBORw0KGgo=");
    assert_eq!(body["truncated"], false);
    assert_eq!(body["byte_count"], 8);
}

#[tokio::test]
async fn v1_attachment_preview_serves_pdf_derived_text_only_after_extraction() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "spec.pdf",
        "application/pdf",
        AttachmentSafetyScanStatus::Clean,
        b"%PDF-1.4\n",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("attachment pdf preview response");

    assert_eq!(response.status(), StatusCode::PRECONDITION_FAILED);
    let body = response_json(response).await;
    assert_eq!(body["error"], "failed_precondition");
    assert_eq!(body["message"], "extract attachment text before preview");

    store_completed_derived_text(
        context.pool(),
        &seeded.attachment_id,
        "Safe text derived from the PDF artifact.",
    )
    .await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("derived pdf preview response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["content_type"], "application/pdf");
    assert_eq!(body["preview_kind"], "text");
    assert_eq!(body["text"], "Safe text derived from the PDF artifact.");
    assert!(body["data_url"].is_null());
}

#[tokio::test]
async fn v1_attachment_preview_prefers_a_durable_safe_pdf_bitmap_artifact() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "preview.pdf",
        "application/pdf",
        AttachmentSafetyScanStatus::Clean,
        b"%PDF-1.4\n",
    )
    .await;
    store_completed_safe_preview(context.pool(), &seeded.attachment_id, b"\x89PNG\r\n\x1a\n").await;
    let app = router(&context.connection_string()).await;

    let response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("safe pdf preview response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["content_type"], "application/pdf");
    assert_eq!(body["preview_kind"], "image");
    assert_eq!(body["text"], "");
    assert_eq!(body["data_url"], "data:image/png;base64,iVBORw0KGgo=");

    sqlx::query("UPDATE communication_attachments SET sha256 = $1 WHERE attachment_id = $2")
        .bind("sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd")
        .bind(&seeded.attachment_id)
        .execute(context.pool())
        .await
        .expect("invalidate safe preview source hash");
    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("stale safe preview response");
    assert_eq!(response.status(), StatusCode::PRECONDITION_FAILED);
    let body = response_json(response).await;
    assert_eq!(body["error"], "failed_precondition");
    assert_eq!(body["message"], "extract attachment text before preview");
}

#[tokio::test]
async fn v1_attachment_preview_rejects_malicious_attachment_metadata() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "danger.txt",
        "text/plain",
        AttachmentSafetyScanStatus::Malicious,
        b"This text must not be exposed through preview.",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("attachment preview rejection response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"], "invalid_communication_query");
    assert_eq!(
        body["message"],
        "attachment preview is blocked by attachment scan status"
    );
}

#[tokio::test]
async fn v1_attachment_preview_rejects_unscanned_attachment_metadata() {
    let context = TestContext::new().await;
    let seeded = seed_text_attachment(
        context.pool().clone(),
        "pending.txt",
        "text/plain",
        AttachmentSafetyScanStatus::NotScanned,
        b"This attachment remains quarantined until a clean verdict.",
    )
    .await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/preview",
            seeded.attachment_id
        )))
        .await
        .expect("attachment preview quarantine response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"], "invalid_communication_query");
    assert_eq!(
        body["message"],
        "attachment preview is blocked by attachment scan status"
    );
}

struct SeededAttachment {
    attachment_id: String,
    message_id: String,
}

async fn seed_text_attachment(
    pool: sqlx::PgPool,
    filename: &str,
    content_type: &str,
    scan_status: AttachmentSafetyScanStatus,
    bytes: &[u8],
) -> SeededAttachment {
    let suffix = uid();
    let account_id = format!("acct-attachment-preview-{suffix}");
    let provider_record_id = format!("provider-attachment-preview-{suffix}");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let storage_store = CommunicationStorageStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Attachment Preview Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{provider_record_id}"),
            &account_id,
            "email_message",
            &provider_record_id,
            format!("sha256:{:0>64}", "f"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": "Attachment preview",
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Preview the attached text metadata safely."
            }),
        ))
        .await
        .expect("record raw source");
    let message_id = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id;

    let local_blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let local_blob = local_blob_store
        .put_blob(bytes)
        .await
        .expect("write text blob");
    let blob = storage_store
        .upsert_blob(&NewCommunicationBlob::from_local_blob(&local_blob).content_type(content_type))
        .await
        .expect("store text blob metadata");
    let attachment = storage_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message_id,
                &raw.raw_record_id,
                blob.blob_id,
                format!("part-{filename}"),
                content_type,
                local_blob.size_bytes,
                local_blob.sha256,
            )
            .filename(filename)
            .disposition(CommunicationAttachmentDisposition::Attachment)
            .scan_report(AttachmentSafetyScanReport {
                status: scan_status,
                engine: None,
                checked_at: None,
                summary: None,
                metadata: json!({}),
            }),
        )
        .await
        .expect("store text attachment");

    SeededAttachment {
        attachment_id: attachment.attachment_id,
        message_id,
    }
}

async fn store_completed_derived_text(pool: &sqlx::PgPool, attachment_id: &str, text: &str) {
    let source_sha256: String =
        sqlx::query_scalar("SELECT sha256 FROM communication_attachments WHERE attachment_id = $1")
            .bind(attachment_id)
            .fetch_one(pool)
            .await
            .expect("attachment source hash");
    let local_blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let derived_blob = local_blob_store
        .put_blob(text.as_bytes())
        .await
        .expect("write derived attachment text blob");
    let stored_blob = CommunicationStorageStore::new(pool.clone())
        .upsert_blob(
            &NewCommunicationBlob::from_local_blob(&derived_blob)
                .content_type("text/plain; charset=utf-8"),
        )
        .await
        .expect("store derived attachment text blob metadata");

    sqlx::query(
        r#"
        INSERT INTO communication_attachment_extractions (
            attachment_id, status, extractor, source_sha256, extracted_blob_id,
            extracted_size_bytes, extracted_at
        ) VALUES ($1, 'completed', 'makosh.attachment-extractor.v1', $2, $3, $4, now())
        ON CONFLICT (attachment_id) DO UPDATE SET
            status = EXCLUDED.status,
            extractor = EXCLUDED.extractor,
            source_sha256 = EXCLUDED.source_sha256,
            extracted_blob_id = EXCLUDED.extracted_blob_id,
            extracted_size_bytes = EXCLUDED.extracted_size_bytes,
            extracted_at = EXCLUDED.extracted_at,
            failure_summary = NULL,
            updated_at = now()
        "#,
    )
    .bind(attachment_id)
    .bind(source_sha256)
    .bind(stored_blob.blob_id)
    .bind(derived_blob.size_bytes)
    .execute(pool)
    .await
    .expect("store completed derived attachment text");
}

async fn store_completed_safe_preview(pool: &sqlx::PgPool, attachment_id: &str, bytes: &[u8]) {
    let source_sha256: String =
        sqlx::query_scalar("SELECT sha256 FROM communication_attachments WHERE attachment_id = $1")
            .bind(attachment_id)
            .fetch_one(pool)
            .await
            .expect("attachment source hash");
    let local_blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let preview_blob = local_blob_store
        .put_blob(bytes)
        .await
        .expect("write safe preview blob");
    let stored_blob = CommunicationStorageStore::new(pool.clone())
        .upsert_blob(
            &NewCommunicationBlob::from_local_blob(&preview_blob).content_type("image/png"),
        )
        .await
        .expect("store safe preview blob metadata");

    sqlx::query(
        r#"
        INSERT INTO communication_attachment_safe_previews (
            attachment_id, status, renderer, source_sha256, preview_blob_id,
            preview_content_type, preview_size_bytes, rendered_at
        ) VALUES ($1, 'completed', 'makosh.attachment-extractor.pdf_preview.v1', $2, $3, 'image/png', $4, now())
        "#,
    )
    .bind(attachment_id)
    .bind(source_sha256)
    .bind(stored_blob.blob_id)
    .bind(preview_blob.size_bytes)
    .execute(pool)
    .await
    .expect("store completed safe preview");
}

async fn store_completed_content_disarm(pool: &sqlx::PgPool, attachment_id: &str, bytes: &[u8]) {
    let source_sha256: String =
        sqlx::query_scalar("SELECT sha256 FROM communication_attachments WHERE attachment_id = $1")
            .bind(attachment_id)
            .fetch_one(pool)
            .await
            .expect("attachment source hash");
    let blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let artifact = blob_store.put_blob(bytes).await.expect("write CDR blob");
    let stored_blob = CommunicationStorageStore::new(pool.clone())
        .upsert_blob(
            &NewCommunicationBlob::from_local_blob(&artifact).content_type("application/pdf"),
        )
        .await
        .expect("store CDR blob metadata");
    sqlx::query(
        "INSERT INTO communication_attachment_cdr_artifacts (attachment_id, status, renderer, source_sha256, artifact_blob_id, artifact_content_type, artifact_size_bytes, disarmed_at) VALUES ($1, 'completed', 'makosh.attachment-extractor.pdf_cdr.v1', $2, $3, 'application/pdf', $4, now())",
    )
    .bind(attachment_id)
    .bind(source_sha256)
    .bind(stored_blob.blob_id)
    .bind(artifact.size_bytes)
    .execute(pool)
    .await
    .expect("store completed CDR artifact");
}

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

fn post(uri: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("request")
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
