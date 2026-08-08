//! Mail-owned provider reads for accepted address-book page commands.

use std::collections::BTreeSet;

use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_mail_address_book_contract::{
    MAIL_RUNTIME_MODULE_ID_V1, MailAddressBookEnvelopeContextV1,
    MailAddressBookResultEnvelopeContextV1, build_mail_address_book_entry_observed_v1,
    build_mail_address_book_page_completed_result_v1,
    build_mail_address_book_page_rejected_result_v1,
    wire::{
        MailAddressBookEntryObservedV1, MailAddressBookPageCompletedV1,
        MailAddressBookPageRejectedV1, MailAddressBookProviderKindV1, MailAddressBookRejectCodeV1,
    },
};
use makosh_mail_address_book_persistence::{
    MailAddressBookPersistenceErrorV1, PendingMailAddressBookFetchV1,
};
use makosh_mail_api::{MailAddressBookProviderV1, MailInboundTransportV1};
use makosh_mail_carddav::{CardDavAdapterErrorV1, CardDavContactV1};
use makosh_mail_google_people::{GooglePeopleAdapterErrorV1, GooglePeopleContactV1};
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    address_book_provider::{carddav_client_v1, google_people_client_v1},
    managed::{MailAdmittedRuntime, MailBootstrapError},
};

const GOOGLE_CURSOR_PREFIX: &[u8] = b"google-page-v1\0";
const CARDDAV_CURSOR_PREFIX: &[u8] = b"carddav-offset-v1\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookFetchWorkerErrorV1 {
    InvalidClock,
    Persistence,
    Envelope,
}

struct ProviderPageV1 {
    provider_kind: MailAddressBookProviderKindV1,
    entries: Vec<ProviderEntryV1>,
    next_cursor: Option<Vec<u8>>,
}

struct ProviderEntryV1 {
    provider_entry_id: String,
    provider_etag: Option<String>,
    display_name: String,
    email_addresses: Vec<String>,
    phone_numbers: Vec<String>,
}

pub async fn process_next_mail_address_book_fetch_v1(
    runtime: &mut MailAdmittedRuntime,
    now_unix_seconds: i64,
) -> Result<bool, MailAddressBookFetchWorkerErrorV1> {
    if now_unix_seconds <= 0 {
        return Err(MailAddressBookFetchWorkerErrorV1::InvalidClock);
    }
    let Some(job) = runtime
        .address_book_persistence
        .pending_fetches(1)
        .await
        .map_err(persistence_error)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    if runtime.select_account(&job.admission.account_id).is_err()
        || !runtime.provider_io_permitted()
    {
        persist_rejection(
            runtime,
            &job,
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeAccountUnavailable,
            now_unix_seconds,
        )
        .await?;
        return Ok(true);
    }
    let page = match fetch_provider_page(runtime, &job).await {
        Ok(page) => page,
        Err(code) => {
            persist_rejection(runtime, &job, code, now_unix_seconds).await?;
            return Ok(true);
        }
    };
    let records = match build_page_records(runtime, &job, page, now_unix_seconds) {
        Ok(records) => records,
        Err(MailAddressBookFetchWorkerErrorV1::Envelope) => {
            persist_rejection(
                runtime,
                &job,
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest,
                now_unix_seconds,
            )
            .await?;
            return Ok(true);
        }
        Err(error) => return Err(error),
    };
    runtime
        .address_book_persistence
        .complete_fetch_command(job.admission.command_id, &records, now_unix_seconds)
        .await
        .map_err(persistence_error)?;
    Ok(true)
}

async fn fetch_provider_page(
    runtime: &mut MailAdmittedRuntime,
    job: &PendingMailAddressBookFetchV1,
) -> Result<ProviderPageV1, MailAddressBookRejectCodeV1> {
    match runtime.address_book.provider {
        MailAddressBookProviderV1::GooglePeople => fetch_google_page(runtime, job).await,
        MailAddressBookProviderV1::IcloudCardDav => fetch_carddav_page(runtime, job).await,
        MailAddressBookProviderV1::None => {
            Err(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeAccountUnavailable)
        }
    }
}

async fn fetch_google_page(
    runtime: &mut MailAdmittedRuntime,
    job: &PendingMailAddressBookFetchV1,
) -> Result<ProviderPageV1, MailAddressBookRejectCodeV1> {
    if !matches!(runtime.account.inbound, MailInboundTransportV1::Gmail(_)) {
        return Err(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeAccountUnavailable);
    }
    let binding = runtime
        .durable
        .gmail_oauth_credential_binding(&job.admission.account_id)
        .await
        .map_err(|_| MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable)?
        .ok_or(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable)?;
    if !binding.contacts_write_authorized {
        return Err(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeWriteScopeRequired);
    }
    let token = runtime
        .resolve_gmail_access_token()
        .await
        .map_err(map_bootstrap_error)?;
    let token = std::str::from_utf8(&token)
        .map_err(|_| MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable)?;
    let cursor = decode_google_cursor(job.admission.continuation_cursor.as_deref())?;
    let page_size = u16::try_from(job.admission.page_size)
        .map_err(|_| MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest)?;
    let endpoint = runtime
        .address_book
        .google_people_endpoint
        .as_ref()
        .ok_or(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeAccountUnavailable)?;
    let page = google_people_client_v1(endpoint)
        .map_err(map_google_error)?
        .list_connections(token, cursor.as_deref(), None, page_size)
        .await
        .map_err(map_google_error)?;
    Ok(ProviderPageV1 {
        provider_kind: MailAddressBookProviderKindV1::MailAddressBookProviderKindGooglePeople,
        entries: page.contacts.into_iter().map(google_entry).collect(),
        next_cursor: page
            .next_page_token
            .as_deref()
            .map(encode_google_cursor)
            .transpose()?,
    })
}

async fn fetch_carddav_page(
    runtime: &MailAdmittedRuntime,
    job: &PendingMailAddressBookFetchV1,
) -> Result<ProviderPageV1, MailAddressBookRejectCodeV1> {
    if !matches!(runtime.account.inbound, MailInboundTransportV1::Imap(_)) {
        return Err(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeAccountUnavailable);
    }
    let (username, password) = runtime
        .carddav_credentials()
        .ok_or(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable)?;
    let endpoint = runtime
        .address_book
        .carddav_endpoint
        .as_ref()
        .ok_or(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeAccountUnavailable)?;
    let mut contacts = carddav_client_v1(endpoint)
        .map_err(map_carddav_error)?
        .list_contacts(username, password)
        .await
        .map_err(map_carddav_error)?;
    contacts.sort_by(|left, right| left.href.cmp(&right.href));
    let offset = decode_carddav_cursor(job.admission.continuation_cursor.as_deref())?;
    if offset > contacts.len() {
        return Err(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest);
    }
    let page_size = usize::try_from(job.admission.page_size)
        .map_err(|_| MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest)?;
    let total = contacts.len();
    let end = offset.saturating_add(page_size).min(total);
    let entries = contacts
        .drain(offset..end)
        .map(carddav_entry)
        .collect::<Vec<_>>();
    let next_cursor = (end < total).then(|| encode_carddav_cursor(end));
    Ok(ProviderPageV1 {
        provider_kind: MailAddressBookProviderKindV1::MailAddressBookProviderKindIcloudCarddav,
        entries,
        next_cursor,
    })
}

fn build_page_records(
    runtime: &MailAdmittedRuntime,
    job: &PendingMailAddressBookFetchV1,
    page: ProviderPageV1,
    now_unix_seconds: i64,
) -> Result<Vec<OutboxRecordV1>, MailAddressBookFetchWorkerErrorV1> {
    let context = MailAddressBookEnvelopeContextV1 {
        module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.clone(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: now_unix_seconds,
        recorded_at_nanos: 0,
    };
    let mut provider_ids = BTreeSet::new();
    let mut records = Vec::with_capacity(page.entries.len() + 1);
    for entry in page.entries {
        if !provider_ids.insert(entry.provider_entry_id.clone()) {
            return Err(MailAddressBookFetchWorkerErrorV1::Envelope);
        }
        let digest = entry_digest(&entry);
        let observation_id = observation_id(
            job.admission.command_id,
            job.admission.page_sequence,
            &entry.provider_entry_id,
        );
        let source_revision = source_revision(now_unix_seconds)?;
        let payload = MailAddressBookEntryObservedV1 {
            observation_id: observation_id.to_vec(),
            run_id: job.admission.run_id.to_vec(),
            logical_owner_id: job.admission.logical_owner_id.clone(),
            account_id: job.admission.account_id.clone(),
            provider_kind: page.provider_kind as i32,
            provider_entry_id: entry.provider_entry_id,
            provider_etag: entry.provider_etag,
            display_name: entry.display_name,
            email_addresses: entry.email_addresses,
            phone_numbers: entry.phone_numbers,
            observed_at: Some(Timestamp {
                seconds: now_unix_seconds,
                nanos: 0,
            }),
            source_revision,
            entry_digest: digest.to_vec(),
            page_sequence: job.admission.page_sequence,
        };
        records.push(
            build_mail_address_book_entry_observed_v1(
                job.admission.command_message_id,
                payload,
                &context,
            )
            .map_err(|_| MailAddressBookFetchWorkerErrorV1::Envelope)?,
        );
    }
    records.push(
        build_mail_address_book_page_completed_result_v1(
            job.admission.command_message_id,
            MailAddressBookPageCompletedV1 {
                command_id: job.admission.command_id.to_vec(),
                run_id: job.admission.run_id.to_vec(),
                page_sequence: job.admission.page_sequence,
                observed_entries: u32::try_from(records.len())
                    .map_err(|_| MailAddressBookFetchWorkerErrorV1::Envelope)?,
                next_continuation_cursor: page.next_cursor,
            },
            &result_context(runtime, job, now_unix_seconds),
        )
        .map_err(|_| MailAddressBookFetchWorkerErrorV1::Envelope)?,
    );
    Ok(records)
}

async fn persist_rejection(
    runtime: &MailAdmittedRuntime,
    job: &PendingMailAddressBookFetchV1,
    code: MailAddressBookRejectCodeV1,
    now_unix_seconds: i64,
) -> Result<(), MailAddressBookFetchWorkerErrorV1> {
    let retryable = matches!(
        code,
        MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable
            | MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable
    );
    let result = build_mail_address_book_page_rejected_result_v1(
        job.admission.command_message_id,
        MailAddressBookPageRejectedV1 {
            command_id: job.admission.command_id.to_vec(),
            run_id: job.admission.run_id.to_vec(),
            code: code as i32,
            retryable,
        },
        &result_context(runtime, job, now_unix_seconds),
    )
    .map_err(|_| MailAddressBookFetchWorkerErrorV1::Envelope)?;
    runtime
        .address_book_persistence
        .complete_fetch_command(job.admission.command_id, &[result], now_unix_seconds)
        .await
        .map_err(persistence_error)?;
    Ok(())
}

fn result_context(
    runtime: &MailAdmittedRuntime,
    job: &PendingMailAddressBookFetchV1,
    now_unix_seconds: i64,
) -> MailAddressBookResultEnvelopeContextV1 {
    MailAddressBookResultEnvelopeContextV1 {
        runtime_instance_id: runtime.runtime_instance_id.clone(),
        runtime_generation: runtime.runtime_generation,
        completed_at_unix_seconds: now_unix_seconds,
        completed_at_nanos: 0,
        execution_attempt: job.execution_attempt,
    }
}

fn google_entry(contact: GooglePeopleContactV1) -> ProviderEntryV1 {
    ProviderEntryV1 {
        provider_entry_id: contact.resource_name,
        provider_etag: Some(contact.etag),
        display_name: contact.display_name,
        email_addresses: canonical_values(contact.email_addresses),
        phone_numbers: canonical_values(contact.phone_numbers),
    }
}

fn carddav_entry(contact: CardDavContactV1) -> ProviderEntryV1 {
    ProviderEntryV1 {
        provider_entry_id: contact.href,
        provider_etag: Some(contact.etag),
        display_name: contact.display_name,
        email_addresses: canonical_values(contact.email_addresses),
        phone_numbers: canonical_values(contact.phone_numbers),
    }
}

fn entry_digest(entry: &ProviderEntryV1) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hash_field(&mut hasher, b"provider_entry_id", &entry.provider_entry_id);
    match entry.provider_etag.as_deref() {
        Some(value) => hash_field(&mut hasher, b"provider_etag", value),
        None => hasher.update(b"provider_etag\0none\0"),
    }
    hash_field(&mut hasher, b"display_name", &entry.display_name);
    hasher.update(b"email_addresses\0");
    hasher.update(entry.email_addresses.len().to_be_bytes());
    for value in &entry.email_addresses {
        hash_field(&mut hasher, b"email", value);
    }
    hasher.update(b"phone_numbers\0");
    hasher.update(entry.phone_numbers.len().to_be_bytes());
    for value in &entry.phone_numbers {
        hash_field(&mut hasher, b"phone", value);
    }
    hasher.finalize().into()
}

fn hash_field(hasher: &mut Sha256, label: &[u8], value: &str) {
    hasher.update(label.len().to_be_bytes());
    hasher.update(label);
    hasher.update(value.len().to_be_bytes());
    hasher.update(value.as_bytes());
}

fn canonical_values(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn observation_id(command_id: [u8; 16], page_sequence: u64, provider_id: &str) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"mail-address-book-observation-v1");
    hasher.update(command_id);
    hasher.update(page_sequence.to_be_bytes());
    hasher.update(provider_id.len().to_be_bytes());
    hasher.update(provider_id.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16].try_into().expect("fixed digest width")
}

fn source_revision(
    observed_at_unix_seconds: i64,
) -> Result<u64, MailAddressBookFetchWorkerErrorV1> {
    u64::try_from(observed_at_unix_seconds)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(MailAddressBookFetchWorkerErrorV1::InvalidClock)
}

fn encode_google_cursor(token: &str) -> Result<Vec<u8>, MailAddressBookRejectCodeV1> {
    if token.is_empty() || !token.is_ascii() || token.len() > 4_000 {
        return Err(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest);
    }
    let mut cursor = GOOGLE_CURSOR_PREFIX.to_vec();
    cursor.extend_from_slice(token.as_bytes());
    Ok(cursor)
}

fn decode_google_cursor(
    cursor: Option<&[u8]>,
) -> Result<Option<String>, MailAddressBookRejectCodeV1> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    let token = cursor
        .strip_prefix(GOOGLE_CURSOR_PREFIX)
        .ok_or(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest)?;
    let token = std::str::from_utf8(token)
        .map_err(|_| MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest)?;
    if token.is_empty() || token.len() > 4_000 {
        return Err(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest);
    }
    Ok(Some(token.to_owned()))
}

fn encode_carddav_cursor(offset: usize) -> Vec<u8> {
    let mut cursor = CARDDAV_CURSOR_PREFIX.to_vec();
    cursor.extend_from_slice(&u64::try_from(offset).unwrap_or(u64::MAX).to_be_bytes());
    cursor
}

fn decode_carddav_cursor(cursor: Option<&[u8]>) -> Result<usize, MailAddressBookRejectCodeV1> {
    let Some(cursor) = cursor else {
        return Ok(0);
    };
    let offset = cursor
        .strip_prefix(CARDDAV_CURSOR_PREFIX)
        .and_then(|value| <[u8; 8]>::try_from(value).ok())
        .map(u64::from_be_bytes)
        .ok_or(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest)?;
    usize::try_from(offset)
        .map_err(|_| MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest)
}

fn map_bootstrap_error(error: MailBootstrapError) -> MailAddressBookRejectCodeV1 {
    match error {
        MailBootstrapError::Persistence => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable
        }
        _ => MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable,
    }
}

fn map_google_error(error: GooglePeopleAdapterErrorV1) -> MailAddressBookRejectCodeV1 {
    match error {
        GooglePeopleAdapterErrorV1::Unavailable
        | GooglePeopleAdapterErrorV1::OutcomeUnknown
        | GooglePeopleAdapterErrorV1::InvalidResponse => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable
        }
        GooglePeopleAdapterErrorV1::ProviderRejected(401 | 403) => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable
        }
        GooglePeopleAdapterErrorV1::ProviderRejected(429 | 500..=599) => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable
        }
        GooglePeopleAdapterErrorV1::InvalidRequest
        | GooglePeopleAdapterErrorV1::EtagConflict
        | GooglePeopleAdapterErrorV1::ProviderRejected(_) => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest
        }
    }
}

fn map_carddav_error(error: CardDavAdapterErrorV1) -> MailAddressBookRejectCodeV1 {
    match error {
        CardDavAdapterErrorV1::Unavailable | CardDavAdapterErrorV1::InvalidResponse => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable
        }
        CardDavAdapterErrorV1::ProviderRejected(401 | 403) => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable
        }
        CardDavAdapterErrorV1::ProviderRejected(429 | 500..=599) => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable
        }
        CardDavAdapterErrorV1::InvalidRequest
        | CardDavAdapterErrorV1::ProviderRejected(_)
        | CardDavAdapterErrorV1::DiscoveryFailed
        | CardDavAdapterErrorV1::ReadOnlyProvider => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest
        }
    }
}

fn persistence_error(_: MailAddressBookPersistenceErrorV1) -> MailAddressBookFetchWorkerErrorV1 {
    MailAddressBookFetchWorkerErrorV1::Persistence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_cursors_are_typed_and_not_cross_decoded() {
        let google = encode_google_cursor("next-token").expect("google cursor");
        assert_eq!(
            decode_google_cursor(Some(&google)).unwrap().as_deref(),
            Some("next-token")
        );
        assert!(decode_carddav_cursor(Some(&google)).is_err());
        let carddav = encode_carddav_cursor(17);
        assert_eq!(decode_carddav_cursor(Some(&carddav)), Ok(17));
        assert!(decode_google_cursor(Some(&carddav)).is_err());
    }

    #[test]
    fn provider_entry_identity_is_stable_and_content_sensitive() {
        let entry = ProviderEntryV1 {
            provider_entry_id: "people/1".to_owned(),
            provider_etag: Some("etag-1".to_owned()),
            display_name: "Ada".to_owned(),
            email_addresses: vec!["ada@example.test".to_owned()],
            phone_numbers: Vec::new(),
        };
        assert_eq!(
            observation_id([1; 16], 2, "people/1"),
            observation_id([1; 16], 2, "people/1")
        );
        assert_ne!(
            observation_id([1; 16], 2, "people/1"),
            observation_id([1; 16], 3, "people/1")
        );
        assert_eq!(source_revision(1_800_000_000), Ok(1_800_000_000));
        assert_eq!(
            source_revision(0),
            Err(MailAddressBookFetchWorkerErrorV1::InvalidClock)
        );
        let mut reordered = entry;
        reordered.email_addresses = vec![
            "second@example.test".to_owned(),
            "ada@example.test".to_owned(),
            "ada@example.test".to_owned(),
        ];
        let canonical = canonical_values(reordered.email_addresses);
        assert_eq!(
            canonical,
            vec![
                "ada@example.test".to_owned(),
                "second@example.test".to_owned()
            ]
        );
    }
}
