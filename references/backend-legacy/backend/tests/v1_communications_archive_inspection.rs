use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::io::{Cursor, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

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

const T: &str = "v1comms-archive-inspection-test-token";

#[tokio::test]
async fn v1_attachment_archive_inspection_persists_a_hash_bound_zip_report() {
    let context = TestContext::new().await;
    let seeded = seed_zip_attachment(context.pool().clone()).await;
    let app = router(&context.connection_string()).await;

    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/attachments/{}/archive-inspection",
            seeded.attachment_id
        )))
        .await
        .expect("archive inspection response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["attachment_id"], seeded.attachment_id);
    assert_eq!(body["message_id"], seeded.message_id);
    assert_eq!(body["filename"], "evidence.zip");
    assert_eq!(body["content_type"], "application/zip");
    assert_eq!(body["scan_status"], "clean");
    assert_eq!(body["report"]["archive_kind"], "zip");
    assert_eq!(body["report"]["entry_count"], 2);
    assert_eq!(body["report"]["total_uncompressed_bytes"], 17);
    assert_eq!(body["report"]["has_nested_archive"], false);
    assert_eq!(
        body["report"]["entries"][0]["normalized_path"],
        "docs/readme.txt"
    );
    assert_eq!(
        body["report"]["entries"][1]["normalized_path"],
        "invoice.txt"
    );

    let scan_metadata = sqlx::query_scalar::<_, Value>(
        "SELECT scan_metadata FROM communication_attachments WHERE attachment_id = $1",
    )
    .bind(&seeded.attachment_id)
    .fetch_one(context.pool())
    .await
    .expect("persisted archive inspection metadata");
    assert_eq!(
        scan_metadata["archive_inspection"]["version"],
        json!(1),
        "archive inspection cache version"
    );
    assert_eq!(
        scan_metadata["archive_inspection"]["source_sha256"], seeded.source_sha256,
        "report must be bound to the currently attached blob"
    );
    assert_eq!(
        scan_metadata["archive_inspection"]["report"]["entry_count"],
        json!(2)
    );
}

struct SeededAttachment {
    attachment_id: String,
    message_id: String,
    source_sha256: String,
}

async fn seed_zip_attachment(pool: sqlx::PgPool) -> SeededAttachment {
    let suffix = uid();
    let account_id = format!("acct-archive-inspection-{suffix}");
    let provider_record_id = format!("provider-archive-inspection-{suffix}");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let storage_store = CommunicationStorageStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Archive Inspection Gmail",
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
            format!("sha256:{:0>64}", "e"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": "Archive inspection",
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": "Please inspect the attached archive metadata."
            }),
        ))
        .await
        .expect("record raw source");
    let message_id = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id;

    let zip_bytes = zip_bytes(&[
        ("docs/readme.txt", b"hello" as &[u8]),
        ("invoice.txt", b"invoice data" as &[u8]),
    ]);
    let local_blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let local_blob = local_blob_store
        .put_blob(&zip_bytes)
        .await
        .expect("write zip blob");
    let blob = storage_store
        .upsert_blob(
            &NewCommunicationBlob::from_local_blob(&local_blob).content_type("application/zip"),
        )
        .await
        .expect("store zip blob metadata");
    let attachment = storage_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message_id,
                &raw.raw_record_id,
                blob.blob_id,
                "part-evidence-zip",
                "application/zip",
                local_blob.size_bytes,
                local_blob.sha256,
            )
            .filename("evidence.zip")
            .disposition(CommunicationAttachmentDisposition::Attachment)
            .scan_report(AttachmentSafetyScanReport {
                status: AttachmentSafetyScanStatus::Clean,
                engine: None,
                checked_at: None,
                summary: None,
                metadata: json!({}),
            }),
        )
        .await
        .expect("store zip attachment");

    SeededAttachment {
        attachment_id: attachment.attachment_id,
        message_id,
        source_sha256: attachment.sha256,
    }
}

fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    for (name, bytes) in entries {
        writer.start_file(*name, options).unwrap();
        writer.write_all(bytes).unwrap();
    }

    writer.finish().unwrap().into_inner()
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
