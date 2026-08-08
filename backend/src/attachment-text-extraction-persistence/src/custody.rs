//! Exact custody command outbox owned by the text-extraction workflow.

use makosh_attachment_text_extraction_core::{
    AttachmentTextExtractionErrorV1, AttachmentTextExtractionStateV1,
    AttachmentTextExtractionStatusV1, AttachmentTextExtractionTransitionV1,
    transition_attachment_text_status_v1,
};
use makosh_attachment_text_extraction_ingress::{
    ATTACHMENT_TEXT_EXTRACTION_MAX_PROOF_BYTES_V1, ATTACHMENT_TEXT_EXTRACTION_MAX_SOURCE_BYTES_V1,
    attachment_text_custody_delegated_message_id_v1,
    attachment_text_custody_delegation_rejected_message_id_v1,
    attachment_text_custody_delegation_request_id_v1,
    wire::{
        AttachmentTextCustodyDelegatedV1, AttachmentTextCustodyDelegationRejectCodeV1,
        AttachmentTextCustodyDelegationRejectedV1,
    },
};
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::{Postgres, Transaction};

use crate::{
    AttachmentTextExtractionPersistenceErrorV1, AttachmentTextExtractionPersistenceV1,
    PersistAttachmentTextCustodyDelegationV1, PersistAttachmentTextCustodyResultOutcomeV1,
    UnpublishedAttachmentTextCustodyDelegationV1,
    jobs::{AttachmentTextDelegatedWorkV1, enqueue_attachment_text_work},
    model::{state_from_code, valid_id16, valid_owner, valid_sha256, valid_timestamp_millis},
    repository::{append_realtime, update_run_status},
};

const MAX_OUTBOX_ITEMS_V1: u16 = 64;
const MAX_ENVELOPE_BYTES_V1: usize = 8_192;

impl AttachmentTextExtractionPersistenceV1 {
    pub async fn store_custody_delegation_outbox(
        &self,
        logical_owner_id: &str,
        record: &PersistAttachmentTextCustodyDelegationV1,
    ) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id16(&record.request_id)
            || !valid_id16(&record.run_id)
            || !valid_id16(&record.candidate_message_id)
            || !valid_id16(&record.safety_message_id)
            || !valid_sha256(&record.envelope_sha256)
            || record.created_at_unix_millis <= 0
            || record.exact_envelope_bytes.is_empty()
            || record.exact_envelope_bytes.len() > MAX_ENVELOPE_BYTES_V1
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        if record.request_id
            != attachment_text_custody_delegation_request_id_v1(
                record.run_id,
                record.candidate_message_id,
                record.safety_message_id,
            )
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let digest: [u8; 32] = Sha256::digest(&record.exact_envelope_bytes).into();
        if record.envelope_sha256 != digest {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let evidence = sqlx::query(
            "SELECT r.run_id FROM makosh_data.attachment_text_extraction_runs r JOIN makosh_data.attachment_text_extraction_scan_candidates c ON c.logical_owner_id=r.logical_owner_id AND c.attachment_anchor_id=r.attachment_anchor_id JOIN makosh_data.attachment_text_extraction_safety_facts s ON s.logical_owner_id=r.logical_owner_id AND s.attachment_anchor_id=r.attachment_anchor_id WHERE r.logical_owner_id=$1 AND r.run_id=$2 AND c.message_id=$3 AND s.message_id=$4 AND s.next_state=5",
        )
        .bind(logical_owner_id)
        .bind(record.run_id.as_slice())
        .bind(record.candidate_message_id.as_slice())
        .bind(record.safety_message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_unavailable)?
        .ok_or(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
        if id16_row(evidence.try_get("run_id").map_err(invalid_row)?)? != record.run_id {
            return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.attachment_text_extraction_custody_outbox (logical_owner_id,request_id,run_id,candidate_message_id,safety_message_id,envelope_sha256,exact_envelope_bytes,published_at_unix_millis,created_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,NULL,$8) ON CONFLICT (logical_owner_id,run_id) DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(record.request_id.as_slice())
        .bind(record.run_id.as_slice())
        .bind(record.candidate_message_id.as_slice())
        .bind(record.safety_message_id.as_slice())
        .bind(digest.as_slice())
        .bind(&record.exact_envelope_bytes)
        .bind(record.created_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        if inserted.rows_affected() == 0 {
            let existing = sqlx::query("SELECT request_id,envelope_sha256,exact_envelope_bytes FROM makosh_data.attachment_text_extraction_custody_outbox WHERE logical_owner_id=$1 AND run_id=$2")
                .bind(logical_owner_id).bind(record.run_id.as_slice()).fetch_one(&mut *transaction).await.map_err(storage_unavailable)?;
            if existing
                .try_get::<Vec<u8>, _>("request_id")
                .map_err(invalid_row)?
                != record.request_id
                || existing
                    .try_get::<Vec<u8>, _>("envelope_sha256")
                    .map_err(invalid_row)?
                    != digest
                || existing
                    .try_get::<Vec<u8>, _>("exact_envelope_bytes")
                    .map_err(invalid_row)?
                    != record.exact_envelope_bytes
            {
                transaction.rollback().await.map_err(storage_unavailable)?;
                return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
            }
        }
        transaction.commit().await.map_err(storage_unavailable)
    }

    pub async fn unpublished_custody_delegation_outbox(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<
        Vec<UnpublishedAttachmentTextCustodyDelegationV1>,
        AttachmentTextExtractionPersistenceErrorV1,
    > {
        if !valid_owner(logical_owner_id) || !(1..=MAX_OUTBOX_ITEMS_V1).contains(&limit) {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        sqlx::query("SELECT request_id,envelope_sha256,exact_envelope_bytes FROM makosh_data.attachment_text_extraction_custody_outbox WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL ORDER BY created_at_unix_millis,request_id LIMIT $2")
            .bind(logical_owner_id).bind(i64::from(limit)).fetch_all(&self.pool).await.map_err(storage_unavailable)?
            .into_iter().map(|row| {
                let item = UnpublishedAttachmentTextCustodyDelegationV1 {
                    message_id: id16(
                        &row.try_get::<Vec<u8>, _>("request_id")
                            .map_err(invalid_row)?,
                    )?,
                    envelope_sha256: id32(
                        &row.try_get::<Vec<u8>, _>("envelope_sha256")
                            .map_err(invalid_row)?,
                    )?,
                    exact_envelope_bytes: row.try_get("exact_envelope_bytes").map_err(invalid_row)?,
                };
                if item.exact_envelope_bytes.is_empty() || item.exact_envelope_bytes.len() > MAX_ENVELOPE_BYTES_V1
                    || Sha256::digest(&item.exact_envelope_bytes).as_slice() != item.envelope_sha256 {
                    return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow);
                }
                Ok(item)
            }).collect()
    }

    pub async fn mark_custody_delegation_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id16(&message_id)
            || !valid_sha256(&envelope_sha256)
            || published_at_unix_millis <= 0
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let changed = sqlx::query("UPDATE makosh_data.attachment_text_extraction_custody_outbox SET published_at_unix_millis=$1 WHERE logical_owner_id=$2 AND request_id=$3 AND envelope_sha256=$4 AND published_at_unix_millis IS NULL AND created_at_unix_millis <= $1")
            .bind(published_at_unix_millis).bind(logical_owner_id).bind(message_id.as_slice()).bind(envelope_sha256.as_slice())
            .execute(&self.pool).await.map_err(storage_unavailable)?.rows_affected();
        if changed == 1 {
            Ok(())
        } else {
            Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)
        }
    }

    pub async fn persist_custody_delegated_result(
        &self,
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        command_message_id: [u8; 16],
        payload: &AttachmentTextCustodyDelegatedV1,
        processed_at_unix_millis: i64,
    ) -> Result<
        PersistAttachmentTextCustodyResultOutcomeV1,
        AttachmentTextExtractionPersistenceErrorV1,
    > {
        let request_id = validate_delegated_result(
            message_id,
            envelope_sha256,
            command_message_id,
            payload,
            processed_at_unix_millis,
        )?;
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let request = lock_request(&mut transaction, &payload.logical_owner_id, request_id).await?;
        validate_delegated_against_request(payload, &request)?;
        match insert_result_inbox(
            &mut transaction,
            ResultInboxFactV1 {
                logical_owner_id: &payload.logical_owner_id,
                message_id,
                envelope_sha256,
                request_id,
                run_id: request.run_id,
                attachment_anchor_id: request.attachment_anchor_id,
                result_kind: 1,
                processed_at_unix_millis,
            },
        )
        .await?
        {
            ResultInboxInsertV1::Duplicate => {
                transaction.commit().await.map_err(storage_unavailable)?;
                return Ok(PersistAttachmentTextCustodyResultOutcomeV1::Replayed);
            }
            ResultInboxInsertV1::Conflict => {
                transaction.rollback().await.map_err(storage_unavailable)?;
                return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
            }
            ResultInboxInsertV1::New => {}
        }
        let work = AttachmentTextDelegatedWorkV1 {
            request: makosh_attachment_text_extraction_core::AttachmentTextExtractionRequestV1 {
                run_id: request.run_id,
                operation_id: request.operation_id,
                attachment_anchor_id: request.attachment_anchor_id,
            },
            delegation_request_id: request_id,
            delegation_result_message_id: message_id,
            candidate_message_id: request.candidate_message_id,
            safety_message_id: request.safety_message_id,
            source_reference_id: id16(&payload.source_reference_id)?,
            source_receipt_sha256: id32_input(&payload.receipt_sha256)?,
            source_declared_size: payload.declared_size,
            custody_transfer_source_proof: payload.custody_transfer_source_proof.clone(),
        };
        enqueue_attachment_text_work(
            &mut transaction,
            &payload.logical_owner_id,
            &work,
            processed_at_unix_millis,
        )
        .await?;
        transition_custody_result_run(
            &mut transaction,
            &payload.logical_owner_id,
            request.run_id,
            AttachmentTextExtractionTransitionV1::BeginExtraction,
            processed_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(PersistAttachmentTextCustodyResultOutcomeV1::Recorded)
    }

    pub async fn persist_custody_delegation_rejected_result(
        &self,
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        command_message_id: [u8; 16],
        payload: &AttachmentTextCustodyDelegationRejectedV1,
        processed_at_unix_millis: i64,
    ) -> Result<
        PersistAttachmentTextCustodyResultOutcomeV1,
        AttachmentTextExtractionPersistenceErrorV1,
    > {
        let request_id = validate_rejected_result(
            message_id,
            envelope_sha256,
            command_message_id,
            payload,
            processed_at_unix_millis,
        )?;
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let request = lock_request(&mut transaction, &payload.logical_owner_id, request_id).await?;
        if request.run_id != id16(&payload.extraction_run_id)?
            || request.attachment_anchor_id != id16(&payload.attachment_anchor_id)?
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
        }
        match insert_result_inbox(
            &mut transaction,
            ResultInboxFactV1 {
                logical_owner_id: &payload.logical_owner_id,
                message_id,
                envelope_sha256,
                request_id,
                run_id: request.run_id,
                attachment_anchor_id: request.attachment_anchor_id,
                result_kind: 2,
                processed_at_unix_millis,
            },
        )
        .await?
        {
            ResultInboxInsertV1::Duplicate => {
                transaction.commit().await.map_err(storage_unavailable)?;
                return Ok(PersistAttachmentTextCustodyResultOutcomeV1::Replayed);
            }
            ResultInboxInsertV1::Conflict => {
                transaction.rollback().await.map_err(storage_unavailable)?;
                return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
            }
            ResultInboxInsertV1::New => {}
        }
        let error = if payload.code
            == AttachmentTextCustodyDelegationRejectCodeV1::AttachmentTextExtractionCustodyDelegationRejectCodeV1NotSafe
                as i32
        {
            AttachmentTextExtractionErrorV1::NotSafe
        } else {
            AttachmentTextExtractionErrorV1::CustodyRejected
        };
        transition_custody_result_run(
            &mut transaction,
            &payload.logical_owner_id,
            request.run_id,
            AttachmentTextExtractionTransitionV1::Reject(error),
            processed_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(PersistAttachmentTextCustodyResultOutcomeV1::Recorded)
    }
}

#[derive(Clone, Debug)]
struct LockedDelegationRequestV1 {
    run_id: [u8; 16],
    operation_id: [u8; 16],
    attachment_anchor_id: [u8; 16],
    candidate_message_id: [u8; 16],
    safety_message_id: [u8; 16],
    candidate_declared_size: u64,
    candidate_receipt_sha256: [u8; 32],
}

enum ResultInboxInsertV1 {
    New,
    Duplicate,
    Conflict,
}

struct ResultInboxFactV1<'a> {
    logical_owner_id: &'a str,
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    request_id: [u8; 16],
    run_id: [u8; 16],
    attachment_anchor_id: [u8; 16],
    result_kind: i16,
    processed_at_unix_millis: i64,
}

async fn lock_request(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    request_id: [u8; 16],
) -> Result<LockedDelegationRequestV1, AttachmentTextExtractionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT o.run_id,o.candidate_message_id,o.safety_message_id,r.operation_id,r.attachment_anchor_id,c.declared_size AS candidate_declared_size,c.blob_receipt_sha256 AS candidate_receipt_sha256 FROM makosh_data.attachment_text_extraction_custody_outbox o JOIN makosh_data.attachment_text_extraction_runs r ON r.logical_owner_id=o.logical_owner_id AND r.run_id=o.run_id JOIN makosh_data.attachment_text_extraction_scan_candidates c ON c.logical_owner_id=o.logical_owner_id AND c.message_id=o.candidate_message_id AND c.attachment_anchor_id=r.attachment_anchor_id WHERE o.logical_owner_id=$1 AND o.request_id=$2 FOR UPDATE OF o,r,c",
    )
    .bind(logical_owner_id)
    .bind(request_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .ok_or(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
    let locked = LockedDelegationRequestV1 {
        run_id: id16_row(row.try_get("run_id").map_err(invalid_row)?)?,
        operation_id: id16_row(row.try_get("operation_id").map_err(invalid_row)?)?,
        attachment_anchor_id: id16_row(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?,
        candidate_message_id: id16_row(row.try_get("candidate_message_id").map_err(invalid_row)?)?,
        safety_message_id: id16_row(row.try_get("safety_message_id").map_err(invalid_row)?)?,
        candidate_declared_size: u64::try_from(
            row.try_get::<i64, _>("candidate_declared_size")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        candidate_receipt_sha256: id32_row(
            row.try_get("candidate_receipt_sha256")
                .map_err(invalid_row)?,
        )?,
    };
    if request_id
        != attachment_text_custody_delegation_request_id_v1(
            locked.run_id,
            locked.candidate_message_id,
            locked.safety_message_id,
        )
    {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow);
    }
    Ok(locked)
}

async fn insert_result_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    fact: ResultInboxFactV1<'_>,
) -> Result<ResultInboxInsertV1, AttachmentTextExtractionPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.attachment_text_extraction_custody_result_inbox (logical_owner_id,message_id,envelope_sha256,request_id,run_id,attachment_anchor_id,result_kind,processed_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT DO NOTHING",
    )
    .bind(fact.logical_owner_id)
    .bind(fact.message_id.as_slice())
    .bind(fact.envelope_sha256.as_slice())
    .bind(fact.request_id.as_slice())
    .bind(fact.run_id.as_slice())
    .bind(fact.attachment_anchor_id.as_slice())
    .bind(fact.result_kind)
    .bind(fact.processed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    if inserted.rows_affected() == 1 {
        return Ok(ResultInboxInsertV1::New);
    }
    let row = sqlx::query(
        "SELECT message_id,envelope_sha256,request_id,run_id,attachment_anchor_id,result_kind FROM makosh_data.attachment_text_extraction_custody_result_inbox WHERE logical_owner_id=$1 AND (message_id=$2 OR request_id=$3) FOR UPDATE",
    )
    .bind(fact.logical_owner_id)
    .bind(fact.message_id.as_slice())
    .bind(fact.request_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .ok_or(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
    let exact = id16_row(row.try_get("message_id").map_err(invalid_row)?)? == fact.message_id
        && id32_row(row.try_get("envelope_sha256").map_err(invalid_row)?)? == fact.envelope_sha256
        && id16_row(row.try_get("request_id").map_err(invalid_row)?)? == fact.request_id
        && id16_row(row.try_get("run_id").map_err(invalid_row)?)? == fact.run_id
        && id16_row(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?
            == fact.attachment_anchor_id
        && row.try_get::<i16, _>("result_kind").map_err(invalid_row)? == fact.result_kind;
    Ok(if exact {
        ResultInboxInsertV1::Duplicate
    } else {
        ResultInboxInsertV1::Conflict
    })
}

async fn transition_custody_result_run(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    transition: AttachmentTextExtractionTransitionV1,
    occurred_at_unix_millis: i64,
) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
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
    if !matches!(
        state,
        AttachmentTextExtractionStateV1::Accepted
            | AttachmentTextExtractionStateV1::AwaitingEvidence
    ) {
        return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
    }
    let revision = u64::try_from(
        row.try_get::<i64, _>("state_revision")
            .map_err(invalid_row)?,
    )
    .map_err(invalid_row)?;
    let current = AttachmentTextExtractionStatusV1 {
        state,
        state_revision: revision,
        format: None,
        extracted_size_bytes: 0,
        extraction_truncated: false,
        error: None,
    };
    let next = transition_attachment_text_status_v1(&current, transition)
        .map_err(|_| AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
    if !update_run_status(
        transaction,
        logical_owner_id,
        run_id,
        revision,
        &next,
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
        &next,
        occurred_at_unix_millis,
    )
    .await
}

fn validate_delegated_result(
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    command_message_id: [u8; 16],
    payload: &AttachmentTextCustodyDelegatedV1,
    processed_at_unix_millis: i64,
) -> Result<[u8; 16], AttachmentTextExtractionPersistenceErrorV1> {
    let request_id = id16(&payload.request_id)?;
    if message_id != attachment_text_custody_delegated_message_id_v1(request_id)
        || command_message_id != request_id
        || !valid_sha256(&envelope_sha256)
        || !valid_timestamp_millis(processed_at_unix_millis)
        || !valid_owner(&payload.logical_owner_id)
        || !valid_id16(&id16(&payload.extraction_run_id)?)
        || !valid_id16(&id16(&payload.attachment_anchor_id)?)
        || !valid_id16(&id16(&payload.candidate_message_id)?)
        || !valid_id16(&id16(&payload.safety_message_id)?)
        || !valid_id16(&id16(&payload.source_reference_id)?)
        || !valid_sha256(&id32_input(&payload.receipt_sha256)?)
        || !(1..=ATTACHMENT_TEXT_EXTRACTION_MAX_SOURCE_BYTES_V1).contains(&payload.declared_size)
        || !(1..=ATTACHMENT_TEXT_EXTRACTION_MAX_PROOF_BYTES_V1)
            .contains(&payload.custody_transfer_source_proof.len())
    {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
    }
    Ok(request_id)
}

fn validate_rejected_result(
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    command_message_id: [u8; 16],
    payload: &AttachmentTextCustodyDelegationRejectedV1,
    processed_at_unix_millis: i64,
) -> Result<[u8; 16], AttachmentTextExtractionPersistenceErrorV1> {
    let request_id = id16(&payload.request_id)?;
    if message_id != attachment_text_custody_delegation_rejected_message_id_v1(request_id)
        || command_message_id != request_id
        || !valid_sha256(&envelope_sha256)
        || !valid_timestamp_millis(processed_at_unix_millis)
        || !valid_owner(&payload.logical_owner_id)
        || !valid_id16(&id16(&payload.extraction_run_id)?)
        || !valid_id16(&id16(&payload.attachment_anchor_id)?)
        || AttachmentTextCustodyDelegationRejectCodeV1::try_from(payload.code).is_err()
        || payload.code
            == AttachmentTextCustodyDelegationRejectCodeV1::AttachmentTextExtractionCustodyDelegationRejectCodeV1Unspecified
                as i32
    {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
    }
    Ok(request_id)
}

fn validate_delegated_against_request(
    payload: &AttachmentTextCustodyDelegatedV1,
    request: &LockedDelegationRequestV1,
) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
    if request.run_id != id16(&payload.extraction_run_id)?
        || request.attachment_anchor_id != id16(&payload.attachment_anchor_id)?
        || request.candidate_message_id != id16(&payload.candidate_message_id)?
        || request.safety_message_id != id16(&payload.safety_message_id)?
        || request.candidate_declared_size != payload.declared_size
        || request.candidate_receipt_sha256 != id32_input(&payload.receipt_sha256)?
    {
        return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], AttachmentTextExtractionPersistenceErrorV1> {
    value.try_into().map_err(invalid_input)
}
fn id16_row(value: Vec<u8>) -> Result<[u8; 16], AttachmentTextExtractionPersistenceErrorV1> {
    value.try_into().map_err(invalid_row)
}
fn id32_input(value: &[u8]) -> Result<[u8; 32], AttachmentTextExtractionPersistenceErrorV1> {
    value.try_into().map_err(invalid_input)
}
fn id32(value: &[u8]) -> Result<[u8; 32], AttachmentTextExtractionPersistenceErrorV1> {
    value.try_into().map_err(invalid_row)
}
fn id32_row(value: Vec<u8>) -> Result<[u8; 32], AttachmentTextExtractionPersistenceErrorV1> {
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
    fn outbox_bounds_are_explicit() {
        assert_eq!(MAX_OUTBOX_ITEMS_V1, 64);
        assert_eq!(MAX_ENVELOPE_BYTES_V1, 8_192);
    }

    #[test]
    fn delegated_result_must_preserve_candidate_size_and_receipt() {
        let request = locked_request();
        let payload = delegated_payload();
        assert_eq!(
            validate_delegated_against_request(&payload, &request),
            Ok(())
        );

        let mut mismatched_size = payload.clone();
        mismatched_size.declared_size += 1;
        assert_eq!(
            validate_delegated_against_request(&mismatched_size, &request),
            Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)
        );

        let mut mismatched_receipt = payload;
        mismatched_receipt.receipt_sha256 = vec![10; 32];
        assert_eq!(
            validate_delegated_against_request(&mismatched_receipt, &request),
            Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)
        );
    }

    fn locked_request() -> LockedDelegationRequestV1 {
        LockedDelegationRequestV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            attachment_anchor_id: [3; 16],
            candidate_message_id: [4; 16],
            safety_message_id: [5; 16],
            candidate_declared_size: 42,
            candidate_receipt_sha256: [9; 32],
        }
    }

    fn delegated_payload() -> AttachmentTextCustodyDelegatedV1 {
        AttachmentTextCustodyDelegatedV1 {
            request_id: vec![6; 16],
            extraction_run_id: vec![1; 16],
            attachment_anchor_id: vec![3; 16],
            candidate_message_id: vec![4; 16],
            safety_message_id: vec![5; 16],
            source_reference_id: vec![7; 16],
            declared_size: 42,
            receipt_sha256: vec![9; 32],
            custody_transfer_source_proof: vec![8; 64],
            logical_owner_id: "owner-1".to_owned(),
        }
    }
}
