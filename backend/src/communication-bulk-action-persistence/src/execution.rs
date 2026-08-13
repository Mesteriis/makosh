use sqlx::Row;

use crate::{
    BulkDeliveryPersistenceErrorV1, CommunicationBulkActionPersistenceV1,
    realtime::insert_batch_transition, valid_bounded_identity, valid_id16, valid_timestamp,
};

const STATE_PENDING: i16 = 1;
const STATE_DISPATCHING: i16 = 2;
const STATE_ACCEPTED: i16 = 3;
const STATE_RETRYABLE: i16 = 4;
const STATE_REJECTED: i16 = 5;
const ERROR_RETRY_EXHAUSTED: i16 = 4;

pub const MAX_TARGET_ATTEMPTS_V1: u16 = 3;
pub const TARGET_LEASE_SECONDS_V1: i64 = 30;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkDeliveryTargetClaimV1 {
    pub logical_owner_id: String,
    pub batch_id: [u8; 16],
    pub target_operation_id: [u8; 16],
    pub conversation_id: [u8; 16],
    pub reply_to_message_id: Option<[u8; 16]>,
    pub body_utf8: Vec<u8>,
    pub worker_id: String,
    pub claim_epoch: u64,
    pub attempt_count: u16,
    pub lease_expires_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompleteTargetOutcomeV1 {
    Accepted,
    RetryScheduled { next_attempt_at_unix_seconds: i64 },
    RetryExhausted,
    Rejected,
}

impl CommunicationBulkActionPersistenceV1 {
    pub async fn claim_next_target(
        &self,
        logical_owner_id: &str,
        worker_id: &str,
        now_unix_seconds: i64,
    ) -> Result<Option<BulkDeliveryTargetClaimV1>, BulkDeliveryPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id)
            || !valid_bounded_identity(worker_id)
            || !valid_timestamp(now_unix_seconds)
        {
            return Err(BulkDeliveryPersistenceErrorV1::InvalidInput);
        }
        let lease_expires_at = now_unix_seconds
            .checked_add(TARGET_LEASE_SECONDS_V1)
            .ok_or(BulkDeliveryPersistenceErrorV1::InvalidInput)?;
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;

        let exhausted_batches = sqlx::query(
            "WITH exhausted AS (
               UPDATE makosh_data.communication_bulk_action_targets
               SET state = $1, error_code = $2, claimed_by = NULL,
                   lease_expires_at_unix_seconds = NULL,
                   updated_at_unix_seconds = $3
               WHERE logical_owner_id = $4 AND state = $5
                 AND lease_expires_at_unix_seconds < $3
                 AND attempt_count >= $6
               RETURNING logical_owner_id, batch_id
             ), changed_batches AS (
               SELECT DISTINCT logical_owner_id, batch_id FROM exhausted
             )
             UPDATE makosh_data.communication_bulk_action_batches AS batches
             SET state_revision = state_revision + 1,
                 updated_at_unix_seconds = $3
             FROM changed_batches
             WHERE batches.logical_owner_id = changed_batches.logical_owner_id
               AND batches.batch_id = changed_batches.batch_id
             RETURNING batches.batch_id",
        )
        .bind(STATE_REJECTED)
        .bind(ERROR_RETRY_EXHAUSTED)
        .bind(now_unix_seconds)
        .bind(logical_owner_id)
        .bind(STATE_DISPATCHING)
        .bind(i16::try_from(MAX_TARGET_ATTEMPTS_V1).expect("bounded attempts"))
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
        for row in exhausted_batches {
            let batch_id = id16(row.try_get("batch_id").map_err(row_error)?)?;
            insert_batch_transition(
                &mut transaction,
                logical_owner_id,
                &batch_id,
                now_unix_seconds,
            )
            .await?;
        }

        let row = sqlx::query(
            "WITH candidate AS (
               SELECT logical_owner_id, batch_id, target_operation_id
               FROM makosh_data.communication_bulk_action_targets
               WHERE logical_owner_id = $1 AND attempt_count < $2
                 AND (
                   state = $3
                   OR (state = $4 AND next_attempt_at_unix_seconds <= $5)
                   OR (state = $6 AND lease_expires_at_unix_seconds < $5)
                 )
               ORDER BY COALESCE(next_attempt_at_unix_seconds, 0),
                        updated_at_unix_seconds, batch_id, ordinal
               FOR UPDATE SKIP LOCKED
               LIMIT 1
             )
             UPDATE makosh_data.communication_bulk_action_targets AS targets
             SET state = $6, attempt_count = targets.attempt_count + 1,
                 claimed_by = $7, claim_epoch = targets.claim_epoch + 1,
                 lease_expires_at_unix_seconds = $8,
                 next_attempt_at_unix_seconds = NULL, error_code = NULL,
                 updated_at_unix_seconds = $5
             FROM candidate
             WHERE targets.logical_owner_id = candidate.logical_owner_id
               AND targets.batch_id = candidate.batch_id
               AND targets.target_operation_id = candidate.target_operation_id
             RETURNING targets.*",
        )
        .bind(logical_owner_id)
        .bind(i16::try_from(MAX_TARGET_ATTEMPTS_V1).expect("bounded attempts"))
        .bind(STATE_PENDING)
        .bind(STATE_RETRYABLE)
        .bind(now_unix_seconds)
        .bind(STATE_DISPATCHING)
        .bind(worker_id)
        .bind(lease_expires_at)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
        let Some(row) = row else {
            transaction
                .commit()
                .await
                .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
            return Ok(None);
        };
        let batch_id = id16(row.try_get("batch_id").map_err(row_error)?)?;
        increment_batch_revision(
            &mut transaction,
            logical_owner_id,
            &batch_id,
            now_unix_seconds,
        )
        .await?;
        insert_batch_transition(
            &mut transaction,
            logical_owner_id,
            &batch_id,
            now_unix_seconds,
        )
        .await?;
        let claim = claim_from_row(&row)?;
        transaction
            .commit()
            .await
            .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(Some(claim))
    }

    pub async fn mark_target_accepted(
        &self,
        claim: &BulkDeliveryTargetClaimV1,
        delivery_intent_id: [u8; 16],
        now_unix_seconds: i64,
    ) -> Result<CompleteTargetOutcomeV1, BulkDeliveryPersistenceErrorV1> {
        if !valid_claim(claim)
            || !valid_id16(&delivery_intent_id)
            || !valid_timestamp(now_unix_seconds)
        {
            return Err(BulkDeliveryPersistenceErrorV1::InvalidInput);
        }
        self.complete_claim(
            claim,
            STATE_ACCEPTED,
            Some(delivery_intent_id),
            None,
            None,
            now_unix_seconds,
        )
        .await?;
        Ok(CompleteTargetOutcomeV1::Accepted)
    }

    pub async fn mark_target_retryable(
        &self,
        claim: &BulkDeliveryTargetClaimV1,
        error_code: u16,
        now_unix_seconds: i64,
    ) -> Result<CompleteTargetOutcomeV1, BulkDeliveryPersistenceErrorV1> {
        if !valid_claim(claim)
            || !(1..=5).contains(&error_code)
            || !valid_timestamp(now_unix_seconds)
        {
            return Err(BulkDeliveryPersistenceErrorV1::InvalidInput);
        }
        if claim.attempt_count >= MAX_TARGET_ATTEMPTS_V1 {
            self.complete_claim(
                claim,
                STATE_REJECTED,
                None,
                Some(ERROR_RETRY_EXHAUSTED),
                None,
                now_unix_seconds,
            )
            .await?;
            return Ok(CompleteTargetOutcomeV1::RetryExhausted);
        }
        let next_attempt = now_unix_seconds
            .checked_add(retry_delay_seconds(claim.attempt_count))
            .ok_or(BulkDeliveryPersistenceErrorV1::InvalidInput)?;
        self.complete_claim(
            claim,
            STATE_RETRYABLE,
            None,
            Some(
                i16::try_from(error_code)
                    .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidInput)?,
            ),
            Some(next_attempt),
            now_unix_seconds,
        )
        .await?;
        Ok(CompleteTargetOutcomeV1::RetryScheduled {
            next_attempt_at_unix_seconds: next_attempt,
        })
    }

    pub async fn mark_target_rejected(
        &self,
        claim: &BulkDeliveryTargetClaimV1,
        error_code: u16,
        now_unix_seconds: i64,
    ) -> Result<CompleteTargetOutcomeV1, BulkDeliveryPersistenceErrorV1> {
        if !valid_claim(claim)
            || !(1..=5).contains(&error_code)
            || !valid_timestamp(now_unix_seconds)
        {
            return Err(BulkDeliveryPersistenceErrorV1::InvalidInput);
        }
        self.complete_claim(
            claim,
            STATE_REJECTED,
            None,
            Some(
                i16::try_from(error_code)
                    .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidInput)?,
            ),
            None,
            now_unix_seconds,
        )
        .await?;
        Ok(CompleteTargetOutcomeV1::Rejected)
    }

    async fn complete_claim(
        &self,
        claim: &BulkDeliveryTargetClaimV1,
        state: i16,
        delivery_intent_id: Option<[u8; 16]>,
        error_code: Option<i16>,
        next_attempt_at: Option<i64>,
        now_unix_seconds: i64,
    ) -> Result<(), BulkDeliveryPersistenceErrorV1> {
        let claim_epoch = i64::try_from(claim.claim_epoch)
            .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidInput)?;
        let mut transaction = self
            .begin_owner_transaction(&claim.logical_owner_id)
            .await?;
        let changed = sqlx::query(
            "UPDATE makosh_data.communication_bulk_action_targets
             SET state = $1, delivery_intent_id = $2, error_code = $3,
                 next_attempt_at_unix_seconds = $4, claimed_by = NULL,
                 lease_expires_at_unix_seconds = NULL,
                 updated_at_unix_seconds = $5
             WHERE logical_owner_id = $6 AND batch_id = $7
               AND target_operation_id = $8 AND state = $9
               AND claimed_by = $10 AND claim_epoch = $11",
        )
        .bind(state)
        .bind(delivery_intent_id.map(|id| id.to_vec()))
        .bind(error_code)
        .bind(next_attempt_at)
        .bind(now_unix_seconds)
        .bind(&claim.logical_owner_id)
        .bind(claim.batch_id.as_slice())
        .bind(claim.target_operation_id.as_slice())
        .bind(STATE_DISPATCHING)
        .bind(&claim.worker_id)
        .bind(claim_epoch)
        .execute(&mut *transaction)
        .await
        .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if changed != 1 {
            return Err(BulkDeliveryPersistenceErrorV1::ClaimLost);
        }
        increment_batch_revision(
            &mut transaction,
            &claim.logical_owner_id,
            &claim.batch_id,
            now_unix_seconds,
        )
        .await?;
        insert_batch_transition(
            &mut transaction,
            &claim.logical_owner_id,
            &claim.batch_id,
            now_unix_seconds,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)
    }
}

fn retry_delay_seconds(attempt_count: u16) -> i64 {
    1_i64 << u32::from(attempt_count.min(MAX_TARGET_ATTEMPTS_V1))
}

async fn increment_batch_revision(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    batch_id: &[u8; 16],
    now_unix_seconds: i64,
) -> Result<(), BulkDeliveryPersistenceErrorV1> {
    let changed = sqlx::query(
        "UPDATE makosh_data.communication_bulk_action_batches
         SET state_revision = state_revision + 1, updated_at_unix_seconds = $1
         WHERE logical_owner_id = $2 AND batch_id = $3",
    )
    .bind(now_unix_seconds)
    .bind(logical_owner_id)
    .bind(batch_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?
    .rows_affected();
    if changed == 1 {
        Ok(())
    } else {
        Err(BulkDeliveryPersistenceErrorV1::InvalidRow)
    }
}

fn claim_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<BulkDeliveryTargetClaimV1, BulkDeliveryPersistenceErrorV1> {
    Ok(BulkDeliveryTargetClaimV1 {
        logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
        batch_id: id16(row.try_get("batch_id").map_err(row_error)?)?,
        target_operation_id: id16(row.try_get("target_operation_id").map_err(row_error)?)?,
        conversation_id: id16(
            row.try_get("canonical_conversation_id")
                .map_err(row_error)?,
        )?,
        reply_to_message_id: optional_id16(
            row.try_get("canonical_reply_message_id")
                .map_err(row_error)?,
        )?,
        body_utf8: row.try_get("body_utf8").map_err(row_error)?,
        worker_id: row.try_get("claimed_by").map_err(row_error)?,
        claim_epoch: positive_u64(row.try_get("claim_epoch").map_err(row_error)?)?,
        attempt_count: positive_u16(row.try_get("attempt_count").map_err(row_error)?)?,
        lease_expires_at_unix_seconds: row
            .try_get("lease_expires_at_unix_seconds")
            .map_err(row_error)?,
    })
}

fn valid_claim(claim: &BulkDeliveryTargetClaimV1) -> bool {
    valid_bounded_identity(&claim.logical_owner_id)
        && valid_bounded_identity(&claim.worker_id)
        && valid_id16(&claim.batch_id)
        && valid_id16(&claim.target_operation_id)
        && valid_id16(&claim.conversation_id)
        && claim.reply_to_message_id.as_ref().is_none_or(valid_id16)
        && !claim.body_utf8.is_empty()
        && claim.body_utf8.len() <= 64 * 1024
        && claim.claim_epoch > 0
        && (1..=MAX_TARGET_ATTEMPTS_V1).contains(&claim.attempt_count)
        && valid_timestamp(claim.lease_expires_at_unix_seconds)
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], BulkDeliveryPersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(valid_id16)
        .ok_or(BulkDeliveryPersistenceErrorV1::InvalidRow)
}

fn optional_id16(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; 16]>, BulkDeliveryPersistenceErrorV1> {
    value.map(id16).transpose()
}

fn positive_u64(value: i64) -> Result<u64, BulkDeliveryPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(BulkDeliveryPersistenceErrorV1::InvalidRow)
}

fn positive_u16(value: i16) -> Result<u16, BulkDeliveryPersistenceErrorV1> {
    u16::try_from(value)
        .ok()
        .filter(|value| (1..=MAX_TARGET_ATTEMPTS_V1).contains(value))
        .ok_or(BulkDeliveryPersistenceErrorV1::InvalidRow)
}

fn row_error(_: sqlx::Error) -> BulkDeliveryPersistenceErrorV1 {
    BulkDeliveryPersistenceErrorV1::InvalidRow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_delay_is_deterministic_and_bounded_by_attempt_ceiling() {
        assert_eq!(retry_delay_seconds(1), 2);
        assert_eq!(retry_delay_seconds(2), 4);
        assert_eq!(retry_delay_seconds(3), 8);
        assert_eq!(retry_delay_seconds(100), 8);
    }
}
