use makosh_mail_contacts_sync_core::MailContactsSyncStateV1;
use sqlx::Row;

use crate::{
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1,
    MailContactsSyncScheduledTerminalOutcomeV1, PendingMailContactsSyncScheduledTerminalV1,
    QueueMailContactsSyncScheduledTerminalV1,
    model::{nonzero, valid_envelope, valid_identity},
    repository::{insert_outbox, state_from_code},
};

impl MailContactsSyncPersistenceV1 {
    pub async fn pending_scheduled_terminal(
        &self,
        logical_owner_id: &str,
    ) -> Result<
        Option<PendingMailContactsSyncScheduledTerminalV1>,
        MailContactsSyncPersistenceErrorV1,
    > {
        if !valid_identity(logical_owner_id) {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT scheduled.run_id, scheduled.command_message_id, scheduled.lease_epoch,
                    scheduled.lease_expires_at_unix_millis, runs.state
             FROM makosh_data.mail_contacts_sync_scheduler_runs scheduled
             JOIN makosh_data.mail_contacts_sync_runs runs
               ON runs.logical_owner_id = scheduled.logical_owner_id
              AND runs.run_id = scheduled.run_id
             WHERE scheduled.logical_owner_id = $1
               AND NOT scheduled.terminal_receipt_queued
               AND runs.state IN (6, 7)
             ORDER BY scheduled.lease_expires_at_unix_millis, scheduled.run_id
             LIMIT 1",
        )
        .bind(logical_owner_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(storage)?;
        row.map(|row| {
            let run_id = bytes16(&row, "run_id")?;
            let command_message_id = bytes16(&row, "command_message_id")?;
            let state: i16 = row
                .try_get("state")
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
            let outcome = match state_from_code(state)? {
                MailContactsSyncStateV1::Completed => {
                    MailContactsSyncScheduledTerminalOutcomeV1::Succeeded
                }
                MailContactsSyncStateV1::Rejected => {
                    MailContactsSyncScheduledTerminalOutcomeV1::Failed
                }
                _ => return Err(MailContactsSyncPersistenceErrorV1::InvalidRow),
            };
            Ok(PendingMailContactsSyncScheduledTerminalV1 {
                run_id,
                command_message_id,
                lease_epoch: unsigned(
                    row.try_get("lease_epoch")
                        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
                )?,
                lease_expires_at_unix_millis: unsigned(
                    row.try_get("lease_expires_at_unix_millis")
                        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
                )?,
                outcome,
            })
        })
        .transpose()
    }

    pub async fn queue_scheduled_terminal(
        &self,
        input: &QueueMailContactsSyncScheduledTerminalV1,
    ) -> Result<bool, MailContactsSyncPersistenceErrorV1> {
        if !valid_identity(&input.logical_owner_id)
            || !nonzero(&input.run_id)
            || !valid_envelope(&input.terminal_receipt)
            || input.queued_at_unix_millis <= 0
        {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let row = sqlx::query(
            "SELECT terminal_receipt_queued
             FROM makosh_data.mail_contacts_sync_scheduler_runs
             WHERE logical_owner_id = $1 AND run_id = $2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)?;
        if row
            .try_get::<bool, _>("terminal_receipt_queued")
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?
        {
            transaction.commit().await.map_err(storage)?;
            return Ok(false);
        }
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.terminal_receipt,
            input.queued_at_unix_millis,
        )
        .await?;
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_contacts_sync_scheduler_runs
             SET terminal_receipt_queued = TRUE
             WHERE logical_owner_id = $1 AND run_id = $2
               AND NOT terminal_receipt_queued",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if updated != 1 {
            return Err(MailContactsSyncPersistenceErrorV1::RevisionConflict);
        }
        transaction.commit().await.map_err(storage)?;
        Ok(true)
    }
}

fn bytes16(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<[u8; 16], MailContactsSyncPersistenceErrorV1> {
    row.try_get::<Vec<u8>, _>(name)
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?
        .try_into()
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)
}

fn unsigned(value: i64) -> Result<u64, MailContactsSyncPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(MailContactsSyncPersistenceErrorV1::InvalidRow)
}

fn storage(_: sqlx::Error) -> MailContactsSyncPersistenceErrorV1 {
    MailContactsSyncPersistenceErrorV1::StorageUnavailable
}
