//! Attachment Security owner-local inbox, join, scan-job and exact outbox persistence.

mod delegation;
mod jobs;
mod observation;
mod preview_delegation;
mod recovery;
mod schema;
mod text_delegation;

use makosh_attachment_security_core::AttachmentSecurityQuarantineEvidenceV1;
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

pub use delegation::{
    AttachmentSecurityArchiveDelegationWorkV1, ClaimedAttachmentSecurityArchiveDelegationV1,
    PersistAttachmentSecurityArchiveDelegationOutcomeV1,
    RetryAttachmentSecurityArchiveDelegationOutcomeV1,
};
pub use jobs::{
    AttachmentSecurityTargetBlobReceiptV1, ClaimedAttachmentSecurityScanJobV1,
    RetryAttachmentSecurityScanJobOutcomeV1, attachment_security_scan_job_id_v1,
};
pub use preview_delegation::{
    AttachmentSecurityPreviewDelegationWorkV1, ClaimedAttachmentSecurityPreviewDelegationV1,
    PersistAttachmentSecurityPreviewDelegationOutcomeV1,
    RetryAttachmentSecurityPreviewDelegationOutcomeV1,
};
pub use recovery::{
    ATTACHMENT_SECURITY_RETRY_POLICY_REVISION_V2, ATTACHMENT_SECURITY_RETRY_POLICY_REVISION_V3,
};
pub use schema::{
    ATTACHMENT_SECURITY_SCHEMA_V1, ATTACHMENT_SECURITY_SCHEMA_V2, ATTACHMENT_SECURITY_SCHEMA_V6,
    ATTACHMENT_SECURITY_SCHEMA_V7, ATTACHMENT_SECURITY_SCHEMA_V8,
    ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V1, ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V2,
    ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V6, ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V7,
    ATTACHMENT_SECURITY_STORAGE_BUNDLE_REVISION_V8, attachment_security_storage_bundle_v1,
};
pub use text_delegation::{
    AttachmentSecurityTextDelegationWorkV1, ClaimedAttachmentSecurityTextDelegationV1,
    PersistAttachmentSecurityTextDelegationOutcomeV1,
    RetryAttachmentSecurityTextDelegationOutcomeV1,
};

pub const PACKAGE: &str = "makosh-attachment-security-persistence";

pub struct AttachmentSecurityPersistenceV1 {
    pub(crate) pool: PgPool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentSecurityRetryPolicyV1 {
    max_attempts: u32,
}

impl AttachmentSecurityRetryPolicyV1 {
    pub fn new(max_attempts: u32) -> Result<Self, AttachmentSecurityPersistenceErrorV1> {
        if !(1..=32).contains(&max_attempts) {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        Ok(Self { max_attempts })
    }

    #[must_use]
    pub const fn max_attempts(self) -> u32 {
        self.max_attempts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PersistAttachmentSecurityObservationOutcomeV1 {
    Duplicate,
    Waiting,
    Runnable { job_id: [u8; 16] },
    Quarantined(AttachmentSecurityQuarantineEvidenceV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    ClaimLost,
    OutboxHashConflict,
    EvidenceConflict,
}

impl AttachmentSecurityPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, AttachmentSecurityPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(AttachmentSecurityPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
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
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), AttachmentSecurityPersistenceErrorV1> {
        sqlx::query(
            "SELECT 1 FROM makosh_data.attachment_security_join_locks, makosh_data.attachment_security_event_inbox, makosh_data.attachment_security_scan_candidates, makosh_data.attachment_security_canonical_states, makosh_data.attachment_security_join_quarantines, makosh_data.attachment_security_verdict_outbox, makosh_data.attachment_security_scan_jobs, makosh_data.attachment_security_archive_delegation_inbox, makosh_data.attachment_security_archive_delegation_jobs, makosh_data.attachment_security_archive_delegation_outbox, makosh_data.attachment_security_text_extraction_delegation_inbox, makosh_data.attachment_security_text_extraction_delegation_jobs, makosh_data.attachment_security_text_extraction_delegation_outbox, makosh_data.attachment_security_preview_delegation_inbox, makosh_data.attachment_security_preview_delegation_jobs, makosh_data.attachment_security_preview_delegation_outbox LIMIT 0",
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)
    }
}

pub(crate) fn valid_id16(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_timestamp(value: i64) -> bool {
    (-62_135_596_800..=253_402_300_799).contains(&value)
}

pub(crate) fn id16(value: &[u8]) -> Result<[u8; 16], AttachmentSecurityPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)
}

pub(crate) fn id32(value: &[u8]) -> Result<[u8; 32], AttachmentSecurityPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_policy_is_bounded() {
        assert_eq!(
            AttachmentSecurityRetryPolicyV1::new(0),
            Err(AttachmentSecurityPersistenceErrorV1::InvalidInput)
        );
        assert_eq!(
            AttachmentSecurityRetryPolicyV1::new(33),
            Err(AttachmentSecurityPersistenceErrorV1::InvalidInput)
        );
        assert_eq!(
            AttachmentSecurityRetryPolicyV1::new(8)
                .expect("policy")
                .max_attempts(),
            8
        );
    }
}
