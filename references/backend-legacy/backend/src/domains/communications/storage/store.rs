use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;
use sqlx::postgres::PgPool;

use makosh_events_postgres::store::EventStore;

use crate::domains::communications::archive_inspection::{
    ArchiveInspectionReport, archive_inspection_cache_metadata,
};

use super::errors::CommunicationStorageError;
use super::ids::{mail_attachment_id, mail_blob_id};
use super::models::{
    NewCommunicationAttachment, NewCommunicationBlob, StoredCommunicationAttachment,
    StoredCommunicationAttachmentWithBlob, StoredCommunicationBlob,
};
use super::rows::{row_to_mail_attachment, row_to_mail_attachment_with_blob, row_to_mail_blob};
use super::scanner::AttachmentSafetyScanReport;
use super::validation::validate_non_empty;

#[derive(Clone)]
pub struct CommunicationStorageStore {
    pub(crate) pool: PgPool,
}

impl CommunicationStorageStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_blob(
        &self,
        blob: &NewCommunicationBlob,
    ) -> Result<StoredCommunicationBlob, CommunicationStorageError> {
        let blob = blob.validate()?;
        let blob_id = mail_blob_id(&blob.sha256);

        let row = sqlx::query(
            r#"
            INSERT INTO communication_mail_blobs (
                blob_id,
                storage_kind,
                storage_path,
                sha256,
                size_bytes,
                content_type
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (sha256)
            DO UPDATE SET
                content_type = COALESCE(communication_mail_blobs.content_type, EXCLUDED.content_type)
            RETURNING
                blob_id,
                storage_kind,
                storage_path,
                sha256,
                size_bytes,
                content_type,
                created_at
            "#,
        )
        .bind(&blob_id)
        .bind(&blob.storage_kind)
        .bind(&blob.storage_path)
        .bind(&blob.sha256)
        .bind(blob.size_bytes)
        .bind(&blob.content_type)
        .fetch_one(&self.pool)
        .await?;

        row_to_mail_blob(row)
    }

    pub async fn upsert_attachment(
        &self,
        attachment: &NewCommunicationAttachment,
    ) -> Result<StoredCommunicationAttachment, CommunicationStorageError> {
        let attachment = attachment.validate()?;
        let attachment_id =
            mail_attachment_id(&attachment.message_id, &attachment.provider_attachment_id);
        let mut transaction = self.pool.begin().await?;
        let previous_scan_status = sqlx::query_scalar::<_, String>(
            r#"
            SELECT scan_status
            FROM communication_attachments
            WHERE message_id = $1 AND provider_attachment_id = $2
            FOR UPDATE
            "#,
        )
        .bind(&attachment.message_id)
        .bind(&attachment.provider_attachment_id)
        .fetch_optional(&mut *transaction)
        .await?;

        let row = sqlx::query(
            r#"
            INSERT INTO communication_attachments (
                attachment_id,
                message_id,
                raw_record_id,
                blob_id,
                provider_attachment_id,
                filename,
                content_type,
                size_bytes,
                sha256,
                disposition,
                scan_status,
                scan_engine,
                scan_checked_at,
                scan_summary,
                scan_metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, now())
            ON CONFLICT (message_id, provider_attachment_id)
            DO UPDATE SET
                raw_record_id = EXCLUDED.raw_record_id,
                blob_id = EXCLUDED.blob_id,
                filename = EXCLUDED.filename,
                content_type = EXCLUDED.content_type,
                size_bytes = EXCLUDED.size_bytes,
                sha256 = EXCLUDED.sha256,
                disposition = EXCLUDED.disposition,
                scan_status = EXCLUDED.scan_status,
                scan_engine = EXCLUDED.scan_engine,
                scan_checked_at = EXCLUDED.scan_checked_at,
                scan_summary = EXCLUDED.scan_summary,
                scan_metadata = EXCLUDED.scan_metadata,
                updated_at = now()
            RETURNING
                attachment_id,
                message_id,
                raw_record_id,
                blob_id,
                provider_attachment_id,
                filename,
                content_type,
                size_bytes,
                sha256,
                disposition,
                scan_status,
                scan_engine,
                scan_checked_at,
                scan_summary,
                scan_metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&attachment_id)
        .bind(&attachment.message_id)
        .bind(&attachment.raw_record_id)
        .bind(&attachment.blob_id)
        .bind(&attachment.provider_attachment_id)
        .bind(&attachment.filename)
        .bind(&attachment.content_type)
        .bind(attachment.size_bytes)
        .bind(&attachment.sha256)
        .bind(attachment.disposition.as_str())
        .bind(attachment.scan_report.status.as_str())
        .bind(&attachment.scan_report.engine)
        .bind(attachment.scan_report.checked_at)
        .bind(&attachment.scan_report.summary)
        .bind(&attachment.scan_report.metadata)
        .fetch_one(&mut *transaction)
        .await?;
        let stored = row_to_mail_attachment(row)?;
        if previous_scan_status.as_deref() != Some(stored.scan_status.as_str()) {
            let occurred_at = Utc::now();
            let event = NewEventEnvelope::builder(
                format!(
                    "communication_attachment_processing:{}:{}:{}",
                    stored.attachment_id,
                    stored.scan_status.as_str(),
                    occurred_at.timestamp_micros()
                ),
                "communication.attachment.processing_changed.v1",
                occurred_at,
                json!({ "kind": "communication_attachment" }),
                json!({
                    "kind": "communication_attachment",
                    "id": stored.attachment_id,
                    "message_id": stored.message_id,
                }),
            )
            .actor(json!({ "actor_id": "makosh-attachment-scanner" }))
            .payload(json!({
                "attachment_id": stored.attachment_id,
                "message_id": stored.message_id,
                "previous_scan_status": previous_scan_status,
                "scan_status": stored.scan_status.as_str(),
                "scan_engine": stored.scan_engine,
                "scan_checked_at": stored.scan_checked_at,
            }))
            .provenance(json!({
                "source_kind": "communication_attachment_store",
                "source_id": stored.attachment_id,
            }))
            .build()?;
            EventStore::append_in_transaction(&mut transaction, &event).await?;
        }
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn attachments_for_message(
        &self,
        message_id: &str,
    ) -> Result<Vec<StoredCommunicationAttachmentWithBlob>, CommunicationStorageError> {
        let message_id = validate_non_empty("message_id", message_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                a.attachment_id,
                a.message_id,
                a.raw_record_id,
                a.blob_id,
                a.provider_attachment_id,
                a.filename,
                a.content_type,
                a.size_bytes,
                a.sha256,
                a.disposition,
                a.scan_status,
                a.scan_engine,
                a.scan_checked_at,
                a.scan_summary,
                a.scan_metadata,
                a.created_at,
                a.updated_at,
                b.storage_kind AS blob_storage_kind,
                b.storage_path AS blob_storage_path
            FROM communication_attachments a
            JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
            WHERE a.message_id = $1
            ORDER BY a.created_at ASC, a.attachment_id ASC
            "#,
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_mail_attachment_with_blob)
            .collect()
    }

    pub async fn attachment_by_id(
        &self,
        attachment_id: &str,
    ) -> Result<Option<StoredCommunicationAttachmentWithBlob>, CommunicationStorageError> {
        let attachment_id = validate_non_empty("attachment_id", attachment_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                a.attachment_id,
                a.message_id,
                a.raw_record_id,
                a.blob_id,
                a.provider_attachment_id,
                a.filename,
                a.content_type,
                a.size_bytes,
                a.sha256,
                a.disposition,
                a.scan_status,
                a.scan_engine,
                a.scan_checked_at,
                a.scan_summary,
                a.scan_metadata,
                a.created_at,
                a.updated_at,
                b.storage_kind AS blob_storage_kind,
                b.storage_path AS blob_storage_path
            FROM communication_attachments a
            JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
            WHERE a.attachment_id = $1
            "#,
        )
        .bind(attachment_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_mail_attachment_with_blob).transpose()
    }

    pub async fn list_not_scanned_attachments(
        &self,
        limit: i64,
    ) -> Result<Vec<StoredCommunicationAttachmentWithBlob>, CommunicationStorageError> {
        let rows = sqlx::query(
            r#"
            SELECT
                a.attachment_id,
                a.message_id,
                a.raw_record_id,
                a.blob_id,
                a.provider_attachment_id,
                a.filename,
                a.content_type,
                a.size_bytes,
                a.sha256,
                a.disposition,
                a.scan_status,
                a.scan_engine,
                a.scan_checked_at,
                a.scan_summary,
                a.scan_metadata,
                a.created_at,
                a.updated_at,
                b.storage_kind AS blob_storage_kind,
                b.storage_path AS blob_storage_path
            FROM communication_attachments a
            JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
            WHERE a.scan_status = 'not_scanned'
            ORDER BY a.created_at ASC, a.attachment_id ASC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_mail_attachment_with_blob)
            .collect()
    }

    /// Stores a retry verdict only while the attachment still points to the same blob and has
    /// not already received a newer scan decision.
    pub async fn persist_not_scanned_attachment_verdict(
        &self,
        attachment_id: &str,
        expected_sha256: &str,
        report: &AttachmentSafetyScanReport,
    ) -> Result<Option<StoredCommunicationAttachment>, CommunicationStorageError> {
        let attachment_id = validate_non_empty("attachment_id", attachment_id)?;
        let expected_sha256 = validate_non_empty("expected_sha256", expected_sha256)?;
        let report = report.validate()?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_attachments
            SET
                scan_status = $3,
                scan_engine = $4,
                scan_checked_at = $5,
                scan_summary = $6,
                scan_metadata = $7,
                updated_at = now()
            WHERE attachment_id = $1
              AND sha256 = $2
              AND scan_status = 'not_scanned'
            RETURNING
                attachment_id,
                message_id,
                raw_record_id,
                blob_id,
                provider_attachment_id,
                filename,
                content_type,
                size_bytes,
                sha256,
                disposition,
                scan_status,
                scan_engine,
                scan_checked_at,
                scan_summary,
                scan_metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&attachment_id)
        .bind(&expected_sha256)
        .bind(report.status.as_str())
        .bind(&report.engine)
        .bind(report.checked_at)
        .bind(&report.summary)
        .bind(&report.metadata)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(row) = row else {
            transaction.commit().await?;
            return Ok(None);
        };
        let stored = row_to_mail_attachment(row)?;
        let occurred_at = Utc::now();
        let event = NewEventEnvelope::builder(
            format!(
                "communication_attachment_processing:{}:{}:{}",
                stored.attachment_id,
                stored.scan_status.as_str(),
                occurred_at.timestamp_micros()
            ),
            "communication.attachment.processing_changed.v1",
            occurred_at,
            json!({ "kind": "communication_attachment" }),
            json!({
                "kind": "communication_attachment",
                "id": stored.attachment_id,
                "message_id": stored.message_id,
            }),
        )
        .actor(json!({ "actor_id": "makosh-attachment-scanner" }))
        .payload(json!({
            "attachment_id": stored.attachment_id,
            "message_id": stored.message_id,
            "previous_scan_status": "not_scanned",
            "scan_status": stored.scan_status.as_str(),
            "scan_engine": stored.scan_engine,
            "scan_checked_at": stored.scan_checked_at,
        }))
        .provenance(json!({
            "source_kind": "communication_attachment_rescan",
            "source_id": stored.attachment_id,
        }))
        .build()?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;
        Ok(Some(stored))
    }

    /// Persists a completed archive inspection only when its blob is still current.
    pub async fn persist_archive_inspection(
        &self,
        attachment_id: &str,
        source_sha256: &str,
        report: &ArchiveInspectionReport,
    ) -> Result<bool, CommunicationStorageError> {
        let attachment_id = validate_non_empty("attachment_id", attachment_id)?;
        let source_sha256 = validate_non_empty("source_sha256", source_sha256)?;
        let metadata = archive_inspection_cache_metadata(&source_sha256, report);

        let updated = sqlx::query_scalar::<_, String>(
            r#"
            UPDATE communication_attachments
            SET
                scan_metadata = jsonb_set(
                    COALESCE(scan_metadata, '{}'::jsonb),
                    '{archive_inspection}',
                    $3::jsonb,
                    true
                ),
                updated_at = now()
            WHERE attachment_id = $1
              AND sha256 = $2
            RETURNING attachment_id
            "#,
        )
        .bind(attachment_id)
        .bind(source_sha256)
        .bind(metadata)
        .fetch_optional(&self.pool)
        .await?;

        Ok(updated.is_some())
    }
}
