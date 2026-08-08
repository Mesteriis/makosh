#![forbid(unsafe_code)]

mod model;
mod repository;
mod schema;

use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub use model::PersistedReplayOperationV1;
pub use repository::{
    ReplayCommandOutboxRecordV1, ReplayOperationCreateOutcomeV1, ReplayPersistenceErrorV1,
    ReplayResultAcceptOutcomeV1, ReplayResultInboxRecordV1,
};
pub use schema::{
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_SCHEMA_V1, ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_SCHEMA_V2,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_BUNDLE_REVISION_V1,
    attachment_preview_evidence_replay_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-attachment-preview-evidence-replay-persistence";

pub struct AttachmentPreviewEvidenceReplayPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl AttachmentPreviewEvidenceReplayPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, ReplayPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(ReplayPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| ReplayPersistenceErrorV1::StorageUnavailable)?;
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
            .map_err(|_| ReplayPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), ReplayPersistenceErrorV1> {
        sqlx::query(
            "SELECT 1 FROM makosh_data.attachment_preview_evidence_replay_operations, makosh_data.attachment_preview_evidence_replay_anchor_producers, makosh_data.attachment_preview_evidence_replay_anchor_result_messages, makosh_data.attachment_preview_evidence_replay_anchor_command_outbox, makosh_data.attachment_preview_evidence_replay_anchor_result_inbox LIMIT 0",
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| ReplayPersistenceErrorV1::StorageUnavailable)
    }
}
