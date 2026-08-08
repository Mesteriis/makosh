//! Owner-local persistence for archive-inspection requests, event joins and fenced jobs.

mod custody;
mod jobs;
mod model;
mod observations;
mod runs;
mod schema;

use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub use model::{
    ARCHIVE_INSPECTION_MAX_ATTEMPTS_V1, ARCHIVE_INSPECTION_REALTIME_LIMIT_V1,
    ArchiveInspectionLeaseV1, ArchiveInspectionRealtimeTransitionV1,
    ArchiveInspectionTargetBlobReceiptV1, ClaimedArchiveInspectionJobV1,
    CreateArchiveInspectionRunOutcomeV1, CreateArchiveInspectionRunV1,
    PendingArchiveInspectionCustodyDelegationV1, PersistArchiveInspectionCustodyResultOutcomeV1,
    PersistArchiveInspectionFactOutcomeV1, PersistedArchiveInspectionRunV1,
    UnpublishedArchiveInspectionCustodyDelegationV1, archive_inspection_job_id_v1,
    archive_inspection_request_fingerprint_v1, archive_inspection_run_id_v1,
    archive_inspection_terminal_evidence_id_v1,
};
pub use schema::{
    ATTACHMENT_ARCHIVE_INSPECTION_SCHEMA_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_BUNDLE_REVISION_V1,
    attachment_archive_inspection_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-attachment-archive-inspection-persistence";

pub struct AttachmentArchiveInspectionPersistenceV1 {
    pub(crate) pool: PgPool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    EvidenceConflict,
    ClaimLost,
}

impl AttachmentArchiveInspectionPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, ArchiveInspectionPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(ArchiveInspectionPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
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
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
        sqlx::query(
            "SELECT 1 FROM makosh_data.attachment_archive_inspection_runs, makosh_data.attachment_archive_inspection_event_inbox, makosh_data.attachment_archive_inspection_scan_candidates, makosh_data.attachment_archive_inspection_safety_facts, makosh_data.attachment_archive_inspection_custody_delegation_requests, makosh_data.attachment_archive_inspection_custody_result_inbox, makosh_data.attachment_archive_inspection_jobs, makosh_data.attachment_archive_inspection_reports, makosh_data.attachment_archive_inspection_report_entries, makosh_data.attachment_archive_inspection_realtime LIMIT 0",
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)
    }
}

pub(crate) fn id16(value: &[u8]) -> Result<[u8; 16], ArchiveInspectionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)
}

pub(crate) fn id32(value: &[u8]) -> Result<[u8; 32], ArchiveInspectionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)
}

pub(crate) fn unsigned(value: i64) -> Result<u64, ArchiveInspectionPersistenceErrorV1> {
    u64::try_from(value).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)
}

pub(crate) fn positive_u32(value: i32) -> Result<u32, ArchiveInspectionPersistenceErrorV1> {
    let value =
        u32::try_from(value).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    if value == 0 {
        return Err(ArchiveInspectionPersistenceErrorV1::InvalidRow);
    }
    Ok(value)
}
