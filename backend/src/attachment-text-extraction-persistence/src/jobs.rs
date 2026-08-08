//! Fenced durable extraction jobs and terminal artifact metadata.

use makosh_attachment_text_extraction_core::{
    AttachmentTextExtractionErrorV1, AttachmentTextExtractionRequestV1,
    AttachmentTextExtractionStateV1, AttachmentTextExtractionStatusV1,
    AttachmentTextExtractionTransitionV1, transition_attachment_text_status_v1,
};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AttachmentTextExtractionPersistenceErrorV1, AttachmentTextExtractionPersistenceV1,
    ClaimedAttachmentTextExtractionJobV1, PersistedAttachmentTextArtifactV1, TextExtractionLeaseV1,
    TextExtractionTargetBlobReceiptV1,
    model::{
        ATTACHMENT_TEXT_EXTRACTION_MAX_ATTEMPTS_V1, attachment_text_extraction_job_id_v1,
        format_code, state_from_code, valid_id16, valid_owner, valid_sha256,
        valid_timestamp_millis, valid_worker,
    },
    repository::{append_realtime, update_run_status},
};

const MAX_LEASE_MILLIS_V1: u64 = 300_000;
const MAX_DERIVED_BYTES_V1: u64 = 1_048_576;

pub(crate) struct AttachmentTextDelegatedWorkV1 {
    pub request: AttachmentTextExtractionRequestV1,
    pub delegation_request_id: [u8; 16],
    pub delegation_result_message_id: [u8; 16],
    pub candidate_message_id: [u8; 16],
    pub safety_message_id: [u8; 16],
    pub source_reference_id: [u8; 16],
    pub source_receipt_sha256: [u8; 32],
    pub source_declared_size: u64,
    pub custody_transfer_source_proof: Vec<u8>,
}

impl AttachmentTextExtractionPersistenceV1 {
    pub async fn claim_next_job(
        &self,
        logical_owner_id: &str,
        worker_id: &str,
        runtime_generation: u64,
        grant_epoch: u64,
        now_unix_millis: i64,
        lease_millis: u64,
    ) -> Result<
        Option<ClaimedAttachmentTextExtractionJobV1>,
        AttachmentTextExtractionPersistenceErrorV1,
    > {
        if !valid_owner(logical_owner_id)
            || !valid_worker(worker_id)
            || runtime_generation == 0
            || grant_epoch == 0
            || !valid_timestamp_millis(now_unix_millis)
            || !(1..=MAX_LEASE_MILLIS_V1).contains(&lease_millis)
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let lease_expires_at_unix_millis = now_unix_millis
            .checked_add(i64::try_from(lease_millis).map_err(invalid_input)?)
            .ok_or(AttachmentTextExtractionPersistenceErrorV1::InvalidInput)?;
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let row = sqlx::query(
            "SELECT job_id FROM makosh_data.attachment_text_extraction_jobs WHERE logical_owner_id=$1 AND state=1 AND attempt_count<max_attempts ORDER BY created_at_unix_millis,job_id FOR UPDATE SKIP LOCKED LIMIT 1",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        let Some(row) = row else {
            transaction.commit().await.map_err(storage_unavailable)?;
            return Ok(None);
        };
        let job_id = id16(row.try_get("job_id").map_err(invalid_row)?)?;
        let changed = sqlx::query(
            "UPDATE makosh_data.attachment_text_extraction_jobs SET state=2,attempt_count=attempt_count+1,worker_id=$3,runtime_generation=$4,grant_epoch=$5,lease_fence=lease_fence+1,lease_expires_at_unix_millis=$6,updated_at_unix_millis=$7 WHERE logical_owner_id=$1 AND job_id=$2 AND state=1",
        )
        .bind(logical_owner_id)
        .bind(job_id.as_slice())
        .bind(worker_id)
        .bind(i64::try_from(runtime_generation).map_err(invalid_input)?)
        .bind(i64::try_from(grant_epoch).map_err(invalid_input)?)
        .bind(lease_expires_at_unix_millis)
        .bind(now_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?
        .rows_affected();
        if changed != 1 {
            return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
        }
        let claimed = load_claimed_job(&mut transaction, logical_owner_id, job_id).await?;
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(Some(claimed))
    }

    pub async fn record_target_blob_receipt(
        &self,
        claimed: &ClaimedAttachmentTextExtractionJobV1,
        receipt: TextExtractionTargetBlobReceiptV1,
        recorded_at_unix_millis: i64,
    ) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
        if !valid_claim(claimed)
            || !valid_id16(&receipt.reference_id)
            || !valid_sha256(&receipt.receipt_sha256)
            || !valid_timestamp_millis(recorded_at_unix_millis)
            || recorded_at_unix_millis > claimed.lease.lease_expires_at_unix_millis
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let changed = sqlx::query(
            "UPDATE makosh_data.attachment_text_extraction_jobs SET target_reference_id=$8,target_receipt_sha256=$9,updated_at_unix_millis=$7 WHERE logical_owner_id=$1 AND job_id=$2 AND state=2 AND worker_id=$3 AND runtime_generation=$4 AND grant_epoch=$5 AND lease_fence=$6 AND lease_expires_at_unix_millis>$7 AND ((target_reference_id IS NULL AND target_receipt_sha256 IS NULL) OR (target_reference_id=$8 AND target_receipt_sha256=$9))",
        )
        .bind(&claimed.logical_owner_id)
        .bind(claimed.job_id.as_slice())
        .bind(&claimed.lease.worker_id)
        .bind(i64::try_from(claimed.lease.runtime_generation).map_err(invalid_input)?)
        .bind(i64::try_from(claimed.lease.grant_epoch).map_err(invalid_input)?)
        .bind(i64::try_from(claimed.lease.lease_fence).map_err(invalid_input)?)
        .bind(recorded_at_unix_millis)
        .bind(receipt.reference_id.as_slice())
        .bind(receipt.receipt_sha256.as_slice())
        .execute(&self.pool)
        .await
        .map_err(storage_unavailable)?
        .rows_affected();
        if changed == 1 {
            Ok(())
        } else {
            Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)
        }
    }

    pub async fn complete_job(
        &self,
        claimed: &ClaimedAttachmentTextExtractionJobV1,
        artifact: PersistedAttachmentTextArtifactV1,
        completed_at_unix_millis: i64,
    ) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
        if !valid_claim(claimed)
            || !valid_artifact_for_claim(claimed, &artifact)
            || !valid_timestamp_millis(completed_at_unix_millis)
            || completed_at_unix_millis > claimed.lease.lease_expires_at_unix_millis
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        verify_claim(&mut transaction, claimed, completed_at_unix_millis).await?;
        verify_target_receipt(&mut transaction, claimed).await?;
        let (state, revision) = load_extracting_run(
            &mut transaction,
            &claimed.logical_owner_id,
            claimed.request.run_id,
        )
        .await?;
        let next = transition_attachment_text_status_v1(
            &in_progress_status(state, revision)?,
            AttachmentTextExtractionTransitionV1::Complete {
                format: artifact.format,
                extracted_size_bytes: artifact.extracted_size_bytes,
                extraction_truncated: artifact.extraction_truncated,
            },
        )
        .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
        sqlx::query(
            "INSERT INTO makosh_data.attachment_text_extraction_artifacts (logical_owner_id,run_id,derived_reference_id,derived_receipt_sha256,source_receipt_sha256,parser_identity_sha256,format_code,extracted_size_bytes,extraction_truncated,committed_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(&claimed.logical_owner_id)
        .bind(artifact.run_id.as_slice())
        .bind(artifact.derived_reference_id.as_slice())
        .bind(artifact.derived_receipt_sha256.as_slice())
        .bind(artifact.source_receipt_sha256.as_slice())
        .bind(artifact.parser_identity_sha256.as_slice())
        .bind(format_code(artifact.format))
        .bind(i64::try_from(artifact.extracted_size_bytes).map_err(invalid_input)?)
        .bind(artifact.extraction_truncated)
        .bind(completed_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        persist_terminal_status(
            &mut transaction,
            &claimed.logical_owner_id,
            claimed.request.run_id,
            revision,
            &next,
            completed_at_unix_millis,
        )
        .await?;
        finish_claim(&mut transaction, claimed, 3, completed_at_unix_millis).await?;
        transaction.commit().await.map_err(storage_unavailable)
    }

    pub async fn reject_job(
        &self,
        claimed: &ClaimedAttachmentTextExtractionJobV1,
        error: AttachmentTextExtractionErrorV1,
        completed_at_unix_millis: i64,
    ) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
        if !valid_claim(claimed)
            || !valid_timestamp_millis(completed_at_unix_millis)
            || completed_at_unix_millis > claimed.lease.lease_expires_at_unix_millis
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        verify_claim(&mut transaction, claimed, completed_at_unix_millis).await?;
        let (state, revision) = load_extracting_run(
            &mut transaction,
            &claimed.logical_owner_id,
            claimed.request.run_id,
        )
        .await?;
        let transition = if error == AttachmentTextExtractionErrorV1::Unsupported {
            AttachmentTextExtractionTransitionV1::MarkUnsupported
        } else {
            AttachmentTextExtractionTransitionV1::Reject(error)
        };
        let next =
            transition_attachment_text_status_v1(&in_progress_status(state, revision)?, transition)
                .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
        persist_terminal_status(
            &mut transaction,
            &claimed.logical_owner_id,
            claimed.request.run_id,
            revision,
            &next,
            completed_at_unix_millis,
        )
        .await?;
        finish_claim(&mut transaction, claimed, 3, completed_at_unix_millis).await?;
        transaction.commit().await.map_err(storage_unavailable)
    }

    pub async fn recover_expired_jobs(
        &self,
        logical_owner_id: &str,
        now_unix_millis: i64,
    ) -> Result<u32, AttachmentTextExtractionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || !valid_timestamp_millis(now_unix_millis) {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let retry_count = sqlx::query(
            "UPDATE makosh_data.attachment_text_extraction_jobs SET state=1,worker_id=NULL,runtime_generation=NULL,grant_epoch=NULL,lease_expires_at_unix_millis=NULL,updated_at_unix_millis=$2 WHERE logical_owner_id=$1 AND state=2 AND lease_expires_at_unix_millis<=$2 AND attempt_count<max_attempts",
        )
        .bind(logical_owner_id)
        .bind(now_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?
        .rows_affected();
        let exhausted = sqlx::query(
            "SELECT job_id,run_id FROM makosh_data.attachment_text_extraction_jobs WHERE logical_owner_id=$1 AND state=2 AND lease_expires_at_unix_millis<=$2 AND attempt_count>=max_attempts ORDER BY job_id FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(now_unix_millis)
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        for row in &exhausted {
            let job_id = id16(row.try_get("job_id").map_err(invalid_row)?)?;
            let run_id = id16(row.try_get("run_id").map_err(invalid_row)?)?;
            let changed = sqlx::query(
                "UPDATE makosh_data.attachment_text_extraction_jobs SET state=4,worker_id=NULL,runtime_generation=NULL,grant_epoch=NULL,lease_expires_at_unix_millis=NULL,updated_at_unix_millis=$3 WHERE logical_owner_id=$1 AND job_id=$2 AND state=2",
            )
            .bind(logical_owner_id)
            .bind(job_id.as_slice())
            .bind(now_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(storage_unavailable)?
            .rows_affected();
            if changed != 1 {
                return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
            }
            let (state, revision) =
                load_extracting_run(&mut transaction, logical_owner_id, run_id).await?;
            let next = transition_attachment_text_status_v1(
                &in_progress_status(state, revision)?,
                AttachmentTextExtractionTransitionV1::Reject(
                    AttachmentTextExtractionErrorV1::Unavailable,
                ),
            )
            .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
            persist_terminal_status(
                &mut transaction,
                logical_owner_id,
                run_id,
                revision,
                &next,
                now_unix_millis,
            )
            .await?;
        }
        let exhausted_count = u64::try_from(exhausted.len()).map_err(invalid_row)?;
        let total = retry_count
            .checked_add(exhausted_count)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(AttachmentTextExtractionPersistenceErrorV1::InvalidRow)?;
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(total)
    }
}

pub(crate) async fn enqueue_attachment_text_work(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    work: &AttachmentTextDelegatedWorkV1,
    created_at_unix_millis: i64,
) -> Result<[u8; 16], AttachmentTextExtractionPersistenceErrorV1> {
    let job_id = attachment_text_extraction_job_id_v1(
        &work.request,
        work.delegation_request_id,
        work.delegation_result_message_id,
    );
    let proof_sha256: [u8; 32] = Sha256::digest(&work.custody_transfer_source_proof).into();
    sqlx::query(
        "INSERT INTO makosh_data.attachment_text_extraction_jobs (logical_owner_id,job_id,run_id,request_id,result_message_id,attachment_anchor_id,candidate_message_id,safety_message_id,source_reference_id,target_reference_id,target_receipt_sha256,source_receipt_sha256,source_declared_size,custody_transfer_source_proof,custody_proof_sha256,state,attempt_count,max_attempts,worker_id,runtime_generation,grant_epoch,lease_fence,lease_expires_at_unix_millis,created_at_unix_millis,updated_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,NULL,NULL,$10,$11,$12,$13,1,0,$14,NULL,NULL,NULL,0,NULL,$15,$15) ON CONFLICT (logical_owner_id,run_id) DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(job_id.as_slice())
    .bind(work.request.run_id.as_slice())
    .bind(work.delegation_request_id.as_slice())
    .bind(work.delegation_result_message_id.as_slice())
    .bind(work.request.attachment_anchor_id.as_slice())
    .bind(work.candidate_message_id.as_slice())
    .bind(work.safety_message_id.as_slice())
    .bind(work.source_reference_id.as_slice())
    .bind(work.source_receipt_sha256.as_slice())
    .bind(i64::try_from(work.source_declared_size).map_err(invalid_input)?)
    .bind(&work.custody_transfer_source_proof)
    .bind(proof_sha256.as_slice())
    .bind(i32::try_from(ATTACHMENT_TEXT_EXTRACTION_MAX_ATTEMPTS_V1).expect("bounded attempts"))
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    let row = sqlx::query(
        "SELECT job_id,request_id,result_message_id,attachment_anchor_id,candidate_message_id,safety_message_id,source_reference_id,source_receipt_sha256,source_declared_size,custody_transfer_source_proof,custody_proof_sha256 FROM makosh_data.attachment_text_extraction_jobs WHERE logical_owner_id=$1 AND run_id=$2",
    )
    .bind(logical_owner_id)
    .bind(work.request.run_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    let exact = id16(row.try_get("job_id").map_err(invalid_row)?)? == job_id
        && id16(row.try_get("request_id").map_err(invalid_row)?)? == work.delegation_request_id
        && id16(row.try_get("result_message_id").map_err(invalid_row)?)?
            == work.delegation_result_message_id
        && id16(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?
            == work.request.attachment_anchor_id
        && id16(row.try_get("candidate_message_id").map_err(invalid_row)?)?
            == work.candidate_message_id
        && id16(row.try_get("safety_message_id").map_err(invalid_row)?)? == work.safety_message_id
        && id16(row.try_get("source_reference_id").map_err(invalid_row)?)?
            == work.source_reference_id
        && id32(row.try_get("source_receipt_sha256").map_err(invalid_row)?)?
            == work.source_receipt_sha256
        && u64::try_from(
            row.try_get::<i64, _>("source_declared_size")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?
            == work.source_declared_size
        && row
            .try_get::<Vec<u8>, _>("custody_transfer_source_proof")
            .map_err(invalid_row)?
            == work.custody_transfer_source_proof
        && id32(row.try_get("custody_proof_sha256").map_err(invalid_row)?)? == proof_sha256;
    if !exact {
        return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
    }
    Ok(job_id)
}

async fn load_claimed_job(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    job_id: [u8; 16],
) -> Result<ClaimedAttachmentTextExtractionJobV1, AttachmentTextExtractionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT j.run_id,r.operation_id,j.attachment_anchor_id,j.request_id,j.result_message_id,i.envelope_sha256,j.candidate_message_id,j.safety_message_id,j.source_reference_id,j.target_reference_id,j.target_receipt_sha256,j.source_receipt_sha256,j.source_declared_size,j.custody_transfer_source_proof,j.attempt_count,j.max_attempts,j.worker_id,j.runtime_generation,j.grant_epoch,j.lease_fence,j.lease_expires_at_unix_millis FROM makosh_data.attachment_text_extraction_jobs j JOIN makosh_data.attachment_text_extraction_runs r ON r.logical_owner_id=j.logical_owner_id AND r.run_id=j.run_id JOIN makosh_data.attachment_text_extraction_custody_result_inbox i ON i.logical_owner_id=j.logical_owner_id AND i.message_id=j.result_message_id WHERE j.logical_owner_id=$1 AND j.job_id=$2 AND j.state=2",
    )
    .bind(logical_owner_id)
    .bind(job_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .ok_or(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
    let claimed = ClaimedAttachmentTextExtractionJobV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        job_id,
        request: AttachmentTextExtractionRequestV1 {
            run_id: id16(row.try_get("run_id").map_err(invalid_row)?)?,
            operation_id: id16(row.try_get("operation_id").map_err(invalid_row)?)?,
            attachment_anchor_id: id16(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?,
        },
        delegation_request_id: id16(row.try_get("request_id").map_err(invalid_row)?)?,
        delegation_result_message_id: id16(row.try_get("result_message_id").map_err(invalid_row)?)?,
        delegation_result_envelope_sha256: id32(
            row.try_get("envelope_sha256").map_err(invalid_row)?,
        )?,
        candidate_message_id: id16(row.try_get("candidate_message_id").map_err(invalid_row)?)?,
        safety_message_id: id16(row.try_get("safety_message_id").map_err(invalid_row)?)?,
        source_reference_id: id16(row.try_get("source_reference_id").map_err(invalid_row)?)?,
        target_blob_receipt: target_receipt_from_row(&row)?,
        source_receipt_sha256: id32(row.try_get("source_receipt_sha256").map_err(invalid_row)?)?,
        source_declared_size: u64::try_from(
            row.try_get::<i64, _>("source_declared_size")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        custody_transfer_source_proof: row
            .try_get("custody_transfer_source_proof")
            .map_err(invalid_row)?,
        attempt_count: u32::try_from(
            row.try_get::<i32, _>("attempt_count")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        max_attempts: u32::try_from(row.try_get::<i32, _>("max_attempts").map_err(invalid_row)?)
            .map_err(invalid_row)?,
        lease: TextExtractionLeaseV1 {
            worker_id: row.try_get("worker_id").map_err(invalid_row)?,
            runtime_generation: u64::try_from(
                row.try_get::<i64, _>("runtime_generation")
                    .map_err(invalid_row)?,
            )
            .map_err(invalid_row)?,
            grant_epoch: u64::try_from(row.try_get::<i64, _>("grant_epoch").map_err(invalid_row)?)
                .map_err(invalid_row)?,
            lease_fence: u64::try_from(row.try_get::<i64, _>("lease_fence").map_err(invalid_row)?)
                .map_err(invalid_row)?,
            lease_expires_at_unix_millis: row
                .try_get("lease_expires_at_unix_millis")
                .map_err(invalid_row)?,
        },
    };
    if !valid_claim(&claimed) {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow);
    }
    Ok(claimed)
}

async fn verify_claim(
    transaction: &mut Transaction<'_, Postgres>,
    claimed: &ClaimedAttachmentTextExtractionJobV1,
    completed_at_unix_millis: i64,
) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT worker_id,runtime_generation,grant_epoch,lease_fence,lease_expires_at_unix_millis FROM makosh_data.attachment_text_extraction_jobs WHERE logical_owner_id=$1 AND job_id=$2 AND state=2 FOR UPDATE",
    )
    .bind(&claimed.logical_owner_id)
    .bind(claimed.job_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .ok_or(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
    let exact = row.try_get::<String, _>("worker_id").map_err(invalid_row)?
        == claimed.lease.worker_id
        && u64::try_from(
            row.try_get::<i64, _>("runtime_generation")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?
            == claimed.lease.runtime_generation
        && u64::try_from(row.try_get::<i64, _>("grant_epoch").map_err(invalid_row)?)
            .map_err(invalid_row)?
            == claimed.lease.grant_epoch
        && u64::try_from(row.try_get::<i64, _>("lease_fence").map_err(invalid_row)?)
            .map_err(invalid_row)?
            == claimed.lease.lease_fence
        && row
            .try_get::<i64, _>("lease_expires_at_unix_millis")
            .map_err(invalid_row)?
            == claimed.lease.lease_expires_at_unix_millis
        && completed_at_unix_millis <= claimed.lease.lease_expires_at_unix_millis;
    if exact {
        Ok(())
    } else {
        Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)
    }
}

async fn verify_target_receipt(
    transaction: &mut Transaction<'_, Postgres>,
    claimed: &ClaimedAttachmentTextExtractionJobV1,
) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT target_reference_id,target_receipt_sha256 FROM makosh_data.attachment_text_extraction_jobs WHERE logical_owner_id=$1 AND job_id=$2 AND state=2 AND lease_fence=$3 FOR UPDATE",
    )
    .bind(&claimed.logical_owner_id)
    .bind(claimed.job_id.as_slice())
    .bind(i64::try_from(claimed.lease.lease_fence).map_err(invalid_input)?)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .ok_or(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
    let reference = row
        .try_get::<Option<Vec<u8>>, _>("target_reference_id")
        .map_err(invalid_row)?
        .ok_or(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
    let receipt = row
        .try_get::<Option<Vec<u8>>, _>("target_receipt_sha256")
        .map_err(invalid_row)?
        .ok_or(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
    if !valid_id16(&id16(reference)?) || !valid_sha256(&id32(receipt)?) {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow);
    }
    Ok(())
}

async fn finish_claim(
    transaction: &mut Transaction<'_, Postgres>,
    claimed: &ClaimedAttachmentTextExtractionJobV1,
    terminal_state: i16,
    completed_at_unix_millis: i64,
) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
    let changed = sqlx::query(
        "UPDATE makosh_data.attachment_text_extraction_jobs SET state=$3,worker_id=NULL,runtime_generation=NULL,grant_epoch=NULL,lease_expires_at_unix_millis=NULL,updated_at_unix_millis=$4 WHERE logical_owner_id=$1 AND job_id=$2 AND state=2 AND lease_fence=$5",
    )
    .bind(&claimed.logical_owner_id)
    .bind(claimed.job_id.as_slice())
    .bind(terminal_state)
    .bind(completed_at_unix_millis)
    .bind(i64::try_from(claimed.lease.lease_fence).map_err(invalid_input)?)
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .rows_affected();
    if changed == 1 {
        Ok(())
    } else {
        Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)
    }
}

async fn load_extracting_run(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
) -> Result<(AttachmentTextExtractionStateV1, u64), AttachmentTextExtractionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT state,state_revision FROM makosh_data.attachment_text_extraction_runs WHERE logical_owner_id=$1 AND run_id=$2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .ok_or(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
    let state = state_from_code(row.try_get("state").map_err(invalid_row)?)?;
    if state != AttachmentTextExtractionStateV1::Extracting {
        return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
    }
    let revision = u64::try_from(
        row.try_get::<i64, _>("state_revision")
            .map_err(invalid_row)?,
    )
    .map_err(invalid_row)?;
    Ok((state, revision))
}

fn in_progress_status(
    state: AttachmentTextExtractionStateV1,
    revision: u64,
) -> Result<AttachmentTextExtractionStatusV1, AttachmentTextExtractionPersistenceErrorV1> {
    if state != AttachmentTextExtractionStateV1::Extracting || revision == 0 {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow);
    }
    Ok(AttachmentTextExtractionStatusV1 {
        state,
        state_revision: revision,
        format: None,
        extracted_size_bytes: 0,
        extraction_truncated: false,
        error: None,
    })
}

async fn persist_terminal_status(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    revision: u64,
    next: &AttachmentTextExtractionStatusV1,
    occurred_at_unix_millis: i64,
) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
    if !update_run_status(
        transaction,
        logical_owner_id,
        run_id,
        revision,
        next,
        occurred_at_unix_millis,
    )
    .await?
    {
        return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
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

fn target_receipt_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<Option<TextExtractionTargetBlobReceiptV1>, AttachmentTextExtractionPersistenceErrorV1> {
    let reference = row
        .try_get::<Option<Vec<u8>>, _>("target_reference_id")
        .map_err(invalid_row)?;
    let receipt = row
        .try_get::<Option<Vec<u8>>, _>("target_receipt_sha256")
        .map_err(invalid_row)?;
    match (reference, receipt) {
        (None, None) => Ok(None),
        (Some(reference), Some(receipt)) => Ok(Some(TextExtractionTargetBlobReceiptV1 {
            reference_id: id16(reference)?,
            receipt_sha256: id32(receipt)?,
        })),
        _ => Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow),
    }
}

fn valid_claim(value: &ClaimedAttachmentTextExtractionJobV1) -> bool {
    valid_owner(&value.logical_owner_id)
        && valid_id16(&value.job_id)
        && valid_id16(&value.request.run_id)
        && valid_id16(&value.request.operation_id)
        && valid_id16(&value.request.attachment_anchor_id)
        && valid_id16(&value.delegation_request_id)
        && valid_id16(&value.delegation_result_message_id)
        && valid_sha256(&value.delegation_result_envelope_sha256)
        && valid_id16(&value.candidate_message_id)
        && valid_id16(&value.safety_message_id)
        && valid_id16(&value.source_reference_id)
        && valid_sha256(&value.source_receipt_sha256)
        && value.source_declared_size > 0
        && (1..=2_048).contains(&value.custody_transfer_source_proof.len())
        && value.attempt_count > 0
        && value.attempt_count <= value.max_attempts
        && value.max_attempts <= ATTACHMENT_TEXT_EXTRACTION_MAX_ATTEMPTS_V1
        && valid_worker(&value.lease.worker_id)
        && value.lease.runtime_generation > 0
        && value.lease.grant_epoch > 0
        && value.lease.lease_fence > 0
        && valid_timestamp_millis(value.lease.lease_expires_at_unix_millis)
}

fn valid_artifact_for_claim(
    claimed: &ClaimedAttachmentTextExtractionJobV1,
    artifact: &PersistedAttachmentTextArtifactV1,
) -> bool {
    artifact.run_id == claimed.request.run_id
        && valid_id16(&artifact.derived_reference_id)
        && artifact.derived_reference_id != claimed.source_reference_id
        && valid_sha256(&artifact.derived_receipt_sha256)
        && artifact.source_receipt_sha256 == claimed.source_receipt_sha256
        && valid_sha256(&artifact.parser_identity_sha256)
        && (1..=MAX_DERIVED_BYTES_V1).contains(&artifact.extracted_size_bytes)
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], AttachmentTextExtractionPersistenceErrorV1> {
    value.try_into().map_err(invalid_row)
}

fn id32(value: Vec<u8>) -> Result<[u8; 32], AttachmentTextExtractionPersistenceErrorV1> {
    value.try_into().map_err(invalid_row)
}

fn storage_unavailable<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::StorageUnavailable
}

fn invalid_input<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::InvalidInput
}

fn invalid_row<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::InvalidRow
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

    fn claim() -> ClaimedAttachmentTextExtractionJobV1 {
        ClaimedAttachmentTextExtractionJobV1 {
            logical_owner_id: "owner".to_owned(),
            job_id: [1; 16],
            request: AttachmentTextExtractionRequestV1 {
                run_id: [2; 16],
                operation_id: [3; 16],
                attachment_anchor_id: [4; 16],
            },
            delegation_request_id: [5; 16],
            delegation_result_message_id: [6; 16],
            delegation_result_envelope_sha256: [7; 32],
            candidate_message_id: [8; 16],
            safety_message_id: [9; 16],
            source_reference_id: [10; 16],
            target_blob_receipt: None,
            source_receipt_sha256: [11; 32],
            source_declared_size: 42,
            custody_transfer_source_proof: vec![12],
            attempt_count: 1,
            max_attempts: ATTACHMENT_TEXT_EXTRACTION_MAX_ATTEMPTS_V1,
            lease: TextExtractionLeaseV1 {
                worker_id: "worker".to_owned(),
                runtime_generation: 1,
                grant_epoch: 1,
                lease_fence: 1,
                lease_expires_at_unix_millis: 1,
            },
        }
    }
}
