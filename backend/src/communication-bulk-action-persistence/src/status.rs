use sqlx::Row;

use crate::{
    BulkDeliveryPersistenceErrorV1, CommunicationBulkActionPersistenceV1, valid_bounded_identity,
    valid_id16,
};

const MAX_STATUS_PAGE_V1: u16 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulkDeliveryBatchStateV1 {
    Accepted,
    Completed,
    CompletedWithErrors,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulkDeliveryTargetStateV1 {
    Pending,
    Dispatching,
    Accepted,
    Retryable,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkDeliveryTargetStatusV1 {
    pub target_operation_id: [u8; 16],
    pub state: BulkDeliveryTargetStateV1,
    pub delivery_intent_id: Option<[u8; 16]>,
    pub error_code: Option<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkDeliveryStatusPageV1 {
    pub batch_id: [u8; 16],
    pub state: BulkDeliveryBatchStateV1,
    pub state_revision: u64,
    pub targets: Vec<BulkDeliveryTargetStatusV1>,
    pub next_cursor: Option<u16>,
}

impl CommunicationBulkActionPersistenceV1 {
    pub async fn status_page(
        &self,
        logical_owner_id: &str,
        batch_id: [u8; 16],
        limit: u16,
        cursor: Option<u16>,
    ) -> Result<BulkDeliveryStatusPageV1, BulkDeliveryPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id)
            || !valid_id16(&batch_id)
            || limit == 0
            || limit > MAX_STATUS_PAGE_V1
        {
            return Err(BulkDeliveryPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let batch = sqlx::query(
            "SELECT state_revision,
                    COUNT(*) FILTER (WHERE targets.state = 3) AS accepted_count,
                    COUNT(*) FILTER (WHERE targets.state = 5) AS rejected_count,
                    COUNT(*) AS target_count
             FROM makosh_data.communication_bulk_action_batches AS batches
             JOIN makosh_data.communication_bulk_action_targets AS targets
               ON targets.logical_owner_id = batches.logical_owner_id
              AND targets.batch_id = batches.batch_id
             WHERE batches.logical_owner_id = $1 AND batches.batch_id = $2
             GROUP BY batches.state_revision",
        )
        .bind(logical_owner_id)
        .bind(batch_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?
        .ok_or(BulkDeliveryPersistenceErrorV1::NotFound)?;
        let state_revision = positive_u64(
            batch
                .try_get("state_revision")
                .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?,
        )?;
        let accepted_count: i64 = batch
            .try_get("accepted_count")
            .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?;
        let rejected_count: i64 = batch
            .try_get("rejected_count")
            .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?;
        let target_count: i64 = batch
            .try_get("target_count")
            .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?;
        let state = derive_batch_state(accepted_count, rejected_count, target_count)?;
        let start = cursor.unwrap_or(0);
        let rows = sqlx::query(
            "SELECT target_operation_id, ordinal, state,
                    delivery_intent_id, error_code
             FROM makosh_data.communication_bulk_action_targets
             WHERE logical_owner_id = $1 AND batch_id = $2 AND ordinal >= $3
             ORDER BY ordinal
             LIMIT $4",
        )
        .bind(logical_owner_id)
        .bind(batch_id.as_slice())
        .bind(i16::try_from(start).map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidInput)?)
        .bind(i64::from(limit) + 1)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
        let has_more = rows.len() > usize::from(limit);
        let mut targets = Vec::with_capacity(rows.len().min(usize::from(limit)));
        for row in rows.iter().take(usize::from(limit)) {
            targets.push(status_from_row(row)?);
        }
        let next_cursor = if has_more {
            start.checked_add(limit)
        } else {
            None
        };
        let page = BulkDeliveryStatusPageV1 {
            batch_id,
            state,
            state_revision,
            targets,
            next_cursor,
        };
        transaction
            .commit()
            .await
            .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(page)
    }
}

fn derive_batch_state(
    accepted_count: i64,
    rejected_count: i64,
    target_count: i64,
) -> Result<BulkDeliveryBatchStateV1, BulkDeliveryPersistenceErrorV1> {
    if target_count <= 0
        || accepted_count < 0
        || rejected_count < 0
        || accepted_count + rejected_count > target_count
    {
        return Err(BulkDeliveryPersistenceErrorV1::InvalidRow);
    }
    Ok(if accepted_count + rejected_count < target_count {
        BulkDeliveryBatchStateV1::Accepted
    } else if accepted_count == target_count {
        BulkDeliveryBatchStateV1::Completed
    } else if rejected_count == target_count {
        BulkDeliveryBatchStateV1::Rejected
    } else {
        BulkDeliveryBatchStateV1::CompletedWithErrors
    })
}

fn status_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<BulkDeliveryTargetStatusV1, BulkDeliveryPersistenceErrorV1> {
    let state: i16 = row
        .try_get("state")
        .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?;
    let delivery_intent_id: Option<Vec<u8>> = row
        .try_get("delivery_intent_id")
        .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?;
    let error_code: Option<i16> = row
        .try_get("error_code")
        .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?;
    Ok(BulkDeliveryTargetStatusV1 {
        target_operation_id: id16(
            row.try_get("target_operation_id")
                .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?,
        )?,
        state: match state {
            1 => BulkDeliveryTargetStateV1::Pending,
            2 => BulkDeliveryTargetStateV1::Dispatching,
            3 => BulkDeliveryTargetStateV1::Accepted,
            4 => BulkDeliveryTargetStateV1::Retryable,
            5 => BulkDeliveryTargetStateV1::Rejected,
            _ => return Err(BulkDeliveryPersistenceErrorV1::InvalidRow),
        },
        delivery_intent_id: delivery_intent_id.map(id16).transpose()?,
        error_code: error_code
            .map(|value| {
                u16::try_from(value)
                    .ok()
                    .filter(|value| (1..=5).contains(value))
                    .ok_or(BulkDeliveryPersistenceErrorV1::InvalidRow)
            })
            .transpose()?,
    })
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], BulkDeliveryPersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(valid_id16)
        .ok_or(BulkDeliveryPersistenceErrorV1::InvalidRow)
}

fn positive_u64(value: i64) -> Result<u64, BulkDeliveryPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(BulkDeliveryPersistenceErrorV1::InvalidRow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_only_contract_batch_states() {
        assert_eq!(
            derive_batch_state(0, 0, 2),
            Ok(BulkDeliveryBatchStateV1::Accepted)
        );
        assert_eq!(
            derive_batch_state(2, 0, 2),
            Ok(BulkDeliveryBatchStateV1::Completed)
        );
        assert_eq!(
            derive_batch_state(1, 1, 2),
            Ok(BulkDeliveryBatchStateV1::CompletedWithErrors)
        );
        assert_eq!(
            derive_batch_state(0, 2, 2),
            Ok(BulkDeliveryBatchStateV1::Rejected)
        );
    }
}
