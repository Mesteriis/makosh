//! Fenced durable render jobs and terminal derived-artifact metadata.

use makosh_attachment_preview_api::wire::{AttachmentPreviewErrorCodeV1, AttachmentPreviewStateV1};
use makosh_attachment_preview_core::{
    AttachmentPreviewTransitionV1, transition_attachment_preview_status_v1,
    validate_preview_output_v1,
};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AttachmentPreviewPersistenceErrorV1, AttachmentPreviewPersistenceV1,
    ClaimedAttachmentPreviewJobV1, PreviewJobLeaseV1, PreviewTargetBlobReceiptV1,
    RenderedAttachmentPreviewArtifactV1,
    model::{
        ATTACHMENT_PREVIEW_MAX_ATTEMPTS_V1, ATTACHMENT_PREVIEW_MAX_PROOF_BYTES_V1,
        ATTACHMENT_PREVIEW_MAX_SOURCE_BYTES_V1, attachment_preview_job_id_v1, content_type_code,
        kind_code, valid_id16, valid_owner, valid_sha256, valid_timestamp_millis, valid_worker,
    },
    repository::{
        append_realtime, id16, id32, invalid_row, load_run_for_update, storage_unavailable,
        update_run_status,
    },
};

const MAX_LEASE_MILLIS_V1: u64 = 300_000;

pub(crate) struct DelegatedPreviewWorkV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub delegation_request_id: [u8; 16],
    pub delegation_result_message_id: [u8; 16],
    pub delegation_result_envelope_sha256: [u8; 32],
    pub candidate_message_id: [u8; 16],
    pub safety_message_id: [u8; 16],
    pub source_reference_id: [u8; 16],
    pub source_receipt_sha256: [u8; 32],
    pub source_declared_size: u64,
    pub custody_transfer_source_proof: Vec<u8>,
}

pub(crate) async fn enqueue_preview_work(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    work: &DelegatedPreviewWorkV1,
    created_at_unix_millis: i64,
) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
    if !valid_owner(logical_owner_id)
        || !valid_id16(&work.run_id)
        || !valid_id16(&work.operation_id)
        || !valid_id16(&work.attachment_anchor_id)
        || !valid_id16(&work.delegation_request_id)
        || !valid_id16(&work.delegation_result_message_id)
        || !valid_sha256(&work.delegation_result_envelope_sha256)
        || !valid_id16(&work.candidate_message_id)
        || !valid_id16(&work.safety_message_id)
        || !valid_id16(&work.source_reference_id)
        || !valid_sha256(&work.source_receipt_sha256)
        || !(1..=ATTACHMENT_PREVIEW_MAX_SOURCE_BYTES_V1).contains(&work.source_declared_size)
        || !(1..=ATTACHMENT_PREVIEW_MAX_PROOF_BYTES_V1)
            .contains(&work.custody_transfer_source_proof.len())
        || !valid_timestamp_millis(created_at_unix_millis)
    {
        return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
    }
    let job_id = attachment_preview_job_id_v1(
        work.run_id,
        work.operation_id,
        work.attachment_anchor_id,
        work.delegation_request_id,
        work.delegation_result_message_id,
    );
    let proof_sha256: [u8; 32] = Sha256::digest(&work.custody_transfer_source_proof).into();
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.attachment_preview_jobs (logical_owner_id,job_id,run_id,request_id,result_message_id,result_envelope_sha256,attachment_anchor_id,candidate_message_id,safety_message_id,source_reference_id,source_receipt_sha256,source_declared_size,custody_transfer_source_proof,custody_proof_sha256,target_reference_id,target_receipt_sha256,state,attempt_count,max_attempts,worker_id,runtime_generation,grant_epoch,lease_fence,lease_expires_at_unix_millis,created_at_unix_millis,updated_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,NULL,NULL,1,0,$15,NULL,NULL,NULL,0,NULL,$16,$16) ON CONFLICT (logical_owner_id,run_id) DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(job_id.as_slice())
    .bind(work.run_id.as_slice())
    .bind(work.delegation_request_id.as_slice())
    .bind(work.delegation_result_message_id.as_slice())
    .bind(work.delegation_result_envelope_sha256.as_slice())
    .bind(work.attachment_anchor_id.as_slice())
    .bind(work.candidate_message_id.as_slice())
    .bind(work.safety_message_id.as_slice())
    .bind(work.source_reference_id.as_slice())
    .bind(work.source_receipt_sha256.as_slice())
    .bind(i64::try_from(work.source_declared_size).map_err(invalid_input)?)
    .bind(&work.custody_transfer_source_proof)
    .bind(proof_sha256.as_slice())
    .bind(i32::try_from(ATTACHMENT_PREVIEW_MAX_ATTEMPTS_V1).map_err(invalid_input)?)
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    if inserted.rows_affected() == 1 {
        return Ok(());
    }
    let existing = sqlx::query(
        "SELECT job_id,request_id,result_message_id,result_envelope_sha256,source_reference_id,source_receipt_sha256,source_declared_size,custody_proof_sha256 FROM makosh_data.attachment_preview_jobs WHERE logical_owner_id=$1 AND run_id=$2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(work.run_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    if id16(existing.try_get("job_id").map_err(invalid_row)?)? != job_id
        || id16(existing.try_get("request_id").map_err(invalid_row)?)? != work.delegation_request_id
        || id16(existing.try_get("result_message_id").map_err(invalid_row)?)?
            != work.delegation_result_message_id
        || id32(
            existing
                .try_get("result_envelope_sha256")
                .map_err(invalid_row)?,
        )? != work.delegation_result_envelope_sha256
        || id16(
            existing
                .try_get("source_reference_id")
                .map_err(invalid_row)?,
        )? != work.source_reference_id
        || id32(
            existing
                .try_get("source_receipt_sha256")
                .map_err(invalid_row)?,
        )? != work.source_receipt_sha256
        || u64::try_from(
            existing
                .try_get::<i64, _>("source_declared_size")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?
            != work.source_declared_size
        || id32(
            existing
                .try_get("custody_proof_sha256")
                .map_err(invalid_row)?,
        )? != proof_sha256
    {
        return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
    }
    Ok(())
}

impl AttachmentPreviewPersistenceV1 {
    pub async fn recover_expired_jobs(
        &self,
        logical_owner_id: &str,
        now_unix_millis: i64,
    ) -> Result<u32, AttachmentPreviewPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || !valid_timestamp_millis(now_unix_millis) {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let retry_count = sqlx::query(
            "UPDATE makosh_data.attachment_preview_jobs SET state=1,worker_id=NULL,runtime_generation=NULL,grant_epoch=NULL,lease_expires_at_unix_millis=NULL,updated_at_unix_millis=$2 WHERE logical_owner_id=$1 AND state=2 AND lease_expires_at_unix_millis<=$2 AND attempt_count<max_attempts",
        )
        .bind(logical_owner_id)
        .bind(now_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?
        .rows_affected();
        let exhausted = sqlx::query(
            "SELECT job_id,run_id FROM makosh_data.attachment_preview_jobs WHERE logical_owner_id=$1 AND state=2 AND lease_expires_at_unix_millis<=$2 AND attempt_count>=max_attempts ORDER BY job_id FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(now_unix_millis)
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        for row in &exhausted {
            let job_id = id16(row.try_get("job_id").map_err(invalid_row)?)?;
            let run_id = id16(row.try_get("run_id").map_err(invalid_row)?)?;
            let current = load_run_for_update(&mut transaction, logical_owner_id, run_id)
                .await?
                .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
            if current.status.state != AttachmentPreviewStateV1::Rendering {
                return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
            }
            let next = transition_attachment_preview_status_v1(
                &current.status,
                AttachmentPreviewTransitionV1::Reject(AttachmentPreviewErrorCodeV1::Unavailable),
            )
            .map_err(|_| AttachmentPreviewPersistenceErrorV1::EvidenceConflict)?;
            let changed = sqlx::query(
                "UPDATE makosh_data.attachment_preview_jobs SET state=4,worker_id=NULL,runtime_generation=NULL,grant_epoch=NULL,lease_expires_at_unix_millis=NULL,updated_at_unix_millis=$3 WHERE logical_owner_id=$1 AND job_id=$2 AND state=2",
            )
            .bind(logical_owner_id)
            .bind(job_id.as_slice())
            .bind(now_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(storage_unavailable)?
            .rows_affected();
            if changed != 1
                || !update_run_status(
                    &mut transaction,
                    logical_owner_id,
                    run_id,
                    current.status.state_revision,
                    &next,
                    now_unix_millis,
                )
                .await?
            {
                return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
            }
            append_realtime(
                &mut transaction,
                logical_owner_id,
                run_id,
                &next,
                now_unix_millis,
            )
            .await?;
        }
        let total = retry_count
            .checked_add(u64::try_from(exhausted.len()).map_err(invalid_row)?)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(total)
    }

    pub async fn claim_next_job(
        &self,
        logical_owner_id: &str,
        worker_id: &str,
        runtime_generation: u64,
        grant_epoch: u64,
        now_unix_millis: i64,
        lease_millis: u64,
    ) -> Result<Option<ClaimedAttachmentPreviewJobV1>, AttachmentPreviewPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_worker(worker_id)
            || runtime_generation == 0
            || grant_epoch == 0
            || !valid_timestamp_millis(now_unix_millis)
            || !(1..=MAX_LEASE_MILLIS_V1).contains(&lease_millis)
        {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        let lease_expires_at = now_unix_millis
            .checked_add(i64::try_from(lease_millis).map_err(invalid_input)?)
            .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidInput)?;
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let row = sqlx::query(
            "SELECT job_id FROM makosh_data.attachment_preview_jobs WHERE logical_owner_id=$1 AND attempt_count<max_attempts AND (state=1 OR (state=2 AND lease_expires_at_unix_millis<$2)) ORDER BY created_at_unix_millis,job_id FOR UPDATE SKIP LOCKED LIMIT 1",
        )
        .bind(logical_owner_id)
        .bind(now_unix_millis)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        let Some(row) = row else {
            transaction.commit().await.map_err(storage_unavailable)?;
            return Ok(None);
        };
        let job_id = id16(row.try_get("job_id").map_err(invalid_row)?)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.attachment_preview_jobs SET state=2,attempt_count=attempt_count+1,worker_id=$1,runtime_generation=$2,grant_epoch=$3,lease_fence=lease_fence+1,lease_expires_at_unix_millis=$4,updated_at_unix_millis=$5 WHERE logical_owner_id=$6 AND job_id=$7 RETURNING job_id,run_id,request_id,result_message_id,result_envelope_sha256,attachment_anchor_id,candidate_message_id,safety_message_id,source_reference_id,source_receipt_sha256,source_declared_size,custody_transfer_source_proof,target_reference_id,target_receipt_sha256,attempt_count,max_attempts,lease_fence,lease_expires_at_unix_millis",
        )
        .bind(worker_id)
        .bind(i64::try_from(runtime_generation).map_err(invalid_input)?)
        .bind(i64::try_from(grant_epoch).map_err(invalid_input)?)
        .bind(lease_expires_at)
        .bind(now_unix_millis)
        .bind(logical_owner_id)
        .bind(job_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        let run_id = id16(updated.try_get("run_id").map_err(invalid_row)?)?;
        let run = sqlx::query(
            "SELECT operation_id FROM makosh_data.attachment_preview_runs WHERE logical_owner_id=$1 AND run_id=$2 AND state=3 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_unavailable)?
        .ok_or(AttachmentPreviewPersistenceErrorV1::EvidenceConflict)?;
        let claimed = claimed_from_rows(
            logical_owner_id,
            worker_id,
            runtime_generation,
            grant_epoch,
            updated,
            id16(run.try_get("operation_id").map_err(invalid_row)?)?,
        )?;
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(Some(claimed))
    }

    pub async fn record_target_blob_receipt(
        &self,
        logical_owner_id: &str,
        job_id: [u8; 16],
        lease: &PreviewJobLeaseV1,
        receipt: PreviewTargetBlobReceiptV1,
        now_unix_millis: i64,
    ) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
        validate_lease_input(logical_owner_id, job_id, lease, now_unix_millis)?;
        if !valid_id16(&receipt.reference_id) || !valid_sha256(&receipt.receipt_sha256) {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        let changed = sqlx::query(
            "UPDATE makosh_data.attachment_preview_jobs SET target_reference_id=$1,target_receipt_sha256=$2,updated_at_unix_millis=$3 WHERE logical_owner_id=$4 AND job_id=$5 AND state=2 AND worker_id=$6 AND runtime_generation=$7 AND grant_epoch=$8 AND lease_fence=$9 AND lease_expires_at_unix_millis>=$3 AND target_reference_id IS NULL",
        )
        .bind(receipt.reference_id.as_slice())
        .bind(receipt.receipt_sha256.as_slice())
        .bind(now_unix_millis)
        .bind(logical_owner_id)
        .bind(job_id.as_slice())
        .bind(&lease.worker_id)
        .bind(i64::try_from(lease.runtime_generation).map_err(invalid_input)?)
        .bind(i64::try_from(lease.grant_epoch).map_err(invalid_input)?)
        .bind(i64::try_from(lease.lease_fence).map_err(invalid_input)?)
        .execute(&self.pool)
        .await
        .map_err(storage_unavailable)?
        .rows_affected();
        if changed == 1 {
            return Ok(());
        }
        let existing = sqlx::query(
            "SELECT target_reference_id,target_receipt_sha256 FROM makosh_data.attachment_preview_jobs WHERE logical_owner_id=$1 AND job_id=$2 AND state=2 AND worker_id=$3 AND runtime_generation=$4 AND grant_epoch=$5 AND lease_fence=$6 AND lease_expires_at_unix_millis>=$7",
        )
        .bind(logical_owner_id)
        .bind(job_id.as_slice())
        .bind(&lease.worker_id)
        .bind(i64::try_from(lease.runtime_generation).map_err(invalid_input)?)
        .bind(i64::try_from(lease.grant_epoch).map_err(invalid_input)?)
        .bind(i64::try_from(lease.lease_fence).map_err(invalid_input)?)
        .bind(now_unix_millis)
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_unavailable)?
        .ok_or(AttachmentPreviewPersistenceErrorV1::StaleFence)?;
        if existing
            .try_get::<Option<Vec<u8>>, _>("target_reference_id")
            .map_err(invalid_row)?
            .map(id16)
            .transpose()?
            == Some(receipt.reference_id)
            && existing
                .try_get::<Option<Vec<u8>>, _>("target_receipt_sha256")
                .map_err(invalid_row)?
                .map(id32)
                .transpose()?
                == Some(receipt.receipt_sha256)
        {
            Ok(())
        } else {
            Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict)
        }
    }

    pub async fn complete_job(
        &self,
        logical_owner_id: &str,
        job_id: [u8; 16],
        lease: &PreviewJobLeaseV1,
        artifact: RenderedAttachmentPreviewArtifactV1,
        committed_at_unix_millis: i64,
    ) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
        validate_lease_input(logical_owner_id, job_id, lease, committed_at_unix_millis)?;
        if !valid_id16(&artifact.target_blob_receipt.reference_id)
            || !valid_sha256(&artifact.target_blob_receipt.receipt_sha256)
            || !valid_sha256(&artifact.renderer_identity_sha256)
            || artifact.preview_kind
                == makosh_attachment_preview_api::wire::AttachmentPreviewKindV1::Unspecified
            || validate_preview_output_v1(artifact.content_type, artifact.preview_size_bytes)
                .is_err()
        {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let job = lock_leased_job(
            &mut transaction,
            logical_owner_id,
            job_id,
            lease,
            committed_at_unix_millis,
        )
        .await?;
        if job.target_blob_receipt != Some(artifact.target_blob_receipt) {
            return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
        }
        let current = load_run_for_update(&mut transaction, logical_owner_id, job.run_id)
            .await?
            .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
        if current.status.state != AttachmentPreviewStateV1::Rendering {
            return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.attachment_preview_artifacts (logical_owner_id,run_id,derived_reference_id,derived_receipt_sha256,source_receipt_sha256,renderer_identity_sha256,preview_kind,content_type,preview_size_bytes,truncated,runtime_generation,grant_epoch,committed_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) ON CONFLICT (logical_owner_id,run_id) DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(job.run_id.as_slice())
        .bind(artifact.target_blob_receipt.reference_id.as_slice())
        .bind(artifact.target_blob_receipt.receipt_sha256.as_slice())
        .bind(job.source_receipt_sha256.as_slice())
        .bind(artifact.renderer_identity_sha256.as_slice())
        .bind(kind_code(artifact.preview_kind))
        .bind(content_type_code(artifact.content_type))
        .bind(i64::try_from(artifact.preview_size_bytes).map_err(invalid_input)?)
        .bind(artifact.truncated)
        .bind(i64::try_from(lease.runtime_generation).map_err(invalid_input)?)
        .bind(i64::try_from(lease.grant_epoch).map_err(invalid_input)?)
        .bind(committed_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        if inserted.rows_affected() != 1 {
            return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
        }
        let next = transition_attachment_preview_status_v1(
            &current.status,
            AttachmentPreviewTransitionV1::Complete {
                preview_kind: artifact.preview_kind,
                content_type: artifact.content_type,
                preview_size_bytes: artifact.preview_size_bytes,
                truncated: artifact.truncated,
            },
        )
        .map_err(|_| AttachmentPreviewPersistenceErrorV1::InvalidInput)?;
        finish_job_and_run(
            &mut transaction,
            logical_owner_id,
            job_id,
            lease,
            job.run_id,
            &current.status,
            &next,
            3,
            committed_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_unavailable)
    }

    pub async fn fail_job(
        &self,
        logical_owner_id: &str,
        job_id: [u8; 16],
        lease: &PreviewJobLeaseV1,
        error: AttachmentPreviewErrorCodeV1,
        occurred_at_unix_millis: i64,
    ) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
        validate_lease_input(logical_owner_id, job_id, lease, occurred_at_unix_millis)?;
        let transition = if error == AttachmentPreviewErrorCodeV1::Unsupported {
            AttachmentPreviewTransitionV1::MarkUnsupported
        } else {
            AttachmentPreviewTransitionV1::Reject(error)
        };
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let job = lock_leased_job(
            &mut transaction,
            logical_owner_id,
            job_id,
            lease,
            occurred_at_unix_millis,
        )
        .await?;
        let current = load_run_for_update(&mut transaction, logical_owner_id, job.run_id)
            .await?
            .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
        let next = transition_attachment_preview_status_v1(&current.status, transition)
            .map_err(|_| AttachmentPreviewPersistenceErrorV1::InvalidInput)?;
        finish_job_and_run(
            &mut transaction,
            logical_owner_id,
            job_id,
            lease,
            job.run_id,
            &current.status,
            &next,
            4,
            occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_unavailable)
    }
}

struct LockedJobV1 {
    run_id: [u8; 16],
    source_receipt_sha256: [u8; 32],
    target_blob_receipt: Option<PreviewTargetBlobReceiptV1>,
}

async fn lock_leased_job(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    job_id: [u8; 16],
    lease: &PreviewJobLeaseV1,
    now_unix_millis: i64,
) -> Result<LockedJobV1, AttachmentPreviewPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT run_id,source_receipt_sha256,target_reference_id,target_receipt_sha256 FROM makosh_data.attachment_preview_jobs WHERE logical_owner_id=$1 AND job_id=$2 AND state=2 AND worker_id=$3 AND runtime_generation=$4 AND grant_epoch=$5 AND lease_fence=$6 AND lease_expires_at_unix_millis>=$7 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(job_id.as_slice())
    .bind(&lease.worker_id)
    .bind(i64::try_from(lease.runtime_generation).map_err(invalid_input)?)
    .bind(i64::try_from(lease.grant_epoch).map_err(invalid_input)?)
    .bind(i64::try_from(lease.lease_fence).map_err(invalid_input)?)
    .bind(now_unix_millis)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .ok_or(AttachmentPreviewPersistenceErrorV1::StaleFence)?;
    let reference = row
        .try_get::<Option<Vec<u8>>, _>("target_reference_id")
        .map_err(invalid_row)?
        .map(id16)
        .transpose()?;
    let receipt = row
        .try_get::<Option<Vec<u8>>, _>("target_receipt_sha256")
        .map_err(invalid_row)?
        .map(id32)
        .transpose()?;
    let target_blob_receipt = match (reference, receipt) {
        (Some(reference_id), Some(receipt_sha256)) => Some(PreviewTargetBlobReceiptV1 {
            reference_id,
            receipt_sha256,
        }),
        (None, None) => None,
        _ => return Err(AttachmentPreviewPersistenceErrorV1::InvalidRow),
    };
    Ok(LockedJobV1 {
        run_id: id16(row.try_get("run_id").map_err(invalid_row)?)?,
        source_receipt_sha256: id32(row.try_get("source_receipt_sha256").map_err(invalid_row)?)?,
        target_blob_receipt,
    })
}

#[allow(clippy::too_many_arguments)]
async fn finish_job_and_run(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    job_id: [u8; 16],
    lease: &PreviewJobLeaseV1,
    run_id: [u8; 16],
    current: &makosh_attachment_preview_core::AttachmentPreviewStatusV1,
    next: &makosh_attachment_preview_core::AttachmentPreviewStatusV1,
    job_state: i16,
    occurred_at_unix_millis: i64,
) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
    let changed = sqlx::query(
        "UPDATE makosh_data.attachment_preview_jobs SET state=$1,worker_id=NULL,runtime_generation=NULL,grant_epoch=NULL,lease_expires_at_unix_millis=NULL,updated_at_unix_millis=$2 WHERE logical_owner_id=$3 AND job_id=$4 AND state=2 AND worker_id=$5 AND runtime_generation=$6 AND grant_epoch=$7 AND lease_fence=$8",
    )
    .bind(job_state)
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(job_id.as_slice())
    .bind(&lease.worker_id)
    .bind(i64::try_from(lease.runtime_generation).map_err(invalid_input)?)
    .bind(i64::try_from(lease.grant_epoch).map_err(invalid_input)?)
    .bind(i64::try_from(lease.lease_fence).map_err(invalid_input)?)
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .rows_affected();
    if changed != 1
        || !update_run_status(
            transaction,
            logical_owner_id,
            run_id,
            current.state_revision,
            next,
            occurred_at_unix_millis,
        )
        .await?
    {
        return Err(AttachmentPreviewPersistenceErrorV1::StaleFence);
    }
    append_realtime(
        transaction,
        logical_owner_id,
        run_id,
        next,
        occurred_at_unix_millis,
    )
    .await
}

fn claimed_from_rows(
    logical_owner_id: &str,
    worker_id: &str,
    runtime_generation: u64,
    grant_epoch: u64,
    row: sqlx::postgres::PgRow,
    operation_id: [u8; 16],
) -> Result<ClaimedAttachmentPreviewJobV1, AttachmentPreviewPersistenceErrorV1> {
    let target_reference = row
        .try_get::<Option<Vec<u8>>, _>("target_reference_id")
        .map_err(invalid_row)?
        .map(id16)
        .transpose()?;
    let target_receipt = row
        .try_get::<Option<Vec<u8>>, _>("target_receipt_sha256")
        .map_err(invalid_row)?
        .map(id32)
        .transpose()?;
    let target_blob_receipt = match (target_reference, target_receipt) {
        (Some(reference_id), Some(receipt_sha256)) => Some(PreviewTargetBlobReceiptV1 {
            reference_id,
            receipt_sha256,
        }),
        (None, None) => None,
        _ => return Err(AttachmentPreviewPersistenceErrorV1::InvalidRow),
    };
    let claimed = ClaimedAttachmentPreviewJobV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        job_id: id16(row.try_get("job_id").map_err(invalid_row)?)?,
        run_id: id16(row.try_get("run_id").map_err(invalid_row)?)?,
        operation_id,
        attachment_anchor_id: id16(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?,
        delegation_request_id: id16(row.try_get("request_id").map_err(invalid_row)?)?,
        delegation_result_message_id: id16(row.try_get("result_message_id").map_err(invalid_row)?)?,
        delegation_result_envelope_sha256: id32(
            row.try_get("result_envelope_sha256").map_err(invalid_row)?,
        )?,
        candidate_message_id: id16(row.try_get("candidate_message_id").map_err(invalid_row)?)?,
        safety_message_id: id16(row.try_get("safety_message_id").map_err(invalid_row)?)?,
        source_reference_id: id16(row.try_get("source_reference_id").map_err(invalid_row)?)?,
        source_receipt_sha256: id32(row.try_get("source_receipt_sha256").map_err(invalid_row)?)?,
        source_declared_size: u64::try_from(
            row.try_get::<i64, _>("source_declared_size")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        custody_transfer_source_proof: row
            .try_get("custody_transfer_source_proof")
            .map_err(invalid_row)?,
        target_blob_receipt,
        attempt_count: u32::try_from(
            row.try_get::<i32, _>("attempt_count")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        max_attempts: u32::try_from(row.try_get::<i32, _>("max_attempts").map_err(invalid_row)?)
            .map_err(invalid_row)?,
        lease: PreviewJobLeaseV1 {
            worker_id: worker_id.to_owned(),
            runtime_generation,
            grant_epoch,
            lease_fence: u64::try_from(row.try_get::<i64, _>("lease_fence").map_err(invalid_row)?)
                .map_err(invalid_row)?,
            lease_expires_at_unix_millis: row
                .try_get("lease_expires_at_unix_millis")
                .map_err(invalid_row)?,
        },
    };
    if claimed.attempt_count == 0
        || claimed.attempt_count > claimed.max_attempts
        || claimed.max_attempts != ATTACHMENT_PREVIEW_MAX_ATTEMPTS_V1
        || !(1..=ATTACHMENT_PREVIEW_MAX_SOURCE_BYTES_V1).contains(&claimed.source_declared_size)
        || !(1..=ATTACHMENT_PREVIEW_MAX_PROOF_BYTES_V1)
            .contains(&claimed.custody_transfer_source_proof.len())
    {
        return Err(AttachmentPreviewPersistenceErrorV1::InvalidRow);
    }
    Ok(claimed)
}

fn validate_lease_input(
    logical_owner_id: &str,
    job_id: [u8; 16],
    lease: &PreviewJobLeaseV1,
    now_unix_millis: i64,
) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
    if !valid_owner(logical_owner_id)
        || !valid_id16(&job_id)
        || !valid_worker(&lease.worker_id)
        || lease.runtime_generation == 0
        || lease.grant_epoch == 0
        || lease.lease_fence == 0
        || lease.lease_expires_at_unix_millis < now_unix_millis
        || !valid_timestamp_millis(now_unix_millis)
    {
        Err(AttachmentPreviewPersistenceErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}

fn invalid_input<T>(_: T) -> AttachmentPreviewPersistenceErrorV1 {
    AttachmentPreviewPersistenceErrorV1::InvalidInput
}
