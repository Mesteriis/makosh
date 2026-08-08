#![forbid(unsafe_code)]
//! Owner-local PostgreSQL persistence for the Attachment Preview workflow.

mod custody;
mod evidence;
mod jobs;
mod model;
mod repository;
mod schema;
mod tickets;

use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub use model::{
    ATTACHMENT_PREVIEW_REALTIME_LIMIT_V1, ClaimedAttachmentPreviewJobV1,
    CreateAttachmentPreviewRunOutcomeV1, CreateAttachmentPreviewRunV1,
    IssueAttachmentPreviewTicketV1, IssuedAttachmentPreviewTicketV1,
    PendingAttachmentPreviewCustodyDelegationV1, PersistAttachmentPreviewCustodyDelegationV1,
    PersistAttachmentPreviewCustodyResultOutcomeV1, PersistAttachmentPreviewFactOutcomeV1,
    PersistedAttachmentPreviewArtifactV1, PersistedAttachmentPreviewRunV1, PreviewJobLeaseV1,
    PreviewRealtimeTransitionV1, PreviewTargetBlobReceiptV1, RedeemedAttachmentPreviewTicketV1,
    RenderedAttachmentPreviewArtifactV1, UnpublishedAttachmentPreviewCustodyDelegationV1,
    attachment_preview_job_id_v1, attachment_preview_request_fingerprint_v1,
    attachment_preview_run_id_v1,
};
pub use schema::{
    ATTACHMENT_PREVIEW_SCHEMA_V1, ATTACHMENT_PREVIEW_STORAGE_BUNDLE_REVISION_V1,
    attachment_preview_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-attachment-preview-persistence";

pub struct AttachmentPreviewPersistenceV1 {
    pub(crate) pool: PgPool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    EvidenceConflict,
    NotFound,
    TicketExpired,
    TicketUsed,
    StaleFence,
}

impl AttachmentPreviewPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, AttachmentPreviewPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(AttachmentPreviewPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| AttachmentPreviewPersistenceErrorV1::StorageUnavailable)?;
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
            .map_err(|_| AttachmentPreviewPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
        sqlx::query(
            "SELECT 1 FROM makosh_data.attachment_preview_runs, makosh_data.attachment_preview_event_inbox, makosh_data.attachment_preview_scan_candidates, makosh_data.attachment_preview_safety_facts, makosh_data.attachment_preview_custody_outbox, makosh_data.attachment_preview_custody_result_inbox, makosh_data.attachment_preview_jobs, makosh_data.attachment_preview_artifacts, makosh_data.attachment_preview_read_tickets, makosh_data.attachment_preview_realtime LIMIT 0",
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| AttachmentPreviewPersistenceErrorV1::StorageUnavailable)
    }
}
