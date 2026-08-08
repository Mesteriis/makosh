use makosh_contacts_command_api::{
    contact_upsert_rejected_contract_reference_v1, contact_upserted_contract_reference_v1,
    wire::{
        ContactUpsertFromMailAddressBookEntryRejectedV1, ContactUpsertOutcomeV1,
        ContactUpsertedFromMailAddressBookEntryV1,
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
use makosh_mail_contacts_sync_persistence::{
    MailContactsSyncContactOutcomeV1, MailContactsSyncEntryOutcomeInputV1,
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;

use crate::MailContactsSyncProviderRuntimeContextV1;

const CONTACTS_RUNTIME_MODULE_ID_V1: &str = "makosh-contacts-runtime";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncContactsResultErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(MailContactsSyncPersistenceErrorV1),
    EventUnavailable,
}

pub async fn consume_contact_upserted_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &MailContactsSyncProviderRuntimeContextV1,
) -> Result<bool, MailContactsSyncContactsResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = accepted_record(delivery.exact_bytes())?;
    let envelope = exact_result(
        &record,
        &contact_upserted_contract_reference_v1(),
        ResultOutcomeV1::Succeeded,
    )?;
    let payload = ContactUpsertedFromMailAddressBookEntryV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailContactsSyncContactsResultErrorV1::InvalidPayload)?;
    let command_id = result_command_id(&envelope, &payload.command_id)?;
    if payload.logical_owner_id != runtime.logical_owner_id
        || payload.contact_revision == 0
        || id16(&payload.contact_id).is_err()
    {
        return Err(MailContactsSyncContactsResultErrorV1::InvalidPayload);
    }
    let outcome = match ContactUpsertOutcomeV1::try_from(payload.outcome) {
        Ok(ContactUpsertOutcomeV1::ContactUpsertOutcomeCreated) => {
            MailContactsSyncContactOutcomeV1::Created
        }
        Ok(ContactUpsertOutcomeV1::ContactUpsertOutcomeUpdated) => {
            MailContactsSyncContactOutcomeV1::Updated
        }
        Ok(ContactUpsertOutcomeV1::ContactUpsertOutcomeUnchanged) => {
            MailContactsSyncContactOutcomeV1::Unchanged
        }
        _ => return Err(MailContactsSyncContactsResultErrorV1::InvalidPayload),
    };
    persist_and_ack(persistence, delivery, &record, runtime, command_id, outcome).await
}

pub async fn consume_contact_upsert_rejected_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &MailContactsSyncProviderRuntimeContextV1,
) -> Result<bool, MailContactsSyncContactsResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = accepted_record(delivery.exact_bytes())?;
    let envelope = exact_result(
        &record,
        &contact_upsert_rejected_contract_reference_v1(),
        ResultOutcomeV1::Rejected,
    )?;
    let payload =
        ContactUpsertFromMailAddressBookEntryRejectedV1::decode(envelope.payload.as_slice())
            .map_err(|_| MailContactsSyncContactsResultErrorV1::InvalidPayload)?;
    let command_id = result_command_id(&envelope, &payload.command_id)?;
    if payload.logical_owner_id != runtime.logical_owner_id || payload.code == 0 {
        return Err(MailContactsSyncContactsResultErrorV1::InvalidPayload);
    }
    persist_and_ack(
        persistence,
        delivery,
        &record,
        runtime,
        command_id,
        MailContactsSyncContactOutcomeV1::Rejected,
    )
    .await
}

async fn persist_and_ack(
    persistence: &MailContactsSyncPersistenceV1,
    delivery: makosh_events_jetstream::RuntimePullDeliveryV1,
    record: &OutboxRecordV1,
    runtime: &MailContactsSyncProviderRuntimeContextV1,
    command_id: [u8; 16],
    outcome: MailContactsSyncContactOutcomeV1,
) -> Result<bool, MailContactsSyncContactsResultErrorV1> {
    if runtime.logical_owner_id.is_empty() || runtime.now_unix_millis <= 0 {
        return Err(MailContactsSyncContactsResultErrorV1::InvalidPayload);
    }
    persistence
        .accept_contact_outcome(&MailContactsSyncEntryOutcomeInputV1 {
            logical_owner_id: runtime.logical_owner_id.clone(),
            contact_command_id: command_id,
            message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            outcome,
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(MailContactsSyncContactsResultErrorV1::Persistence)?;
    let run_id = envelope_run_id_for_command(persistence, runtime, command_id).await?;
    crate::advance_ready_page_v1(persistence, runtime, run_id)
        .await
        .map_err(|error| match error {
            crate::MailContactsSyncProgressErrorV1::InvalidContext => {
                MailContactsSyncContactsResultErrorV1::InvalidPayload
            }
            crate::MailContactsSyncProgressErrorV1::Persistence(error) => {
                MailContactsSyncContactsResultErrorV1::Persistence(error)
            }
        })?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

async fn envelope_run_id_for_command(
    persistence: &MailContactsSyncPersistenceV1,
    runtime: &MailContactsSyncProviderRuntimeContextV1,
    command_id: [u8; 16],
) -> Result<[u8; 16], MailContactsSyncContactsResultErrorV1> {
    persistence
        .run_id_for_contact_command(&runtime.logical_owner_id, &command_id)
        .await
        .map_err(MailContactsSyncContactsResultErrorV1::Persistence)
}

fn exact_result(
    record: &OutboxRecordV1,
    expected: &ContractReferenceV1,
    expected_outcome: ResultOutcomeV1,
) -> Result<makosh_events_protocol::v1::DurableEnvelopeV1, MailContactsSyncContactsResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailContactsSyncContactsResultErrorV1::InvalidEnvelope)?;
    let Some(Semantics::Result(result)) = envelope.semantics.as_ref() else {
        return Err(MailContactsSyncContactsResultErrorV1::InvalidEnvelope);
    };
    if !exact_contract(envelope.contract.as_ref(), expected)
        || envelope.source.as_ref().is_none_or(|source| {
            source.module_id != CONTACTS_RUNTIME_MODULE_ID_V1 || source.runtime_generation == 0
        })
        || result.outcome != expected_outcome as i32
        || result.execution_attempt == 0
    {
        return Err(MailContactsSyncContactsResultErrorV1::InvalidEnvelope);
    }
    Ok(envelope)
}

fn result_command_id(
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
    payload_command_id: &[u8],
) -> Result<[u8; 16], MailContactsSyncContactsResultErrorV1> {
    let command_id = id16(payload_command_id)?;
    let Some(Semantics::Result(result)) = envelope.semantics.as_ref() else {
        return Err(MailContactsSyncContactsResultErrorV1::InvalidEnvelope);
    };
    if result.command_id.as_slice() != command_id
        || result.command_message_id.as_slice() != command_id
    {
        return Err(MailContactsSyncContactsResultErrorV1::InvalidEnvelope);
    }
    Ok(command_id)
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

fn accepted_record(bytes: &[u8]) -> Result<OutboxRecordV1, MailContactsSyncContactsResultErrorV1> {
    OutboxRecordV1::accept(bytes.to_vec())
        .map_err(|_| MailContactsSyncContactsResultErrorV1::InvalidEnvelope)
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailContactsSyncContactsResultErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(MailContactsSyncContactsResultErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> MailContactsSyncContactsResultErrorV1 {
    MailContactsSyncContactsResultErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contact_outcome_mapping_is_exact() {
        assert_eq!(
            ContactUpsertOutcomeV1::try_from(1),
            Ok(ContactUpsertOutcomeV1::ContactUpsertOutcomeCreated)
        );
        assert!(ContactUpsertOutcomeV1::try_from(0).is_ok());
    }
}
