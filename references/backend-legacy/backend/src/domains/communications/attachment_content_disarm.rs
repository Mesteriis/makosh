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
    AttachmentTextExtractionError, RichAttachmentExtractionKind, disarm_rich_attachment,
    rich_attachment_extraction_kind, rich_attachment_extractor_address,
};
use makosh_events_postgres::store::EventStore;

const PDF_CDR_RENDERER: &str = "makosh.attachment_extractor.pdf_cdr.v1";
const DOCX_CDR_RENDERER: &str = "makosh.attachment_extractor.docx_cdr.v1";
const MAX_CDR_ARTIFACT_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone)]
pub struct AttachmentContentDisarmService {
    pool: PgPool,
    storage: CommunicationStorageStore,
    blobs: LocalCommunicationBlobStore,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachmentContentDisarmOutcome {
    Completed {
        attachment_id: String,
        artifact_blob_id: String,
        artifact_size_bytes: i64,
    },
    Unsupported {
        attachment_id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentContentDisarmArtifact {
    pub attachment_id: String,
    pub bytes: Vec<u8>,
    pub content_type: String,
}

struct CdrRecord<'a> {
    attachment_id: &'a str,
    message_id: &'a str,
    status: &'a str,
    renderer: &'a str,
    source_sha256: &'a str,
    completed: Option<(&'a str, i64)>,
    failure: Option<String>,
}

impl AttachmentContentDisarmService {
    pub fn new(pool: PgPool, blobs: LocalCommunicationBlobStore) -> Self {
        Self {
            storage: CommunicationStorageStore::new(pool.clone()),
            pool,
            blobs,
        }
    }

    pub async fn generate(
        &self,
        attachment_id: &str,
    ) -> Result<AttachmentContentDisarmOutcome, AttachmentContentDisarmError> {
        let attachment = self
            .storage
            .attachment_by_id(attachment_id)
            .await?
            .ok_or(AttachmentContentDisarmError::NotFound)?;
        if attachment.attachment.scan_status.as_str() != "clean" {
            return Err(AttachmentContentDisarmError::Quarantined);
        }
        if attachment.storage_kind != "local_fs" {
            return Err(AttachmentContentDisarmError::UnsupportedStorage);
        }
        let Some(kind) = rich_attachment_extraction_kind(
            &attachment.attachment.content_type,
            attachment.attachment.filename.as_deref(),
        ) else {
            self.record(CdrRecord {
                attachment_id: &attachment.attachment.attachment_id,
                message_id: &attachment.attachment.message_id,
                status: "unsupported",
                renderer: PDF_CDR_RENDERER,
                source_sha256: &attachment.attachment.sha256,
                completed: None,
                failure: None,
            })
            .await?;
            return Ok(AttachmentContentDisarmOutcome::Unsupported {
                attachment_id: attachment.attachment.attachment_id,
            });
        };
        let Some(renderer) = cdr_renderer(kind) else {
            self.record(CdrRecord {
                attachment_id: &attachment.attachment.attachment_id,
                message_id: &attachment.attachment.message_id,
                status: "unsupported",
                renderer: PDF_CDR_RENDERER,
                source_sha256: &attachment.attachment.sha256,
                completed: None,
                failure: None,
            })
            .await?;
            return Ok(AttachmentContentDisarmOutcome::Unsupported {
                attachment_id: attachment.attachment.attachment_id,
            });
        };
        self.record(CdrRecord {
            attachment_id: &attachment.attachment.attachment_id,
            message_id: &attachment.attachment.message_id,
            status: "executing",
            renderer,
            source_sha256: &attachment.attachment.sha256,
            completed: None,
            failure: None,
        })
        .await?;
        let worker = match rich_attachment_extractor_address() {
            Some(worker) => worker,
            None => {
                self.record(CdrRecord {
                    attachment_id: &attachment.attachment.attachment_id,
                    message_id: &attachment.attachment.message_id,
                    status: "failed",
                    renderer,
                    source_sha256: &attachment.attachment.sha256,
                    completed: None,
                    failure: Some("worker_not_configured".to_owned()),
                })
                .await?;
                return Err(AttachmentContentDisarmError::WorkerNotConfigured);
            }
        };
        let artifact = match disarm_rich_attachment(&worker, kind, &attachment.storage_path).await {
            Ok(artifact) => artifact,
            Err(error) => {
                self.record(CdrRecord {
                    attachment_id: &attachment.attachment.attachment_id,
                    message_id: &attachment.attachment.message_id,
                    status: "failed",
                    renderer,
                    source_sha256: &attachment.attachment.sha256,
                    completed: None,
                    failure: Some(error.to_string()),
                })
                .await?;
                return Err(AttachmentContentDisarmError::Rendering(error));
            }
        };
        let local_blob = self.blobs.put_blob(&artifact.bytes).await?;
        let stored_blob = self
            .storage
            .upsert_blob(
                &NewCommunicationBlob::from_local_blob(&local_blob)
                    .content_type(&artifact.content_type),
            )
            .await?;
        self.record(CdrRecord {
            attachment_id: &attachment.attachment.attachment_id,
            message_id: &attachment.attachment.message_id,
            status: "completed",
            renderer,
            source_sha256: &attachment.attachment.sha256,
            completed: Some((&stored_blob.blob_id, local_blob.size_bytes)),
            failure: None,
        })
        .await?;
        Ok(AttachmentContentDisarmOutcome::Completed {
            attachment_id: attachment.attachment.attachment_id,
            artifact_blob_id: stored_blob.blob_id,
            artifact_size_bytes: local_blob.size_bytes,
        })
    }

    pub async fn completed_artifact(
        &self,
        attachment_id: &str,
    ) -> Result<Option<AttachmentContentDisarmArtifact>, AttachmentContentDisarmError> {
        let attachment = self
            .storage
            .attachment_by_id(attachment_id)
            .await?
            .ok_or(AttachmentContentDisarmError::NotFound)?;
        if attachment.attachment.scan_status.as_str() != "clean" {
            return Err(AttachmentContentDisarmError::Quarantined);
        }
        let row = sqlx::query("SELECT c.source_sha256, c.artifact_content_type, b.storage_kind, b.storage_path FROM communication_attachment_cdr_artifacts c JOIN communication_mail_blobs b ON b.blob_id = c.artifact_blob_id WHERE c.attachment_id = $1 AND c.status = 'completed'")
            .bind(&attachment.attachment.attachment_id).fetch_optional(&self.pool).await?;
        let Some(row) = row else {
            return Ok(None);
        };
        if row.try_get::<String, _>("source_sha256")? != attachment.attachment.sha256 {
            return Ok(None);
        }
        if row.try_get::<String, _>("storage_kind")? != "local_fs"
            || row.try_get::<String, _>("artifact_content_type")? != "application/pdf"
        {
            return Err(AttachmentContentDisarmError::InvalidArtifact);
        }
        let bytes = self
            .blobs
            .read_blob(&row.try_get::<String, _>("storage_path")?)
            .await?;
        if bytes.is_empty()
            || bytes.len() > MAX_CDR_ARTIFACT_BYTES
            || !bytes.starts_with(b"%PDF-")
            || !bytes[bytes.len().saturating_sub(1024)..]
                .windows(5)
                .any(|part| part == b"%%EOF")
        {
            return Err(AttachmentContentDisarmError::InvalidArtifact);
        }
        Ok(Some(AttachmentContentDisarmArtifact {
            attachment_id: attachment.attachment.attachment_id,
            bytes,
            content_type: "application/pdf".to_owned(),
        }))
    }

    async fn record(&self, record: CdrRecord<'_>) -> Result<(), AttachmentContentDisarmError> {
        let CdrRecord {
            attachment_id,
            message_id,
            status,
            renderer,
            source_sha256,
            completed,
            failure,
        } = record;
        let mut transaction = self.pool.begin().await?;
        let (blob_id, size) = completed.map_or((None, None), |(id, size)| (Some(id), Some(size)));
        sqlx::query("INSERT INTO communication_attachment_cdr_artifacts (attachment_id, status, renderer, source_sha256, artifact_blob_id, artifact_content_type, artifact_size_bytes, failure_summary, disarmed_at) VALUES ($1,$2,$3,$4,$5,CASE WHEN $5 IS NULL THEN NULL ELSE 'application/pdf' END,$6,$7,CASE WHEN $5 IS NULL THEN NULL ELSE now() END) ON CONFLICT (attachment_id) DO UPDATE SET status=EXCLUDED.status, renderer=EXCLUDED.renderer, source_sha256=EXCLUDED.source_sha256, artifact_blob_id=EXCLUDED.artifact_blob_id, artifact_content_type=EXCLUDED.artifact_content_type, artifact_size_bytes=EXCLUDED.artifact_size_bytes, failure_summary=EXCLUDED.failure_summary, disarmed_at=EXCLUDED.disarmed_at, updated_at=now()")
            .bind(attachment_id).bind(status).bind(renderer).bind(source_sha256).bind(blob_id).bind(size).bind(failure).execute(&mut *transaction).await?;
        let now = Utc::now();
        let event = NewEventEnvelope::builder(format!("communication_attachment_cdr:{attachment_id}:{status}:{}", now.timestamp_micros()), "communication.attachment.processing_changed.v1", now, json!({"kind":"communication_attachment"}), json!({"kind":"communication_attachment","id":attachment_id,"message_id":message_id}))
            .actor(json!({"actor_id":"makosh-attachment-cdr"}))
            .payload(json!({"attachment_id":attachment_id,"message_id":message_id,"processing_kind":"content_disarm","status":status,"renderer":renderer,"source_sha256":source_sha256}))
            .provenance(json!({"source_kind":"communication_attachment_cdr","source_id":attachment_id})).build()?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;
        Ok(())
    }
}

fn cdr_renderer(kind: RichAttachmentExtractionKind) -> Option<&'static str> {
    match kind {
        RichAttachmentExtractionKind::Pdf => Some(PDF_CDR_RENDERER),
        RichAttachmentExtractionKind::Docx => Some(DOCX_CDR_RENDERER),
        RichAttachmentExtractionKind::Ocr => None,
    }
}

#[derive(Debug, Error)]
pub enum AttachmentContentDisarmError {
    #[error("attachment was not found")]
    NotFound,
    #[error("attachment is quarantined until a clean scan verdict")]
    Quarantined,
    #[error("attachment requires local blob storage")]
    UnsupportedStorage,
    #[error("attachment CDR worker is not configured")]
    WorkerNotConfigured,
    #[error("attachment CDR renderer failed: {0}")]
    Rendering(#[from] AttachmentTextExtractionError),
    #[error("attachment CDR artifact is invalid")]
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
