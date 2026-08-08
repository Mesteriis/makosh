//! Owner-local, idempotent retry-policy reconciliation.

use sqlx::query;

use crate::{AttachmentSecurityPersistenceErrorV1, AttachmentSecurityPersistenceV1};

pub const ATTACHMENT_SECURITY_RETRY_POLICY_REVISION_V2: i16 = 2;
pub const ATTACHMENT_SECURITY_RETRY_POLICY_REVISION_V3: i16 = 3;

impl AttachmentSecurityPersistenceV1 {
    /// Applies each one-time retry-policy reconciliation in revision order.
    ///
    /// Exact revision and progress predicates make this data reconciliation,
    /// not a generic terminal-job requeue API.
    pub async fn reconcile_retry_policies_v3(
        &self,
    ) -> Result<u64, AttachmentSecurityPersistenceErrorV1> {
        let custody_outcome = query(
            "UPDATE makosh_data.attachment_security_scan_jobs SET state = 1, attempt_count = 0, next_attempt_at_unix_seconds = 0, claimed_by = NULL, lease_expires_at_unix_seconds = NULL, completed_at_unix_seconds = NULL, retry_policy_revision = $1 WHERE state = 3 AND target_blob_reference_id IS NULL AND target_blob_receipt_sha256 IS NULL AND outbox_message_id IS NULL AND retry_policy_revision = 1",
        )
        .bind(ATTACHMENT_SECURITY_RETRY_POLICY_REVISION_V2)
        .execute(&self.pool)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        let scanner_outcome = query(
            "UPDATE makosh_data.attachment_security_scan_jobs SET state = 1, attempt_count = 0, next_attempt_at_unix_seconds = 0, claimed_by = NULL, lease_expires_at_unix_seconds = NULL, completed_at_unix_seconds = NULL, retry_policy_revision = $1 WHERE state = 3 AND target_blob_reference_id IS NOT NULL AND target_blob_receipt_sha256 IS NOT NULL AND outbox_message_id IS NULL AND retry_policy_revision = 2",
        )
        .bind(ATTACHMENT_SECURITY_RETRY_POLICY_REVISION_V3)
        .execute(&self.pool)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        Ok(custody_outcome.rows_affected() + scanner_outcome.rows_affected())
    }
}
