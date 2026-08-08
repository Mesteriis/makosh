use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use crate::{
    DIGEST_BYTES_V1, MAX_ACCOUNT_ID_BYTES_V1, MAX_DISPLAY_NAME_CHARS_V1, MAX_EMAILS_V1,
    MAX_LOGICAL_OWNER_ID_BYTES_V1, MAX_PHONES_V1, MAX_PROVIDER_ENTRY_ID_BYTES_V1,
    MAX_PROVIDER_ETAG_BYTES_V1, STABLE_ID_BYTES_V1, normalize_email_v1, normalize_phone_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactProviderKindV1 {
    Gmail,
    Icloud,
}

impl ContactProviderKindV1 {
    pub(crate) fn label(self) -> &'static [u8] {
        match self {
            Self::Gmail => b"gmail",
            Self::Icloud => b"icloud",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContactTimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactProviderProvenanceV1 {
    pub source_account_id: String,
    pub provider_kind: ContactProviderKindV1,
    pub provider_entry_id: String,
    pub provider_etag: Option<String>,
    pub source_revision: u64,
    pub entry_digest: [u8; DIGEST_BYTES_V1],
    pub observed_at: ContactTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactUpsertDraftV1 {
    pub logical_owner_id: String,
    pub display_name: String,
    pub email_addresses: Vec<String>,
    pub phone_numbers: Vec<String>,
    pub provenance: ContactProviderProvenanceV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactV1 {
    pub contact_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub display_name: String,
    pub email_addresses: Vec<String>,
    pub phone_numbers: Vec<String>,
    pub contact_revision: u64,
    pub provenance: ContactProviderProvenanceV1,
    pub created_at: ContactTimestampV1,
    pub updated_at: ContactTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactIdentityMatchV1 {
    pub provider_link_contact_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    pub email_contact_ids: Vec<[u8; STABLE_ID_BYTES_V1]>,
    pub phone_contact_ids: Vec<[u8; STABLE_ID_BYTES_V1]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactUpsertOutcomeV1 {
    Created,
    Updated,
    Unchanged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactsValidationErrorV1 {
    InvalidOwner,
    InvalidContactId,
    InvalidAccountId,
    InvalidProviderEntryId,
    InvalidProviderEtag,
    InvalidDisplayName,
    InvalidEmail,
    InvalidPhone,
    MissingIdentity,
    InvalidSourceRevision,
    InvalidDigest,
    InvalidTimestamp,
    InvalidRevision,
    DuplicateIdentity,
}

pub fn derive_contact_id_v1(
    logical_owner_id: &str,
    provenance: &ContactProviderProvenanceV1,
) -> Result<[u8; STABLE_ID_BYTES_V1], ContactsValidationErrorV1> {
    validate_owner(logical_owner_id)?;
    validate_provenance(provenance)?;
    let mut hasher = Sha256::new();
    update_part(&mut hasher, b"makosh.contacts.mail-provider-entry.id.v1");
    update_part(&mut hasher, logical_owner_id.as_bytes());
    update_part(&mut hasher, provenance.provider_kind.label());
    update_part(&mut hasher, provenance.source_account_id.as_bytes());
    update_part(&mut hasher, provenance.provider_entry_id.as_bytes());
    Ok(hasher.finalize()[..STABLE_ID_BYTES_V1]
        .try_into()
        .expect("fixed digest"))
}

pub fn upsert_fingerprint_v1(
    draft: &ContactUpsertDraftV1,
) -> Result<[u8; DIGEST_BYTES_V1], ContactsValidationErrorV1> {
    let normalized = normalize_draft(draft)?;
    let mut hasher = Sha256::new();
    update_part(
        &mut hasher,
        b"makosh.contacts.mail-provider-entry.upsert.v1",
    );
    update_part(&mut hasher, normalized.logical_owner_id.as_bytes());
    update_part(&mut hasher, normalized.display_name.as_bytes());
    for email in &normalized.email_addresses {
        update_part(&mut hasher, email.as_bytes());
    }
    for phone in &normalized.phone_numbers {
        update_part(&mut hasher, phone.as_bytes());
    }
    update_part(&mut hasher, &normalized.provenance.entry_digest);
    update_part(
        &mut hasher,
        &normalized.provenance.source_revision.to_be_bytes(),
    );
    Ok(hasher.finalize().into())
}

pub fn validate_contact_v1(contact: &ContactV1) -> Result<(), ContactsValidationErrorV1> {
    if !nonzero(&contact.contact_id) {
        return Err(ContactsValidationErrorV1::InvalidContactId);
    }
    validate_owner(&contact.logical_owner_id)?;
    validate_content(
        &contact.display_name,
        &contact.email_addresses,
        &contact.phone_numbers,
        true,
    )?;
    validate_provenance(&contact.provenance)?;
    if contact.contact_revision == 0 {
        return Err(ContactsValidationErrorV1::InvalidRevision);
    }
    if !valid_timestamp(contact.created_at)
        || !valid_timestamp(contact.updated_at)
        || contact.updated_at.unix_seconds < contact.created_at.unix_seconds
    {
        return Err(ContactsValidationErrorV1::InvalidTimestamp);
    }
    Ok(())
}

pub(crate) fn normalize_draft(
    draft: &ContactUpsertDraftV1,
) -> Result<ContactUpsertDraftV1, ContactsValidationErrorV1> {
    validate_owner(&draft.logical_owner_id)?;
    validate_provenance(&draft.provenance)?;
    validate_content(
        &draft.display_name,
        &draft.email_addresses,
        &draft.phone_numbers,
        false,
    )?;
    let emails = normalized_unique(&draft.email_addresses, normalize_email_v1)?;
    let phones = normalized_unique(&draft.phone_numbers, normalize_phone_v1)?;
    if emails.is_empty() && phones.is_empty() && draft.display_name.trim().is_empty() {
        return Err(ContactsValidationErrorV1::MissingIdentity);
    }
    Ok(ContactUpsertDraftV1 {
        logical_owner_id: draft.logical_owner_id.clone(),
        display_name: draft.display_name.trim().to_owned(),
        email_addresses: emails,
        phone_numbers: phones,
        provenance: draft.provenance.clone(),
    })
}

fn normalized_unique(
    values: &[String],
    normalize: fn(&str) -> Result<String, ContactsValidationErrorV1>,
) -> Result<Vec<String>, ContactsValidationErrorV1> {
    let mut normalized = BTreeSet::new();
    for value in values {
        if !normalized.insert(normalize(value)?) {
            return Err(ContactsValidationErrorV1::DuplicateIdentity);
        }
    }
    Ok(normalized.into_iter().collect())
}

fn validate_content(
    display_name: &str,
    emails: &[String],
    phones: &[String],
    normalized: bool,
) -> Result<(), ContactsValidationErrorV1> {
    if display_name.chars().count() > MAX_DISPLAY_NAME_CHARS_V1
        || display_name.chars().any(char::is_control)
    {
        return Err(ContactsValidationErrorV1::InvalidDisplayName);
    }
    if emails.len() > MAX_EMAILS_V1 || phones.len() > MAX_PHONES_V1 {
        return Err(ContactsValidationErrorV1::DuplicateIdentity);
    }
    if normalized {
        for email in emails {
            if normalize_email_v1(email)? != *email {
                return Err(ContactsValidationErrorV1::InvalidEmail);
            }
        }
        for phone in phones {
            if normalize_phone_v1(phone)? != *phone {
                return Err(ContactsValidationErrorV1::InvalidPhone);
            }
        }
    }
    Ok(())
}

fn validate_owner(value: &str) -> Result<(), ContactsValidationErrorV1> {
    if value.is_empty()
        || value.len() > MAX_LOGICAL_OWNER_ID_BYTES_V1
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        return Err(ContactsValidationErrorV1::InvalidOwner);
    }
    Ok(())
}

fn validate_provenance(
    value: &ContactProviderProvenanceV1,
) -> Result<(), ContactsValidationErrorV1> {
    if value.source_account_id.trim().is_empty()
        || value.source_account_id.len() > MAX_ACCOUNT_ID_BYTES_V1
        || value.source_account_id.chars().any(char::is_control)
    {
        return Err(ContactsValidationErrorV1::InvalidAccountId);
    }
    if value.provider_entry_id.trim().is_empty()
        || value.provider_entry_id.len() > MAX_PROVIDER_ENTRY_ID_BYTES_V1
        || value.provider_entry_id.chars().any(char::is_control)
    {
        return Err(ContactsValidationErrorV1::InvalidProviderEntryId);
    }
    if value.provider_etag.as_deref().is_some_and(|etag| {
        etag.trim().is_empty()
            || etag.len() > MAX_PROVIDER_ETAG_BYTES_V1
            || etag.chars().any(char::is_control)
    }) {
        return Err(ContactsValidationErrorV1::InvalidProviderEtag);
    }
    if value.source_revision == 0 {
        return Err(ContactsValidationErrorV1::InvalidSourceRevision);
    }
    if !nonzero(&value.entry_digest) {
        return Err(ContactsValidationErrorV1::InvalidDigest);
    }
    if !valid_timestamp(value.observed_at) {
        return Err(ContactsValidationErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn valid_timestamp(value: ContactTimestampV1) -> bool {
    value.unix_seconds > 0 && (0..1_000_000_000).contains(&value.nanos)
}

fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn update_part(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
}
