use makosh_calendar_core::{
    CalendarConstraintsV1, CalendarEventRecordV1, CalendarEventStateV1, CalendarLifecycleErrorV1,
    CalendarOutcomeKindV1, CalendarOutcomeV1, CalendarParticipantResponseV1,
    CalendarParticipantRoleV1, CalendarParticipantV1, CalendarReminderStateV1, CalendarReminderV1,
    CalendarTimestampV1, add_calendar_participant_v1, add_calendar_reminder_v1,
    create_calendar_event_v1, fire_calendar_reminder_v1, record_calendar_outcome_v1,
    remove_calendar_participant_v1, remove_calendar_reminder_v1, set_calendar_constraints_v1,
    set_calendar_event_state_v1, update_calendar_event_v1, update_calendar_participant_v1,
    validate_calendar_event_record_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    CalendarLifecycleCommitV1, CalendarLifecycleMutationV1, CalendarLifecycleOperationOutcomeV1,
    CalendarLifecycleOperationV1, CalendarOutboxRecordV1, CalendarPersistenceErrorV1,
    CalendarSchedulerCommitV1, CalendarSchedulerInputOutcomeV1, CalendarSchedulerInputV1,
    model::{
        valid_commit, valid_operation, valid_owner, valid_scheduler_commit, valid_scheduler_input,
    },
};

#[derive(Clone)]
pub struct CalendarPersistenceV1 {
    pool: PgPool,
}

pub struct CalendarOutboxPublishClaimV1 {
    transaction: Transaction<'static, Postgres>,
    logical_owner_id: String,
    record: CalendarOutboxRecordV1,
    created_at_unix_millis: i64,
}

impl CalendarOutboxPublishClaimV1 {
    #[must_use]
    pub fn record(&self) -> &CalendarOutboxRecordV1 {
        &self.record
    }

    pub async fn mark_published(
        mut self,
        expected_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), CalendarPersistenceErrorV1> {
        if expected_sha256 != self.record.envelope_sha256
            || Sha256::digest(&self.record.envelope_bytes).as_slice() != expected_sha256
            || published_at_unix_millis < self.created_at_unix_millis
        {
            return Err(CalendarPersistenceErrorV1::OutboxConflict);
        }
        let affected = sqlx::query(
            "UPDATE makosh_data.calendar_outbox SET published_at_unix_millis=$3 \
             WHERE logical_owner_id=$1 AND message_id=$2 AND envelope_sha256=$4 \
             AND published_at_unix_millis IS NULL",
        )
        .bind(&self.logical_owner_id)
        .bind(self.record.message_id.as_slice())
        .bind(published_at_unix_millis)
        .bind(expected_sha256.as_slice())
        .execute(&mut *self.transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if affected != 1 {
            return Err(CalendarPersistenceErrorV1::OutboxConflict);
        }
        self.transaction.commit().await.map_err(storage)
    }
}

impl CalendarPersistenceV1 {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, CalendarPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(CalendarPersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(
                u16::try_from(pgbouncer_port)
                    .map_err(|_| CalendarPersistenceErrorV1::StorageUnavailable)?,
            )
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(storage)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), CalendarPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage)
    }

    async fn begin_owner(
        &self,
        logical_owner_id: &str,
    ) -> Result<Transaction<'_, Postgres>, CalendarPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(CalendarPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        Ok(transaction)
    }

    pub async fn load_operation_replay(
        &self,
        logical_owner_id: &str,
        operation_id: [u8; 16],
        request_sha256: [u8; 32],
        request_bytes: &[u8],
    ) -> Result<Option<Vec<u8>>, CalendarPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || operation_id.iter().all(|byte| *byte == 0)
            || request_sha256.iter().all(|byte| *byte == 0)
            || request_bytes.is_empty()
            || request_bytes.len() > crate::model::CALENDAR_MAX_CLIENT_MESSAGE_BYTES_V1
            || Sha256::digest(request_bytes).as_slice() != request_sha256
        {
            return Err(CalendarPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let replay = load_operation_replay_raw(
            &mut transaction,
            logical_owner_id,
            operation_id,
            request_sha256,
            request_bytes,
            None,
        )
        .await?;
        transaction.commit().await.map_err(storage)?;
        Ok(replay)
    }

    pub async fn apply_lifecycle_operation<F>(
        &self,
        input: CalendarLifecycleOperationV1,
        build_commit: F,
    ) -> Result<CalendarLifecycleOperationOutcomeV1, CalendarPersistenceErrorV1>
    where
        F: FnOnce(
            &CalendarEventRecordV1,
        ) -> Result<CalendarLifecycleCommitV1, CalendarPersistenceErrorV1>,
    {
        if !valid_operation(&input) {
            return Err(CalendarPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1 || encode($2, 'hex'), 0))")
            .bind(&input.logical_owner_id)
            .bind(input.operation_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        if let Some(response_bytes) = load_operation_replay_raw(
            &mut transaction,
            &input.logical_owner_id,
            input.operation_id,
            input.request_sha256,
            &input.request_bytes,
            Some(input.mutation.operation_kind()),
        )
        .await?
        {
            transaction.commit().await.map_err(storage)?;
            return Ok(CalendarLifecycleOperationOutcomeV1::Replayed { response_bytes });
        }

        let creating = matches!(&input.mutation, CalendarLifecycleMutationV1::Create(_));
        let mut event = match &input.mutation {
            CalendarLifecycleMutationV1::Create(draft) => {
                if draft.logical_owner_id != input.logical_owner_id
                    || draft.operation_id != input.operation_id
                {
                    return Err(CalendarPersistenceErrorV1::InvalidInput);
                }
                create_calendar_event_v1(draft.clone()).map_err(core_error)?
            }
            mutation => load_event(
                &mut transaction,
                &input.logical_owner_id,
                mutation
                    .calendar_event_id()
                    .ok_or(CalendarPersistenceErrorV1::InvalidInput)?,
                true,
            )
            .await?
            .ok_or(CalendarPersistenceErrorV1::NotFound)?,
        };
        apply_mutation(&mut event, &input.mutation)?;
        validate_calendar_event_record_v1(&event).map_err(core_error)?;
        persist_event(&mut transaction, &event, creating).await?;
        replace_children(&mut transaction, &event).await?;

        let commit = build_commit(&event)?;
        if !valid_commit(&commit) {
            return Err(CalendarPersistenceErrorV1::InvalidInput);
        }
        for record in &commit.outbox {
            let inserted = sqlx::query(
                "INSERT INTO makosh_data.calendar_outbox (logical_owner_id,message_id,semantic_kind, \
                 envelope_sha256,envelope_bytes,created_at_unix_millis) \
                 VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT DO NOTHING",
            )
            .bind(&input.logical_owner_id)
            .bind(record.message_id.as_slice())
            .bind(record.semantic_kind)
            .bind(record.envelope_sha256.as_slice())
            .bind(&record.envelope_bytes)
            .bind(input.received_at_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?
            .rows_affected();
            if inserted != 1 {
                return Err(CalendarPersistenceErrorV1::OutboxConflict);
            }
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.calendar_client_operations (logical_owner_id,operation_id, \
             operation_kind,request_sha256,request_bytes,calendar_event_id,event_revision, \
             response_sha256,response_bytes,received_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .bind(input.mutation.operation_kind())
        .bind(input.request_sha256.as_slice())
        .bind(&input.request_bytes)
        .bind(event.calendar_event_id.as_slice())
        .bind(i64_value(event.event_revision)?)
        .bind(commit.response_sha256.as_slice())
        .bind(&commit.response_bytes)
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if inserted != 1 {
            return Err(CalendarPersistenceErrorV1::OperationConflict);
        }
        transaction.commit().await.map_err(storage)?;
        Ok(CalendarLifecycleOperationOutcomeV1::Applied {
            event: Box::new(event),
            response_bytes: commit.response_bytes,
        })
    }

    pub async fn get_event(
        &self,
        logical_owner_id: &str,
        calendar_event_id: [u8; 16],
    ) -> Result<Option<CalendarEventRecordV1>, CalendarPersistenceErrorV1> {
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let event =
            load_event(&mut transaction, logical_owner_id, calendar_event_id, false).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(event)
    }

    pub async fn list_events(
        &self,
        logical_owner_id: &str,
        after_calendar_event_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<CalendarEventRecordV1>, CalendarPersistenceErrorV1> {
        self.query_events(logical_owner_id, None, after_calendar_event_id, limit)
            .await
    }

    pub async fn search_events(
        &self,
        logical_owner_id: &str,
        query: &str,
        after_calendar_event_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<CalendarEventRecordV1>, CalendarPersistenceErrorV1> {
        if query.trim().is_empty()
            || query.chars().count() > 200
            || query.chars().any(char::is_control)
        {
            return Err(CalendarPersistenceErrorV1::InvalidInput);
        }
        self.query_events(
            logical_owner_id,
            Some(query.trim()),
            after_calendar_event_id,
            limit,
        )
        .await
    }

    async fn query_events(
        &self,
        logical_owner_id: &str,
        query: Option<&str>,
        after_calendar_event_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<CalendarEventRecordV1>, CalendarPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || limit == 0 || limit > 201 {
            return Err(CalendarPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let rows = if let Some(query) = query {
            sqlx::query(
                "SELECT calendar_event_id FROM makosh_data.calendar_events \
                 WHERE logical_owner_id=$1 AND ($2::bytea IS NULL OR calendar_event_id>$2) \
                 AND (title ILIKE '%' || $3 || '%' OR description ILIKE '%' || $3 || '%') \
                 ORDER BY calendar_event_id LIMIT $4",
            )
            .bind(logical_owner_id)
            .bind(after_calendar_event_id.map(|value| value.to_vec()))
            .bind(query)
            .bind(i64::from(limit))
            .fetch_all(&mut *transaction)
            .await
            .map_err(storage)?
        } else {
            sqlx::query(
                "SELECT calendar_event_id FROM makosh_data.calendar_events \
                 WHERE logical_owner_id=$1 AND ($2::bytea IS NULL OR calendar_event_id>$2) \
                 ORDER BY calendar_event_id LIMIT $3",
            )
            .bind(logical_owner_id)
            .bind(after_calendar_event_id.map(|value| value.to_vec()))
            .bind(i64::from(limit))
            .fetch_all(&mut *transaction)
            .await
            .map_err(storage)?
        };
        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let calendar_event_id = fixed(row.try_get("calendar_event_id").map_err(storage)?)?;
            events.push(
                load_event(&mut transaction, logical_owner_id, calendar_event_id, false)
                    .await?
                    .ok_or(CalendarPersistenceErrorV1::InvalidRow)?,
            );
        }
        transaction.commit().await.map_err(storage)?;
        Ok(events)
    }

    pub async fn record_scheduler_result_once(
        &self,
        input: &CalendarSchedulerInputV1,
    ) -> Result<CalendarSchedulerInputOutcomeV1, CalendarPersistenceErrorV1> {
        if !valid_scheduler_input(input)
            || input.operation_kind != 1
            || input.expected_command_message_id.is_none()
            || input.lease_expires_at_unix_millis.is_some()
        {
            return Err(CalendarPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        lock_scheduler_message(&mut transaction, &input.logical_owner_id, input.message_id).await?;
        if scheduler_replay(&mut transaction, input).await? {
            transaction.commit().await.map_err(storage)?;
            return Ok(CalendarSchedulerInputOutcomeV1::Replayed);
        }
        let command_message_id = input
            .expected_command_message_id
            .ok_or(CalendarPersistenceErrorV1::InvalidInput)?;
        let owns_command = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM makosh_data.calendar_outbox WHERE logical_owner_id=$1 \
             AND message_id=$2 AND semantic_kind=2",
        )
        .bind(&input.logical_owner_id)
        .bind(command_message_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage)?;
        if owns_command != 1 {
            return Err(CalendarPersistenceErrorV1::OperationConflict);
        }
        let calendar_event_id = calendar_event_id_for_reminder(
            &mut transaction,
            &input.logical_owner_id,
            input.reminder_id,
            false,
        )
        .await?;
        insert_scheduler_inbox(&mut transaction, input, calendar_event_id).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(CalendarSchedulerInputOutcomeV1::Applied)
    }

    pub async fn apply_reminder_due_once<F>(
        &self,
        input: &CalendarSchedulerInputV1,
        fired_at: CalendarTimestampV1,
        build_commit: F,
    ) -> Result<CalendarSchedulerInputOutcomeV1, CalendarPersistenceErrorV1>
    where
        F: FnOnce(
            &CalendarEventRecordV1,
            bool,
        ) -> Result<CalendarSchedulerCommitV1, CalendarPersistenceErrorV1>,
    {
        if !valid_scheduler_input(input)
            || input.operation_kind != 2
            || input.expected_command_message_id.is_some()
            || input
                .lease_expires_at_unix_millis
                .is_none_or(|expires| input.completed_at_unix_millis >= expires)
        {
            return Err(CalendarPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        lock_scheduler_message(&mut transaction, &input.logical_owner_id, input.message_id).await?;
        if scheduler_replay(&mut transaction, input).await? {
            transaction.commit().await.map_err(storage)?;
            return Ok(CalendarSchedulerInputOutcomeV1::Replayed);
        }
        let calendar_event_id = calendar_event_id_for_reminder(
            &mut transaction,
            &input.logical_owner_id,
            input.reminder_id,
            true,
        )
        .await?;
        let mut event = load_event(
            &mut transaction,
            &input.logical_owner_id,
            calendar_event_id,
            true,
        )
        .await?
        .ok_or(CalendarPersistenceErrorV1::NotFound)?;
        let reminder = event
            .reminders
            .iter()
            .find(|value| value.reminder_id == input.reminder_id)
            .ok_or(CalendarPersistenceErrorV1::NotFound)?;
        let changed = match reminder.state {
            CalendarReminderStateV1::Pending => {
                let expected_revision = event.event_revision;
                fire_calendar_reminder_v1(
                    &mut event,
                    expected_revision,
                    input.reminder_id,
                    fired_at,
                )
                .map_err(core_error)?;
                true
            }
            CalendarReminderStateV1::Fired => false,
            CalendarReminderStateV1::Cancelled => {
                return Err(CalendarPersistenceErrorV1::OperationConflict);
            }
        };
        if changed {
            validate_calendar_event_record_v1(&event).map_err(core_error)?;
            persist_event(&mut transaction, &event, false).await?;
            replace_children(&mut transaction, &event).await?;
        }
        let commit = build_commit(&event, changed)?;
        if !valid_scheduler_commit(&commit) {
            return Err(CalendarPersistenceErrorV1::InvalidInput);
        }
        for record in &commit.outbox {
            insert_outbox_record(
                &mut transaction,
                &input.logical_owner_id,
                record,
                input.completed_at_unix_millis,
            )
            .await?;
        }
        insert_scheduler_inbox(&mut transaction, input, calendar_event_id).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(CalendarSchedulerInputOutcomeV1::Applied)
    }

    pub async fn claim_next_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Option<CalendarOutboxPublishClaimV1>, CalendarPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(CalendarPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        let row = sqlx::query(
            "SELECT message_id,semantic_kind,envelope_sha256,envelope_bytes,created_at_unix_millis \
             FROM makosh_data.calendar_outbox WHERE logical_owner_id=$1 \
             AND published_at_unix_millis IS NULL ORDER BY outbox_sequence \
             LIMIT 1 FOR UPDATE SKIP LOCKED",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let Some(row) = row else {
            transaction.commit().await.map_err(storage)?;
            return Ok(None);
        };
        let record = CalendarOutboxRecordV1 {
            message_id: fixed(row.try_get("message_id").map_err(storage)?)?,
            semantic_kind: row.try_get("semantic_kind").map_err(storage)?,
            envelope_sha256: fixed(row.try_get("envelope_sha256").map_err(storage)?)?,
            envelope_bytes: row.try_get("envelope_bytes").map_err(storage)?,
        };
        if !crate::model::valid_commit(&CalendarLifecycleCommitV1 {
            response_sha256: Sha256::digest(b"claim").into(),
            response_bytes: b"claim".to_vec(),
            outbox: vec![record.clone()],
        }) {
            return Err(CalendarPersistenceErrorV1::InvalidRow);
        }
        Ok(Some(CalendarOutboxPublishClaimV1 {
            transaction,
            logical_owner_id: logical_owner_id.to_owned(),
            record,
            created_at_unix_millis: row.try_get("created_at_unix_millis").map_err(storage)?,
        }))
    }
}

async fn lock_scheduler_message(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    message_id: [u8; 16],
) -> Result<(), CalendarPersistenceErrorV1> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1 || encode($2, 'hex'), 0))")
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
    Ok(())
}

async fn scheduler_replay(
    transaction: &mut Transaction<'_, Postgres>,
    input: &CalendarSchedulerInputV1,
) -> Result<bool, CalendarPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT envelope_sha256,envelope_bytes,operation_kind,reminder_id,completed_at_unix_millis \
         FROM makosh_data.calendar_scheduler_inbox WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
    )
    .bind(&input.logical_owner_id)
    .bind(input.message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(row) = row else { return Ok(false) };
    let envelope_sha256: Vec<u8> = row.try_get("envelope_sha256").map_err(storage)?;
    let envelope_bytes: Vec<u8> = row.try_get("envelope_bytes").map_err(storage)?;
    let operation_kind: i16 = row.try_get("operation_kind").map_err(storage)?;
    let reminder_id: Option<Vec<u8>> = row.try_get("reminder_id").map_err(storage)?;
    let _completed_at_unix_millis: i64 =
        row.try_get("completed_at_unix_millis").map_err(storage)?;
    if envelope_sha256.as_slice() != input.envelope_sha256
        || envelope_bytes != input.envelope_bytes
        || operation_kind != input.operation_kind
        || reminder_id.as_deref() != Some(input.reminder_id.as_slice())
    {
        return Err(CalendarPersistenceErrorV1::OperationConflict);
    }
    Ok(true)
}

async fn calendar_event_id_for_reminder(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    reminder_id: [u8; 16],
    lock: bool,
) -> Result<[u8; 16], CalendarPersistenceErrorV1> {
    let row = if lock {
        sqlx::query(
            "SELECT calendar_event_id FROM makosh_data.calendar_reminders \
             WHERE logical_owner_id=$1 AND reminder_id=$2 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(reminder_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?
    } else {
        sqlx::query(
            "SELECT calendar_event_id FROM makosh_data.calendar_reminders \
             WHERE logical_owner_id=$1 AND reminder_id=$2",
        )
        .bind(logical_owner_id)
        .bind(reminder_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?
    };
    let row = row.ok_or(CalendarPersistenceErrorV1::NotFound)?;
    fixed(row.try_get("calendar_event_id").map_err(storage)?)
}

async fn insert_scheduler_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &CalendarSchedulerInputV1,
    calendar_event_id: [u8; 16],
) -> Result<(), CalendarPersistenceErrorV1> {
    let affected = sqlx::query(
        "INSERT INTO makosh_data.calendar_scheduler_inbox (logical_owner_id,message_id, \
         envelope_sha256,envelope_bytes,operation_kind,calendar_event_id,reminder_id, \
         completed_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(&input.logical_owner_id)
    .bind(input.message_id.as_slice())
    .bind(input.envelope_sha256.as_slice())
    .bind(&input.envelope_bytes)
    .bind(input.operation_kind)
    .bind(calendar_event_id.as_slice())
    .bind(input.reminder_id.as_slice())
    .bind(input.completed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?
    .rows_affected();
    if affected != 1 {
        return Err(CalendarPersistenceErrorV1::OperationConflict);
    }
    Ok(())
}

async fn insert_outbox_record(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    record: &CalendarOutboxRecordV1,
    created_at_unix_millis: i64,
) -> Result<(), CalendarPersistenceErrorV1> {
    let affected = sqlx::query(
        "INSERT INTO makosh_data.calendar_outbox (logical_owner_id,message_id,semantic_kind, \
         envelope_sha256,envelope_bytes,created_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6) \
         ON CONFLICT DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(record.message_id.as_slice())
    .bind(record.semantic_kind)
    .bind(record.envelope_sha256.as_slice())
    .bind(&record.envelope_bytes)
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?
    .rows_affected();
    if affected != 1 {
        return Err(CalendarPersistenceErrorV1::OutboxConflict);
    }
    Ok(())
}

fn apply_mutation(
    event: &mut CalendarEventRecordV1,
    mutation: &CalendarLifecycleMutationV1,
) -> Result<(), CalendarPersistenceErrorV1> {
    match mutation {
        CalendarLifecycleMutationV1::Create(_) => Ok(()),
        CalendarLifecycleMutationV1::Update {
            expected_revision,
            title,
            description,
            starts_at,
            ends_at,
            timezone,
            changed_at,
            ..
        } => update_calendar_event_v1(
            event,
            *expected_revision,
            title.clone(),
            description.clone(),
            *starts_at,
            *ends_at,
            timezone.clone(),
            *changed_at,
        )
        .map_err(core_error),
        CalendarLifecycleMutationV1::SetState {
            expected_revision,
            state,
            changed_at,
            ..
        } => set_calendar_event_state_v1(event, *expected_revision, *state, *changed_at)
            .map_err(core_error),
        CalendarLifecycleMutationV1::AddParticipant {
            operation_id,
            expected_revision,
            display_name,
            address,
            role,
            response,
            changed_at,
            ..
        } => add_calendar_participant_v1(
            event,
            *expected_revision,
            *operation_id,
            display_name.clone(),
            address.clone(),
            *role,
            *response,
            *changed_at,
        )
        .map(|_| ())
        .map_err(core_error),
        CalendarLifecycleMutationV1::UpdateParticipant {
            expected_revision,
            participant_id,
            display_name,
            address,
            role,
            response,
            changed_at,
            ..
        } => update_calendar_participant_v1(
            event,
            *expected_revision,
            *participant_id,
            display_name.clone(),
            address.clone(),
            *role,
            *response,
            *changed_at,
        )
        .map_err(core_error),
        CalendarLifecycleMutationV1::RemoveParticipant {
            expected_revision,
            participant_id,
            changed_at,
            ..
        } => {
            remove_calendar_participant_v1(event, *expected_revision, *participant_id, *changed_at)
                .map_err(core_error)
        }
        CalendarLifecycleMutationV1::SetConstraints {
            expected_revision,
            earliest_start,
            latest_end,
            minimum_duration_minutes,
            timezone,
            changed_at,
            ..
        } => set_calendar_constraints_v1(
            event,
            *expected_revision,
            *earliest_start,
            *latest_end,
            *minimum_duration_minutes,
            timezone.clone(),
            *changed_at,
        )
        .map_err(core_error),
        CalendarLifecycleMutationV1::AddReminder {
            operation_id,
            expected_revision,
            due_at,
            changed_at,
            ..
        } => add_calendar_reminder_v1(
            event,
            *expected_revision,
            *operation_id,
            *due_at,
            *changed_at,
        )
        .map(|_| ())
        .map_err(core_error),
        CalendarLifecycleMutationV1::RemoveReminder {
            expected_revision,
            reminder_id,
            changed_at,
            ..
        } => remove_calendar_reminder_v1(event, *expected_revision, *reminder_id, *changed_at)
            .map_err(core_error),
        CalendarLifecycleMutationV1::RecordOutcome {
            operation_id,
            expected_revision,
            kind,
            note,
            recorded_at,
            ..
        } => record_calendar_outcome_v1(
            event,
            *expected_revision,
            *operation_id,
            *kind,
            note.clone(),
            *recorded_at,
        )
        .map(|_| ())
        .map_err(core_error),
    }
}

async fn load_operation_replay_raw(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    operation_id: [u8; 16],
    request_sha256: [u8; 32],
    request_bytes: &[u8],
    operation_kind: Option<i16>,
) -> Result<Option<Vec<u8>>, CalendarPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT operation_kind,request_sha256,request_bytes,response_sha256,response_bytes \
         FROM makosh_data.calendar_client_operations \
         WHERE logical_owner_id=$1 AND operation_id=$2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(row) = row else { return Ok(None) };
    let stored_kind: i16 = row.try_get("operation_kind").map_err(storage)?;
    let stored_request_sha: [u8; 32] = fixed(row.try_get("request_sha256").map_err(storage)?)?;
    let stored_request_bytes: Vec<u8> = row.try_get("request_bytes").map_err(storage)?;
    let response_sha: [u8; 32] = fixed(row.try_get("response_sha256").map_err(storage)?)?;
    let response_bytes: Vec<u8> = row.try_get("response_bytes").map_err(storage)?;
    if operation_kind.is_some_and(|value| value != stored_kind)
        || stored_request_sha != request_sha256
        || stored_request_bytes != request_bytes
        || Sha256::digest(&response_bytes).as_slice() != response_sha
    {
        return Err(CalendarPersistenceErrorV1::OperationConflict);
    }
    Ok(Some(response_bytes))
}

async fn load_event(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    calendar_event_id: [u8; 16],
    for_update: bool,
) -> Result<Option<CalendarEventRecordV1>, CalendarPersistenceErrorV1> {
    let sql = if for_update {
        "SELECT title,description,starts_at_unix_seconds,starts_at_nanos,ends_at_unix_seconds, \
         ends_at_nanos,timezone,event_state,event_revision,created_at_unix_seconds,created_at_nanos, \
         updated_at_unix_seconds,updated_at_nanos FROM makosh_data.calendar_events \
         WHERE logical_owner_id=$1 AND calendar_event_id=$2 FOR UPDATE"
    } else {
        "SELECT title,description,starts_at_unix_seconds,starts_at_nanos,ends_at_unix_seconds, \
         ends_at_nanos,timezone,event_state,event_revision,created_at_unix_seconds,created_at_nanos, \
         updated_at_unix_seconds,updated_at_nanos FROM makosh_data.calendar_events \
         WHERE logical_owner_id=$1 AND calendar_event_id=$2"
    };
    let Some(row) = sqlx::query(sql)
        .bind(logical_owner_id)
        .bind(calendar_event_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?
    else {
        return Ok(None);
    };
    let participant_rows = sqlx::query(
        "SELECT participant_id,display_name,address,participant_role,participant_response, \
         updated_at_event_revision FROM makosh_data.calendar_participants \
         WHERE logical_owner_id=$1 AND calendar_event_id=$2 ORDER BY participant_id",
    )
    .bind(logical_owner_id)
    .bind(calendar_event_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?;
    let constraint_row = sqlx::query(
        "SELECT earliest_start_unix_seconds,earliest_start_nanos,latest_end_unix_seconds, \
         latest_end_nanos,minimum_duration_minutes,timezone,updated_at_event_revision \
         FROM makosh_data.calendar_constraints WHERE logical_owner_id=$1 AND calendar_event_id=$2",
    )
    .bind(logical_owner_id)
    .bind(calendar_event_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let reminder_rows = sqlx::query(
        "SELECT reminder_id,due_at_unix_seconds,due_at_nanos,reminder_state,updated_at_event_revision \
         FROM makosh_data.calendar_reminders WHERE logical_owner_id=$1 AND calendar_event_id=$2 \
         ORDER BY reminder_id",
    )
    .bind(logical_owner_id)
    .bind(calendar_event_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?;
    let outcome_rows = sqlx::query(
        "SELECT outcome_id,outcome_kind,note,recorded_at_unix_seconds,recorded_at_nanos, \
         recorded_at_event_revision FROM makosh_data.calendar_outcomes \
         WHERE logical_owner_id=$1 AND calendar_event_id=$2 ORDER BY outcome_id",
    )
    .bind(logical_owner_id)
    .bind(calendar_event_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?;
    let event = CalendarEventRecordV1 {
        calendar_event_id,
        logical_owner_id: logical_owner_id.to_owned(),
        title: row.try_get("title").map_err(storage)?,
        description: row.try_get("description").map_err(storage)?,
        starts_at: timestamp(&row, "starts_at_unix_seconds", "starts_at_nanos")?,
        ends_at: timestamp(&row, "ends_at_unix_seconds", "ends_at_nanos")?,
        timezone: row.try_get("timezone").map_err(storage)?,
        state: decode_event_state(row.try_get("event_state").map_err(storage)?)?,
        event_revision: positive_u64(row.try_get("event_revision").map_err(storage)?)?,
        participants: participant_rows
            .iter()
            .map(decode_participant)
            .collect::<Result<_, _>>()?,
        constraints: constraint_row
            .as_ref()
            .map(decode_constraints)
            .transpose()?,
        reminders: reminder_rows
            .iter()
            .map(decode_reminder)
            .collect::<Result<_, _>>()?,
        outcomes: outcome_rows
            .iter()
            .map(decode_outcome)
            .collect::<Result<_, _>>()?,
        created_at: timestamp(&row, "created_at_unix_seconds", "created_at_nanos")?,
        updated_at: timestamp(&row, "updated_at_unix_seconds", "updated_at_nanos")?,
    };
    validate_calendar_event_record_v1(&event).map_err(core_error)?;
    Ok(Some(event))
}

async fn persist_event(
    transaction: &mut Transaction<'_, Postgres>,
    event: &CalendarEventRecordV1,
    create: bool,
) -> Result<(), CalendarPersistenceErrorV1> {
    let affected = if create {
        sqlx::query(
            "INSERT INTO makosh_data.calendar_events (logical_owner_id,calendar_event_id,title, \
             description,starts_at_unix_seconds,starts_at_nanos,ends_at_unix_seconds,ends_at_nanos, \
             timezone,event_state,event_revision,created_at_unix_seconds,created_at_nanos, \
             updated_at_unix_seconds,updated_at_nanos) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)",
        )
        .bind(&event.logical_owner_id)
        .bind(event.calendar_event_id.as_slice())
        .bind(&event.title)
        .bind(&event.description)
        .bind(event.starts_at.unix_seconds)
        .bind(event.starts_at.nanos)
        .bind(event.ends_at.unix_seconds)
        .bind(event.ends_at.nanos)
        .bind(&event.timezone)
        .bind(encode_event_state(event.state))
        .bind(i64_value(event.event_revision)?)
        .bind(event.created_at.unix_seconds)
        .bind(event.created_at.nanos)
        .bind(event.updated_at.unix_seconds)
        .bind(event.updated_at.nanos)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?
        .rows_affected()
    } else {
        sqlx::query(
            "UPDATE makosh_data.calendar_events SET title=$3,description=$4, \
             starts_at_unix_seconds=$5,starts_at_nanos=$6,ends_at_unix_seconds=$7,ends_at_nanos=$8, \
             timezone=$9,event_state=$10,event_revision=$11,updated_at_unix_seconds=$12, \
             updated_at_nanos=$13 WHERE logical_owner_id=$1 AND calendar_event_id=$2",
        )
        .bind(&event.logical_owner_id)
        .bind(event.calendar_event_id.as_slice())
        .bind(&event.title)
        .bind(&event.description)
        .bind(event.starts_at.unix_seconds)
        .bind(event.starts_at.nanos)
        .bind(event.ends_at.unix_seconds)
        .bind(event.ends_at.nanos)
        .bind(&event.timezone)
        .bind(encode_event_state(event.state))
        .bind(i64_value(event.event_revision)?)
        .bind(event.updated_at.unix_seconds)
        .bind(event.updated_at.nanos)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?
        .rows_affected()
    };
    if affected != 1 {
        return Err(CalendarPersistenceErrorV1::RevisionConflict);
    }
    Ok(())
}

async fn replace_children(
    transaction: &mut Transaction<'_, Postgres>,
    event: &CalendarEventRecordV1,
) -> Result<(), CalendarPersistenceErrorV1> {
    sqlx::query(
        "DELETE FROM makosh_data.calendar_participants WHERE logical_owner_id=$1 AND calendar_event_id=$2",
    )
    .bind(&event.logical_owner_id)
    .bind(event.calendar_event_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    sqlx::query(
        "DELETE FROM makosh_data.calendar_constraints WHERE logical_owner_id=$1 AND calendar_event_id=$2",
    )
    .bind(&event.logical_owner_id)
    .bind(event.calendar_event_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    sqlx::query(
        "DELETE FROM makosh_data.calendar_reminders WHERE logical_owner_id=$1 AND calendar_event_id=$2",
    )
    .bind(&event.logical_owner_id)
    .bind(event.calendar_event_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    sqlx::query(
        "DELETE FROM makosh_data.calendar_outcomes WHERE logical_owner_id=$1 AND calendar_event_id=$2",
    )
    .bind(&event.logical_owner_id)
    .bind(event.calendar_event_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    for value in &event.participants {
        sqlx::query(
            "INSERT INTO makosh_data.calendar_participants (logical_owner_id,calendar_event_id, \
             participant_id,display_name,address,participant_role,participant_response, \
             updated_at_event_revision) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(&event.logical_owner_id)
        .bind(event.calendar_event_id.as_slice())
        .bind(value.participant_id.as_slice())
        .bind(&value.display_name)
        .bind(&value.address)
        .bind(encode_participant_role(value.role))
        .bind(encode_participant_response(value.response))
        .bind(i64_value(value.updated_at_event_revision)?)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
    }
    if let Some(value) = &event.constraints {
        sqlx::query(
            "INSERT INTO makosh_data.calendar_constraints (logical_owner_id,calendar_event_id, \
             earliest_start_unix_seconds,earliest_start_nanos,latest_end_unix_seconds, \
             latest_end_nanos,minimum_duration_minutes,timezone,updated_at_event_revision) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        )
        .bind(&event.logical_owner_id)
        .bind(event.calendar_event_id.as_slice())
        .bind(value.earliest_start.unix_seconds)
        .bind(value.earliest_start.nanos)
        .bind(value.latest_end.unix_seconds)
        .bind(value.latest_end.nanos)
        .bind(
            i32::try_from(value.minimum_duration_minutes)
                .map_err(|_| CalendarPersistenceErrorV1::InvalidInput)?,
        )
        .bind(&value.timezone)
        .bind(i64_value(value.updated_at_event_revision)?)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
    }
    for value in &event.reminders {
        sqlx::query(
            "INSERT INTO makosh_data.calendar_reminders (logical_owner_id,calendar_event_id, \
             reminder_id,due_at_unix_seconds,due_at_nanos,reminder_state,updated_at_event_revision) \
             VALUES ($1,$2,$3,$4,$5,$6,$7)",
        )
        .bind(&event.logical_owner_id)
        .bind(event.calendar_event_id.as_slice())
        .bind(value.reminder_id.as_slice())
        .bind(value.due_at.unix_seconds)
        .bind(value.due_at.nanos)
        .bind(encode_reminder_state(value.state))
        .bind(i64_value(value.updated_at_event_revision)?)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
    }
    for value in &event.outcomes {
        sqlx::query(
            "INSERT INTO makosh_data.calendar_outcomes (logical_owner_id,calendar_event_id, \
             outcome_id,outcome_kind,note,recorded_at_unix_seconds,recorded_at_nanos, \
             recorded_at_event_revision) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(&event.logical_owner_id)
        .bind(event.calendar_event_id.as_slice())
        .bind(value.outcome_id.as_slice())
        .bind(encode_outcome_kind(value.kind))
        .bind(&value.note)
        .bind(value.recorded_at.unix_seconds)
        .bind(value.recorded_at.nanos)
        .bind(i64_value(value.recorded_at_event_revision)?)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
    }
    Ok(())
}

fn decode_participant(
    row: &sqlx::postgres::PgRow,
) -> Result<CalendarParticipantV1, CalendarPersistenceErrorV1> {
    Ok(CalendarParticipantV1 {
        participant_id: fixed(row.try_get("participant_id").map_err(storage)?)?,
        display_name: row.try_get("display_name").map_err(storage)?,
        address: row.try_get("address").map_err(storage)?,
        role: decode_participant_role(row.try_get("participant_role").map_err(storage)?)?,
        response: decode_participant_response(
            row.try_get("participant_response").map_err(storage)?,
        )?,
        updated_at_event_revision: positive_u64(
            row.try_get("updated_at_event_revision").map_err(storage)?,
        )?,
    })
}

fn decode_constraints(
    row: &sqlx::postgres::PgRow,
) -> Result<CalendarConstraintsV1, CalendarPersistenceErrorV1> {
    Ok(CalendarConstraintsV1 {
        earliest_start: timestamp(row, "earliest_start_unix_seconds", "earliest_start_nanos")?,
        latest_end: timestamp(row, "latest_end_unix_seconds", "latest_end_nanos")?,
        minimum_duration_minutes: u32::try_from(
            row.try_get::<i32, _>("minimum_duration_minutes")
                .map_err(storage)?,
        )
        .map_err(|_| CalendarPersistenceErrorV1::InvalidRow)?,
        timezone: row.try_get("timezone").map_err(storage)?,
        updated_at_event_revision: positive_u64(
            row.try_get("updated_at_event_revision").map_err(storage)?,
        )?,
    })
}

fn decode_reminder(
    row: &sqlx::postgres::PgRow,
) -> Result<CalendarReminderV1, CalendarPersistenceErrorV1> {
    Ok(CalendarReminderV1 {
        reminder_id: fixed(row.try_get("reminder_id").map_err(storage)?)?,
        due_at: timestamp(row, "due_at_unix_seconds", "due_at_nanos")?,
        state: decode_reminder_state(row.try_get("reminder_state").map_err(storage)?)?,
        updated_at_event_revision: positive_u64(
            row.try_get("updated_at_event_revision").map_err(storage)?,
        )?,
    })
}

fn decode_outcome(
    row: &sqlx::postgres::PgRow,
) -> Result<CalendarOutcomeV1, CalendarPersistenceErrorV1> {
    Ok(CalendarOutcomeV1 {
        outcome_id: fixed(row.try_get("outcome_id").map_err(storage)?)?,
        kind: decode_outcome_kind(row.try_get("outcome_kind").map_err(storage)?)?,
        note: row.try_get("note").map_err(storage)?,
        recorded_at: timestamp(row, "recorded_at_unix_seconds", "recorded_at_nanos")?,
        recorded_at_event_revision: positive_u64(
            row.try_get("recorded_at_event_revision").map_err(storage)?,
        )?,
    })
}

fn timestamp(
    row: &sqlx::postgres::PgRow,
    seconds: &str,
    nanos: &str,
) -> Result<CalendarTimestampV1, CalendarPersistenceErrorV1> {
    Ok(CalendarTimestampV1 {
        unix_seconds: row.try_get(seconds).map_err(storage)?,
        nanos: row.try_get(nanos).map_err(storage)?,
    })
}

fn encode_event_state(value: CalendarEventStateV1) -> i16 {
    match value {
        CalendarEventStateV1::Scheduled => 1,
        CalendarEventStateV1::Completed => 2,
        CalendarEventStateV1::Cancelled => 3,
    }
}
fn decode_event_state(value: i16) -> Result<CalendarEventStateV1, CalendarPersistenceErrorV1> {
    match value {
        1 => Ok(CalendarEventStateV1::Scheduled),
        2 => Ok(CalendarEventStateV1::Completed),
        3 => Ok(CalendarEventStateV1::Cancelled),
        _ => Err(CalendarPersistenceErrorV1::InvalidRow),
    }
}
fn encode_participant_role(value: CalendarParticipantRoleV1) -> i16 {
    match value {
        CalendarParticipantRoleV1::Organizer => 1,
        CalendarParticipantRoleV1::Required => 2,
        CalendarParticipantRoleV1::Optional => 3,
    }
}
fn decode_participant_role(
    value: i16,
) -> Result<CalendarParticipantRoleV1, CalendarPersistenceErrorV1> {
    match value {
        1 => Ok(CalendarParticipantRoleV1::Organizer),
        2 => Ok(CalendarParticipantRoleV1::Required),
        3 => Ok(CalendarParticipantRoleV1::Optional),
        _ => Err(CalendarPersistenceErrorV1::InvalidRow),
    }
}
fn encode_participant_response(value: CalendarParticipantResponseV1) -> i16 {
    match value {
        CalendarParticipantResponseV1::Pending => 1,
        CalendarParticipantResponseV1::Accepted => 2,
        CalendarParticipantResponseV1::Declined => 3,
        CalendarParticipantResponseV1::Tentative => 4,
    }
}
fn decode_participant_response(
    value: i16,
) -> Result<CalendarParticipantResponseV1, CalendarPersistenceErrorV1> {
    match value {
        1 => Ok(CalendarParticipantResponseV1::Pending),
        2 => Ok(CalendarParticipantResponseV1::Accepted),
        3 => Ok(CalendarParticipantResponseV1::Declined),
        4 => Ok(CalendarParticipantResponseV1::Tentative),
        _ => Err(CalendarPersistenceErrorV1::InvalidRow),
    }
}
fn encode_reminder_state(value: CalendarReminderStateV1) -> i16 {
    match value {
        CalendarReminderStateV1::Pending => 1,
        CalendarReminderStateV1::Fired => 2,
        CalendarReminderStateV1::Cancelled => 3,
    }
}
fn decode_reminder_state(
    value: i16,
) -> Result<CalendarReminderStateV1, CalendarPersistenceErrorV1> {
    match value {
        1 => Ok(CalendarReminderStateV1::Pending),
        2 => Ok(CalendarReminderStateV1::Fired),
        3 => Ok(CalendarReminderStateV1::Cancelled),
        _ => Err(CalendarPersistenceErrorV1::InvalidRow),
    }
}
fn encode_outcome_kind(value: CalendarOutcomeKindV1) -> i16 {
    match value {
        CalendarOutcomeKindV1::Completed => 1,
        CalendarOutcomeKindV1::Cancelled => 2,
        CalendarOutcomeKindV1::NoShow => 3,
    }
}
fn decode_outcome_kind(value: i16) -> Result<CalendarOutcomeKindV1, CalendarPersistenceErrorV1> {
    match value {
        1 => Ok(CalendarOutcomeKindV1::Completed),
        2 => Ok(CalendarOutcomeKindV1::Cancelled),
        3 => Ok(CalendarOutcomeKindV1::NoShow),
        _ => Err(CalendarPersistenceErrorV1::InvalidRow),
    }
}

fn core_error(error: CalendarLifecycleErrorV1) -> CalendarPersistenceErrorV1 {
    match error {
        CalendarLifecycleErrorV1::InvalidRevision => CalendarPersistenceErrorV1::RevisionConflict,
        CalendarLifecycleErrorV1::ParticipantExists
        | CalendarLifecycleErrorV1::ParticipantNotFound
        | CalendarLifecycleErrorV1::OrganizerExists
        | CalendarLifecycleErrorV1::ReminderExists
        | CalendarLifecycleErrorV1::ReminderNotFound
        | CalendarLifecycleErrorV1::ReminderNotPending
        | CalendarLifecycleErrorV1::OutcomeExists
        | CalendarLifecycleErrorV1::InvalidStateTransition => {
            CalendarPersistenceErrorV1::OperationConflict
        }
        _ => CalendarPersistenceErrorV1::InvalidInput,
    }
}

fn fixed<const N: usize>(value: Vec<u8>) -> Result<[u8; N], CalendarPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| CalendarPersistenceErrorV1::InvalidRow)
}

fn positive_u64(value: i64) -> Result<u64, CalendarPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(CalendarPersistenceErrorV1::InvalidRow)
}

fn i64_value(value: u64) -> Result<i64, CalendarPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| CalendarPersistenceErrorV1::InvalidInput)
}

fn storage(_: sqlx::Error) -> CalendarPersistenceErrorV1 {
    CalendarPersistenceErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_errors_are_bounded_and_outbox_claim_is_transactional() {
        assert_eq!(
            core_error(CalendarLifecycleErrorV1::InvalidRevision),
            CalendarPersistenceErrorV1::RevisionConflict
        );
        let source = include_str!("repository.rs");
        assert!(source.contains("FOR UPDATE SKIP LOCKED"));
        assert!(source.contains("set_config('makosh.logical_owner_id'"));
    }
}
