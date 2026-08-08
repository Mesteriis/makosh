use sqlx::Row;

use crate::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardPersistenceErrorV1,
    operations::id16, valid_bounded_identity, valid_id16, valid_timestamp,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardCleanupReasonV1 {
    DeliveryAccepted,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardCleanupJobV1 {
    pub forward_id: [u8; 16],
    pub blob_reference: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
    pub reason: CrossChannelForwardCleanupReasonV1,
    pub attempt_count: u16,
}

impl CommunicationCrossChannelForwardPersistenceV1 {
    pub async fn next_cleanup(
        &self,
        logical_owner_id: &str,
        now_unix_millis: i64,
    ) -> Result<Option<CrossChannelForwardCleanupJobV1>, CrossChannelForwardPersistenceErrorV1>
    {
        if !valid_bounded_identity(logical_owner_id) || !valid_timestamp(now_unix_millis) {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT cleanup.forward_id, cleanup.source_blob_reference,
                    operation.source_body_length, operation.source_body_sha256,
                    cleanup.source_custody_proof, cleanup.reason,
                    cleanup.attempt_count
             FROM makosh_data.communication_cross_channel_forward_cleanup AS cleanup
             JOIN makosh_data.communication_cross_channel_forward_operations AS operation
               ON operation.logical_owner_id = cleanup.logical_owner_id
              AND operation.forward_id = cleanup.forward_id
             WHERE cleanup.logical_owner_id = $1
               AND cleanup.completed_at_unix_millis IS NULL
               AND cleanup.next_attempt_at_unix_millis <= $2
             ORDER BY cleanup.next_attempt_at_unix_millis, cleanup.forward_id
             LIMIT 1",
        )
        .bind(logical_owner_id)
        .bind(now_unix_millis)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        row.map(cleanup_from_row).transpose()
    }

    pub async fn complete_cleanup(
        &self,
        logical_owner_id: &str,
        forward_id: &[u8; 16],
        completed_at_unix_millis: i64,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id)
            || !valid_id16(forward_id)
            || !valid_timestamp(completed_at_unix_millis)
        {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_cross_channel_forward_cleanup
             SET completed_at_unix_millis = $1,
                 updated_at_unix_millis = $1
             WHERE logical_owner_id = $2 AND forward_id = $3
               AND completed_at_unix_millis IS NULL",
        )
        .bind(completed_at_unix_millis)
        .bind(logical_owner_id)
        .bind(forward_id.as_slice())
        .execute(&self.pool)
        .await
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        cleanup_result(updated)
    }

    pub async fn reschedule_cleanup(
        &self,
        logical_owner_id: &str,
        forward_id: &[u8; 16],
        expected_attempt_count: u16,
        next_attempt_at_unix_millis: i64,
        rescheduled_at_unix_millis: i64,
    ) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id)
            || !valid_id16(forward_id)
            || expected_attempt_count > 32
            || !valid_timestamp(rescheduled_at_unix_millis)
            || next_attempt_at_unix_millis <= rescheduled_at_unix_millis
        {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let expected_attempt_count = i16::try_from(expected_attempt_count)
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_cross_channel_forward_cleanup
             SET attempt_count = LEAST(attempt_count + 1, 32),
                 next_attempt_at_unix_millis = $1,
                 updated_at_unix_millis = $2
             WHERE logical_owner_id = $3 AND forward_id = $4
               AND completed_at_unix_millis IS NULL
               AND attempt_count = $5",
        )
        .bind(next_attempt_at_unix_millis)
        .bind(rescheduled_at_unix_millis)
        .bind(logical_owner_id)
        .bind(forward_id.as_slice())
        .bind(expected_attempt_count)
        .execute(&self.pool)
        .await
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        cleanup_result(updated)
    }
}

fn cleanup_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<CrossChannelForwardCleanupJobV1, CrossChannelForwardPersistenceErrorV1> {
    let reference: Vec<u8> = row
        .try_get("source_blob_reference")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let proof: Vec<u8> = row
        .try_get("source_custody_proof")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let reference = reference
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let body_length: i32 = row
        .try_get("source_body_length")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let declared_bytes = u64::try_from(body_length)
        .ok()
        .filter(|value| (1..=64 * 1024).contains(value))
        .ok_or(CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let sha256: Vec<u8> = row
        .try_get("source_body_sha256")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    let sha256 = sha256
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    if proof.is_empty() || proof.len() > 4_096 {
        return Err(CrossChannelForwardPersistenceErrorV1::InvalidRow);
    }
    let attempt_count: i16 = row
        .try_get("attempt_count")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    Ok(CrossChannelForwardCleanupJobV1 {
        forward_id: id16(
            row.try_get("forward_id")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?,
        blob_reference: reference,
        declared_bytes,
        sha256,
        custody_proof: proof,
        reason: match row
            .try_get::<i16, _>("reason")
            .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?
        {
            1 => CrossChannelForwardCleanupReasonV1::DeliveryAccepted,
            2 => CrossChannelForwardCleanupReasonV1::Rejected,
            _ => return Err(CrossChannelForwardPersistenceErrorV1::InvalidRow),
        },
        attempt_count: u16::try_from(attempt_count)
            .ok()
            .filter(|value| *value <= 32)
            .ok_or(CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
    })
}

fn cleanup_result(rows_affected: u64) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
    if rows_affected == 1 {
        Ok(())
    } else {
        Err(CrossChannelForwardPersistenceErrorV1::NotFound)
    }
}
