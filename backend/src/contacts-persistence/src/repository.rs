use std::collections::BTreeSet;

use makosh_contacts_core::{
    ContactIdentityMatchV1, ContactProviderKindV1, ContactProviderProvenanceV1, ContactTimestampV1,
    ContactUpsertOutcomeV1, ContactV1, decide_contact_upsert_v1, normalize_email_v1,
    normalize_phone_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::model::{
    CONTACTS_OUTBOX_LIMIT_V1, valid_apply, valid_bounded_text, valid_mutation_outbox, valid_outbox,
    valid_owner, valid_reject, valid_source_reservation, valid_source_result,
};
use crate::{
    AppliedMailEntryCommandV1, ApplyMailEntryCommandV1, ContactMailEntryRejectCodeV1,
    ContactMailSyncSourceLinkV1, ContactMailSyncSourceRejectCodeV1, ContactMailSyncSourceResultV1,
    ContactMailSyncSourceSnapshotV1, ContactMutationOutboxV1, ContactsOutboxRecordV1,
    ContactsPersistenceErrorV1, PersistContactMailSyncSourceResultV1, RejectMailEntryCommandV1,
    RejectedMailEntryCommandV1, ReserveContactMailSyncSourceV1,
};

#[derive(Clone)]
pub struct ContactsPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl ContactsPersistenceV1 {
    #[cfg(feature = "conformance-test-support")]
    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, ContactsPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(ContactsPersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(
                u16::try_from(pgbouncer_port)
                    .map_err(|_| ContactsPersistenceErrorV1::StorageUnavailable)?,
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

    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn verify_storage_ready(&self) -> Result<(), ContactsPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage)
    }

    pub async fn apply_mail_entry<F>(
        &self,
        input: &ApplyMailEntryCommandV1,
        build_terminal_result: F,
    ) -> Result<AppliedMailEntryCommandV1, ContactsPersistenceErrorV1>
    where
        F: FnOnce(
            &ContactV1,
            ContactUpsertOutcomeV1,
        ) -> Result<ContactMutationOutboxV1, ContactsPersistenceErrorV1>,
    {
        if !valid_apply(input) {
            return Err(ContactsPersistenceErrorV1::InvalidInput);
        }
        let fingerprint = input.command_fingerprint();
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let inserted = reserve_inbox(&mut transaction, input, fingerprint).await?;
        if !inserted {
            let replay = load_inbox(&mut transaction, input).await?;
            transaction.commit().await.map_err(storage)?;
            return Ok(replay);
        }

        let identity_match = load_identity_match(&mut transaction, input).await?;
        let target_id = unique_target(&identity_match)?;
        let existing = match target_id {
            Some(contact_id) => Some(
                load_contact(&mut transaction, &input.draft.logical_owner_id, contact_id).await?,
            ),
            None => None,
        };
        let (contact, outcome) =
            decide_contact_upsert_v1(input.draft.clone(), identity_match, existing.as_ref())
                .map_err(map_decision)?;
        let mutation_outbox = build_terminal_result(&contact, outcome)?;
        if !valid_mutation_outbox(&mutation_outbox)
            || (outcome == ContactUpsertOutcomeV1::Unchanged
                && mutation_outbox.changed_event.is_some())
            || (outcome != ContactUpsertOutcomeV1::Unchanged
                && mutation_outbox.changed_event.is_none())
        {
            return Err(ContactsPersistenceErrorV1::InvalidInput);
        }
        if outcome != ContactUpsertOutcomeV1::Unchanged {
            persist_contact(&mut transaction, &contact, outcome).await?;
        }
        // Provider provenance has an independent freshness lifecycle. Refresh it even
        // when canonical Contact fields and contact_revision remain unchanged.
        persist_provider_link(&mut transaction, &contact).await?;
        insert_outbox(
            &mut transaction,
            &input.draft.logical_owner_id,
            &mutation_outbox.terminal_result,
            input.completed_at_unix_millis,
        )
        .await?;
        if let Some(changed_event) = &mutation_outbox.changed_event {
            insert_outbox(
                &mut transaction,
                &input.draft.logical_owner_id,
                changed_event,
                input.completed_at_unix_millis,
            )
            .await?;
        }
        complete_inbox(
            &mut transaction,
            input,
            &contact,
            outcome,
            mutation_outbox.terminal_result.message_id,
        )
        .await?;
        transaction.commit().await.map_err(storage)?;
        Ok(AppliedMailEntryCommandV1 {
            contact_id: contact.contact_id,
            contact_revision: contact.contact_revision,
            outcome,
            terminal_result: mutation_outbox.terminal_result,
            replayed: false,
        })
    }

    pub async fn reject_mail_entry(
        &self,
        input: &RejectMailEntryCommandV1,
    ) -> Result<RejectedMailEntryCommandV1, ContactsPersistenceErrorV1> {
        if !valid_reject(input) {
            return Err(ContactsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let inserted = reserve_rejected_inbox(&mut transaction, input).await?;
        if !inserted {
            let replay = load_rejected_inbox(&mut transaction, input).await?;
            transaction.commit().await.map_err(storage)?;
            return Ok(replay);
        }
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.terminal_result,
            input.completed_at_unix_millis,
        )
        .await?;
        complete_rejected_inbox(&mut transaction, input).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(RejectedMailEntryCommandV1 {
            code: input.code,
            terminal_result: input.terminal_result.clone(),
            replayed: false,
        })
    }

    pub async fn contact_mail_sync_source_snapshot(
        &self,
        logical_owner_id: &str,
        contact_id: [u8; 16],
        expected_contact_revision: u64,
        target_mail_account_id: &str,
    ) -> Result<ContactMailSyncSourceSnapshotV1, ContactsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || contact_id.iter().all(|byte| *byte == 0)
            || expected_contact_revision == 0
            || !valid_bounded_text(target_mail_account_id, 256)
        {
            return Err(ContactsPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT display_name, contact_revision FROM makosh_data.contacts_state \
             WHERE logical_owner_id = $1 AND contact_id = $2",
        )
        .bind(logical_owner_id)
        .bind(contact_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage)?
        .ok_or(ContactsPersistenceErrorV1::NotFound)?;
        let revision = u64_value(row.get::<i64, _>("contact_revision"))?;
        if revision != expected_contact_revision {
            return Err(ContactsPersistenceErrorV1::StaleSource);
        }
        let email_addresses = sqlx::query_scalar::<_, String>(
            "SELECT normalized_email FROM makosh_data.contacts_email_identities \
             WHERE logical_owner_id = $1 AND contact_id = $2 ORDER BY normalized_email",
        )
        .bind(logical_owner_id)
        .bind(contact_id.as_slice())
        .fetch_all(&self.pool)
        .await
        .map_err(storage)?;
        let phone_numbers = sqlx::query_scalar::<_, String>(
            "SELECT normalized_phone FROM makosh_data.contacts_phone_identities \
             WHERE logical_owner_id = $1 AND contact_id = $2 ORDER BY normalized_phone",
        )
        .bind(logical_owner_id)
        .bind(contact_id.as_slice())
        .fetch_all(&self.pool)
        .await
        .map_err(storage)?;
        let links = sqlx::query(
            "SELECT provider_entry_id, provider_etag FROM makosh_data.contacts_provider_links \
             WHERE logical_owner_id = $1 AND contact_id = $2 AND source_account_id = $3 \
             ORDER BY provider_kind, provider_entry_id LIMIT 2",
        )
        .bind(logical_owner_id)
        .bind(contact_id.as_slice())
        .bind(target_mail_account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(storage)?;
        if links.len() > 1 {
            return Err(ContactsPersistenceErrorV1::PolicyRejected);
        }
        let target_account_link = links.first().map(|link| ContactMailSyncSourceLinkV1 {
            provider_entry_id: link.get("provider_entry_id"),
            provider_etag: link.get("provider_etag"),
        });
        Ok(ContactMailSyncSourceSnapshotV1 {
            contact_id,
            contact_revision: revision,
            display_name: row.get("display_name"),
            email_addresses,
            phone_numbers,
            target_account_link,
        })
    }

    pub async fn reserve_contact_mail_sync_source(
        &self,
        input: &ReserveContactMailSyncSourceV1,
    ) -> Result<Option<ContactMailSyncSourceResultV1>, ContactsPersistenceErrorV1> {
        if !valid_source_reservation(input) {
            return Err(ContactsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        let inserted = reserve_source_inbox(&mut transaction, input).await?;
        let replay = if inserted {
            None
        } else {
            load_source_inbox(&mut transaction, input).await?
        };
        transaction.commit().await.map_err(storage)?;
        Ok(replay)
    }

    pub async fn persist_contact_mail_sync_source_result(
        &self,
        input: &PersistContactMailSyncSourceResultV1,
    ) -> Result<ContactMailSyncSourceResultV1, ContactsPersistenceErrorV1> {
        if !valid_source_result(input) {
            return Err(ContactsPersistenceErrorV1::InvalidInput);
        }
        let reservation = source_reservation(input);
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        if let Some(replay) = load_source_inbox(&mut transaction, &reservation).await? {
            if replay.reject_code != input.reject_code
                || replay.terminal_result != input.terminal_result
            {
                return Err(ContactsPersistenceErrorV1::CommandConflict);
            }
            transaction.commit().await.map_err(storage)?;
            return Ok(replay);
        }
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.terminal_result,
            input.completed_at_unix_millis,
        )
        .await?;
        complete_source_inbox(&mut transaction, input).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(ContactMailSyncSourceResultV1 {
            terminal_result: input.terminal_result.clone(),
            reject_code: input.reject_code,
            replayed: false,
        })
    }

    pub async fn load_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Vec<ContactsOutboxRecordV1>, ContactsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(ContactsPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT message_id, envelope_sha256, envelope_bytes FROM makosh_data.contacts_outbox \
             WHERE logical_owner_id = $1 AND published_at_unix_millis IS NULL \
             ORDER BY created_at_unix_millis, message_id LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(CONTACTS_OUTBOX_LIMIT_V1))
        .fetch_all(&self.pool)
        .await
        .map_err(storage)?
        .iter()
        .map(decode_outbox)
        .collect()
    }

    pub async fn mark_outbox_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        published_at_unix_millis: i64,
    ) -> Result<(), ContactsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || message_id.iter().all(|byte| *byte == 0)
            || published_at_unix_millis <= 0
        {
            return Err(ContactsPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "UPDATE makosh_data.contacts_outbox SET published_at_unix_millis = $3 \
             WHERE logical_owner_id = $1 AND message_id = $2 \
             AND (published_at_unix_millis IS NULL OR published_at_unix_millis = $3)",
        )
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .bind(published_at_unix_millis)
        .execute(&self.pool)
        .await
        .map_err(storage)?;
        Ok(())
    }
}

async fn reserve_source_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &ReserveContactMailSyncSourceV1,
) -> Result<bool, ContactsPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.contacts_mail_sync_source_inbox (logical_owner_id, \
         command_message_id, command_envelope_sha256, operation_id, command_fingerprint, \
         contact_id, expected_contact_revision, target_mail_account_id, received_at_unix_millis) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT DO NOTHING",
    )
    .bind(&input.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .bind(input.command_envelope_sha256.as_slice())
    .bind(input.operation_id.as_slice())
    .bind(input.command_fingerprint().as_slice())
    .bind(input.contact_id.as_slice())
    .bind(i64_value(input.expected_contact_revision)?)
    .bind(&input.target_mail_account_id)
    .bind(input.received_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)
    .map(|result| result.rows_affected() == 1)
}

async fn load_source_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &ReserveContactMailSyncSourceV1,
) -> Result<Option<ContactMailSyncSourceResultV1>, ContactsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT command_envelope_sha256, operation_id, command_fingerprint, contact_id, \
         expected_contact_revision, target_mail_account_id, completed, reject_code, \
         result_message_id FROM makosh_data.contacts_mail_sync_source_inbox \
         WHERE logical_owner_id = $1 AND command_message_id = $2 FOR UPDATE",
    )
    .bind(&input.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(row) = row else {
        let reused_operation = sqlx::query_scalar::<_, bool>(
            "SELECT TRUE FROM makosh_data.contacts_mail_sync_source_inbox \
             WHERE logical_owner_id = $1 AND operation_id = $2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?
        .is_some();
        return Err(if reused_operation {
            ContactsPersistenceErrorV1::CommandConflict
        } else {
            ContactsPersistenceErrorV1::NotFound
        });
    };
    if bytes32(&row, "command_envelope_sha256")? != input.command_envelope_sha256
        || bytes16(&row, "operation_id")? != input.operation_id
        || bytes32(&row, "command_fingerprint")? != input.command_fingerprint()
        || bytes16(&row, "contact_id")? != input.contact_id
        || u64_value(row.get::<i64, _>("expected_contact_revision"))?
            != input.expected_contact_revision
        || row.get::<String, _>("target_mail_account_id") != input.target_mail_account_id
    {
        return Err(ContactsPersistenceErrorV1::InboxConflict);
    }
    if !row.get::<bool, _>("completed") {
        return Ok(None);
    }
    let reject_code = row
        .get::<Option<i16>, _>("reject_code")
        .map(decode_source_reject_code)
        .transpose()?;
    let terminal_result = load_outbox(
        transaction,
        &input.logical_owner_id,
        bytes16(&row, "result_message_id")?,
    )
    .await?;
    Ok(Some(ContactMailSyncSourceResultV1 {
        terminal_result,
        reject_code,
        replayed: true,
    }))
}

fn source_reservation(
    input: &PersistContactMailSyncSourceResultV1,
) -> ReserveContactMailSyncSourceV1 {
    ReserveContactMailSyncSourceV1 {
        command_message_id: input.command_message_id,
        command_envelope_sha256: input.command_envelope_sha256,
        operation_id: input.operation_id,
        contact_id: input.contact_id,
        expected_contact_revision: input.expected_contact_revision,
        target_mail_account_id: input.target_mail_account_id.clone(),
        logical_owner_id: input.logical_owner_id.clone(),
        received_at_unix_millis: input.received_at_unix_millis,
    }
}

async fn complete_source_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &PersistContactMailSyncSourceResultV1,
) -> Result<(), ContactsPersistenceErrorV1> {
    let result = sqlx::query(
        "UPDATE makosh_data.contacts_mail_sync_source_inbox SET completed = TRUE, \
         reject_code = $3, result_message_id = $4, completed_at_unix_millis = $5 \
         WHERE logical_owner_id = $1 AND command_message_id = $2 AND NOT completed",
    )
    .bind(&input.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .bind(input.reject_code.map(|value| value as i16))
    .bind(input.terminal_result.message_id.as_slice())
    .bind(input.completed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(ContactsPersistenceErrorV1::CommandConflict);
    }
    Ok(())
}

async fn reserve_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &ApplyMailEntryCommandV1,
    fingerprint: [u8; 32],
) -> Result<bool, ContactsPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.contacts_mail_entry_inbox (logical_owner_id, \
         command_message_id, command_envelope_sha256, command_id, command_fingerprint, \
         entry_digest, received_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7) \
         ON CONFLICT DO NOTHING",
    )
    .bind(&input.draft.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .bind(input.command_envelope_sha256.as_slice())
    .bind(input.command_id.as_slice())
    .bind(fingerprint.as_slice())
    .bind(input.draft.provenance.entry_digest.as_slice())
    .bind(input.received_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)
    .map(|result| result.rows_affected() == 1)
}

async fn reserve_rejected_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &RejectMailEntryCommandV1,
) -> Result<bool, ContactsPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.contacts_mail_entry_inbox (logical_owner_id, \
         command_message_id, command_envelope_sha256, command_id, command_fingerprint, \
         entry_digest, received_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7) \
         ON CONFLICT DO NOTHING",
    )
    .bind(&input.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .bind(input.command_envelope_sha256.as_slice())
    .bind(input.command_id.as_slice())
    .bind(input.command_fingerprint().as_slice())
    .bind(input.entry_digest.as_slice())
    .bind(input.received_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)
    .map(|result| result.rows_affected() == 1)
}

async fn load_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &ApplyMailEntryCommandV1,
) -> Result<AppliedMailEntryCommandV1, ContactsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT command_envelope_sha256, command_id, command_fingerprint, entry_digest, \
         completed, contact_id, contact_revision, outcome, reject_code, result_message_id \
         FROM makosh_data.contacts_mail_entry_inbox \
         WHERE logical_owner_id = $1 AND command_message_id = $2 FOR UPDATE",
    )
    .bind(&input.draft.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(row) = row else {
        let reused_command = sqlx::query_scalar::<_, bool>(
            "SELECT TRUE FROM makosh_data.contacts_mail_entry_inbox \
             WHERE logical_owner_id = $1 AND command_id = $2 FOR UPDATE",
        )
        .bind(&input.draft.logical_owner_id)
        .bind(input.command_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?
        .is_some();
        return Err(if reused_command {
            ContactsPersistenceErrorV1::CommandConflict
        } else {
            ContactsPersistenceErrorV1::NotFound
        });
    };
    let envelope_hash = bytes32(&row, "command_envelope_sha256")?;
    let command_id = bytes16(&row, "command_id")?;
    let fingerprint = bytes32(&row, "command_fingerprint")?;
    let entry_digest = bytes32(&row, "entry_digest")?;
    if envelope_hash != input.command_envelope_sha256
        || command_id != input.command_id
        || fingerprint != input.command_fingerprint()
        || entry_digest != input.draft.provenance.entry_digest
    {
        return Err(ContactsPersistenceErrorV1::InboxConflict);
    }
    if !row.get::<bool, _>("completed") {
        return Err(ContactsPersistenceErrorV1::CommandConflict);
    }
    if let Some(code) = row.get::<Option<i16>, _>("reject_code") {
        return Err(reject_error(decode_reject_code(code)?));
    }
    let outcome = decode_outcome(row.get::<i16, _>("outcome"))?;
    let result_message_id = bytes16(&row, "result_message_id")?;
    let terminal_result = load_outbox(
        transaction,
        &input.draft.logical_owner_id,
        result_message_id,
    )
    .await?;
    Ok(AppliedMailEntryCommandV1 {
        contact_id: bytes16(&row, "contact_id")?,
        contact_revision: u64_value(row.get("contact_revision"))?,
        outcome,
        terminal_result,
        replayed: true,
    })
}

async fn load_rejected_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &RejectMailEntryCommandV1,
) -> Result<RejectedMailEntryCommandV1, ContactsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT command_message_id, command_envelope_sha256, command_fingerprint, entry_digest, \
         completed, reject_code, result_message_id FROM makosh_data.contacts_mail_entry_inbox \
         WHERE logical_owner_id = $1 AND (command_message_id = $2 OR command_id = $3) \
         ORDER BY command_message_id = $2 DESC LIMIT 1 FOR UPDATE",
    )
    .bind(&input.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .bind(input.command_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?
    .ok_or(ContactsPersistenceErrorV1::NotFound)?;
    if bytes16(&row, "command_message_id")? != input.command_message_id {
        return Err(ContactsPersistenceErrorV1::CommandConflict);
    }
    if bytes32(&row, "command_envelope_sha256")? != input.command_envelope_sha256
        || bytes32(&row, "command_fingerprint")? != input.command_fingerprint()
        || bytes32(&row, "entry_digest")? != input.entry_digest
        || !row.get::<bool, _>("completed")
    {
        return Err(ContactsPersistenceErrorV1::InboxConflict);
    }
    let code = decode_reject_code(
        row.try_get::<Option<i16>, _>("reject_code")
            .map_err(|_| ContactsPersistenceErrorV1::InvalidRow)?
            .ok_or(ContactsPersistenceErrorV1::InvalidRow)?,
    )?;
    if code != input.code {
        return Err(ContactsPersistenceErrorV1::CommandConflict);
    }
    let terminal_result = load_outbox(
        transaction,
        &input.logical_owner_id,
        bytes16(&row, "result_message_id")?,
    )
    .await?;
    Ok(RejectedMailEntryCommandV1 {
        code,
        terminal_result,
        replayed: true,
    })
}

async fn load_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    message_id: [u8; 16],
) -> Result<ContactsOutboxRecordV1, ContactsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT message_id, envelope_sha256, envelope_bytes FROM makosh_data.contacts_outbox \
         WHERE logical_owner_id = $1 AND message_id = $2",
    )
    .bind(owner)
    .bind(message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?
    .ok_or(ContactsPersistenceErrorV1::InvalidRow)?;
    decode_outbox(&row)
}

async fn load_identity_match(
    transaction: &mut Transaction<'_, Postgres>,
    input: &ApplyMailEntryCommandV1,
) -> Result<ContactIdentityMatchV1, ContactsPersistenceErrorV1> {
    let owner = &input.draft.logical_owner_id;
    let provenance = &input.draft.provenance;
    let provider_link_contact_id = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT contact_id FROM makosh_data.contacts_provider_links \
         WHERE logical_owner_id = $1 AND provider_kind = $2 AND source_account_id = $3 \
         AND provider_entry_id = $4 FOR UPDATE",
    )
    .bind(owner)
    .bind(provider_kind(provenance.provider_kind))
    .bind(&provenance.source_account_id)
    .bind(&provenance.provider_entry_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?
    .map(|value| vec16(value, "provider link"))
    .transpose()?;
    let emails = normalized_emails(&input.draft.email_addresses)?;
    let phones = normalized_phones(&input.draft.phone_numbers)?;
    let email_contact_ids = load_identity_ids(
        transaction,
        "SELECT contact_id FROM makosh_data.contacts_email_identities \
         WHERE logical_owner_id = $1 AND normalized_email = ANY($2) FOR UPDATE",
        owner,
        &emails,
    )
    .await?;
    let phone_contact_ids = load_identity_ids(
        transaction,
        "SELECT contact_id FROM makosh_data.contacts_phone_identities \
         WHERE logical_owner_id = $1 AND normalized_phone = ANY($2) FOR UPDATE",
        owner,
        &phones,
    )
    .await?;
    Ok(ContactIdentityMatchV1 {
        provider_link_contact_id,
        email_contact_ids,
        phone_contact_ids,
    })
}

async fn load_identity_ids(
    transaction: &mut Transaction<'_, Postgres>,
    query: &'static str,
    owner: &str,
    identities: &[String],
) -> Result<Vec<[u8; 16]>, ContactsPersistenceErrorV1> {
    if identities.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_scalar::<_, Vec<u8>>(query)
        .bind(owner)
        .bind(identities)
        .fetch_all(&mut **transaction)
        .await
        .map_err(storage)?
        .into_iter()
        .map(|value| vec16(value, "identity"))
        .collect()
}

fn unique_target(
    identity_match: &ContactIdentityMatchV1,
) -> Result<Option<[u8; 16]>, ContactsPersistenceErrorV1> {
    let targets: BTreeSet<_> = identity_match
        .email_contact_ids
        .iter()
        .chain(&identity_match.phone_contact_ids)
        .copied()
        .collect();
    if targets.len() > 1 {
        return Err(ContactsPersistenceErrorV1::IdentityAmbiguous);
    }
    Ok(identity_match
        .provider_link_contact_id
        .or_else(|| targets.into_iter().next()))
}

async fn load_contact(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    contact_id: [u8; 16],
) -> Result<ContactV1, ContactsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT display_name, contact_revision, created_at_unix_seconds, created_at_nanos, \
         updated_at_unix_seconds, updated_at_nanos FROM makosh_data.contacts_state \
         WHERE logical_owner_id = $1 AND contact_id = $2 FOR UPDATE",
    )
    .bind(owner)
    .bind(contact_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?
    .ok_or(ContactsPersistenceErrorV1::NotFound)?;
    let emails = sqlx::query_scalar::<_, String>(
        "SELECT normalized_email FROM makosh_data.contacts_email_identities \
         WHERE logical_owner_id = $1 AND contact_id = $2 ORDER BY normalized_email",
    )
    .bind(owner)
    .bind(contact_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?;
    let phones = sqlx::query_scalar::<_, String>(
        "SELECT normalized_phone FROM makosh_data.contacts_phone_identities \
         WHERE logical_owner_id = $1 AND contact_id = $2 ORDER BY normalized_phone",
    )
    .bind(owner)
    .bind(contact_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?;
    let provenance = load_latest_provenance(transaction, owner, contact_id).await?;
    Ok(ContactV1 {
        contact_id,
        logical_owner_id: owner.to_owned(),
        display_name: row.get("display_name"),
        email_addresses: emails,
        phone_numbers: phones,
        contact_revision: u64_value(row.get("contact_revision"))?,
        provenance,
        created_at: ContactTimestampV1 {
            unix_seconds: row.get("created_at_unix_seconds"),
            nanos: row.get("created_at_nanos"),
        },
        updated_at: ContactTimestampV1 {
            unix_seconds: row.get("updated_at_unix_seconds"),
            nanos: row.get("updated_at_nanos"),
        },
    })
}

async fn load_latest_provenance(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    contact_id: [u8; 16],
) -> Result<ContactProviderProvenanceV1, ContactsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT provider_kind, source_account_id, provider_entry_id, provider_etag, \
         source_revision, entry_digest, observed_at_unix_seconds, observed_at_nanos \
         FROM makosh_data.contacts_provider_links WHERE logical_owner_id = $1 AND contact_id = $2 \
         ORDER BY observed_at_unix_seconds DESC, observed_at_nanos DESC, provider_entry_id LIMIT 1",
    )
    .bind(owner)
    .bind(contact_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?
    .ok_or(ContactsPersistenceErrorV1::InvalidRow)?;
    Ok(ContactProviderProvenanceV1 {
        source_account_id: row.get("source_account_id"),
        provider_kind: decode_provider(row.get("provider_kind"))?,
        provider_entry_id: row.get("provider_entry_id"),
        provider_etag: row.get("provider_etag"),
        source_revision: u64_value(row.get("source_revision"))?,
        entry_digest: bytes32(&row, "entry_digest")?,
        observed_at: ContactTimestampV1 {
            unix_seconds: row.get("observed_at_unix_seconds"),
            nanos: row.get("observed_at_nanos"),
        },
    })
}

async fn persist_contact(
    transaction: &mut Transaction<'_, Postgres>,
    contact: &ContactV1,
    outcome: ContactUpsertOutcomeV1,
) -> Result<(), ContactsPersistenceErrorV1> {
    if outcome == ContactUpsertOutcomeV1::Unchanged {
        return Err(ContactsPersistenceErrorV1::InvalidInput);
    }
    let result = sqlx::query(
        "INSERT INTO makosh_data.contacts_state (logical_owner_id, contact_id, display_name, \
         contact_revision, created_at_unix_seconds, created_at_nanos, updated_at_unix_seconds, \
         updated_at_nanos) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) \
         ON CONFLICT (logical_owner_id, contact_id) DO UPDATE SET display_name = EXCLUDED.display_name, \
         contact_revision = EXCLUDED.contact_revision, updated_at_unix_seconds = \
         EXCLUDED.updated_at_unix_seconds, updated_at_nanos = EXCLUDED.updated_at_nanos \
         WHERE makosh_data.contacts_state.contact_revision + 1 = EXCLUDED.contact_revision",
    )
    .bind(&contact.logical_owner_id)
    .bind(contact.contact_id.as_slice())
    .bind(&contact.display_name)
    .bind(i64_value(contact.contact_revision)?)
    .bind(contact.created_at.unix_seconds)
    .bind(contact.created_at.nanos)
    .bind(contact.updated_at.unix_seconds)
    .bind(contact.updated_at.nanos)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(ContactsPersistenceErrorV1::CommandConflict);
    }
    sqlx::query(
        "DELETE FROM makosh_data.contacts_email_identities WHERE logical_owner_id = $1 AND contact_id = $2",
    )
    .bind(&contact.logical_owner_id)
    .bind(contact.contact_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    sqlx::query(
        "DELETE FROM makosh_data.contacts_phone_identities WHERE logical_owner_id = $1 AND contact_id = $2",
    )
    .bind(&contact.logical_owner_id)
    .bind(contact.contact_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    for email in &contact.email_addresses {
        sqlx::query(
            "INSERT INTO makosh_data.contacts_email_identities (logical_owner_id, normalized_email, contact_id) VALUES ($1,$2,$3)",
        )
        .bind(&contact.logical_owner_id)
        .bind(email)
        .bind(contact.contact_id.as_slice())
        .execute(&mut **transaction)
        .await
        .map_err(identity_conflict)?;
    }
    for phone in &contact.phone_numbers {
        sqlx::query(
            "INSERT INTO makosh_data.contacts_phone_identities (logical_owner_id, normalized_phone, contact_id) VALUES ($1,$2,$3)",
        )
        .bind(&contact.logical_owner_id)
        .bind(phone)
        .bind(contact.contact_id.as_slice())
        .execute(&mut **transaction)
        .await
        .map_err(identity_conflict)?;
    }
    Ok(())
}

async fn persist_provider_link(
    transaction: &mut Transaction<'_, Postgres>,
    contact: &ContactV1,
) -> Result<(), ContactsPersistenceErrorV1> {
    let provenance = &contact.provenance;
    let result = sqlx::query(
        "INSERT INTO makosh_data.contacts_provider_links (logical_owner_id, provider_kind, \
         source_account_id, provider_entry_id, contact_id, provider_etag, source_revision, \
         entry_digest, observed_at_unix_seconds, observed_at_nanos) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) \
         ON CONFLICT (logical_owner_id, provider_kind, source_account_id, provider_entry_id) \
         DO UPDATE SET provider_etag = EXCLUDED.provider_etag, source_revision = EXCLUDED.source_revision, \
         entry_digest = EXCLUDED.entry_digest, observed_at_unix_seconds = EXCLUDED.observed_at_unix_seconds, \
         observed_at_nanos = EXCLUDED.observed_at_nanos WHERE makosh_data.contacts_provider_links.contact_id = \
         EXCLUDED.contact_id AND makosh_data.contacts_provider_links.source_revision <= EXCLUDED.source_revision",
    )
    .bind(&contact.logical_owner_id)
    .bind(provider_kind(provenance.provider_kind))
    .bind(&provenance.source_account_id)
    .bind(&provenance.provider_entry_id)
    .bind(contact.contact_id.as_slice())
    .bind(&provenance.provider_etag)
    .bind(i64_value(provenance.source_revision)?)
    .bind(provenance.entry_digest.as_slice())
    .bind(provenance.observed_at.unix_seconds)
    .bind(provenance.observed_at.nanos)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(ContactsPersistenceErrorV1::StaleSource);
    }
    Ok(())
}

async fn insert_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    record: &ContactsOutboxRecordV1,
    created_at_unix_millis: i64,
) -> Result<(), ContactsPersistenceErrorV1> {
    if !valid_outbox(record) {
        return Err(ContactsPersistenceErrorV1::InvalidInput);
    }
    let result = sqlx::query(
        "INSERT INTO makosh_data.contacts_outbox (logical_owner_id, message_id, envelope_sha256, \
         envelope_bytes, created_at_unix_millis) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
    )
    .bind(owner)
    .bind(record.message_id.as_slice())
    .bind(record.envelope_sha256.as_slice())
    .bind(&record.envelope_bytes)
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(ContactsPersistenceErrorV1::CommandConflict);
    }
    Ok(())
}

async fn complete_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &ApplyMailEntryCommandV1,
    contact: &ContactV1,
    outcome: ContactUpsertOutcomeV1,
    result_message_id: [u8; 16],
) -> Result<(), ContactsPersistenceErrorV1> {
    let result = sqlx::query(
        "UPDATE makosh_data.contacts_mail_entry_inbox SET completed = TRUE, contact_id = $3, \
         contact_revision = $4, outcome = $5, result_message_id = $6, \
         completed_at_unix_millis = $7 WHERE logical_owner_id = $1 AND \
         command_message_id = $2 AND NOT completed",
    )
    .bind(&input.draft.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .bind(contact.contact_id.as_slice())
    .bind(i64_value(contact.contact_revision)?)
    .bind(outcome_code(outcome))
    .bind(result_message_id.as_slice())
    .bind(input.completed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(ContactsPersistenceErrorV1::CommandConflict);
    }
    Ok(())
}

async fn complete_rejected_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &RejectMailEntryCommandV1,
) -> Result<(), ContactsPersistenceErrorV1> {
    let result = sqlx::query(
        "UPDATE makosh_data.contacts_mail_entry_inbox SET completed = TRUE, reject_code = $3, \
         result_message_id = $4, completed_at_unix_millis = $5 WHERE logical_owner_id = $1 \
         AND command_message_id = $2 AND NOT completed",
    )
    .bind(&input.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .bind(input.code as i16)
    .bind(input.terminal_result.message_id.as_slice())
    .bind(input.completed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(ContactsPersistenceErrorV1::CommandConflict);
    }
    Ok(())
}

fn decode_outbox(
    row: &sqlx::postgres::PgRow,
) -> Result<ContactsOutboxRecordV1, ContactsPersistenceErrorV1> {
    let record = ContactsOutboxRecordV1 {
        message_id: bytes16(row, "message_id")?,
        envelope_sha256: bytes32(row, "envelope_sha256")?,
        envelope_bytes: row.get("envelope_bytes"),
    };
    valid_outbox(&record)
        .then_some(record)
        .ok_or(ContactsPersistenceErrorV1::InvalidRow)
}

fn normalized_emails(values: &[String]) -> Result<Vec<String>, ContactsPersistenceErrorV1> {
    values
        .iter()
        .map(|value| {
            normalize_email_v1(value).map_err(|_| ContactsPersistenceErrorV1::InvalidInput)
        })
        .collect()
}

fn normalized_phones(values: &[String]) -> Result<Vec<String>, ContactsPersistenceErrorV1> {
    values
        .iter()
        .map(|value| {
            normalize_phone_v1(value).map_err(|_| ContactsPersistenceErrorV1::InvalidInput)
        })
        .collect()
}

fn provider_kind(value: ContactProviderKindV1) -> i16 {
    match value {
        ContactProviderKindV1::Gmail => 1,
        ContactProviderKindV1::Icloud => 2,
    }
}

fn decode_provider(value: i16) -> Result<ContactProviderKindV1, ContactsPersistenceErrorV1> {
    match value {
        1 => Ok(ContactProviderKindV1::Gmail),
        2 => Ok(ContactProviderKindV1::Icloud),
        _ => Err(ContactsPersistenceErrorV1::InvalidRow),
    }
}

fn outcome_code(value: ContactUpsertOutcomeV1) -> i16 {
    match value {
        ContactUpsertOutcomeV1::Created => 1,
        ContactUpsertOutcomeV1::Updated => 2,
        ContactUpsertOutcomeV1::Unchanged => 3,
    }
}

fn decode_outcome(value: i16) -> Result<ContactUpsertOutcomeV1, ContactsPersistenceErrorV1> {
    match value {
        1 => Ok(ContactUpsertOutcomeV1::Created),
        2 => Ok(ContactUpsertOutcomeV1::Updated),
        3 => Ok(ContactUpsertOutcomeV1::Unchanged),
        _ => Err(ContactsPersistenceErrorV1::InvalidRow),
    }
}

fn decode_reject_code(
    value: i16,
) -> Result<ContactMailEntryRejectCodeV1, ContactsPersistenceErrorV1> {
    match value {
        1 => Ok(ContactMailEntryRejectCodeV1::InvalidRequest),
        2 => Ok(ContactMailEntryRejectCodeV1::IdentityAmbiguous),
        3 => Ok(ContactMailEntryRejectCodeV1::ProviderLinkConflict),
        4 => Ok(ContactMailEntryRejectCodeV1::StaleSource),
        5 => Ok(ContactMailEntryRejectCodeV1::Policy),
        _ => Err(ContactsPersistenceErrorV1::InvalidRow),
    }
}

fn decode_source_reject_code(
    value: i16,
) -> Result<ContactMailSyncSourceRejectCodeV1, ContactsPersistenceErrorV1> {
    match value {
        1 => Ok(ContactMailSyncSourceRejectCodeV1::InvalidRequest),
        2 => Ok(ContactMailSyncSourceRejectCodeV1::ContactMissing),
        3 => Ok(ContactMailSyncSourceRejectCodeV1::StaleContactRevision),
        4 => Ok(ContactMailSyncSourceRejectCodeV1::ContentLimit),
        5 => Ok(ContactMailSyncSourceRejectCodeV1::Policy),
        _ => Err(ContactsPersistenceErrorV1::InvalidRow),
    }
}

fn reject_error(value: ContactMailEntryRejectCodeV1) -> ContactsPersistenceErrorV1 {
    match value {
        ContactMailEntryRejectCodeV1::InvalidRequest => ContactsPersistenceErrorV1::InvalidInput,
        ContactMailEntryRejectCodeV1::IdentityAmbiguous => {
            ContactsPersistenceErrorV1::IdentityAmbiguous
        }
        ContactMailEntryRejectCodeV1::ProviderLinkConflict => {
            ContactsPersistenceErrorV1::ProviderLinkConflict
        }
        ContactMailEntryRejectCodeV1::StaleSource => ContactsPersistenceErrorV1::StaleSource,
        ContactMailEntryRejectCodeV1::Policy => ContactsPersistenceErrorV1::PolicyRejected,
    }
}

fn map_decision(
    value: makosh_contacts_core::ContactUpsertDecisionErrorV1,
) -> ContactsPersistenceErrorV1 {
    use makosh_contacts_core::ContactUpsertDecisionErrorV1 as Error;
    match value {
        Error::InvalidDraft | Error::ExistingContactRequired => {
            ContactsPersistenceErrorV1::InvalidInput
        }
        Error::IdentityAmbiguous => ContactsPersistenceErrorV1::IdentityAmbiguous,
        Error::ProviderLinkConflict => ContactsPersistenceErrorV1::ProviderLinkConflict,
    }
}

fn identity_conflict(error: sqlx::Error) -> ContactsPersistenceErrorV1 {
    match &error {
        sqlx::Error::Database(database) if database.code().as_deref() == Some("23505") => {
            ContactsPersistenceErrorV1::IdentityAmbiguous
        }
        _ => ContactsPersistenceErrorV1::StorageUnavailable,
    }
}

fn storage(_: sqlx::Error) -> ContactsPersistenceErrorV1 {
    ContactsPersistenceErrorV1::StorageUnavailable
}

fn bytes16(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<[u8; 16], ContactsPersistenceErrorV1> {
    vec16(
        row.try_get(name)
            .map_err(|_| ContactsPersistenceErrorV1::InvalidRow)?,
        name,
    )
}

fn bytes32(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<[u8; 32], ContactsPersistenceErrorV1> {
    let value: Vec<u8> = row
        .try_get(name)
        .map_err(|_| ContactsPersistenceErrorV1::InvalidRow)?;
    value
        .try_into()
        .map_err(|_| ContactsPersistenceErrorV1::InvalidRow)
}

fn vec16(value: Vec<u8>, _: &str) -> Result<[u8; 16], ContactsPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| ContactsPersistenceErrorV1::InvalidRow)
}

fn i64_value(value: u64) -> Result<i64, ContactsPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| ContactsPersistenceErrorV1::InvalidInput)
}

fn u64_value(value: i64) -> Result<u64, ContactsPersistenceErrorV1> {
    u64::try_from(value).map_err(|_| ContactsPersistenceErrorV1::InvalidRow)
}
