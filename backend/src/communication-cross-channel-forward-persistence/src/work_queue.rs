use makosh_communication_cross_channel_forward_core::CrossChannelForwardStateV1;
use sqlx::Row;

use crate::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardPersistenceErrorV1,
    operations::{
        STATE_ACCEPTED, STATE_DELIVERY_ACCEPTED, STATE_DISPATCHING, STATE_PREPARING_SOURCE,
        STATE_REJECTED, id16, optional_id16, positive_u64, state_from_code,
    },
    realtime::insert_forward_transition,
    valid_bounded_identity, valid_id16, valid_timestamp,
};

pub const FORWARD_WORK_LEASE_MILLIS_V1: i64 = 30_000;
const MAX_BODY_BYTES_V1: u32 = 64 * 1024;
const MAX_BLOB_REFERENCE_BYTES_V1: usize = 1_024;
const MAX_CUSTODY_PROOF_BYTES_V1: usize = 4_096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardWorkStageV1 {
    PreparingSource,
    Dispatching,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardPreparedSourceV1 {
    pub source_revision: u64,
    pub body_sha256: [u8; 32],
    pub body_length: u32,
    pub blob_reference: Vec<u8>,
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardClaimV1 {
    pub logical_owner_id: String,
    pub forward_id: [u8; 16],
    pub source_message_id: [u8; 16],
    pub target_conversation_id: [u8; 16],
    pub target_reply_to_message_id: Option<[u8; 16]>,
    pub stage: CrossChannelForwardWorkStageV1,
    pub prepared_source: Option<CrossChannelForwardPreparedSourceV1>,
    pub worker_id: String,
    pub claim_epoch: u64,
    pub attempt_count: u16,
    pub lease_expires_at_unix_millis: i64,
}

impl CommunicationCrossChannelForwardPersistenceV1 {
    pub async fn claim_next_forward(
        &self,
        logical_owner_id: &str,
        worker_id: &str,
        now_unix_millis: i64,
    ) -> Result<Option<CrossChannelForwardClaimV1>, CrossChannelForwardPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id)
            || !valid_bounded_identity(worker_id)
            || !valid_timestamp(now_unix_millis)
        {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let lease_expires_at_unix_millis = now_unix_millis
            .checked_add(FORWARD_WORK_LEASE_MILLIS_V1)
            .ok_or(CrossChannelForwardPersistenceErrorV1::InvalidInput)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        let row = sqlx::query(
            "WITH candidate AS (
               SELECT logical_owner_id, forward_id, state AS previous_state
               FROM makosh_data.communication_cross_channel_forward_operations
               WHERE logical_owner_id = $1
                 AND state BETWEEN 1 AND 3
                 AND next_attempt_at_unix_millis <= $2
                 AND (
                   claimed_by IS NULL
                   OR lease_expires_at_unix_millis < $2
                 )
               ORDER BY next_attempt_at_unix_millis,
                        updated_at_unix_millis, forward_id
               FOR UPDATE SKIP LOCKED
               LIMIT 1
             )
             UPDATE makosh_data.communication_cross_channel_forward_operations AS operation
             SET state = CASE
                   WHEN operation.state = 1 THEN 2
                   ELSE operation.state
                 END,
                 state_revision = CASE
                   WHEN operation.state = 1 THEN operation.state_revision + 1
                   ELSE operation.state_revision
                 END,
                 claimed_by = $3,
                 claim_epoch = operation.claim_epoch + 1,
                 lease_expires_at_unix_millis = $4,
                 updated_at_unix_millis = $2
             FROM candidate
             WHERE operation.logical_owner_id = candidate.logical_owner_id
               AND operation.forward_id = candidate.forward_id
             RETURNING candidate.previous_state, operation.*",
        )
        .bind(logical_owner_id)
        .bind(now_unix_millis)
        .bind(worker_id)
        .bind(lease_expires_at_unix_millis)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        let Some(row) = row else {
            transaction
                .commit()
                .await
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
            return Ok(None);
        };
        let previous_state: i16 = row
            .try_get("previous_state")
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
        let claim = claim_from_row(&row)?;
        if previous_state == STATE_ACCEPTED {
            insert_forward_transition(
                &mut transaction,
                logical_owner_id,
                &claim.forward_id,
                now_unix_millis,
            )
            .await?;
        }
        transaction
            .commit()
            .await
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        Ok(Some(claim))
    }

    pub async fn record_prepared_source(
        &self,
        claim: &CrossChannelForwardClaimV1,
        prepared_source: &CrossChannelForwardPreparedSourceV1,
        prepared_at_unix_millis: i64,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        if !valid_claim(claim)
            || claim.stage != CrossChannelForwardWorkStageV1::PreparingSource
            || claim.prepared_source.is_some()
            || !valid_prepared_source(prepared_source)
            || !valid_timestamp(prepared_at_unix_millis)
            || prepared_at_unix_millis > claim.lease_expires_at_unix_millis
        {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let source_revision = i64::try_from(prepared_source.source_revision)
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?;
        let body_length = i32::try_from(prepared_source.body_length)
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_cross_channel_forward_operations
             SET source_revision = $1, source_body_sha256 = $2,
                 source_body_length = $3, source_blob_reference = $4,
                 source_custody_proof = $5, attempt_count = 0,
                 next_attempt_at_unix_millis = $6,
                 claimed_by = NULL, lease_expires_at_unix_millis = NULL,
                 updated_at_unix_millis = $6
             WHERE logical_owner_id = $7 AND forward_id = $8
               AND state = $9 AND claimed_by = $10 AND claim_epoch = $11
               AND lease_expires_at_unix_millis >= $6
               AND source_revision IS NULL",
        )
        .bind(source_revision)
        .bind(prepared_source.body_sha256.as_slice())
        .bind(body_length)
        .bind(&prepared_source.blob_reference)
        .bind(&prepared_source.custody_proof)
        .bind(prepared_at_unix_millis)
        .bind(&claim.logical_owner_id)
        .bind(claim.forward_id.as_slice())
        .bind(STATE_PREPARING_SOURCE)
        .bind(&claim.worker_id)
        .bind(
            i64::try_from(claim.claim_epoch)
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?,
        )
        .execute(&self.pool)
        .await
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        claim_result(updated)
    }

    pub async fn begin_dispatch(
        &self,
        claim: &CrossChannelForwardClaimV1,
        begun_at_unix_millis: i64,
    ) -> Result<CrossChannelForwardClaimV1, CrossChannelForwardPersistenceErrorV1> {
        if !valid_claim(claim)
            || claim.stage != CrossChannelForwardWorkStageV1::PreparingSource
            || claim.prepared_source.is_none()
            || !valid_timestamp(begun_at_unix_millis)
            || begun_at_unix_millis > claim.lease_expires_at_unix_millis
        {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        let row = sqlx::query(
            "UPDATE makosh_data.communication_cross_channel_forward_operations
             SET state = $1, state_revision = state_revision + 1,
                 updated_at_unix_millis = $2
             WHERE logical_owner_id = $3 AND forward_id = $4
               AND state = $5 AND claimed_by = $6 AND claim_epoch = $7
               AND lease_expires_at_unix_millis >= $2
               AND source_revision IS NOT NULL
             RETURNING *",
        )
        .bind(STATE_DISPATCHING)
        .bind(begun_at_unix_millis)
        .bind(&claim.logical_owner_id)
        .bind(claim.forward_id.as_slice())
        .bind(STATE_PREPARING_SOURCE)
        .bind(&claim.worker_id)
        .bind(
            i64::try_from(claim.claim_epoch)
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?,
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?
        .ok_or(CrossChannelForwardPersistenceErrorV1::ClaimLost)?;
        let dispatch_claim = claim_from_row(&row)?;
        insert_forward_transition(
            &mut transaction,
            &claim.logical_owner_id,
            &claim.forward_id,
            begun_at_unix_millis,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        Ok(dispatch_claim)
    }

    pub async fn mark_delivery_accepted(
        &self,
        claim: &CrossChannelForwardClaimV1,
        delivery_intent_id: [u8; 16],
        accepted_at_unix_millis: i64,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        if !valid_claim(claim)
            || claim.stage != CrossChannelForwardWorkStageV1::Dispatching
            || !valid_id16(&delivery_intent_id)
            || !valid_timestamp(accepted_at_unix_millis)
            || accepted_at_unix_millis > claim.lease_expires_at_unix_millis
        {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        self.complete_terminal(
            claim,
            STATE_DELIVERY_ACCEPTED,
            Some(delivery_intent_id),
            None,
            1,
            accepted_at_unix_millis,
        )
        .await
    }

    pub async fn mark_rejected(
        &self,
        claim: &CrossChannelForwardClaimV1,
        error_code: u16,
        rejected_at_unix_millis: i64,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        if !valid_claim(claim)
            || !(1..=7).contains(&error_code)
            || !valid_timestamp(rejected_at_unix_millis)
            || rejected_at_unix_millis > claim.lease_expires_at_unix_millis
        {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        self.complete_terminal(
            claim,
            STATE_REJECTED,
            None,
            Some(error_code),
            2,
            rejected_at_unix_millis,
        )
        .await
    }

    pub async fn reschedule_claim(
        &self,
        claim: &CrossChannelForwardClaimV1,
        next_attempt_at_unix_millis: i64,
        rescheduled_at_unix_millis: i64,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        if !valid_claim(claim)
            || !valid_timestamp(rescheduled_at_unix_millis)
            || next_attempt_at_unix_millis <= rescheduled_at_unix_millis
            || rescheduled_at_unix_millis > claim.lease_expires_at_unix_millis
        {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_cross_channel_forward_operations
             SET attempt_count = LEAST(attempt_count + 1, 32),
                 next_attempt_at_unix_millis = $1,
                 claimed_by = NULL, lease_expires_at_unix_millis = NULL,
                 updated_at_unix_millis = $2
             WHERE logical_owner_id = $3 AND forward_id = $4
               AND state BETWEEN 1 AND 3
               AND claimed_by = $5 AND claim_epoch = $6
               AND lease_expires_at_unix_millis >= $2",
        )
        .bind(next_attempt_at_unix_millis)
        .bind(rescheduled_at_unix_millis)
        .bind(&claim.logical_owner_id)
        .bind(claim.forward_id.as_slice())
        .bind(&claim.worker_id)
        .bind(
            i64::try_from(claim.claim_epoch)
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?,
        )
        .execute(&self.pool)
        .await
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        claim_result(updated)
    }

    async fn complete_terminal(
        &self,
        claim: &CrossChannelForwardClaimV1,
        terminal_state: i16,
        delivery_intent_id: Option<[u8; 16]>,
        error_code: Option<u16>,
        cleanup_reason: i16,
        completed_at_unix_millis: i64,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        if let Some(prepared_source) = &claim.prepared_source {
            sqlx::query(
                "INSERT INTO makosh_data.communication_cross_channel_forward_cleanup (
                   logical_owner_id, forward_id, source_blob_reference,
                   source_custody_proof, reason, attempt_count,
                   next_attempt_at_unix_millis, created_at_unix_millis,
                   updated_at_unix_millis
                 ) VALUES ($1, $2, $3, $4, $5, 0, $6, $6, $6)
                 ON CONFLICT (logical_owner_id, forward_id) DO NOTHING",
            )
            .bind(&claim.logical_owner_id)
            .bind(claim.forward_id.as_slice())
            .bind(&prepared_source.blob_reference)
            .bind(&prepared_source.custody_proof)
            .bind(cleanup_reason)
            .bind(completed_at_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_cross_channel_forward_operations
             SET state = $1, state_revision = state_revision + 1,
                 delivery_intent_id = $2, error_code = $3,
                 source_blob_reference = NULL, source_custody_proof = NULL,
                 claimed_by = NULL, lease_expires_at_unix_millis = NULL,
                 updated_at_unix_millis = $4
             WHERE logical_owner_id = $5 AND forward_id = $6
               AND state BETWEEN 1 AND 3
               AND claimed_by = $7 AND claim_epoch = $8
               AND lease_expires_at_unix_millis >= $4",
        )
        .bind(terminal_state)
        .bind(delivery_intent_id.map(|id| id.to_vec()))
        .bind(
            error_code
                .map(i16::try_from)
                .transpose()
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?,
        )
        .bind(completed_at_unix_millis)
        .bind(&claim.logical_owner_id)
        .bind(claim.forward_id.as_slice())
        .bind(&claim.worker_id)
        .bind(
            i64::try_from(claim.claim_epoch)
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?,
        )
        .execute(&mut *transaction)
        .await
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if updated != 1 {
            return Err(CrossChannelForwardPersistenceErrorV1::ClaimLost);
        }
        insert_forward_transition(
            &mut transaction,
            &claim.logical_owner_id,
            &claim.forward_id,
            completed_at_unix_millis,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)
    }
}

fn claim_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<CrossChannelForwardClaimV1, CrossChannelForwardPersistenceErrorV1> {
    let state_code: i16 = row
        .try_get("state")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let state = state_from_code(state_code)?;
    let stage = match state {
        CrossChannelForwardStateV1::PreparingSource => {
            CrossChannelForwardWorkStageV1::PreparingSource
        }
        CrossChannelForwardStateV1::Dispatching => CrossChannelForwardWorkStageV1::Dispatching,
        _ => return Err(CrossChannelForwardPersistenceErrorV1::InvalidRow),
    };
    let source_revision: Option<i64> = row
        .try_get("source_revision")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let source_body_sha256: Option<Vec<u8>> = row
        .try_get("source_body_sha256")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let source_body_length: Option<i32> = row
        .try_get("source_body_length")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let source_blob_reference: Option<Vec<u8>> = row
        .try_get("source_blob_reference")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let source_custody_proof: Option<Vec<u8>> = row
        .try_get("source_custody_proof")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let prepared_source = match (
        source_revision,
        source_body_sha256,
        source_body_length,
        source_blob_reference,
        source_custody_proof,
    ) {
        (None, None, None, None, None) => None,
        (Some(revision), Some(sha256), Some(length), Some(reference), Some(proof)) => {
            let prepared = CrossChannelForwardPreparedSourceV1 {
                source_revision: positive_u64(revision)?,
                body_sha256: sha256
                    .try_into()
                    .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
                body_length: u32::try_from(length)
                    .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
                blob_reference: reference,
                custody_proof: proof,
            };
            if !valid_prepared_source(&prepared) {
                return Err(CrossChannelForwardPersistenceErrorV1::InvalidRow);
            }
            Some(prepared)
        }
        _ => return Err(CrossChannelForwardPersistenceErrorV1::InvalidRow),
    };
    let lease_expires_at_unix_millis: i64 = row
        .try_get("lease_expires_at_unix_millis")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let attempt_count: i16 = row
        .try_get("attempt_count")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let claim = CrossChannelForwardClaimV1 {
        logical_owner_id: row
            .try_get("logical_owner_id")
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        forward_id: id16(
            row.try_get("forward_id")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?,
        source_message_id: id16(
            row.try_get("source_message_id")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?,
        target_conversation_id: id16(
            row.try_get("target_conversation_id")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?,
        target_reply_to_message_id: optional_id16(
            row.try_get("target_reply_message_id")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?,
        stage,
        prepared_source,
        worker_id: row
            .try_get("claimed_by")
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        claim_epoch: positive_u64(
            row.try_get("claim_epoch")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?,
        attempt_count: u16::try_from(attempt_count)
            .ok()
            .filter(|value| *value <= 32)
            .ok_or(CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        lease_expires_at_unix_millis,
    };
    if !valid_claim(&claim) {
        return Err(CrossChannelForwardPersistenceErrorV1::InvalidRow);
    }
    Ok(claim)
}

fn valid_prepared_source(source: &CrossChannelForwardPreparedSourceV1) -> bool {
    source.source_revision > 0
        && (1..=MAX_BODY_BYTES_V1).contains(&source.body_length)
        && source.body_sha256.iter().any(|byte| *byte != 0)
        && (1..=MAX_BLOB_REFERENCE_BYTES_V1).contains(&source.blob_reference.len())
        && (1..=MAX_CUSTODY_PROOF_BYTES_V1).contains(&source.custody_proof.len())
}

fn valid_claim(claim: &CrossChannelForwardClaimV1) -> bool {
    valid_bounded_identity(&claim.logical_owner_id)
        && valid_bounded_identity(&claim.worker_id)
        && valid_id16(&claim.forward_id)
        && valid_id16(&claim.source_message_id)
        && valid_id16(&claim.target_conversation_id)
        && claim
            .target_reply_to_message_id
            .as_ref()
            .is_none_or(valid_id16)
        && claim.claim_epoch > 0
        && claim.attempt_count <= 32
        && valid_timestamp(claim.lease_expires_at_unix_millis)
        && claim
            .prepared_source
            .as_ref()
            .is_none_or(valid_prepared_source)
        && (claim.stage != CrossChannelForwardWorkStageV1::Dispatching
            || claim.prepared_source.is_some())
}

fn claim_result(rows_affected: u64) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
    if rows_affected == 1 {
        Ok(())
    } else {
        Err(CrossChannelForwardPersistenceErrorV1::ClaimLost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepared_source_is_bounded_and_requires_non_zero_digest() {
        let valid = CrossChannelForwardPreparedSourceV1 {
            source_revision: 1,
            body_sha256: [1; 32],
            body_length: 16,
            blob_reference: vec![2; 16],
            custody_proof: vec![3; 32],
        };
        assert!(valid_prepared_source(&valid));
        let mut invalid = valid;
        invalid.body_sha256 = [0; 32];
        assert!(!valid_prepared_source(&invalid));
    }
}
