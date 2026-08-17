//! Mail-owned sanitized Person-source producer and durable lifecycle relay.

use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_mail_address_book_contract::{
    MailAddressBookEnvelopeBuildErrorV1, MailAddressBookEnvelopeContextV1,
    MailAddressBookResultEnvelopeContextV1, build_mail_person_source_account_ready_v1,
    build_mail_person_source_account_retired_v1, build_mail_person_source_observed_v1,
    build_mail_person_source_page_completed_v1, build_mail_person_source_removed_v1,
    build_mail_person_source_updated_v1, mail_person_source_claims_digest_v1,
    mail_person_source_tombstone_digest_v1,
    wire_person_source::{
        MailPersonSourceAccountReadyV1, MailPersonSourceAccountRetiredV1, MailPersonSourceClaimsV1,
        MailPersonSourceIdentityV1, MailPersonSourceObservedV1, MailPersonSourcePageCompletedV1,
        MailPersonSourceProvenanceV1, MailPersonSourceRemovedV1, MailPersonSourceUpdatedV1,
    },
};
use makosh_mail_address_book_persistence::{
    MailAddressBookPersistenceErrorV1, MailAddressBookPersistenceV1,
    MailPersonSourceAccountMappingV1, MailPersonSourceChangeKindV1,
    MailPersonSourceEnvelopeRecordV1,
};
use prost_types::Timestamp;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const MAIL_PERSON_SOURCE_REMOVALS_PER_PAGE_V1: usize = 500;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MailPersonSourcePublicAccountMappingV1 {
    pub integration_public_id: [u8; 16],
    pub account_public_id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonSourceProducerErrorV1 {
    InvalidInput,
    EntropyUnavailable,
    EnvelopeRejected,
    PersistenceUnavailable,
    EventUnavailable,
}

pub async fn ensure_public_account_ready_v1(
    persistence: &MailAddressBookPersistenceV1,
    logical_owner_id: &str,
    private_account_key: &str,
    context: &MailAddressBookEnvelopeContextV1,
    now_unix_millis: i64,
) -> Result<MailPersonSourceAccountMappingV1, MailPersonSourceProducerErrorV1> {
    let mapping = match persistence
        .load_person_source_account_mapping(logical_owner_id, private_account_key)
        .await
    {
        Ok(mapping) => mapping,
        Err(MailAddressBookPersistenceErrorV1::NotFound) => {
            ensure_public_account_mapping_v1(
                persistence,
                logical_owner_id,
                private_account_key,
                now_unix_millis,
            )
            .await?
        }
        Err(_) => return Err(MailPersonSourceProducerErrorV1::PersistenceUnavailable),
    };
    if persistence
        .load_person_source_account_lifecycle_record(logical_owner_id, &mapping, false)
        .await
        .map_err(|_| MailPersonSourceProducerErrorV1::PersistenceUnavailable)?
        .is_some()
    {
        return Ok(mapping);
    }
    let account_event_id = lifecycle_id(
        b"makosh.mail.person-source.account-ready.v1",
        logical_owner_id,
        mapping.account_public_id,
        mapping.mapping_revision,
    );
    let causation_message_id = lifecycle_id(
        b"makosh.mail.person-source.account-ready-causation.v1",
        logical_owner_id,
        mapping.account_public_id,
        mapping.mapping_revision,
    );
    let record = build_mail_person_source_account_ready_v1(
        causation_message_id,
        MailPersonSourceAccountReadyV1 {
            account_event_id: account_event_id.to_vec(),
            logical_owner_id: logical_owner_id.to_owned(),
            integration_public_id: mapping.integration_public_id.to_vec(),
            account_public_id: mapping.account_public_id.to_vec(),
            mapping_revision: mapping.mapping_revision,
            observed_at: Some(timestamp(now_unix_millis)?),
        },
        context,
    )
    .map_err(envelope_error)?;
    persistence
        .record_person_source_account_lifecycle_once(
            logical_owner_id,
            mapping.clone(),
            false,
            MailPersonSourceEnvelopeRecordV1::from_outbox(&record),
            now_unix_millis,
        )
        .await
        .map_err(|_| MailPersonSourceProducerErrorV1::PersistenceUnavailable)?;
    Ok(mapping)
}

pub async fn record_public_account_retired_v1(
    persistence: &MailAddressBookPersistenceV1,
    logical_owner_id: &str,
    private_account_key: &str,
    context: &MailAddressBookEnvelopeContextV1,
    now_unix_millis: i64,
) -> Result<MailPersonSourceAccountMappingV1, MailPersonSourceProducerErrorV1> {
    let mapping = persistence
        .load_person_source_account_mapping(logical_owner_id, private_account_key)
        .await
        .map_err(|_| MailPersonSourceProducerErrorV1::PersistenceUnavailable)?;
    if persistence
        .load_person_source_account_lifecycle_record(logical_owner_id, &mapping, true)
        .await
        .map_err(|_| MailPersonSourceProducerErrorV1::PersistenceUnavailable)?
        .is_some()
    {
        return Ok(mapping);
    }
    let account_event_id = lifecycle_id(
        b"makosh.mail.person-source.account-retired.v1",
        logical_owner_id,
        mapping.account_public_id,
        mapping.mapping_revision,
    );
    let causation_message_id = lifecycle_id(
        b"makosh.mail.person-source.account-retired-causation.v1",
        logical_owner_id,
        mapping.account_public_id,
        mapping.mapping_revision,
    );
    let record = build_mail_person_source_account_retired_v1(
        causation_message_id,
        MailPersonSourceAccountRetiredV1 {
            account_event_id: account_event_id.to_vec(),
            logical_owner_id: logical_owner_id.to_owned(),
            integration_public_id: mapping.integration_public_id.to_vec(),
            account_public_id: mapping.account_public_id.to_vec(),
            mapping_revision: mapping.mapping_revision,
            retired_at: Some(timestamp(now_unix_millis)?),
        },
        context,
    )
    .map_err(envelope_error)?;
    persistence
        .record_person_source_account_lifecycle_once(
            logical_owner_id,
            mapping.clone(),
            true,
            MailPersonSourceEnvelopeRecordV1::from_outbox(&record),
            now_unix_millis,
        )
        .await
        .map_err(|_| MailPersonSourceProducerErrorV1::PersistenceUnavailable)?;
    Ok(mapping)
}

pub async fn relay_public_account_lifecycle_once_v1(
    persistence: &MailAddressBookPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    now_unix_millis: i64,
) -> Result<bool, MailPersonSourceProducerErrorV1> {
    let Some(pending) = persistence
        .load_pending_person_source_lifecycle_outbox(logical_owner_id)
        .await
        .map_err(|_| MailPersonSourceProducerErrorV1::PersistenceUnavailable)?
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, &pending.record.envelope_bytes)
        .await
        .map_err(|_| MailPersonSourceProducerErrorV1::EventUnavailable)?;
    persistence
        .mark_person_source_lifecycle_outbox_published(
            logical_owner_id,
            pending.record.message_id,
            pending.record.envelope_sha256,
            now_unix_millis,
        )
        .await
        .map_err(|_| MailPersonSourceProducerErrorV1::PersistenceUnavailable)?;
    Ok(true)
}

fn lifecycle_id(
    domain: &[u8],
    logical_owner_id: &str,
    account_public_id: [u8; 16],
    mapping_revision: u64,
) -> [u8; 16] {
    let revision = mapping_revision.to_be_bytes();
    let mut digest = Sha256::new();
    for value in [
        domain,
        logical_owner_id.as_bytes(),
        account_public_id.as_slice(),
        revision.as_slice(),
    ] {
        digest.update((value.len() as u64).to_be_bytes());
        digest.update(value);
    }
    digest.finalize()[..16].try_into().expect("SHA-256 prefix")
}

fn timestamp(unix_millis: i64) -> Result<Timestamp, MailPersonSourceProducerErrorV1> {
    if unix_millis <= 0 {
        return Err(MailPersonSourceProducerErrorV1::InvalidInput);
    }
    Ok(Timestamp {
        seconds: unix_millis / 1_000,
        nanos: ((unix_millis % 1_000) * 1_000_000) as i32,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourceSyntheticRemovalPageV1 {
    pub page_id: [u8; 16],
    pub page_sequence: u64,
    pub source_ids: Vec<[u8; 16]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonSourcePublicChangeV1 {
    Observed,
    Unchanged,
    Updated,
}

#[must_use]
pub const fn public_change_from_storage_v1(
    value: MailPersonSourceChangeKindV1,
) -> MailPersonSourcePublicChangeV1 {
    match value {
        MailPersonSourceChangeKindV1::Observed => MailPersonSourcePublicChangeV1::Observed,
        MailPersonSourceChangeKindV1::Unchanged => MailPersonSourcePublicChangeV1::Unchanged,
        MailPersonSourceChangeKindV1::Updated => MailPersonSourcePublicChangeV1::Updated,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourcePublicChangeInputV1 {
    pub command_message_id: [u8; 16],
    pub observation_id: [u8; 16],
    pub run_id: [u8; 16],
    pub logical_owner_id: String,
    pub page_sequence: u64,
    pub source: MailPersonSourceIdentityV1,
    pub claims: MailPersonSourceClaimsV1,
    pub source_revision: u64,
    pub observed_at: Timestamp,
    pub context: MailAddressBookEnvelopeContextV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MailPersonSourceSyntheticRemovalV1 {
    pub integration_public_id: [u8; 16],
    pub account_public_id: [u8; 16],
    pub provider_source_contact_public_id: [u8; 16],
    pub source_revision: u64,
}

pub struct MailPersonSourceSyntheticRemovalRecordsV1 {
    pub source_records: Vec<OutboxRecordV1>,
    pub page_completed: OutboxRecordV1,
}

impl MailPersonSourceSyntheticRemovalRecordsV1 {
    #[must_use]
    pub fn all_records(&self) -> Vec<&OutboxRecordV1> {
        self.source_records
            .iter()
            .chain(std::iter::once(&self.page_completed))
            .collect()
    }
}

pub fn issue_public_account_mapping_v1(
    logical_owner_id: &str,
    private_account_key: &str,
) -> Result<MailPersonSourcePublicAccountMappingV1, MailPersonSourceProducerErrorV1> {
    let mut seed = [0_u8; 32];
    getrandom::fill(&mut seed).map_err(|_| MailPersonSourceProducerErrorV1::EntropyUnavailable)?;
    derive_public_account_mapping_v1(logical_owner_id, private_account_key, seed)
}

pub async fn ensure_public_account_mapping_v1(
    persistence: &MailAddressBookPersistenceV1,
    logical_owner_id: &str,
    private_account_key: &str,
    created_at_unix_millis: i64,
) -> Result<MailPersonSourceAccountMappingV1, MailPersonSourceProducerErrorV1> {
    let issued = issue_public_account_mapping_v1(logical_owner_id, private_account_key)?;
    persistence
        .ensure_person_source_account_mapping(
            logical_owner_id,
            private_account_key,
            MailPersonSourceAccountMappingV1 {
                integration_public_id: issued.integration_public_id,
                account_public_id: issued.account_public_id,
                mapping_revision: 1,
            },
            created_at_unix_millis,
        )
        .await
        .map_err(|_| MailPersonSourceProducerErrorV1::PersistenceUnavailable)
}

pub fn derive_public_account_mapping_v1(
    logical_owner_id: &str,
    private_account_key: &str,
    random_seed: [u8; 32],
) -> Result<MailPersonSourcePublicAccountMappingV1, MailPersonSourceProducerErrorV1> {
    if logical_owner_id.is_empty()
        || logical_owner_id.len() > 128
        || !logical_owner_id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
        || private_account_key.is_empty()
        || private_account_key.len() > 256
        || random_seed.iter().all(|byte| *byte == 0)
    {
        return Err(MailPersonSourceProducerErrorV1::InvalidInput);
    }
    Ok(MailPersonSourcePublicAccountMappingV1 {
        integration_public_id: public_id(
            b"makosh.mail.person-source.integration-public-id.v1",
            logical_owner_id,
            private_account_key,
            random_seed,
        ),
        account_public_id: public_id(
            b"makosh.mail.person-source.account-public-id.v1",
            logical_owner_id,
            private_account_key,
            random_seed,
        ),
    })
}

pub fn issue_public_source_contact_id_v1(
    logical_owner_id: &str,
    account_public_id: [u8; 16],
    private_record_key: &[u8],
) -> Result<[u8; 16], MailPersonSourceProducerErrorV1> {
    let mut seed = [0_u8; 32];
    getrandom::fill(&mut seed).map_err(|_| MailPersonSourceProducerErrorV1::EntropyUnavailable)?;
    derive_public_source_contact_id_v1(
        logical_owner_id,
        account_public_id,
        private_record_key,
        seed,
    )
}

pub fn derive_public_source_contact_id_v1(
    logical_owner_id: &str,
    account_public_id: [u8; 16],
    private_record_key: &[u8],
    random_seed: [u8; 32],
) -> Result<[u8; 16], MailPersonSourceProducerErrorV1> {
    validate_owner(logical_owner_id)?;
    if account_public_id.iter().all(|byte| *byte == 0)
        || private_record_key.is_empty()
        || private_record_key.len() > 512
        || random_seed.iter().all(|byte| *byte == 0)
    {
        return Err(MailPersonSourceProducerErrorV1::InvalidInput);
    }
    let mut digest = Sha256::new();
    for part in [
        b"makosh.mail.person-source.contact-public-id.v1".as_slice(),
        logical_owner_id.as_bytes(),
        account_public_id.as_slice(),
        private_record_key,
        random_seed.as_slice(),
    ] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    Ok(digest.finalize()[..16].try_into().expect("SHA-256 prefix"))
}

pub fn plan_synthetic_removal_pages_v1(
    run_id: [u8; 16],
    first_page_sequence: u64,
    source_ids: &[[u8; 16]],
) -> Result<Vec<MailPersonSourceSyntheticRemovalPageV1>, MailPersonSourceProducerErrorV1> {
    let sorted = source_ids.iter().copied().collect::<BTreeSet<_>>();
    if run_id.iter().all(|byte| *byte == 0)
        || !(1..=4_096).contains(&first_page_sequence)
        || sorted.len() != source_ids.len()
        || sorted.iter().any(|id| id.iter().all(|byte| *byte == 0))
    {
        return Err(MailPersonSourceProducerErrorV1::InvalidInput);
    }
    sorted
        .into_iter()
        .collect::<Vec<_>>()
        .chunks(MAIL_PERSON_SOURCE_REMOVALS_PER_PAGE_V1)
        .enumerate()
        .map(|(index, chunk)| {
            let page_sequence = first_page_sequence
                .checked_add(
                    u64::try_from(index)
                        .map_err(|_| MailPersonSourceProducerErrorV1::InvalidInput)?,
                )
                .filter(|value| *value <= 4_096)
                .ok_or(MailPersonSourceProducerErrorV1::InvalidInput)?;
            Ok(MailPersonSourceSyntheticRemovalPageV1 {
                page_id: mail_person_source_fetch_id_v1(run_id, page_sequence),
                page_sequence,
                source_ids: chunk.to_vec(),
            })
        })
        .collect()
}

pub fn build_public_source_change_v1(
    input: &MailPersonSourcePublicChangeInputV1,
    change: MailPersonSourcePublicChangeV1,
) -> Result<Option<OutboxRecordV1>, MailPersonSourceProducerErrorV1> {
    if input.command_message_id != mail_person_source_fetch_id_v1(input.run_id, input.page_sequence)
    {
        return Err(MailPersonSourceProducerErrorV1::InvalidInput);
    }
    if change == MailPersonSourcePublicChangeV1::Unchanged {
        return Ok(None);
    }
    let source_digest = mail_person_source_claims_digest_v1(&input.source, &input.claims)
        .map_err(|_| MailPersonSourceProducerErrorV1::InvalidInput)?;
    let provenance = MailPersonSourceProvenanceV1 {
        source_revision: input.source_revision,
        source_digest: source_digest.to_vec(),
        observed_at: Some(input.observed_at),
    };
    let result = match change {
        MailPersonSourcePublicChangeV1::Observed => build_mail_person_source_observed_v1(
            input.command_message_id,
            MailPersonSourceObservedV1 {
                observation_id: input.observation_id.to_vec(),
                run_id: input.run_id.to_vec(),
                logical_owner_id: input.logical_owner_id.clone(),
                page_sequence: input.page_sequence,
                source: Some(input.source.clone()),
                claims: Some(input.claims.clone()),
                provenance: Some(provenance),
            },
            &input.context,
        ),
        MailPersonSourcePublicChangeV1::Updated => build_mail_person_source_updated_v1(
            input.command_message_id,
            MailPersonSourceUpdatedV1 {
                observation_id: input.observation_id.to_vec(),
                run_id: input.run_id.to_vec(),
                logical_owner_id: input.logical_owner_id.clone(),
                page_sequence: input.page_sequence,
                source: Some(input.source.clone()),
                claims: Some(input.claims.clone()),
                provenance: Some(provenance),
            },
            &input.context,
        ),
        MailPersonSourcePublicChangeV1::Unchanged => return Ok(None),
    };
    result.map(Some).map_err(envelope_error)
}

pub fn build_synthetic_removal_page_v1(
    logical_owner_id: &str,
    run_id: [u8; 16],
    page_sequence: u64,
    removals: &[MailPersonSourceSyntheticRemovalV1],
    has_more: bool,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<MailPersonSourceSyntheticRemovalRecordsV1, MailPersonSourceProducerErrorV1> {
    validate_owner(logical_owner_id)?;
    if run_id.iter().all(|byte| *byte == 0)
        || !(1..=4_096).contains(&page_sequence)
        || removals.is_empty()
        || removals.len() > MAIL_PERSON_SOURCE_REMOVALS_PER_PAGE_V1
        || context.recorded_at_unix_seconds <= 0
    {
        return Err(MailPersonSourceProducerErrorV1::InvalidInput);
    }
    let mut ordered = removals.to_vec();
    ordered.sort_by_key(|value| value.provider_source_contact_public_id);
    if ordered.windows(2).any(|pair| {
        pair[0].provider_source_contact_public_id == pair[1].provider_source_contact_public_id
    }) {
        return Err(MailPersonSourceProducerErrorV1::InvalidInput);
    }
    let command_id = mail_person_source_fetch_id_v1(run_id, page_sequence);
    let observed_at = Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    };
    let mut source_records = Vec::with_capacity(ordered.len());
    for removal in &ordered {
        if removal.source_revision == 0 {
            return Err(MailPersonSourceProducerErrorV1::InvalidInput);
        }
        let source = MailPersonSourceIdentityV1 {
            integration_public_id: removal.integration_public_id.to_vec(),
            account_public_id: removal.account_public_id.to_vec(),
            provider_source_contact_public_id: removal.provider_source_contact_public_id.to_vec(),
        };
        let source_digest = mail_person_source_tombstone_digest_v1(&source)
            .map_err(|_| MailPersonSourceProducerErrorV1::InvalidInput)?;
        let observation_id = deterministic_id16_v1(
            b"makosh.mail.person-source.synthetic-removal.v1",
            &command_id,
            &removal.provider_source_contact_public_id,
        );
        source_records.push(
            build_mail_person_source_removed_v1(
                command_id,
                MailPersonSourceRemovedV1 {
                    observation_id: observation_id.to_vec(),
                    run_id: run_id.to_vec(),
                    logical_owner_id: logical_owner_id.to_owned(),
                    page_sequence,
                    source: Some(source),
                    provenance: Some(MailPersonSourceProvenanceV1 {
                        source_revision: removal.source_revision,
                        source_digest: source_digest.to_vec(),
                        observed_at: Some(observed_at),
                    }),
                },
                context,
            )
            .map_err(envelope_error)?,
        );
    }
    let mut digest = Sha256::new();
    for record in &source_records {
        digest.update(record.envelope_sha256());
    }
    let page_digest: [u8; 32] = digest.finalize().into();
    let account_public_id = ordered[0].account_public_id;
    if ordered
        .iter()
        .any(|value| value.account_public_id != account_public_id)
    {
        return Err(MailPersonSourceProducerErrorV1::InvalidInput);
    }
    let page_completed = build_mail_person_source_page_completed_v1(
        command_id,
        MailPersonSourcePageCompletedV1 {
            command_id: command_id.to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: logical_owner_id.to_owned(),
            account_public_id: account_public_id.to_vec(),
            page_sequence,
            observed_sources: 0,
            updated_sources: 0,
            removed_sources: u32::try_from(source_records.len())
                .map_err(|_| MailPersonSourceProducerErrorV1::InvalidInput)?,
            has_more,
            page_digest: page_digest.to_vec(),
            completed_at: Some(observed_at),
        },
        &MailAddressBookResultEnvelopeContextV1 {
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            completed_at_unix_seconds: context.recorded_at_unix_seconds,
            completed_at_nanos: context.recorded_at_nanos,
            execution_attempt: 1,
        },
    )
    .map_err(envelope_error)?;
    Ok(MailPersonSourceSyntheticRemovalRecordsV1 {
        source_records,
        page_completed,
    })
}

#[must_use]
pub fn mail_person_source_fetch_id_v1(run_id: [u8; 16], page_sequence: u64) -> [u8; 16] {
    deterministic_id16_v1(
        b"mail-persons-sync.fetch-page.v1",
        &run_id,
        &page_sequence.to_be_bytes(),
    )
}

fn deterministic_id16_v1(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    for part in [label, first, second] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    digest.finalize()[..16].try_into().expect("SHA-256 prefix")
}

fn envelope_error(_: MailAddressBookEnvelopeBuildErrorV1) -> MailPersonSourceProducerErrorV1 {
    MailPersonSourceProducerErrorV1::EnvelopeRejected
}

fn validate_owner(value: &str) -> Result<(), MailPersonSourceProducerErrorV1> {
    if !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        Ok(())
    } else {
        Err(MailPersonSourceProducerErrorV1::InvalidInput)
    }
}

fn public_id(domain: &[u8], owner: &str, account: &str, seed: [u8; 32]) -> [u8; 16] {
    let mut digest = Sha256::new();
    for part in [
        domain,
        owner.as_bytes(),
        account.as_bytes(),
        seed.as_slice(),
    ] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    digest.finalize()[..16].try_into().expect("SHA-256 prefix")
}
