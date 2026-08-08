use sqlx::Row;

use crate::cleanup::{DelayedDeliveryBodyCleanupReasonV1, enqueue_body_cleanup};
use crate::{
    CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryBodyReceiptV1,
    DelayedDeliveryDurableMessageV1, DelayedDeliveryPersistenceErrorV1, SchedulerExecutionFenceV1,
    valid_body_receipt, valid_durable_message, valid_execution_fence, valid_id16,
};

const STATE_SCHEDULED: i16 = 3;
const STATE_DUE: i16 = 4;
const STATE_DISPATCHING: i16 = 5;
const STATE_DELIVERY_ACCEPTED: i16 = 6;
const STATE_CANCEL_REQUESTED: i16 = 7;
const STATE_FAILED: i16 = 9;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimDueExecutionV1 {
    pub logical_owner_id: String,
    pub delayed_operation_id: [u8; 16],
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub fence: SchedulerExecutionFenceV1,
    pub acceptance_receipt: DelayedDeliveryDurableMessageV1,
    pub claimed_at_unix_millis: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryExecutionClaimV1 {
    pub logical_owner_id: String,
    pub delayed_operation_id: [u8; 16],
    pub delivery_operation_id: [u8; 16],
    pub conversation_id: [u8; 16],
    pub reply_to_message_id: Option<[u8; 16]>,
    pub body_receipt: DelayedDeliveryBodyReceiptV1,
    pub fence: SchedulerExecutionFenceV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimDueExecutionOutcomeV1 {
    Claimed(DelayedDeliveryExecutionClaimV1),
    Duplicate(DelayedDeliveryExecutionClaimV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkDeliveryAcceptedV1 {
    pub claim: DelayedDeliveryExecutionClaimV1,
    pub terminal_receipt: DelayedDeliveryDurableMessageV1,
    pub accepted_at_unix_millis: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkDeliveryFailedV1 {
    pub claim: DelayedDeliveryExecutionClaimV1,
    pub error_code: u16,
    pub terminal_receipt: DelayedDeliveryDurableMessageV1,
    pub failed_at_unix_millis: u64,
}

impl CommunicationDelayedDeliveryPersistenceV1 {
    pub async fn claim_due_execution(
        &self,
        command: &ClaimDueExecutionV1,
    ) -> Result<ClaimDueExecutionOutcomeV1, DelayedDeliveryPersistenceErrorV1> {
        if !valid_claim_request(command) {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let claimed_at = signed(command.claimed_at_unix_millis)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        let inserted = insert_due_inbox(&mut transaction, command, claimed_at).await?;
        if !inserted {
            verify_due_inbox(&mut transaction, command).await?;
            let claim = load_claim(
                &mut transaction,
                &command.logical_owner_id,
                &command.delayed_operation_id,
            )
            .await?;
            if claim.fence != command.fence {
                return Err(DelayedDeliveryPersistenceErrorV1::Conflict);
            }
            transaction
                .commit()
                .await
                .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
            return Ok(ClaimDueExecutionOutcomeV1::Duplicate(claim));
        }
        transition_to_due(&mut transaction, command, claimed_at).await?;
        transition_to_dispatching(&mut transaction, command, claimed_at).await?;
        crate::realtime::insert_operation_transition(
            &mut transaction,
            &command.logical_owner_id,
            &command.delayed_operation_id,
            claimed_at,
        )
        .await?;
        insert_receipt_outbox(
            &mut transaction,
            &command.logical_owner_id,
            &command.delayed_operation_id,
            &command.acceptance_receipt,
            claimed_at,
        )
        .await?;
        let claim = load_claim(
            &mut transaction,
            &command.logical_owner_id,
            &command.delayed_operation_id,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(ClaimDueExecutionOutcomeV1::Claimed(claim))
    }

    pub async fn mark_delivery_accepted(
        &self,
        command: &MarkDeliveryAcceptedV1,
    ) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
        if !valid_claim(&command.claim)
            || command.accepted_at_unix_millis == 0
            || !valid_receipt(&command.terminal_receipt, "scheduler.job_run.result.v1")
            || command.claim.fence.lease_expires_at_unix_millis <= command.accepted_at_unix_millis
        {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let accepted_at = signed(command.accepted_at_unix_millis)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        let affected = sqlx::query(
            "UPDATE makosh_data.communication_delayed_delivery_operations
             SET state = $3, state_revision = state_revision + 1,
                 updated_at_unix_millis = $4
             WHERE logical_owner_id = $1 AND delayed_operation_id = $2
               AND delivery_operation_id = $5 AND state = $6
               AND scheduler_run_id = $7
               AND scheduler_schedule_revision = $8
               AND scheduler_lease_epoch = $9
               AND scheduler_lease_expires_at_unix_millis > $4",
        )
        .bind(&command.claim.logical_owner_id)
        .bind(command.claim.delayed_operation_id.as_slice())
        .bind(STATE_DELIVERY_ACCEPTED)
        .bind(accepted_at)
        .bind(command.claim.delivery_operation_id.as_slice())
        .bind(STATE_DISPATCHING)
        .bind(command.claim.fence.run_id.as_slice())
        .bind(signed(command.claim.fence.schedule_revision)?)
        .bind(signed(command.claim.fence.lease_epoch)?)
        .execute(&mut *transaction)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if affected != 1 {
            return Err(DelayedDeliveryPersistenceErrorV1::ClaimLost);
        }
        enqueue_body_cleanup(
            &mut transaction,
            &command.claim.logical_owner_id,
            &command.claim.delayed_operation_id,
            DelayedDeliveryBodyCleanupReasonV1::DeliveryAccepted,
            accepted_at,
        )
        .await?;
        crate::realtime::insert_operation_transition(
            &mut transaction,
            &command.claim.logical_owner_id,
            &command.claim.delayed_operation_id,
            accepted_at,
        )
        .await?;
        insert_receipt_outbox(
            &mut transaction,
            &command.claim.logical_owner_id,
            &command.claim.delayed_operation_id,
            &command.terminal_receipt,
            accepted_at,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)
    }

    pub async fn mark_delivery_failed(
        &self,
        command: &MarkDeliveryFailedV1,
    ) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
        if !valid_claim(&command.claim)
            || !(1..=7).contains(&command.error_code)
            || command.failed_at_unix_millis == 0
            || !valid_receipt(&command.terminal_receipt, "scheduler.job_run.result.v1")
            || command.claim.fence.lease_expires_at_unix_millis <= command.failed_at_unix_millis
        {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let failed_at = signed(command.failed_at_unix_millis)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        let affected = sqlx::query(
            "UPDATE makosh_data.communication_delayed_delivery_operations
             SET state = $3, state_revision = state_revision + 1,
                 error_code = $4, updated_at_unix_millis = $5
             WHERE logical_owner_id = $1 AND delayed_operation_id = $2
               AND delivery_operation_id = $6 AND state = $7
               AND scheduler_run_id = $8
               AND scheduler_schedule_revision = $9
               AND scheduler_lease_epoch = $10
               AND scheduler_lease_expires_at_unix_millis > $5",
        )
        .bind(&command.claim.logical_owner_id)
        .bind(command.claim.delayed_operation_id.as_slice())
        .bind(STATE_FAILED)
        .bind(
            i16::try_from(command.error_code)
                .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidInput)?,
        )
        .bind(failed_at)
        .bind(command.claim.delivery_operation_id.as_slice())
        .bind(STATE_DISPATCHING)
        .bind(command.claim.fence.run_id.as_slice())
        .bind(signed(command.claim.fence.schedule_revision)?)
        .bind(signed(command.claim.fence.lease_epoch)?)
        .execute(&mut *transaction)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if affected != 1 {
            return Err(DelayedDeliveryPersistenceErrorV1::ClaimLost);
        }
        enqueue_body_cleanup(
            &mut transaction,
            &command.claim.logical_owner_id,
            &command.claim.delayed_operation_id,
            DelayedDeliveryBodyCleanupReasonV1::DeliveryRejected,
            failed_at,
        )
        .await?;
        crate::realtime::insert_operation_transition(
            &mut transaction,
            &command.claim.logical_owner_id,
            &command.claim.delayed_operation_id,
            failed_at,
        )
        .await?;
        insert_receipt_outbox(
            &mut transaction,
            &command.claim.logical_owner_id,
            &command.claim.delayed_operation_id,
            &command.terminal_receipt,
            failed_at,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)
    }
}

async fn transition_to_due(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    command: &ClaimDueExecutionV1,
    claimed_at: i64,
) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
    let affected = sqlx::query(
        "UPDATE makosh_data.communication_delayed_delivery_operations
         SET state = $3, state_revision = state_revision + 1,
             updated_at_unix_millis = $4
         WHERE logical_owner_id = $1 AND delayed_operation_id = $2
           AND state IN ($5, $6) AND scheduler_schedule_revision = $7",
    )
    .bind(&command.logical_owner_id)
    .bind(command.delayed_operation_id.as_slice())
    .bind(STATE_DUE)
    .bind(claimed_at)
    .bind(STATE_SCHEDULED)
    .bind(STATE_CANCEL_REQUESTED)
    .bind(signed(command.fence.schedule_revision)?)
    .execute(&mut **transaction)
    .await
    .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
    .rows_affected();
    exact_mutation(affected)
}

async fn transition_to_dispatching(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    command: &ClaimDueExecutionV1,
    claimed_at: i64,
) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
    let affected = sqlx::query(
        "UPDATE makosh_data.communication_delayed_delivery_operations
         SET state = $3, state_revision = state_revision + 1,
             scheduler_run_id = $4, scheduler_lease_epoch = $5,
             scheduler_lease_expires_at_unix_millis = $6,
             updated_at_unix_millis = $7
         WHERE logical_owner_id = $1 AND delayed_operation_id = $2
           AND state = $8",
    )
    .bind(&command.logical_owner_id)
    .bind(command.delayed_operation_id.as_slice())
    .bind(STATE_DISPATCHING)
    .bind(command.fence.run_id.as_slice())
    .bind(signed(command.fence.lease_epoch)?)
    .bind(signed(command.fence.lease_expires_at_unix_millis)?)
    .bind(claimed_at)
    .bind(STATE_DUE)
    .execute(&mut **transaction)
    .await
    .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
    .rows_affected();
    exact_mutation(affected)
}

async fn insert_due_inbox(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    command: &ClaimDueExecutionV1,
    claimed_at: i64,
) -> Result<bool, DelayedDeliveryPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.communication_delayed_delivery_scheduler_inbox (
           logical_owner_id, message_id, envelope_sha256,
           delayed_operation_id, received_at_unix_millis
         ) VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (logical_owner_id, message_id) DO NOTHING",
    )
    .bind(&command.logical_owner_id)
    .bind(command.command_message_id.as_slice())
    .bind(command.command_envelope_sha256.as_slice())
    .bind(command.delayed_operation_id.as_slice())
    .bind(claimed_at)
    .execute(&mut **transaction)
    .await
    .map(|result| result.rows_affected() == 1)
    .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)
}

async fn verify_due_inbox(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    command: &ClaimDueExecutionV1,
) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT envelope_sha256, delayed_operation_id
         FROM makosh_data.communication_delayed_delivery_scheduler_inbox
         WHERE logical_owner_id = $1 AND message_id = $2",
    )
    .bind(&command.logical_owner_id)
    .bind(command.command_message_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_error)?;
    let hash: Vec<u8> = row.try_get("envelope_sha256").map_err(row_error)?;
    let operation_id: Vec<u8> = row.try_get("delayed_operation_id").map_err(row_error)?;
    if hash.as_slice() != command.command_envelope_sha256
        || operation_id.as_slice() != command.delayed_operation_id
    {
        return Err(DelayedDeliveryPersistenceErrorV1::Conflict);
    }
    Ok(())
}

async fn insert_receipt_outbox(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    delayed_operation_id: &[u8; 16],
    receipt: &DelayedDeliveryDurableMessageV1,
    created_at: i64,
) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.communication_delayed_delivery_scheduler_receipt_outbox (
           logical_owner_id, message_id, delayed_operation_id, receipt_kind,
           envelope_sha256, envelope_bytes, created_at_unix_millis
         ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(logical_owner_id)
    .bind(receipt.message_id.as_slice())
    .bind(delayed_operation_id.as_slice())
    .bind(receipt.contract_kind)
    .bind(receipt.envelope_sha256.as_slice())
    .bind(&receipt.envelope_bytes)
    .bind(created_at)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(storage_error)
}

async fn load_claim(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    delayed_operation_id: &[u8; 16],
) -> Result<DelayedDeliveryExecutionClaimV1, DelayedDeliveryPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT delivery_operation_id, canonical_conversation_id,
                canonical_reply_message_id, body_reference_id,
                body_declared_bytes, body_sha256, body_custody_proof,
                scheduler_run_id, scheduler_schedule_revision,
                scheduler_lease_epoch, scheduler_lease_expires_at_unix_millis,
                state
         FROM makosh_data.communication_delayed_delivery_operations
         WHERE logical_owner_id = $1 AND delayed_operation_id = $2",
    )
    .bind(logical_owner_id)
    .bind(delayed_operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?
    .ok_or(DelayedDeliveryPersistenceErrorV1::NotFound)?;
    let state: i16 = row.try_get("state").map_err(row_error)?;
    if !matches!(state, STATE_DISPATCHING | STATE_DELIVERY_ACCEPTED) {
        return Err(DelayedDeliveryPersistenceErrorV1::Conflict);
    }
    let claim = DelayedDeliveryExecutionClaimV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        delayed_operation_id: *delayed_operation_id,
        delivery_operation_id: id16(row.try_get("delivery_operation_id").map_err(row_error)?)?,
        conversation_id: id16(
            row.try_get("canonical_conversation_id")
                .map_err(row_error)?,
        )?,
        reply_to_message_id: optional_id16(
            row.try_get("canonical_reply_message_id")
                .map_err(row_error)?,
        )?,
        body_receipt: DelayedDeliveryBodyReceiptV1 {
            reference_id: id16(row.try_get("body_reference_id").map_err(row_error)?)?,
            declared_bytes: positive_u64(row.try_get("body_declared_bytes").map_err(row_error)?)?,
            sha256: id32(row.try_get("body_sha256").map_err(row_error)?)?,
            custody_proof: row.try_get("body_custody_proof").map_err(row_error)?,
        },
        fence: SchedulerExecutionFenceV1 {
            run_id: id16(row.try_get("scheduler_run_id").map_err(row_error)?)?,
            schedule_revision: positive_u64(
                row.try_get("scheduler_schedule_revision")
                    .map_err(row_error)?,
            )?,
            lease_epoch: positive_u64(row.try_get("scheduler_lease_epoch").map_err(row_error)?)?,
            lease_expires_at_unix_millis: positive_u64(
                row.try_get("scheduler_lease_expires_at_unix_millis")
                    .map_err(row_error)?,
            )?,
        },
    };
    valid_claim(&claim)
        .then_some(claim)
        .ok_or(DelayedDeliveryPersistenceErrorV1::InvalidRow)
}

fn valid_claim_request(command: &ClaimDueExecutionV1) -> bool {
    valid_owner(&command.logical_owner_id)
        && valid_id16(&command.delayed_operation_id)
        && valid_id16(&command.command_message_id)
        && command
            .command_envelope_sha256
            .iter()
            .any(|byte| *byte != 0)
        && command.claimed_at_unix_millis > 0
        && valid_execution_fence(&command.fence, command.claimed_at_unix_millis)
        && valid_receipt(
            &command.acceptance_receipt,
            "scheduler.job_run.acceptance.v1",
        )
}

fn valid_claim(claim: &DelayedDeliveryExecutionClaimV1) -> bool {
    valid_owner(&claim.logical_owner_id)
        && valid_id16(&claim.delayed_operation_id)
        && valid_id16(&claim.delivery_operation_id)
        && valid_id16(&claim.conversation_id)
        && claim.reply_to_message_id.as_ref().is_none_or(valid_id16)
        && valid_body_receipt(&claim.body_receipt)
        && valid_execution_fence(&claim.fence, 0)
}

fn valid_receipt(message: &DelayedDeliveryDurableMessageV1, expected: &str) -> bool {
    message.contract_kind == expected && valid_durable_message(message)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

fn exact_mutation(rows: u64) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
    (rows == 1)
        .then_some(())
        .ok_or(DelayedDeliveryPersistenceErrorV1::Conflict)
}

fn signed(value: u64) -> Result<i64, DelayedDeliveryPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidInput)
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
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)
}

fn optional_id16(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; 16]>, DelayedDeliveryPersistenceErrorV1> {
    value.map(id16).transpose()
}

fn id32(value: Vec<u8>) -> Result<[u8; 32], DelayedDeliveryPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)
}

fn storage_error(_: sqlx::Error) -> DelayedDeliveryPersistenceErrorV1 {
    DelayedDeliveryPersistenceErrorV1::StorageUnavailable
}

fn row_error(_: sqlx::Error) -> DelayedDeliveryPersistenceErrorV1 {
    DelayedDeliveryPersistenceErrorV1::InvalidRow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_or_expired_claim_is_rejected() {
        let request = ClaimDueExecutionV1 {
            logical_owner_id: "owner".to_owned(),
            delayed_operation_id: [1; 16],
            command_message_id: [2; 16],
            command_envelope_sha256: [3; 32],
            fence: SchedulerExecutionFenceV1 {
                run_id: [4; 16],
                schedule_revision: 5,
                lease_epoch: 6,
                lease_expires_at_unix_millis: 10_000,
            },
            acceptance_receipt: DelayedDeliveryDurableMessageV1 {
                message_id: [7; 16],
                contract_kind: "scheduler.job_run.acceptance.v1",
                envelope_sha256: [8; 32],
                envelope_bytes: vec![9; 64],
            },
            claimed_at_unix_millis: 9_999,
        };
        assert!(valid_claim_request(&request));
        let mut expired = request;
        expired.claimed_at_unix_millis = 10_000;
        assert!(!valid_claim_request(&expired));
    }
}
