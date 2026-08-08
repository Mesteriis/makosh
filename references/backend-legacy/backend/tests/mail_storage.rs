use chrono::Utc;
use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::time::{SystemTime, UNIX_EPOCH};

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::archive_inspection::{
    ArchiveEntryInspection, ArchiveInspectionReport, cached_archive_inspection_report,
};
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use makosh_hub_backend::domains::communications::storage::errors::CommunicationStorageError;
use makosh_hub_backend::domains::communications::storage::models::{
    CommunicationAttachmentDisposition, NewCommunicationAttachment, NewCommunicationBlob,
};
use makosh_hub_backend::domains::communications::storage::scanner::{
    AttachmentSafetyScanReport, AttachmentSafetyScanStatus,
};
use makosh_hub_backend::domains::communications::storage::store::CommunicationStorageStore;

use makosh_hub_backend::platform::storage::database::Database;
use serde_json::json;

#[tokio::test]
async fn local_mail_blob_store_writes_content_addressed_blob_under_root() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let store = LocalCommunicationBlobStore::new(temp_dir.path());
    let first = store
        .put_blob(b"raw message bytes")
        .await
        .expect("write first blob");
    let second = store
        .put_blob(b"raw message bytes")
        .await
        .expect("write same blob again");

    assert_eq!(first, second);
    assert_eq!(first.storage_kind, "local_fs");
    assert_eq!(first.size_bytes, 17);
    assert!(first.sha256.starts_with("sha256:"));
    assert!(!first.storage_path.starts_with('/'));
    assert!(!first.storage_path.contains(".."));
    assert!(temp_dir.path().join(&first.storage_path).is_file());
}

#[tokio::test]
async fn mail_storage_records_attachment_metadata_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool.clone());
    let mail_store = CommunicationStorageStore::new(pool.clone());
    let blob_root = tempfile::tempdir().expect("blob root");
    let local_blob_store = LocalCommunicationBlobStore::new(blob_root.path());
    let suffix = unique_suffix();
    let account_id = format!("acct_mail_storage_{suffix}");
    let provider_record_id = format!("mail-storage-message-{suffix}");

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Icloud,
            "Mail storage account",
            format!("mail-storage-{suffix}@example.invalid"),
        ))
        .await
        .expect("provider account");
    let raw = communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw-mail-storage-{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:raw-mail-storage-{suffix}"),
                format!("batch-mail-storage-{suffix}"),
                json!({
                    "subject": "Attachment storage",
                    "from": "sender@example.invalid",
                    "to": ["recipient@example.invalid"],
                    "body_text": "See attached file."
                }),
            )
            .provenance(json!({"source": "mail_storage_test"})),
        )
        .await
        .expect("raw record");
    let message = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");

    let local_blob = local_blob_store
        .put_blob(b"pdf contents")
        .await
        .expect("write local attachment blob");
    let blob = mail_store
        .upsert_blob(
            &NewCommunicationBlob::from_local_blob(&local_blob).content_type("application/pdf"),
        )
        .await
        .expect("upsert blob");
    let attachment = mail_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message.message_id,
                &raw.raw_record_id,
                &blob.blob_id,
                "part-1",
                "application/pdf",
                local_blob.size_bytes,
                &blob.sha256,
            )
            .filename("invoice.pdf")
            .disposition(CommunicationAttachmentDisposition::Attachment),
        )
        .await
        .expect("upsert attachment");

    assert_eq!(attachment.message_id, message.message_id);
    assert_eq!(attachment.raw_record_id, raw.raw_record_id);
    assert_eq!(attachment.blob_id, blob.blob_id);
    assert_eq!(attachment.filename.as_deref(), Some("invoice.pdf"));
    assert_eq!(attachment.content_type, "application/pdf");
    assert_eq!(attachment.size_bytes, 12);
    assert_eq!(
        attachment.disposition,
        CommunicationAttachmentDisposition::Attachment
    );
    assert_eq!(
        attachment.scan_status,
        AttachmentSafetyScanStatus::NotScanned
    );
    assert!(attachment.scan_engine.is_none());
    assert!(attachment.scan_checked_at.is_none());
    assert!(attachment.scan_summary.is_none());
    assert_eq!(attachment.scan_metadata, json!({}));

    let attachment_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM communication_attachments WHERE message_id = $1",
    )
    .bind(&message.message_id)
    .fetch_one(&pool)
    .await
    .expect("attachment count");
    assert_eq!(attachment_count, 1);

    let event_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM event_log WHERE event_type = 'communication.attachment.processing_changed.v1' AND payload->>'attachment_id' = $1",
    )
    .bind(&attachment.attachment_id)
    .fetch_one(&pool)
    .await
    .expect("attachment processing event count");
    assert_eq!(event_count, 1);

    mail_store
        .upsert_attachment(
            &NewCommunicationAttachment::new(
                &message.message_id,
                &raw.raw_record_id,
                &blob.blob_id,
                "part-1",
                "application/pdf",
                local_blob.size_bytes,
                &blob.sha256,
            )
            .filename("invoice.pdf")
            .disposition(CommunicationAttachmentDisposition::Attachment),
        )
        .await
        .expect("repeat unchanged attachment upsert");
    let event_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM event_log WHERE event_type = 'communication.attachment.processing_changed.v1' AND payload->>'attachment_id' = $1",
    )
    .bind(&attachment.attachment_id)
    .fetch_one(&pool)
    .await
    .expect("unchanged attachment processing event count");
    assert_eq!(event_count, 1);

    let rescan = mail_store
        .persist_not_scanned_attachment_verdict(
            &attachment.attachment_id,
            &blob.sha256,
            &AttachmentSafetyScanReport {
                status: AttachmentSafetyScanStatus::Clean,
                engine: Some("test-clamav".to_owned()),
                checked_at: Some(Utc::now()),
                summary: Some("test scanner clean verdict".to_owned()),
                metadata: json!({"verdict": "clean"}),
            },
        )
        .await
        .expect("persist conditional rescan")
        .expect("not scanned attachment must accept its first verdict");
    assert_eq!(rescan.scan_status, AttachmentSafetyScanStatus::Clean);
    assert!(
        mail_store
            .persist_not_scanned_attachment_verdict(
                &attachment.attachment_id,
                &blob.sha256,
                &AttachmentSafetyScanReport {
                    status: AttachmentSafetyScanStatus::Malicious,
                    engine: Some("test-clamav".to_owned()),
                    checked_at: Some(Utc::now()),
                    summary: Some("must not overwrite a newer scan state".to_owned()),
                    metadata: json!({"verdict": "malicious"}),
                },
            )
            .await
            .expect("conditional rescan after state change")
            .is_none(),
        "retry must not overwrite a newer scan state"
    );
    let event_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM event_log WHERE event_type = 'communication.attachment.processing_changed.v1' AND payload->>'attachment_id' = $1",
    )
    .bind(&attachment.attachment_id)
    .fetch_one(&pool)
    .await
    .expect("conditional rescan event count");
    assert_eq!(event_count, 2);

    let archive_report = ArchiveInspectionReport {
        archive_kind: "zip".to_owned(),
        entry_count: 1,
        total_uncompressed_bytes: 12,
        has_nested_archive: false,
        entries: vec![ArchiveEntryInspection {
            name: "invoice.pdf".to_owned(),
            normalized_path: "invoice.pdf".to_owned(),
            compressed_size: 12,
            uncompressed_size: 12,
            is_dir: false,
            is_nested_archive: false,
        }],
    };
    assert!(
        mail_store
            .persist_archive_inspection(&attachment.attachment_id, &blob.sha256, &archive_report)
            .await
            .expect("persist archive inspection")
    );
    let stored_attachment = mail_store
        .attachment_by_id(&attachment.attachment_id)
        .await
        .expect("load attachment")
        .expect("attachment exists");
    assert_eq!(
        cached_archive_inspection_report(&stored_attachment.attachment.scan_metadata, &blob.sha256),
        Some(archive_report)
    );
    assert!(
        !mail_store
            .persist_archive_inspection(
                &attachment.attachment_id,
                "sha256:replaced-blob",
                &ArchiveInspectionReport {
                    archive_kind: "zip".to_owned(),
                    entry_count: 0,
                    total_uncompressed_bytes: 0,
                    has_nested_archive: false,
                    entries: vec![],
                },
            )
            .await
            .expect("stale source is ignored")
    );
}

#[tokio::test]
async fn mail_blob_metadata_rejects_unsafe_storage_path_before_database_write() {
    let store =
        CommunicationStorageStore::new(sqlx::PgPool::connect_lazy("postgres://unused").unwrap());
    let error = store
        .upsert_blob(&NewCommunicationBlob::new(
            "local_fs",
            "../outside.blob",
            "sha256:unsafe",
            1,
        ))
        .await
        .expect_err("unsafe path must fail");

    assert!(
        matches!(error, CommunicationStorageError::UnsafeStoragePath(ref path) if path == "../outside.blob"),
        "expected UnsafeStoragePath, got {error:?}"
    );
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
