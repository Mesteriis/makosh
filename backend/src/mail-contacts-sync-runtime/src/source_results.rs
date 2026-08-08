use makosh_contacts_mail_sync_source_api::{
    CONTACT_MAIL_SYNC_SOURCE_MAX_BYTES_V1, CONTACT_MAIL_SYNC_SOURCE_MAX_PROOF_BYTES_V1,
    contact_mail_sync_source_prepared_contract_reference_v1,
    contact_mail_sync_source_rejected_contract_reference_v1,
    wire::{ContactMailSyncSourcePreparedV1, ContactMailSyncSourceRejectedV1},
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ContractRefV1, ResultMetadataV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_address_book_contract::{
    MailAddressBookEnvelopeContextV1, build_upsert_mail_address_book_entry_command_v1,
    wire::UpsertMailAddressBookEntryCommandV1,
};
use makosh_mail_contacts_sync_persistence::{
    CompleteContactMailSyncSourceV1, MailContactsSyncPersistenceErrorV1,
    MailContactsSyncPersistenceV1, OutboxEnvelopeV1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MailContactsSyncSourceResultErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(MailContactsSyncPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) struct MailContactsSyncSourceResultContextV1<'a> {
    pub logical_owner_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub(crate) async fn consume_source_prepared_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &MailContactsSyncSourceResultContextV1<'_>,
) -> Result<bool, MailContactsSyncSourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailContactsSyncSourceResultErrorV1::InvalidEnvelope)?;
    let prepared = decode_prepared(&record, context)?;
    let operation_id = id16(&prepared.operation_id)?;
    let operation = persistence
        .load_reverse_operation(context.logical_owner_id, operation_id)
        .await
        .map_err(MailContactsSyncSourceResultErrorV1::Persistence)?;
    if operation.state != 1
        || operation.contact_id.as_slice() != prepared.contact_id
        || operation.contact_revision != prepared.contact_revision
    {
        return Err(MailContactsSyncSourceResultErrorV1::InvalidPayload);
    }
    let receipt = prepared
        .source_content
        .ok_or(MailContactsSyncSourceResultErrorV1::InvalidPayload)?;
    let reference_id = id16(&receipt.reference_id)?;
    let sha256 = id32(&receipt.sha256)?;
    if !(1..=CONTACT_MAIL_SYNC_SOURCE_MAX_BYTES_V1).contains(&receipt.declared_bytes)
        || receipt.custody_transfer_source_proof.is_empty()
        || receipt.custody_transfer_source_proof.len() > CONTACT_MAIL_SYNC_SOURCE_MAX_PROOF_BYTES_V1
    {
        return Err(MailContactsSyncSourceResultErrorV1::InvalidPayload);
    }
    let command_id = mail_command_id(operation_id);
    let command = build_upsert_mail_address_book_entry_command_v1(
        UpsertMailAddressBookEntryCommandV1 {
            command_id: command_id.to_vec(),
            run_id: operation_id.to_vec(),
            logical_owner_id: context.logical_owner_id.to_owned(),
            account_id: operation.account_id,
            contact_snapshot_reference_id: reference_id.to_vec(),
            contact_snapshot_sha256: sha256.to_vec(),
            expected_contact_revision: operation.contact_revision,
            contact_snapshot_declared_bytes: receipt.declared_bytes,
            contact_snapshot_custody_source_proof: receipt.custody_transfer_source_proof,
        },
        context.now_unix_millis / 1_000 + crate::MAIL_CONTACTS_SYNC_COMMAND_DEADLINE_SECONDS_V1,
        &envelope_context(context),
    )
    .map_err(|_| MailContactsSyncSourceResultErrorV1::InvalidPayload)?;
    persistence
        .complete_contact_mail_sync_source(&CompleteContactMailSyncSourceV1 {
            logical_owner_id: context.logical_owner_id.to_owned(),
            result_message_id: *record.message_id(),
            result_envelope_sha256: *record.envelope_sha256(),
            operation_id,
            mail_command: Some(outbox(&command)),
            rejected: false,
            occurred_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map_err(MailContactsSyncSourceResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

pub(crate) async fn consume_source_rejected_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &MailContactsSyncSourceResultContextV1<'_>,
) -> Result<bool, MailContactsSyncSourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailContactsSyncSourceResultErrorV1::InvalidEnvelope)?;
    let rejected = decode_rejected(&record, context)?;
    let operation_id = id16(&rejected.operation_id)?;
    let operation = persistence
        .load_reverse_operation(context.logical_owner_id, operation_id)
        .await
        .map_err(MailContactsSyncSourceResultErrorV1::Persistence)?;
    if operation.state != 1 || rejected.code == 0 {
        return Err(MailContactsSyncSourceResultErrorV1::InvalidPayload);
    }
    persistence
        .complete_contact_mail_sync_source(&CompleteContactMailSyncSourceV1 {
            logical_owner_id: context.logical_owner_id.to_owned(),
            result_message_id: *record.message_id(),
            result_envelope_sha256: *record.envelope_sha256(),
            operation_id,
            mail_command: None,
            rejected: true,
            occurred_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map_err(MailContactsSyncSourceResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

fn decode_prepared(
    record: &OutboxRecordV1,
    context: &MailContactsSyncSourceResultContextV1<'_>,
) -> Result<ContactMailSyncSourcePreparedV1, MailContactsSyncSourceResultErrorV1> {
    let envelope = decode_result(
        record,
        context,
        &contact_mail_sync_source_prepared_contract_reference_v1(),
    )?;
    let payload = ContactMailSyncSourcePreparedV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailContactsSyncSourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != context.logical_owner_id
        || !result_identity_matches(&envelope, &payload.operation_id, record)
    {
        return Err(MailContactsSyncSourceResultErrorV1::InvalidPayload);
    }
    Ok(payload)
}

fn decode_rejected(
    record: &OutboxRecordV1,
    context: &MailContactsSyncSourceResultContextV1<'_>,
) -> Result<ContactMailSyncSourceRejectedV1, MailContactsSyncSourceResultErrorV1> {
    let envelope = decode_result(
        record,
        context,
        &contact_mail_sync_source_rejected_contract_reference_v1(),
    )?;
    let payload = ContactMailSyncSourceRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailContactsSyncSourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != context.logical_owner_id
        || !result_identity_matches(&envelope, &payload.operation_id, record)
    {
        return Err(MailContactsSyncSourceResultErrorV1::InvalidPayload);
    }
    Ok(payload)
}

fn result_identity_matches(
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
    operation_id: &[u8],
    record: &OutboxRecordV1,
) -> bool {
    let Some(Semantics::Result(result)) = envelope.semantics.as_ref() else {
        return false;
    };
    operation_id.len() == 16
        && operation_id.iter().any(|byte| *byte != 0)
        && result.command_id == operation_id
        && result.command_message_id == operation_id
        && envelope.partition_key == operation_id
        && envelope.correlation_id == operation_id
        && envelope.causation_message_id == operation_id
        && record.message_id() != operation_id
}

fn decode_result(
    record: &OutboxRecordV1,
    context: &MailContactsSyncSourceResultContextV1<'_>,
    expected: &ContractReferenceV1,
) -> Result<makosh_events_protocol::v1::DurableEnvelopeV1, MailContactsSyncSourceResultErrorV1> {
    if context.now_unix_millis <= 0 {
        return Err(MailContactsSyncSourceResultErrorV1::InvalidPayload);
    }
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailContactsSyncSourceResultErrorV1::InvalidEnvelope)?;
    validate_contract(envelope.contract.as_ref(), expected)?;
    if envelope.source.as_ref().is_none_or(|source| {
        source.module_id != "makosh-contacts-runtime" || source.runtime_generation == 0
    }) || !matches!(
        envelope.semantics,
        Some(Semantics::Result(ResultMetadataV1 { .. }))
    ) {
        return Err(MailContactsSyncSourceResultErrorV1::InvalidEnvelope);
    }
    Ok(envelope)
}

fn validate_contract(
    actual: Option<&ContractRefV1>,
    expected: &ContractReferenceV1,
) -> Result<(), MailContactsSyncSourceResultErrorV1> {
    if actual.is_none_or(|actual| {
        actual.owner != expected.owner
            || actual.name != expected.name
            || actual.major != expected.major
            || actual.revision != expected.revision
            || actual.schema_sha256 != expected.schema_sha256
    }) {
        return Err(MailContactsSyncSourceResultErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn envelope_context(
    context: &MailContactsSyncSourceResultContextV1<'_>,
) -> MailAddressBookEnvelopeContextV1 {
    MailAddressBookEnvelopeContextV1 {
        module_id: "makosh-mail-contacts-sync-runtime".to_owned(),
        runtime_instance_id: context.runtime_instance_id.to_owned(),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((context.now_unix_millis % 1_000) * 1_000_000)
            .unwrap_or_default(),
    }
}

fn mail_command_id(operation_id: [u8; 16]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"mail-contacts-sync-provider-upsert-command-v1");
    hash.update(operation_id);
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn outbox(record: &OutboxRecordV1) -> OutboxEnvelopeV1 {
    OutboxEnvelopeV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailContactsSyncSourceResultErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(MailContactsSyncSourceResultErrorV1::InvalidPayload)
}

fn id32(value: &[u8]) -> Result<[u8; 32], MailContactsSyncSourceResultErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(MailContactsSyncSourceResultErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> MailContactsSyncSourceResultErrorV1 {
    MailContactsSyncSourceResultErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_command_identity_is_operation_stable() {
        assert_eq!(mail_command_id([1; 16]), mail_command_id([1; 16]));
        assert_ne!(mail_command_id([1; 16]), mail_command_id([2; 16]));
    }
}
