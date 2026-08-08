use makosh_attachment_archive_inspection_core::{
    ArchiveInspectionErrorV1, ArchiveInspectionReportV1, ArchiveInspectionRequestV1,
    ArchiveInspectionStateV1, ArchiveInspectionStatusV1, ArchiveInspectionTransitionV1,
    transition_archive_inspection_status_v1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    ARCHIVE_INSPECTION_MAX_ATTEMPTS_V1, ArchiveInspectionLeaseV1,
    ArchiveInspectionPersistenceErrorV1, ArchiveInspectionTargetBlobReceiptV1,
    AttachmentArchiveInspectionPersistenceV1, ClaimedArchiveInspectionJobV1,
    archive_inspection_job_id_v1, id16, id32,
    model::{
        archive_inspection_terminal_evidence_id_v1, entry_kind_code, state_from_code, valid_owner,
        valid_report, valid_sha256, valid_timestamp_millis, valid_worker,
    },
    positive_u32,
    runs::persist_status,
    unsigned,
};

const MAX_LEASE_MILLIS_V1: u64 = 300_000;

pub(crate) struct ArchiveInspectionDelegatedWorkV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub candidate_message_id: [u8; 16],
    pub safety_message_id: [u8; 16],
    pub delegation_request_id: [u8; 16],
    pub delegation_result_message_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub source_reference_id: [u8; 16],
    pub declared_size: u64,
    pub blob_receipt_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
    pub safety_evidence_id: [u8; 16],
}

impl AttachmentArchiveInspectionPersistenceV1 {
    pub async fn claim_next_job(
        &self,
        logical_owner_id: &str,
        worker_id: &str,
        runtime_generation: u64,
        grant_epoch: u64,
        now_unix_millis: i64,
        lease_millis: u64,
    ) -> Result<Option<ClaimedArchiveInspectionJobV1>, ArchiveInspectionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_worker(worker_id)
            || runtime_generation == 0
            || grant_epoch == 0
            || !valid_timestamp_millis(now_unix_millis)
            || !(1..=MAX_LEASE_MILLIS_V1).contains(&lease_millis)
        {
            return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
        }
        let lease_expires_at_unix_millis = now_unix_millis
            .checked_add(
                i64::try_from(lease_millis)
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?,
            )
            .ok_or(ArchiveInspectionPersistenceErrorV1::InvalidInput)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        let row = sqlx::query(
            "SELECT job_id FROM makosh_data.attachment_archive_inspection_jobs WHERE logical_owner_id = $1 AND state = 1 AND attempt_count < max_attempts ORDER BY created_at_unix_millis, job_id FOR UPDATE SKIP LOCKED LIMIT 1",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        let Some(row) = row else {
            transaction
                .commit()
                .await
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
            return Ok(None);
        };
        let job_id = id16(
            row.try_get::<Vec<u8>, _>("job_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )?;
        let updated = sqlx::query(
            "UPDATE makosh_data.attachment_archive_inspection_jobs SET state = 2, attempt_count = attempt_count + 1, worker_id = $3, runtime_generation = $4, grant_epoch = $5, lease_fence = lease_fence + 1, lease_expires_at_unix_millis = $6, updated_at_unix_millis = $7 WHERE logical_owner_id = $1 AND job_id = $2 AND state = 1",
        )
        .bind(logical_owner_id)
        .bind(job_id.as_slice())
        .bind(worker_id)
        .bind(i64::try_from(runtime_generation).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
        .bind(i64::try_from(grant_epoch).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
        .bind(lease_expires_at_unix_millis)
        .bind(now_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if updated != 1 {
            return Err(ArchiveInspectionPersistenceErrorV1::ClaimLost);
        }
        let claimed = load_claimed_job(&mut transaction, logical_owner_id, job_id).await?;
        transaction
            .commit()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        Ok(Some(claimed))
    }

    pub async fn complete_job(
        &self,
        claimed: &ClaimedArchiveInspectionJobV1,
        report: &ArchiveInspectionReportV1,
        completed_at_unix_millis: i64,
    ) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
        if !valid_claim(claimed)
            || !valid_report(report)
            || !valid_timestamp_millis(completed_at_unix_millis)
            || completed_at_unix_millis > claimed.lease.lease_expires_at_unix_millis
        {
            return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        verify_claim(&mut transaction, claimed, completed_at_unix_millis).await?;
        insert_report(
            &mut transaction,
            &claimed.logical_owner_id,
            claimed.request.run_id,
            report,
            completed_at_unix_millis,
        )
        .await?;
        finish_job(&mut transaction, claimed, completed_at_unix_millis).await?;
        transition_run_from_inspecting(
            &mut transaction,
            &claimed.logical_owner_id,
            claimed.request.run_id,
            ArchiveInspectionTransitionV1::Complete(report.clone()),
            None,
            completed_at_unix_millis,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)
    }

    pub async fn record_target_blob_receipt(
        &self,
        claimed: &ClaimedArchiveInspectionJobV1,
        receipt: ArchiveInspectionTargetBlobReceiptV1,
        recorded_at_unix_millis: i64,
    ) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
        if !valid_claim(claimed)
            || !valid_sha256(&receipt.receipt_sha256)
            || receipt.reference_id.iter().all(|byte| *byte == 0)
            || !valid_timestamp_millis(recorded_at_unix_millis)
            || recorded_at_unix_millis > claimed.lease.lease_expires_at_unix_millis
        {
            return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.attachment_archive_inspection_jobs
             SET target_reference_id = $8, target_receipt_sha256 = $9,
                 updated_at_unix_millis = $7
             WHERE logical_owner_id = $1 AND job_id = $2 AND state = 2
               AND worker_id = $3 AND runtime_generation = $4
               AND grant_epoch = $5 AND lease_fence = $6
               AND lease_expires_at_unix_millis > $7
               AND (
                 (target_reference_id IS NULL AND target_receipt_sha256 IS NULL)
                 OR (target_reference_id = $8 AND target_receipt_sha256 = $9)
               )",
        )
        .bind(&claimed.logical_owner_id)
        .bind(claimed.job_id.as_slice())
        .bind(&claimed.lease.worker_id)
        .bind(
            i64::try_from(claimed.lease.runtime_generation)
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?,
        )
        .bind(
            i64::try_from(claimed.lease.grant_epoch)
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?,
        )
        .bind(
            i64::try_from(claimed.lease.lease_fence)
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?,
        )
        .bind(recorded_at_unix_millis)
        .bind(receipt.reference_id.as_slice())
        .bind(receipt.receipt_sha256.as_slice())
        .execute(&self.pool)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        if updated.rows_affected() != 1 {
            return Err(ArchiveInspectionPersistenceErrorV1::ClaimLost);
        }
        Ok(())
    }

    pub async fn reject_job(
        &self,
        claimed: &ClaimedArchiveInspectionJobV1,
        error: ArchiveInspectionErrorV1,
        completed_at_unix_millis: i64,
    ) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
        if !valid_claim(claimed)
            || !valid_timestamp_millis(completed_at_unix_millis)
            || completed_at_unix_millis > claimed.lease.lease_expires_at_unix_millis
        {
            return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        verify_claim(&mut transaction, claimed, completed_at_unix_millis).await?;
        finish_job(&mut transaction, claimed, completed_at_unix_millis).await?;
        transition_run_from_inspecting(
            &mut transaction,
            &claimed.logical_owner_id,
            claimed.request.run_id,
            ArchiveInspectionTransitionV1::Reject(error),
            Some(error),
            completed_at_unix_millis,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)
    }

    pub async fn recover_expired_jobs(
        &self,
        logical_owner_id: &str,
        now_unix_millis: i64,
    ) -> Result<u32, ArchiveInspectionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || !valid_timestamp_millis(now_unix_millis) {
            return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        let retry_rows = sqlx::query(
            "UPDATE makosh_data.attachment_archive_inspection_jobs SET state = 1, worker_id = NULL, runtime_generation = NULL, grant_epoch = NULL, lease_expires_at_unix_millis = NULL, updated_at_unix_millis = $2 WHERE logical_owner_id = $1 AND state = 2 AND lease_expires_at_unix_millis <= $2 AND attempt_count < max_attempts RETURNING job_id",
        )
        .bind(logical_owner_id)
        .bind(now_unix_millis)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        let exhausted = sqlx::query(
            "SELECT job_id, run_id FROM makosh_data.attachment_archive_inspection_jobs WHERE logical_owner_id = $1 AND state = 2 AND lease_expires_at_unix_millis <= $2 AND attempt_count >= max_attempts ORDER BY job_id FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(now_unix_millis)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        for row in &exhausted {
            let job_id = id16(
                row.try_get::<Vec<u8>, _>("job_id")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?;
            let run_id = id16(
                row.try_get::<Vec<u8>, _>("run_id")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?;
            let updated = sqlx::query(
                "UPDATE makosh_data.attachment_archive_inspection_jobs SET state = 3, worker_id = NULL, runtime_generation = NULL, grant_epoch = NULL, lease_expires_at_unix_millis = NULL, updated_at_unix_millis = $3 WHERE logical_owner_id = $1 AND job_id = $2 AND state = 2",
            )
            .bind(logical_owner_id)
            .bind(job_id.as_slice())
            .bind(now_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?
            .rows_affected();
            if updated != 1 {
                return Err(ArchiveInspectionPersistenceErrorV1::ClaimLost);
            }
            transition_run_from_inspecting(
                &mut transaction,
                logical_owner_id,
                run_id,
                ArchiveInspectionTransitionV1::Reject(ArchiveInspectionErrorV1::Unavailable),
                Some(ArchiveInspectionErrorV1::Unavailable),
                now_unix_millis,
            )
            .await?;
        }
        let recovered = retry_rows
            .len()
            .checked_add(exhausted.len())
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
        transaction
            .commit()
            .await
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
        Ok(recovered)
    }
}

pub(crate) async fn enqueue_archive_inspection_work(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    work: &ArchiveInspectionDelegatedWorkV1,
    created_at_unix_millis: i64,
) -> Result<[u8; 16], ArchiveInspectionPersistenceErrorV1> {
    let request = ArchiveInspectionRequestV1 {
        run_id: work.run_id,
        operation_id: work.operation_id,
        attachment_anchor_id: work.attachment_anchor_id,
    };
    let job_id = archive_inspection_job_id_v1(
        &request,
        work.candidate_message_id,
        work.safety_message_id,
        work.delegation_request_id,
        work.delegation_result_message_id,
    );
    sqlx::query(
        "INSERT INTO makosh_data.attachment_archive_inspection_jobs (logical_owner_id, job_id, run_id, candidate_message_id, safety_message_id, delegation_request_id, delegation_result_message_id, attachment_anchor_id, source_reference_id, declared_size, blob_receipt_sha256, custody_transfer_source_proof, safety_evidence_id, state, attempt_count, max_attempts, worker_id, runtime_generation, grant_epoch, lease_fence, lease_expires_at_unix_millis, created_at_unix_millis, updated_at_unix_millis) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 1, 0, $14, NULL, NULL, NULL, 0, NULL, $15, $15) ON CONFLICT (logical_owner_id, run_id) DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(job_id.as_slice())
    .bind(work.run_id.as_slice())
    .bind(work.candidate_message_id.as_slice())
    .bind(work.safety_message_id.as_slice())
    .bind(work.delegation_request_id.as_slice())
    .bind(work.delegation_result_message_id.as_slice())
    .bind(work.attachment_anchor_id.as_slice())
    .bind(work.source_reference_id.as_slice())
    .bind(i64::try_from(work.declared_size).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
    .bind(work.blob_receipt_sha256.as_slice())
    .bind(&work.custody_transfer_source_proof)
    .bind(work.safety_evidence_id.as_slice())
    .bind(i32::try_from(ARCHIVE_INSPECTION_MAX_ATTEMPTS_V1).expect("bounded max attempts"))
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    let row = sqlx::query(
        "SELECT job_id, candidate_message_id, safety_message_id, delegation_request_id, delegation_result_message_id, attachment_anchor_id, source_reference_id, declared_size, blob_receipt_sha256, custody_transfer_source_proof, safety_evidence_id FROM makosh_data.attachment_archive_inspection_jobs WHERE logical_owner_id = $1 AND run_id = $2",
    )
    .bind(logical_owner_id)
    .bind(work.run_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    let exact = id16(
        row.try_get::<Vec<u8>, _>("job_id")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
            .as_slice(),
    )? == job_id
        && id16(
            row.try_get::<Vec<u8>, _>("candidate_message_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )? == work.candidate_message_id
        && id16(
            row.try_get::<Vec<u8>, _>("safety_message_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )? == work.safety_message_id
        && id16(
            row.try_get::<Vec<u8>, _>("delegation_request_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )? == work.delegation_request_id
        && id16(
            row.try_get::<Vec<u8>, _>("delegation_result_message_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )? == work.delegation_result_message_id
        && id16(
            row.try_get::<Vec<u8>, _>("attachment_anchor_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )? == work.attachment_anchor_id
        && id16(
            row.try_get::<Vec<u8>, _>("source_reference_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )? == work.source_reference_id
        && unsigned(
            row.try_get("declared_size")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        )? == work.declared_size
        && id32(
            row.try_get::<Vec<u8>, _>("blob_receipt_sha256")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )? == work.blob_receipt_sha256
        && row
            .try_get::<Vec<u8>, _>("custody_transfer_source_proof")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
            == work.custody_transfer_source_proof
        && id16(
            row.try_get::<Vec<u8>, _>("safety_evidence_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )? == work.safety_evidence_id;
    if !exact {
        return Err(ArchiveInspectionPersistenceErrorV1::EvidenceConflict);
    }
    Ok(job_id)
}

async fn load_claimed_job(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    job_id: [u8; 16],
) -> Result<ClaimedArchiveInspectionJobV1, ArchiveInspectionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT jobs.job_id, jobs.run_id, runs.operation_id, jobs.candidate_message_id,
                jobs.safety_message_id, jobs.delegation_request_id,
                jobs.delegation_result_message_id,
                result_inbox.envelope_sha256 AS delegation_result_envelope_sha256,
                jobs.attachment_anchor_id, jobs.source_reference_id,
                jobs.target_reference_id, jobs.target_receipt_sha256,
                jobs.declared_size, jobs.blob_receipt_sha256,
                jobs.custody_transfer_source_proof, jobs.safety_evidence_id,
                jobs.attempt_count, jobs.max_attempts, jobs.worker_id,
                jobs.runtime_generation, jobs.grant_epoch, jobs.lease_fence,
                jobs.lease_expires_at_unix_millis
         FROM makosh_data.attachment_archive_inspection_jobs jobs
         JOIN makosh_data.attachment_archive_inspection_runs runs
           ON runs.logical_owner_id = jobs.logical_owner_id
          AND runs.run_id = jobs.run_id
         JOIN makosh_data.attachment_archive_inspection_custody_result_inbox result_inbox
           ON result_inbox.logical_owner_id = jobs.logical_owner_id
          AND result_inbox.message_id = jobs.delegation_result_message_id
         WHERE jobs.logical_owner_id = $1 AND jobs.job_id = $2 AND jobs.state = 2",
    )
    .bind(logical_owner_id)
    .bind(job_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::ClaimLost)?;
    let run_id = id16(
        row.try_get::<Vec<u8>, _>("run_id")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
            .as_slice(),
    )?;
    Ok(ClaimedArchiveInspectionJobV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        job_id,
        request: ArchiveInspectionRequestV1 {
            run_id,
            operation_id: id16(
                row.try_get::<Vec<u8>, _>("operation_id")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?,
            attachment_anchor_id: id16(
                row.try_get::<Vec<u8>, _>("attachment_anchor_id")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                    .as_slice(),
            )?,
        },
        candidate_message_id: id16(
            row.try_get::<Vec<u8>, _>("candidate_message_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )?,
        safety_message_id: id16(
            row.try_get::<Vec<u8>, _>("safety_message_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )?,
        delegation_request_id: id16(
            row.try_get::<Vec<u8>, _>("delegation_request_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )?,
        delegation_result_message_id: id16(
            row.try_get::<Vec<u8>, _>("delegation_result_message_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )?,
        delegation_result_envelope_sha256: id32(
            row.try_get::<Vec<u8>, _>("delegation_result_envelope_sha256")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )?,
        source_reference_id: id16(
            row.try_get::<Vec<u8>, _>("source_reference_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )?,
        target_blob_receipt: target_blob_receipt_from_row(&row)?,
        declared_size: unsigned(
            row.try_get("declared_size")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        )?,
        blob_receipt_sha256: id32(
            row.try_get::<Vec<u8>, _>("blob_receipt_sha256")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )?,
        custody_transfer_source_proof: row
            .try_get("custody_transfer_source_proof")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        safety_evidence_id: id16(
            row.try_get::<Vec<u8>, _>("safety_evidence_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
                .as_slice(),
        )?,
        attempt_count: positive_u32(
            row.try_get("attempt_count")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        )?,
        max_attempts: positive_u32(
            row.try_get("max_attempts")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        )?,
        lease: ArchiveInspectionLeaseV1 {
            worker_id: row
                .try_get("worker_id")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
            runtime_generation: unsigned(
                row.try_get("runtime_generation")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
            )?,
            grant_epoch: unsigned(
                row.try_get("grant_epoch")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
            )?,
            lease_fence: unsigned(
                row.try_get("lease_fence")
                    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
            )?,
            lease_expires_at_unix_millis: row
                .try_get("lease_expires_at_unix_millis")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        },
    })
}

async fn verify_claim(
    transaction: &mut Transaction<'_, Postgres>,
    claimed: &ClaimedArchiveInspectionJobV1,
    completed_at_unix_millis: i64,
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT worker_id, runtime_generation, grant_epoch, lease_fence, lease_expires_at_unix_millis FROM makosh_data.attachment_archive_inspection_jobs WHERE logical_owner_id = $1 AND job_id = $2 AND state = 2 FOR UPDATE",
    )
    .bind(&claimed.logical_owner_id)
    .bind(claimed.job_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?
    .ok_or(ArchiveInspectionPersistenceErrorV1::ClaimLost)?;
    let worker_id: String = row
        .try_get("worker_id")
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    let runtime_generation = unsigned(
        row.try_get("runtime_generation")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
    )?;
    let grant_epoch = unsigned(
        row.try_get("grant_epoch")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
    )?;
    let lease_fence = unsigned(
        row.try_get("lease_fence")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
    )?;
    let lease_expires_at_unix_millis: i64 = row
        .try_get("lease_expires_at_unix_millis")
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    if worker_id != claimed.lease.worker_id
        || runtime_generation != claimed.lease.runtime_generation
        || grant_epoch != claimed.lease.grant_epoch
        || lease_fence != claimed.lease.lease_fence
        || lease_expires_at_unix_millis != claimed.lease.lease_expires_at_unix_millis
        || completed_at_unix_millis > lease_expires_at_unix_millis
    {
        return Err(ArchiveInspectionPersistenceErrorV1::ClaimLost);
    }
    Ok(())
}

async fn finish_job(
    transaction: &mut Transaction<'_, Postgres>,
    claimed: &ClaimedArchiveInspectionJobV1,
    completed_at_unix_millis: i64,
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    let updated = sqlx::query(
        "UPDATE makosh_data.attachment_archive_inspection_jobs SET state = 3, worker_id = NULL, runtime_generation = NULL, grant_epoch = NULL, lease_expires_at_unix_millis = NULL, updated_at_unix_millis = $3 WHERE logical_owner_id = $1 AND job_id = $2 AND state = 2 AND lease_fence = $4",
    )
    .bind(&claimed.logical_owner_id)
    .bind(claimed.job_id.as_slice())
    .bind(completed_at_unix_millis)
    .bind(i64::try_from(claimed.lease.lease_fence).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
    .execute(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?
    .rows_affected();
    if updated != 1 {
        return Err(ArchiveInspectionPersistenceErrorV1::ClaimLost);
    }
    Ok(())
}

async fn insert_report(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    report: &ArchiveInspectionReportV1,
    completed_at_unix_millis: i64,
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.attachment_archive_inspection_reports (logical_owner_id, run_id, entry_count, total_uncompressed_bytes, completed_at_unix_millis) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(i32::try_from(report.entry_count).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
    .bind(i64::try_from(report.total_uncompressed_bytes).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
    .bind(completed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    for (ordinal, entry) in report.entries.iter().enumerate() {
        sqlx::query(
            "INSERT INTO makosh_data.attachment_archive_inspection_report_entries (logical_owner_id, run_id, entry_ordinal, normalized_path_utf8, compressed_size, uncompressed_size, entry_kind) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .bind(i32::try_from(ordinal).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
        .bind(entry.normalized_path.as_bytes())
        .bind(i64::try_from(entry.compressed_size).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
        .bind(i64::try_from(entry.uncompressed_size).map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?)
        .bind(entry_kind_code(entry.kind))
        .execute(&mut **transaction)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    }
    Ok(())
}

async fn transition_run_from_inspecting(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    transition: ArchiveInspectionTransitionV1,
    terminal_error: Option<ArchiveInspectionErrorV1>,
    occurred_at_unix_millis: i64,
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT state, state_revision FROM makosh_data.attachment_archive_inspection_runs WHERE logical_owner_id = $1 AND run_id = $2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    let state = state_from_code(
        row.try_get("state")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
    )?;
    let revision = unsigned(
        row.try_get("state_revision")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
    )?;
    if state != ArchiveInspectionStateV1::Inspecting {
        return Err(ArchiveInspectionPersistenceErrorV1::ClaimLost);
    }
    let next = transition_archive_inspection_status_v1(
        &ArchiveInspectionStatusV1 {
            state,
            state_revision: revision,
            report: None,
            error: None,
        },
        transition,
    )
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    let rejection_evidence_id =
        terminal_error.map(|error| archive_inspection_terminal_evidence_id_v1(run_id, error));
    persist_status(
        transaction,
        logical_owner_id,
        run_id,
        revision,
        &next,
        rejection_evidence_id,
        occurred_at_unix_millis,
    )
    .await
}

fn valid_claim(claimed: &ClaimedArchiveInspectionJobV1) -> bool {
    valid_owner(&claimed.logical_owner_id)
        && valid_id(&claimed.job_id)
        && valid_id(&claimed.request.run_id)
        && valid_id(&claimed.request.operation_id)
        && valid_id(&claimed.request.attachment_anchor_id)
        && valid_id(&claimed.candidate_message_id)
        && valid_id(&claimed.safety_message_id)
        && valid_id(&claimed.delegation_request_id)
        && valid_id(&claimed.delegation_result_message_id)
        && valid_sha256(&claimed.delegation_result_envelope_sha256)
        && valid_id(&claimed.source_reference_id)
        && claimed.target_blob_receipt.is_none_or(|receipt| {
            valid_id(&receipt.reference_id) && valid_sha256(&receipt.receipt_sha256)
        })
        && claimed.declared_size > 0
        && valid_sha256(&claimed.blob_receipt_sha256)
        && (1..=2_048).contains(&claimed.custody_transfer_source_proof.len())
        && valid_id(&claimed.safety_evidence_id)
        && claimed.attempt_count > 0
        && claimed.attempt_count <= claimed.max_attempts
        && claimed.max_attempts <= 32
        && valid_worker(&claimed.lease.worker_id)
        && claimed.lease.runtime_generation > 0
        && claimed.lease.grant_epoch > 0
        && claimed.lease.lease_fence > 0
        && valid_timestamp_millis(claimed.lease.lease_expires_at_unix_millis)
}

fn target_blob_receipt_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<Option<ArchiveInspectionTargetBlobReceiptV1>, ArchiveInspectionPersistenceErrorV1> {
    let reference_id = row
        .try_get::<Option<Vec<u8>>, _>("target_reference_id")
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    let receipt_sha256 = row
        .try_get::<Option<Vec<u8>>, _>("target_receipt_sha256")
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    match (reference_id, receipt_sha256) {
        (None, None) => Ok(None),
        (Some(reference_id), Some(receipt_sha256)) => {
            Ok(Some(ArchiveInspectionTargetBlobReceiptV1 {
                reference_id: id16(&reference_id)?,
                receipt_sha256: id32(&receipt_sha256)?,
            }))
        }
        _ => Err(ArchiveInspectionPersistenceErrorV1::InvalidRow),
    }
}

fn valid_id(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_validation_fences_runtime_generation_grant_epoch_and_lease() {
        let mut value = claim();
        assert!(valid_claim(&value));
        value.lease.runtime_generation = 0;
        assert!(!valid_claim(&value));
        value = claim();
        value.lease.grant_epoch = 0;
        assert!(!valid_claim(&value));
        value = claim();
        value.lease.lease_fence = 0;
        assert!(!valid_claim(&value));
    }

    fn claim() -> ClaimedArchiveInspectionJobV1 {
        ClaimedArchiveInspectionJobV1 {
            logical_owner_id: "owner-1".to_owned(),
            job_id: [1; 16],
            request: ArchiveInspectionRequestV1 {
                run_id: [2; 16],
                operation_id: [3; 16],
                attachment_anchor_id: [4; 16],
            },
            candidate_message_id: [5; 16],
            safety_message_id: [6; 16],
            delegation_request_id: [7; 16],
            delegation_result_message_id: [8; 16],
            delegation_result_envelope_sha256: [9; 32],
            source_reference_id: [9; 16],
            target_blob_receipt: None,
            declared_size: 512,
            blob_receipt_sha256: [10; 32],
            custody_transfer_source_proof: vec![11; 64],
            safety_evidence_id: [12; 16],
            attempt_count: 1,
            max_attempts: 8,
            lease: ArchiveInspectionLeaseV1 {
                worker_id: "worker-1".to_owned(),
                runtime_generation: 1,
                grant_epoch: 1,
                lease_fence: 1,
                lease_expires_at_unix_millis: 1_700_000_010_000,
            },
        }
    }
}
