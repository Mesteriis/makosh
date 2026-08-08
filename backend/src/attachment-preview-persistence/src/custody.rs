//! Exact custody outbox and result inbox owned by Attachment Preview.

use makosh_attachment_preview_api::wire::{AttachmentPreviewErrorCodeV1, AttachmentPreviewStateV1};
use makosh_attachment_preview_core::{
    AttachmentPreviewTransitionV1, transition_attachment_preview_status_v1,
};
use makosh_attachment_preview_ingress::{
    ATTACHMENT_PREVIEW_MAX_PROOF_BYTES_V1, ATTACHMENT_PREVIEW_MAX_SOURCE_BYTES_V1,
    attachment_preview_custody_delegated_message_id_v1,
    attachment_preview_custody_delegation_rejected_message_id_v1,
    attachment_preview_custody_delegation_request_id_v1,
    wire::{
        AttachmentPreviewCustodyDelegatedV1, AttachmentPreviewCustodyDelegationRejectCodeV1,
        AttachmentPreviewCustodyDelegationRejectedV1,
    },
};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AttachmentPreviewPersistenceErrorV1, AttachmentPreviewPersistenceV1,
    PersistAttachmentPreviewCustodyDelegationV1, PersistAttachmentPreviewCustodyResultOutcomeV1,
    UnpublishedAttachmentPreviewCustodyDelegationV1,
    jobs::{DelegatedPreviewWorkV1, enqueue_preview_work},
    model::{valid_id16, valid_owner, valid_sha256, valid_timestamp_millis},
    repository::{
        append_realtime, id16, id32, invalid_row, load_run_for_update, storage_unavailable,
        update_run_status,
    },
};

const MAX_OUTBOX_ITEMS_V1: u16 = 64;
const MAX_ENVELOPE_BYTES_V1: usize = 8_192;

impl AttachmentPreviewPersistenceV1 {
    pub async fn store_custody_delegation_outbox(
        &self,
        logical_owner_id: &str,
        record: &PersistAttachmentPreviewCustodyDelegationV1,
    ) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id16(&record.request_id)
            || !valid_id16(&record.run_id)
            || !valid_id16(&record.candidate_message_id)
            || !valid_id16(&record.safety_message_id)
            || !valid_sha256(&record.envelope_sha256)
            || !valid_timestamp_millis(record.created_at_unix_millis)
            || record.exact_envelope_bytes.is_empty()
            || record.exact_envelope_bytes.len() > MAX_ENVELOPE_BYTES_V1
            || record.request_id
                != attachment_preview_custody_delegation_request_id_v1(
                    record.run_id,
                    record.candidate_message_id,
                    record.safety_message_id,
                )
        {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        let digest: [u8; 32] = Sha256::digest(&record.exact_envelope_bytes).into();
        if digest != record.envelope_sha256 {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let evidence = sqlx::query(
            "SELECT r.run_id FROM makosh_data.attachment_preview_runs r JOIN makosh_data.attachment_preview_scan_candidates c ON c.logical_owner_id=r.logical_owner_id AND c.attachment_anchor_id=r.attachment_anchor_id JOIN makosh_data.attachment_preview_safety_facts s ON s.logical_owner_id=r.logical_owner_id AND s.attachment_anchor_id=r.attachment_anchor_id WHERE r.logical_owner_id=$1 AND r.run_id=$2 AND r.state=2 AND c.message_id=$3 AND s.message_id=$4 AND s.expected_state=3 AND s.next_state=4 FOR UPDATE OF r",
        )
        .bind(logical_owner_id)
        .bind(record.run_id.as_slice())
        .bind(record.candidate_message_id.as_slice())
        .bind(record.safety_message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_unavailable)?
        .ok_or(AttachmentPreviewPersistenceErrorV1::EvidenceConflict)?;
        if id16(evidence.try_get("run_id").map_err(invalid_row)?)? != record.run_id {
            return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.attachment_preview_custody_outbox (logical_owner_id,request_id,run_id,candidate_message_id,safety_message_id,envelope_sha256,exact_envelope_bytes,published_at_unix_millis,created_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,NULL,$8) ON CONFLICT (logical_owner_id,run_id) DO NOTHING",
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
            let existing = sqlx::query(
                "SELECT request_id,envelope_sha256,exact_envelope_bytes FROM makosh_data.attachment_preview_custody_outbox WHERE logical_owner_id=$1 AND run_id=$2 FOR UPDATE",
            )
            .bind(logical_owner_id)
            .bind(record.run_id.as_slice())
            .fetch_one(&mut *transaction)
            .await
            .map_err(storage_unavailable)?;
            if id16(existing.try_get("request_id").map_err(invalid_row)?)? != record.request_id
                || id32(existing.try_get("envelope_sha256").map_err(invalid_row)?)? != digest
                || existing
                    .try_get::<Vec<u8>, _>("exact_envelope_bytes")
                    .map_err(invalid_row)?
                    != record.exact_envelope_bytes
            {
                return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
            }
        }
        transaction.commit().await.map_err(storage_unavailable)
    }

    pub async fn unpublished_custody_delegation_outbox(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<
        Vec<UnpublishedAttachmentPreviewCustodyDelegationV1>,
        AttachmentPreviewPersistenceErrorV1,
    > {
        if !valid_owner(logical_owner_id) || !(1..=MAX_OUTBOX_ITEMS_V1).contains(&limit) {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT request_id,envelope_sha256,exact_envelope_bytes FROM makosh_data.attachment_preview_custody_outbox WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL ORDER BY created_at_unix_millis,request_id LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_unavailable)?
        .into_iter()
        .map(|row| {
            let item = UnpublishedAttachmentPreviewCustodyDelegationV1 {
                message_id: id16(row.try_get("request_id").map_err(invalid_row)?)?,
                envelope_sha256: id32(
                    row.try_get("envelope_sha256").map_err(invalid_row)?,
                )?,
                exact_envelope_bytes: row
                    .try_get("exact_envelope_bytes")
                    .map_err(invalid_row)?,
            };
            if item.exact_envelope_bytes.is_empty()
                || item.exact_envelope_bytes.len() > MAX_ENVELOPE_BYTES_V1
                || Sha256::digest(&item.exact_envelope_bytes).as_slice()
                    != item.envelope_sha256
            {
                return Err(AttachmentPreviewPersistenceErrorV1::InvalidRow);
            }
            Ok(item)
        })
        .collect()
    }

    pub async fn mark_custody_delegation_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id16(&message_id)
            || !valid_sha256(&envelope_sha256)
            || !valid_timestamp_millis(published_at_unix_millis)
        {
            return Err(AttachmentPreviewPersistenceErrorV1::InvalidInput);
        }
        let changed = sqlx::query(
            "UPDATE makosh_data.attachment_preview_custody_outbox SET published_at_unix_millis=$1 WHERE logical_owner_id=$2 AND request_id=$3 AND envelope_sha256=$4 AND published_at_unix_millis IS NULL AND created_at_unix_millis<=$1",
        )
        .bind(published_at_unix_millis)
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .bind(envelope_sha256.as_slice())
        .execute(&self.pool)
        .await
        .map_err(storage_unavailable)?
        .rows_affected();
        if changed == 1 {
            Ok(())
        } else {
            Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict)
        }
    }

    pub async fn persist_custody_delegated_result(
        &self,
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        command_message_id: [u8; 16],
        payload: &AttachmentPreviewCustodyDelegatedV1,
        processed_at_unix_millis: i64,
    ) -> Result<PersistAttachmentPreviewCustodyResultOutcomeV1, AttachmentPreviewPersistenceErrorV1>
    {
        let request_id = validate_delegated(
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
                return Ok(PersistAttachmentPreviewCustodyResultOutcomeV1::Replayed);
            }
            ResultInboxInsertV1::Conflict => {
                return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
            }
            ResultInboxInsertV1::New => {}
        }
        enqueue_preview_work(
            &mut transaction,
            &payload.logical_owner_id,
            &DelegatedPreviewWorkV1 {
                run_id: request.run_id,
                operation_id: request.operation_id,
                attachment_anchor_id: request.attachment_anchor_id,
                delegation_request_id: request_id,
                delegation_result_message_id: message_id,
                delegation_result_envelope_sha256: envelope_sha256,
                candidate_message_id: request.candidate_message_id,
                safety_message_id: request.safety_message_id,
                source_reference_id: id16(payload.source_reference_id.clone())?,
                source_receipt_sha256: id32(payload.receipt_sha256.clone())?,
                source_declared_size: payload.declared_size,
                custody_transfer_source_proof: payload.custody_transfer_source_proof.clone(),
            },
            processed_at_unix_millis,
        )
        .await?;
        transition_result_run(
            &mut transaction,
            &payload.logical_owner_id,
            request.run_id,
            AttachmentPreviewTransitionV1::BeginRendering,
            processed_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(PersistAttachmentPreviewCustodyResultOutcomeV1::Recorded)
    }

    pub async fn persist_custody_delegation_rejected_result(
        &self,
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        command_message_id: [u8; 16],
        payload: &AttachmentPreviewCustodyDelegationRejectedV1,
        processed_at_unix_millis: i64,
    ) -> Result<PersistAttachmentPreviewCustodyResultOutcomeV1, AttachmentPreviewPersistenceErrorV1>
    {
        let request_id = validate_rejected(
            message_id,
            envelope_sha256,
            command_message_id,
            payload,
            processed_at_unix_millis,
        )?;
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        let request = lock_request(&mut transaction, &payload.logical_owner_id, request_id).await?;
        if request.run_id != id16(payload.preview_run_id.clone())?
            || request.attachment_anchor_id != id16(payload.attachment_anchor_id.clone())?
        {
            return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
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
                return Ok(PersistAttachmentPreviewCustodyResultOutcomeV1::Replayed);
            }
            ResultInboxInsertV1::Conflict => {
                return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
            }
            ResultInboxInsertV1::New => {}
        }
        let error =
            if payload.code == AttachmentPreviewCustodyDelegationRejectCodeV1::NotSafe as i32 {
                AttachmentPreviewErrorCodeV1::NotSafe
            } else {
                AttachmentPreviewErrorCodeV1::CustodyRejected
            };
        transition_result_run(
            &mut transaction,
            &payload.logical_owner_id,
            request.run_id,
            AttachmentPreviewTransitionV1::Reject(error),
            processed_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(PersistAttachmentPreviewCustodyResultOutcomeV1::Recorded)
    }
}

struct LockedRequestV1 {
    run_id: [u8; 16],
    operation_id: [u8; 16],
    attachment_anchor_id: [u8; 16],
    candidate_message_id: [u8; 16],
    safety_message_id: [u8; 16],
    candidate_declared_size: u64,
    candidate_receipt_sha256: [u8; 32],
}

async fn lock_request(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    request_id: [u8; 16],
) -> Result<LockedRequestV1, AttachmentPreviewPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT o.run_id,o.candidate_message_id,o.safety_message_id,r.operation_id,r.attachment_anchor_id,c.declared_size AS candidate_declared_size,c.source_receipt_sha256 AS candidate_receipt_sha256 FROM makosh_data.attachment_preview_custody_outbox o JOIN makosh_data.attachment_preview_runs r ON r.logical_owner_id=o.logical_owner_id AND r.run_id=o.run_id JOIN makosh_data.attachment_preview_scan_candidates c ON c.logical_owner_id=o.logical_owner_id AND c.message_id=o.candidate_message_id AND c.attachment_anchor_id=r.attachment_anchor_id WHERE o.logical_owner_id=$1 AND o.request_id=$2 FOR UPDATE OF o,r,c",
    )
    .bind(logical_owner_id)
    .bind(request_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_unavailable)?
    .ok_or(AttachmentPreviewPersistenceErrorV1::EvidenceConflict)?;
    Ok(LockedRequestV1 {
        run_id: id16(row.try_get("run_id").map_err(invalid_row)?)?,
        operation_id: id16(row.try_get("operation_id").map_err(invalid_row)?)?,
        attachment_anchor_id: id16(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?,
        candidate_message_id: id16(row.try_get("candidate_message_id").map_err(invalid_row)?)?,
        safety_message_id: id16(row.try_get("safety_message_id").map_err(invalid_row)?)?,
        candidate_declared_size: u64::try_from(
            row.try_get::<i64, _>("candidate_declared_size")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        candidate_receipt_sha256: id32(
            row.try_get("candidate_receipt_sha256")
                .map_err(invalid_row)?,
        )?,
    })
}

fn validate_delegated_against_request(
    payload: &AttachmentPreviewCustodyDelegatedV1,
    request: &LockedRequestV1,
) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
    if id16(payload.preview_run_id.clone())? != request.run_id
        || id16(payload.attachment_anchor_id.clone())? != request.attachment_anchor_id
        || id16(payload.candidate_message_id.clone())? != request.candidate_message_id
        || id16(payload.safety_message_id.clone())? != request.safety_message_id
        || payload.declared_size != request.candidate_declared_size
        || id32(payload.receipt_sha256.clone())? != request.candidate_receipt_sha256
    {
        Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict)
    } else {
        Ok(())
    }
}

fn validate_delegated(
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    command_message_id: [u8; 16],
    payload: &AttachmentPreviewCustodyDelegatedV1,
    processed_at_unix_millis: i64,
) -> Result<[u8; 16], AttachmentPreviewPersistenceErrorV1> {
    let request_id = id16_input(&payload.request_id)?;
    if !valid_id16(&message_id)
        || !valid_sha256(&envelope_sha256)
        || command_message_id != request_id
        || message_id != attachment_preview_custody_delegated_message_id_v1(request_id)
        || !valid_owner(&payload.logical_owner_id)
        || id16_input(&payload.preview_run_id).is_err()
        || id16_input(&payload.attachment_anchor_id).is_err()
        || id16_input(&payload.candidate_message_id).is_err()
        || id16_input(&payload.safety_message_id).is_err()
        || id16_input(&payload.source_reference_id).is_err()
        || id32_input(&payload.receipt_sha256).is_err()
        || !(1..=ATTACHMENT_PREVIEW_MAX_SOURCE_BYTES_V1).contains(&payload.declared_size)
        || !(1..=ATTACHMENT_PREVIEW_MAX_PROOF_BYTES_V1)
            .contains(&payload.custody_transfer_source_proof.len())
        || !valid_timestamp_millis(processed_at_unix_millis)
    {
        Err(AttachmentPreviewPersistenceErrorV1::InvalidInput)
    } else {
        Ok(request_id)
    }
}

fn validate_rejected(
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    command_message_id: [u8; 16],
    payload: &AttachmentPreviewCustodyDelegationRejectedV1,
    processed_at_unix_millis: i64,
) -> Result<[u8; 16], AttachmentPreviewPersistenceErrorV1> {
    let request_id = id16_input(&payload.request_id)?;
    if !valid_id16(&message_id)
        || !valid_sha256(&envelope_sha256)
        || command_message_id != request_id
        || message_id != attachment_preview_custody_delegation_rejected_message_id_v1(request_id)
        || !valid_owner(&payload.logical_owner_id)
        || id16_input(&payload.preview_run_id).is_err()
        || id16_input(&payload.attachment_anchor_id).is_err()
        || AttachmentPreviewCustodyDelegationRejectCodeV1::try_from(payload.code).is_err()
        || payload.code == AttachmentPreviewCustodyDelegationRejectCodeV1::Unspecified as i32
        || !valid_timestamp_millis(processed_at_unix_millis)
    {
        Err(AttachmentPreviewPersistenceErrorV1::InvalidInput)
    } else {
        Ok(request_id)
    }
}

#[derive(Clone, Copy)]
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

enum ResultInboxInsertV1 {
    New,
    Duplicate,
    Conflict,
}

async fn insert_result_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    fact: ResultInboxFactV1<'_>,
) -> Result<ResultInboxInsertV1, AttachmentPreviewPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.attachment_preview_custody_result_inbox (logical_owner_id,message_id,envelope_sha256,request_id,run_id,attachment_anchor_id,result_kind,processed_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (logical_owner_id,message_id) DO NOTHING",
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
        "SELECT envelope_sha256,request_id,run_id,attachment_anchor_id,result_kind FROM makosh_data.attachment_preview_custody_result_inbox WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
    )
    .bind(fact.logical_owner_id)
    .bind(fact.message_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_unavailable)?;
    let exact = id32(row.try_get("envelope_sha256").map_err(invalid_row)?)? == fact.envelope_sha256
        && id16(row.try_get("request_id").map_err(invalid_row)?)? == fact.request_id
        && id16(row.try_get("run_id").map_err(invalid_row)?)? == fact.run_id
        && id16(row.try_get("attachment_anchor_id").map_err(invalid_row)?)?
            == fact.attachment_anchor_id
        && row.try_get::<i16, _>("result_kind").map_err(invalid_row)? == fact.result_kind;
    Ok(if exact {
        ResultInboxInsertV1::Duplicate
    } else {
        ResultInboxInsertV1::Conflict
    })
}

async fn transition_result_run(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    transition: AttachmentPreviewTransitionV1,
    occurred_at_unix_millis: i64,
) -> Result<(), AttachmentPreviewPersistenceErrorV1> {
    let current = load_run_for_update(transaction, logical_owner_id, run_id)
        .await?
        .ok_or(AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
    if current.status.state != AttachmentPreviewStateV1::AwaitingEvidence {
        return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
    }
    let next = transition_attachment_preview_status_v1(&current.status, transition)
        .map_err(|_| AttachmentPreviewPersistenceErrorV1::InvalidRow)?;
    if !update_run_status(
        transaction,
        logical_owner_id,
        run_id,
        current.status.state_revision,
        &next,
        occurred_at_unix_millis,
    )
    .await?
    {
        return Err(AttachmentPreviewPersistenceErrorV1::EvidenceConflict);
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

fn id16_input(value: &[u8]) -> Result<[u8; 16], AttachmentPreviewPersistenceErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| AttachmentPreviewPersistenceErrorV1::InvalidInput)?;
    if valid_id16(&value) {
        Ok(value)
    } else {
        Err(AttachmentPreviewPersistenceErrorV1::InvalidInput)
    }
}

fn id32_input(value: &[u8]) -> Result<[u8; 32], AttachmentPreviewPersistenceErrorV1> {
    let value: [u8; 32] = value
        .try_into()
        .map_err(|_| AttachmentPreviewPersistenceErrorV1::InvalidInput)?;
    if valid_sha256(&value) {
        Ok(value)
    } else {
        Err(AttachmentPreviewPersistenceErrorV1::InvalidInput)
    }
}
