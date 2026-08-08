use makosh_mail_contacts_sync_core::{
    MailContactsSyncStateV1, MailContactsSyncTransitionV1, transition_mail_contacts_sync_v1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AdvanceMailContactsSyncPageV1, MailContactsSyncAdvanceOutcomeV1,
    MailContactsSyncContactOutcomeV1, MailContactsSyncEntryInputV1,
    MailContactsSyncEntryOutcomeInputV1, MailContactsSyncPageProgressV1,
    MailContactsSyncPageResultInputV1, MailContactsSyncPersistenceErrorV1,
    MailContactsSyncPersistenceOutcomeV1, MailContactsSyncPersistenceV1,
    model::{nonzero, valid_envelope, valid_identity},
    repository::{
        insert_outbox, insert_realtime, load_for_update, rejection_code, signed, state_code,
    },
};

impl MailContactsSyncPersistenceV1 {
    pub async fn run_id_for_contact_command(
        &self,
        logical_owner_id: &str,
        contact_command_id: &[u8; 16],
    ) -> Result<[u8; 16], MailContactsSyncPersistenceErrorV1> {
        if !valid_identity(logical_owner_id) || !nonzero(contact_command_id) {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT run_id FROM makosh_data.mail_contacts_sync_entries
             WHERE logical_owner_id = $1 AND contact_command_id = $2",
        )
        .bind(logical_owner_id)
        .bind(contact_command_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)?
        .try_into()
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)
    }

    pub async fn advance_ready_page(
        &self,
        input: &AdvanceMailContactsSyncPageV1,
    ) -> Result<MailContactsSyncAdvanceOutcomeV1, MailContactsSyncPersistenceErrorV1> {
        if !valid_identity(&input.logical_owner_id)
            || !nonzero(&input.run_id)
            || input.occurred_at_unix_millis <= 0
            || input
                .next_page_command
                .as_ref()
                .is_some_and(|command| !valid_envelope(command))
        {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let current =
            load_for_update(&mut transaction, &input.logical_owner_id, &input.run_id).await?;
        if current.status.state != MailContactsSyncStateV1::ApplyingContacts {
            transaction.commit().await.map_err(storage)?;
            return Ok(MailContactsSyncAdvanceOutcomeV1::Idle);
        }
        let counts = sqlx::query(
            "SELECT expected_entries,
                    (SELECT COUNT(*) FROM makosh_data.mail_contacts_sync_entries entries
                     WHERE entries.logical_owner_id = pages.logical_owner_id
                       AND entries.run_id = pages.run_id
                       AND entries.page_sequence = pages.page_sequence
                       AND entries.outcome_accounted) AS accounted_entries
             FROM makosh_data.mail_contacts_sync_pages pages
             WHERE logical_owner_id = $1 AND run_id = $2 AND page_sequence = $3",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(signed(current.status.page_sequence)?)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)?;
        if count32(&counts, "expected_entries")? != count32(&counts, "accounted_entries")? {
            transaction.commit().await.map_err(storage)?;
            return Ok(MailContactsSyncAdvanceOutcomeV1::PendingContacts);
        }
        let transition = if current.status.continuation_cursor.is_some() {
            if input.next_page_command.is_none() {
                return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
            }
            MailContactsSyncTransitionV1::BeginProviderPage
        } else if current.draft.direction
            == makosh_mail_contacts_sync_core::MailContactsSyncDirectionV1::Bidirectional
            && current.status.counters.contacts_created + current.status.counters.contacts_updated
                > 0
        {
            if input.next_page_command.is_some() {
                return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
            }
            MailContactsSyncTransitionV1::BeginProviderWrite
        } else {
            if input.next_page_command.is_some() {
                return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
            }
            MailContactsSyncTransitionV1::Complete
        };
        let next =
            transition_mail_contacts_sync_v1(&current.status, current.draft.direction, transition)
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidTransition)?;
        update_run(
            &mut transaction,
            &input.logical_owner_id,
            &input.run_id,
            &current,
            &next,
            input.occurred_at_unix_millis,
        )
        .await?;
        if let Some(command) = input.next_page_command.as_ref() {
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
        transaction.commit().await.map_err(storage)?;
        Ok(MailContactsSyncAdvanceOutcomeV1::Applied)
    }

    pub async fn accept_provider_entry(
        &self,
        input: &MailContactsSyncEntryInputV1,
    ) -> Result<MailContactsSyncPersistenceOutcomeV1, MailContactsSyncPersistenceErrorV1> {
        validate_entry(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        if duplicate_inbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.observation_message_id,
            &input.observation_envelope_sha256,
            &input.run_id,
        )
        .await?
        {
            transaction.commit().await.map_err(storage)?;
            return Ok(MailContactsSyncPersistenceOutcomeV1::Duplicate);
        }
        let run = load_for_update(&mut transaction, &input.logical_owner_id, &input.run_id).await?;
        if run.status.state != MailContactsSyncStateV1::FetchingProviderPage
            || input.page_sequence != run.status.page_sequence + 1
        {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidTransition);
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_contacts_sync_entries (
               logical_owner_id, run_id, page_sequence, contact_command_id, entry_digest,
               observation_message_id, observation_envelope_sha256, outcome, outcome_accounted
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, 0, FALSE)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(signed(input.page_sequence)?)
        .bind(input.contact_command_id.as_slice())
        .bind(input.entry_digest.as_slice())
        .bind(input.observation_message_id.as_slice())
        .bind(input.observation_envelope_sha256.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        insert_inbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.observation_message_id,
            &input.observation_envelope_sha256,
            &input.run_id,
            input.occurred_at_unix_millis,
        )
        .await?;
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.contact_command,
            input.occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage)?;
        Ok(MailContactsSyncPersistenceOutcomeV1::Applied)
    }

    pub async fn accept_provider_page(
        &self,
        input: &MailContactsSyncPageResultInputV1,
    ) -> Result<MailContactsSyncPersistenceOutcomeV1, MailContactsSyncPersistenceErrorV1> {
        validate_page(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        if duplicate_inbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.message_id,
            &input.envelope_sha256,
            &input.run_id,
        )
        .await?
        {
            transaction.commit().await.map_err(storage)?;
            return Ok(MailContactsSyncPersistenceOutcomeV1::Duplicate);
        }
        let current =
            load_for_update(&mut transaction, &input.logical_owner_id, &input.run_id).await?;
        let recorded: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM makosh_data.mail_contacts_sync_entries
             WHERE logical_owner_id = $1 AND run_id = $2 AND page_sequence = $3",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(signed(input.page_sequence)?)
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage)?;
        let expected = i64::from(input.observed_entries);
        if recorded < expected {
            transaction.commit().await.map_err(storage)?;
            return Ok(MailContactsSyncPersistenceOutcomeV1::PendingPrerequisites);
        }
        if recorded > expected {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidTransition);
        }
        let next = transition_mail_contacts_sync_v1(
            &current.status,
            current.draft.direction,
            MailContactsSyncTransitionV1::ProviderPageObserved {
                page_sequence: input.page_sequence,
                continuation_cursor: input.next_continuation_cursor.clone(),
                observed_entries: input.observed_entries,
            },
        )
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidTransition)?;
        update_run(
            &mut transaction,
            &input.logical_owner_id,
            &input.run_id,
            &current,
            &next,
            input.occurred_at_unix_millis,
        )
        .await?;
        sqlx::query(
            "INSERT INTO makosh_data.mail_contacts_sync_pages (
               logical_owner_id, run_id, page_sequence, expected_entries,
               next_continuation_cursor, completed_message_id, completed_envelope_sha256
             ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.run_id.as_slice())
        .bind(signed(input.page_sequence)?)
        .bind(i64::from(input.observed_entries))
        .bind(&input.next_continuation_cursor)
        .bind(input.message_id.as_slice())
        .bind(input.envelope_sha256.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        insert_inbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.message_id,
            &input.envelope_sha256,
            &input.run_id,
            input.occurred_at_unix_millis,
        )
        .await?;
        account_pending_outcomes(
            &mut transaction,
            &input.logical_owner_id,
            &input.run_id,
            input.occurred_at_unix_millis,
        )
        .await?;
        insert_realtime(
            &mut transaction,
            &input.logical_owner_id,
            &input.run_id,
            input.occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage)?;
        Ok(MailContactsSyncPersistenceOutcomeV1::Applied)
    }

    pub async fn accept_contact_outcome(
        &self,
        input: &MailContactsSyncEntryOutcomeInputV1,
    ) -> Result<MailContactsSyncPersistenceOutcomeV1, MailContactsSyncPersistenceErrorV1> {
        validate_outcome(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let run_id: [u8; 16] = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT run_id FROM makosh_data.mail_contacts_sync_entries
             WHERE logical_owner_id = $1 AND contact_command_id = $2",
        )
        .bind(&input.logical_owner_id)
        .bind(input.contact_command_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)?
        .try_into()
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
        // Every orchestration mutation locks the run before an entry. This
        // stable order prevents page completion and an early Contacts result
        // from deadlocking one another.
        let _run = load_for_update(&mut transaction, &input.logical_owner_id, &run_id).await?;
        let entry = sqlx::query(
            "SELECT outcome, outcome_message_id, outcome_envelope_sha256
             FROM makosh_data.mail_contacts_sync_entries
             WHERE logical_owner_id = $1 AND contact_command_id = $2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.contact_command_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)?;
        let existing: i16 = entry
            .try_get("outcome")
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
        if existing != 0 {
            let same = entry
                .try_get::<Option<Vec<u8>>, _>("outcome_message_id")
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?
                .is_some_and(|value| value.as_slice() == input.message_id)
                && entry
                    .try_get::<Option<Vec<u8>>, _>("outcome_envelope_sha256")
                    .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?
                    .is_some_and(|value| value.as_slice() == input.envelope_sha256)
                && existing == outcome_code(input.outcome);
            transaction.commit().await.map_err(storage)?;
            return if same {
                Ok(MailContactsSyncPersistenceOutcomeV1::Duplicate)
            } else {
                Err(MailContactsSyncPersistenceErrorV1::InboxConflict)
            };
        }
        if duplicate_inbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.message_id,
            &input.envelope_sha256,
            &run_id,
        )
        .await?
        {
            return Err(MailContactsSyncPersistenceErrorV1::InboxConflict);
        }
        sqlx::query(
            "UPDATE makosh_data.mail_contacts_sync_entries SET
               outcome = $1, outcome_message_id = $2, outcome_envelope_sha256 = $3
             WHERE logical_owner_id = $4 AND contact_command_id = $5 AND outcome = 0",
        )
        .bind(outcome_code(input.outcome))
        .bind(input.message_id.as_slice())
        .bind(input.envelope_sha256.as_slice())
        .bind(&input.logical_owner_id)
        .bind(input.contact_command_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        insert_inbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.message_id,
            &input.envelope_sha256,
            &run_id,
            input.occurred_at_unix_millis,
        )
        .await?;
        let changed = account_pending_outcomes(
            &mut transaction,
            &input.logical_owner_id,
            &run_id,
            input.occurred_at_unix_millis,
        )
        .await?;
        if changed {
            insert_realtime(
                &mut transaction,
                &input.logical_owner_id,
                &run_id,
                input.occurred_at_unix_millis,
            )
            .await?;
        }
        transaction.commit().await.map_err(storage)?;
        Ok(MailContactsSyncPersistenceOutcomeV1::Applied)
    }

    pub async fn page_progress(
        &self,
        logical_owner_id: &str,
        run_id: &[u8; 16],
    ) -> Result<MailContactsSyncPageProgressV1, MailContactsSyncPersistenceErrorV1> {
        if !valid_identity(logical_owner_id) || !nonzero(run_id) {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT page_sequence, expected_entries, next_continuation_cursor,
                    (SELECT COUNT(*) FROM makosh_data.mail_contacts_sync_entries entries
                     WHERE entries.logical_owner_id = pages.logical_owner_id
                       AND entries.run_id = pages.run_id
                       AND entries.page_sequence = pages.page_sequence) AS recorded_entries,
                    (SELECT COUNT(*) FROM makosh_data.mail_contacts_sync_entries entries
                     WHERE entries.logical_owner_id = pages.logical_owner_id
                       AND entries.run_id = pages.run_id
                       AND entries.page_sequence = pages.page_sequence
                       AND entries.outcome_accounted) AS accounted_entries
             FROM makosh_data.mail_contacts_sync_pages pages
             WHERE logical_owner_id = $1 AND run_id = $2
             ORDER BY page_sequence DESC LIMIT 1",
        )
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage)?
        .ok_or(MailContactsSyncPersistenceErrorV1::NotFound)?;
        Ok(MailContactsSyncPageProgressV1 {
            run_id: *run_id,
            page_sequence: unsigned(
                row.try_get("page_sequence")
                    .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
            )?,
            expected_entries: count32(&row, "expected_entries")?,
            recorded_entries: count32(&row, "recorded_entries")?,
            accounted_entries: count32(&row, "accounted_entries")?,
            next_continuation_cursor: row
                .try_get("next_continuation_cursor")
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
        })
    }
}

async fn account_pending_outcomes(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: &[u8; 16],
    occurred_at_unix_millis: i64,
) -> Result<bool, MailContactsSyncPersistenceErrorV1> {
    let current = load_for_update(transaction, logical_owner_id, run_id).await?;
    if current.status.state != MailContactsSyncStateV1::ApplyingContacts {
        return Ok(false);
    }
    let row = sqlx::query(
        "SELECT
           COUNT(*) FILTER (WHERE outcome = 1) AS created,
           COUNT(*) FILTER (WHERE outcome = 2) AS updated,
           COUNT(*) FILTER (WHERE outcome = 3) AS unchanged,
           COUNT(*) FILTER (WHERE outcome = 4) AS rejected
         FROM makosh_data.mail_contacts_sync_entries
         WHERE logical_owner_id = $1 AND run_id = $2 AND outcome != 0 AND NOT outcome_accounted",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage)?;
    let counts = [
        count32(&row, "created")?,
        count32(&row, "updated")?,
        count32(&row, "unchanged")?,
        count32(&row, "rejected")?,
    ];
    if counts.iter().all(|value| *value == 0) {
        return Ok(false);
    }
    let next = transition_mail_contacts_sync_v1(
        &current.status,
        current.draft.direction,
        MailContactsSyncTransitionV1::ContactsApplied {
            created: counts[0],
            updated: counts[1],
            unchanged: counts[2],
            rejected: counts[3],
        },
    )
    .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidTransition)?;
    update_run(
        transaction,
        logical_owner_id,
        run_id,
        &current,
        &next,
        occurred_at_unix_millis,
    )
    .await?;
    sqlx::query(
        "UPDATE makosh_data.mail_contacts_sync_entries SET outcome_accounted = TRUE
         WHERE logical_owner_id = $1 AND run_id = $2 AND outcome != 0 AND NOT outcome_accounted",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    Ok(true)
}

pub(crate) async fn update_run(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: &[u8; 16],
    current: &crate::PersistedMailContactsSyncRunV1,
    next: &makosh_mail_contacts_sync_core::MailContactsSyncStatusV1,
    occurred_at_unix_millis: i64,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    let updated = sqlx::query(
        "UPDATE makosh_data.mail_contacts_sync_runs SET
           state = $1, state_revision = $2, page_sequence = $3, continuation_cursor = $4,
           provider_entries_seen = $5, contacts_created = $6, contacts_updated = $7,
           contacts_unchanged = $8, provider_entries_written = $9, rejected_entries = $10,
           rejection_code = $11, updated_at_unix_millis = $12
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
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(signed(current.status.state_revision)?)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?
    .rows_affected();
    if updated == 1 {
        Ok(())
    } else {
        Err(MailContactsSyncPersistenceErrorV1::RevisionConflict)
    }
}

async fn duplicate_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    message_id: &[u8; 16],
    envelope_sha256: &[u8; 32],
    run_id: &[u8; 16],
) -> Result<bool, MailContactsSyncPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT envelope_sha256, run_id FROM makosh_data.mail_contacts_sync_inbox
         WHERE logical_owner_id = $1 AND message_id = $2",
    )
    .bind(logical_owner_id)
    .bind(message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(row) = row else {
        return Ok(false);
    };
    let hash: Vec<u8> = row
        .try_get("envelope_sha256")
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    let stored_run: Vec<u8> = row
        .try_get("run_id")
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    if hash.as_slice() == envelope_sha256 && stored_run.as_slice() == run_id {
        Ok(true)
    } else {
        Err(MailContactsSyncPersistenceErrorV1::InboxConflict)
    }
}

async fn insert_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    message_id: &[u8; 16],
    envelope_sha256: &[u8; 32],
    run_id: &[u8; 16],
    occurred_at_unix_millis: i64,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.mail_contacts_sync_inbox (
           logical_owner_id, message_id, envelope_sha256, run_id, processed_at_unix_millis
         ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(logical_owner_id)
    .bind(message_id.as_slice())
    .bind(envelope_sha256.as_slice())
    .bind(run_id.as_slice())
    .bind(occurred_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    Ok(())
}

fn validate_entry(
    input: &MailContactsSyncEntryInputV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    if !valid_identity(&input.logical_owner_id)
        || !nonzero(&input.run_id)
        || input.page_sequence == 0
        || !nonzero(&input.observation_message_id)
        || !nonzero(&input.observation_envelope_sha256)
        || !nonzero(&input.contact_command_id)
        || !nonzero(&input.entry_digest)
        || input.contact_command.message_id != input.contact_command_id
        || !valid_envelope(&input.contact_command)
        || input.occurred_at_unix_millis <= 0
    {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_page(
    input: &MailContactsSyncPageResultInputV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    if !valid_identity(&input.logical_owner_id)
        || !nonzero(&input.run_id)
        || input.page_sequence == 0
        || !nonzero(&input.message_id)
        || !nonzero(&input.envelope_sha256)
        || input.observed_entries > 500
        || input.occurred_at_unix_millis <= 0
        || input
            .next_continuation_cursor
            .as_ref()
            .is_some_and(|value| value.is_empty() || value.len() > 4096)
    {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_outcome(
    input: &MailContactsSyncEntryOutcomeInputV1,
) -> Result<(), MailContactsSyncPersistenceErrorV1> {
    if !valid_identity(&input.logical_owner_id)
        || !nonzero(&input.contact_command_id)
        || !nonzero(&input.message_id)
        || !nonzero(&input.envelope_sha256)
        || input.occurred_at_unix_millis <= 0
    {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

const fn outcome_code(value: MailContactsSyncContactOutcomeV1) -> i16 {
    match value {
        MailContactsSyncContactOutcomeV1::Created => 1,
        MailContactsSyncContactOutcomeV1::Updated => 2,
        MailContactsSyncContactOutcomeV1::Unchanged => 3,
        MailContactsSyncContactOutcomeV1::Rejected => 4,
    }
}

fn count32(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<u32, MailContactsSyncPersistenceErrorV1> {
    let value: i64 = row
        .try_get(name)
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    u32::try_from(value).map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)
}

fn unsigned(value: i64) -> Result<u64, MailContactsSyncPersistenceErrorV1> {
    u64::try_from(value).map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)
}

fn storage(_: sqlx::Error) -> MailContactsSyncPersistenceErrorV1 {
    MailContactsSyncPersistenceErrorV1::StorageUnavailable
}
