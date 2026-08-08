use makosh_communication_delayed_delivery_core::{
    DelayedDeliveryOperationV1, DelayedDeliveryStateV1,
};
use sha2::{Digest, Sha256};
use sqlx::Row;

use crate::cleanup::{DelayedDeliveryBodyCleanupReasonV1, enqueue_body_cleanup};
use crate::{
    CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryBodyReceiptV1,
    DelayedDeliveryDurableMessageV1, DelayedDeliveryPersistenceErrorV1, valid_body_receipt,
    valid_durable_message,
};

const STATE_SCHEDULE_PENDING: i16 = 2;
const STATE_SCHEDULED: i16 = 3;
const STATE_DISPATCHING: i16 = 5;
const STATE_DELIVERY_ACCEPTED: i16 = 6;
const STATE_CANCEL_REQUESTED: i16 = 7;
const STATE_CANCELLED: i16 = 8;
const STATE_FAILED: i16 = 9;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateDelayedDeliveryOperationV1 {
    pub logical_owner_id: String,
    pub operation: DelayedDeliveryOperationV1,
    pub body_receipt: DelayedDeliveryBodyReceiptV1,
    pub scheduler_command: DelayedDeliveryDurableMessageV1,
    pub created_at_unix_millis: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CreateDelayedDeliveryOperationOutcomeV1 {
    Created { state_revision: u64 },
    Existing { state_revision: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerScheduleResultV1 {
    Ensured { schedule_revision: u64 },
    Cancelled,
    TooLate,
    Rejected { error_code: u16 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplySchedulerResultV1 {
    pub logical_owner_id: String,
    pub delayed_operation_id: [u8; 16],
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub result: SchedulerScheduleResultV1,
    pub received_at_unix_millis: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplySchedulerResultOutcomeV1 {
    Applied(DelayedDeliveryOperationStatusV1),
    Duplicate(DelayedDeliveryOperationStatusV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestDelayedDeliveryCancellationV1 {
    pub logical_owner_id: String,
    pub delayed_operation_id: [u8; 16],
    pub expected_revision: u64,
    pub scheduler_command: DelayedDeliveryDurableMessageV1,
    pub requested_at_unix_millis: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryOperationStatusV1 {
    pub delayed_operation_id: [u8; 16],
    pub delivery_operation_id: [u8; 16],
    pub state: DelayedDeliveryStateV1,
    pub state_revision: u64,
    pub deliver_at_unix_millis: u64,
    pub scheduler_schedule_revision: Option<u64>,
    pub error_code: Option<u16>,
    pub created_at_unix_millis: u64,
    pub updated_at_unix_millis: u64,
}

impl CommunicationDelayedDeliveryPersistenceV1 {
    pub async fn create_operation(
        &self,
        command: &CreateDelayedDeliveryOperationV1,
    ) -> Result<CreateDelayedDeliveryOperationOutcomeV1, DelayedDeliveryPersistenceErrorV1> {
        if !valid_owner(&command.logical_owner_id)
            || !valid_body_receipt(&command.body_receipt)
            || !valid_scheduler_command(&command.scheduler_command)
            || command.created_at_unix_millis == 0
        {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let created_at = signed(command.created_at_unix_millis)?;
        let deliver_at = signed(command.operation.deliver_at_unix_millis())?;
        let declared_bytes = signed(command.body_receipt.declared_bytes)?;
        let fingerprint = operation_fingerprint(command);
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.communication_delayed_delivery_operations (
               logical_owner_id, delayed_operation_id, delivery_operation_id,
               canonical_conversation_id, canonical_reply_message_id,
               request_fingerprint, body_reference_id, body_declared_bytes,
               body_sha256, body_custody_proof, deliver_at_unix_millis,
               state, state_revision, created_at_unix_millis, updated_at_unix_millis
             ) VALUES (
               $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 1, $13, $13
             ) ON CONFLICT (logical_owner_id, delayed_operation_id) DO NOTHING",
        )
        .bind(&command.logical_owner_id)
        .bind(command.operation.delayed_operation_id().as_slice())
        .bind(command.operation.delivery_operation_id().as_slice())
        .bind(command.operation.conversation_id().as_slice())
        .bind(
            command
                .operation
                .reply_to_message_id()
                .map(|value| value.to_vec()),
        )
        .bind(fingerprint.as_slice())
        .bind(command.body_receipt.reference_id.as_slice())
        .bind(declared_bytes)
        .bind(command.body_receipt.sha256.as_slice())
        .bind(&command.body_receipt.custody_proof)
        .bind(deliver_at)
        .bind(STATE_SCHEDULE_PENDING)
        .bind(created_at)
        .execute(&mut *transaction)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
        .rows_affected()
            == 1;
        if inserted {
            crate::realtime::insert_operation_transition(
                &mut transaction,
                &command.logical_owner_id,
                command.operation.delayed_operation_id(),
                created_at,
            )
            .await?;
            insert_outbox(
                &mut transaction,
                &command.logical_owner_id,
                command.operation.delayed_operation_id(),
                &command.scheduler_command,
                created_at,
            )
            .await?;
        } else {
            verify_existing_outbox(
                &mut transaction,
                &command.logical_owner_id,
                command.operation.delayed_operation_id(),
                &command.scheduler_command,
            )
            .await?;
        }
        let row = sqlx::query(
            "SELECT request_fingerprint, state_revision
             FROM makosh_data.communication_delayed_delivery_operations
             WHERE logical_owner_id = $1 AND delayed_operation_id = $2",
        )
        .bind(&command.logical_owner_id)
        .bind(command.operation.delayed_operation_id().as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        let stored_fingerprint: Vec<u8> = row
            .try_get("request_fingerprint")
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?;
        if stored_fingerprint.as_slice() != fingerprint {
            return Err(DelayedDeliveryPersistenceErrorV1::Conflict);
        }
        let state_revision = positive_u64(
            row.try_get("state_revision")
                .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?,
        )?;
        transaction
            .commit()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(if inserted {
            CreateDelayedDeliveryOperationOutcomeV1::Created { state_revision }
        } else {
            CreateDelayedDeliveryOperationOutcomeV1::Existing { state_revision }
        })
    }

    pub async fn request_cancellation(
        &self,
        command: &RequestDelayedDeliveryCancellationV1,
    ) -> Result<DelayedDeliveryOperationStatusV1, DelayedDeliveryPersistenceErrorV1> {
        if !valid_owner(&command.logical_owner_id)
            || command.delayed_operation_id.iter().all(|byte| *byte == 0)
            || command.expected_revision == 0
            || command.requested_at_unix_millis == 0
            || !valid_scheduler_command(&command.scheduler_command)
        {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let requested_at = signed(command.requested_at_unix_millis)?;
        let expected_revision = signed(command.expected_revision)?;
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
               AND state_revision = $5 AND state IN ($6, $7)",
        )
        .bind(&command.logical_owner_id)
        .bind(command.delayed_operation_id.as_slice())
        .bind(STATE_CANCEL_REQUESTED)
        .bind(requested_at)
        .bind(expected_revision)
        .bind(STATE_SCHEDULE_PENDING)
        .bind(STATE_SCHEDULED)
        .execute(&mut *transaction)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if affected != 1 {
            let status = status_in_transaction(
                &mut transaction,
                &command.logical_owner_id,
                &command.delayed_operation_id,
            )
            .await?;
            if status.state == DelayedDeliveryStateV1::CancelRequested
                && status.state_revision == command.expected_revision.saturating_add(1)
            {
                verify_existing_outbox(
                    &mut transaction,
                    &command.logical_owner_id,
                    &command.delayed_operation_id,
                    &command.scheduler_command,
                )
                .await?;
                transaction
                    .commit()
                    .await
                    .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
                return Ok(status);
            }
            return Err(DelayedDeliveryPersistenceErrorV1::StaleRevision);
        }
        crate::realtime::insert_operation_transition(
            &mut transaction,
            &command.logical_owner_id,
            &command.delayed_operation_id,
            requested_at,
        )
        .await?;
        insert_outbox(
            &mut transaction,
            &command.logical_owner_id,
            &command.delayed_operation_id,
            &command.scheduler_command,
            requested_at,
        )
        .await?;
        let status = status_in_transaction(
            &mut transaction,
            &command.logical_owner_id,
            &command.delayed_operation_id,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(status)
    }

    pub async fn apply_scheduler_result(
        &self,
        command: &ApplySchedulerResultV1,
    ) -> Result<ApplySchedulerResultOutcomeV1, DelayedDeliveryPersistenceErrorV1> {
        if !valid_scheduler_result(command) {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let received_at = signed(command.received_at_unix_millis)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.communication_delayed_delivery_scheduler_inbox (
               logical_owner_id, message_id, envelope_sha256,
               delayed_operation_id, received_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (logical_owner_id, message_id) DO NOTHING",
        )
        .bind(&command.logical_owner_id)
        .bind(command.message_id.as_slice())
        .bind(command.envelope_sha256.as_slice())
        .bind(command.delayed_operation_id.as_slice())
        .bind(received_at)
        .execute(&mut *transaction)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
        .rows_affected()
            == 1;
        if !inserted {
            verify_duplicate_inbox(&mut transaction, command).await?;
            let status = status_in_transaction(
                &mut transaction,
                &command.logical_owner_id,
                &command.delayed_operation_id,
            )
            .await?;
            transaction
                .commit()
                .await
                .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
            return Ok(ApplySchedulerResultOutcomeV1::Duplicate(status));
        }
        apply_scheduler_transition(&mut transaction, command, received_at).await?;
        crate::realtime::insert_operation_transition(
            &mut transaction,
            &command.logical_owner_id,
            &command.delayed_operation_id,
            received_at,
        )
        .await?;
        let status = status_in_transaction(
            &mut transaction,
            &command.logical_owner_id,
            &command.delayed_operation_id,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        Ok(ApplySchedulerResultOutcomeV1::Applied(status))
    }
}

async fn insert_outbox(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    delayed_operation_id: &[u8; 16],
    message: &DelayedDeliveryDurableMessageV1,
    created_at: i64,
) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.communication_delayed_delivery_outbox (
           logical_owner_id, message_id, delayed_operation_id, contract_kind,
           envelope_sha256, envelope_bytes, created_at_unix_millis
         ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(logical_owner_id)
    .bind(message.message_id.as_slice())
    .bind(delayed_operation_id.as_slice())
    .bind(message.contract_kind)
    .bind(message.envelope_sha256.as_slice())
    .bind(&message.envelope_bytes)
    .bind(created_at)
    .execute(&mut **transaction)
    .await
    .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
    Ok(())
}

async fn verify_duplicate_inbox(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    command: &ApplySchedulerResultV1,
) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT envelope_sha256, delayed_operation_id
         FROM makosh_data.communication_delayed_delivery_scheduler_inbox
         WHERE logical_owner_id = $1 AND message_id = $2",
    )
    .bind(&command.logical_owner_id)
    .bind(command.message_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
    let hash: Vec<u8> = row
        .try_get("envelope_sha256")
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?;
    let operation_id: Vec<u8> = row
        .try_get("delayed_operation_id")
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?;
    if hash.as_slice() != command.envelope_sha256
        || operation_id.as_slice() != command.delayed_operation_id
    {
        return Err(DelayedDeliveryPersistenceErrorV1::Conflict);
    }
    Ok(())
}

async fn verify_existing_outbox(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    delayed_operation_id: &[u8; 16],
    message: &DelayedDeliveryDurableMessageV1,
) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT message_id, envelope_sha256, envelope_bytes
         FROM makosh_data.communication_delayed_delivery_outbox
         WHERE logical_owner_id = $1 AND delayed_operation_id = $2
           AND message_id = $3
           AND contract_kind = 'scheduler.schedule.command.v1'",
    )
    .bind(logical_owner_id)
    .bind(delayed_operation_id.as_slice())
    .bind(message.message_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
    let message_id: Vec<u8> = row.try_get("message_id").map_err(row_error)?;
    let hash: Vec<u8> = row.try_get("envelope_sha256").map_err(row_error)?;
    let bytes: Vec<u8> = row.try_get("envelope_bytes").map_err(row_error)?;
    if message_id.as_slice() != message.message_id
        || hash.as_slice() != message.envelope_sha256
        || bytes != message.envelope_bytes
    {
        return Err(DelayedDeliveryPersistenceErrorV1::Conflict);
    }
    Ok(())
}

async fn apply_scheduler_transition(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    command: &ApplySchedulerResultV1,
    received_at: i64,
) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
    if let SchedulerScheduleResultV1::Ensured { schedule_revision } = command.result {
        let affected = sqlx::query(
            "UPDATE makosh_data.communication_delayed_delivery_operations
             SET state = CASE WHEN state = $3 THEN $4 ELSE state END,
                 state_revision = state_revision + 1,
                 scheduler_schedule_revision = $5,
                 error_code = NULL, updated_at_unix_millis = $6
             WHERE logical_owner_id = $1 AND delayed_operation_id = $2
               AND state IN ($3, $7)",
        )
        .bind(&command.logical_owner_id)
        .bind(command.delayed_operation_id.as_slice())
        .bind(STATE_SCHEDULE_PENDING)
        .bind(STATE_SCHEDULED)
        .bind(signed(schedule_revision)?)
        .bind(received_at)
        .bind(STATE_CANCEL_REQUESTED)
        .execute(&mut **transaction)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if affected != 1 {
            return Err(DelayedDeliveryPersistenceErrorV1::Conflict);
        }
        return Ok(());
    }
    if command.result == SchedulerScheduleResultV1::TooLate {
        let affected = sqlx::query(
            "UPDATE makosh_data.communication_delayed_delivery_operations
             SET state = CASE WHEN state = $3 THEN $4 ELSE state END,
                 state_revision = state_revision + 1,
                 updated_at_unix_millis = $5
             WHERE logical_owner_id = $1 AND delayed_operation_id = $2
               AND state IN ($3, $6, $7)",
        )
        .bind(&command.logical_owner_id)
        .bind(command.delayed_operation_id.as_slice())
        .bind(STATE_CANCEL_REQUESTED)
        .bind(STATE_SCHEDULED)
        .bind(received_at)
        .bind(STATE_DISPATCHING)
        .bind(STATE_DELIVERY_ACCEPTED)
        .execute(&mut **transaction)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
        .rows_affected();
        if affected != 1 {
            return Err(DelayedDeliveryPersistenceErrorV1::Conflict);
        }
        return Ok(());
    }
    let (next_state, schedule_revision, error_code, allowed_states): (
        i16,
        Option<i64>,
        Option<i16>,
        &[i16],
    ) = match command.result {
        SchedulerScheduleResultV1::Ensured { .. } => unreachable!("handled above"),
        SchedulerScheduleResultV1::Cancelled => {
            (STATE_CANCELLED, None, None, &[STATE_CANCEL_REQUESTED])
        }
        SchedulerScheduleResultV1::TooLate => unreachable!("handled above"),
        SchedulerScheduleResultV1::Rejected { error_code } => (
            STATE_FAILED,
            None,
            Some(
                i16::try_from(error_code)
                    .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidInput)?,
            ),
            &[STATE_SCHEDULE_PENDING, STATE_CANCEL_REQUESTED],
        ),
    };
    let affected = sqlx::query(
        "UPDATE makosh_data.communication_delayed_delivery_operations
         SET state = $3, state_revision = state_revision + 1,
             scheduler_schedule_revision = COALESCE($4, scheduler_schedule_revision),
             error_code = $5, updated_at_unix_millis = $6
         WHERE logical_owner_id = $1 AND delayed_operation_id = $2
           AND state = ANY($7)",
    )
    .bind(&command.logical_owner_id)
    .bind(command.delayed_operation_id.as_slice())
    .bind(next_state)
    .bind(schedule_revision)
    .bind(error_code)
    .bind(received_at)
    .bind(allowed_states)
    .execute(&mut **transaction)
    .await
    .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
    .rows_affected();
    if affected != 1 {
        return Err(DelayedDeliveryPersistenceErrorV1::Conflict);
    }
    let cleanup_reason = match command.result {
        SchedulerScheduleResultV1::Cancelled => {
            Some(DelayedDeliveryBodyCleanupReasonV1::DeliveryCancelled)
        }
        SchedulerScheduleResultV1::Rejected { .. } => {
            Some(DelayedDeliveryBodyCleanupReasonV1::DeliveryRejected)
        }
        SchedulerScheduleResultV1::Ensured { .. } | SchedulerScheduleResultV1::TooLate => None,
    };
    if let Some(reason) = cleanup_reason {
        enqueue_body_cleanup(
            transaction,
            &command.logical_owner_id,
            &command.delayed_operation_id,
            reason,
            received_at,
        )
        .await?;
    }
    Ok(())
}

async fn status_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    delayed_operation_id: &[u8; 16],
) -> Result<DelayedDeliveryOperationStatusV1, DelayedDeliveryPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT delayed_operation_id, delivery_operation_id, state, state_revision,
                deliver_at_unix_millis, scheduler_schedule_revision, error_code,
                created_at_unix_millis, updated_at_unix_millis
         FROM makosh_data.communication_delayed_delivery_operations
         WHERE logical_owner_id = $1 AND delayed_operation_id = $2",
    )
    .bind(logical_owner_id)
    .bind(delayed_operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
    .ok_or(DelayedDeliveryPersistenceErrorV1::NotFound)?;
    status_from_row(&row)
}

pub(crate) fn status_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<DelayedDeliveryOperationStatusV1, DelayedDeliveryPersistenceErrorV1> {
    Ok(DelayedDeliveryOperationStatusV1 {
        delayed_operation_id: id16(row.try_get("delayed_operation_id").map_err(row_error)?)?,
        delivery_operation_id: id16(row.try_get("delivery_operation_id").map_err(row_error)?)?,
        state: state_from_code(row.try_get("state").map_err(row_error)?)?,
        state_revision: positive_u64(row.try_get("state_revision").map_err(row_error)?)?,
        deliver_at_unix_millis: positive_u64(
            row.try_get("deliver_at_unix_millis").map_err(row_error)?,
        )?,
        scheduler_schedule_revision: optional_positive_u64(
            row.try_get("scheduler_schedule_revision")
                .map_err(row_error)?,
        )?,
        error_code: row
            .try_get::<Option<i16>, _>("error_code")
            .map_err(row_error)?
            .map(|value| {
                u16::try_from(value).map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)
            })
            .transpose()?,
        created_at_unix_millis: positive_u64(
            row.try_get("created_at_unix_millis").map_err(row_error)?,
        )?,
        updated_at_unix_millis: positive_u64(
            row.try_get("updated_at_unix_millis").map_err(row_error)?,
        )?,
    })
}

fn operation_fingerprint(command: &CreateDelayedDeliveryOperationV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(command.operation.delayed_operation_id());
    hash.update(command.operation.delivery_operation_id());
    hash.update(command.operation.conversation_id());
    if let Some(reply_id) = command.operation.reply_to_message_id() {
        hash.update([1]);
        hash.update(reply_id);
    } else {
        hash.update([0]);
    }
    hash.update(command.operation.deliver_at_unix_millis().to_be_bytes());
    hash.update(command.body_receipt.reference_id);
    hash.update(command.body_receipt.declared_bytes.to_be_bytes());
    hash.update(command.body_receipt.sha256);
    hash.update(&command.body_receipt.custody_proof);
    hash.finalize().into()
}

fn valid_scheduler_command(message: &DelayedDeliveryDurableMessageV1) -> bool {
    message.contract_kind == "scheduler.schedule.command.v1" && valid_durable_message(message)
}

fn valid_scheduler_result(command: &ApplySchedulerResultV1) -> bool {
    valid_owner(&command.logical_owner_id)
        && command.delayed_operation_id.iter().any(|byte| *byte != 0)
        && command.message_id.iter().any(|byte| *byte != 0)
        && command.envelope_sha256.iter().any(|byte| *byte != 0)
        && command.received_at_unix_millis > 0
        && match command.result {
            SchedulerScheduleResultV1::Ensured { schedule_revision } => schedule_revision > 0,
            SchedulerScheduleResultV1::Rejected { error_code } => (1..=7).contains(&error_code),
            SchedulerScheduleResultV1::Cancelled | SchedulerScheduleResultV1::TooLate => true,
        }
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
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

fn optional_positive_u64(
    value: Option<i64>,
) -> Result<Option<u64>, DelayedDeliveryPersistenceErrorV1> {
    value.map(positive_u64).transpose()
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], DelayedDeliveryPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)
}

pub(crate) fn state_from_code(
    value: i16,
) -> Result<DelayedDeliveryStateV1, DelayedDeliveryPersistenceErrorV1> {
    match value {
        1 => Ok(DelayedDeliveryStateV1::Accepted),
        2 => Ok(DelayedDeliveryStateV1::SchedulePending),
        3 => Ok(DelayedDeliveryStateV1::Scheduled),
        4 => Ok(DelayedDeliveryStateV1::Due),
        5 => Ok(DelayedDeliveryStateV1::Dispatching),
        6 => Ok(DelayedDeliveryStateV1::DeliveryAccepted),
        7 => Ok(DelayedDeliveryStateV1::CancelRequested),
        8 => Ok(DelayedDeliveryStateV1::Cancelled),
        9 => Ok(DelayedDeliveryStateV1::Failed),
        _ => Err(DelayedDeliveryPersistenceErrorV1::InvalidRow),
    }
}

fn row_error(_: sqlx::Error) -> DelayedDeliveryPersistenceErrorV1 {
    DelayedDeliveryPersistenceErrorV1::InvalidRow
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_communication_delayed_delivery_core::{
        DelayedDeliveryDraftV1, prepare_delayed_delivery_v1,
    };

    fn command() -> CreateDelayedDeliveryOperationV1 {
        CreateDelayedDeliveryOperationV1 {
            logical_owner_id: "owner".to_owned(),
            operation: prepare_delayed_delivery_v1(
                DelayedDeliveryDraftV1 {
                    delayed_operation_id: [1; 16],
                    delivery_operation_id: [2; 16],
                    conversation_id: [3; 16],
                    reply_to_message_id: None,
                    body_utf8: b"private body".to_vec(),
                    deliver_at_unix_millis: 10_000,
                },
                1_000,
            )
            .expect("valid operation"),
            body_receipt: DelayedDeliveryBodyReceiptV1 {
                reference_id: [4; 16],
                declared_bytes: 12,
                sha256: [5; 32],
                custody_proof: vec![6; 32],
            },
            scheduler_command: DelayedDeliveryDurableMessageV1 {
                message_id: [7; 16],
                contract_kind: "scheduler.schedule.command.v1",
                envelope_sha256: [8; 32],
                envelope_bytes: vec![9; 64],
            },
            created_at_unix_millis: 1_000,
        }
    }

    #[test]
    fn operation_fingerprint_is_stable_and_covers_custody() {
        let first = command();
        let mut second = first.clone();
        assert_eq!(
            operation_fingerprint(&first),
            operation_fingerprint(&second)
        );
        second.body_receipt.reference_id = [10; 16];
        assert_ne!(
            operation_fingerprint(&first),
            operation_fingerprint(&second)
        );
    }

    #[test]
    fn scheduler_result_validation_is_exact() {
        let mut result = ApplySchedulerResultV1 {
            logical_owner_id: "owner".to_owned(),
            delayed_operation_id: [1; 16],
            message_id: [2; 16],
            envelope_sha256: [3; 32],
            result: SchedulerScheduleResultV1::Ensured {
                schedule_revision: 1,
            },
            received_at_unix_millis: 4,
        };
        assert!(valid_scheduler_result(&result));
        result.result = SchedulerScheduleResultV1::Rejected { error_code: 8 };
        assert!(!valid_scheduler_result(&result));
    }
}
