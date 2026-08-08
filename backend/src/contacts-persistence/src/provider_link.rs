use makosh_contacts_core::ContactProviderKindV1;
use sqlx::{Postgres, Row, Transaction};

use crate::{
    AppliedMailProviderLinkCommandV1, BindMailProviderLinkCommandV1,
    ContactProviderLinkBindOutcomeV1, ContactProviderLinkBindRejectCodeV1, ContactsOutboxRecordV1,
    ContactsPersistenceErrorV1, ContactsPersistenceV1,
};

impl ContactsPersistenceV1 {
    pub async fn bind_mail_provider_link<F>(
        &self,
        input: &BindMailProviderLinkCommandV1,
        build_terminal: F,
    ) -> Result<AppliedMailProviderLinkCommandV1, ContactsPersistenceErrorV1>
    where
        F: FnOnce(
            ContactProviderLinkBindOutcomeV1,
        ) -> Result<ContactsOutboxRecordV1, ContactsPersistenceErrorV1>,
    {
        validate_input(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        if !reserve(&mut transaction, input).await? {
            let replay = load_replay(&mut transaction, input).await?;
            transaction.commit().await.map_err(storage)?;
            return Ok(replay);
        }
        let outcome = decide_and_apply(&mut transaction, input).await?;
        let terminal = build_terminal(outcome)?;
        if !crate::model::valid_outbox(&terminal) {
            return Err(ContactsPersistenceErrorV1::InvalidInput);
        }
        insert_outbox(&mut transaction, input, &terminal).await?;
        complete(&mut transaction, input, outcome, terminal.message_id).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(AppliedMailProviderLinkCommandV1 {
            outcome,
            terminal_result: terminal,
            replayed: false,
        })
    }
}

fn validate_input(input: &BindMailProviderLinkCommandV1) -> Result<(), ContactsPersistenceErrorV1> {
    if !valid_owner(&input.logical_owner_id)
        || input.command_message_id.iter().all(|value| *value == 0)
        || input
            .command_envelope_sha256
            .iter()
            .all(|value| *value == 0)
        || input.command_id.iter().all(|value| *value == 0)
        || input.contact_id.iter().all(|value| *value == 0)
        || input.expected_contact_revision == 0
        || !valid_text(&input.source_account_id, 256)
        || !valid_text(&input.provider_entry_id, 512)
        || input
            .provider_etag
            .as_deref()
            .is_some_and(|value| !valid_text(value, 512))
        || input.received_at_unix_millis <= 0
        || input.completed_at_unix_millis < input.received_at_unix_millis
    {
        return Err(ContactsPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

async fn reserve(
    transaction: &mut Transaction<'_, Postgres>,
    input: &BindMailProviderLinkCommandV1,
) -> Result<bool, ContactsPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.contacts_mail_provider_link_inbox (logical_owner_id, \
         command_message_id, command_envelope_sha256, command_id, contact_id, \
         expected_contact_revision, source_account_id, provider_kind, provider_entry_id, \
         provider_etag, received_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) \
         ON CONFLICT DO NOTHING",
    )
    .bind(&input.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .bind(input.command_envelope_sha256.as_slice())
    .bind(input.command_id.as_slice())
    .bind(input.contact_id.as_slice())
    .bind(
        i64::try_from(input.expected_contact_revision)
            .map_err(|_| ContactsPersistenceErrorV1::InvalidInput)?,
    )
    .bind(&input.source_account_id)
    .bind(provider_kind(input.provider_kind))
    .bind(&input.provider_entry_id)
    .bind(&input.provider_etag)
    .bind(input.received_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)
    .map(|result| result.rows_affected() == 1)
}

async fn decide_and_apply(
    transaction: &mut Transaction<'_, Postgres>,
    input: &BindMailProviderLinkCommandV1,
) -> Result<ContactProviderLinkBindOutcomeV1, ContactsPersistenceErrorV1> {
    let revision = sqlx::query_scalar::<_, i64>(
        "SELECT contact_revision FROM makosh_data.contacts_state WHERE logical_owner_id=$1 \
         AND contact_id=$2 FOR UPDATE",
    )
    .bind(&input.logical_owner_id)
    .bind(input.contact_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(revision) = revision else {
        return Ok(ContactProviderLinkBindOutcomeV1::Rejected(
            ContactProviderLinkBindRejectCodeV1::ContactMissing,
        ));
    };
    if u64::try_from(revision).ok() != Some(input.expected_contact_revision) {
        return Ok(ContactProviderLinkBindOutcomeV1::Rejected(
            ContactProviderLinkBindRejectCodeV1::StaleContactRevision,
        ));
    }
    let collision = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT contact_id FROM makosh_data.contacts_provider_links WHERE logical_owner_id=$1 \
         AND provider_kind=$2 AND source_account_id=$3 AND provider_entry_id=$4 FOR UPDATE",
    )
    .bind(&input.logical_owner_id)
    .bind(provider_kind(input.provider_kind))
    .bind(&input.source_account_id)
    .bind(&input.provider_entry_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    if collision
        .as_deref()
        .is_some_and(|value| value != input.contact_id)
    {
        return Ok(ContactProviderLinkBindOutcomeV1::Rejected(
            ContactProviderLinkBindRejectCodeV1::ProviderLinkConflict,
        ));
    }
    let existing = sqlx::query_scalar::<_, String>(
        "SELECT provider_entry_id FROM makosh_data.contacts_provider_links WHERE \
         logical_owner_id=$1 AND contact_id=$2 AND source_account_id=$3 ORDER BY provider_kind, \
         provider_entry_id LIMIT 1 FOR UPDATE",
    )
    .bind(&input.logical_owner_id)
    .bind(input.contact_id.as_slice())
    .bind(&input.source_account_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    if existing
        .as_deref()
        .is_some_and(|value| value != input.provider_entry_id)
    {
        return Ok(ContactProviderLinkBindOutcomeV1::Rejected(
            ContactProviderLinkBindRejectCodeV1::ProviderLinkConflict,
        ));
    }
    sqlx::query(
        "INSERT INTO makosh_data.contacts_provider_links (logical_owner_id, provider_kind, \
         source_account_id, provider_entry_id, contact_id, provider_etag, source_revision, \
         entry_digest, observed_at_unix_seconds, observed_at_nanos) VALUES \
         ($1,$2,$3,$4,$5,$6,$7,$8,$9,0) ON CONFLICT (logical_owner_id, provider_kind, \
         source_account_id, provider_entry_id) DO UPDATE SET provider_etag=EXCLUDED.provider_etag WHERE \
         makosh_data.contacts_provider_links.contact_id=EXCLUDED.contact_id",
    )
    .bind(&input.logical_owner_id)
    .bind(provider_kind(input.provider_kind))
    .bind(&input.source_account_id)
    .bind(&input.provider_entry_id)
    .bind(input.contact_id.as_slice())
    .bind(&input.provider_etag)
    .bind(revision)
    .bind(input.command_envelope_sha256.as_slice())
    .bind(input.completed_at_unix_millis / 1_000)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    Ok(ContactProviderLinkBindOutcomeV1::Bound {
        contact_revision: input.expected_contact_revision,
    })
}

async fn insert_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &BindMailProviderLinkCommandV1,
    terminal: &ContactsOutboxRecordV1,
) -> Result<(), ContactsPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.contacts_outbox (logical_owner_id, message_id, envelope_sha256, \
         envelope_bytes, created_at_unix_millis) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
    )
    .bind(&input.logical_owner_id)
    .bind(terminal.message_id.as_slice())
    .bind(terminal.envelope_sha256.as_slice())
    .bind(&terminal.envelope_bytes)
    .bind(input.completed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if inserted.rows_affected() != 1 {
        return Err(ContactsPersistenceErrorV1::CommandConflict);
    }
    Ok(())
}

async fn complete(
    transaction: &mut Transaction<'_, Postgres>,
    input: &BindMailProviderLinkCommandV1,
    outcome: ContactProviderLinkBindOutcomeV1,
    result_message_id: [u8; 16],
) -> Result<(), ContactsPersistenceErrorV1> {
    let reject_code = match outcome {
        ContactProviderLinkBindOutcomeV1::Bound { .. } => None,
        ContactProviderLinkBindOutcomeV1::Rejected(code) => Some(code as i16),
    };
    let updated = sqlx::query(
        "UPDATE makosh_data.contacts_mail_provider_link_inbox SET completed=TRUE, reject_code=$3, \
         result_message_id=$4, completed_at_unix_millis=$5 WHERE logical_owner_id=$1 AND \
         command_message_id=$2 AND NOT completed",
    )
    .bind(&input.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .bind(reject_code)
    .bind(result_message_id.as_slice())
    .bind(input.completed_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if updated.rows_affected() != 1 {
        return Err(ContactsPersistenceErrorV1::CommandConflict);
    }
    Ok(())
}

async fn load_replay(
    transaction: &mut Transaction<'_, Postgres>,
    input: &BindMailProviderLinkCommandV1,
) -> Result<AppliedMailProviderLinkCommandV1, ContactsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT inbox.command_envelope_sha256, inbox.command_id, inbox.contact_id, \
         inbox.expected_contact_revision, inbox.source_account_id, inbox.provider_kind, \
         inbox.provider_entry_id, inbox.provider_etag, inbox.completed, inbox.reject_code, \
         outbox.message_id, outbox.envelope_sha256, outbox.envelope_bytes FROM \
         makosh_data.contacts_mail_provider_link_inbox AS inbox LEFT JOIN \
         makosh_data.contacts_outbox AS outbox ON outbox.logical_owner_id=inbox.logical_owner_id \
         AND outbox.message_id=inbox.result_message_id WHERE inbox.logical_owner_id=$1 AND \
         inbox.command_message_id=$2 FOR UPDATE OF inbox",
    )
    .bind(&input.logical_owner_id)
    .bind(input.command_message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?
    .ok_or(ContactsPersistenceErrorV1::InboxConflict)?;
    let exact = row.get::<Vec<u8>, _>("command_envelope_sha256").as_slice()
        == input.command_envelope_sha256
        && row.get::<Vec<u8>, _>("command_id").as_slice() == input.command_id
        && row.get::<Vec<u8>, _>("contact_id").as_slice() == input.contact_id
        && u64::try_from(row.get::<i64, _>("expected_contact_revision")).ok()
            == Some(input.expected_contact_revision)
        && row.get::<String, _>("source_account_id") == input.source_account_id
        && row.get::<i16, _>("provider_kind") == provider_kind(input.provider_kind)
        && row.get::<String, _>("provider_entry_id") == input.provider_entry_id
        && row.get::<Option<String>, _>("provider_etag") == input.provider_etag;
    if !exact || !row.get::<bool, _>("completed") {
        return Err(ContactsPersistenceErrorV1::InboxConflict);
    }
    let reject_code = row.get::<Option<i16>, _>("reject_code");
    let outcome = match reject_code {
        None => ContactProviderLinkBindOutcomeV1::Bound {
            contact_revision: input.expected_contact_revision,
        },
        Some(value) => ContactProviderLinkBindOutcomeV1::Rejected(decode_reject(value)?),
    };
    Ok(AppliedMailProviderLinkCommandV1 {
        outcome,
        terminal_result: ContactsOutboxRecordV1 {
            message_id: bytes::<16>(&row, "message_id")?,
            envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
            envelope_bytes: row.get("envelope_bytes"),
        },
        replayed: true,
    })
}

fn provider_kind(value: ContactProviderKindV1) -> i16 {
    match value {
        ContactProviderKindV1::Gmail => 1,
        ContactProviderKindV1::Icloud => 2,
    }
}

fn decode_reject(
    value: i16,
) -> Result<ContactProviderLinkBindRejectCodeV1, ContactsPersistenceErrorV1> {
    match value {
        1 => Ok(ContactProviderLinkBindRejectCodeV1::InvalidRequest),
        2 => Ok(ContactProviderLinkBindRejectCodeV1::ContactMissing),
        3 => Ok(ContactProviderLinkBindRejectCodeV1::StaleContactRevision),
        4 => Ok(ContactProviderLinkBindRejectCodeV1::ProviderLinkConflict),
        5 => Ok(ContactProviderLinkBindRejectCodeV1::Policy),
        _ => Err(ContactsPersistenceErrorV1::InvalidRow),
    }
}

fn bytes<const N: usize>(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<[u8; N], ContactsPersistenceErrorV1> {
    row.get::<Option<Vec<u8>>, _>(name)
        .ok_or(ContactsPersistenceErrorV1::InvalidRow)?
        .try_into()
        .map_err(|_| ContactsPersistenceErrorV1::InvalidRow)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

fn storage(_: sqlx::Error) -> ContactsPersistenceErrorV1 {
    ContactsPersistenceErrorV1::StorageUnavailable
}
