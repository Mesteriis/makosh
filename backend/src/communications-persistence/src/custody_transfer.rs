//! Owner-local custody work queue. Source receipts remain private and only a
//! successful Blob Platform transfer may create a canonical target reference.

use makosh_communications_api::{
    CommunicationBodyBlobReferenceV1, CommunicationConversationIdV1, CommunicationMessageIdV1,
    CommunicationObservationIdV1,
};
use sqlx::Row;

use crate::{
    CommunicationsDerivedIndexJobOperationV1, CommunicationsDerivedIndexJobV1,
    CommunicationsDurablePersistence, communications_derived_index_job_id_v1,
};

const SEARCH_PROJECTION_REVISION_V1: u32 = 1;
const MAX_INDEX_BYTES: u64 = 256 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimedCommunicationsBodyCustodyTransferV1 {
    pub evidence_id: CommunicationObservationIdV1,
    pub envelope_sha256: [u8; 32],
    pub source_reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub plaintext_sha256: [u8; 32],
    pub source_custody_proof: Vec<u8>,
    pub worker_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsBodyCustodyTransferErrorV1 {
    StorageUnavailable,
    InvalidRow,
    ClaimLost,
}

impl CommunicationsDurablePersistence {
    pub async fn reuse_completed_body_custody_transfer(
        &self,
        claimed: &ClaimedCommunicationsBodyCustodyTransferV1,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, CommunicationsBodyCustodyTransferErrorV1> {
        let row = sqlx::query(
            "SELECT evidence.body_blob_ref, evidence.body_blob_reference_id, evidence.body_blob_declared_bytes, evidence.body_blob_sha256 FROM makosh_data.communications_body_custody_transfers AS source JOIN makosh_data.communications_body_custody_transfer_lifecycle AS lifecycle ON lifecycle.evidence_id = source.evidence_id JOIN makosh_data.communications_evidence_summaries AS evidence ON evidence.observation_id = source.evidence_id WHERE lifecycle.state = 2 AND evidence.body_state = 4 AND source.source_reference_id = $1 AND source.declared_bytes = $2 AND source.plaintext_sha256 = $3 ORDER BY lifecycle.completed_at_unix_seconds DESC NULLS LAST, source.evidence_id DESC LIMIT 1",
        )
        .bind(claimed.source_reference_id.as_slice())
        .bind(i64::try_from(claimed.declared_bytes).map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?)
        .bind(claimed.plaintext_sha256.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        let Some(row) = row else { return Ok(false) };
        let blob_ref: String = row
            .try_get("body_blob_ref")
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?;
        let reference_id: Vec<u8> = row
            .try_get("body_blob_reference_id")
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?;
        let declared_bytes: i64 = row
            .try_get("body_blob_declared_bytes")
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?;
        let sha256: Vec<u8> = row
            .try_get("body_blob_sha256")
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?;
        self.complete_body_custody_transfer(
            claimed,
            CommunicationBodyBlobReferenceV1 {
                blob_ref,
                reference_id: reference_id
                    .try_into()
                    .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?,
                declared_bytes: u64::try_from(declared_bytes)
                    .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?,
                sha256: sha256
                    .try_into()
                    .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?,
            },
            completed_at_unix_seconds,
        )
        .await?;
        Ok(true)
    }

    pub async fn claim_next_body_custody_transfer(
        &self,
        worker_id: &str,
        now_unix_seconds: i64,
        lease_expires_at_unix_seconds: i64,
    ) -> Result<
        Option<ClaimedCommunicationsBodyCustodyTransferV1>,
        CommunicationsBodyCustodyTransferErrorV1,
    > {
        if worker_id.is_empty()
            || worker_id.len() > 256
            || lease_expires_at_unix_seconds <= now_unix_seconds
        {
            return Err(CommunicationsBodyCustodyTransferErrorV1::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        let row = sqlx::query(
            "SELECT transfer.evidence_id, transfer.envelope_sha256, transfer.source_reference_id, transfer.declared_bytes, transfer.plaintext_sha256, transfer.source_custody_proof FROM makosh_data.communications_body_custody_transfer_lifecycle AS lifecycle JOIN makosh_data.communications_body_custody_transfers AS transfer ON transfer.evidence_id = lifecycle.evidence_id JOIN makosh_data.communications_evidence_summaries AS evidence ON evidence.observation_id = lifecycle.evidence_id WHERE lifecycle.state = 1 AND (lifecycle.lease_expires_at_unix_seconds IS NULL OR lifecycle.lease_expires_at_unix_seconds <= $1) ORDER BY evidence.observed_at_unix_seconds DESC, lifecycle.evidence_id DESC LIMIT 1 FOR UPDATE OF lifecycle SKIP LOCKED",
        )
        .bind(now_unix_seconds)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        let Some(row) = row else {
            transaction
                .commit()
                .await
                .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
            return Ok(None);
        };
        let claimed = claimed_from_row(row, worker_id)?;
        let lease = sqlx::query(
            "UPDATE makosh_data.communications_body_custody_transfer_lifecycle SET claimed_by = $2, lease_expires_at_unix_seconds = $3 WHERE evidence_id = $1 AND state = 1",
        )
        .bind(claimed.evidence_id.bytes().as_slice())
        .bind(worker_id)
        .bind(lease_expires_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        if lease.rows_affected() != 1 {
            return Err(CommunicationsBodyCustodyTransferErrorV1::ClaimLost);
        }
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        Ok(Some(claimed))
    }

    pub async fn complete_body_custody_transfer(
        &self,
        claimed: &ClaimedCommunicationsBodyCustodyTransferV1,
        target: CommunicationBodyBlobReferenceV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), CommunicationsBodyCustodyTransferErrorV1> {
        if target.declared_bytes != claimed.declared_bytes
            || target.sha256 != claimed.plaintext_sha256
        {
            return Err(CommunicationsBodyCustodyTransferErrorV1::InvalidRow);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        let evidence = sqlx::query(
            "SELECT message.message_id, message.conversation_id, evidence.observed_at_unix_seconds FROM makosh_data.communications_evidence_summaries AS evidence LEFT JOIN makosh_data.communications_messages AS message ON message.last_evidence_id = evidence.observation_id WHERE evidence.observation_id = $1 AND evidence.body_state = 2",
        )
        .bind(claimed.evidence_id.bytes().as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?
        .ok_or(CommunicationsBodyCustodyTransferErrorV1::ClaimLost)?;
        let message_id = evidence
            .try_get::<Option<Vec<u8>>, _>("message_id")
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?
            .map(|value| id16(&value))
            .transpose()?;
        let conversation_id = evidence
            .try_get::<Option<Vec<u8>>, _>("conversation_id")
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?
            .map(|value| id16(&value))
            .transpose()?;
        let observed_at_unix_seconds: i64 = evidence
            .try_get("observed_at_unix_seconds")
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?;
        let settled = sqlx::query(
            "UPDATE makosh_data.communications_body_custody_transfer_lifecycle SET state = 2, completed_at_unix_seconds = $3, claimed_by = NULL, lease_expires_at_unix_seconds = NULL WHERE evidence_id = $1 AND state = 1 AND claimed_by = $2",
        )
        .bind(claimed.evidence_id.bytes().as_slice())
        .bind(&claimed.worker_id)
        .bind(completed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        if settled.rows_affected() != 1 {
            return Err(CommunicationsBodyCustodyTransferErrorV1::ClaimLost);
        }
        let summary = sqlx::query(
            "UPDATE makosh_data.communications_evidence_summaries SET body_state = 4, body_blob_ref = $2, body_blob_reference_id = $3, body_blob_declared_bytes = $4, body_blob_sha256 = $5 WHERE observation_id = $1 AND body_state = 2",
        )
        .bind(claimed.evidence_id.bytes().as_slice())
        .bind(&target.blob_ref)
        .bind(target.reference_id.as_slice())
        .bind(i64::try_from(target.declared_bytes).map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?)
        .bind(target.sha256.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        if summary.rows_affected() != 1 {
            return Err(CommunicationsBodyCustodyTransferErrorV1::ClaimLost);
        }
        if message_id.is_some() != conversation_id.is_some() {
            return Err(CommunicationsBodyCustodyTransferErrorV1::InvalidRow);
        }
        if let Some(message_id) = message_id {
            let message = sqlx::query(
                "UPDATE makosh_data.communications_messages SET body_state = 3, canonical_body_state = 4, canonical_revision = canonical_revision + 1 WHERE message_id = $1 AND last_evidence_id = $2",
            )
            .bind(message_id.as_slice())
            .bind(claimed.evidence_id.bytes().as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
            if message.rows_affected() != 1 {
                return Err(CommunicationsBodyCustodyTransferErrorV1::ClaimLost);
            }
        }
        if let (Some(message_id), Some(conversation_id)) = (message_id, conversation_id)
            && target.declared_bytes <= MAX_INDEX_BYTES
        {
            let job = CommunicationsDerivedIndexJobV1 {
                job_id: communications_derived_index_job_id_v1(
                    claimed.evidence_id.bytes(),
                    message_id,
                    SEARCH_PROJECTION_REVISION_V1,
                ),
                operation: CommunicationsDerivedIndexJobOperationV1::Index,
                evidence_id: claimed.evidence_id,
                message_id: CommunicationMessageIdV1::new(message_id),
                conversation_id: Some(CommunicationConversationIdV1::new(conversation_id)),
                blob: Some(target),
                projection_revision: SEARCH_PROJECTION_REVISION_V1,
                observed_at_unix_seconds,
                created_at_unix_seconds: completed_at_unix_seconds,
            };
            enqueue_index_job(&mut transaction, &job).await?;
        }
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)
    }

    pub async fn fail_body_custody_transfer(
        &self,
        claimed: &ClaimedCommunicationsBodyCustodyTransferV1,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, CommunicationsBodyCustodyTransferErrorV1> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        let result = sqlx::query(
            "UPDATE makosh_data.communications_body_custody_transfer_lifecycle SET state = 3, completed_at_unix_seconds = $3, claimed_by = NULL, lease_expires_at_unix_seconds = NULL WHERE evidence_id = $1 AND state = 1 AND claimed_by = $2",
        )
        .bind(claimed.evidence_id.bytes().as_slice())
        .bind(&claimed.worker_id)
        .bind(completed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        if result.rows_affected() != 1 {
            return Ok(false);
        }
        let summary = sqlx::query(
            "UPDATE makosh_data.communications_evidence_summaries SET body_state = 3, body_admission_failure = 4 WHERE observation_id = $1 AND body_state = 2",
        )
        .bind(claimed.evidence_id.bytes().as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        if summary.rows_affected() != 1 {
            return Err(CommunicationsBodyCustodyTransferErrorV1::ClaimLost);
        }
        sqlx::query(
            "UPDATE makosh_data.communications_messages SET body_state = 3, canonical_body_state = 3, canonical_revision = canonical_revision + 1 WHERE last_evidence_id = $1 AND canonical_body_state = 2",
        )
        .bind(claimed.evidence_id.bytes().as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        // A superseded evidence revision no longer owns the message projection.
        // Its own summary and lifecycle still have to settle terminally so it
        // cannot block newer custody work forever.
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        Ok(true)
    }

    pub async fn release_body_custody_transfer(
        &self,
        claimed: &ClaimedCommunicationsBodyCustodyTransferV1,
    ) -> Result<bool, CommunicationsBodyCustodyTransferErrorV1> {
        let result = sqlx::query(
            "UPDATE makosh_data.communications_body_custody_transfer_lifecycle SET claimed_by = NULL, lease_expires_at_unix_seconds = NULL WHERE evidence_id = $1 AND state = 1 AND claimed_by = $2",
        )
        .bind(claimed.evidence_id.bytes().as_slice())
        .bind(&claimed.worker_id)
        .execute(&self.pool)
        .await
        .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
        Ok(result.rows_affected() == 1)
    }
}

async fn enqueue_index_job(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    job: &CommunicationsDerivedIndexJobV1,
) -> Result<(), CommunicationsBodyCustodyTransferErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.communications_derived_index_jobs (job_id, operation, evidence_id, message_id, conversation_id, blob_ref, blob_reference_id, blob_declared_bytes, blob_sha256, projection_revision, observed_at_unix_seconds, created_at_unix_seconds) VALUES ($1, 1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) ON CONFLICT (job_id) DO UPDATE SET operation = EXCLUDED.operation, evidence_id = EXCLUDED.evidence_id, message_id = EXCLUDED.message_id, conversation_id = EXCLUDED.conversation_id, blob_ref = EXCLUDED.blob_ref, blob_reference_id = EXCLUDED.blob_reference_id, blob_declared_bytes = EXCLUDED.blob_declared_bytes, blob_sha256 = EXCLUDED.blob_sha256, projection_revision = EXCLUDED.projection_revision, observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds, created_at_unix_seconds = EXCLUDED.created_at_unix_seconds, completed_at_unix_seconds = NULL, outcome = NULL, failure_code = NULL, claimed_by = NULL, lease_expires_at_unix_seconds = NULL WHERE makosh_data.communications_derived_index_jobs.completed_at_unix_seconds IS NOT NULL",
    )
    .bind(job.job_id.as_slice())
    .bind(job.evidence_id.bytes().as_slice())
    .bind(job.message_id.bytes().as_slice())
    .bind(job.conversation_id.expect("index job has conversation").bytes().as_slice())
    .bind(job.blob.as_ref().expect("index job has blob").blob_ref.as_str())
    .bind(job.blob.as_ref().expect("index job has blob").reference_id.as_slice())
    .bind(i64::try_from(job.blob.as_ref().expect("index job has blob").declared_bytes).map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?)
    .bind(job.blob.as_ref().expect("index job has blob").sha256.as_slice())
    .bind(i32::try_from(job.projection_revision).map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?)
    .bind(job.observed_at_unix_seconds)
    .bind(job.created_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::StorageUnavailable)?;
    Ok(())
}

fn claimed_from_row(
    row: sqlx::postgres::PgRow,
    worker_id: &str,
) -> Result<ClaimedCommunicationsBodyCustodyTransferV1, CommunicationsBodyCustodyTransferErrorV1> {
    Ok(ClaimedCommunicationsBodyCustodyTransferV1 {
        evidence_id: CommunicationObservationIdV1::new(id16(
            &row.try_get::<Vec<u8>, _>("evidence_id")
                .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?,
        )?),
        envelope_sha256: id32(
            &row.try_get::<Vec<u8>, _>("envelope_sha256")
                .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?,
        )?,
        source_reference_id: id16(
            &row.try_get::<Vec<u8>, _>("source_reference_id")
                .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?,
        )?,
        declared_bytes: u64::try_from(
            row.try_get::<i64, _>("declared_bytes")
                .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?,
        )
        .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?,
        plaintext_sha256: id32(
            &row.try_get::<Vec<u8>, _>("plaintext_sha256")
                .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?,
        )?,
        source_custody_proof: row
            .try_get("source_custody_proof")
            .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)?,
        worker_id: worker_id.to_owned(),
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsBodyCustodyTransferErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)
}

fn id32(value: &[u8]) -> Result<[u8; 32], CommunicationsBodyCustodyTransferErrorV1> {
    value
        .try_into()
        .map_err(|_| CommunicationsBodyCustodyTransferErrorV1::InvalidRow)
}
