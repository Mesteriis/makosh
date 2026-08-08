use sqlx::Row;

use crate::{
    CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryBodyReceiptV1,
    DelayedDeliveryPersistenceErrorV1, valid_body_receipt, valid_id16,
};

const STATE_DELIVERY_ACCEPTED: i16 = 6;
const STATE_CANCELLED: i16 = 8;
const STATE_FAILED: i16 = 9;
const MAX_ATTEMPTS: u32 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum DelayedDeliveryBodyCleanupReasonV1 {
    DeliveryAccepted = 1,
    DeliveryRejected = 2,
    DeliveryCancelled = 3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryBodyCleanupJobV1 {
    pub logical_owner_id: String,
    pub delayed_operation_id: [u8; 16],
    pub body_receipt: DelayedDeliveryBodyReceiptV1,
    pub reason: DelayedDeliveryBodyCleanupReasonV1,
    pub attempt_count: u32,
}

impl CommunicationDelayedDeliveryPersistenceV1 {
    pub async fn next_body_cleanup(
        &self,
        logical_owner_id: &str,
        now_unix_millis: u64,
    ) -> Result<Option<DelayedDeliveryBodyCleanupJobV1>, DelayedDeliveryPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || now_unix_millis == 0 {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let now = signed(now_unix_millis)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        reconcile_terminal_cleanup(&mut transaction, logical_owner_id, now).await?;
        let row = sqlx::query(
            "SELECT cleanup.delayed_operation_id, cleanup.reason, cleanup.attempt_count,
                    operation.body_reference_id, operation.body_declared_bytes,
                    operation.body_sha256, operation.body_custody_proof
             FROM makosh_data.communication_delayed_delivery_body_cleanup AS cleanup
             JOIN makosh_data.communication_delayed_delivery_operations AS operation
               ON operation.logical_owner_id = cleanup.logical_owner_id
              AND operation.delayed_operation_id = cleanup.delayed_operation_id
             WHERE cleanup.logical_owner_id = $1
               AND cleanup.completed_at_unix_millis IS NULL
               AND cleanup.next_attempt_at_unix_millis <= $2
             ORDER BY cleanup.next_attempt_at_unix_millis, cleanup.delayed_operation_id
             LIMIT 1",
        )
        .bind(logical_owner_id)
        .bind(now)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        transaction
            .commit()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        row.map(|row| cleanup_job(logical_owner_id, &row))
            .transpose()
    }

    pub async fn complete_body_cleanup(
        &self,
        logical_owner_id: &str,
        delayed_operation_id: &[u8; 16],
        completed_at_unix_millis: u64,
    ) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id16(delayed_operation_id)
            || completed_at_unix_millis == 0
        {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let completed_at = signed(completed_at_unix_millis)?;
        let affected = sqlx::query(
            "UPDATE makosh_data.communication_delayed_delivery_body_cleanup
             SET completed_at_unix_millis = $3, updated_at_unix_millis = $3
             WHERE logical_owner_id = $1 AND delayed_operation_id = $2
               AND completed_at_unix_millis IS NULL",
        )
        .bind(logical_owner_id)
        .bind(delayed_operation_id.as_slice())
        .bind(completed_at)
        .execute(&self.pool)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if affected != 1 {
            return Err(DelayedDeliveryPersistenceErrorV1::Conflict);
        }
        Ok(())
    }

    pub async fn reschedule_body_cleanup(
        &self,
        logical_owner_id: &str,
        delayed_operation_id: &[u8; 16],
        expected_attempt_count: u32,
        next_attempt_at_unix_millis: u64,
        rescheduled_at_unix_millis: u64,
    ) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id16(delayed_operation_id)
            || expected_attempt_count > MAX_ATTEMPTS
            || next_attempt_at_unix_millis == 0
            || rescheduled_at_unix_millis == 0
            || next_attempt_at_unix_millis <= rescheduled_at_unix_millis
        {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let next_attempt_at = signed(next_attempt_at_unix_millis)?;
        let rescheduled_at = signed(rescheduled_at_unix_millis)?;
        let affected = sqlx::query(
            "UPDATE makosh_data.communication_delayed_delivery_body_cleanup
             SET attempt_count = LEAST(attempt_count + 1, 32),
                 next_attempt_at_unix_millis = $4,
                 updated_at_unix_millis = $5
             WHERE logical_owner_id = $1 AND delayed_operation_id = $2
               AND attempt_count = $3
               AND completed_at_unix_millis IS NULL",
        )
        .bind(logical_owner_id)
        .bind(delayed_operation_id.as_slice())
        .bind(
            i32::try_from(expected_attempt_count)
                .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidInput)?,
        )
        .bind(next_attempt_at)
        .bind(rescheduled_at)
        .execute(&self.pool)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if affected != 1 {
            return Err(DelayedDeliveryPersistenceErrorV1::Conflict);
        }
        Ok(())
    }
}

pub(crate) async fn enqueue_body_cleanup(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    delayed_operation_id: &[u8; 16],
    reason: DelayedDeliveryBodyCleanupReasonV1,
    created_at_unix_millis: i64,
) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.communication_delayed_delivery_body_cleanup (
           logical_owner_id, delayed_operation_id, reason, attempt_count,
           next_attempt_at_unix_millis, created_at_unix_millis,
           updated_at_unix_millis
         ) VALUES ($1, $2, $3, 0, $4, $4, $4)
         ON CONFLICT (logical_owner_id, delayed_operation_id) DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(delayed_operation_id.as_slice())
    .bind(reason as i16)
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
    .rows_affected();
    if inserted == 1 {
        return Ok(());
    }
    let existing = sqlx::query_scalar::<_, i16>(
        "SELECT reason
         FROM makosh_data.communication_delayed_delivery_body_cleanup
         WHERE logical_owner_id = $1 AND delayed_operation_id = $2",
    )
    .bind(logical_owner_id)
    .bind(delayed_operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
    (existing == Some(reason as i16))
        .then_some(())
        .ok_or(DelayedDeliveryPersistenceErrorV1::Conflict)
}

async fn reconcile_terminal_cleanup(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    now_unix_millis: i64,
) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.communication_delayed_delivery_body_cleanup (
           logical_owner_id, delayed_operation_id, reason, attempt_count,
           next_attempt_at_unix_millis, created_at_unix_millis,
           updated_at_unix_millis
         )
         SELECT logical_owner_id, delayed_operation_id,
                CASE state WHEN $2 THEN 1 WHEN $3 THEN 3 ELSE 2 END,
                0, $5, $5, $5
         FROM makosh_data.communication_delayed_delivery_operations
         WHERE logical_owner_id = $1 AND state = ANY($4)
         ON CONFLICT (logical_owner_id, delayed_operation_id) DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(STATE_DELIVERY_ACCEPTED)
    .bind(STATE_CANCELLED)
    .bind([STATE_DELIVERY_ACCEPTED, STATE_CANCELLED, STATE_FAILED].as_slice())
    .bind(now_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
    Ok(())
}

fn cleanup_job(
    logical_owner_id: &str,
    row: &sqlx::postgres::PgRow,
) -> Result<DelayedDeliveryBodyCleanupJobV1, DelayedDeliveryPersistenceErrorV1> {
    let body_receipt = DelayedDeliveryBodyReceiptV1 {
        reference_id: id16(row.try_get("body_reference_id").map_err(row_error)?)?,
        declared_bytes: positive_u64(row.try_get("body_declared_bytes").map_err(row_error)?)?,
        sha256: id32(row.try_get("body_sha256").map_err(row_error)?)?,
        custody_proof: row.try_get("body_custody_proof").map_err(row_error)?,
    };
    if !valid_body_receipt(&body_receipt) {
        return Err(DelayedDeliveryPersistenceErrorV1::InvalidRow);
    }
    Ok(DelayedDeliveryBodyCleanupJobV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        delayed_operation_id: id16(row.try_get("delayed_operation_id").map_err(row_error)?)?,
        body_receipt,
        reason: cleanup_reason(row.try_get("reason").map_err(row_error)?)?,
        attempt_count: u32::try_from(row.try_get::<i32, _>("attempt_count").map_err(row_error)?)
            .ok()
            .filter(|attempts| *attempts <= MAX_ATTEMPTS)
            .ok_or(DelayedDeliveryPersistenceErrorV1::InvalidRow)?,
    })
}

fn cleanup_reason(
    value: i16,
) -> Result<DelayedDeliveryBodyCleanupReasonV1, DelayedDeliveryPersistenceErrorV1> {
    match value {
        1 => Ok(DelayedDeliveryBodyCleanupReasonV1::DeliveryAccepted),
        2 => Ok(DelayedDeliveryBodyCleanupReasonV1::DeliveryRejected),
        3 => Ok(DelayedDeliveryBodyCleanupReasonV1::DeliveryCancelled),
        _ => Err(DelayedDeliveryPersistenceErrorV1::InvalidRow),
    }
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128
}

fn signed(value: u64) -> Result<i64, DelayedDeliveryPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidInput)
}

fn row_error(_: sqlx::Error) -> DelayedDeliveryPersistenceErrorV1 {
    DelayedDeliveryPersistenceErrorV1::InvalidRow
}

fn positive_u64(value: i64) -> Result<u64, DelayedDeliveryPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(DelayedDeliveryPersistenceErrorV1::InvalidRow)
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], DelayedDeliveryPersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(valid_id16)
        .ok_or(DelayedDeliveryPersistenceErrorV1::InvalidRow)
}

fn id32(value: Vec<u8>) -> Result<[u8; 32], DelayedDeliveryPersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(DelayedDeliveryPersistenceErrorV1::InvalidRow)
}
