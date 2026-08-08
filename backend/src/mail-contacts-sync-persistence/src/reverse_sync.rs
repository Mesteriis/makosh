use makosh_mail_contacts_sync_core::{
    MailContactsSyncRejectCodeV1, MailContactsSyncStateV1, MailContactsSyncTransitionV1,
    transition_mail_contacts_sync_v1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AcceptContactChangedForMailSyncOutcomeV1, AcceptContactChangedForMailSyncV1,
    CompleteContactMailSyncSourceOutcomeV1, CompleteContactMailSyncSourceV1,
    CompleteContactsProviderLinkOutcomeV1, CompleteContactsProviderLinkV1,
    CompleteMailAddressBookUpsertOutcomeV1, CompleteMailAddressBookUpsertV1,
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1,
    MailContactsSyncProviderWriteOutcomeV1, MailContactsSyncReverseOperationV1,
    repository::{insert_realtime, load_for_update},
    reverse_model::{
        validate_changed_input, validate_contacts_link_completion, validate_mail_completion,
        validate_source_completion,
    },
};

impl MailContactsSyncPersistenceV1 {
    pub async fn provider_link_operation_for_command(
        &self,
        logical_owner_id: &str,
        contacts_command_message_id: [u8; 16],
    ) -> Result<[u8; 16], MailContactsSyncPersistenceErrorV1> {
        if !crate::model::valid_identity(logical_owner_id)
            || contacts_command_message_id.iter().all(|byte| *byte == 0)
        {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        let operation_id = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT operation_id FROM \
             makosh_data.mail_contacts_sync_provider_link_reconciliation WHERE \
             logical_owner_id=$1 AND contacts_command_message_id=$2",
        )
        .bind(logical_owner_id)
        .bind(contacts_command_message_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)?;
        operation_id
            .try_into()
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)
    }

    pub async fn load_reverse_operation(
        &self,
        logical_owner_id: &str,
        operation_id: [u8; 16],
    ) -> Result<MailContactsSyncReverseOperationV1, MailContactsSyncPersistenceErrorV1> {
        if !crate::model::valid_identity(logical_owner_id)
            || operation_id.iter().all(|byte| *byte == 0)
        {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT configuration_instance_id, account_id, contact_id, contact_revision, state, \
                    origin_run_id, mail_command_message_id \
             FROM makosh_data.mail_contacts_sync_reverse_operations \
             WHERE logical_owner_id = $1 AND operation_id = $2",
        )
        .bind(logical_owner_id)
        .bind(operation_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)?;
        decode_operation(operation_id, &row)
    }

    pub async fn accept_contact_changed_for_mail_sync(
        &self,
        input: &AcceptContactChangedForMailSyncV1,
    ) -> Result<AcceptContactChangedForMailSyncOutcomeV1, MailContactsSyncPersistenceErrorV1> {
        validate_changed_input(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.mail_contacts_sync_reverse_inbox (logical_owner_id, \
             event_message_id, event_envelope_sha256, completed_at_unix_millis) \
             VALUES ($1,$2,$3,$4) ON CONFLICT DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.event_message_id.as_slice())
        .bind(input.event_envelope_sha256.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        if inserted.rows_affected() == 0 {
            validate_replay(&mut transaction, input).await?;
            transaction.commit().await.map_err(storage)?;
            return Ok(AcceptContactChangedForMailSyncOutcomeV1::Duplicate);
        }
        for operation in &input.operations {
            sqlx::query(
                "INSERT INTO makosh_data.mail_contacts_sync_reverse_operations \
                 (logical_owner_id, operation_id, source_event_message_id, \
                  configuration_instance_id, account_id, contact_id, contact_revision, state, \
                  origin_run_id, source_command_message_id, created_at_unix_millis, \
                  updated_at_unix_millis) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,1,$8,$9,$10,$10) ON CONFLICT DO NOTHING",
            )
            .bind(&input.logical_owner_id)
            .bind(operation.operation_id.as_slice())
            .bind(input.event_message_id.as_slice())
            .bind(&operation.configuration_instance_id)
            .bind(&operation.account_id)
            .bind(operation.contact_id.as_slice())
            .bind(
                i64::try_from(operation.contact_revision)
                    .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidInput)?,
            )
            .bind(operation.origin_run_id.map(|value| value.to_vec()))
            .bind(operation.source_prepare_command.message_id.as_slice())
            .bind(input.occurred_at_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
            super::repository::insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                &operation.source_prepare_command,
                input.occurred_at_unix_millis,
            )
            .await?;
        }
        transaction.commit().await.map_err(storage)?;
        Ok(AcceptContactChangedForMailSyncOutcomeV1::Applied {
            operations: u16::try_from(input.operations.len())
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidInput)?,
        })
    }

    pub async fn complete_contact_mail_sync_source(
        &self,
        input: &CompleteContactMailSyncSourceV1,
    ) -> Result<CompleteContactMailSyncSourceOutcomeV1, MailContactsSyncPersistenceErrorV1> {
        validate_source_completion(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let inserted = reserve_result_inbox(&mut transaction, input).await?;
        if !inserted {
            validate_result_replay(&mut transaction, input).await?;
            transaction.commit().await.map_err(storage)?;
            return Ok(CompleteContactMailSyncSourceOutcomeV1::Duplicate);
        }
        let current = sqlx::query(
            "SELECT state FROM makosh_data.mail_contacts_sync_reverse_operations \
             WHERE logical_owner_id = $1 AND operation_id = $2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)?;
        if current.get::<i16, _>("state") != 1 {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidTransition);
        }
        if let Some(command) = &input.mail_command {
            super::repository::insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                command,
                input.occurred_at_unix_millis,
            )
            .await?;
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_contacts_sync_reverse_operations SET state = $3, \
             mail_command_message_id = $4, terminal_message_id = $5, \
             updated_at_unix_millis = $6 WHERE logical_owner_id = $1 AND operation_id = $2 \
             AND state = 1",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .bind(if input.rejected { 4_i16 } else { 2_i16 })
        .bind(
            input
                .mail_command
                .as_ref()
                .map(|value| value.message_id.to_vec()),
        )
        .bind(input.rejected.then_some(input.result_message_id.to_vec()))
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        if updated.rows_affected() != 1 {
            return Err(MailContactsSyncPersistenceErrorV1::RevisionConflict);
        }
        transaction.commit().await.map_err(storage)?;
        Ok(CompleteContactMailSyncSourceOutcomeV1::Applied)
    }

    pub async fn complete_mail_address_book_upsert(
        &self,
        input: &CompleteMailAddressBookUpsertV1,
    ) -> Result<CompleteMailAddressBookUpsertOutcomeV1, MailContactsSyncPersistenceErrorV1> {
        validate_mail_completion(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        if !reserve_event_inbox(
            &mut transaction,
            &input.logical_owner_id,
            input.result_message_id,
            input.result_envelope_sha256,
            input.occurred_at_unix_millis,
        )
        .await?
        {
            validate_event_replay(
                &mut transaction,
                &input.logical_owner_id,
                input.result_message_id,
                input.result_envelope_sha256,
            )
            .await?;
            transaction.commit().await.map_err(storage)?;
            return Ok(CompleteMailAddressBookUpsertOutcomeV1::Duplicate);
        }
        let row = sqlx::query(
            "SELECT state, mail_command_message_id \
             FROM makosh_data.mail_contacts_sync_reverse_operations \
             WHERE logical_owner_id=$1 AND operation_id=$2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)?;
        let command_message_id: Vec<u8> = row.get("mail_command_message_id");
        if row.get::<i16, _>("state") != 2
            || command_message_id.as_slice() != input.mail_command_message_id
        {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidTransition);
        }
        if let Some(command) = &input.contacts_link_command {
            sqlx::query(
                "INSERT INTO makosh_data.mail_contacts_sync_provider_link_reconciliation \
                 (logical_owner_id, operation_id, mail_result_message_id, \
                  mail_result_envelope_sha256, contacts_command_message_id, state, \
                  created_at_unix_millis, updated_at_unix_millis) \
                 VALUES ($1,$2,$3,$4,$5,1,$6,$6)",
            )
            .bind(&input.logical_owner_id)
            .bind(input.operation_id.as_slice())
            .bind(input.result_message_id.as_slice())
            .bind(input.result_envelope_sha256.as_slice())
            .bind(command.message_id.as_slice())
            .bind(input.occurred_at_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
            super::repository::insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                command,
                input.occurred_at_unix_millis,
            )
            .await?;
        } else {
            update_reverse_terminal(&mut transaction, input, 5, input.result_message_id).await?;
        }
        let origin_run_id = sqlx::query_scalar::<_, Option<Vec<u8>>>(
            "SELECT origin_run_id FROM makosh_data.mail_contacts_sync_reverse_operations \
             WHERE logical_owner_id=$1 AND operation_id=$2",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage)?
        .map(|value| {
            value
                .try_into()
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)
        })
        .transpose()?;
        if input.contacts_link_command.is_none()
            && let Some(run_id) = origin_run_id
        {
            apply_provider_outcome_to_run(
                &mut transaction,
                &input.logical_owner_id,
                run_id,
                input.outcome,
                input.occurred_at_unix_millis,
            )
            .await?;
        }
        transaction.commit().await.map_err(storage)?;
        Ok(CompleteMailAddressBookUpsertOutcomeV1::Applied)
    }

    pub async fn complete_contacts_provider_link(
        &self,
        input: &CompleteContactsProviderLinkV1,
    ) -> Result<CompleteContactsProviderLinkOutcomeV1, MailContactsSyncPersistenceErrorV1> {
        validate_contacts_link_completion(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        if !reserve_event_inbox(
            &mut transaction,
            &input.logical_owner_id,
            input.result_message_id,
            input.result_envelope_sha256,
            input.occurred_at_unix_millis,
        )
        .await?
        {
            validate_event_replay(
                &mut transaction,
                &input.logical_owner_id,
                input.result_message_id,
                input.result_envelope_sha256,
            )
            .await?;
            transaction.commit().await.map_err(storage)?;
            return Ok(CompleteContactsProviderLinkOutcomeV1::Duplicate);
        }
        let row = sqlx::query(
            "SELECT contacts_command_message_id, state FROM \
             makosh_data.mail_contacts_sync_provider_link_reconciliation WHERE \
             logical_owner_id=$1 AND operation_id=$2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)?;
        if row.get::<i16, _>("state") != 1
            || row
                .get::<Vec<u8>, _>("contacts_command_message_id")
                .as_slice()
                != input.contacts_command_message_id
        {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidTransition);
        }
        let reconciliation_state = if input.reject_code.is_none() {
            2_i16
        } else {
            3_i16
        };
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_contacts_sync_provider_link_reconciliation SET state=$3, \
             terminal_message_id=$4, reject_code=$5, updated_at_unix_millis=$6 WHERE \
             logical_owner_id=$1 AND operation_id=$2 AND state=1",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .bind(reconciliation_state)
        .bind(input.result_message_id.as_slice())
        .bind(input.reject_code.map(reject_code))
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        if updated.rows_affected() != 1 {
            return Err(MailContactsSyncPersistenceErrorV1::RevisionConflict);
        }
        let reverse_state = if input.reject_code.is_none() {
            3_i16
        } else {
            5_i16
        };
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_contacts_sync_reverse_operations SET state=$3, \
             terminal_message_id=$4, updated_at_unix_millis=$5 WHERE logical_owner_id=$1 AND \
             operation_id=$2 AND state=2",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .bind(reverse_state)
        .bind(input.result_message_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        if updated.rows_affected() != 1 {
            return Err(MailContactsSyncPersistenceErrorV1::RevisionConflict);
        }
        let origin_run_id = sqlx::query_scalar::<_, Option<Vec<u8>>>(
            "SELECT origin_run_id FROM makosh_data.mail_contacts_sync_reverse_operations WHERE \
             logical_owner_id=$1 AND operation_id=$2",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage)?
        .map(|value| {
            value
                .try_into()
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)
        })
        .transpose()?;
        if let Some(run_id) = origin_run_id {
            let outcome = input.reject_code.map_or(
                MailContactsSyncProviderWriteOutcomeV1::Succeeded,
                MailContactsSyncProviderWriteOutcomeV1::Rejected,
            );
            apply_provider_outcome_to_run(
                &mut transaction,
                &input.logical_owner_id,
                run_id,
                outcome,
                input.occurred_at_unix_millis,
            )
            .await?;
        }
        transaction.commit().await.map_err(storage)?;
        Ok(CompleteContactsProviderLinkOutcomeV1::Applied)
    }
}

async fn update_reverse_terminal(
    transaction: &mut Transaction<'_, Postgres>,
    input: &CompleteMailAddressBookUpsertV1,
    state: i16,
    terminal_message_id: [u8; 16],
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    let updated = sqlx::query(
        "UPDATE makosh_data.mail_contacts_sync_reverse_operations SET state=$3, \
         terminal_message_id=$4, updated_at_unix_millis=$5 WHERE logical_owner_id=$1 AND \
         operation_id=$2 AND state=2",
    )
    .bind(&input.logical_owner_id)
    .bind(input.operation_id.as_slice())
    .bind(state)
    .bind(terminal_message_id.as_slice())
    .bind(input.occurred_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if updated.rows_affected() != 1 {
        return Err(MailContactsSyncPersistenceErrorV1::RevisionConflict);
    }
    Ok(())
}

async fn reserve_result_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &CompleteContactMailSyncSourceV1,
) -> Result<bool, MailContactsSyncPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.mail_contacts_sync_reverse_inbox (logical_owner_id, \
         event_message_id, event_envelope_sha256, completed_at_unix_millis) VALUES ($1,$2,$3,$4) \
         ON CONFLICT DO NOTHING",
    )
    .bind(&input.logical_owner_id)
    .bind(input.result_message_id.as_slice())
    .bind(input.result_envelope_sha256.as_slice())
    .bind(input.occurred_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)
    .map(|result| result.rows_affected() == 1)
}

async fn reserve_event_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    occurred_at_unix_millis: i64,
) -> Result<bool, MailContactsSyncPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.mail_contacts_sync_reverse_inbox (logical_owner_id, \
         event_message_id, event_envelope_sha256, completed_at_unix_millis) VALUES ($1,$2,$3,$4) \
         ON CONFLICT DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(message_id.as_slice())
    .bind(envelope_sha256.as_slice())
    .bind(occurred_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)
    .map(|result| result.rows_affected() == 1)
}

async fn validate_event_replay(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT event_envelope_sha256 FROM makosh_data.mail_contacts_sync_reverse_inbox \
         WHERE logical_owner_id=$1 AND event_message_id=$2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?
    .ok_or(MailContactsSyncPersistenceErrorV1::InboxConflict)?;
    let hash: Vec<u8> = row.get("event_envelope_sha256");
    if hash.as_slice() != envelope_sha256 {
        return Err(MailContactsSyncPersistenceErrorV1::InboxConflict);
    }
    Ok(())
}

async fn validate_result_replay(
    transaction: &mut Transaction<'_, Postgres>,
    input: &CompleteContactMailSyncSourceV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT event_envelope_sha256 FROM makosh_data.mail_contacts_sync_reverse_inbox \
         WHERE logical_owner_id = $1 AND event_message_id = $2 FOR UPDATE",
    )
    .bind(&input.logical_owner_id)
    .bind(input.result_message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?
    .ok_or(MailContactsSyncPersistenceErrorV1::InboxConflict)?;
    let hash: Vec<u8> = row.get("event_envelope_sha256");
    if hash.as_slice() != input.result_envelope_sha256 {
        return Err(MailContactsSyncPersistenceErrorV1::InboxConflict);
    }
    Ok(())
}

fn decode_operation(
    operation_id: [u8; 16],
    row: &sqlx::postgres::PgRow,
) -> Result<MailContactsSyncReverseOperationV1, MailContactsSyncPersistenceErrorV1> {
    let contact_id: Vec<u8> = row.get("contact_id");
    let contact_id = contact_id
        .try_into()
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    let contact_revision = u64::try_from(row.get::<i64, _>("contact_revision"))
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    let state = u8::try_from(row.get::<i16, _>("state"))
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    if !(1..=5).contains(&state) {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidRow);
    }
    Ok(MailContactsSyncReverseOperationV1 {
        operation_id,
        configuration_instance_id: row.get("configuration_instance_id"),
        account_id: row.get("account_id"),
        contact_id,
        contact_revision,
        state,
        origin_run_id: row
            .try_get::<Option<Vec<u8>>, _>("origin_run_id")
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?
            .map(|value| {
                value
                    .try_into()
                    .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)
            })
            .transpose()?,
        mail_command_message_id: row
            .try_get::<Option<Vec<u8>>, _>("mail_command_message_id")
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?
            .map(|value| {
                value
                    .try_into()
                    .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)
            })
            .transpose()?,
    })
}

async fn apply_provider_outcome_to_run(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    outcome: MailContactsSyncProviderWriteOutcomeV1,
    occurred_at_unix_millis: i64,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    let current = load_for_update(transaction, logical_owner_id, &run_id).await?;
    if current.status.state != MailContactsSyncStateV1::WritingProvider {
        return if matches!(
            current.status.state,
            MailContactsSyncStateV1::ReconcilingOutcome
                | MailContactsSyncStateV1::Completed
                | MailContactsSyncStateV1::Rejected
        ) {
            Ok(())
        } else {
            Err(MailContactsSyncPersistenceErrorV1::InvalidTransition)
        };
    }
    let next = match outcome {
        MailContactsSyncProviderWriteOutcomeV1::Succeeded => {
            let written = transition_mail_contacts_sync_v1(
                &current.status,
                current.draft.direction,
                MailContactsSyncTransitionV1::ProviderWriteApplied { written: 1 },
            )
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidTransition)?;
            let expected = written
                .counters
                .contacts_created
                .checked_add(written.counters.contacts_updated)
                .ok_or(MailContactsSyncPersistenceErrorV1::InvalidTransition)?;
            if written.counters.provider_entries_written == expected {
                transition_mail_contacts_sync_v1(
                    &written,
                    current.draft.direction,
                    MailContactsSyncTransitionV1::Complete,
                )
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidTransition)?
            } else {
                written
            }
        }
        MailContactsSyncProviderWriteOutcomeV1::OutcomeUnknown => transition_mail_contacts_sync_v1(
            &current.status,
            current.draft.direction,
            MailContactsSyncTransitionV1::ReconcileOutcome,
        )
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidTransition)?,
        MailContactsSyncProviderWriteOutcomeV1::Rejected(code) => transition_mail_contacts_sync_v1(
            &current.status,
            current.draft.direction,
            MailContactsSyncTransitionV1::Reject(code),
        )
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidTransition)?,
    };
    super::orchestration::update_run(
        transaction,
        logical_owner_id,
        &run_id,
        &current,
        &next,
        occurred_at_unix_millis,
    )
    .await?;
    insert_realtime(
        transaction,
        logical_owner_id,
        &run_id,
        occurred_at_unix_millis,
    )
    .await
}

fn reject_code(value: MailContactsSyncRejectCodeV1) -> i16 {
    match value {
        MailContactsSyncRejectCodeV1::InvalidRequest => 1,
        MailContactsSyncRejectCodeV1::AccountUnavailable => 2,
        MailContactsSyncRejectCodeV1::RemoteWriteBlocked => 3,
        MailContactsSyncRejectCodeV1::EtagConflict => 4,
        MailContactsSyncRejectCodeV1::ProviderUnavailable
        | MailContactsSyncRejectCodeV1::ContactsRejected
        | MailContactsSyncRejectCodeV1::Policy
        | MailContactsSyncRejectCodeV1::OutcomeUnknown => 5,
    }
}

async fn validate_replay(
    transaction: &mut Transaction<'_, Postgres>,
    input: &AcceptContactChangedForMailSyncV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT event_envelope_sha256 FROM makosh_data.mail_contacts_sync_reverse_inbox \
         WHERE logical_owner_id = $1 AND event_message_id = $2 FOR UPDATE",
    )
    .bind(&input.logical_owner_id)
    .bind(input.event_message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?
    .ok_or(MailContactsSyncPersistenceErrorV1::InboxConflict)?;
    let hash: Vec<u8> = row.get("event_envelope_sha256");
    if hash.as_slice() != input.event_envelope_sha256 {
        return Err(MailContactsSyncPersistenceErrorV1::InboxConflict);
    }
    Ok(())
}

fn storage(_: sqlx::Error) -> MailContactsSyncPersistenceErrorV1 {
    MailContactsSyncPersistenceErrorV1::StorageUnavailable
}
