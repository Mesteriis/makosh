//! Durable Text Extraction custody-delegation inbox, work claims and result outbox.

use makosh_attachment_text_extraction_ingress::{
    ATTACHMENT_SECURITY_TEXT_EXTRACTION_DELEGATION_CAPABILITY_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_CUSTODY_DELEGATED_CONTRACT_NAME_V1,
    ATTACHMENT_TEXT_EXTRACTION_CUSTODY_DELEGATION_REJECTED_CONTRACT_NAME_V1,
    ATTACHMENT_TEXT_EXTRACTION_CUSTODY_DELEGATION_REQUESTED_CONTRACT_NAME_V1,
    ATTACHMENT_TEXT_EXTRACTION_INGRESS_CONTRACT_MAJOR_V1,
    ATTACHMENT_TEXT_EXTRACTION_INGRESS_CONTRACT_REVISION_V1,
    ATTACHMENT_TEXT_EXTRACTION_INGRESS_OWNER_V1, ATTACHMENT_TEXT_EXTRACTION_INGRESS_SCHEMA_SHA256,
    ATTACHMENT_TEXT_EXTRACTION_MAX_PROOF_BYTES_V1, ATTACHMENT_TEXT_EXTRACTION_MAX_SOURCE_BYTES_V1,
    attachment_text_custody_delegated_message_id_v1,
    attachment_text_custody_delegation_rejected_message_id_v1,
    wire::{
        AttachmentTextCustodyDelegatedV1, AttachmentTextCustodyDelegationRejectCodeV1,
        AttachmentTextCustodyDelegationRejectedV1, RequestAttachmentTextCustodyDelegationV1,
    },
};
use makosh_communications_attachment_contract::{
    COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVATION_SCHEMA_SHA256,
    admission::{
        COMMUNICATION_ATTACHMENT_CONTRACT_MAJOR, COMMUNICATION_ATTACHMENT_CONTRACT_OWNER,
        COMMUNICATION_ATTACHMENT_CONTRACT_REVISION,
        COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVED_CONTRACT_NAME,
    },
    safety_verdict_v1::{
        AttachmentSafetyExpectedStateV1, AttachmentSafetyVerdictObservationV1,
        AttachmentSafetyVerdictV1,
    },
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ContractRefV1, DurableEnvelopeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use prost::Message;
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AttachmentSecurityPersistenceErrorV1, AttachmentSecurityPersistenceV1, id16, id32, valid_id16,
    valid_sha256, valid_timestamp,
};

const MAX_EXACT_ENVELOPE_BYTES: usize = 8_192;
const MAX_ATTEMPTS: u32 = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityTextDelegationWorkV1 {
    Delegate {
        request: RequestAttachmentTextCustodyDelegationV1,
        current_reference_id: [u8; 16],
        current_receipt_sha256: [u8; 32],
        declared_size: u64,
        predecessor_custody_source_proof: Vec<u8>,
    },
    Reject {
        request: RequestAttachmentTextCustodyDelegationV1,
        code: AttachmentTextCustodyDelegationRejectCodeV1,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimedAttachmentSecurityTextDelegationV1 {
    pub command_message_id: [u8; 16],
    pub work: AttachmentSecurityTextDelegationWorkV1,
    pub worker_id: String,
    pub attempt_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistAttachmentSecurityTextDelegationOutcomeV1 {
    Duplicate,
    Ready,
    Rejected(AttachmentTextCustodyDelegationRejectCodeV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryAttachmentSecurityTextDelegationOutcomeV1 {
    Scheduled,
    Exhausted,
}

struct VerifiedDelegationSourceV1 {
    current_reference_id: [u8; 16],
    current_receipt_sha256: [u8; 32],
    declared_size: u64,
    predecessor_custody_source_proof: Vec<u8>,
}

impl AttachmentSecurityPersistenceV1 {
    pub async fn persist_text_delegation_request(
        &self,
        exact_envelope_bytes: &[u8],
        consumed_at_unix_seconds: i64,
    ) -> Result<
        PersistAttachmentSecurityTextDelegationOutcomeV1,
        AttachmentSecurityPersistenceErrorV1,
    > {
        if exact_envelope_bytes.is_empty()
            || exact_envelope_bytes.len() > MAX_EXACT_ENVELOPE_BYTES
            || !valid_timestamp(consumed_at_unix_seconds)
        {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let record = OutboxRecordV1::accept(exact_envelope_bytes.to_vec())
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?;
        let (envelope, request) = decode_request(record.exact_bytes())?;
        let message_id = id16(&envelope.message_id)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        let inserted = insert_request_inbox(
            &mut transaction,
            &record,
            &request,
            consumed_at_unix_seconds,
        )
        .await?;
        if !inserted {
            verify_request_replay(&mut transaction, &record).await?;
            transaction
                .commit()
                .await
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
            return Ok(PersistAttachmentSecurityTextDelegationOutcomeV1::Duplicate);
        }
        let source = verify_delegation_source(&mut transaction, &request).await?;
        let rejection_code = source.as_ref().map(|_| None).unwrap_or(Some(
            AttachmentTextCustodyDelegationRejectCodeV1::AttachmentTextExtractionCustodyDelegationRejectCodeV1NotSafe,
        ));
        insert_delegation_job(
            &mut transaction,
            message_id,
            &request,
            source.as_ref(),
            rejection_code,
            consumed_at_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        Ok(rejection_code.map_or(
            PersistAttachmentSecurityTextDelegationOutcomeV1::Ready,
            PersistAttachmentSecurityTextDelegationOutcomeV1::Rejected,
        ))
    }

    pub async fn claim_next_text_delegation(
        &self,
        worker_id: &str,
        claimed_at_unix_seconds: i64,
        lease_expires_at_unix_seconds: i64,
    ) -> Result<
        Option<ClaimedAttachmentSecurityTextDelegationV1>,
        AttachmentSecurityPersistenceErrorV1,
    > {
        if !valid_worker_id(worker_id)
            || !valid_timestamp(claimed_at_unix_seconds)
            || !valid_timestamp(lease_expires_at_unix_seconds)
            || lease_expires_at_unix_seconds <= claimed_at_unix_seconds
        {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "WITH next_job AS (
               SELECT request_message_id
               FROM makosh_data.attachment_security_text_extraction_delegation_jobs
               WHERE state = 1 AND next_attempt_at_unix_seconds <= $2
                 AND attempt_count < 8
                 AND (lease_expires_at_unix_seconds IS NULL OR lease_expires_at_unix_seconds <= $2)
               ORDER BY next_attempt_at_unix_seconds, request_message_id
               LIMIT 1 FOR UPDATE SKIP LOCKED
             )
             UPDATE makosh_data.attachment_security_text_extraction_delegation_jobs AS job
             SET claimed_by = $1, lease_expires_at_unix_seconds = $3,
                 attempt_count = job.attempt_count + 1
             FROM next_job,
                  makosh_data.attachment_security_text_extraction_delegation_inbox AS inbox
             WHERE job.request_message_id = next_job.request_message_id
               AND inbox.message_id = job.request_message_id
             RETURNING job.request_message_id, job.current_reference_id,
                       job.current_receipt_sha256, job.declared_size,
                       job.predecessor_custody_source_proof, job.rejection_code,
                       job.attempt_count, inbox.request_id, inbox.extraction_run_id,
                       inbox.attachment_anchor_id, inbox.candidate_message_id,
                       inbox.candidate_envelope_sha256, inbox.safety_message_id,
                       inbox.safety_evidence_id, inbox.logical_owner_id",
        )
        .bind(worker_id)
        .bind(claimed_at_unix_seconds)
        .bind(lease_expires_at_unix_seconds)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        row.map(|row| claimed_from_row(row, worker_id)).transpose()
    }

    pub async fn retry_text_delegation(
        &self,
        claimed: &ClaimedAttachmentSecurityTextDelegationV1,
        recorded_at_unix_seconds: i64,
        next_attempt_at_unix_seconds: i64,
    ) -> Result<RetryAttachmentSecurityTextDelegationOutcomeV1, AttachmentSecurityPersistenceErrorV1>
    {
        if !valid_claim(claimed)
            || !valid_timestamp(recorded_at_unix_seconds)
            || !valid_timestamp(next_attempt_at_unix_seconds)
        {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        if claimed.attempt_count >= MAX_ATTEMPTS {
            return Ok(RetryAttachmentSecurityTextDelegationOutcomeV1::Exhausted);
        }
        if next_attempt_at_unix_seconds <= recorded_at_unix_seconds {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.attachment_security_text_extraction_delegation_jobs
             SET claimed_by = NULL, lease_expires_at_unix_seconds = NULL,
                 next_attempt_at_unix_seconds = $5
             WHERE request_message_id = $1 AND state = 1 AND claimed_by = $2
               AND attempt_count = $3 AND lease_expires_at_unix_seconds > $4",
        )
        .bind(claimed.command_message_id.as_slice())
        .bind(&claimed.worker_id)
        .bind(
            i32::try_from(claimed.attempt_count)
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?,
        )
        .bind(recorded_at_unix_seconds)
        .bind(next_attempt_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        if updated.rows_affected() != 1 {
            return Err(AttachmentSecurityPersistenceErrorV1::ClaimLost);
        }
        Ok(RetryAttachmentSecurityTextDelegationOutcomeV1::Scheduled)
    }

    pub async fn complete_text_delegation_with_outbox(
        &self,
        claimed: &ClaimedAttachmentSecurityTextDelegationV1,
        result: &OutboxRecordV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), AttachmentSecurityPersistenceErrorV1> {
        if !valid_claim(claimed)
            || !valid_timestamp(completed_at_unix_seconds)
            || result.exact_bytes().len() > MAX_EXACT_ENVELOPE_BYTES
            || !valid_result_for_claim(claimed, result)?
        {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        insert_result_outbox(&mut transaction, result, completed_at_unix_seconds).await?;
        let updated = sqlx::query(
            "UPDATE makosh_data.attachment_security_text_extraction_delegation_jobs
             SET state = 2, result_message_id = $5, completed_at_unix_seconds = $4,
                 claimed_by = NULL, lease_expires_at_unix_seconds = NULL
             WHERE request_message_id = $1 AND state = 1 AND claimed_by = $2
               AND attempt_count = $3 AND lease_expires_at_unix_seconds > $4",
        )
        .bind(claimed.command_message_id.as_slice())
        .bind(&claimed.worker_id)
        .bind(
            i32::try_from(claimed.attempt_count)
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?,
        )
        .bind(completed_at_unix_seconds)
        .bind(result.message_id().as_slice())
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

    pub async fn pending_text_delegation_outbox(
        &self,
        limit: u32,
    ) -> Result<Vec<OutboxRecordV1>, AttachmentSecurityPersistenceErrorV1> {
        if !(1..=256).contains(&limit) {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT exact_envelope_bytes
             FROM makosh_data.attachment_security_text_extraction_delegation_outbox
             WHERE published_at_unix_seconds IS NULL
             ORDER BY created_at_unix_seconds, message_id LIMIT $1",
        )
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?
        .into_iter()
        .map(|row| {
            let bytes = row
                .try_get::<Vec<u8>, _>("exact_envelope_bytes")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
            OutboxRecordV1::accept(bytes)
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)
        })
        .collect()
    }

    pub async fn mark_text_delegation_outbox_published(
        &self,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, AttachmentSecurityPersistenceErrorV1> {
        if !valid_id16(&message_id) || !valid_timestamp(published_at_unix_seconds) {
            return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.attachment_security_text_extraction_delegation_outbox
             SET published_at_unix_seconds = $2
             WHERE message_id = $1 AND published_at_unix_seconds IS NULL",
        )
        .bind(message_id.as_slice())
        .bind(published_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
        Ok(updated.rows_affected() == 1)
    }
}

async fn insert_request_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    record: &OutboxRecordV1,
    request: &RequestAttachmentTextCustodyDelegationV1,
    consumed_at_unix_seconds: i64,
) -> Result<bool, AttachmentSecurityPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.attachment_security_text_extraction_delegation_inbox (
           message_id, envelope_sha256, exact_envelope_bytes, request_id,
           extraction_run_id, attachment_anchor_id, candidate_message_id,
           candidate_envelope_sha256, safety_message_id, safety_evidence_id,
           logical_owner_id, consumed_at_unix_seconds
         ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
         ON CONFLICT (message_id) DO NOTHING",
    )
    .bind(record.message_id().as_slice())
    .bind(record.envelope_sha256().as_slice())
    .bind(record.exact_bytes())
    .bind(&request.request_id)
    .bind(&request.extraction_run_id)
    .bind(&request.attachment_anchor_id)
    .bind(&request.candidate_message_id)
    .bind(&request.candidate_envelope_sha256)
    .bind(&request.safety_message_id)
    .bind(&request.safety_evidence_id)
    .bind(&request.logical_owner_id)
    .bind(consumed_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    Ok(inserted.rows_affected() == 1)
}

async fn verify_request_replay(
    transaction: &mut Transaction<'_, Postgres>,
    record: &OutboxRecordV1,
) -> Result<(), AttachmentSecurityPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT envelope_sha256, exact_envelope_bytes
         FROM makosh_data.attachment_security_text_extraction_delegation_inbox
         WHERE message_id = $1",
    )
    .bind(record.message_id().as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    let sha = id32(
        &row.try_get::<Vec<u8>, _>("envelope_sha256")
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
    )?;
    let bytes = row
        .try_get::<Vec<u8>, _>("exact_envelope_bytes")
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
    if sha == *record.envelope_sha256() && bytes == record.exact_bytes() {
        Ok(())
    } else {
        Err(AttachmentSecurityPersistenceErrorV1::EvidenceConflict)
    }
}

async fn verify_delegation_source(
    transaction: &mut Transaction<'_, Postgres>,
    request: &RequestAttachmentTextCustodyDelegationV1,
) -> Result<Option<VerifiedDelegationSourceV1>, AttachmentSecurityPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT job.target_blob_reference_id, job.target_blob_receipt_sha256,
                candidate.declared_size, candidate.custody_transfer_source_proof,
                verdict.exact_envelope_bytes
         FROM makosh_data.attachment_security_scan_jobs AS job
         JOIN makosh_data.attachment_security_scan_candidates AS candidate
           ON candidate.message_id = job.candidate_message_id
         JOIN makosh_data.attachment_security_event_inbox AS candidate_inbox
           ON candidate_inbox.message_id = candidate.message_id
         JOIN makosh_data.attachment_security_verdict_outbox AS verdict
           ON verdict.message_id = job.outbox_message_id
         WHERE job.state = 2 AND job.attachment_anchor_id = $1
           AND job.candidate_message_id = $2 AND candidate_inbox.envelope_sha256 = $3
           AND job.target_blob_reference_id IS NOT NULL
           AND job.target_blob_receipt_sha256 IS NOT NULL",
    )
    .bind(&request.attachment_anchor_id)
    .bind(&request.candidate_message_id)
    .bind(&request.candidate_envelope_sha256)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let verdict_bytes = row
        .try_get::<Vec<u8>, _>("exact_envelope_bytes")
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
    if !safe_verdict_matches(request, &verdict_bytes)? {
        return Ok(None);
    }
    Ok(Some(VerifiedDelegationSourceV1 {
        current_reference_id: id16(
            &row.try_get::<Vec<u8>, _>("target_blob_reference_id")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        )?,
        current_receipt_sha256: id32(
            &row.try_get::<Vec<u8>, _>("target_blob_receipt_sha256")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        )?,
        declared_size: u64::try_from(
            row.try_get::<i64, _>("declared_size")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        )
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        predecessor_custody_source_proof: row
            .try_get("custody_transfer_source_proof")
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
    }))
}

async fn insert_delegation_job(
    transaction: &mut Transaction<'_, Postgres>,
    message_id: [u8; 16],
    request: &RequestAttachmentTextCustodyDelegationV1,
    source: Option<&VerifiedDelegationSourceV1>,
    rejection_code: Option<AttachmentTextCustodyDelegationRejectCodeV1>,
    created_at_unix_seconds: i64,
) -> Result<(), AttachmentSecurityPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.attachment_security_text_extraction_delegation_jobs (
           request_message_id, request_id, current_reference_id,
           current_receipt_sha256, declared_size, predecessor_custody_source_proof,
           rejection_code, state, next_attempt_at_unix_seconds
         ) VALUES ($1,$2,$3,$4,$5,$6,$7,1,$8)",
    )
    .bind(message_id.as_slice())
    .bind(&request.request_id)
    .bind(source.map(|value| value.current_reference_id.to_vec()))
    .bind(source.map(|value| value.current_receipt_sha256.to_vec()))
    .bind(
        source
            .map(|value| i64::try_from(value.declared_size))
            .transpose()
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?,
    )
    .bind(source.map(|value| value.predecessor_custody_source_proof.as_slice()))
    .bind(rejection_code.map(|value| value as i16))
    .bind(created_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    Ok(())
}

async fn insert_result_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    record: &OutboxRecordV1,
    created_at_unix_seconds: i64,
) -> Result<(), AttachmentSecurityPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.attachment_security_text_extraction_delegation_outbox (
           message_id, envelope_sha256, exact_envelope_bytes, created_at_unix_seconds
         ) VALUES ($1,$2,$3,$4) ON CONFLICT (message_id) DO NOTHING",
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
        "SELECT envelope_sha256, exact_envelope_bytes
         FROM makosh_data.attachment_security_text_extraction_delegation_outbox
         WHERE message_id = $1",
    )
    .bind(record.message_id().as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| AttachmentSecurityPersistenceErrorV1::StorageUnavailable)?;
    let sha = id32(
        &row.try_get::<Vec<u8>, _>("envelope_sha256")
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
    )?;
    let bytes = row
        .try_get::<Vec<u8>, _>("exact_envelope_bytes")
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
    if sha == *record.envelope_sha256() && bytes == record.exact_bytes() {
        Ok(())
    } else {
        Err(AttachmentSecurityPersistenceErrorV1::OutboxHashConflict)
    }
}

fn decode_request(
    exact_envelope_bytes: &[u8],
) -> Result<
    (DurableEnvelopeV1, RequestAttachmentTextCustodyDelegationV1),
    AttachmentSecurityPersistenceErrorV1,
> {
    let envelope = decode_envelope_v1(exact_envelope_bytes)
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?;
    let request = RequestAttachmentTextCustodyDelegationV1::decode(envelope.payload.as_slice())
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?;
    let Some(Semantics::Command(metadata)) = envelope.semantics.as_ref() else {
        return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
    };
    if !exact_contract(
        envelope.contract.as_ref(),
        ATTACHMENT_TEXT_EXTRACTION_INGRESS_OWNER_V1,
        ATTACHMENT_TEXT_EXTRACTION_CUSTODY_DELEGATION_REQUESTED_CONTRACT_NAME_V1,
        ATTACHMENT_TEXT_EXTRACTION_INGRESS_CONTRACT_MAJOR_V1,
        ATTACHMENT_TEXT_EXTRACTION_INGRESS_CONTRACT_REVISION_V1,
        &ATTACHMENT_TEXT_EXTRACTION_INGRESS_SCHEMA_SHA256,
    ) || metadata.target_capability
        != ATTACHMENT_SECURITY_TEXT_EXTRACTION_DELEGATION_CAPABILITY_ID_V1
        || metadata.command_id != request.request_id
        || envelope.message_id != request.request_id
        || envelope.partition_key != request.extraction_run_id
        || !valid_request(&request)
    {
        return Err(AttachmentSecurityPersistenceErrorV1::InvalidInput);
    }
    Ok((envelope, request))
}

fn safe_verdict_matches(
    request: &RequestAttachmentTextCustodyDelegationV1,
    exact_envelope_bytes: &[u8],
) -> Result<bool, AttachmentSecurityPersistenceErrorV1> {
    let envelope = decode_envelope_v1(exact_envelope_bytes)
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
    let Some(Semantics::Observation(_)) = envelope.semantics.as_ref() else {
        return Ok(false);
    };
    let payload = AttachmentSafetyVerdictObservationV1::decode(envelope.payload.as_slice())
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
    Ok(exact_contract(
        envelope.contract.as_ref(),
        COMMUNICATION_ATTACHMENT_CONTRACT_OWNER,
        COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVED_CONTRACT_NAME,
        COMMUNICATION_ATTACHMENT_CONTRACT_MAJOR,
        COMMUNICATION_ATTACHMENT_CONTRACT_REVISION,
        &COMMUNICATION_ATTACHMENT_SAFETY_VERDICT_OBSERVATION_SCHEMA_SHA256,
    ) && envelope.partition_key == request.attachment_anchor_id
        && payload.attachment_anchor_id == request.attachment_anchor_id
        && payload.evidence_id == request.safety_evidence_id
        && payload.expected_state == AttachmentSafetyExpectedStateV1::BlobAdmitted as i32
        && payload.verdict == AttachmentSafetyVerdictV1::SafeForDelivery as i32)
}

fn claimed_from_row(
    row: sqlx::postgres::PgRow,
    worker_id: &str,
) -> Result<ClaimedAttachmentSecurityTextDelegationV1, AttachmentSecurityPersistenceErrorV1> {
    let request = request_from_row(&row)?;
    let rejection_code = row
        .try_get::<Option<i16>, _>("rejection_code")
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?;
    let work = if let Some(code) = rejection_code {
        AttachmentSecurityTextDelegationWorkV1::Reject {
            request,
            code: AttachmentTextCustodyDelegationRejectCodeV1::try_from(i32::from(code))
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        }
    } else {
        AttachmentSecurityTextDelegationWorkV1::Delegate {
            request,
            current_reference_id: id16(&required_bytes(&row, "current_reference_id")?)?,
            current_receipt_sha256: id32(&required_bytes(&row, "current_receipt_sha256")?)?,
            declared_size: u64::try_from(
                row.try_get::<i64, _>("declared_size")
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            )
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
            predecessor_custody_source_proof: required_bytes(
                &row,
                "predecessor_custody_source_proof",
            )?,
        }
    };
    let claimed = ClaimedAttachmentSecurityTextDelegationV1 {
        command_message_id: id16(&required_bytes(&row, "request_message_id")?)?,
        work,
        worker_id: worker_id.to_owned(),
        attempt_count: u32::try_from(
            row.try_get::<i32, _>("attempt_count")
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
        )
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
    };
    valid_claim(&claimed)
        .then_some(claimed)
        .ok_or(AttachmentSecurityPersistenceErrorV1::InvalidRow)
}

fn request_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<RequestAttachmentTextCustodyDelegationV1, AttachmentSecurityPersistenceErrorV1> {
    let request = RequestAttachmentTextCustodyDelegationV1 {
        request_id: required_bytes(row, "request_id")?,
        extraction_run_id: required_bytes(row, "extraction_run_id")?,
        attachment_anchor_id: required_bytes(row, "attachment_anchor_id")?,
        candidate_message_id: required_bytes(row, "candidate_message_id")?,
        candidate_envelope_sha256: required_bytes(row, "candidate_envelope_sha256")?,
        safety_message_id: required_bytes(row, "safety_message_id")?,
        safety_evidence_id: required_bytes(row, "safety_evidence_id")?,
        logical_owner_id: row
            .try_get("logical_owner_id")
            .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)?,
    };
    valid_request(&request)
        .then_some(request)
        .ok_or(AttachmentSecurityPersistenceErrorV1::InvalidRow)
}

fn valid_result_for_claim(
    claimed: &ClaimedAttachmentSecurityTextDelegationV1,
    record: &OutboxRecordV1,
) -> Result<bool, AttachmentSecurityPersistenceErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?;
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_ref() else {
        return Ok(false);
    };
    if metadata.command_message_id != claimed.command_message_id {
        return Ok(false);
    }
    match &claimed.work {
        AttachmentSecurityTextDelegationWorkV1::Delegate {
            request,
            current_reference_id,
            current_receipt_sha256,
            declared_size,
            ..
        } => {
            let payload = AttachmentTextCustodyDelegatedV1::decode(envelope.payload.as_slice())
                .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?;
            Ok(exact_contract(
                envelope.contract.as_ref(),
                ATTACHMENT_TEXT_EXTRACTION_INGRESS_OWNER_V1,
                ATTACHMENT_TEXT_EXTRACTION_CUSTODY_DELEGATED_CONTRACT_NAME_V1,
                ATTACHMENT_TEXT_EXTRACTION_INGRESS_CONTRACT_MAJOR_V1,
                ATTACHMENT_TEXT_EXTRACTION_INGRESS_CONTRACT_REVISION_V1,
                &ATTACHMENT_TEXT_EXTRACTION_INGRESS_SCHEMA_SHA256,
            ) && record.message_id()
                == &attachment_text_custody_delegated_message_id_v1(id16(&request.request_id)?)
                && payload.request_id == request.request_id
                && payload.extraction_run_id == request.extraction_run_id
                && payload.attachment_anchor_id == request.attachment_anchor_id
                && payload.candidate_message_id == request.candidate_message_id
                && payload.safety_message_id == request.safety_message_id
                && payload.source_reference_id == *current_reference_id
                && payload.receipt_sha256 == *current_receipt_sha256
                && payload.declared_size == *declared_size
                && (1..=ATTACHMENT_TEXT_EXTRACTION_MAX_PROOF_BYTES_V1)
                    .contains(&payload.custody_transfer_source_proof.len())
                && payload.logical_owner_id == request.logical_owner_id)
        }
        AttachmentSecurityTextDelegationWorkV1::Reject { request, code } => {
            let payload =
                AttachmentTextCustodyDelegationRejectedV1::decode(envelope.payload.as_slice())
                    .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidInput)?;
            Ok(exact_contract(
                envelope.contract.as_ref(),
                ATTACHMENT_TEXT_EXTRACTION_INGRESS_OWNER_V1,
                ATTACHMENT_TEXT_EXTRACTION_CUSTODY_DELEGATION_REJECTED_CONTRACT_NAME_V1,
                ATTACHMENT_TEXT_EXTRACTION_INGRESS_CONTRACT_MAJOR_V1,
                ATTACHMENT_TEXT_EXTRACTION_INGRESS_CONTRACT_REVISION_V1,
                &ATTACHMENT_TEXT_EXTRACTION_INGRESS_SCHEMA_SHA256,
            ) && record.message_id()
                == &attachment_text_custody_delegation_rejected_message_id_v1(id16(
                    &request.request_id,
                )?)
                && payload.request_id == request.request_id
                && payload.extraction_run_id == request.extraction_run_id
                && payload.attachment_anchor_id == request.attachment_anchor_id
                && payload.code == *code as i32
                && payload.logical_owner_id == request.logical_owner_id)
        }
    }
}

fn valid_claim(value: &ClaimedAttachmentSecurityTextDelegationV1) -> bool {
    valid_id16(&value.command_message_id) && valid_worker_id(&value.worker_id)
        && (1..=MAX_ATTEMPTS).contains(&value.attempt_count)
        && match &value.work {
            AttachmentSecurityTextDelegationWorkV1::Delegate {
                request, current_reference_id, current_receipt_sha256,
                declared_size, predecessor_custody_source_proof,
            } => valid_request(request) && valid_id16(current_reference_id)
                && valid_sha256(current_receipt_sha256)
                && (1..=ATTACHMENT_TEXT_EXTRACTION_MAX_SOURCE_BYTES_V1).contains(declared_size)
                && (1..=ATTACHMENT_TEXT_EXTRACTION_MAX_PROOF_BYTES_V1).contains(&predecessor_custody_source_proof.len()),
            AttachmentSecurityTextDelegationWorkV1::Reject { request, code } => {
                valid_request(request)
                    && *code != AttachmentTextCustodyDelegationRejectCodeV1::AttachmentTextExtractionCustodyDelegationRejectCodeV1Unspecified
            }
        }
}

fn valid_request(value: &RequestAttachmentTextCustodyDelegationV1) -> bool {
    [
        value.request_id.as_slice(),
        value.extraction_run_id.as_slice(),
        value.attachment_anchor_id.as_slice(),
        value.candidate_message_id.as_slice(),
        value.safety_message_id.as_slice(),
        value.safety_evidence_id.as_slice(),
    ]
    .into_iter()
    .all(|value| value.len() == 16 && value.iter().any(|byte| *byte != 0))
        && value.candidate_envelope_sha256.len() == 32
        && value
            .candidate_envelope_sha256
            .iter()
            .any(|byte| *byte != 0)
        && valid_owner(&value.logical_owner_id)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_worker_id(value: &str) -> bool {
    valid_owner(value)
}

fn exact_contract(
    value: Option<&ContractRefV1>,
    owner: &str,
    name: &str,
    major: u32,
    revision: u32,
    schema_sha256: &[u8; 32],
) -> bool {
    value.is_some_and(|value| {
        value.owner == owner
            && value.name == name
            && value.major == major
            && value.revision == revision
            && value.schema_sha256 == schema_sha256
    })
}

fn required_bytes(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<Vec<u8>, AttachmentSecurityPersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| AttachmentSecurityPersistenceErrorV1::InvalidRow)
}

#[cfg(test)]
mod tests {
    use makosh_attachment_text_extraction_ingress::{
        AttachmentTextCustodyEnvelopeContextV1,
        build_request_attachment_text_custody_delegation_outbox_record_v1,
    };

    use super::*;

    #[test]
    fn exact_request_decoder_rejects_tampered_command_target() {
        let request = request();
        let record = build_request_attachment_text_custody_delegation_outbox_record_v1(
            request.clone(),
            1_700_000_100,
            &AttachmentTextCustodyEnvelopeContextV1 {
                module_id: "makosh-attachment-text-extraction-runtime".to_owned(),
                runtime_instance_id: "text-runtime-1".to_owned(),
                runtime_generation: 2,
                recorded_at_unix_seconds: 1_700_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("request record");
        let (_, decoded) = decode_request(record.exact_bytes()).expect("exact request");
        assert_eq!(decoded, request);
        let mut envelope = decode_envelope_v1(record.exact_bytes()).expect("envelope");
        let Some(Semantics::Command(metadata)) = envelope.semantics.as_mut() else {
            panic!("command");
        };
        metadata.target_capability = "attachment_security.wrong.v1".to_owned();
        assert_eq!(
            decode_request(&envelope.encode_to_vec()),
            Err(AttachmentSecurityPersistenceErrorV1::InvalidInput)
        );
    }

    fn request() -> RequestAttachmentTextCustodyDelegationV1 {
        RequestAttachmentTextCustodyDelegationV1 {
            request_id: vec![1; 16],
            extraction_run_id: vec![2; 16],
            attachment_anchor_id: vec![3; 16],
            candidate_message_id: vec![4; 16],
            candidate_envelope_sha256: vec![5; 32],
            safety_message_id: vec![6; 16],
            safety_evidence_id: vec![7; 16],
            logical_owner_id: "owner-a".to_owned(),
        }
    }
}
