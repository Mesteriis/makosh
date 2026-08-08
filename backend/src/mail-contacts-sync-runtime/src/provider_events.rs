use makosh_contacts_command_api::{
    ContactsCommandEnvelopeContextV1, build_upsert_contact_command_outbox_record_v1,
    wire::{
        MailAddressBookProviderKindV1 as ContactsProviderKindV1,
        UpsertContactFromMailAddressBookEntryCommandV1,
    },
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ContractRefV1, ResultOutcomeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_address_book_contract::{
    MailAddressBookContractV1,
    wire::{
        MailAddressBookEntryObservedV1, MailAddressBookPageCompletedV1,
        MailAddressBookPageRejectedV1, MailAddressBookProviderKindV1, MailAddressBookRejectCodeV1,
    },
};
use makosh_mail_contacts_sync_core::{MailContactsSyncRejectCodeV1, MailContactsSyncTransitionV1};
use makosh_mail_contacts_sync_persistence::{
    MailContactsSyncEntryInputV1, MailContactsSyncPageResultInputV1,
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1,
    MailContactsSyncTransitionInputV1, OutboxEnvelopeV1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;
use sha2::{Digest, Sha256};

use crate::MAIL_CONTACTS_SYNC_COMMAND_DEADLINE_SECONDS_V1;

const MAIL_RUNTIME_MODULE_ID_V1: &str = "makosh-mail-runtime";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncProviderRuntimeContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncProviderEventErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(MailContactsSyncPersistenceErrorV1),
    EventUnavailable,
}

pub async fn consume_mail_address_book_entry_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &MailContactsSyncProviderRuntimeContextV1,
) -> Result<bool, MailContactsSyncProviderEventErrorV1> {
    validate_runtime(runtime)?;
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = accepted_record(delivery.exact_bytes())?;
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailContactsSyncProviderEventErrorV1::InvalidEnvelope)?;
    validate_source_and_contract(
        &envelope.contract,
        MailAddressBookContractV1::EntryObserved,
        envelope
            .source
            .as_ref()
            .map(|source| source.module_id.as_str()),
        envelope
            .source
            .as_ref()
            .map_or(0, |source| source.runtime_generation),
    )?;
    let Some(Semantics::Observation(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailContactsSyncProviderEventErrorV1::InvalidEnvelope);
    };
    if metadata.observation_id.as_slice() != record.message_id()
        || metadata.source_cursor_sha256.len() != 32
    {
        return Err(MailContactsSyncProviderEventErrorV1::InvalidEnvelope);
    }
    let payload = MailAddressBookEntryObservedV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailContactsSyncProviderEventErrorV1::InvalidPayload)?;
    let run_id = id16(&payload.run_id)?;
    let observation_id = id16(&payload.observation_id)?;
    let entry_digest = id32(&payload.entry_digest)?;
    if observation_id != *record.message_id()
        || payload.logical_owner_id != runtime.logical_owner_id
        || envelope.partition_key.as_slice() != run_id
        || envelope.correlation_id.as_slice() != run_id
        || payload.page_sequence == 0
    {
        return Err(MailContactsSyncProviderEventErrorV1::InvalidPayload);
    }
    let run = persistence
        .load_run(&runtime.logical_owner_id, &run_id)
        .await
        .map_err(MailContactsSyncProviderEventErrorV1::Persistence)?;
    if run.draft.account_id != payload.account_id {
        return Err(MailContactsSyncProviderEventErrorV1::InvalidPayload);
    }
    let provider_kind = contacts_provider_kind(payload.provider_kind)?;
    let command_id = digest16(
        b"mail-contacts-sync/contact-command/v1",
        &run_id,
        record.message_id(),
    );
    let command = build_upsert_contact_command_outbox_record_v1(
        UpsertContactFromMailAddressBookEntryCommandV1 {
            command_id: command_id.to_vec(),
            logical_owner_id: runtime.logical_owner_id.clone(),
            source_account_id: payload.account_id,
            provider_kind: provider_kind as i32,
            provider_entry_id: payload.provider_entry_id,
            provider_etag: payload.provider_etag,
            display_name: payload.display_name,
            email_addresses: payload.email_addresses,
            phone_numbers: payload.phone_numbers,
            observed_at: payload.observed_at,
            source_revision: payload.source_revision,
            entry_digest: entry_digest.to_vec(),
        },
        runtime.now_unix_millis / 1_000 + MAIL_CONTACTS_SYNC_COMMAND_DEADLINE_SECONDS_V1,
        &ContactsCommandEnvelopeContextV1 {
            module_id: makosh_mail_contacts_sync_api::MAIL_CONTACTS_SYNC_MODULE_ID_V1.to_owned(),
            runtime_instance_id: runtime.runtime_instance_id.clone(),
            runtime_generation: runtime.runtime_generation,
            recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
            recorded_at_nanos: nanos(runtime.now_unix_millis),
        },
    )
    .map_err(|_| MailContactsSyncProviderEventErrorV1::InvalidPayload)?;
    persistence
        .accept_provider_entry(&MailContactsSyncEntryInputV1 {
            logical_owner_id: runtime.logical_owner_id.clone(),
            run_id,
            page_sequence: payload.page_sequence,
            observation_message_id: *record.message_id(),
            observation_envelope_sha256: *record.envelope_sha256(),
            contact_command_id: command_id,
            entry_digest,
            contact_command: outbox(&command),
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(MailContactsSyncProviderEventErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

pub async fn consume_mail_address_book_page_completed_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &MailContactsSyncProviderRuntimeContextV1,
) -> Result<bool, MailContactsSyncProviderEventErrorV1> {
    validate_runtime(runtime)?;
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = accepted_record(delivery.exact_bytes())?;
    let envelope = exact_result(
        &record,
        MailAddressBookContractV1::PageCompleted,
        ResultOutcomeV1::Succeeded,
    )?;
    let payload = MailAddressBookPageCompletedV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailContactsSyncProviderEventErrorV1::InvalidPayload)?;
    let run_id = id16(&payload.run_id)?;
    validate_result_identity(&envelope, &payload.command_id, &run_id)?;
    let outcome = persistence
        .accept_provider_page(&MailContactsSyncPageResultInputV1 {
            logical_owner_id: runtime.logical_owner_id.clone(),
            run_id,
            page_sequence: payload.page_sequence,
            message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            observed_entries: payload.observed_entries,
            next_continuation_cursor: payload.next_continuation_cursor,
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(MailContactsSyncProviderEventErrorV1::Persistence)?;
    if outcome == makosh_mail_contacts_sync_persistence::MailContactsSyncPersistenceOutcomeV1::PendingPrerequisites {
        return Ok(false);
    }
    crate::advance_ready_page_v1(persistence, runtime, run_id)
        .await
        .map_err(|error| match error {
            crate::MailContactsSyncProgressErrorV1::InvalidContext => {
                MailContactsSyncProviderEventErrorV1::InvalidPayload
            }
            crate::MailContactsSyncProgressErrorV1::Persistence(error) => {
                MailContactsSyncProviderEventErrorV1::Persistence(error)
            }
        })?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

pub async fn consume_mail_address_book_page_rejected_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &MailContactsSyncProviderRuntimeContextV1,
) -> Result<bool, MailContactsSyncProviderEventErrorV1> {
    validate_runtime(runtime)?;
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = accepted_record(delivery.exact_bytes())?;
    let envelope = exact_result(
        &record,
        MailAddressBookContractV1::PageRejected,
        ResultOutcomeV1::Rejected,
    )?;
    let payload = MailAddressBookPageRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailContactsSyncProviderEventErrorV1::InvalidPayload)?;
    let run_id = id16(&payload.run_id)?;
    validate_result_identity(&envelope, &payload.command_id, &run_id)?;
    let run = persistence
        .load_run(&runtime.logical_owner_id, &run_id)
        .await
        .map_err(MailContactsSyncProviderEventErrorV1::Persistence)?;
    persistence
        .apply_transition(MailContactsSyncTransitionInputV1 {
            logical_owner_id: runtime.logical_owner_id.clone(),
            run_id,
            direction: run.draft.direction,
            message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            transition: MailContactsSyncTransitionV1::Reject(provider_rejection(payload.code)?),
            next_command: None,
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(MailContactsSyncProviderEventErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

fn exact_result(
    record: &OutboxRecordV1,
    contract: MailAddressBookContractV1,
    outcome: ResultOutcomeV1,
) -> Result<makosh_events_protocol::v1::DurableEnvelopeV1, MailContactsSyncProviderEventErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailContactsSyncProviderEventErrorV1::InvalidEnvelope)?;
    validate_source_and_contract(
        &envelope.contract,
        contract,
        envelope
            .source
            .as_ref()
            .map(|source| source.module_id.as_str()),
        envelope
            .source
            .as_ref()
            .map_or(0, |source| source.runtime_generation),
    )?;
    let Some(Semantics::Result(result)) = envelope.semantics.as_ref() else {
        return Err(MailContactsSyncProviderEventErrorV1::InvalidEnvelope);
    };
    if result.outcome != outcome as i32 || result.execution_attempt == 0 {
        return Err(MailContactsSyncProviderEventErrorV1::InvalidEnvelope);
    }
    Ok(envelope)
}

fn validate_result_identity(
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
    payload_command_id: &[u8],
    run_id: &[u8; 16],
) -> Result<(), MailContactsSyncProviderEventErrorV1> {
    let Some(Semantics::Result(result)) = envelope.semantics.as_ref() else {
        return Err(MailContactsSyncProviderEventErrorV1::InvalidEnvelope);
    };
    let command_id = id16(payload_command_id)?;
    if result.command_id.as_slice() != command_id
        || result.command_message_id.as_slice() != command_id
        || envelope.partition_key.as_slice() != run_id
        || envelope.correlation_id.as_slice() != run_id
    {
        return Err(MailContactsSyncProviderEventErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn validate_source_and_contract(
    actual: &Option<ContractRefV1>,
    expected: MailAddressBookContractV1,
    source_module_id: Option<&str>,
    source_runtime_generation: u64,
) -> Result<(), MailContactsSyncProviderEventErrorV1> {
    if !exact_contract(actual.as_ref(), &expected.reference())
        || source_module_id != Some(MAIL_RUNTIME_MODULE_ID_V1)
        || source_runtime_generation == 0
    {
        return Err(MailContactsSyncProviderEventErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn exact_contract(actual: Option<&ContractRefV1>, expected: &ContractReferenceV1) -> bool {
    actual.is_some_and(|actual| {
        actual.owner == expected.owner
            && actual.name == expected.name
            && actual.major == expected.major
            && actual.revision == expected.revision
            && actual.schema_sha256 == expected.schema_sha256
    })
}

fn contacts_provider_kind(
    value: i32,
) -> Result<ContactsProviderKindV1, MailContactsSyncProviderEventErrorV1> {
    match MailAddressBookProviderKindV1::try_from(value) {
        Ok(MailAddressBookProviderKindV1::MailAddressBookProviderKindGooglePeople) => {
            Ok(ContactsProviderKindV1::MailAddressBookProviderKindGmail)
        }
        Ok(MailAddressBookProviderKindV1::MailAddressBookProviderKindIcloudCarddav) => {
            Ok(ContactsProviderKindV1::MailAddressBookProviderKindIcloud)
        }
        _ => Err(MailContactsSyncProviderEventErrorV1::InvalidPayload),
    }
}

fn provider_rejection(
    value: i32,
) -> Result<MailContactsSyncRejectCodeV1, MailContactsSyncProviderEventErrorV1> {
    match MailAddressBookRejectCodeV1::try_from(value) {
        Ok(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeAccountUnavailable) => {
            Ok(MailContactsSyncRejectCodeV1::AccountUnavailable)
        }
        Ok(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable)
        | Ok(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable) => {
            Ok(MailContactsSyncRejectCodeV1::ProviderUnavailable)
        }
        Ok(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeWriteScopeRequired)
        | Ok(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeReadOnlyProvider) => {
            Ok(MailContactsSyncRejectCodeV1::RemoteWriteBlocked)
        }
        Ok(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeEtagConflict) => {
            Ok(MailContactsSyncRejectCodeV1::EtagConflict)
        }
        Ok(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeOutcomeUnknown) => {
            Ok(MailContactsSyncRejectCodeV1::OutcomeUnknown)
        }
        Ok(MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest) => {
            Ok(MailContactsSyncRejectCodeV1::InvalidRequest)
        }
        Ok(MailAddressBookRejectCodeV1::MailAddressBookRejectCodePolicy) => {
            Ok(MailContactsSyncRejectCodeV1::Policy)
        }
        _ => Err(MailContactsSyncProviderEventErrorV1::InvalidPayload),
    }
}

fn accepted_record(bytes: &[u8]) -> Result<OutboxRecordV1, MailContactsSyncProviderEventErrorV1> {
    OutboxRecordV1::accept(bytes.to_vec())
        .map_err(|_| MailContactsSyncProviderEventErrorV1::InvalidEnvelope)
}

fn outbox(record: &OutboxRecordV1) -> OutboxEnvelopeV1 {
    OutboxEnvelopeV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    }
}

fn validate_runtime(
    runtime: &MailContactsSyncProviderRuntimeContextV1,
) -> Result<(), MailContactsSyncProviderEventErrorV1> {
    if runtime.logical_owner_id.is_empty()
        || runtime.runtime_instance_id.is_empty()
        || runtime.runtime_generation == 0
        || runtime.now_unix_millis <= 0
    {
        return Err(MailContactsSyncProviderEventErrorV1::InvalidPayload);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailContactsSyncProviderEventErrorV1> {
    nonzero_array(value)
}

fn id32(value: &[u8]) -> Result<[u8; 32], MailContactsSyncProviderEventErrorV1> {
    nonzero_array(value)
}

fn nonzero_array<const N: usize>(
    value: &[u8],
) -> Result<[u8; N], MailContactsSyncProviderEventErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|array: &[u8; N]| array.iter().any(|byte| *byte != 0))
        .ok_or(MailContactsSyncProviderEventErrorV1::InvalidPayload)
}

fn digest16(label: &[u8], left: &[u8], right: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(label);
    digest.update((left.len() as u64).to_be_bytes());
    digest.update(left);
    digest.update((right.len() as u64).to_be_bytes());
    digest.update(right);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

fn nanos(now_unix_millis: i64) -> i32 {
    i32::try_from((now_unix_millis % 1_000) * 1_000_000).unwrap_or_default()
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> MailContactsSyncProviderEventErrorV1 {
    MailContactsSyncProviderEventErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_kind_is_mapped_without_leaking_provider_selection_into_workflow() {
        assert_eq!(
            contacts_provider_kind(
                MailAddressBookProviderKindV1::MailAddressBookProviderKindGooglePeople as i32
            ),
            Ok(ContactsProviderKindV1::MailAddressBookProviderKindGmail)
        );
        assert!(contacts_provider_kind(0).is_err());
    }

    #[test]
    fn provider_rejections_are_bounded() {
        assert_eq!(
            provider_rejection(
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodeEtagConflict as i32
            ),
            Ok(MailContactsSyncRejectCodeV1::EtagConflict)
        );
        assert!(provider_rejection(0).is_err());
    }
}
