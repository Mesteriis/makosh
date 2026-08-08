//! Bounded scan-job claims, retry exhaustion and exact verdict outbox storage.

use makosh_attachment_security_core::AttachmentSecurityScanJobV1;
use makosh_communications_attachment_contract::{
    AttachmentSafetyExpectedStateV1, AttachmentSafetyVerdictOutboxRecordV1,
    AttachmentSafetyVerdictV1,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    ATTACHMENT_SECURITY_RETRY_POLICY_REVISION_V3, AttachmentSecurityPersistenceErrorV1,
    AttachmentSecurityPersistenceV1, AttachmentSecurityRetryPolicyV1, id16, id32, valid_id16,
    valid_sha256, valid_timestamp,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimedAttachmentSecurityScanJobV1 {
    pub job_id: [u8; 16],
    pub job: AttachmentSecurityScanJobV1,
    pub candidate_envelope_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
    pub target_blob_receipt: Option<AttachmentSecurityTargetBlobReceiptV1>,
    pub worker_id: String,
    pub attempt_count: u32,
    pub max_attempts: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentSecurityTargetBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub receipt_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryAttachmentSecurityScanJobOutcomeV1 {
    Scheduled,
    Exhausted,
}

pub(crate) async fn enqueue_scan_job(
    transaction: &mut Transaction<'_, Postgres>,
    job: &AttachmentSecurityScanJobV1,
    retry_policy: AttachmentSecurityRetryPolicyV1,
    created_at_unix_seconds: i64,
) -> Result<[u8; 16], AttachmentSecurityPersistenceErrorV1> {
    let job_id = attachment_security_scan_job_id_v1(job);
    let declared_size = i64::try_from(job.declared_size)
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?;
    let max_attempts = i32::try_from(retry_policy.max_attempts())
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?;
    sqlx::query(
        "INSERT INTO makosh_data.attachment_security_scan_jobs (job_id, candidate_message_id, canonical_state_message_id, attachment_anchor_id, blob_reference_id, declared_size, blob_receipt_sha256, causation_message_id, correlation_id, state, max_attempts, next_attempt_at_unix_seconds, retry_policy_revision) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 1, $10, $11, $12) ON CONFLICT (job_id) DO NOTHING",
    )
    .bind(job_id.as_slice())
    .bind(job.candidate_message_id.as_slice())
    .bind(job.canonical_state_message_id.as_slice())
    .bind(job.attachment_anchor_id.as_slice())
    .bind(job.blob_reference_id.as_slice())
    .bind(declared_size)
    .bind(job.blob_receipt_sha256.as_slice())
    .bind(job.causation_message_id.as_slice())
    .bind(job.correlation_id.as_slice())
    .bind(max_attempts)
    .bind(created_at_unix_seconds)
    .bind(ATTACHMENT_SECURITY_RETRY_POLICY_REVISION_V3)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    Ok(job_id)
}

impl AttachmentSecurityPersistenceV1 {
    pub async fn claim_next_scan_job(
        &self,
        worker_id: &str,
        claimed_at_unix_seconds: i64,
        lease_expires_at_unix_seconds: i64,
    ) -> Result<Option<ClaimedAttachmentSecurityScanJobV1>, AttachmentSecurityPersistenceErrorV1>
    {
        if !valid_worker_id(worker_id)
            || !valid_timestamp(claimed_at_unix_seconds)
            || !valid_timestamp(lease_expires_at_unix_seconds)
            || lease_expires_at_unix_seconds <= claimed_at_unix_seconds
        {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        sqlx::query(
            "UPDATE makosh_data.attachment_security_scan_jobs SET state = 3, completed_at_unix_seconds = $1, claimed_by = NULL, lease_expires_at_unix_seconds = NULL WHERE state = 1 AND attempt_count >= max_attempts AND (lease_expires_at_unix_seconds IS NULL OR lease_expires_at_unix_seconds <= $1)",
        )
        .bind(claimed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        let row = sqlx::query(
            "WITH next_job AS (SELECT job_id FROM makosh_data.attachment_security_scan_jobs WHERE state = 1 AND next_attempt_at_unix_seconds <= $2 AND attempt_count < max_attempts AND (lease_expires_at_unix_seconds IS NULL OR lease_expires_at_unix_seconds <= $2) ORDER BY next_attempt_at_unix_seconds ASC, job_id ASC LIMIT 1 FOR UPDATE SKIP LOCKED) UPDATE makosh_data.attachment_security_scan_jobs AS job SET claimed_by = $1, lease_expires_at_unix_seconds = $3, attempt_count = job.attempt_count + 1 FROM next_job, makosh_data.attachment_security_scan_candidates AS source_candidate, makosh_data.attachment_security_event_inbox AS candidate_inbox WHERE job.job_id = next_job.job_id AND source_candidate.message_id = job.candidate_message_id AND candidate_inbox.message_id = source_candidate.message_id RETURNING job.job_id, job.candidate_message_id, job.canonical_state_message_id, job.attachment_anchor_id, job.blob_reference_id, job.declared_size, job.blob_receipt_sha256, job.causation_message_id, job.correlation_id, job.attempt_count, job.max_attempts, source_candidate.custody_transfer_source_proof, candidate_inbox.envelope_sha256 AS candidate_envelope_sha256, job.target_blob_reference_id, job.target_blob_receipt_sha256",
        )
        .bind(worker_id)
        .bind(claimed_at_unix_seconds)
        .bind(lease_expires_at_unix_seconds)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        let claimed = row
            .map(|row| claimed_job_from_row(row, worker_id))
            .transpose()?;
        transaction
            .commit()
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        Ok(claimed)
    }

    pub async fn record_target_blob_receipt(
        &self,
        claimed: &ClaimedAttachmentSecurityScanJobV1,
        receipt: AttachmentSecurityTargetBlobReceiptV1,
        recorded_at_unix_seconds: i64,
    ) -> Result<(), AttachmentSecurityPersistenceErrorV1> {
        if !valid_claim(claimed)
            || !valid_id16(&receipt.reference_id)
            || !valid_sha256(&receipt.receipt_sha256)
            || !valid_timestamp(recorded_at_unix_seconds)
        {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.attachment_security_scan_jobs SET target_blob_reference_id = $5, target_blob_receipt_sha256 = $6 WHERE job_id = $1 AND state = 1 AND claimed_by = $2 AND attempt_count = $3 AND lease_expires_at_unix_seconds > $4 AND ((target_blob_reference_id IS NULL AND target_blob_receipt_sha256 IS NULL) OR (target_blob_reference_id = $5 AND target_blob_receipt_sha256 = $6))",
        )
        .bind(claimed.job_id.as_slice())
        .bind(&claimed.worker_id)
        .bind(
            i32::try_from(claimed.attempt_count)
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?,
        )
        .bind(recorded_at_unix_seconds)
        .bind(receipt.reference_id.as_slice())
        .bind(receipt.receipt_sha256.as_slice())
        .execute(&self.pool)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        if updated.rows_affected() != 1 {
            return Err(AttachmentSecurityPersistenceErrorV1::ClaimLost);
        }
        Ok(())
    }

    pub async fn retry_scan_job(
        &self,
        claimed: &ClaimedAttachmentSecurityScanJobV1,
        recorded_at_unix_seconds: i64,
        next_attempt_at_unix_seconds: i64,
    ) -> Result<RetryAttachmentSecurityScanJobOutcomeV1, AttachmentSecurityPersistenceErrorV1> {
        if !valid_claim(claimed)
            || !valid_timestamp(recorded_at_unix_seconds)
            || !valid_timestamp(next_attempt_at_unix_seconds)
        {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        let row = sqlx::query(
            "SELECT attempt_count, max_attempts FROM makosh_data.attachment_security_scan_jobs WHERE job_id = $1 AND state = 1 AND claimed_by = $2 AND attempt_count = $3 AND lease_expires_at_unix_seconds > $4 FOR UPDATE",
        )
        .bind(claimed.job_id.as_slice())
        .bind(&claimed.worker_id)
        .bind(
            i32::try_from(claimed.attempt_count)
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?,
        )
        .bind(recorded_at_unix_seconds)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?
        .ok_or(AttachmentSecurityPersistenceErrorV1::ClaimLost)?;
        let attempt_count: i32 = row
            .try_get("attempt_count")
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
        let max_attempts: i32 = row
            .try_get("max_attempts")
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
        let outcome = if attempt_count >= max_attempts {
            sqlx::query(
                "UPDATE makosh_data.attachment_security_scan_jobs SET state = 3, completed_at_unix_seconds = $4, claimed_by = NULL, lease_expires_at_unix_seconds = NULL WHERE job_id = $1 AND state = 1 AND claimed_by = $2 AND attempt_count = $3",
            )
            .bind(claimed.job_id.as_slice())
            .bind(&claimed.worker_id)
            .bind(attempt_count)
            .bind(recorded_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
            RetryAttachmentSecurityScanJobOutcomeV1::Exhausted
        } else {
            if next_attempt_at_unix_seconds <= recorded_at_unix_seconds {
                return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
            }
            sqlx::query(
                "UPDATE makosh_data.attachment_security_scan_jobs SET next_attempt_at_unix_seconds = $4, claimed_by = NULL, lease_expires_at_unix_seconds = NULL WHERE job_id = $1 AND state = 1 AND claimed_by = $2 AND attempt_count = $3",
            )
            .bind(claimed.job_id.as_slice())
            .bind(&claimed.worker_id)
            .bind(attempt_count)
            .bind(next_attempt_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
            RetryAttachmentSecurityScanJobOutcomeV1::Scheduled
        };
        transaction
            .commit()
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        Ok(outcome)
    }

    pub async fn complete_scan_job_with_outbox(
        &self,
        claimed: &ClaimedAttachmentSecurityScanJobV1,
        verdict_record: &AttachmentSafetyVerdictOutboxRecordV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), AttachmentSecurityPersistenceErrorV1> {
        if !valid_claim(claimed)
            || !valid_timestamp(completed_at_unix_seconds)
            || !valid_verdict_for_claim(claimed, verdict_record, completed_at_unix_seconds)
        {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let exact_record = verdict_record.record();
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        let current = sqlx::query(
            "SELECT job_id FROM makosh_data.attachment_security_scan_jobs WHERE job_id = $1 AND state = 1 AND claimed_by = $2 AND lease_expires_at_unix_seconds > $3 AND attempt_count = $4 FOR UPDATE",
        )
        .bind(claimed.job_id.as_slice())
        .bind(&claimed.worker_id)
        .bind(completed_at_unix_seconds)
        .bind(
            i32::try_from(claimed.attempt_count)
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?,
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        if current.is_none() {
            return Err(AttachmentSecurityPersistenceErrorV1::ClaimLost);
        }
        insert_exact_outbox(&mut transaction, exact_record, completed_at_unix_seconds).await?;
        let updated = sqlx::query(
            "UPDATE makosh_data.attachment_security_scan_jobs SET state = 2, completed_at_unix_seconds = $4, outbox_message_id = $5, claimed_by = NULL, lease_expires_at_unix_seconds = NULL WHERE job_id = $1 AND state = 1 AND claimed_by = $2 AND attempt_count = $3",
        )
        .bind(claimed.job_id.as_slice())
        .bind(&claimed.worker_id)
        .bind(
            i32::try_from(claimed.attempt_count)
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?,
        )
        .bind(completed_at_unix_seconds)
        .bind(exact_record.message_id().as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        if updated.rows_affected() != 1 {
            return Err(AttachmentSecurityPersistenceErrorV1::ClaimLost);
        }
        transaction
            .commit()
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)
    }

    pub async fn pending_verdict_outbox(
        &self,
        limit: u32,
    ) -> Result<Vec<OutboxRecordV1>, AttachmentSecurityPersistenceErrorV1> {
        if !(1..=256).contains(&limit) {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let rows = sqlx::query(
            "SELECT exact_envelope_bytes FROM makosh_data.attachment_security_verdict_outbox WHERE published_at_unix_seconds IS NULL ORDER BY created_at_unix_seconds ASC, message_id ASC LIMIT $1",
        )
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        rows.into_iter()
            .map(|row| {
                let bytes = row
                    .try_get::<Vec<u8>, _>("exact_envelope_bytes")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
                OutboxRecordV1::accept(bytes)
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)
            })
            .collect()
    }

    pub async fn mark_verdict_outbox_published(
        &self,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, AttachmentSecurityPersistenceErrorV1> {
        if !valid_id16(&message_id) || !valid_timestamp(published_at_unix_seconds) {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let result = sqlx::query(
            "UPDATE makosh_data.attachment_security_verdict_outbox SET published_at_unix_seconds = $2 WHERE message_id = $1 AND published_at_unix_seconds IS NULL",
        )
        .bind(message_id.as_slice())
        .bind(published_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        Ok(result.rows_affected() == 1)
    }
}

#[must_use]
pub fn attachment_security_scan_job_id_v1(job: &AttachmentSecurityScanJobV1) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.attachment-security.scan-job.v1\0");
    digest.update(job.candidate_message_id);
    digest.update(job.canonical_state_message_id);
    digest.update(job.attachment_anchor_id);
    let value: [u8; 32] = digest.finalize().into();
    value[..16].try_into().expect("fixed SHA-256 prefix")
}

async fn insert_exact_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    record: &OutboxRecordV1,
    created_at_unix_seconds: i64,
) -> Result<(), AttachmentSecurityPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.attachment_security_verdict_outbox (message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds) VALUES ($1, $2, $3, $4) ON CONFLICT (message_id) DO NOTHING",
    )
    .bind(record.message_id().as_slice())
    .bind(record.envelope_sha256().as_slice())
    .bind(record.exact_bytes())
    .bind(created_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    if inserted.rows_affected() == 1 {
        return Ok(());
    }
    let row = sqlx::query(
        "SELECT envelope_sha256, exact_envelope_bytes FROM makosh_data.attachment_security_verdict_outbox WHERE message_id = $1",
    )
    .bind(record.message_id().as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    let stored_sha256 = id32(
        &row.try_get::<Vec<u8>, _>("envelope_sha256")
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
    )?;
    let stored_bytes = row
        .try_get::<Vec<u8>, _>("exact_envelope_bytes")
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
    if stored_sha256 == *record.envelope_sha256() && stored_bytes == record.exact_bytes() {
        Ok(())
    } else {
        Err(AttachmentSecurityPersistenceErrorV1::OutboxHashConflict)
    }
}

fn claimed_job_from_row(
    row: sqlx::postgres::PgRow,
    worker_id: &str,
) -> Result<ClaimedAttachmentSecurityScanJobV1, AttachmentSecurityPersistenceErrorV1> {
    Ok(ClaimedAttachmentSecurityScanJobV1 {
        job_id: id16(
            &row.try_get::<Vec<u8>, _>("job_id")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        )?,
        job: AttachmentSecurityScanJobV1 {
            candidate_message_id: id16(
                &row.try_get::<Vec<u8>, _>("candidate_message_id")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            canonical_state_message_id: id16(
                &row.try_get::<Vec<u8>, _>("canonical_state_message_id")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            attachment_anchor_id: id16(
                &row.try_get::<Vec<u8>, _>("attachment_anchor_id")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            blob_reference_id: id16(
                &row.try_get::<Vec<u8>, _>("blob_reference_id")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            declared_size: u64::try_from(
                row.try_get::<i64, _>("declared_size")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            blob_receipt_sha256: id32(
                &row.try_get::<Vec<u8>, _>("blob_receipt_sha256")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            causation_message_id: id16(
                &row.try_get::<Vec<u8>, _>("causation_message_id")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
            correlation_id: id16(
                &row.try_get::<Vec<u8>, _>("correlation_id")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )?,
        },
        candidate_envelope_sha256: id32(
            &row.try_get::<Vec<u8>, _>("candidate_envelope_sha256")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        )?,
        custody_transfer_source_proof: row
            .try_get("custody_transfer_source_proof")
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        target_blob_receipt: target_blob_receipt_from_row(&row)?,
        worker_id: worker_id.to_owned(),
        attempt_count: u32::try_from(
            row.try_get::<i32, _>("attempt_count")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        )
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        max_attempts: u32::try_from(
            row.try_get::<i32, _>("max_attempts")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        )
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
    })
}

fn valid_claim(value: &ClaimedAttachmentSecurityScanJobV1) -> bool {
    valid_id16(&value.job_id)
        && valid_sha256(&value.candidate_envelope_sha256)
        && (1..=2_048).contains(&value.custody_transfer_source_proof.len())
        && value.target_blob_receipt.is_none_or(|receipt| {
            valid_id16(&receipt.reference_id) && valid_sha256(&receipt.receipt_sha256)
        })
        && valid_worker_id(&value.worker_id)
        && value.attempt_count > 0
        && value.attempt_count <= value.max_attempts
        && (1..=32).contains(&value.max_attempts)
}

fn target_blob_receipt_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<Option<AttachmentSecurityTargetBlobReceiptV1>, AttachmentSecurityPersistenceErrorV1> {
    let reference_id = row
        .try_get::<Option<Vec<u8>>, _>("target_blob_reference_id")
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
    let receipt_sha256 = row
        .try_get::<Option<Vec<u8>>, _>("target_blob_receipt_sha256")
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
    match (reference_id, receipt_sha256) {
        (None, None) => Ok(None),
        (Some(reference_id), Some(receipt_sha256)) => {
            Ok(Some(AttachmentSecurityTargetBlobReceiptV1 {
                reference_id: id16(&reference_id)?,
                receipt_sha256: id32(&receipt_sha256)?,
            }))
        }
        _ => Err(AttachmentSecurityPersistenceErrorV1::InvalidRow),
    }
}

fn valid_verdict_for_claim(
    claimed: &ClaimedAttachmentSecurityScanJobV1,
    verdict_record: &AttachmentSafetyVerdictOutboxRecordV1,
    completed_at_unix_seconds: i64,
) -> bool {
    let fact = verdict_record.fact();
    fact.attachment_anchor_id == claimed.job.attachment_anchor_id
        && fact.causation_message_id == claimed.job.causation_message_id
        && fact.correlation_id == claimed.job.correlation_id
        && fact.expected_state == AttachmentSafetyExpectedStateV1::BlobAdmitted
        && matches!(
            fact.verdict,
            AttachmentSafetyVerdictV1::SafeForDelivery | AttachmentSafetyVerdictV1::Quarantined
        )
        && fact.observed_at_unix_seconds <= completed_at_unix_seconds
}

fn valid_worker_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_communications_attachment_contract::{
        AttachmentObservationEnvelopeContextV1, AttachmentSafetyVerdictFactV1,
        build_attachment_safety_verdict_outbox_record_v1,
    };

    #[test]
    fn scan_job_identity_is_deterministic_and_input_scoped() {
        let job = job();
        let mut other = job.clone();
        other.canonical_state_message_id = [9; 16];

        assert_eq!(
            attachment_security_scan_job_id_v1(&job),
            attachment_security_scan_job_id_v1(&job)
        );
        assert_ne!(
            attachment_security_scan_job_id_v1(&job),
            attachment_security_scan_job_id_v1(&other)
        );
    }

    #[test]
    fn worker_identity_is_bounded_and_provider_neutral() {
        assert!(valid_worker_id("attachment-security-1"));
        assert!(!valid_worker_id(""));
        assert!(!valid_worker_id("Mail Worker"));
    }

    #[test]
    fn completion_accepts_only_a_matching_blob_admitted_verdict() {
        let claimed = ClaimedAttachmentSecurityScanJobV1 {
            job_id: [8; 16],
            job: job(),
            candidate_envelope_sha256: [11; 32],
            custody_transfer_source_proof: vec![12; 64],
            target_blob_receipt: None,
            worker_id: "attachment-security-1".to_owned(),
            attempt_count: 1,
            max_attempts: 3,
        };
        let fact = AttachmentSafetyVerdictFactV1 {
            attachment_anchor_id: claimed.job.attachment_anchor_id,
            evidence_id: [9; 16],
            causation_message_id: claimed.job.causation_message_id,
            correlation_id: claimed.job.correlation_id,
            expected_state: AttachmentSafetyExpectedStateV1::BlobAdmitted,
            verdict: AttachmentSafetyVerdictV1::SafeForDelivery,
            observed_at_unix_seconds: 1_700_000_000,
        };
        let record = build_attachment_safety_verdict_outbox_record_v1(
            &fact,
            &AttachmentObservationEnvelopeContextV1 {
                runtime_instance_id: "attachment-security-runtime-test".to_owned(),
                runtime_generation: 2,
                module_id: "attachment-security-runtime".to_owned(),
                recorded_at_unix_seconds: 1_700_000_001,
                recorded_at_nanos: 0,
            },
        )
        .expect("verdict record");

        assert!(valid_verdict_for_claim(&claimed, &record, 1_700_000_001));

        let mut wrong_correlation = fact;
        wrong_correlation.correlation_id = [10; 16];
        let wrong_record = build_attachment_safety_verdict_outbox_record_v1(
            &wrong_correlation,
            &AttachmentObservationEnvelopeContextV1 {
                runtime_instance_id: "attachment-security-runtime-test".to_owned(),
                runtime_generation: 2,
                module_id: "attachment-security-runtime".to_owned(),
                recorded_at_unix_seconds: 1_700_000_001,
                recorded_at_nanos: 0,
            },
        )
        .expect("wrong correlation remains a valid contract record");
        assert!(!valid_verdict_for_claim(
            &claimed,
            &wrong_record,
            1_700_000_001
        ));
    }

    fn job() -> AttachmentSecurityScanJobV1 {
        AttachmentSecurityScanJobV1 {
            candidate_message_id: [1; 16],
            canonical_state_message_id: [2; 16],
            attachment_anchor_id: [3; 16],
            blob_reference_id: [4; 16],
            declared_size: 5,
            blob_receipt_sha256: [6; 32],
            causation_message_id: [2; 16],
            correlation_id: [7; 16],
        }
    }
}
