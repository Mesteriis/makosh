//! Mail-owned provider reads for the sanitized Person-source protocol.

use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePublishPermitV1, receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ActorKindV1, FenceKindV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_address_book_contract::{
    MAIL_PERSON_SOURCE_CAPABILITY_ID_V1, MAIL_PERSON_SOURCE_COMMAND_SOURCE_MODULE_ID_V1,
    MAIL_RUNTIME_MODULE_ID_V1, MailAddressBookEnvelopeContextV1,
    MailAddressBookResultEnvelopeContextV1, MailPersonSourceContractV1,
    build_mail_person_source_page_completed_v1, mail_person_source_claims_digest_v1,
    validate_fetch_mail_person_source_page_v1,
    wire_person_source::{
        FetchMailPersonSourcePageCommandV1, MailPersonSourceClaimsV1, MailPersonSourceIdentityV1,
        MailPersonSourcePageCompletedV1,
    },
};
use makosh_mail_address_book_persistence::{
    MailAddressBookPersistenceErrorV1, MailAddressBookPersistenceV1,
    MailPersonSourceAtomicFetchCommitV1, MailPersonSourceEnvelopeRecordV1,
    MailPersonSourceFetchOutputV1, MailPersonSourceObservationV1,
    MailPersonSourceRemovalPageCommitV1, MailPersonSourceSnapshotCommitV1,
    mail_person_source_semantic_order_key_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    address_book_fetch_worker::{ProviderEntryV1, fetch_provider_person_source_page_v1},
    managed::MailAdmittedRuntime,
    person_source_producer::{
        MailPersonSourcePublicChangeInputV1, MailPersonSourceSyntheticRemovalV1,
        build_public_source_change_v1, build_synthetic_removal_page_v1,
        issue_public_source_contact_id_v1, public_change_from_storage_v1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourceFetchAdmissionV1 {
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub command_envelope_bytes: Vec<u8>,
    pub run_id: [u8; 16],
    pub logical_owner_id: String,
    pub account_public_id: [u8; 16],
    pub page_sequence: u64,
    pub page_size: u32,
    pub deadline_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonSourceFetchWorkerErrorV1 {
    InvalidEnvelope,
    WrongContract,
    WrongSource,
    WrongAudience,
    OwnerMismatch,
    InvalidPayload,
    Expired,
    Persistence,
    ProviderUnavailable,
    EventUnavailable,
}

pub async fn consume_and_process_person_source_fetch_v1(
    runtime: &mut MailAdmittedRuntime,
    now_unix_seconds: i64,
) -> Result<bool, MailPersonSourceFetchWorkerErrorV1> {
    let delivery = match receive_runtime_pull_delivery(
        &runtime.event_connection,
        &runtime.person_source_fetch_subscribe_permit,
    )
    .await
    {
        Ok(delivery) => delivery,
        Err(_) => return Ok(false),
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailPersonSourceFetchWorkerErrorV1::InvalidEnvelope)?;
    let admission =
        decode_person_source_fetch_identity_v1(&record, &runtime.logical_human_owner_id)?;
    let command = MailPersonSourceEnvelopeRecordV1 {
        message_id: admission.command_message_id,
        envelope_sha256: admission.command_envelope_sha256,
        envelope_bytes: admission.command_envelope_bytes.clone(),
    };
    if runtime
        .address_book_persistence
        .accept_person_source_synthetic_fetch_continuation_once(
            &admission.logical_owner_id,
            admission.account_public_id,
            admission.run_id,
            admission.page_sequence,
            &command,
            now_unix_seconds
                .checked_mul(1_000)
                .ok_or(MailPersonSourceFetchWorkerErrorV1::InvalidPayload)?,
        )
        .await
        .map_err(persistence_error)?
    {
        delivery
            .acknowledge()
            .await
            .map_err(|_| MailPersonSourceFetchWorkerErrorV1::EventUnavailable)?;
        return Ok(true);
    }
    if runtime
        .address_book_persistence
        .load_person_source_fetch_replay(
            &admission.logical_owner_id,
            admission.account_public_id,
            admission.run_id,
            admission.page_sequence,
            &command,
        )
        .await
        .map_err(persistence_error)?
        .is_some()
    {
        delivery
            .acknowledge()
            .await
            .map_err(|_| MailPersonSourceFetchWorkerErrorV1::EventUnavailable)?;
        return Ok(true);
    }
    if now_unix_seconds > admission.deadline_unix_seconds {
        return Err(MailPersonSourceFetchWorkerErrorV1::Expired);
    }
    let binding = runtime
        .address_book_persistence
        .load_person_source_account_binding_by_public_id(
            &admission.logical_owner_id,
            admission.account_public_id,
        )
        .await
        .map_err(persistence_error)?;
    let fetch_state = runtime
        .address_book_persistence
        .load_person_source_fetch_state(
            &admission.logical_owner_id,
            admission.account_public_id,
            admission.run_id,
            admission.page_sequence,
        )
        .await
        .map_err(persistence_error)?;
    let expected_cursor = fetch_state.and_then(|state| state.provider_cursor);
    let page = fetch_provider_person_source_page_v1(
        runtime,
        &binding.private_account_key,
        expected_cursor.as_deref(),
        admission.page_size,
    )
    .await
    .map_err(|_| MailPersonSourceFetchWorkerErrorV1::ProviderUnavailable)?;
    let processed_at_unix_millis = now_unix_seconds
        .checked_mul(1_000)
        .ok_or(MailPersonSourceFetchWorkerErrorV1::InvalidPayload)?;
    let claims = page
        .entries
        .iter()
        .map(public_claims)
        .collect::<Result<Vec<_>, _>>()?;
    let mut existing_source_ids = Vec::with_capacity(page.entries.len());
    for entry in &page.entries {
        existing_source_ids.push(
            runtime
                .address_book_persistence
                .load_person_source_contact_public_id(
                    &admission.logical_owner_id,
                    admission.account_public_id,
                    entry.provider_entry_id.as_bytes(),
                )
                .await
                .map_err(persistence_error)?,
        );
    }
    let planned_removal_count = if page.next_cursor.is_none() {
        let mut seen = runtime
            .address_book_persistence
            .load_person_source_run_seen_ids(
                &admission.logical_owner_id,
                admission.account_public_id,
                admission.run_id,
            )
            .await
            .map_err(persistence_error)?;
        seen.extend(existing_source_ids.iter().flatten().copied());
        seen.sort_unstable();
        seen.dedup();
        runtime
            .address_book_persistence
            .preview_person_source_removals(
                &admission.logical_owner_id,
                admission.account_public_id,
                &seen,
            )
            .await
            .map_err(persistence_error)?
            .len()
    } else {
        0
    };
    let public_has_more = page_has_more_v1(page.next_cursor.is_some(), planned_removal_count);
    let commit = MailPersonSourceAtomicFetchCommitV1 {
        logical_owner_id: admission.logical_owner_id.clone(),
        account_public_id: admission.account_public_id,
        run_id: admission.run_id,
        page_sequence: admission.page_sequence,
        expected_provider_cursor: expected_cursor,
        next_provider_cursor: page.next_cursor.clone(),
        public_has_more,
        has_more: page.next_cursor.is_some(),
        command: MailPersonSourceEnvelopeRecordV1 {
            message_id: admission.command_message_id,
            envelope_sha256: admission.command_envelope_sha256,
            envelope_bytes: admission.command_envelope_bytes.clone(),
        },
        processed_at_unix_millis,
    };
    let context = MailAddressBookEnvelopeContextV1 {
        module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.clone(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: now_unix_seconds,
        recorded_at_nanos: 0,
    };
    let entries = &page.entries;
    let outcome = runtime
        .address_book_persistence
        .commit_person_source_fetch_atomically_once(
            &commit,
            || {
                prepare_observations(
                    &admission,
                    binding.mapping.integration_public_id,
                    entries,
                    &claims,
                    &existing_source_ids,
                    processed_at_unix_millis,
                )
            },
            |changes| {
                build_fetch_outputs(
                    &admission,
                    binding.mapping.integration_public_id,
                    entries,
                    &claims,
                    changes,
                    public_has_more,
                    &context,
                )
            },
        )
        .await
        .map_err(persistence_error)?;
    if !commit.has_more && !(outcome.replayed && outcome.terminal_snapshot_succeeded) {
        let completed_at_unix_millis = outcome.processed_at_unix_millis;
        let terminal_context = MailAddressBookEnvelopeContextV1 {
            module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
            runtime_instance_id: runtime.runtime_instance_id.clone(),
            runtime_generation: runtime.runtime_generation,
            recorded_at_unix_seconds: completed_at_unix_millis.div_euclid(1_000),
            recorded_at_nanos: i32::try_from(
                completed_at_unix_millis.rem_euclid(1_000) * 1_000_000,
            )
            .map_err(|_| MailPersonSourceFetchWorkerErrorV1::InvalidPayload)?,
        };
        terminalize_snapshot(
            runtime,
            &commit,
            &outcome.outputs,
            &terminal_context,
            completed_at_unix_millis,
        )
        .await?;
    }
    delivery
        .acknowledge()
        .await
        .map_err(|_| MailPersonSourceFetchWorkerErrorV1::EventUnavailable)?;
    Ok(true)
}

const fn page_has_more_v1(provider_has_more: bool, planned_removal_count: usize) -> bool {
    provider_has_more || planned_removal_count != 0
}

pub async fn relay_person_source_fetch_outbox_once_v1(
    persistence: &MailAddressBookPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    logical_owner_id: &str,
    published_at_unix_millis: i64,
) -> Result<bool, MailPersonSourceFetchWorkerErrorV1> {
    let Some(pending) = persistence
        .load_pending_person_source_fetch_outbox(logical_owner_id)
        .await
        .map_err(persistence_error)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    let record = OutboxRecordV1::accept(pending.record.envelope_bytes.clone())
        .map_err(|_| MailPersonSourceFetchWorkerErrorV1::InvalidEnvelope)?;
    connection
        .publish_exact(permit, record.exact_bytes())
        .await
        .map_err(|_| MailPersonSourceFetchWorkerErrorV1::EventUnavailable)?;
    persistence
        .mark_person_source_fetch_outbox_published(
            logical_owner_id,
            pending.record.message_id,
            pending.record.envelope_sha256,
            published_at_unix_millis,
        )
        .await
        .map_err(persistence_error)?;
    Ok(true)
}

fn prepare_observations(
    admission: &MailPersonSourceFetchAdmissionV1,
    integration_public_id: [u8; 16],
    entries: &[ProviderEntryV1],
    claims: &[MailPersonSourceClaimsV1],
    existing_source_ids: &[Option<[u8; 16]>],
    observed_at_unix_millis: i64,
) -> Result<Vec<MailPersonSourceObservationV1>, MailAddressBookPersistenceErrorV1> {
    entries
        .iter()
        .zip(claims)
        .zip(existing_source_ids)
        .map(|((entry, claims), existing_source_id)| {
            let provider_record_key = entry.provider_entry_id.as_bytes().to_vec();
            let proposed_source_public_id = match existing_source_id {
                Some(value) => *value,
                None => issue_public_source_contact_id_v1(
                    &admission.logical_owner_id,
                    admission.account_public_id,
                    &provider_record_key,
                )
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
            };
            let source = MailPersonSourceIdentityV1 {
                integration_public_id: integration_public_id.to_vec(),
                account_public_id: admission.account_public_id.to_vec(),
                provider_source_contact_public_id: proposed_source_public_id.to_vec(),
            };
            let claims_digest = mail_person_source_claims_digest_v1(&source, claims)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
            Ok(MailPersonSourceObservationV1 {
                logical_owner_id: admission.logical_owner_id.clone(),
                account_public_id: admission.account_public_id,
                provider_record_key,
                provider_record_etag: entry
                    .provider_etag
                    .as_ref()
                    .map(|value| value.as_bytes().to_vec()),
                proposed_source_public_id,
                claims_digest,
                observed_at_unix_millis,
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn build_fetch_outputs(
    admission: &MailPersonSourceFetchAdmissionV1,
    integration_public_id: [u8; 16],
    entries: &[ProviderEntryV1],
    claims: &[MailPersonSourceClaimsV1],
    changes: &[makosh_mail_address_book_persistence::MailPersonSourceObservationOutcomeV1],
    has_more: bool,
    context: &MailAddressBookEnvelopeContextV1,
) -> Result<Vec<MailPersonSourceFetchOutputV1>, MailAddressBookPersistenceErrorV1> {
    let mut outputs = Vec::new();
    for ((entry, claims), change) in entries.iter().zip(claims).zip(changes) {
        let change_kind = public_change_from_storage_v1(change.change_kind);
        let source = MailPersonSourceIdentityV1 {
            integration_public_id: integration_public_id.to_vec(),
            account_public_id: admission.account_public_id.to_vec(),
            provider_source_contact_public_id: change.provider_source_contact_public_id.to_vec(),
        };
        let observation_id = deterministic_id16(
            b"makosh.mail.person-source.observation.v1",
            &admission.command_message_id,
            entry.provider_entry_id.as_bytes(),
        );
        if let Some(record) = build_public_source_change_v1(
            &MailPersonSourcePublicChangeInputV1 {
                command_message_id: admission.command_message_id,
                observation_id,
                run_id: admission.run_id,
                logical_owner_id: admission.logical_owner_id.clone(),
                page_sequence: admission.page_sequence,
                source,
                claims: claims.clone(),
                source_revision: change.source_revision,
                observed_at: Timestamp {
                    seconds: context.recorded_at_unix_seconds,
                    nanos: context.recorded_at_nanos,
                },
                context: context.clone(),
            },
            change_kind,
        )
        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?
        {
            outputs.push(MailPersonSourceFetchOutputV1 {
                semantic_order_key: mail_person_source_semantic_order_key_v1(
                    admission.page_sequence,
                    u16::try_from(outputs.len() + 1)
                        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
                )?,
                record: MailPersonSourceEnvelopeRecordV1::from_outbox(&record),
            });
        }
    }
    let mut page_digest = Sha256::new();
    for output in &outputs {
        page_digest.update(output.record.envelope_sha256);
    }
    let page_digest: [u8; 32] = page_digest.finalize().into();
    let observed_sources = changes
        .iter()
        .filter(|change| {
            matches!(
                change.change_kind,
                makosh_mail_address_book_persistence::MailPersonSourceChangeKindV1::Observed
            )
        })
        .count();
    let updated_sources = changes
        .iter()
        .filter(|change| {
            matches!(
                change.change_kind,
                makosh_mail_address_book_persistence::MailPersonSourceChangeKindV1::Updated
            )
        })
        .count();
    let completed = build_mail_person_source_page_completed_v1(
        admission.command_message_id,
        MailPersonSourcePageCompletedV1 {
            command_id: admission.command_message_id.to_vec(),
            run_id: admission.run_id.to_vec(),
            logical_owner_id: admission.logical_owner_id.clone(),
            account_public_id: admission.account_public_id.to_vec(),
            page_sequence: admission.page_sequence,
            observed_sources: u32::try_from(observed_sources)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
            updated_sources: u32::try_from(updated_sources)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
            removed_sources: 0,
            has_more,
            page_digest: page_digest.to_vec(),
            completed_at: Some(Timestamp {
                seconds: context.recorded_at_unix_seconds,
                nanos: context.recorded_at_nanos,
            }),
        },
        &MailAddressBookResultEnvelopeContextV1 {
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            completed_at_unix_seconds: context.recorded_at_unix_seconds,
            completed_at_nanos: context.recorded_at_nanos,
            execution_attempt: 1,
        },
    )
    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
    outputs.push(MailPersonSourceFetchOutputV1 {
        semantic_order_key: mail_person_source_semantic_order_key_v1(
            admission.page_sequence,
            u16::try_from(outputs.len() + 1)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
        )?,
        record: MailPersonSourceEnvelopeRecordV1::from_outbox(&completed),
    });
    Ok(outputs)
}

async fn terminalize_snapshot(
    runtime: &MailAdmittedRuntime,
    commit: &MailPersonSourceAtomicFetchCommitV1,
    outputs: &[MailPersonSourceFetchOutputV1],
    context: &MailAddressBookEnvelopeContextV1,
    completed_at_unix_millis: i64,
) -> Result<(), MailPersonSourceFetchWorkerErrorV1> {
    let seen = runtime
        .address_book_persistence
        .load_person_source_run_seen_ids(
            &commit.logical_owner_id,
            commit.account_public_id,
            commit.run_id,
        )
        .await
        .map_err(persistence_error)?;
    let removals = runtime
        .address_book_persistence
        .preview_person_source_removals(&commit.logical_owner_id, commit.account_public_id, &seen)
        .await
        .map_err(persistence_error)?;
    let mut removal_pages = Vec::new();
    for (index, chunk) in removals.chunks(500).enumerate() {
        let page_sequence = commit
            .page_sequence
            .checked_add(
                u64::try_from(index + 1)
                    .map_err(|_| MailPersonSourceFetchWorkerErrorV1::InvalidPayload)?,
            )
            .ok_or(MailPersonSourceFetchWorkerErrorV1::InvalidPayload)?;
        let records = build_synthetic_removal_page_v1(
            &commit.logical_owner_id,
            commit.run_id,
            page_sequence,
            &chunk
                .iter()
                .map(|value| MailPersonSourceSyntheticRemovalV1 {
                    integration_public_id: value.integration_public_id,
                    account_public_id: value.account_public_id,
                    provider_source_contact_public_id: value.provider_source_contact_public_id,
                    source_revision: value.source_revision,
                })
                .collect::<Vec<_>>(),
            index + 1 < removals.chunks(500).len(),
            context,
        )
        .map_err(|_| MailPersonSourceFetchWorkerErrorV1::InvalidPayload)?;
        let all = records
            .source_records
            .iter()
            .chain(std::iter::once(&records.page_completed))
            .enumerate()
            .map(|(ordinal, record)| {
                Ok(MailPersonSourceFetchOutputV1 {
                    semantic_order_key: mail_person_source_semantic_order_key_v1(
                        page_sequence,
                        u16::try_from(ordinal + 1)
                            .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
                    )?,
                    record: MailPersonSourceEnvelopeRecordV1::from_outbox(record),
                })
            })
            .collect::<Result<Vec<_>, MailAddressBookPersistenceErrorV1>>()
            .map_err(persistence_error)?;
        removal_pages.push(MailPersonSourceRemovalPageCommitV1 {
            page_sequence,
            source_ids: chunk
                .iter()
                .map(|value| value.provider_source_contact_public_id)
                .collect(),
            outputs: all,
        });
    }
    let terminal_command = outputs
        .last()
        .ok_or(MailPersonSourceFetchWorkerErrorV1::InvalidPayload)?
        .record
        .clone();
    runtime
        .address_book_persistence
        .commit_person_source_snapshot_once(&MailPersonSourceSnapshotCommitV1 {
            logical_owner_id: commit.logical_owner_id.clone(),
            account_public_id: commit.account_public_id,
            run_id: commit.run_id,
            seen_public_source_ids: seen,
            expected_removals: removals,
            removal_pages,
            terminal_command,
            completed_at_unix_millis,
        })
        .await
        .map_err(persistence_error)?;
    Ok(())
}

fn public_claims(
    entry: &ProviderEntryV1,
) -> Result<MailPersonSourceClaimsV1, MailPersonSourceFetchWorkerErrorV1> {
    let display_name = entry
        .display_name
        .trim()
        .chars()
        .take(240)
        .collect::<String>();
    let display_name = (!display_name.is_empty() && !display_name.chars().any(char::is_control))
        .then_some(display_name);
    let mut normalized_emails = entry
        .email_addresses
        .iter()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| value.len() <= 320)
        .collect::<Vec<_>>();
    normalized_emails.sort();
    normalized_emails.dedup();
    let mut normalized_phones = entry
        .phone_numbers
        .iter()
        .map(|value| {
            let digits = value
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>();
            format!("+{digits}")
        })
        .filter(|value| (8..=16).contains(&value.len()) && !value.starts_with("+0"))
        .collect::<Vec<_>>();
    normalized_phones.sort();
    normalized_phones.dedup();
    let claims = MailPersonSourceClaimsV1 {
        display_name,
        normalized_emails,
        normalized_phones,
    };
    let dummy_source = MailPersonSourceIdentityV1 {
        integration_public_id: vec![1; 16],
        account_public_id: vec![2; 16],
        provider_source_contact_public_id: vec![3; 16],
    };
    mail_person_source_claims_digest_v1(&dummy_source, &claims)
        .map_err(|_| MailPersonSourceFetchWorkerErrorV1::InvalidPayload)?;
    Ok(claims)
}

fn deterministic_id16(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    for part in [label, first, second] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    digest.finalize()[..16].try_into().expect("SHA-256 prefix")
}

fn persistence_error(_: MailAddressBookPersistenceErrorV1) -> MailPersonSourceFetchWorkerErrorV1 {
    MailPersonSourceFetchWorkerErrorV1::Persistence
}

pub fn decode_person_source_fetch_v1(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
    now_unix_seconds: i64,
) -> Result<MailPersonSourceFetchAdmissionV1, MailPersonSourceFetchWorkerErrorV1> {
    let admission = decode_person_source_fetch_identity_v1(record, expected_logical_owner_id)?;
    if now_unix_seconds > admission.deadline_unix_seconds {
        return Err(MailPersonSourceFetchWorkerErrorV1::Expired);
    }
    Ok(admission)
}

fn decode_person_source_fetch_identity_v1(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<MailPersonSourceFetchAdmissionV1, MailPersonSourceFetchWorkerErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailPersonSourceFetchWorkerErrorV1::InvalidEnvelope)?;
    let expected = MailPersonSourceContractV1::FetchPageCommand.reference();
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(MailPersonSourceFetchWorkerErrorV1::WrongContract)?;
    if contract.owner != expected.owner
        || contract.name != expected.name
        || contract.major != expected.major
        || contract.revision != expected.revision
        || contract.schema_sha256 != expected.schema_sha256
    {
        return Err(MailPersonSourceFetchWorkerErrorV1::WrongContract);
    }
    let Some(Semantics::Command(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailPersonSourceFetchWorkerErrorV1::WrongContract);
    };
    if metadata.target_capability != MAIL_PERSON_SOURCE_CAPABILITY_ID_V1 {
        return Err(MailPersonSourceFetchWorkerErrorV1::WrongAudience);
    }
    let source = envelope
        .source
        .as_ref()
        .ok_or(MailPersonSourceFetchWorkerErrorV1::WrongSource)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(MailPersonSourceFetchWorkerErrorV1::WrongSource)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(MailPersonSourceFetchWorkerErrorV1::WrongSource)?;
    if source.module_id != MAIL_PERSON_SOURCE_COMMAND_SOURCE_MODULE_ID_V1
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id != MAIL_PERSON_SOURCE_COMMAND_SOURCE_MODULE_ID_V1.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != MAIL_PERSON_SOURCE_COMMAND_SOURCE_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
    {
        return Err(MailPersonSourceFetchWorkerErrorV1::WrongSource);
    }
    let payload = FetchMailPersonSourcePageCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailPersonSourceFetchWorkerErrorV1::InvalidPayload)?;
    validate_fetch_mail_person_source_page_v1(&payload)
        .map_err(|_| MailPersonSourceFetchWorkerErrorV1::InvalidPayload)?;
    if payload.encode_to_vec() != envelope.payload {
        return Err(MailPersonSourceFetchWorkerErrorV1::InvalidPayload);
    }
    let command_id = id16(&payload.command_id)?;
    let run_id = id16(&payload.run_id)?;
    let account_public_id = id16(&payload.account_public_id)?;
    let recorded_at = envelope
        .recorded_at
        .as_ref()
        .ok_or(MailPersonSourceFetchWorkerErrorV1::InvalidEnvelope)?;
    let deadline = metadata
        .deadline
        .as_ref()
        .ok_or(MailPersonSourceFetchWorkerErrorV1::InvalidPayload)?;
    let payload_fingerprint: [u8; 32] = Sha256::digest(&envelope.payload).into();
    if payload.logical_owner_id != expected_logical_owner_id {
        return Err(MailPersonSourceFetchWorkerErrorV1::OwnerMismatch);
    }
    if envelope.message_id.as_slice() != command_id
        || metadata.command_id.as_slice() != command_id
        || metadata.idempotency_key.as_slice() != payload_fingerprint
        || metadata.logical_attempt == 0
        || envelope.partition_key.as_slice() != run_id
        || envelope.correlation_id.as_slice() != run_id
        || !envelope.causation_message_id.is_empty()
        || recorded_at.seconds <= 0
        || !(0..1_000_000_000).contains(&recorded_at.nanos)
        || deadline.nanos != 0
        || deadline.seconds <= recorded_at.seconds
    {
        return Err(MailPersonSourceFetchWorkerErrorV1::InvalidPayload);
    }
    Ok(MailPersonSourceFetchAdmissionV1 {
        command_message_id: command_id,
        command_envelope_sha256: *record.envelope_sha256(),
        command_envelope_bytes: record.exact_bytes().to_vec(),
        run_id,
        logical_owner_id: payload.logical_owner_id,
        account_public_id,
        page_sequence: payload.page_sequence,
        page_size: payload.page_size,
        deadline_unix_seconds: deadline.seconds,
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailPersonSourceFetchWorkerErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| MailPersonSourceFetchWorkerErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(MailPersonSourceFetchWorkerErrorV1::InvalidPayload)
}

#[cfg(test)]
mod tests {
    use makosh_mail_address_book_contract::{
        MailAddressBookEnvelopeContextV1, build_fetch_mail_person_source_page_command_v1,
    };

    use super::*;

    fn command() -> OutboxRecordV1 {
        build_fetch_mail_person_source_page_command_v1(
            FetchMailPersonSourcePageCommandV1 {
                command_id: vec![1; 16],
                run_id: vec![2; 16],
                logical_owner_id: "owner-1".to_owned(),
                account_public_id: vec![3; 16],
                page_sequence: 1,
                page_size: 500,
            },
            200,
            &MailAddressBookEnvelopeContextV1 {
                module_id: MAIL_PERSON_SOURCE_COMMAND_SOURCE_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "workflow-runtime-1".to_owned(),
                runtime_generation: 7,
                recorded_at_unix_seconds: 100,
                recorded_at_nanos: 0,
            },
        )
        .expect("command")
    }

    #[test]
    fn exact_sanitized_fetch_is_accepted_and_expiry_is_bounded() {
        let accepted =
            decode_person_source_fetch_v1(&command(), "owner-1", 150).expect("exact command");
        assert_eq!(accepted.command_message_id, [1; 16]);
        assert_eq!(accepted.account_public_id, [3; 16]);
        assert_eq!(
            decode_person_source_fetch_v1(&command(), "owner-1", 201),
            Err(MailPersonSourceFetchWorkerErrorV1::Expired)
        );
    }

    #[test]
    fn owner_and_source_authority_are_exact() {
        assert_eq!(
            decode_person_source_fetch_v1(&command(), "owner-2", 150),
            Err(MailPersonSourceFetchWorkerErrorV1::OwnerMismatch)
        );
        let envelope = decode_envelope_v1(command().exact_bytes()).expect("envelope");
        let mut changed = envelope;
        changed.source.as_mut().expect("source").module_id = "mail-runtime".to_owned();
        let changed = OutboxRecordV1::accept(changed.encode_to_vec()).expect("changed record");
        assert_eq!(
            decode_person_source_fetch_v1(&changed, "owner-1", 150),
            Err(MailPersonSourceFetchWorkerErrorV1::WrongSource)
        );
    }

    #[test]
    fn terminal_provider_page_continues_when_synthetic_removals_follow() {
        assert!(!page_has_more_v1(false, 0));
        assert!(page_has_more_v1(false, 1));
        assert!(page_has_more_v1(true, 0));
    }
}
