#![forbid(unsafe_code)]
//! Owner-local PostgreSQL persistence for attachment text extraction.

mod custody;
mod jobs;
mod model;
mod observations;
mod repository;
mod schema;
mod translation_source;

use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub use model::{
    ATTACHMENT_TEXT_EXTRACTION_REALTIME_LIMIT_V1, ClaimedAttachmentTextExtractionJobV1,
    CreateAttachmentTextExtractionRunOutcomeV1, CreateAttachmentTextExtractionRunV1,
    PendingAttachmentTextCustodyDelegationV1, PersistAttachmentTextCustodyDelegationV1,
    PersistAttachmentTextCustodyResultOutcomeV1, PersistAttachmentTextFactOutcomeV1,
    PersistTranslationSourceResultOutcomeV1, PersistTranslationSourceResultV1,
    PersistedAttachmentTextArtifactV1, PersistedAttachmentTextExtractionRunV1,
    TextExtractionLeaseV1, TextExtractionRealtimeTransitionV1, TextExtractionTargetBlobReceiptV1,
    TranslationSourceSnapshotOutcomeV1, TranslationSourceSnapshotV1,
    UnpublishedAttachmentTextCustodyDelegationV1, UnpublishedTranslationSourceResultV1,
    attachment_text_extraction_job_id_v1, attachment_text_extraction_request_fingerprint_v1,
    attachment_text_extraction_run_id_v1,
};
pub use schema::{
    ATTACHMENT_TEXT_EXTRACTION_SCHEMA_V1, ATTACHMENT_TEXT_EXTRACTION_STORAGE_BUNDLE_REVISION_V1,
    ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_SCHEMA_V1,
    attachment_text_extraction_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-attachment-text-extraction-persistence";

pub struct AttachmentTextExtractionPersistenceV1 {
    pub(crate) pool: PgPool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextExtractionPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    EvidenceConflict,
}

impl AttachmentTextExtractionPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, AttachmentTextExtractionPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::StorageUnavailable)?;
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(port)
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(
        &self,
    ) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
        sqlx::query(
            "SELECT 1 FROM makosh_data.attachment_text_extraction_runs, makosh_data.attachment_text_extraction_event_inbox, makosh_data.attachment_text_extraction_scan_candidates, makosh_data.attachment_text_extraction_safety_facts, makosh_data.attachment_text_extraction_custody_outbox, makosh_data.attachment_text_extraction_custody_result_inbox, makosh_data.attachment_text_extraction_jobs, makosh_data.attachment_text_extraction_artifacts, makosh_data.attachment_text_extraction_realtime, makosh_data.attachment_text_extraction_translation_source_inbox, makosh_data.attachment_text_extraction_translation_source_outbox LIMIT 0",
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::StorageUnavailable)
    }
}
