use makosh_mail_contacts_sync_core::{
    MailContactsSyncCountersV1, MailContactsSyncDirectionV1, MailContactsSyncDraftV1,
    MailContactsSyncRejectCodeV1, MailContactsSyncStateV1, MailContactsSyncStatusV1,
    MailContactsSyncTriggerV1, transition_mail_contacts_sync_v1,
    validate_mail_contacts_sync_draft_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
};

use crate::model::{
    AcceptScheduledMailContactsSyncDueOutcomeV1, AcceptScheduledMailContactsSyncDueV1,
    CreateMailContactsSyncOutcomeV1, CreateMailContactsSyncRunV1, MailContactsSyncInboxOutcomeV1,
    MailContactsSyncPersistenceErrorV1, MailContactsSyncTransitionInputV1, OutboxEnvelopeV1,
    PersistedMailContactsSyncRunV1, direction_code, nonzero, request_fingerprint, trigger_code,
    valid_envelope, valid_identity,
};

#[derive(Clone)]
pub struct MailContactsSyncPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl MailContactsSyncPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, MailContactsSyncPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(MailContactsSyncPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| MailContactsSyncPersistenceErrorV1::StorageUnavailable)?;
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(port)
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(|_| MailContactsSyncPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    #[cfg(feature = "conformance-test-support")]
    #[must_use]
    pub fn from_test_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn verify_storage_ready(&self) -> Result<(), MailContactsSyncPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| MailContactsSyncPersistenceErrorV1::StorageUnavailable)
    }

    pub async fn create_run(
        &self,
        input: CreateMailContactsSyncRunV1,
    ) -> Result<CreateMailContactsSyncOutcomeV1, MailContactsSyncPersistenceErrorV1> {
        validate_create(&input)?;
        let fingerprint = request_fingerprint(&input.draft);
        let initial_status = transition_mail_contacts_sync_v1(
            &makosh_mail_contacts_sync_core::accepted_mail_contacts_sync_status_v1(),
            input.draft.direction,
            makosh_mail_contacts_sync_core::MailContactsSyncTransitionV1::BeginProviderPage,
        )
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidTransition)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.mail_contacts_sync_runs (
               logical_owner_id, run_id, operation_id, request_fingerprint,
               account_id, direction, trigger_kind, state, state_revision,
               page_sequence, provider_entries_seen, contacts_created,
               contacts_updated, contacts_unchanged, provider_entries_written,
               rejected_entries, created_at_unix_millis, updated_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0, 0, 0, 0, 0, 0, 0, $10, $10)
             ON CONFLICT (logical_owner_id, operation_id) DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.draft.run_id.as_slice())
        .bind(input.draft.operation_id.as_slice())
        .bind(fingerprint.as_slice())
        .bind(&input.draft.account_id)
        .bind(direction_code(input.draft.direction))
        .bind(trigger_code(input.draft.trigger))
        .bind(state_code(initial_status.state))
        .bind(signed(initial_status.state_revision)?)
        .bind(input.created_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if inserted == 1 {
            for command in &input.initial_commands {
                insert_outbox(
                    &mut transaction,
                    &input.logical_owner_id,
                    command,
                    input.created_at_unix_millis,
                )
                .await?;
            }
            insert_realtime(
                &mut transaction,
                &input.logical_owner_id,
                &input.draft.run_id,
                input.created_at_unix_millis,
            )
            .await?;
        }
        transaction.commit().await.map_err(storage_error)?;
        let persisted = self
            .load_by_operation(&input.logical_owner_id, &input.draft.operation_id)
            .await?;
        if persisted.request_fingerprint != fingerprint {
            return Err(MailContactsSyncPersistenceErrorV1::RequestConflict);
        }
        Ok(if inserted == 1 {
            CreateMailContactsSyncOutcomeV1::Created(persisted)
        } else {
            CreateMailContactsSyncOutcomeV1::Existing(persisted)
        })
    }

    pub async fn accept_scheduled_due(
        &self,
        input: AcceptScheduledMailContactsSyncDueV1,
    ) -> Result<AcceptScheduledMailContactsSyncDueOutcomeV1, MailContactsSyncPersistenceErrorV1>
    {
        validate_scheduled_due(&input)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        if let Some(row) = sqlx::query(
            "SELECT envelope_sha256, run_id FROM makosh_data.mail_contacts_sync_inbox
             WHERE logical_owner_id = $1 AND message_id = $2",
        )
        .bind(&input.logical_owner_id)
        .bind(input.command_message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        {
            let stored_hash: Vec<u8> = row.try_get("envelope_sha256").map_err(storage_error)?;
            let stored_run: Vec<u8> = row.try_get("run_id").map_err(storage_error)?;
            if stored_hash.as_slice() != input.command_envelope_sha256
                || stored_run.as_slice() != input.scheduler_run_id
            {
                return Err(MailContactsSyncPersistenceErrorV1::InboxConflict);
            }
            transaction.commit().await.map_err(storage_error)?;
            let run = self
                .load_run(&input.logical_owner_id, &input.scheduler_run_id)
                .await
                .ok();
            return Ok(AcceptScheduledMailContactsSyncDueOutcomeV1::Duplicate(run));
        }

        let mut launched = false;
        if let Some(draft) = input.launch.as_ref() {
            let fingerprint = request_fingerprint(draft);
            let initial_status = transition_mail_contacts_sync_v1(
                &makosh_mail_contacts_sync_core::accepted_mail_contacts_sync_status_v1(),
                draft.direction,
                makosh_mail_contacts_sync_core::MailContactsSyncTransitionV1::BeginProviderPage,
            )
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidTransition)?;
            launched = sqlx::query(
                "INSERT INTO makosh_data.mail_contacts_sync_runs (
                   logical_owner_id, run_id, operation_id, request_fingerprint,
                   account_id, direction, trigger_kind, state, state_revision,
                   page_sequence, provider_entries_seen, contacts_created,
                   contacts_updated, contacts_unchanged, provider_entries_written,
                   rejected_entries, created_at_unix_millis, updated_at_unix_millis
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0, 0, 0, 0, 0, 0, 0, $10, $10)
                 ON CONFLICT (logical_owner_id, operation_id) DO NOTHING",
            )
            .bind(&input.logical_owner_id)
            .bind(draft.run_id.as_slice())
            .bind(draft.operation_id.as_slice())
            .bind(fingerprint.as_slice())
            .bind(&draft.account_id)
            .bind(direction_code(draft.direction))
            .bind(trigger_code(draft.trigger))
            .bind(state_code(initial_status.state))
            .bind(signed(initial_status.state_revision)?)
            .bind(input.occurred_at_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(storage_error)?
            .rows_affected()
                == 1;
            if !launched {
                return Err(MailContactsSyncPersistenceErrorV1::RequestConflict);
            }
        }

        for message in &input.durable_messages {
            insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                message,
                input.occurred_at_unix_millis,
            )
            .await?;
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_contacts_sync_inbox (
               logical_owner_id, message_id, envelope_sha256, run_id, processed_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.command_message_id.as_slice())
        .bind(input.command_envelope_sha256.as_slice())
        .bind(input.scheduler_run_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if launched {
            sqlx::query(
                "INSERT INTO makosh_data.mail_contacts_sync_scheduler_runs (
                   logical_owner_id, run_id, command_message_id, lease_epoch,
                   lease_expires_at_unix_millis
                 ) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(&input.logical_owner_id)
            .bind(input.scheduler_run_id.as_slice())
            .bind(input.command_message_id.as_slice())
            .bind(signed(input.lease_epoch)?)
            .bind(signed(input.lease_expires_at_unix_millis)?)
            .execute(&mut *transaction)
            .await
            .map_err(storage_error)?;
            insert_realtime(
                &mut transaction,
                &input.logical_owner_id,
                &input.scheduler_run_id,
                input.occurred_at_unix_millis,
            )
            .await?;
        }
        transaction.commit().await.map_err(storage_error)?;
        if launched {
            self.load_run(&input.logical_owner_id, &input.scheduler_run_id)
                .await
                .map(AcceptScheduledMailContactsSyncDueOutcomeV1::Launched)
        } else {
            Ok(AcceptScheduledMailContactsSyncDueOutcomeV1::Skipped)
        }
    }

    pub async fn apply_transition(
        &self,
        input: MailContactsSyncTransitionInputV1,
    ) -> Result<MailContactsSyncInboxOutcomeV1, MailContactsSyncPersistenceErrorV1> {
        validate_transition_input(&input)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        if let Some(row) = sqlx::query(
            "SELECT envelope_sha256, run_id FROM makosh_data.mail_contacts_sync_inbox
             WHERE logical_owner_id = $1 AND message_id = $2",
        )
        .bind(&input.logical_owner_id)
        .bind(input.message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        {
            let hash: Vec<u8> = row
                .try_get("envelope_sha256")
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
            let run: Vec<u8> = row
                .try_get("run_id")
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
            if hash.as_slice() != input.envelope_sha256 || run.as_slice() != input.run_id {
                return Err(MailContactsSyncPersistenceErrorV1::InboxConflict);
            }
            transaction.commit().await.map_err(storage_error)?;
            return self
                .load_run(&input.logical_owner_id, &input.run_id)
                .await
                .map(MailContactsSyncInboxOutcomeV1::Duplicate);
        }
        let current =
            load_for_update(&mut transaction, &input.logical_owner_id, &input.run_id).await?;
        if current.draft.direction != input.direction {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        let next =
            transition_mail_contacts_sync_v1(&current.status, input.direction, input.transition)
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidTransition)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_contacts_sync_runs SET
               state = $1, state_revision = $2, page_sequence = $3,
               continuation_cursor = $4, provider_entries_seen = $5,
               contacts_created = $6, contacts_updated = $7,
               contacts_unchanged = $8, provider_entries_written = $9,
               rejected_entries = $10, rejection_code = $11,
               updated_at_unix_millis = $12
             WHERE logical_owner_id = $13 AND run_id = $14 AND state_revision = $15",
        )
        .bind(state_code(next.state))
        .bind(signed(next.state_revision)?)
        .bind(signed(next.page_sequence)?)
        .bind(&next.continuation_cursor)
        .bind(signed(next.counters.provider_entries_seen)?)
        .bind(signed(next.counters.contacts_created)?)
        .bind(signed(next.counters.contacts_updated)?)
        .bind(signed(next.counters.contacts_unchanged)?)
        .bind(signed(next.counters.provider_entries_written)?)
        .bind(signed(next.counters.rejected_entries)?)
        .bind(next.rejection.map(rejection_code))
        .bind(input.occurred_at_unix_millis)
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(signed(current.status.state_revision)?)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if updated != 1 {
            return Err(MailContactsSyncPersistenceErrorV1::RevisionConflict);
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_contacts_sync_inbox (
               logical_owner_id, message_id, envelope_sha256, run_id, processed_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.message_id.as_slice())
        .bind(input.envelope_sha256.as_slice())
        .bind(input.run_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if let Some(command) = input.next_command.as_ref() {
            insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                command,
                input.occurred_at_unix_millis,
            )
            .await?;
        }
        insert_realtime(
            &mut transaction,
            &input.logical_owner_id,
            &input.run_id,
            input.occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        self.load_run(&input.logical_owner_id, &input.run_id)
            .await
            .map(MailContactsSyncInboxOutcomeV1::Applied)
    }

    pub async fn load_run(
        &self,
        logical_owner_id: &str,
        run_id: &[u8; 16],
    ) -> Result<PersistedMailContactsSyncRunV1, MailContactsSyncPersistenceErrorV1> {
        if !valid_identity(logical_owner_id) || !nonzero(run_id) {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(SELECT_RUN)
            .bind(logical_owner_id)
            .bind(run_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)
            .and_then(persisted_from_row)
    }

    async fn load_by_operation(
        &self,
        logical_owner_id: &str,
        operation_id: &[u8; 16],
    ) -> Result<PersistedMailContactsSyncRunV1, MailContactsSyncPersistenceErrorV1> {
        sqlx::query(SELECT_BY_OPERATION)
            .bind(logical_owner_id)
            .bind(operation_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)
            .and_then(persisted_from_row)
    }
}

pub(crate) async fn load_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: &[u8; 16],
) -> Result<PersistedMailContactsSyncRunV1, MailContactsSyncPersistenceErrorV1> {
    sqlx::query(SELECT_RUN_FOR_UPDATE)
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)
        .and_then(persisted_from_row)
}

pub(crate) async fn insert_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    envelope: &OutboxEnvelopeV1,
    created_at_unix_millis: i64,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.mail_contacts_sync_outbox (
           logical_owner_id, message_id, envelope_sha256, envelope_bytes, created_at_unix_millis
         ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(logical_owner_id)
    .bind(envelope.message_id.as_slice())
    .bind(envelope.envelope_sha256.as_slice())
    .bind(&envelope.envelope_bytes)
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?;
    Ok(())
}

pub(crate) async fn insert_realtime(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: &[u8; 16],
    occurred_at_unix_millis: i64,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.mail_contacts_sync_realtime (
           logical_owner_id, run_id, state, state_revision, rejection_code, occurred_at_unix_millis
         ) SELECT logical_owner_id, run_id, state, state_revision, rejection_code, $1
           FROM makosh_data.mail_contacts_sync_runs
           WHERE logical_owner_id = $2 AND run_id = $3",
    )
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?
    .rows_affected();
    if inserted == 1 {
        Ok(())
    } else {
        Err(MailContactsSyncPersistenceErrorV1::InvalidRow)
    }
}

fn validate_create(
    input: &CreateMailContactsSyncRunV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    if !valid_identity(&input.logical_owner_id)
        || validate_mail_contacts_sync_draft_v1(&input.draft).is_err()
        || input.initial_commands.is_empty()
        || input.initial_commands.len() > 4
        || input
            .initial_commands
            .iter()
            .any(|command| !valid_envelope(command))
        || {
            let unique = input
                .initial_commands
                .iter()
                .map(|command| command.message_id)
                .collect::<std::collections::BTreeSet<_>>();
            unique.len() != input.initial_commands.len()
        }
        || input.created_at_unix_millis <= 0
    {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_scheduled_due(
    input: &AcceptScheduledMailContactsSyncDueV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    let messages_are_valid = !input.durable_messages.is_empty()
        && input.durable_messages.len() <= 4
        && input.durable_messages.iter().all(valid_envelope)
        && input
            .durable_messages
            .iter()
            .map(|message| message.message_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == input.durable_messages.len();
    let launch_is_valid = input.launch.as_ref().is_none_or(|draft| {
        validate_mail_contacts_sync_draft_v1(draft).is_ok()
            && draft.trigger == MailContactsSyncTriggerV1::Scheduled
            && draft.run_id == input.scheduler_run_id
            && draft.operation_id == input.scheduler_run_id
    });
    if !valid_identity(&input.logical_owner_id)
        || !nonzero(&input.command_message_id)
        || !nonzero(&input.command_envelope_sha256)
        || !nonzero(&input.scheduler_run_id)
        || input.lease_epoch == 0
        || input.lease_expires_at_unix_millis
            <= u64::try_from(input.occurred_at_unix_millis).unwrap_or_default()
        || !messages_are_valid
        || !launch_is_valid
        || input.occurred_at_unix_millis <= 0
    {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_transition_input(
    input: &MailContactsSyncTransitionInputV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    if !valid_identity(&input.logical_owner_id)
        || !nonzero(&input.run_id)
        || !nonzero(&input.message_id)
        || !nonzero(&input.envelope_sha256)
        || input.occurred_at_unix_millis <= 0
        || input
            .next_command
            .as_ref()
            .is_some_and(|value| !valid_envelope(value))
    {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn persisted_from_row(
    row: PgRow,
) -> Result<PersistedMailContactsSyncRunV1, MailContactsSyncPersistenceErrorV1> {
    let bytes16 = |name| -> Result<[u8; 16], MailContactsSyncPersistenceErrorV1> {
        row.try_get::<Vec<u8>, _>(name)
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?
            .try_into()
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)
    };
    let bytes32 = |name| -> Result<[u8; 32], MailContactsSyncPersistenceErrorV1> {
        row.try_get::<Vec<u8>, _>(name)
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?
            .try_into()
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)
    };
    let direction = direction_from_code(
        row.try_get("direction")
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
    )?;
    let trigger = trigger_from_code(
        row.try_get("trigger_kind")
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
    )?;
    let rejection = row
        .try_get::<Option<i16>, _>("rejection_code")
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?
        .map(rejection_from_code)
        .transpose()?;
    let unsigned = |name| -> Result<u64, MailContactsSyncPersistenceErrorV1> {
        let value: i64 = row
            .try_get(name)
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
        u64::try_from(value).map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)
    };
    Ok(PersistedMailContactsSyncRunV1 {
        logical_owner_id: row
            .try_get("logical_owner_id")
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
        draft: MailContactsSyncDraftV1 {
            run_id: bytes16("run_id")?,
            operation_id: bytes16("operation_id")?,
            account_id: row
                .try_get("account_id")
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
            direction,
            trigger,
        },
        request_fingerprint: bytes32("request_fingerprint")?,
        status: MailContactsSyncStatusV1 {
            state: state_from_code(
                row.try_get("state")
                    .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
            )?,
            state_revision: unsigned("state_revision")?,
            page_sequence: unsigned("page_sequence")?,
            continuation_cursor: row
                .try_get("continuation_cursor")
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
            counters: MailContactsSyncCountersV1 {
                provider_entries_seen: unsigned("provider_entries_seen")?,
                contacts_created: unsigned("contacts_created")?,
                contacts_updated: unsigned("contacts_updated")?,
                contacts_unchanged: unsigned("contacts_unchanged")?,
                provider_entries_written: unsigned("provider_entries_written")?,
                rejected_entries: unsigned("rejected_entries")?,
            },
            rejection,
        },
        created_at_unix_millis: row
            .try_get("created_at_unix_millis")
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
        updated_at_unix_millis: row
            .try_get("updated_at_unix_millis")
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
    })
}

fn storage_error(_: sqlx::Error) -> MailContactsSyncPersistenceErrorV1 {
    MailContactsSyncPersistenceErrorV1::StorageUnavailable
}
pub(crate) fn signed(value: u64) -> Result<i64, MailContactsSyncPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidInput)
}
pub(crate) const fn state_code(value: MailContactsSyncStateV1) -> i16 {
    match value {
        MailContactsSyncStateV1::Accepted => 1,
        MailContactsSyncStateV1::FetchingProviderPage => 2,
        MailContactsSyncStateV1::ApplyingContacts => 3,
        MailContactsSyncStateV1::WritingProvider => 4,
        MailContactsSyncStateV1::ReconcilingOutcome => 5,
        MailContactsSyncStateV1::Completed => 6,
        MailContactsSyncStateV1::Rejected => 7,
    }
}
pub(crate) fn state_from_code(
    value: i16,
) -> Result<MailContactsSyncStateV1, MailContactsSyncPersistenceErrorV1> {
    match value {
        1 => Ok(MailContactsSyncStateV1::Accepted),
        2 => Ok(MailContactsSyncStateV1::FetchingProviderPage),
        3 => Ok(MailContactsSyncStateV1::ApplyingContacts),
        4 => Ok(MailContactsSyncStateV1::WritingProvider),
        5 => Ok(MailContactsSyncStateV1::ReconcilingOutcome),
        6 => Ok(MailContactsSyncStateV1::Completed),
        7 => Ok(MailContactsSyncStateV1::Rejected),
        _ => Err(MailContactsSyncPersistenceErrorV1::InvalidRow),
    }
}
fn direction_from_code(
    value: i16,
) -> Result<MailContactsSyncDirectionV1, MailContactsSyncPersistenceErrorV1> {
    match value {
        1 => Ok(MailContactsSyncDirectionV1::ProviderToContacts),
        2 => Ok(MailContactsSyncDirectionV1::Bidirectional),
        _ => Err(MailContactsSyncPersistenceErrorV1::InvalidRow),
    }
}
fn trigger_from_code(
    value: i16,
) -> Result<MailContactsSyncTriggerV1, MailContactsSyncPersistenceErrorV1> {
    match value {
        1 => Ok(MailContactsSyncTriggerV1::Manual),
        2 => Ok(MailContactsSyncTriggerV1::Scheduled),
        _ => Err(MailContactsSyncPersistenceErrorV1::InvalidRow),
    }
}
pub(crate) const fn rejection_code(value: MailContactsSyncRejectCodeV1) -> i16 {
    match value {
        MailContactsSyncRejectCodeV1::InvalidRequest => 1,
        MailContactsSyncRejectCodeV1::AccountUnavailable => 2,
        MailContactsSyncRejectCodeV1::ProviderUnavailable => 3,
        MailContactsSyncRejectCodeV1::ContactsRejected => 4,
        MailContactsSyncRejectCodeV1::RemoteWriteBlocked => 5,
        MailContactsSyncRejectCodeV1::EtagConflict => 6,
        MailContactsSyncRejectCodeV1::OutcomeUnknown => 7,
        MailContactsSyncRejectCodeV1::Policy => 8,
    }
}
pub(crate) fn rejection_from_code(
    value: i16,
) -> Result<MailContactsSyncRejectCodeV1, MailContactsSyncPersistenceErrorV1> {
    match value {
        1 => Ok(MailContactsSyncRejectCodeV1::InvalidRequest),
        2 => Ok(MailContactsSyncRejectCodeV1::AccountUnavailable),
        3 => Ok(MailContactsSyncRejectCodeV1::ProviderUnavailable),
        4 => Ok(MailContactsSyncRejectCodeV1::ContactsRejected),
        5 => Ok(MailContactsSyncRejectCodeV1::RemoteWriteBlocked),
        6 => Ok(MailContactsSyncRejectCodeV1::EtagConflict),
        7 => Ok(MailContactsSyncRejectCodeV1::OutcomeUnknown),
        8 => Ok(MailContactsSyncRejectCodeV1::Policy),
        _ => Err(MailContactsSyncPersistenceErrorV1::InvalidRow),
    }
}

const SELECT_RUN: &str = concat!(
    "SELECT ",
    "logical_owner_id, run_id, operation_id, request_fingerprint, account_id, direction, trigger_kind, state, state_revision, page_sequence, continuation_cursor, provider_entries_seen, contacts_created, contacts_updated, contacts_unchanged, provider_entries_written, rejected_entries, rejection_code, created_at_unix_millis, updated_at_unix_millis",
    " FROM makosh_data.mail_contacts_sync_runs WHERE logical_owner_id = $1 AND run_id = $2"
);
const SELECT_RUN_FOR_UPDATE: &str = concat!(
    "SELECT ",
    "logical_owner_id, run_id, operation_id, request_fingerprint, account_id, direction, trigger_kind, state, state_revision, page_sequence, continuation_cursor, provider_entries_seen, contacts_created, contacts_updated, contacts_unchanged, provider_entries_written, rejected_entries, rejection_code, created_at_unix_millis, updated_at_unix_millis",
    " FROM makosh_data.mail_contacts_sync_runs WHERE logical_owner_id = $1 AND run_id = $2 FOR UPDATE"
);
const SELECT_BY_OPERATION: &str = concat!(
    "SELECT ",
    "logical_owner_id, run_id, operation_id, request_fingerprint, account_id, direction, trigger_kind, state, state_revision, page_sequence, continuation_cursor, provider_entries_seen, contacts_created, contacts_updated, contacts_unchanged, provider_entries_written, rejected_entries, rejection_code, created_at_unix_millis, updated_at_unix_millis",
    " FROM makosh_data.mail_contacts_sync_runs WHERE logical_owner_id = $1 AND operation_id = $2"
);

#[cfg(test)]
mod tests {
    use makosh_mail_contacts_sync_core::MailContactsSyncTransitionV1;

    use super::*;

    #[test]
    fn select_shapes_remain_owner_local() {
        assert!(SELECT_RUN.contains("request_fingerprint"));
        assert!(SELECT_RUN_FOR_UPDATE.ends_with("FOR UPDATE"));
        assert!(SELECT_BY_OPERATION.contains("mail_contacts_sync_runs"));
        for forbidden in [
            "makosh_data.contacts_",
            "makosh_data.mail_accounts",
            "makosh_data.communications_",
        ] {
            assert!(!SELECT_RUN.contains(forbidden), "{forbidden}");
        }
    }

    #[test]
    fn stable_request_fingerprint_excludes_run_identity() {
        let draft = MailContactsSyncDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            account_id: "account-1".to_owned(),
            direction: MailContactsSyncDirectionV1::ProviderToContacts,
            trigger: MailContactsSyncTriggerV1::Manual,
        };
        let mut replay = draft.clone();
        replay.run_id = [9; 16];
        assert_eq!(request_fingerprint(&draft), request_fingerprint(&replay));
        replay.direction = MailContactsSyncDirectionV1::Bidirectional;
        assert_ne!(request_fingerprint(&draft), request_fingerprint(&replay));
    }

    #[test]
    fn initial_state_advances_when_the_first_fetch_command_is_committed() {
        let accepted = makosh_mail_contacts_sync_core::accepted_mail_contacts_sync_status_v1();
        let fetching = transition_mail_contacts_sync_v1(
            &accepted,
            MailContactsSyncDirectionV1::ProviderToContacts,
            MailContactsSyncTransitionV1::BeginProviderPage,
        )
        .expect("fetching");
        assert_eq!(
            fetching.state,
            MailContactsSyncStateV1::FetchingProviderPage
        );
    }
}
