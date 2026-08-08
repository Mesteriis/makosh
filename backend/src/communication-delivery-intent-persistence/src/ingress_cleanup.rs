use sqlx::Row;

use crate::{
    CommunicationDeliveryIntentPersistenceV1, DeliveryIntentIngressBlobReceiptV1,
    DeliveryIntentPersistenceErrorV1, valid_bounded_identity, valid_id16, valid_timestamp,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentIngressCleanupReasonV1 {
    Submitted,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryIntentIngressCleanupJobV1 {
    pub logical_owner_id: String,
    pub intent_id: [u8; 16],
    pub body_receipt: DeliveryIntentIngressBlobReceiptV1,
    pub reason: DeliveryIntentIngressCleanupReasonV1,
    pub attempt_count: u16,
}

impl CommunicationDeliveryIntentPersistenceV1 {
    pub async fn next_ingress_cleanup(
        &self,
        logical_owner_id: &str,
        now_unix_seconds: i64,
    ) -> Result<Option<DeliveryIntentIngressCleanupJobV1>, DeliveryIntentPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id) || !valid_timestamp(now_unix_seconds) {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT logical_owner_id, intent_id, reference_id, declared_bytes,
                    sha256, custody_source_proof, reason, attempt_count
             FROM makosh_data.communication_delivery_intent_ingress_cleanup
             WHERE logical_owner_id = $1
               AND completed_at_unix_seconds IS NULL
               AND next_attempt_at_unix_seconds <= $2
             ORDER BY next_attempt_at_unix_seconds, intent_id
             LIMIT 1",
        )
        .bind(logical_owner_id)
        .bind(now_unix_seconds)
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?;
        row.map(cleanup_from_row).transpose()
    }

    pub async fn complete_ingress_cleanup(
        &self,
        logical_owner_id: &str,
        intent_id: &[u8; 16],
        completed_at_unix_seconds: i64,
    ) -> Result<(), DeliveryIntentPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id)
            || !valid_id16(intent_id)
            || !valid_timestamp(completed_at_unix_seconds)
        {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_delivery_intent_ingress_cleanup
             SET completed_at_unix_seconds = $1,
                 updated_at_unix_seconds = $1
             WHERE logical_owner_id = $2 AND intent_id = $3
               AND completed_at_unix_seconds IS NULL",
        )
        .bind(completed_at_unix_seconds)
        .bind(logical_owner_id)
        .bind(intent_id.as_slice())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?
        .rows_affected();
        exact_update(updated)
    }

    pub async fn reschedule_ingress_cleanup(
        &self,
        logical_owner_id: &str,
        intent_id: &[u8; 16],
        expected_attempt_count: u16,
        next_attempt_at_unix_seconds: i64,
        rescheduled_at_unix_seconds: i64,
    ) -> Result<(), DeliveryIntentPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id)
            || !valid_id16(intent_id)
            || expected_attempt_count > 32
            || !valid_timestamp(rescheduled_at_unix_seconds)
            || next_attempt_at_unix_seconds <= rescheduled_at_unix_seconds
        {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let expected_attempt_count = i16::try_from(expected_attempt_count)
            .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_delivery_intent_ingress_cleanup
             SET attempt_count = LEAST(attempt_count + 1, 32),
                 next_attempt_at_unix_seconds = $1,
                 updated_at_unix_seconds = $2
             WHERE logical_owner_id = $3 AND intent_id = $4
               AND completed_at_unix_seconds IS NULL
               AND attempt_count = $5",
        )
        .bind(next_attempt_at_unix_seconds)
        .bind(rescheduled_at_unix_seconds)
        .bind(logical_owner_id)
        .bind(intent_id.as_slice())
        .bind(expected_attempt_count)
        .execute(&self.pool)
        .await
        .map_err(storage_error)?
        .rows_affected();
        exact_update(updated)
    }
}

fn cleanup_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<DeliveryIntentIngressCleanupJobV1, DeliveryIntentPersistenceErrorV1> {
    let logical_owner_id: String = row
        .try_get("logical_owner_id")
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
    let intent_id = fixed_nonzero::<16>(
        row.try_get("intent_id")
            .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?,
    )?;
    let reference_id = fixed_nonzero::<16>(
        row.try_get("reference_id")
            .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?,
    )?;
    let sha256 = fixed_nonzero::<32>(
        row.try_get("sha256")
            .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?,
    )?;
    let declared_bytes: i64 = row
        .try_get("declared_bytes")
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
    let declared_bytes = u64::try_from(declared_bytes)
        .ok()
        .filter(|value| (1..=16 * 1024 * 1024).contains(value))
        .ok_or(DeliveryIntentPersistenceErrorV1::InvalidRow)?;
    let custody_source_proof: Vec<u8> = row
        .try_get("custody_source_proof")
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
    if !valid_bounded_identity(&logical_owner_id)
        || !(1..=2_048).contains(&custody_source_proof.len())
    {
        return Err(DeliveryIntentPersistenceErrorV1::InvalidRow);
    }
    let attempt_count: i16 = row
        .try_get("attempt_count")
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
    Ok(DeliveryIntentIngressCleanupJobV1 {
        logical_owner_id,
        intent_id,
        body_receipt: DeliveryIntentIngressBlobReceiptV1 {
            reference_id,
            declared_bytes,
            sha256,
            custody_source_proof,
        },
        reason: match row
            .try_get::<i16, _>("reason")
            .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?
        {
            1 => DeliveryIntentIngressCleanupReasonV1::Submitted,
            2 => DeliveryIntentIngressCleanupReasonV1::Rejected,
            _ => return Err(DeliveryIntentPersistenceErrorV1::InvalidRow),
        },
        attempt_count: u16::try_from(attempt_count)
            .ok()
            .filter(|value| *value <= 32)
            .ok_or(DeliveryIntentPersistenceErrorV1::InvalidRow)?,
    })
}

fn fixed_nonzero<const SIZE: usize>(
    value: Vec<u8>,
) -> Result<[u8; SIZE], DeliveryIntentPersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|bytes: &[u8; SIZE]| bytes.iter().any(|byte| *byte != 0))
        .ok_or(DeliveryIntentPersistenceErrorV1::InvalidRow)
}

fn exact_update(rows_affected: u64) -> Result<(), DeliveryIntentPersistenceErrorV1> {
    if rows_affected == 1 {
        Ok(())
    } else {
        Err(DeliveryIntentPersistenceErrorV1::Conflict)
    }
}

fn storage_error(_: sqlx::Error) -> DeliveryIntentPersistenceErrorV1 {
    DeliveryIntentPersistenceErrorV1::StorageUnavailable
}
