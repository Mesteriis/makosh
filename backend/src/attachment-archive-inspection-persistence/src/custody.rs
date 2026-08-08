use makosh_attachment_archive_inspection_core::{
    ArchiveInspectionCustodyDelegationIntentV1, ArchiveInspectionErrorV1, ArchiveInspectionStateV1,
    ArchiveInspectionTransitionV1, transition_archive_inspection_status_v1,
};
use makosh_attachment_archive_inspection_ingress::{
    ARCHIVE_INSPECTION_CUSTODY_DELEGATION_REQUESTED_CONTRACT_NAME_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_INGRESS_CONTRACT_MAJOR_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_INGRESS_CONTRACT_REVISION_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_INGRESS_OWNER_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_INGRESS_SCHEMA_SHA256,
    archive_inspection_custody_delegated_message_id_v1,
    archive_inspection_custody_delegation_rejected_message_id_v1,
    archive_inspection_custody_delegation_request_id_v1,
    wire::{
        ArchiveInspectionCustodyDelegatedV1, ArchiveInspectionCustodyDelegationRejectCodeV1,
        ArchiveInspectionCustodyDelegationRejectedV1, RequestArchiveInspectionCustodyDelegationV1,
    },
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{DurableEnvelopeV1, durable_envelope_v1::Semantics},
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    ArchiveInspectionPersistenceErrorV1, AttachmentArchiveInspectionPersistenceV1,
    PendingArchiveInspectionCustodyDelegationV1, PersistArchiveInspectionCustodyResultOutcomeV1,
    UnpublishedArchiveInspectionCustodyDelegationV1, archive_inspection_terminal_evidence_id_v1,
    id16, id32,
    jobs::{ArchiveInspectionDelegatedWorkV1, enqueue_archive_inspection_work},
    model::{valid_owner, valid_sha256, valid_timestamp_millis},
    runs::{load_run_for_update, persist_status},
};

const CUSTODY_DELEGATION_OUTBOX_LIMIT_V1: u16 = 64;
const CUSTODY_DELEGATION_MAX_ENVELOPE_BYTES_V1: usize = 8_192;

pub(crate) async fn enqueue_archive_inspection_custody_delegation(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    intent: &ArchiveInspectionCustodyDelegationIntentV1,
    candidate_envelope_sha256: [u8; 32],
    created_at_unix_millis: i64,
) -> Result<[u8; 16], ArchiveInspectionPersistenceErrorV1> {
    let request_id = archive_inspection_custody_delegation_request_id_v1(
        intent.run_id,
        intent.candidate_message_id,
        intent.safety_message_id,
    );
    sqlx::query(
        "INSERT INTO makosh_data.attachment_archive_inspection_custody_delegation_requests (
           logical_owner_id, request_id, run_id, attachment_anchor_id,
           candidate_message_id, candidate_envelope_sha256,
           safety_message_id, safety_evidence_id, state,
           envelope_sha256, exact_envelope_bytes, published_at_unix_millis,
           result_message_id, created_at_unix_millis, updated_at_unix_millis
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1, NULL, NULL, NULL, NULL, $9, $9)
         ON CONFLICT (logical_owner_id, run_id) DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(request_id.as_slice())
    .bind(intent.run_id.as_slice())
    .bind(intent.attachment_anchor_id.as_slice())
    .bind(intent.candidate_message_id.as_slice())
    .bind(candidate_envelope_sha256.as_slice())
    .bind(intent.safety_message_id.as_slice())
    .bind(intent.safety_evidence_id.as_slice())
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    let row = sqlx::query(
        "SELECT request_id, attachment_anchor_id, candidate_message_id,
                candidate_envelope_sha256, safety_message_id, safety_evidence_id
         FROM makosh_data.attachment_archive_inspection_custody_delegation_requests
         WHERE logical_owner_id = $1 AND run_id = $2",
    )
    .bind(logical_owner_id)
    .bind(intent.run_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?;
    let exact = id16(&row_bytes(&row, "request_id")?)? == request_id
        && id16(&row_bytes(&row, "attachment_anchor_id")?)? == intent.attachment_anchor_id
        && id16(&row_bytes(&row, "candidate_message_id")?)? == intent.candidate_message_id
        && id32(&row_bytes(&row, "candidate_envelope_sha256")?)? == candidate_envelope_sha256
        && id16(&row_bytes(&row, "safety_message_id")?)? == intent.safety_message_id
        && id16(&row_bytes(&row, "safety_evidence_id")?)? == intent.safety_evidence_id;
    if !exact {
        return Err(ArchiveInspectionPersistenceErrorV1::EvidenceConflict);
    }
    Ok(request_id)
}

impl AttachmentArchiveInspectionPersistenceV1 {
    pub async fn pending_custody_delegation_requests(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<Vec<PendingArchiveInspectionCustodyDelegationV1>, ArchiveInspectionPersistenceErrorV1>
    {
        if !valid_owner(logical_owner_id)
            || !(1..=CUSTODY_DELEGATION_OUTBOX_LIMIT_V1).contains(&limit)
        {
            return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT request_id, run_id, attachment_anchor_id, candidate_message_id,
                    candidate_envelope_sha256, safety_message_id, safety_evidence_id,
                    created_at_unix_millis
             FROM makosh_data.attachment_archive_inspection_custody_delegation_requests
             WHERE logical_owner_id = $1 AND state = 1
             ORDER BY created_at_unix_millis, request_id
             LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?
        .into_iter()
        .map(|row| {
            let created_at_unix_millis = row
                .try_get("created_at_unix_millis")
                .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
            let request = RequestArchiveInspectionCustodyDelegationV1 {
                request_id: row_bytes(&row, "request_id")?,
                archive_run_id: row_bytes(&row, "run_id")?,
                attachment_anchor_id: row_bytes(&row, "attachment_anchor_id")?,
                candidate_message_id: row_bytes(&row, "candidate_message_id")?,
                candidate_envelope_sha256: row_bytes(&row, "candidate_envelope_sha256")?,
                safety_message_id: row_bytes(&row, "safety_message_id")?,
                safety_evidence_id: row_bytes(&row, "safety_evidence_id")?,
                logical_owner_id: logical_owner_id.to_owned(),
            };
            validate_request(&request)?;
            Ok(PendingArchiveInspectionCustodyDelegationV1 {
                request,
                created_at_unix_millis,
            })
        })
        .collect()
    }

    pub async fn store_custody_delegation_outbox(
        &self,
        logical_owner_id: &str,
        record: &OutboxRecordV1,
        materialized_at_unix_millis: i64,
    ) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_timestamp_millis(materialized_at_unix_millis)
            || record.exact_bytes().len() > CUSTODY_DELEGATION_MAX_ENVELOPE_BYTES_V1
        {
            return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
        }
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes())
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?;
        validate_envelope_v1(&envelope)
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?;
        let request = validate_request_envelope(&envelope)?;
        if record.message_id() != request.request_id.as_slice()
            || record.envelope_sha256() != Sha256::digest(record.exact_bytes()).as_slice()
        {
            return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.attachment_archive_inspection_custody_delegation_requests
             SET state = 2, envelope_sha256 = $1, exact_envelope_bytes = $2,
                 updated_at_unix_millis = $3
             WHERE logical_owner_id = $4 AND request_id = $5 AND state = 1
               AND run_id = $6 AND attachment_anchor_id = $7
               AND candidate_message_id = $8 AND candidate_envelope_sha256 = $9
               AND safety_message_id = $10 AND safety_evidence_id = $11",
        )
        .bind(record.envelope_sha256().as_slice())
        .bind(record.exact_bytes())
        .bind(materialized_at_unix_millis)
        .bind(logical_owner_id)
        .bind(&request.request_id)
        .bind(&request.archive_run_id)
        .bind(&request.attachment_anchor_id)
        .bind(&request.candidate_message_id)
        .bind(&request.candidate_envelope_sha256)
        .bind(&request.safety_message_id)
        .bind(&request.safety_evidence_id)
        .execute(&self.pool)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if updated == 1 {
            Ok(())
        } else {
            Err(ArchiveInspectionPersistenceErrorV1::EvidenceConflict)
        }
    }

    pub async fn unpublished_custody_delegation_outbox(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<
        Vec<UnpublishedArchiveInspectionCustodyDelegationV1>,
        ArchiveInspectionPersistenceErrorV1,
    > {
        if !valid_owner(logical_owner_id)
            || !(1..=CUSTODY_DELEGATION_OUTBOX_LIMIT_V1).contains(&limit)
        {
            return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT request_id, envelope_sha256, exact_envelope_bytes
             FROM makosh_data.attachment_archive_inspection_custody_delegation_requests
             WHERE logical_owner_id = $1 AND state >= 2
               AND published_at_unix_millis IS NULL
             ORDER BY created_at_unix_millis, request_id
             LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?
        .into_iter()
        .map(|row| {
            let event = UnpublishedArchiveInspectionCustodyDelegationV1 {
                message_id: id16(&row_bytes(&row, "request_id")?)?,
                envelope_sha256: id32(&row_bytes(&row, "envelope_sha256")?)?,
                exact_envelope_bytes: row_bytes(&row, "exact_envelope_bytes")?,
            };
            if !valid_sha256(&event.envelope_sha256)
                || event.exact_envelope_bytes.is_empty()
                || event.exact_envelope_bytes.len() > CUSTODY_DELEGATION_MAX_ENVELOPE_BYTES_V1
                || Sha256::digest(&event.exact_envelope_bytes).as_slice() != event.envelope_sha256
            {
                return Err(ArchiveInspectionPersistenceErrorV1::InvalidRow);
            }
            Ok(event)
        })
        .collect()
    }

    pub async fn mark_custody_delegation_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || message_id.iter().all(|byte| *byte == 0)
            || !valid_sha256(&envelope_sha256)
            || !valid_timestamp_millis(published_at_unix_millis)
        {
            return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.attachment_archive_inspection_custody_delegation_requests
             SET published_at_unix_millis = $1, updated_at_unix_millis = $1
             WHERE logical_owner_id = $2 AND request_id = $3
               AND envelope_sha256 = $4 AND state >= 2
               AND published_at_unix_millis IS NULL
               AND created_at_unix_millis <= $1",
        )
        .bind(published_at_unix_millis)
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .bind(envelope_sha256.as_slice())
        .execute(&self.pool)
        .await
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if updated == 1 {
            Ok(())
        } else {
            Err(ArchiveInspectionPersistenceErrorV1::EvidenceConflict)
        }
    }

    pub async fn persist_custody_delegated_result(
        &self,
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        command_message_id: [u8; 16],
        payload: &ArchiveInspectionCustodyDelegatedV1,
        processed_at_unix_millis: i64,
    ) -> Result<PersistArchiveInspectionCustodyResultOutcomeV1, ArchiveInspectionPersistenceErrorV1>
    {
        let request_id = validate_delegated_result(
            message_id,
            envelope_sha256,
            command_message_id,
            payload,
            processed_at_unix_millis,
        )?;
        let mut transaction = self.begin().await?;
        let request = lock_request(&mut transaction, &payload.logical_owner_id, request_id).await?;
        validate_delegated_against_request(payload, &request)?;
        match insert_result_inbox(
            &mut transaction,
            &payload.logical_owner_id,
            message_id,
            envelope_sha256,
            request_id,
            1,
            processed_at_unix_millis,
        )
        .await?
        {
            ResultInboxInsertV1::Duplicate if request.state == 3 => {
                transaction.commit().await.map_err(storage_error)?;
                return Ok(PersistArchiveInspectionCustodyResultOutcomeV1::Duplicate);
            }
            ResultInboxInsertV1::Duplicate | ResultInboxInsertV1::Conflict => {
                return Err(ArchiveInspectionPersistenceErrorV1::EvidenceConflict);
            }
            ResultInboxInsertV1::New => {}
        }
        if request.state != 2 {
            return Err(ArchiveInspectionPersistenceErrorV1::EvidenceConflict);
        }
        let work = ArchiveInspectionDelegatedWorkV1 {
            run_id: request.run_id,
            operation_id: request.operation_id,
            candidate_message_id: request.candidate_message_id,
            safety_message_id: request.safety_message_id,
            delegation_request_id: request_id,
            delegation_result_message_id: message_id,
            attachment_anchor_id: request.attachment_anchor_id,
            source_reference_id: id16(&payload.source_reference_id)?,
            declared_size: payload.declared_size,
            blob_receipt_sha256: id32(&payload.receipt_sha256)?,
            custody_transfer_source_proof: payload.custody_transfer_source_proof.clone(),
            safety_evidence_id: request.safety_evidence_id,
        };
        enqueue_archive_inspection_work(
            &mut transaction,
            &payload.logical_owner_id,
            &work,
            processed_at_unix_millis,
        )
        .await?;
        transition_run(
            &mut transaction,
            &payload.logical_owner_id,
            request.run_id,
            ArchiveInspectionTransitionV1::BeginInspection,
            None,
            processed_at_unix_millis,
        )
        .await?;
        finish_request(
            &mut transaction,
            &payload.logical_owner_id,
            request_id,
            3,
            message_id,
            processed_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(PersistArchiveInspectionCustodyResultOutcomeV1::Recorded)
    }

    pub async fn persist_custody_delegation_rejected_result(
        &self,
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        command_message_id: [u8; 16],
        payload: &ArchiveInspectionCustodyDelegationRejectedV1,
        processed_at_unix_millis: i64,
    ) -> Result<PersistArchiveInspectionCustodyResultOutcomeV1, ArchiveInspectionPersistenceErrorV1>
    {
        let request_id = validate_rejected_result(
            message_id,
            envelope_sha256,
            command_message_id,
            payload,
            processed_at_unix_millis,
        )?;
        let mut transaction = self.begin().await?;
        let request = lock_request(&mut transaction, &payload.logical_owner_id, request_id).await?;
        if request.run_id != id16(&payload.archive_run_id)?
            || request.attachment_anchor_id != id16(&payload.attachment_anchor_id)?
        {
            return Err(ArchiveInspectionPersistenceErrorV1::EvidenceConflict);
        }
        match insert_result_inbox(
            &mut transaction,
            &payload.logical_owner_id,
            message_id,
            envelope_sha256,
            request_id,
            2,
            processed_at_unix_millis,
        )
        .await?
        {
            ResultInboxInsertV1::Duplicate if request.state == 4 => {
                transaction.commit().await.map_err(storage_error)?;
                return Ok(PersistArchiveInspectionCustodyResultOutcomeV1::Duplicate);
            }
            ResultInboxInsertV1::Duplicate | ResultInboxInsertV1::Conflict => {
                return Err(ArchiveInspectionPersistenceErrorV1::EvidenceConflict);
            }
            ResultInboxInsertV1::New => {}
        }
        if request.state != 2 {
            return Err(ArchiveInspectionPersistenceErrorV1::EvidenceConflict);
        }
        let error =
            if payload.code == ArchiveInspectionCustodyDelegationRejectCodeV1::NotSafe as i32 {
                ArchiveInspectionErrorV1::NotSafe
            } else {
                ArchiveInspectionErrorV1::Unavailable
            };
        transition_run(
            &mut transaction,
            &payload.logical_owner_id,
            request.run_id,
            ArchiveInspectionTransitionV1::Reject(error),
            Some(archive_inspection_terminal_evidence_id_v1(
                request.run_id,
                error,
            )),
            processed_at_unix_millis,
        )
        .await?;
        finish_request(
            &mut transaction,
            &payload.logical_owner_id,
            request_id,
            4,
            message_id,
            processed_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(PersistArchiveInspectionCustodyResultOutcomeV1::Recorded)
    }

    async fn begin(
        &self,
    ) -> Result<Transaction<'_, Postgres>, ArchiveInspectionPersistenceErrorV1> {
        self.pool.begin().await.map_err(storage_error)
    }
}

#[derive(Clone, Debug)]
struct LockedDelegationRequestV1 {
    state: i16,
    run_id: [u8; 16],
    operation_id: [u8; 16],
    attachment_anchor_id: [u8; 16],
    candidate_message_id: [u8; 16],
    safety_message_id: [u8; 16],
    safety_evidence_id: [u8; 16],
}

enum ResultInboxInsertV1 {
    New,
    Duplicate,
    Conflict,
}

async fn lock_request(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    request_id: [u8; 16],
) -> Result<LockedDelegationRequestV1, ArchiveInspectionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT requests.state, requests.run_id, runs.operation_id,
                requests.attachment_anchor_id, requests.candidate_message_id,
                requests.safety_message_id, requests.safety_evidence_id
         FROM makosh_data.attachment_archive_inspection_custody_delegation_requests requests
         JOIN makosh_data.attachment_archive_inspection_runs runs
           ON runs.logical_owner_id = requests.logical_owner_id
          AND runs.run_id = requests.run_id
         WHERE requests.logical_owner_id = $1 AND requests.request_id = $2
         FOR UPDATE OF requests, runs",
    )
    .bind(logical_owner_id)
    .bind(request_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| ArchiveInspectionPersistenceErrorV1::EvidenceConflict)?;
    Ok(LockedDelegationRequestV1 {
        state: row
            .try_get("state")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?,
        run_id: id16(&row_bytes(&row, "run_id")?)?,
        operation_id: id16(&row_bytes(&row, "operation_id")?)?,
        attachment_anchor_id: id16(&row_bytes(&row, "attachment_anchor_id")?)?,
        candidate_message_id: id16(&row_bytes(&row, "candidate_message_id")?)?,
        safety_message_id: id16(&row_bytes(&row, "safety_message_id")?)?,
        safety_evidence_id: id16(&row_bytes(&row, "safety_evidence_id")?)?,
    })
}

async fn insert_result_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    request_id: [u8; 16],
    result_kind: i16,
    processed_at_unix_millis: i64,
) -> Result<ResultInboxInsertV1, ArchiveInspectionPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.attachment_archive_inspection_custody_result_inbox (
           logical_owner_id, message_id, envelope_sha256, request_id,
           result_kind, processed_at_unix_millis
         ) VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (logical_owner_id, message_id) DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(message_id.as_slice())
    .bind(envelope_sha256.as_slice())
    .bind(request_id.as_slice())
    .bind(result_kind)
    .bind(processed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?
    .rows_affected()
        == 1;
    if inserted {
        return Ok(ResultInboxInsertV1::New);
    }
    let row = sqlx::query(
        "SELECT envelope_sha256, request_id, result_kind
         FROM makosh_data.attachment_archive_inspection_custody_result_inbox
         WHERE logical_owner_id = $1 AND message_id = $2",
    )
    .bind(logical_owner_id)
    .bind(message_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_error)?;
    let duplicate = id32(&row_bytes(&row, "envelope_sha256")?)? == envelope_sha256
        && id16(&row_bytes(&row, "request_id")?)? == request_id
        && row
            .try_get::<i16, _>("result_kind")
            .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?
            == result_kind;
    Ok(if duplicate {
        ResultInboxInsertV1::Duplicate
    } else {
        ResultInboxInsertV1::Conflict
    })
}

async fn transition_run(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    transition: ArchiveInspectionTransitionV1,
    rejection_evidence_id: Option<[u8; 16]>,
    occurred_at_unix_millis: i64,
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    let current = load_run_for_update(transaction, logical_owner_id, run_id)
        .await?
        .ok_or(ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    if current.status.state != ArchiveInspectionStateV1::AwaitingEvidence {
        return Err(ArchiveInspectionPersistenceErrorV1::EvidenceConflict);
    }
    let next = transition_archive_inspection_status_v1(&current.status, transition)
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)?;
    persist_status(
        transaction,
        logical_owner_id,
        run_id,
        current.status.state_revision,
        &next,
        rejection_evidence_id,
        occurred_at_unix_millis,
    )
    .await
}

async fn finish_request(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    request_id: [u8; 16],
    state: i16,
    result_message_id: [u8; 16],
    updated_at_unix_millis: i64,
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    let updated = sqlx::query(
        "UPDATE makosh_data.attachment_archive_inspection_custody_delegation_requests
         SET state = $1, result_message_id = $2, updated_at_unix_millis = $3
         WHERE logical_owner_id = $4 AND request_id = $5 AND state = 2",
    )
    .bind(state)
    .bind(result_message_id.as_slice())
    .bind(updated_at_unix_millis)
    .bind(logical_owner_id)
    .bind(request_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?
    .rows_affected();
    if updated == 1 {
        Ok(())
    } else {
        Err(ArchiveInspectionPersistenceErrorV1::EvidenceConflict)
    }
}

fn validate_request_envelope(
    envelope: &DurableEnvelopeV1,
) -> Result<RequestArchiveInspectionCustodyDelegationV1, ArchiveInspectionPersistenceErrorV1> {
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(ArchiveInspectionPersistenceErrorV1::InvalidInput)?;
    let Some(Semantics::Command(command)) = envelope.semantics.as_ref() else {
        return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
    };
    let request = RequestArchiveInspectionCustodyDelegationV1::decode(envelope.payload.as_slice())
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidInput)?;
    validate_request(&request)?;
    if contract.owner != ATTACHMENT_ARCHIVE_INSPECTION_INGRESS_OWNER_V1
        || contract.name != ARCHIVE_INSPECTION_CUSTODY_DELEGATION_REQUESTED_CONTRACT_NAME_V1
        || contract.major != ATTACHMENT_ARCHIVE_INSPECTION_INGRESS_CONTRACT_MAJOR_V1
        || contract.revision != ATTACHMENT_ARCHIVE_INSPECTION_INGRESS_CONTRACT_REVISION_V1
        || contract.schema_sha256 != ATTACHMENT_ARCHIVE_INSPECTION_INGRESS_SCHEMA_SHA256
        || command.command_id != request.request_id
        || envelope.message_id != request.request_id
        || envelope.partition_key != request.archive_run_id
        || !envelope.causation_message_id.is_empty()
        || request.encode_to_vec() != envelope.payload
    {
        return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
    }
    Ok(request)
}

fn validate_request(
    request: &RequestArchiveInspectionCustodyDelegationV1,
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    let request_id = id16(&request.request_id)?;
    let run_id = id16(&request.archive_run_id)?;
    let candidate_message_id = id16(&request.candidate_message_id)?;
    let safety_message_id = id16(&request.safety_message_id)?;
    if request_id
        != archive_inspection_custody_delegation_request_id_v1(
            run_id,
            candidate_message_id,
            safety_message_id,
        )
        || id16(&request.attachment_anchor_id)?
            .iter()
            .all(|byte| *byte == 0)
        || !valid_sha256(&id32(&request.candidate_envelope_sha256)?)
        || id16(&request.safety_evidence_id)?
            .iter()
            .all(|byte| *byte == 0)
        || !valid_owner(&request.logical_owner_id)
    {
        return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_delegated_result(
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    command_message_id: [u8; 16],
    payload: &ArchiveInspectionCustodyDelegatedV1,
    processed_at_unix_millis: i64,
) -> Result<[u8; 16], ArchiveInspectionPersistenceErrorV1> {
    let request_id = id16(&payload.request_id)?;
    if message_id != archive_inspection_custody_delegated_message_id_v1(request_id)
        || command_message_id != request_id
        || !valid_sha256(&envelope_sha256)
        || !valid_timestamp_millis(processed_at_unix_millis)
        || !valid_owner(&payload.logical_owner_id)
        || id16(&payload.archive_run_id)?.iter().all(|byte| *byte == 0)
        || id16(&payload.attachment_anchor_id)?
            .iter()
            .all(|byte| *byte == 0)
        || id16(&payload.candidate_message_id)?
            .iter()
            .all(|byte| *byte == 0)
        || id16(&payload.safety_message_id)?
            .iter()
            .all(|byte| *byte == 0)
        || id16(&payload.source_reference_id)?
            .iter()
            .all(|byte| *byte == 0)
        || !valid_sha256(&id32(&payload.receipt_sha256)?)
        || payload.declared_size == 0
        || payload.declared_size > 100 * 1024 * 1024
        || payload.custody_transfer_source_proof.is_empty()
        || payload.custody_transfer_source_proof.len() > 2_048
    {
        return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
    }
    Ok(request_id)
}

fn validate_rejected_result(
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    command_message_id: [u8; 16],
    payload: &ArchiveInspectionCustodyDelegationRejectedV1,
    processed_at_unix_millis: i64,
) -> Result<[u8; 16], ArchiveInspectionPersistenceErrorV1> {
    let request_id = id16(&payload.request_id)?;
    if message_id != archive_inspection_custody_delegation_rejected_message_id_v1(request_id)
        || command_message_id != request_id
        || !valid_sha256(&envelope_sha256)
        || !valid_timestamp_millis(processed_at_unix_millis)
        || !valid_owner(&payload.logical_owner_id)
        || id16(&payload.archive_run_id)?.iter().all(|byte| *byte == 0)
        || id16(&payload.attachment_anchor_id)?
            .iter()
            .all(|byte| *byte == 0)
        || payload.code == 0
    {
        return Err(ArchiveInspectionPersistenceErrorV1::InvalidInput);
    }
    Ok(request_id)
}

fn validate_delegated_against_request(
    payload: &ArchiveInspectionCustodyDelegatedV1,
    request: &LockedDelegationRequestV1,
) -> Result<(), ArchiveInspectionPersistenceErrorV1> {
    if request.run_id != id16(&payload.archive_run_id)?
        || request.attachment_anchor_id != id16(&payload.attachment_anchor_id)?
        || request.candidate_message_id != id16(&payload.candidate_message_id)?
        || request.safety_message_id != id16(&payload.safety_message_id)?
    {
        return Err(ArchiveInspectionPersistenceErrorV1::EvidenceConflict);
    }
    Ok(())
}

fn row_bytes(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<Vec<u8>, ArchiveInspectionPersistenceErrorV1> {
    row.try_get(column)
        .map_err(|_| ArchiveInspectionPersistenceErrorV1::InvalidRow)
}

fn storage_error(_: sqlx::Error) -> ArchiveInspectionPersistenceErrorV1 {
    ArchiveInspectionPersistenceErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_attachment_archive_inspection_ingress::{
        ArchiveInspectionCustodyEnvelopeContextV1,
        build_request_archive_inspection_custody_delegation_outbox_record_v1,
    };

    #[test]
    fn request_validation_binds_run_candidate_and_safety() {
        let run_id = [1; 16];
        let candidate_message_id = [2; 16];
        let safety_message_id = [3; 16];
        let request = RequestArchiveInspectionCustodyDelegationV1 {
            request_id: archive_inspection_custody_delegation_request_id_v1(
                run_id,
                candidate_message_id,
                safety_message_id,
            )
            .to_vec(),
            archive_run_id: run_id.to_vec(),
            attachment_anchor_id: vec![4; 16],
            candidate_message_id: candidate_message_id.to_vec(),
            candidate_envelope_sha256: vec![5; 32],
            safety_message_id: safety_message_id.to_vec(),
            safety_evidence_id: vec![6; 16],
            logical_owner_id: "owner-1".to_owned(),
        };
        assert_eq!(validate_request(&request), Ok(()));
        let mut changed = request;
        changed.safety_message_id = vec![7; 16];
        assert_eq!(
            validate_request(&changed),
            Err(ArchiveInspectionPersistenceErrorV1::InvalidInput)
        );
    }

    #[test]
    fn terminal_message_ids_are_request_and_outcome_scoped() {
        assert_ne!(
            archive_inspection_custody_delegated_message_id_v1([1; 16]),
            archive_inspection_custody_delegation_rejected_message_id_v1([1; 16])
        );
        assert_ne!(
            archive_inspection_custody_delegated_message_id_v1([1; 16]),
            archive_inspection_custody_delegated_message_id_v1([2; 16])
        );
    }

    #[test]
    fn exact_command_envelope_round_trip_rejects_payload_tampering() {
        let run_id = [1; 16];
        let candidate_message_id = [2; 16];
        let safety_message_id = [3; 16];
        let request = RequestArchiveInspectionCustodyDelegationV1 {
            request_id: archive_inspection_custody_delegation_request_id_v1(
                run_id,
                candidate_message_id,
                safety_message_id,
            )
            .to_vec(),
            archive_run_id: run_id.to_vec(),
            attachment_anchor_id: vec![4; 16],
            candidate_message_id: candidate_message_id.to_vec(),
            candidate_envelope_sha256: vec![5; 32],
            safety_message_id: safety_message_id.to_vec(),
            safety_evidence_id: vec![6; 16],
            logical_owner_id: "owner-1".to_owned(),
        };
        let record = build_request_archive_inspection_custody_delegation_outbox_record_v1(
            request.clone(),
            1_800_000_030,
            &ArchiveInspectionCustodyEnvelopeContextV1 {
                module_id: "makosh-attachment-archive-inspection-runtime".to_owned(),
                runtime_instance_id: "archive-runtime-1".to_owned(),
                runtime_generation: 7,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("command");
        let mut envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        assert_eq!(validate_request_envelope(&envelope), Ok(request));
        envelope.partition_key = vec![9; 16];
        assert_eq!(
            validate_request_envelope(&envelope),
            Err(ArchiveInspectionPersistenceErrorV1::InvalidInput)
        );
    }
}
