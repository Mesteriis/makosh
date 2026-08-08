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
    AttachmentTextExtractionError, extract_local_attachment_text, extract_rich_attachment_text,
    is_locally_extractable_text_type, rich_attachment_extraction_kind,
    rich_attachment_extractor_address,
};
use makosh_events_postgres::store::EventStore;

const LOCAL_EXTRACTOR_NAME: &str = "makosh.local_utf8.v1";
const RICH_EXTRACTOR_NAME: &str = "makosh.attachment_extractor.v1";
const MAX_ATTACHMENT_TEXT_READ_BYTES: usize = 64 * 1024;

#[derive(Clone)]
pub struct AttachmentTextExtractionService {
    pool: PgPool,
    storage_store: CommunicationStorageStore,
    blob_store: LocalCommunicationBlobStore,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachmentTextExtractionOutcome {
    Completed {
        attachment_id: String,
        extracted_blob_id: String,
        extracted_size_bytes: i64,
    },
    Unsupported {
        attachment_id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTextContent {
    pub attachment_id: String,
    pub text: String,
    pub truncated: bool,
    pub extracted_size_bytes: i64,
}

struct CompletedAttachmentText<'a> {
    attachment_id: &'a str,
    message_id: &'a str,
    extractor: &'a str,
    source_sha256: &'a str,
    extracted_blob_id: &'a str,
    extracted_size_bytes: i64,
    extracted_text: &'a str,
}

impl AttachmentTextExtractionService {
    pub fn new(pool: PgPool, blob_store: LocalCommunicationBlobStore) -> Self {
        Self {
            storage_store: CommunicationStorageStore::new(pool.clone()),
            pool,
            blob_store,
        }
    }

    pub async fn extract(
        &self,
        attachment_id: &str,
    ) -> Result<AttachmentTextExtractionOutcome, AttachmentTextExtractionServiceError> {
        let attachment = self
            .storage_store
            .attachment_by_id(attachment_id)
            .await?
            .ok_or(AttachmentTextExtractionServiceError::NotFound)?;
        if attachment.attachment.scan_status.as_str() != "clean" {
            return Err(AttachmentTextExtractionServiceError::Quarantined);
        }
        if attachment.storage_kind != "local_fs" {
            return Err(AttachmentTextExtractionServiceError::UnsupportedStorage);
        }

        let local_text = is_locally_extractable_text_type(
            &attachment.attachment.content_type,
            attachment.attachment.filename.as_deref(),
        );
        let rich_kind = (!local_text)
            .then(|| {
                rich_attachment_extraction_kind(
                    &attachment.attachment.content_type,
                    attachment.attachment.filename.as_deref(),
                )
            })
            .flatten();
        let extractor = if local_text {
            LOCAL_EXTRACTOR_NAME
        } else if rich_kind.is_some() {
            RICH_EXTRACTOR_NAME
        } else {
            self.record_non_completed(
                &attachment.attachment.attachment_id,
                &attachment.attachment.message_id,
                "unsupported",
                LOCAL_EXTRACTOR_NAME,
                &attachment.attachment.sha256,
                None,
            )
            .await?;
            return Ok(AttachmentTextExtractionOutcome::Unsupported {
                attachment_id: attachment.attachment.attachment_id,
            });
        };
        self.record_executing(
            &attachment.attachment.attachment_id,
            &attachment.attachment.message_id,
            extractor,
            &attachment.attachment.sha256,
        )
        .await?;

        let extraction = if local_text {
            let bytes = self.blob_store.read_blob(&attachment.storage_path).await?;
            extract_local_attachment_text(
                &attachment.attachment.content_type,
                attachment.attachment.filename.as_deref(),
                &bytes,
            )
        } else if let Some(worker_address) = rich_attachment_extractor_address() {
            extract_rich_attachment_text(
                &worker_address,
                rich_kind.expect("rich extraction kind was selected"),
                &attachment.storage_path,
            )
            .await
            .map(|result| result.text)
        } else {
            Err(AttachmentTextExtractionError::RichWorkerNotConfigured)
        };

        let text = match extraction {
            Ok(text) => text,
            Err(error) => {
                self.record_non_completed(
                    &attachment.attachment.attachment_id,
                    &attachment.attachment.message_id,
                    "failed",
                    extractor,
                    &attachment.attachment.sha256,
                    Some(error.to_string()),
                )
                .await?;
                return Err(AttachmentTextExtractionServiceError::Extraction(error));
            }
        };

        let extracted_blob = self.blob_store.put_blob(text.as_bytes()).await?;
        let stored_blob = self
            .storage_store
            .upsert_blob(
                &NewCommunicationBlob::from_local_blob(&extracted_blob)
                    .content_type("text/plain; charset=utf-8"),
            )
            .await?;
        self.record_completed(CompletedAttachmentText {
            attachment_id: &attachment.attachment.attachment_id,
            message_id: &attachment.attachment.message_id,
            extractor,
            source_sha256: &attachment.attachment.sha256,
            extracted_blob_id: &stored_blob.blob_id,
            extracted_size_bytes: extracted_blob.size_bytes,
            extracted_text: &text,
        })
        .await?;
        if matches!(
            rich_kind,
            Some(
                crate::platform::communications::attachment_text::RichAttachmentExtractionKind::Pdf
                    | crate::platform::communications::attachment_text::RichAttachmentExtractionKind::Docx
            )
        )
            && let Err(error) = crate::domains::communications::attachment_safe_preview::AttachmentSafePreviewService::new(
                self.pool.clone(),
                self.blob_store.clone(),
            )
            .generate(&attachment.attachment.attachment_id)
            .await
        {
            tracing::warn!(
                attachment_id = %attachment.attachment.attachment_id,
                error = %error,
                "attachment safe preview rendering failed after text extraction"
            );
        }
        if matches!(
            rich_kind,
            Some(
                crate::platform::communications::attachment_text::RichAttachmentExtractionKind::Pdf
                    | crate::platform::communications::attachment_text::RichAttachmentExtractionKind::Docx
            )
        )
            && let Err(error) = crate::domains::communications::attachment_content_disarm::AttachmentContentDisarmService::new(
                self.pool.clone(),
                self.blob_store.clone(),
            )
            .generate(&attachment.attachment.attachment_id)
            .await
        {
            tracing::warn!(
                attachment_id = %attachment.attachment.attachment_id,
                error = %error,
                "attachment rich-document content disarm failed after text extraction"
            );
        }

        Ok(AttachmentTextExtractionOutcome::Completed {
            attachment_id: attachment.attachment.attachment_id,
            extracted_blob_id: stored_blob.blob_id,
            extracted_size_bytes: extracted_blob.size_bytes,
        })
    }

    pub async fn completed_text(
        &self,
        attachment_id: &str,
    ) -> Result<Option<AttachmentTextContent>, AttachmentTextExtractionServiceError> {
        let attachment = self
            .storage_store
            .attachment_by_id(attachment_id)
            .await?
            .ok_or(AttachmentTextExtractionServiceError::NotFound)?;
        if attachment.attachment.scan_status.as_str() != "clean" {
            return Err(AttachmentTextExtractionServiceError::Quarantined);
        }

        let row = sqlx::query(
            r#"
            SELECT e.source_sha256, e.extracted_size_bytes, b.storage_kind, b.storage_path
            FROM communication_attachment_extractions e
            JOIN communication_mail_blobs b ON b.blob_id = e.extracted_blob_id
            WHERE e.attachment_id = $1 AND e.status = 'completed'
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
        if storage_kind != "local_fs" {
            return Err(AttachmentTextExtractionServiceError::UnsupportedStorage);
        }
        let storage_path: String = row.try_get("storage_path")?;
        let extracted_size_bytes: i64 = row.try_get("extracted_size_bytes")?;
        let bytes = self.blob_store.read_blob(&storage_path).await?;
        let truncated = bytes.len() > MAX_ATTACHMENT_TEXT_READ_BYTES;
        let visible_bytes = truncate_utf8_bytes(&bytes, MAX_ATTACHMENT_TEXT_READ_BYTES);
        let text = std::str::from_utf8(visible_bytes)
            .map_err(|_| AttachmentTextExtractionServiceError::InvalidDerivedText)?
            .to_owned();

        Ok(Some(AttachmentTextContent {
            attachment_id: attachment.attachment.attachment_id,
            text,
            truncated,
            extracted_size_bytes,
        }))
    }

    async fn record_completed(
        &self,
        completed: CompletedAttachmentText<'_>,
    ) -> Result<(), AttachmentTextExtractionServiceError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO communication_attachment_extractions (
                attachment_id, status, extractor, source_sha256, extracted_blob_id,
                extracted_size_bytes, extracted_at, search_vector
            ) VALUES ($1, 'completed', $2, $3, $4, $5, now(), to_tsvector('simple', $6))
            ON CONFLICT (attachment_id) DO UPDATE SET
                status = EXCLUDED.status,
                extractor = EXCLUDED.extractor,
                source_sha256 = EXCLUDED.source_sha256,
                extracted_blob_id = EXCLUDED.extracted_blob_id,
                extracted_size_bytes = EXCLUDED.extracted_size_bytes,
                search_vector = EXCLUDED.search_vector,
                failure_summary = NULL,
                extracted_at = EXCLUDED.extracted_at,
                updated_at = now()
            "#,
        )
        .bind(completed.attachment_id)
        .bind(completed.extractor)
        .bind(completed.source_sha256)
        .bind(completed.extracted_blob_id)
        .bind(completed.extracted_size_bytes)
        .bind(completed.extracted_text)
        .execute(&mut *transaction)
        .await?;
        self.append_processing_event(
            &mut transaction,
            completed.attachment_id,
            completed.message_id,
            "completed",
            completed.extractor,
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
        extractor: &str,
        source_sha256: &str,
        failure_summary: Option<String>,
    ) -> Result<(), AttachmentTextExtractionServiceError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO communication_attachment_extractions (
                attachment_id, status, extractor, source_sha256, failure_summary
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (attachment_id) DO UPDATE SET
                status = EXCLUDED.status,
                extractor = EXCLUDED.extractor,
                source_sha256 = EXCLUDED.source_sha256,
                extracted_blob_id = NULL,
                extracted_size_bytes = NULL,
                search_vector = NULL,
                failure_summary = EXCLUDED.failure_summary,
                extracted_at = NULL,
                updated_at = now()
            "#,
        )
        .bind(attachment_id)
        .bind(status)
        .bind(extractor)
        .bind(source_sha256)
        .bind(failure_summary)
        .execute(&mut *transaction)
        .await?;
        self.append_processing_event(
            &mut transaction,
            attachment_id,
            message_id,
            status,
            extractor,
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
        extractor: &str,
        source_sha256: &str,
    ) -> Result<(), AttachmentTextExtractionServiceError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO communication_attachment_extractions (
                attachment_id, status, extractor, source_sha256
            ) VALUES ($1, 'executing', $2, $3)
            ON CONFLICT (attachment_id) DO UPDATE SET
                status = EXCLUDED.status,
                extractor = EXCLUDED.extractor,
                source_sha256 = EXCLUDED.source_sha256,
                extracted_blob_id = NULL,
                extracted_size_bytes = NULL,
                search_vector = NULL,
                failure_summary = NULL,
                extracted_at = NULL,
                updated_at = now()
            "#,
        )
        .bind(attachment_id)
        .bind(extractor)
        .bind(source_sha256)
        .execute(&mut *transaction)
        .await?;
        self.append_processing_event(
            &mut transaction,
            attachment_id,
            message_id,
            "executing",
            extractor,
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
        extractor: &str,
        source_sha256: &str,
    ) -> Result<(), AttachmentTextExtractionServiceError> {
        let occurred_at = Utc::now();
        let event = NewEventEnvelope::builder(
            format!(
                "communication_attachment_text_extraction:{}:{}:{}",
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
        .actor(json!({ "actor_id": "makosh-attachment-extractor" }))
        .payload(json!({
            "attachment_id": attachment_id,
            "message_id": message_id,
            "processing_kind": "text_extraction",
            "status": status,
            "extractor": extractor,
            "source_sha256": source_sha256,
        }))
        .provenance(json!({
            "source_kind": "communication_attachment_extraction",
            "source_id": attachment_id,
        }))
        .build()?;
        EventStore::append_in_transaction(transaction, &event).await?;
        Ok(())
    }
}

fn truncate_utf8_bytes(bytes: &[u8], limit: usize) -> &[u8] {
    if bytes.len() <= limit {
        return bytes;
    }
    let mut end = limit;
    while end > 0 && (bytes[end] & 0b1100_0000) == 0b1000_0000 {
        end -= 1;
    }
    &bytes[..end]
}

#[derive(Debug, Error)]
pub enum AttachmentTextExtractionServiceError {
    #[error("attachment was not found")]
    NotFound,
    #[error("attachment remains quarantined until it has a clean scan verdict")]
    Quarantined,
    #[error("attachment extraction requires a local blob")]
    UnsupportedStorage,
    #[error(transparent)]
    Storage(#[from] CommunicationStorageError),
    #[error(transparent)]
    Extraction(#[from] AttachmentTextExtractionError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Event(#[from] makosh_events_postgres::errors::EventStoreError),
    #[error(transparent)]
    EventEnvelope(#[from] makosh_events_api::EventEnvelopeError),
    #[error("derived attachment text is not valid UTF-8")]
    InvalidDerivedText,
}

#[cfg(test)]
mod tests {
    use super::truncate_utf8_bytes;

    #[test]
    fn truncation_keeps_a_valid_utf8_boundary() {
        let source = "abcé".as_bytes();
        assert_eq!(truncate_utf8_bytes(source, 4), b"abc");
        assert_eq!(truncate_utf8_bytes(source, 5), source);
    }
}
