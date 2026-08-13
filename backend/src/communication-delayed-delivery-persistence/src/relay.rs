use sqlx::Row;

use crate::{
    CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryDurableMessageV1,
    DelayedDeliveryPersistenceErrorV1, operations::valid_owner, valid_durable_message, valid_id16,
};

const SCHEDULER_COMMAND_KIND_V1: &str = "scheduler.schedule.command.v1";
const SCHEDULER_ACCEPTANCE_KIND_V1: &str = "scheduler.job_run.acceptance.v1";
const SCHEDULER_RESULT_KIND_V1: &str = "scheduler.job_run.result.v1";
const MAX_RELAY_BATCH_V1: u16 = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedDeliveryOutboxStreamV1 {
    SchedulerCommand,
    SchedulerReceipt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryOutboxRecordV1 {
    pub delayed_operation_id: [u8; 16],
    pub message: DelayedDeliveryDurableMessageV1,
}

impl CommunicationDelayedDeliveryPersistenceV1 {
    pub async fn owns_scheduler_command(
        &self,
        logical_owner_id: &str,
        delayed_operation_id: &[u8; 16],
        message_id: &[u8; 16],
    ) -> Result<bool, DelayedDeliveryPersistenceErrorV1> {
        if logical_owner_id.is_empty() || logical_owner_id.len() > 128 {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let owned = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
               SELECT 1
               FROM makosh_data.communication_delayed_delivery_outbox
               WHERE logical_owner_id = $1
                 AND delayed_operation_id = $2
                 AND message_id = $3
                 AND contract_kind = 'scheduler.schedule.command.v1'
             )",
        )
        .bind(logical_owner_id)
        .bind(delayed_operation_id.as_slice())
        .bind(message_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        transaction
            .commit()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(owned)
    }

    pub async fn pending_scheduler_commands(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<Vec<DelayedDeliveryOutboxRecordV1>, DelayedDeliveryPersistenceErrorV1> {
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let records = pending(
            &mut transaction,
            DelayedDeliveryOutboxStreamV1::SchedulerCommand,
            logical_owner_id,
            limit,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(records)
    }

    pub async fn pending_scheduler_receipts(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<Vec<DelayedDeliveryOutboxRecordV1>, DelayedDeliveryPersistenceErrorV1> {
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let records = pending(
            &mut transaction,
            DelayedDeliveryOutboxStreamV1::SchedulerReceipt,
            logical_owner_id,
            limit,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(records)
    }

    pub async fn mark_scheduler_message_published(
        &self,
        stream: DelayedDeliveryOutboxStreamV1,
        logical_owner_id: &str,
        message_id: &[u8; 16],
        envelope_sha256: &[u8; 32],
        published_at_unix_millis: u64,
    ) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id16(message_id)
            || envelope_sha256.iter().all(|byte| *byte == 0)
            || published_at_unix_millis == 0
        {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let published_at = i64::try_from(published_at_unix_millis)
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidInput)?;
        let query = match stream {
            DelayedDeliveryOutboxStreamV1::SchedulerCommand => {
                "UPDATE makosh_data.communication_delayed_delivery_outbox
                 SET published_at_unix_millis =
                   COALESCE(published_at_unix_millis, $4)
                 WHERE logical_owner_id = $1 AND message_id = $2
                   AND envelope_sha256 = $3
                   AND contract_kind = 'scheduler.schedule.command.v1'
                 RETURNING published_at_unix_millis"
            }
            DelayedDeliveryOutboxStreamV1::SchedulerReceipt => {
                "UPDATE makosh_data.communication_delayed_delivery_scheduler_receipt_outbox
                 SET published_at_unix_millis =
                   COALESCE(published_at_unix_millis, $4)
                 WHERE logical_owner_id = $1 AND message_id = $2
                   AND envelope_sha256 = $3
                 RETURNING published_at_unix_millis"
            }
        };
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let row = sqlx::query(query)
            .bind(logical_owner_id)
            .bind(message_id.as_slice())
            .bind(envelope_sha256.as_slice())
            .bind(published_at)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
            .ok_or(DelayedDeliveryPersistenceErrorV1::Conflict)?;
        let recorded: i64 = row
            .try_get("published_at_unix_millis")
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?;
        if recorded <= 0 {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidRow);
        }
        transaction
            .commit()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(())
    }
}

async fn pending(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    stream: DelayedDeliveryOutboxStreamV1,
    logical_owner_id: &str,
    limit: u16,
) -> Result<Vec<DelayedDeliveryOutboxRecordV1>, DelayedDeliveryPersistenceErrorV1> {
    if !valid_owner(logical_owner_id) || !(1..=MAX_RELAY_BATCH_V1).contains(&limit) {
        return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
    }
    let query = match stream {
        DelayedDeliveryOutboxStreamV1::SchedulerCommand => {
            "SELECT delayed_operation_id, message_id, contract_kind,
                    envelope_sha256, envelope_bytes
             FROM makosh_data.communication_delayed_delivery_outbox
             WHERE logical_owner_id = $1
               AND contract_kind = 'scheduler.schedule.command.v1'
               AND published_at_unix_millis IS NULL
             ORDER BY created_at_unix_millis ASC, message_id ASC
             LIMIT $2"
        }
        DelayedDeliveryOutboxStreamV1::SchedulerReceipt => {
            "SELECT delayed_operation_id, message_id,
                    receipt_kind AS contract_kind,
                    envelope_sha256, envelope_bytes
             FROM makosh_data.communication_delayed_delivery_scheduler_receipt_outbox
             WHERE logical_owner_id = $1
               AND published_at_unix_millis IS NULL
             ORDER BY created_at_unix_millis ASC, message_id ASC
             LIMIT $2"
        }
    };
    let rows = sqlx::query(query)
        .bind(logical_owner_id)
        .bind(i64::from(limit))
        .fetch_all(&mut **transaction)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
    rows.iter()
        .map(|row| record_from_row(row, stream))
        .collect()
}

fn record_from_row(
    row: &sqlx::postgres::PgRow,
    stream: DelayedDeliveryOutboxStreamV1,
) -> Result<DelayedDeliveryOutboxRecordV1, DelayedDeliveryPersistenceErrorV1> {
    let delayed_operation_id = id16(
        row.try_get("delayed_operation_id")
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?,
    )?;
    let contract_kind: String = row
        .try_get("contract_kind")
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?;
    let contract_kind = exact_contract_kind(stream, &contract_kind)
        .ok_or(DelayedDeliveryPersistenceErrorV1::InvalidRow)?;
    let message = DelayedDeliveryDurableMessageV1 {
        message_id: id16(
            row.try_get("message_id")
                .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?,
        )?,
        contract_kind,
        envelope_sha256: id32(
            row.try_get("envelope_sha256")
                .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?,
        )?,
        envelope_bytes: row
            .try_get("envelope_bytes")
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?,
    };
    if !valid_durable_message(&message) {
        return Err(DelayedDeliveryPersistenceErrorV1::InvalidRow);
    }
    Ok(DelayedDeliveryOutboxRecordV1 {
        delayed_operation_id,
        message,
    })
}

fn exact_contract_kind(stream: DelayedDeliveryOutboxStreamV1, value: &str) -> Option<&'static str> {
    match (stream, value) {
        (DelayedDeliveryOutboxStreamV1::SchedulerCommand, SCHEDULER_COMMAND_KIND_V1) => {
            Some(SCHEDULER_COMMAND_KIND_V1)
        }
        (DelayedDeliveryOutboxStreamV1::SchedulerReceipt, SCHEDULER_ACCEPTANCE_KIND_V1) => {
            Some(SCHEDULER_ACCEPTANCE_KIND_V1)
        }
        (DelayedDeliveryOutboxStreamV1::SchedulerReceipt, SCHEDULER_RESULT_KIND_V1) => {
            Some(SCHEDULER_RESULT_KIND_V1)
        }
        _ => None,
    }
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], DelayedDeliveryPersistenceErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?;
    valid_id16(&value)
        .then_some(value)
        .ok_or(DelayedDeliveryPersistenceErrorV1::InvalidRow)
}

fn id32(value: Vec<u8>) -> Result<[u8; 32], DelayedDeliveryPersistenceErrorV1> {
    let value: [u8; 32] = value
        .try_into()
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(DelayedDeliveryPersistenceErrorV1::InvalidRow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_contract_inventory_is_exact_per_stream() {
        assert_eq!(
            exact_contract_kind(
                DelayedDeliveryOutboxStreamV1::SchedulerCommand,
                SCHEDULER_COMMAND_KIND_V1,
            ),
            Some(SCHEDULER_COMMAND_KIND_V1)
        );
        assert_eq!(
            exact_contract_kind(
                DelayedDeliveryOutboxStreamV1::SchedulerReceipt,
                SCHEDULER_ACCEPTANCE_KIND_V1,
            ),
            Some(SCHEDULER_ACCEPTANCE_KIND_V1)
        );
        assert_eq!(
            exact_contract_kind(
                DelayedDeliveryOutboxStreamV1::SchedulerCommand,
                "communication.delayed_delivery.status_changed.v1",
            ),
            None
        );
    }
}
