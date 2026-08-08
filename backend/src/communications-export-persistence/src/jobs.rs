use makosh_communications_export_core::{
    EvidenceExportDirectionV1, MAX_EXPORT_ARTIFACT_BYTES_V1, MAX_EXPORT_SOURCE_BYTES_V1,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use sqlx::{Postgres, Row, Transaction};

use crate::{
    CommunicationsExportPersistenceErrorV1, CommunicationsExportPersistenceV1,
    realtime::insert_realtime_transition, valid_id16, valid_sha256, valid_timestamp,
};

const STATE_PENDING_SOURCE: i16 = 1;
const STATE_MATERIALIZING: i16 = 2;
const STATE_READY: i16 = 3;
const STATE_REJECTED: i16 = 4;
const BODY_ADMITTED_UTF8: i16 = 1;
const BODY_UNAVAILABLE: i16 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsExportSourceReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsExportPreparedItemV1 {
    pub message_id: [u8; 16],
    pub conversation_id: [u8; 16],
    pub evidence_id: [u8; 16],
    pub evidence_revision: u64,
    pub direction: EvidenceExportDirectionV1,
    pub occurred_at_unix_seconds: i64,
    pub observed_at_unix_seconds: i64,
    pub participant_display_label: Option<String>,
    pub body_source: Option<CommunicationsExportSourceReceiptV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommunicationsExportArtifactReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsExportClaimV1 {
    pub export_id: [u8; 16],
    pub logical_owner_id: String,
    pub source_result_message_id: [u8; 16],
    pub source_result_envelope_sha256: [u8; 32],
    pub created_at_unix_seconds: i64,
    pub items: Vec<CommunicationsExportPreparedItemV1>,
    pub worker_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommunicationsExportJobStatusV1 {
    pub export_id: [u8; 16],
    pub state: u8,
    pub requested_items: u32,
    pub completed_items: u32,
    pub artifact: Option<CommunicationsExportArtifactReceiptV1>,
    pub rejection_code: Option<u16>,
}

impl CommunicationsExportPersistenceV1 {
    pub async fn create_export_with_outbox(
        &self,
        export_id: [u8; 16],
        logical_owner_id: &str,
        message_ids: &[[u8; 16]],
        outbox: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<(), CommunicationsExportPersistenceErrorV1> {
        if !valid_id16(&export_id)
            || !valid_logical_owner_id(logical_owner_id)
            || message_ids.is_empty()
            || message_ids.len() > 64
            || message_ids.iter().any(|id| !valid_id16(id))
            || message_ids
                .iter()
                .enumerate()
                .any(|(index, id)| message_ids[..index].contains(id))
            || !valid_timestamp(created_at_unix_seconds)
            || outbox.message_id() != &export_id
        {
            return Err(CommunicationsExportPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        let inserted_job = sqlx::query(
            "INSERT INTO makosh_data.communications_export_jobs (
                export_id, logical_owner_id, state, requested_items, completed_items,
                created_at_unix_seconds, updated_at_unix_seconds
             ) VALUES ($1, $2, $3, $4, 0, $5, $5)
             ON CONFLICT (export_id) DO NOTHING",
        )
        .bind(export_id.as_slice())
        .bind(logical_owner_id)
        .bind(STATE_PENDING_SOURCE)
        .bind(
            i32::try_from(message_ids.len())
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidInput)?,
        )
        .bind(created_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        let existing_job: Option<(String, i32)> = sqlx::query_as(
            "SELECT logical_owner_id, requested_items
             FROM makosh_data.communications_export_jobs
             WHERE export_id = $1",
        )
        .bind(export_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        if existing_job
            != Some((
                logical_owner_id.to_owned(),
                i32::try_from(message_ids.len())
                    .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidInput)?,
            ))
        {
            return Err(CommunicationsExportPersistenceErrorV1::Conflict);
        }
        for (ordinal, message_id) in message_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO makosh_data.communications_export_items (
                    export_id, ordinal, message_id
                 ) VALUES ($1, $2, $3)
                 ON CONFLICT (export_id, ordinal) DO NOTHING",
            )
            .bind(export_id.as_slice())
            .bind(
                i32::try_from(ordinal)
                    .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidInput)?,
            )
            .bind(message_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        }
        let existing_message_ids: Vec<Vec<u8>> = sqlx::query_scalar(
            "SELECT message_id
             FROM makosh_data.communications_export_items
             WHERE export_id = $1
             ORDER BY ordinal",
        )
        .bind(export_id.as_slice())
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        if existing_message_ids.len() != message_ids.len()
            || existing_message_ids
                .iter()
                .zip(message_ids)
                .any(|(existing, requested)| existing.as_slice() != requested)
        {
            return Err(CommunicationsExportPersistenceErrorV1::Conflict);
        }
        if inserted_job.rows_affected() == 0 {
            transaction
                .commit()
                .await
                .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
            return Ok(());
        }
        insert_exact_outbox(&mut transaction, outbox, created_at_unix_seconds).await?;
        insert_realtime_transition(
            &mut transaction,
            logical_owner_id,
            &export_id,
            created_at_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)
    }

    pub async fn record_prepared_result(
        &self,
        result_message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        export_id: [u8; 16],
        logical_owner_id: &str,
        items: &[CommunicationsExportPreparedItemV1],
        consumed_at_unix_seconds: i64,
    ) -> Result<(), CommunicationsExportPersistenceErrorV1> {
        if !valid_id16(&result_message_id)
            || !valid_sha256(&envelope_sha256)
            || !valid_id16(&export_id)
            || !valid_logical_owner_id(logical_owner_id)
            || items.is_empty()
            || items.len() > 64
            || !valid_timestamp(consumed_at_unix_seconds)
            || items.iter().any(|item| !valid_prepared_item(item))
        {
            return Err(CommunicationsExportPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.communications_export_event_inbox (
                message_id, envelope_sha256, event_kind, consumed_at_unix_seconds
             ) VALUES ($1, $2, 1, $3)
             ON CONFLICT (message_id) DO NOTHING",
        )
        .bind(result_message_id.as_slice())
        .bind(envelope_sha256.as_slice())
        .bind(consumed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        if inserted.rows_affected() == 0 {
            let existing: Option<Vec<u8>> = sqlx::query_scalar(
                "SELECT envelope_sha256
                 FROM makosh_data.communications_export_event_inbox
                 WHERE message_id = $1",
            )
            .bind(result_message_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
            return if existing.as_deref() == Some(envelope_sha256.as_slice()) {
                Ok(())
            } else {
                Err(CommunicationsExportPersistenceErrorV1::Conflict)
            };
        }
        let expected_ids: Vec<Vec<u8>> = sqlx::query_scalar(
            "SELECT item.message_id
             FROM makosh_data.communications_export_items item
             JOIN makosh_data.communications_export_jobs job
               ON job.export_id = item.export_id
             WHERE item.export_id = $1 AND job.logical_owner_id = $2
             ORDER BY item.ordinal",
        )
        .bind(export_id.as_slice())
        .bind(logical_owner_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        if expected_ids.len() != items.len()
            || expected_ids
                .iter()
                .zip(items)
                .any(|(expected, item)| expected.as_slice() != item.message_id)
        {
            return Err(CommunicationsExportPersistenceErrorV1::Conflict);
        }
        for (ordinal, item) in items.iter().enumerate() {
            let (body_state, reference_id, declared_bytes, sha256, proof) = if let Some(source) =
                item.body_source.as_ref()
            {
                (
                    BODY_ADMITTED_UTF8,
                    Some(source.reference_id.as_slice()),
                    Some(
                        i64::try_from(source.declared_bytes)
                            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidInput)?,
                    ),
                    Some(source.sha256.as_slice()),
                    Some(source.custody_transfer_source_proof.as_slice()),
                )
            } else {
                (BODY_UNAVAILABLE, None, None, None, None)
            };
            sqlx::query(
                "UPDATE makosh_data.communications_export_items
                 SET conversation_id = $3,
                     evidence_id = $4,
                     evidence_revision = $5,
                     direction = $6,
                     occurred_at_unix_seconds = $7,
                     observed_at_unix_seconds = $8,
                     participant_display_label = $9,
                     body_state = $10,
                     source_reference_id = $11,
                     source_declared_bytes = $12,
                     source_sha256 = $13,
                     source_custody_proof = $14
                 WHERE export_id = $1 AND ordinal = $2
                   AND EXISTS (
                     SELECT 1 FROM makosh_data.communications_export_jobs job
                     WHERE job.export_id = $1 AND job.logical_owner_id = $15
                   )",
            )
            .bind(export_id.as_slice())
            .bind(
                i32::try_from(ordinal)
                    .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidInput)?,
            )
            .bind(item.conversation_id.as_slice())
            .bind(item.evidence_id.as_slice())
            .bind(
                i64::try_from(item.evidence_revision)
                    .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidInput)?,
            )
            .bind(direction_code(item.direction))
            .bind(item.occurred_at_unix_seconds)
            .bind(item.observed_at_unix_seconds)
            .bind(item.participant_display_label.as_deref())
            .bind(body_state)
            .bind(reference_id)
            .bind(declared_bytes)
            .bind(sha256)
            .bind(proof)
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.communications_export_jobs
             SET state = $2,
                 source_result_message_id = $3,
                 updated_at_unix_seconds = $4
             WHERE export_id = $1 AND state = $5 AND logical_owner_id = $6",
        )
        .bind(export_id.as_slice())
        .bind(STATE_MATERIALIZING)
        .bind(result_message_id.as_slice())
        .bind(consumed_at_unix_seconds)
        .bind(STATE_PENDING_SOURCE)
        .bind(logical_owner_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        if updated.rows_affected() != 1 {
            return Err(CommunicationsExportPersistenceErrorV1::Conflict);
        }
        insert_realtime_transition(
            &mut transaction,
            logical_owner_id,
            &export_id,
            consumed_at_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)
    }

    pub async fn record_rejected_result(
        &self,
        result_message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        export_id: [u8; 16],
        logical_owner_id: &str,
        rejection_code: u16,
        consumed_at_unix_seconds: i64,
    ) -> Result<(), CommunicationsExportPersistenceErrorV1> {
        if !valid_id16(&result_message_id)
            || !valid_sha256(&envelope_sha256)
            || !valid_id16(&export_id)
            || !valid_logical_owner_id(logical_owner_id)
            || !(1..=16).contains(&rejection_code)
            || !valid_timestamp(consumed_at_unix_seconds)
        {
            return Err(CommunicationsExportPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.communications_export_event_inbox (
                message_id, envelope_sha256, event_kind, consumed_at_unix_seconds
             ) VALUES ($1, $2, 2, $3)
             ON CONFLICT (message_id) DO NOTHING",
        )
        .bind(result_message_id.as_slice())
        .bind(envelope_sha256.as_slice())
        .bind(consumed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        if inserted.rows_affected() == 0 {
            let existing: Option<Vec<u8>> = sqlx::query_scalar(
                "SELECT envelope_sha256
                 FROM makosh_data.communications_export_event_inbox
                 WHERE message_id = $1",
            )
            .bind(result_message_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
            return if existing.as_deref() == Some(envelope_sha256.as_slice()) {
                Ok(())
            } else {
                Err(CommunicationsExportPersistenceErrorV1::Conflict)
            };
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.communications_export_jobs
             SET state = $2,
                 source_result_message_id = $3,
                 rejection_code = $4,
                 updated_at_unix_seconds = $5,
                 claimed_by = NULL,
                 lease_expires_at_unix_seconds = NULL
             WHERE export_id = $1 AND state IN ($6, $7)
               AND logical_owner_id = $8",
        )
        .bind(export_id.as_slice())
        .bind(STATE_REJECTED)
        .bind(result_message_id.as_slice())
        .bind(
            i16::try_from(rejection_code)
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidInput)?,
        )
        .bind(consumed_at_unix_seconds)
        .bind(STATE_PENDING_SOURCE)
        .bind(STATE_MATERIALIZING)
        .bind(logical_owner_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        if updated.rows_affected() != 1 {
            return Err(CommunicationsExportPersistenceErrorV1::Conflict);
        }
        insert_realtime_transition(
            &mut transaction,
            logical_owner_id,
            &export_id,
            consumed_at_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)
    }

    pub async fn claim_next_materialization(
        &self,
        worker_id: &str,
        now_unix_seconds: i64,
        lease_expires_at_unix_seconds: i64,
    ) -> Result<Option<CommunicationsExportClaimV1>, CommunicationsExportPersistenceErrorV1> {
        if !valid_worker_id(worker_id)
            || !valid_timestamp(now_unix_seconds)
            || lease_expires_at_unix_seconds <= now_unix_seconds
        {
            return Err(CommunicationsExportPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        let export_id: Option<Vec<u8>> = sqlx::query_scalar(
            "SELECT export_id
             FROM makosh_data.communications_export_jobs
             WHERE state = $1
               AND (claimed_by IS NULL OR lease_expires_at_unix_seconds <= $2)
             ORDER BY updated_at_unix_seconds, export_id
             FOR UPDATE SKIP LOCKED
             LIMIT 1",
        )
        .bind(STATE_MATERIALIZING)
        .bind(now_unix_seconds)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        let Some(export_id) = export_id else {
            transaction
                .commit()
                .await
                .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
            return Ok(None);
        };
        sqlx::query(
            "UPDATE makosh_data.communications_export_jobs
             SET claimed_by = $2,
                 lease_expires_at_unix_seconds = $3,
                 updated_at_unix_seconds = $4
             WHERE export_id = $1",
        )
        .bind(export_id.as_slice())
        .bind(worker_id)
        .bind(lease_expires_at_unix_seconds)
        .bind(now_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        let job = sqlx::query(
            "SELECT job.logical_owner_id, job.created_at_unix_seconds,
                    job.source_result_message_id,
                    inbox.envelope_sha256
             FROM makosh_data.communications_export_jobs job
             JOIN makosh_data.communications_export_event_inbox inbox
               ON inbox.message_id = job.source_result_message_id
             WHERE job.export_id = $1",
        )
        .bind(export_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        let created_at_unix_seconds: i64 = job
            .try_get("created_at_unix_seconds")
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?;
        let logical_owner_id: String = job
            .try_get("logical_owner_id")
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?;
        let source_result_message_id = id16(
            &job.try_get::<Vec<u8>, _>("source_result_message_id")
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        )?;
        let source_result_envelope_sha256 = id32(
            &job.try_get::<Vec<u8>, _>("envelope_sha256")
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        )?;
        let rows = sqlx::query(
            "SELECT message_id, conversation_id, evidence_id, evidence_revision,
                    direction, occurred_at_unix_seconds, observed_at_unix_seconds,
                    participant_display_label, body_state, source_reference_id,
                    source_declared_bytes, source_sha256, source_custody_proof
             FROM makosh_data.communications_export_items
             WHERE export_id = $1
             ORDER BY ordinal",
        )
        .bind(export_id.as_slice())
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        let items = rows
            .into_iter()
            .map(prepared_item_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        Ok(Some(CommunicationsExportClaimV1 {
            export_id: id16(&export_id)?,
            logical_owner_id,
            source_result_message_id,
            source_result_envelope_sha256,
            created_at_unix_seconds,
            items,
            worker_id: worker_id.to_owned(),
        }))
    }

    pub async fn complete_materialization(
        &self,
        claim: &CommunicationsExportClaimV1,
        artifact: CommunicationsExportArtifactReceiptV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), CommunicationsExportPersistenceErrorV1> {
        if !valid_claim(claim)
            || !valid_artifact(&artifact)
            || !valid_timestamp(completed_at_unix_seconds)
        {
            return Err(CommunicationsExportPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.communications_export_jobs
             SET state = $2,
                 completed_items = requested_items,
                 artifact_reference_id = $3,
                 artifact_declared_bytes = $4,
                 artifact_sha256 = $5,
                 updated_at_unix_seconds = $6,
                 claimed_by = NULL,
                 lease_expires_at_unix_seconds = NULL
             WHERE export_id = $1 AND state = $7 AND claimed_by = $8
               AND logical_owner_id = $9",
        )
        .bind(claim.export_id.as_slice())
        .bind(STATE_READY)
        .bind(artifact.reference_id.as_slice())
        .bind(
            i64::try_from(artifact.declared_bytes)
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidInput)?,
        )
        .bind(artifact.sha256.as_slice())
        .bind(completed_at_unix_seconds)
        .bind(STATE_MATERIALIZING)
        .bind(&claim.worker_id)
        .bind(&claim.logical_owner_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        if updated.rows_affected() != 1 {
            return Err(CommunicationsExportPersistenceErrorV1::ClaimLost);
        }
        insert_realtime_transition(
            &mut transaction,
            &claim.logical_owner_id,
            &claim.export_id,
            completed_at_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)
    }

    pub async fn release_materialization_claim(
        &self,
        claim: &CommunicationsExportClaimV1,
        now_unix_seconds: i64,
    ) -> Result<(), CommunicationsExportPersistenceErrorV1> {
        if !valid_claim(claim) || !valid_timestamp(now_unix_seconds) {
            return Err(CommunicationsExportPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "UPDATE makosh_data.communications_export_jobs
             SET claimed_by = NULL,
                 lease_expires_at_unix_seconds = NULL,
                 updated_at_unix_seconds = $3
             WHERE export_id = $1 AND state = $4 AND claimed_by = $2
               AND logical_owner_id = $5",
        )
        .bind(claim.export_id.as_slice())
        .bind(&claim.worker_id)
        .bind(now_unix_seconds)
        .bind(STATE_MATERIALIZING)
        .bind(&claim.logical_owner_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)
    }

    pub async fn reject_materialization(
        &self,
        claim: &CommunicationsExportClaimV1,
        rejection_code: u16,
        completed_at_unix_seconds: i64,
    ) -> Result<(), CommunicationsExportPersistenceErrorV1> {
        if !valid_claim(claim)
            || !(1..=16).contains(&rejection_code)
            || !valid_timestamp(completed_at_unix_seconds)
        {
            return Err(CommunicationsExportPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.communications_export_jobs
             SET state = $2,
                 rejection_code = $3,
                 updated_at_unix_seconds = $4,
                 claimed_by = NULL,
                 lease_expires_at_unix_seconds = NULL
             WHERE export_id = $1 AND state = $5 AND claimed_by = $6
               AND logical_owner_id = $7",
        )
        .bind(claim.export_id.as_slice())
        .bind(STATE_REJECTED)
        .bind(
            i16::try_from(rejection_code)
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidInput)?,
        )
        .bind(completed_at_unix_seconds)
        .bind(STATE_MATERIALIZING)
        .bind(&claim.worker_id)
        .bind(&claim.logical_owner_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        if updated.rows_affected() != 1 {
            return Err(CommunicationsExportPersistenceErrorV1::ClaimLost);
        }
        insert_realtime_transition(
            &mut transaction,
            &claim.logical_owner_id,
            &claim.export_id,
            completed_at_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)
    }

    pub async fn job_status(
        &self,
        logical_owner_id: &str,
        export_id: [u8; 16],
    ) -> Result<Option<CommunicationsExportJobStatusV1>, CommunicationsExportPersistenceErrorV1>
    {
        if !valid_id16(&export_id) || !valid_logical_owner_id(logical_owner_id) {
            return Err(CommunicationsExportPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT state, requested_items, completed_items,
                    artifact_reference_id, artifact_declared_bytes,
                    artifact_sha256, rejection_code
             FROM makosh_data.communications_export_jobs
             WHERE export_id = $1 AND logical_owner_id = $2",
        )
        .bind(export_id.as_slice())
        .bind(logical_owner_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        row.map(|row| status_from_row(export_id, row)).transpose()
    }

    pub async fn pending_outbox(
        &self,
        limit: u32,
    ) -> Result<Vec<OutboxRecordV1>, CommunicationsExportPersistenceErrorV1> {
        if limit == 0 || limit > 256 {
            return Err(CommunicationsExportPersistenceErrorV1::InvalidInput);
        }
        let rows = sqlx::query(
            "SELECT exact_envelope_bytes
             FROM makosh_data.communications_export_outbox
             WHERE published_at_unix_seconds IS NULL
             ORDER BY created_at_unix_seconds, message_id
             LIMIT $1",
        )
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        rows.into_iter()
            .map(|row| {
                OutboxRecordV1::accept(
                    row.try_get::<Vec<u8>, _>("exact_envelope_bytes")
                        .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
                )
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)
            })
            .collect()
    }

    pub async fn mark_outbox_published(
        &self,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<(), CommunicationsExportPersistenceErrorV1> {
        if !valid_id16(&message_id) || !valid_timestamp(published_at_unix_seconds) {
            return Err(CommunicationsExportPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "UPDATE makosh_data.communications_export_outbox
             SET published_at_unix_seconds = $2
             WHERE message_id = $1 AND published_at_unix_seconds IS NULL",
        )
        .bind(message_id.as_slice())
        .bind(published_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)
    }
}

async fn insert_exact_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    outbox: &OutboxRecordV1,
    created_at_unix_seconds: i64,
) -> Result<(), CommunicationsExportPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.communications_export_outbox (
            message_id, envelope_sha256, exact_envelope_bytes,
            created_at_unix_seconds, published_at_unix_seconds
         ) VALUES ($1, $2, $3, $4, NULL)
         ON CONFLICT (message_id) DO NOTHING",
    )
    .bind(outbox.message_id().as_slice())
    .bind(outbox.envelope_sha256().as_slice())
    .bind(outbox.exact_bytes())
    .bind(created_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
    if inserted.rows_affected() == 1 {
        return Ok(());
    }
    let existing: Option<(Vec<u8>, Vec<u8>)> = sqlx::query_as(
        "SELECT envelope_sha256, exact_envelope_bytes
         FROM makosh_data.communications_export_outbox
         WHERE message_id = $1",
    )
    .bind(outbox.message_id().as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
    if existing.as_ref().is_some_and(|(sha256, exact_bytes)| {
        sha256.as_slice() == outbox.envelope_sha256()
            && exact_bytes.as_slice() == outbox.exact_bytes()
    }) {
        Ok(())
    } else {
        Err(CommunicationsExportPersistenceErrorV1::Conflict)
    }
}

fn prepared_item_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<CommunicationsExportPreparedItemV1, CommunicationsExportPersistenceErrorV1> {
    let body_state: i16 = row
        .try_get("body_state")
        .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?;
    let body_source = match body_state {
        BODY_ADMITTED_UTF8 => Some(CommunicationsExportSourceReceiptV1 {
            reference_id: id16(
                &row.try_get::<Vec<u8>, _>("source_reference_id")
                    .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
            )?,
            declared_bytes: u64::try_from(
                row.try_get::<i64, _>("source_declared_bytes")
                    .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
            )
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
            sha256: id32(
                &row.try_get::<Vec<u8>, _>("source_sha256")
                    .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
            )?,
            custody_transfer_source_proof: row
                .try_get("source_custody_proof")
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        }),
        BODY_UNAVAILABLE => None,
        _ => return Err(CommunicationsExportPersistenceErrorV1::InvalidRow),
    };
    Ok(CommunicationsExportPreparedItemV1 {
        message_id: id16(
            &row.try_get::<Vec<u8>, _>("message_id")
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        )?,
        conversation_id: id16(
            &row.try_get::<Vec<u8>, _>("conversation_id")
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        )?,
        evidence_id: id16(
            &row.try_get::<Vec<u8>, _>("evidence_id")
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        )?,
        evidence_revision: u64::try_from(
            row.try_get::<i64, _>("evidence_revision")
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        )
        .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        direction: direction_from_code(
            row.try_get("direction")
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        )?,
        occurred_at_unix_seconds: row
            .try_get("occurred_at_unix_seconds")
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        observed_at_unix_seconds: row
            .try_get("observed_at_unix_seconds")
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        participant_display_label: row
            .try_get("participant_display_label")
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        body_source,
    })
}

fn status_from_row(
    export_id: [u8; 16],
    row: sqlx::postgres::PgRow,
) -> Result<CommunicationsExportJobStatusV1, CommunicationsExportPersistenceErrorV1> {
    let state: i16 = row
        .try_get("state")
        .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?;
    let reference_id: Option<Vec<u8>> = row
        .try_get("artifact_reference_id")
        .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?;
    let declared_bytes: Option<i64> = row
        .try_get("artifact_declared_bytes")
        .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?;
    let sha256: Option<Vec<u8>> = row
        .try_get("artifact_sha256")
        .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?;
    let artifact = match (reference_id, declared_bytes, sha256) {
        (Some(reference_id), Some(declared_bytes), Some(sha256)) => {
            Some(CommunicationsExportArtifactReceiptV1 {
                reference_id: id16(&reference_id)?,
                declared_bytes: u64::try_from(declared_bytes)
                    .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
                sha256: id32(&sha256)?,
            })
        }
        (None, None, None) => None,
        _ => return Err(CommunicationsExportPersistenceErrorV1::InvalidRow),
    };
    Ok(CommunicationsExportJobStatusV1 {
        export_id,
        state: u8::try_from(state)
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        requested_items: u32::try_from(
            row.try_get::<i32, _>("requested_items")
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        )
        .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        completed_items: u32::try_from(
            row.try_get::<i32, _>("completed_items")
                .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        )
        .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        artifact,
        rejection_code: row
            .try_get::<Option<i16>, _>("rejection_code")
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?
            .map(u16::try_from)
            .transpose()
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
    })
}

fn valid_prepared_item(item: &CommunicationsExportPreparedItemV1) -> bool {
    valid_id16(&item.message_id)
        && valid_id16(&item.conversation_id)
        && valid_id16(&item.evidence_id)
        && item.evidence_revision > 0
        && valid_timestamp(item.occurred_at_unix_seconds)
        && valid_timestamp(item.observed_at_unix_seconds)
        && item.participant_display_label.as_ref().is_none_or(|label| {
            !label.is_empty() && label.len() <= 512 && !label.chars().any(char::is_control)
        })
        && item.body_source.as_ref().is_none_or(valid_source_receipt)
}

fn valid_source_receipt(receipt: &CommunicationsExportSourceReceiptV1) -> bool {
    valid_id16(&receipt.reference_id)
        && (1..=MAX_EXPORT_SOURCE_BYTES_V1 as u64).contains(&receipt.declared_bytes)
        && valid_sha256(&receipt.sha256)
        && (1..=2_048).contains(&receipt.custody_transfer_source_proof.len())
}

fn valid_artifact(receipt: &CommunicationsExportArtifactReceiptV1) -> bool {
    valid_id16(&receipt.reference_id)
        && (1..=MAX_EXPORT_ARTIFACT_BYTES_V1 as u64).contains(&receipt.declared_bytes)
        && valid_sha256(&receipt.sha256)
}

fn valid_claim(claim: &CommunicationsExportClaimV1) -> bool {
    valid_id16(&claim.export_id)
        && valid_logical_owner_id(&claim.logical_owner_id)
        && valid_id16(&claim.source_result_message_id)
        && valid_sha256(&claim.source_result_envelope_sha256)
        && valid_timestamp(claim.created_at_unix_seconds)
        && !claim.items.is_empty()
        && claim.items.len() <= 64
        && valid_worker_id(&claim.worker_id)
        && claim.items.iter().all(valid_prepared_item)
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn valid_worker_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.is_ascii()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

const fn direction_code(value: EvidenceExportDirectionV1) -> i16 {
    match value {
        EvidenceExportDirectionV1::Incoming => 1,
        EvidenceExportDirectionV1::Outgoing => 2,
        EvidenceExportDirectionV1::Unknown => 3,
    }
}

fn direction_from_code(
    value: i16,
) -> Result<EvidenceExportDirectionV1, CommunicationsExportPersistenceErrorV1> {
    match value {
        1 => Ok(EvidenceExportDirectionV1::Incoming),
        2 => Ok(EvidenceExportDirectionV1::Outgoing),
        3 => Ok(EvidenceExportDirectionV1::Unknown),
        _ => Err(CommunicationsExportPersistenceErrorV1::InvalidRow),
    }
}

fn id16(bytes: &[u8]) -> Result<[u8; 16], CommunicationsExportPersistenceErrorV1> {
    bytes
        .try_into()
        .ok()
        .filter(valid_id16)
        .ok_or(CommunicationsExportPersistenceErrorV1::InvalidRow)
}

fn id32(bytes: &[u8]) -> Result<[u8; 32], CommunicationsExportPersistenceErrorV1> {
    bytes
        .try_into()
        .ok()
        .filter(valid_sha256)
        .ok_or(CommunicationsExportPersistenceErrorV1::InvalidRow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_id_is_bounded_and_owner_neutral() {
        assert!(valid_worker_id("export-worker_1"));
        assert!(!valid_worker_id(""));
        assert!(!valid_worker_id("mail/account@example.test"));
    }

    #[test]
    fn prepared_item_contains_no_provider_or_blob_locator() {
        let item = CommunicationsExportPreparedItemV1 {
            message_id: [1; 16],
            conversation_id: [2; 16],
            evidence_id: [3; 16],
            evidence_revision: 1,
            direction: EvidenceExportDirectionV1::Incoming,
            occurred_at_unix_seconds: 1_700_000_000,
            observed_at_unix_seconds: 1_700_000_001,
            participant_display_label: Some("Alice".to_owned()),
            body_source: None,
        };
        assert!(valid_prepared_item(&item));
    }
}
