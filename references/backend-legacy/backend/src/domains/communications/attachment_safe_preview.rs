use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use crate::domains::communications::storage::errors::CommunicationStorageError;
use crate::domains::communications::storage::models::NewCommunicationBlob;
use crate::domains::communications::storage::store::CommunicationStorageStore;
use crate::platform::communications::attachment_text::{
    AttachmentTextExtractionError, RichAttachmentExtractionKind,
    render_rich_attachment_safe_preview, rich_attachment_extraction_kind,
    rich_attachment_extractor_address,
};
use makosh_events_postgres::store::EventStore;

const PDF_PREVIEW_RENDERER: &str = "makosh.attachment_extractor.pdf_preview.v1";
const DOCX_PREVIEW_RENDERER: &str = "makosh.attachment_extractor.docx_preview.v1";
const UNSUPPORTED_PREVIEW_RENDERER: &str = "makosh.attachment_extractor.safe_preview.v1";
const MAX_SAFE_PREVIEW_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone)]
pub struct AttachmentSafePreviewService {
    pool: PgPool,
    storage_store: CommunicationStorageStore,
    blob_store: LocalCommunicationBlobStore,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachmentSafePreviewOutcome {
    Completed {
        attachment_id: String,
        preview_blob_id: String,
        preview_size_bytes: i64,
    },
    Unsupported {
        attachment_id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentSafePreview {
    pub attachment_id: String,
    pub bytes: Vec<u8>,
    pub content_type: String,
}

struct CompletedSafePreview<'a> {
    attachment_id: &'a str,
    message_id: &'a str,
    renderer: &'a str,
    source_sha256: &'a str,
    preview_blob_id: &'a str,
    content_type: &'a str,
    preview_size_bytes: i64,
}

impl AttachmentSafePreviewService {
    pub fn new(pool: PgPool, blob_store: LocalCommunicationBlobStore) -> Self {
        Self {
            storage_store: CommunicationStorageStore::new(pool.clone()),
            pool,
            blob_store,
        }
    }

    pub async fn generate(
        &self,
        attachment_id: &str,
    ) -> Result<AttachmentSafePreviewOutcome, AttachmentSafePreviewServiceError> {
        let attachment = self
            .storage_store
            .attachment_by_id(attachment_id)
            .await?
            .ok_or(AttachmentSafePreviewServiceError::NotFound)?;
        if attachment.attachment.scan_status.as_str() != "clean" {
            return Err(AttachmentSafePreviewServiceError::Quarantined);
        }
        if attachment.storage_kind != "local_fs" {
            return Err(AttachmentSafePreviewServiceError::UnsupportedStorage);
        }
        let rich_kind = rich_attachment_extraction_kind(
            &attachment.attachment.content_type,
            attachment.attachment.filename.as_deref(),
        );
        let Some((kind, renderer)) = preview_renderer(rich_kind) else {
            self.record_non_completed(
                &attachment.attachment.attachment_id,
                &attachment.attachment.message_id,
                "unsupported",
                UNSUPPORTED_PREVIEW_RENDERER,
                &attachment.attachment.sha256,
                None,
            )
            .await?;
            return Ok(AttachmentSafePreviewOutcome::Unsupported {
                attachment_id: attachment.attachment.attachment_id,
            });
        };

        self.record_executing(
            &attachment.attachment.attachment_id,
            &attachment.attachment.message_id,
            renderer,
            &attachment.attachment.sha256,
        )
        .await?;
        let result = match rich_attachment_extractor_address() {
            Some(worker_address) => {
                render_rich_attachment_safe_preview(&worker_address, kind, &attachment.storage_path)
                    .await
            }
            None => Err(AttachmentTextExtractionError::RichWorkerNotConfigured),
        };
        let rendered = match result {
            Ok(rendered) => rendered,
            Err(error) => {
                self.record_non_completed(
                    &attachment.attachment.attachment_id,
                    &attachment.attachment.message_id,
                    "failed",
                    renderer,
                    &attachment.attachment.sha256,
                    Some(error.to_string()),
                )
                .await?;
                return Err(AttachmentSafePreviewServiceError::Rendering(error));
            }
        };
        let preview_blob = self.blob_store.put_blob(&rendered.bytes).await?;
        let stored_blob = self
            .storage_store
            .upsert_blob(
                &NewCommunicationBlob::from_local_blob(&preview_blob)
                    .content_type(&rendered.content_type),
            )
            .await?;
        self.record_completed(CompletedSafePreview {
            attachment_id: &attachment.attachment.attachment_id,
            message_id: &attachment.attachment.message_id,
            renderer,
            source_sha256: &attachment.attachment.sha256,
            preview_blob_id: &stored_blob.blob_id,
            content_type: &rendered.content_type,
            preview_size_bytes: preview_blob.size_bytes,
        })
        .await?;

        Ok(AttachmentSafePreviewOutcome::Completed {
            attachment_id: attachment.attachment.attachment_id,
            preview_blob_id: stored_blob.blob_id,
            preview_size_bytes: preview_blob.size_bytes,
        })
    }

    pub async fn completed_preview(
        &self,
        attachment_id: &str,
    ) -> Result<Option<AttachmentSafePreview>, AttachmentSafePreviewServiceError> {
        let attachment = self
            .storage_store
            .attachment_by_id(attachment_id)
            .await?
            .ok_or(AttachmentSafePreviewServiceError::NotFound)?;
        if attachment.attachment.scan_status.as_str() != "clean" {
            return Err(AttachmentSafePreviewServiceError::Quarantined);
        }
        let row = sqlx::query(
            r#"
            SELECT p.source_sha256, p.preview_content_type, b.storage_kind, b.storage_path
            FROM communication_attachment_safe_previews p
            JOIN communication_mail_blobs b ON b.blob_id = p.preview_blob_id
            WHERE p.attachment_id = $1 AND p.status = 'completed'
            "#,
        )
        .bind(&attachment.attachment.attachment_id)
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let source_sha256: String = row.try_get("source_sha256")?;
        if source_sha256 != attachment.attachment.sha256 {
            return Ok(None);
        }
        let storage_kind: String = row.try_get("storage_kind")?;
        let content_type: String = row.try_get("preview_content_type")?;
        if storage_kind != "local_fs" || content_type != "image/png" {
            return Err(AttachmentSafePreviewServiceError::InvalidArtifact);
        }
        let storage_path: String = row.try_get("storage_path")?;
        let bytes = self.blob_store.read_blob(&storage_path).await?;
        if bytes.is_empty()
            || bytes.len() > MAX_SAFE_PREVIEW_BYTES
            || !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        {
            return Err(AttachmentSafePreviewServiceError::InvalidArtifact);
        }
        Ok(Some(AttachmentSafePreview {
            attachment_id: attachment.attachment.attachment_id,
            bytes,
            content_type,
        }))
    }

    async fn record_completed(
        &self,
        completed: CompletedSafePreview<'_>,
    ) -> Result<(), AttachmentSafePreviewServiceError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO communication_attachment_safe_previews (
                attachment_id, status, renderer, source_sha256, preview_blob_id,
                preview_content_type, preview_size_bytes, rendered_at
            ) VALUES ($1, 'completed', $2, $3, $4, $5, $6, now())
            ON CONFLICT (attachment_id) DO UPDATE SET
                status = EXCLUDED.status,
                renderer = EXCLUDED.renderer,
                source_sha256 = EXCLUDED.source_sha256,
                preview_blob_id = EXCLUDED.preview_blob_id,
                preview_content_type = EXCLUDED.preview_content_type,
                preview_size_bytes = EXCLUDED.preview_size_bytes,
                failure_summary = NULL,
                rendered_at = EXCLUDED.rendered_at,
                updated_at = now()
            "#,
        )
        .bind(completed.attachment_id)
        .bind(completed.renderer)
        .bind(completed.source_sha256)
        .bind(completed.preview_blob_id)
        .bind(completed.content_type)
        .bind(completed.preview_size_bytes)
        .execute(&mut *transaction)
        .await?;
        self.append_processing_event(
            &mut transaction,
            completed.attachment_id,
            completed.message_id,
            "completed",
            completed.renderer,
            completed.source_sha256,
        )
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    async fn record_non_completed(
        &self,
        attachment_id: &str,
        message_id: &str,
        status: &str,
        renderer: &str,
        source_sha256: &str,
        failure_summary: Option<String>,
    ) -> Result<(), AttachmentSafePreviewServiceError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO communication_attachment_safe_previews (
                attachment_id, status, renderer, source_sha256, failure_summary
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (attachment_id) DO UPDATE SET
                status = EXCLUDED.status,
                renderer = EXCLUDED.renderer,
                source_sha256 = EXCLUDED.source_sha256,
                preview_blob_id = NULL,
                preview_content_type = NULL,
                preview_size_bytes = NULL,
                failure_summary = EXCLUDED.failure_summary,
                rendered_at = NULL,
                updated_at = now()
            "#,
        )
        .bind(attachment_id)
        .bind(status)
        .bind(renderer)
        .bind(source_sha256)
        .bind(failure_summary)
        .execute(&mut *transaction)
        .await?;
        self.append_processing_event(
            &mut transaction,
            attachment_id,
            message_id,
            status,
            renderer,
            source_sha256,
        )
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    async fn record_executing(
        &self,
        attachment_id: &str,
        message_id: &str,
        renderer: &str,
        source_sha256: &str,
    ) -> Result<(), AttachmentSafePreviewServiceError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO communication_attachment_safe_previews (
                attachment_id, status, renderer, source_sha256
            ) VALUES ($1, 'executing', $2, $3)
            ON CONFLICT (attachment_id) DO UPDATE SET
                status = EXCLUDED.status,
                renderer = EXCLUDED.renderer,
                source_sha256 = EXCLUDED.source_sha256,
                preview_blob_id = NULL,
                preview_content_type = NULL,
                preview_size_bytes = NULL,
                failure_summary = NULL,
                rendered_at = NULL,
                updated_at = now()
            "#,
        )
        .bind(attachment_id)
        .bind(renderer)
        .bind(source_sha256)
        .execute(&mut *transaction)
        .await?;
        self.append_processing_event(
            &mut transaction,
            attachment_id,
            message_id,
            "executing",
            renderer,
            source_sha256,
        )
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    async fn append_processing_event(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        attachment_id: &str,
        message_id: &str,
        status: &str,
        renderer: &str,
        source_sha256: &str,
    ) -> Result<(), AttachmentSafePreviewServiceError> {
        let occurred_at = Utc::now();
        let event = NewEventEnvelope::builder(
            format!(
                "communication_attachment_safe_preview:{}:{}:{}",
                attachment_id,
                status,
                occurred_at.timestamp_micros()
            ),
            "communication.attachment.processing_changed.v1",
            occurred_at,
            json!({ "kind": "communication_attachment" }),
            json!({
                "kind": "communication_attachment",
                "id": attachment_id,
                "message_id": message_id,
            }),
        )
        .actor(json!({ "actor_id": "makosh-attachment-preview-renderer" }))
        .payload(json!({
            "attachment_id": attachment_id,
            "message_id": message_id,
            "processing_kind": "safe_preview",
            "status": status,
            "renderer": renderer,
            "source_sha256": source_sha256,
        }))
        .provenance(json!({
            "source_kind": "communication_attachment_safe_preview",
            "source_id": attachment_id,
        }))
        .build()?;
        EventStore::append_in_transaction(transaction, &event).await?;
        Ok(())
    }
}

fn preview_renderer(
    kind: Option<RichAttachmentExtractionKind>,
) -> Option<(RichAttachmentExtractionKind, &'static str)> {
    match kind {
        Some(RichAttachmentExtractionKind::Pdf) => {
            Some((RichAttachmentExtractionKind::Pdf, PDF_PREVIEW_RENDERER))
        }
        Some(RichAttachmentExtractionKind::Docx) => {
            Some((RichAttachmentExtractionKind::Docx, DOCX_PREVIEW_RENDERER))
        }
        Some(RichAttachmentExtractionKind::Ocr) | None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{DOCX_PREVIEW_RENDERER, PDF_PREVIEW_RENDERER, preview_renderer};
    use crate::platform::communications::attachment_text::RichAttachmentExtractionKind;

    #[test]
    fn accepts_only_pdf_and_docx_sandboxed_preview_renderers() {
        assert_eq!(
            preview_renderer(Some(RichAttachmentExtractionKind::Pdf)),
            Some((RichAttachmentExtractionKind::Pdf, PDF_PREVIEW_RENDERER))
        );
        assert_eq!(
            preview_renderer(Some(RichAttachmentExtractionKind::Docx)),
            Some((RichAttachmentExtractionKind::Docx, DOCX_PREVIEW_RENDERER))
        );
        assert_eq!(
            preview_renderer(Some(RichAttachmentExtractionKind::Ocr)),
            None
        );
    }
}

#[derive(Debug, Error)]
pub enum AttachmentSafePreviewServiceError {
    #[error("attachment was not found")]
    NotFound,
    #[error("attachment is quarantined until a clean scan verdict")]
    Quarantined,
    #[error("attachment requires local blob storage")]
    UnsupportedStorage,
    #[error("attachment preview renderer failed: {0}")]
    Rendering(#[from] AttachmentTextExtractionError),
    #[error("attachment safe preview artifact is invalid")]
    InvalidArtifact,
    #[error(transparent)]
    Storage(#[from] CommunicationStorageError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Event(#[from] makosh_events_postgres::errors::EventStoreError),
    #[error(transparent)]
    EventEnvelope(#[from] makosh_events_api::EventEnvelopeError),
}
