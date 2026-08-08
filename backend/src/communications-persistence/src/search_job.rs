//! Typed owner-local derived-index job records; no content or key material crosses this seam.

use makosh_communications_api::{
    CommunicationBodyBlobReferenceV1, CommunicationConversationIdV1, CommunicationMessageIdV1,
    CommunicationObservationIdV1,
};
use sha2::{Digest, Sha256};
use sqlx::Row;

use crate::{CommunicationsDurablePersistence, CommunicationsPersistenceError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsDerivedIndexJobOperationV1 {
    Index,
    Remove,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsDerivedIndexJobV1 {
    pub job_id: [u8; 16],
    pub operation: CommunicationsDerivedIndexJobOperationV1,
    pub evidence_id: CommunicationObservationIdV1,
    pub message_id: CommunicationMessageIdV1,
    pub conversation_id: Option<CommunicationConversationIdV1>,
    pub blob: Option<CommunicationBodyBlobReferenceV1>,
    pub projection_revision: u32,
    pub observed_at_unix_seconds: i64,
    pub created_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsDerivedIndexJobErrorV1 {
    InvalidShape,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsDerivedIndexFailureV1 {
    BlobDenied,
    BlobUnavailable,
    InvalidContent,
    VaultDenied,
    VaultUnavailable,
    DocumentLimit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsDerivedIndexFailureRecordV1 {
    pub evidence_id: CommunicationObservationIdV1,
    pub message_id: CommunicationMessageIdV1,
    pub projection_revision: u32,
    pub observed_at_unix_seconds: i64,
    pub failure: CommunicationsDerivedIndexFailureV1,
    pub recorded_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimedCommunicationsDerivedIndexJobV1 {
    pub job: CommunicationsDerivedIndexJobV1,
    pub worker_id: String,
}

impl CommunicationsDerivedIndexJobV1 {
    pub fn validate(&self) -> Result<(), CommunicationsDerivedIndexJobErrorV1> {
        if self.job_id.iter().all(|byte| *byte == 0) || self.projection_revision == 0 {
            return Err(CommunicationsDerivedIndexJobErrorV1::InvalidShape);
        }
        match self.operation {
            CommunicationsDerivedIndexJobOperationV1::Index
                if self.conversation_id.is_some()
                    && self
                        .blob
                        .as_ref()
                        .is_some_and(|blob| (1..=256 * 1024).contains(&blob.declared_bytes)) =>
            {
                Ok(())
            }
            CommunicationsDerivedIndexJobOperationV1::Remove
                if self.conversation_id.is_none() && self.blob.is_none() =>
            {
                Ok(())
            }
            _ => Err(CommunicationsDerivedIndexJobErrorV1::InvalidShape),
        }
    }
}

impl CommunicationsDurablePersistence {
    pub async fn reconcile_search_projection_jobs(
        &self,
        projection_revision: u32,
        created_at_unix_seconds: i64,
    ) -> Result<usize, CommunicationsPersistenceError> {
        if projection_revision == 0 {
            return Err(CommunicationsPersistenceError::InvalidRow);
        }
        let revision = i32::try_from(projection_revision)
            .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
        let rows = sqlx::query(
            "SELECT message.message_id, message.conversation_id, message.lifecycle_state, evidence.observation_id AS evidence_id, evidence.body_state, evidence.body_blob_ref AS blob_ref, evidence.body_blob_reference_id AS blob_reference_id, evidence.body_blob_declared_bytes AS blob_declared_bytes, evidence.body_blob_sha256 AS blob_sha256, evidence.observed_at_unix_seconds, projection.projection_revision AS indexed_revision, projection.observed_at_unix_seconds AS indexed_observed_at, tombstone.projection_revision AS tombstone_revision, tombstone.observed_at_unix_seconds AS tombstone_observed_at, failure.evidence_id AS failure_evidence_id FROM makosh_data.communications_messages AS message JOIN makosh_data.communications_evidence_summaries AS evidence ON evidence.observation_id = message.last_evidence_id LEFT JOIN makosh_data.communications_derived_index_projections AS projection ON projection.message_id = message.message_id LEFT JOIN makosh_data.communications_derived_index_tombstones AS tombstone ON tombstone.message_id = message.message_id LEFT JOIN makosh_data.communications_derived_index_failures AS failure ON failure.evidence_id = evidence.observation_id WHERE (message.lifecycle_state = 1 AND evidence.body_state = 4 AND ((evidence.body_blob_declared_bytes > 262144 AND failure.evidence_id IS NULL) OR (evidence.body_blob_declared_bytes <= 262144 AND (projection.message_id IS NULL OR projection.projection_revision < $1 OR (projection.projection_revision = $1 AND projection.observed_at_unix_seconds < evidence.observed_at_unix_seconds))))) OR ((message.lifecycle_state <> 1 OR evidence.body_state <> 4) AND (tombstone.message_id IS NULL OR tombstone.projection_revision < $1 OR (tombstone.projection_revision = $1 AND tombstone.observed_at_unix_seconds < evidence.observed_at_unix_seconds))) ORDER BY message.message_id ASC LIMIT 64",
        )
        .bind(revision)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        let mut reconciled = 0_usize;
        for row in rows {
            let message_id = CommunicationMessageIdV1::new(id16(
                &row.try_get::<Vec<u8>, _>("message_id")
                    .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
            )?);
            let conversation_id = CommunicationConversationIdV1::new(id16(
                &row.try_get::<Vec<u8>, _>("conversation_id")
                    .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
            )?);
            let evidence_id = CommunicationObservationIdV1::new(id16(
                &row.try_get::<Vec<u8>, _>("evidence_id")
                    .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
            )?);
            let observed_at_unix_seconds: i64 = row
                .try_get("observed_at_unix_seconds")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let lifecycle_state: i16 = row
                .try_get("lifecycle_state")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let body_state: i16 = row
                .try_get("body_state")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let indexed_revision: Option<i32> = row
                .try_get("indexed_revision")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let indexed_observed_at: Option<i64> = row
                .try_get("indexed_observed_at")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let tombstone_revision: Option<i32> = row
                .try_get("tombstone_revision")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let tombstone_observed_at: Option<i64> = row
                .try_get("tombstone_observed_at")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let failure_exists: Option<Vec<u8>> = row
                .try_get("failure_evidence_id")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
            let current_projection = indexed_revision.zip(indexed_observed_at);
            let current_tombstone = tombstone_revision.zip(tombstone_observed_at);
            if lifecycle_state == 1 && body_state == 4 {
                let blob =
                    blob_from_row(&row)?.ok_or(CommunicationsPersistenceError::InvalidRow)?;
                if blob.declared_bytes > 256 * 1024 {
                    if failure_exists.is_none() {
                        let result = sqlx::query("INSERT INTO makosh_data.communications_derived_index_failures (evidence_id, message_id, projection_revision, observed_at_unix_seconds, failure_code, recorded_at_unix_seconds) VALUES ($1, $2, $3, $4, 6, $5) ON CONFLICT (evidence_id) DO NOTHING")
                            .bind(evidence_id.bytes().as_slice())
                            .bind(message_id.bytes().as_slice())
                            .bind(revision)
                            .bind(observed_at_unix_seconds)
                            .bind(created_at_unix_seconds)
                            .execute(&mut *transaction)
                            .await
                            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
                        reconciled += usize::try_from(result.rows_affected())
                            .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
                    }
                    if is_stale(
                        current_tombstone,
                        projection_revision,
                        observed_at_unix_seconds,
                    ) {
                        let job = CommunicationsDerivedIndexJobV1 {
                            job_id: communications_derived_index_job_id_v1(
                                evidence_id.bytes(),
                                message_id.bytes(),
                                projection_revision,
                            ),
                            operation: CommunicationsDerivedIndexJobOperationV1::Remove,
                            evidence_id,
                            message_id,
                            conversation_id: None,
                            blob: None,
                            projection_revision,
                            observed_at_unix_seconds,
                            created_at_unix_seconds,
                        };
                        reconciled += enqueue_reconciled_job(&mut transaction, &job).await?;
                    }
                } else if is_stale(
                    current_projection,
                    projection_revision,
                    observed_at_unix_seconds,
                ) {
                    let job = CommunicationsDerivedIndexJobV1 {
                        job_id: communications_derived_index_job_id_v1(
                            evidence_id.bytes(),
                            message_id.bytes(),
                            projection_revision,
                        ),
                        operation: CommunicationsDerivedIndexJobOperationV1::Index,
                        evidence_id,
                        message_id,
                        conversation_id: Some(conversation_id),
                        blob: Some(blob),
                        projection_revision,
                        observed_at_unix_seconds,
                        created_at_unix_seconds,
                    };
                    reconciled += enqueue_reconciled_job(&mut transaction, &job).await?;
                }
            } else if is_stale(
                current_tombstone,
                projection_revision,
                observed_at_unix_seconds,
            ) {
                let job = CommunicationsDerivedIndexJobV1 {
                    job_id: communications_derived_index_job_id_v1(
                        evidence_id.bytes(),
                        message_id.bytes(),
                        projection_revision,
                    ),
                    operation: CommunicationsDerivedIndexJobOperationV1::Remove,
                    evidence_id,
                    message_id,
                    conversation_id: None,
                    blob: None,
                    projection_revision,
                    observed_at_unix_seconds,
                    created_at_unix_seconds,
                };
                reconciled += enqueue_reconciled_job(&mut transaction, &job).await?;
            }
        }
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        Ok(reconciled)
    }

    pub async fn claim_next_derived_index_job(
        &self,
        worker_id: &str,
        claimed_at_unix_seconds: i64,
        lease_expires_at_unix_seconds: i64,
    ) -> Result<Option<ClaimedCommunicationsDerivedIndexJobV1>, CommunicationsPersistenceError>
    {
        if worker_id.is_empty()
            || worker_id.len() > 256
            || !worker_id.is_ascii()
            || lease_expires_at_unix_seconds <= claimed_at_unix_seconds
        {
            return Err(CommunicationsPersistenceError::InvalidRow);
        }
        let row = sqlx::query(
            "WITH candidate AS (SELECT job_id FROM makosh_data.communications_derived_index_jobs WHERE completed_at_unix_seconds IS NULL AND (lease_expires_at_unix_seconds IS NULL OR lease_expires_at_unix_seconds <= $2) ORDER BY created_at_unix_seconds ASC, job_id ASC LIMIT 1 FOR UPDATE SKIP LOCKED) UPDATE makosh_data.communications_derived_index_jobs AS job SET claimed_by = $1, lease_expires_at_unix_seconds = $3, attempt_count = job.attempt_count + 1 FROM candidate WHERE job.job_id = candidate.job_id RETURNING job.job_id, job.operation, job.evidence_id, job.message_id, job.conversation_id, job.blob_ref, job.blob_reference_id, job.blob_declared_bytes, job.blob_sha256, job.projection_revision, job.observed_at_unix_seconds, job.created_at_unix_seconds",
        )
        .bind(worker_id)
        .bind(claimed_at_unix_seconds)
        .bind(lease_expires_at_unix_seconds)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        row.map(|row| claimed_job_from_row(row, worker_id))
            .transpose()
    }

    pub async fn complete_derived_index_job(
        &self,
        job_id: [u8; 16],
        worker_id: &str,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, CommunicationsPersistenceError> {
        settle_derived_index_job(
            &self.pool,
            job_id,
            worker_id,
            1,
            None,
            completed_at_unix_seconds,
        )
        .await
    }

    pub async fn fail_derived_index_job(
        &self,
        job_id: [u8; 16],
        worker_id: &str,
        failure: CommunicationsDerivedIndexFailureV1,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, CommunicationsPersistenceError> {
        settle_derived_index_job(
            &self.pool,
            job_id,
            worker_id,
            2,
            Some(failure),
            completed_at_unix_seconds,
        )
        .await
    }

    pub async fn release_derived_index_job(
        &self,
        job_id: [u8; 16],
        worker_id: &str,
    ) -> Result<bool, CommunicationsPersistenceError> {
        let result = sqlx::query(
            "UPDATE makosh_data.communications_derived_index_jobs SET claimed_by = NULL, lease_expires_at_unix_seconds = NULL WHERE job_id = $1 AND claimed_by = $2 AND completed_at_unix_seconds IS NULL",
        )
        .bind(job_id.as_slice())
        .bind(worker_id)
        .execute(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
        Ok(result.rows_affected() == 1)
    }
}

pub fn communications_derived_index_job_id_v1(
    evidence_id: [u8; 16],
    message_id: [u8; 16],
    revision: u32,
) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communications.derived-index-job.v1\0");
    digest.update(evidence_id);
    digest.update(message_id);
    digest.update(revision.to_be_bytes());
    let value: [u8; 32] = digest.finalize().into();
    value[..16].try_into().expect("fixed SHA-256 prefix")
}

fn is_stale(current: Option<(i32, i64)>, revision: u32, observed_at_unix_seconds: i64) -> bool {
    match current {
        None => true,
        Some((current_revision, current_observed_at)) => {
            current_revision < i32::try_from(revision).expect("u32 projection revision fits i32")
                || (current_revision
                    == i32::try_from(revision).expect("u32 projection revision fits i32")
                    && current_observed_at < observed_at_unix_seconds)
        }
    }
}

async fn enqueue_reconciled_job(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    job: &CommunicationsDerivedIndexJobV1,
) -> Result<usize, CommunicationsPersistenceError> {
    job.validate()
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let result = sqlx::query("INSERT INTO makosh_data.communications_derived_index_jobs (job_id, operation, evidence_id, message_id, conversation_id, blob_ref, blob_reference_id, blob_declared_bytes, blob_sha256, projection_revision, observed_at_unix_seconds, created_at_unix_seconds) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) ON CONFLICT (job_id) DO UPDATE SET operation = EXCLUDED.operation, evidence_id = EXCLUDED.evidence_id, message_id = EXCLUDED.message_id, conversation_id = EXCLUDED.conversation_id, blob_ref = EXCLUDED.blob_ref, blob_reference_id = EXCLUDED.blob_reference_id, blob_declared_bytes = EXCLUDED.blob_declared_bytes, blob_sha256 = EXCLUDED.blob_sha256, projection_revision = EXCLUDED.projection_revision, observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds, created_at_unix_seconds = EXCLUDED.created_at_unix_seconds, completed_at_unix_seconds = NULL, outcome = NULL, failure_code = NULL, claimed_by = NULL, lease_expires_at_unix_seconds = NULL WHERE makosh_data.communications_derived_index_jobs.completed_at_unix_seconds IS NOT NULL AND makosh_data.communications_derived_index_jobs.outcome = 1")
        .bind(job.job_id.as_slice())
        .bind(match job.operation { CommunicationsDerivedIndexJobOperationV1::Index => 1_i16, CommunicationsDerivedIndexJobOperationV1::Remove => 2_i16 })
        .bind(job.evidence_id.bytes().as_slice())
        .bind(job.message_id.bytes().as_slice())
        .bind(job.conversation_id.map(|value| value.bytes().to_vec()))
        .bind(job.blob.as_ref().map(|value| value.blob_ref.as_str()))
        .bind(job.blob.as_ref().map(|value| value.reference_id.to_vec()))
        .bind(job.blob.as_ref().map(|value| i64::try_from(value.declared_bytes).expect("body byte limit fits i64")))
        .bind(job.blob.as_ref().map(|value| value.sha256.to_vec()))
        .bind(i32::try_from(job.projection_revision).map_err(|_| CommunicationsPersistenceError::InvalidRow)?)
        .bind(job.observed_at_unix_seconds)
        .bind(job.created_at_unix_seconds)
        .execute(&mut **transaction)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
    usize::try_from(result.rows_affected()).map_err(|_| CommunicationsPersistenceError::InvalidRow)
}

async fn settle_derived_index_job(
    pool: &sqlx::PgPool,
    job_id: [u8; 16],
    worker_id: &str,
    outcome: i16,
    failure: Option<CommunicationsDerivedIndexFailureV1>,
    completed_at_unix_seconds: i64,
) -> Result<bool, CommunicationsPersistenceError> {
    let result = sqlx::query(
        "UPDATE makosh_data.communications_derived_index_jobs SET completed_at_unix_seconds = $3, outcome = $4, failure_code = $5, claimed_by = NULL, lease_expires_at_unix_seconds = NULL WHERE job_id = $1 AND claimed_by = $2 AND completed_at_unix_seconds IS NULL",
    )
    .bind(job_id.as_slice())
    .bind(worker_id)
    .bind(completed_at_unix_seconds)
    .bind(outcome)
    .bind(failure.map(failure_value))
    .execute(pool)
    .await
    .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;
    Ok(result.rows_affected() == 1)
}

fn claimed_job_from_row(
    row: sqlx::postgres::PgRow,
    worker_id: &str,
) -> Result<ClaimedCommunicationsDerivedIndexJobV1, CommunicationsPersistenceError> {
    let operation = match row
        .try_get::<i16, _>("operation")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?
    {
        1 => CommunicationsDerivedIndexJobOperationV1::Index,
        2 => CommunicationsDerivedIndexJobOperationV1::Remove,
        _ => return Err(CommunicationsPersistenceError::InvalidRow),
    };
    let job = CommunicationsDerivedIndexJobV1 {
        job_id: id16(
            &row.try_get::<Vec<u8>, _>("job_id")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
        )?,
        operation,
        evidence_id: CommunicationObservationIdV1::new(id16(
            &row.try_get::<Vec<u8>, _>("evidence_id")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
        )?),
        message_id: CommunicationMessageIdV1::new(id16(
            &row.try_get::<Vec<u8>, _>("message_id")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
        )?),
        conversation_id: row
            .try_get::<Option<Vec<u8>>, _>("conversation_id")
            .map_err(|_| CommunicationsPersistenceError::InvalidRow)?
            .map(|value| id16(&value).map(CommunicationConversationIdV1::new))
            .transpose()?,
        blob: blob_from_row(&row)?,
        projection_revision: u32::try_from(
            row.try_get::<i32, _>("projection_revision")
                .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
        )
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
        observed_at_unix_seconds: row
            .try_get("observed_at_unix_seconds")
            .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
        created_at_unix_seconds: row
            .try_get("created_at_unix_seconds")
            .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
    };
    job.validate()
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    Ok(ClaimedCommunicationsDerivedIndexJobV1 {
        job,
        worker_id: worker_id.to_owned(),
    })
}

fn blob_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<Option<CommunicationBodyBlobReferenceV1>, CommunicationsPersistenceError> {
    let blob_ref: Option<String> = row
        .try_get("blob_ref")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let reference_id: Option<Vec<u8>> = row
        .try_get("blob_reference_id")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let declared_bytes: Option<i64> = row
        .try_get("blob_declared_bytes")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let sha256: Option<Vec<u8>> = row
        .try_get("blob_sha256")
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    match (blob_ref, reference_id, declared_bytes, sha256) {
        (None, None, None, None) => Ok(None),
        (Some(blob_ref), Some(reference_id), Some(declared_bytes), Some(sha256)) => {
            Ok(Some(CommunicationBodyBlobReferenceV1 {
                blob_ref,
                reference_id: id16(&reference_id)?,
                declared_bytes: u64::try_from(declared_bytes)
                    .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
                sha256: id32(&sha256)?,
            }))
        }
        _ => Err(CommunicationsPersistenceError::InvalidRow),
    }
}

fn failure_value(value: CommunicationsDerivedIndexFailureV1) -> i16 {
    match value {
        CommunicationsDerivedIndexFailureV1::BlobDenied => 1,
        CommunicationsDerivedIndexFailureV1::BlobUnavailable => 2,
        CommunicationsDerivedIndexFailureV1::InvalidContent => 3,
        CommunicationsDerivedIndexFailureV1::VaultDenied => 4,
        CommunicationsDerivedIndexFailureV1::VaultUnavailable => 5,
        CommunicationsDerivedIndexFailureV1::DocumentLimit => 6,
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsPersistenceError> {
    value
        .try_into()
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)
}

fn id32(value: &[u8]) -> Result<[u8; 32], CommunicationsPersistenceError> {
    value
        .try_into()
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index_job(declared_bytes: u64) -> CommunicationsDerivedIndexJobV1 {
        CommunicationsDerivedIndexJobV1 {
            job_id: [1; 16],
            operation: CommunicationsDerivedIndexJobOperationV1::Index,
            evidence_id: CommunicationObservationIdV1::new([2; 16]),
            message_id: CommunicationMessageIdV1::new([3; 16]),
            conversation_id: Some(CommunicationConversationIdV1::new([4; 16])),
            blob: Some(CommunicationBodyBlobReferenceV1 {
                blob_ref: "owner-local-blob".to_owned(),
                reference_id: [5; 16],
                declared_bytes,
                sha256: [6; 32],
            }),
            projection_revision: 1,
            observed_at_unix_seconds: 1,
            created_at_unix_seconds: 1,
        }
    }

    #[test]
    fn index_job_accepts_the_full_admitted_blob_range_only() {
        assert_eq!(index_job(256 * 1024).validate(), Ok(()));
        assert_eq!(
            index_job(256 * 1024 + 1).validate(),
            Err(CommunicationsDerivedIndexJobErrorV1::InvalidShape),
        );
    }

    #[test]
    fn reconciliation_only_requeues_missing_or_older_projection_state() {
        assert!(is_stale(None, 1, 10));
        assert!(is_stale(Some((1, 9)), 1, 10));
        assert!(is_stale(Some((1, 10)), 2, 1));
        assert!(!is_stale(Some((1, 10)), 1, 10));
        assert!(!is_stale(Some((2, 1)), 1, 10));
    }
}
