//! Workflow reconciliation of Contacts-owned provider-link terminal results.

use makosh_contacts_command_api::{
    CONTACTS_MODULE_ID_V1, bind_mail_address_book_provider_link_rejected_contract_reference_v1,
    mail_address_book_provider_link_bound_contract_reference_v1,
    wire::{
        BindMailAddressBookProviderLinkRejectedV1, BindProviderLinkRejectCodeV1,
        MailAddressBookProviderLinkBoundV1,
    },
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ContractRefV1, ResultMetadataV1, ResultOutcomeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_contacts_sync_core::MailContactsSyncRejectCodeV1;
use makosh_mail_contacts_sync_persistence::{
    CompleteContactsProviderLinkV1, MailContactsSyncPersistenceErrorV1,
    MailContactsSyncPersistenceV1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MailContactsSyncProviderLinkResultErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(MailContactsSyncPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) struct MailContactsSyncProviderLinkResultContextV1<'a> {
    pub logical_owner_id: &'a str,
    pub now_unix_millis: i64,
}

pub(crate) async fn consume_provider_link_bound_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &MailContactsSyncProviderLinkResultContextV1<'_>,
) -> Result<bool, MailContactsSyncProviderLinkResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailContactsSyncProviderLinkResultErrorV1::InvalidEnvelope)?;
    let envelope = decode_result(
        &record,
        &mail_address_book_provider_link_bound_contract_reference_v1(),
        ResultOutcomeV1::Succeeded,
        context,
    )?;
    let payload = MailAddressBookProviderLinkBoundV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailContactsSyncProviderLinkResultErrorV1::InvalidPayload)?;
    let identity = result_identity(&record, &envelope, &payload.command_id)?;
    let operation_id = persistence
        .provider_link_operation_for_command(context.logical_owner_id, identity.command_message_id)
        .await
        .map_err(MailContactsSyncProviderLinkResultErrorV1::Persistence)?;
    let operation = persistence
        .load_reverse_operation(context.logical_owner_id, operation_id)
        .await
        .map_err(MailContactsSyncProviderLinkResultErrorV1::Persistence)?;
    if payload.logical_owner_id != context.logical_owner_id
        || payload.contact_id.as_slice() != operation.contact_id
        || payload.contact_revision != operation.contact_revision
    {
        return Err(MailContactsSyncProviderLinkResultErrorV1::InvalidPayload);
    }
    complete(persistence, &record, operation_id, identity, None, context).await?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

pub(crate) async fn consume_provider_link_rejected_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &MailContactsSyncProviderLinkResultContextV1<'_>,
) -> Result<bool, MailContactsSyncProviderLinkResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailContactsSyncProviderLinkResultErrorV1::InvalidEnvelope)?;
    let envelope = decode_result(
        &record,
        &bind_mail_address_book_provider_link_rejected_contract_reference_v1(),
        ResultOutcomeV1::Rejected,
        context,
    )?;
    let payload = BindMailAddressBookProviderLinkRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailContactsSyncProviderLinkResultErrorV1::InvalidPayload)?;
    let identity = result_identity(&record, &envelope, &payload.command_id)?;
    if payload.logical_owner_id != context.logical_owner_id
        || matches!(
            BindProviderLinkRejectCodeV1::try_from(payload.code),
            Ok(BindProviderLinkRejectCodeV1::BindProviderLinkRejectCodeUnspecified) | Err(_)
        )
    {
        return Err(MailContactsSyncProviderLinkResultErrorV1::InvalidPayload);
    }
    let operation_id = persistence
        .provider_link_operation_for_command(context.logical_owner_id, identity.command_message_id)
        .await
        .map_err(MailContactsSyncProviderLinkResultErrorV1::Persistence)?;
    complete(
        persistence,
        &record,
        operation_id,
        identity,
        Some(MailContactsSyncRejectCodeV1::ContactsRejected),
        context,
    )
    .await?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

#[derive(Clone, Copy)]
struct ResultIdentityV1 {
    command_message_id: [u8; 16],
}

async fn complete(
    persistence: &MailContactsSyncPersistenceV1,
    record: &OutboxRecordV1,
    operation_id: [u8; 16],
    identity: ResultIdentityV1,
    reject_code: Option<MailContactsSyncRejectCodeV1>,
    context: &MailContactsSyncProviderLinkResultContextV1<'_>,
) -> Result<(), MailContactsSyncProviderLinkResultErrorV1> {
    persistence
        .complete_contacts_provider_link(&CompleteContactsProviderLinkV1 {
            logical_owner_id: context.logical_owner_id.to_owned(),
            result_message_id: *record.message_id(),
            result_envelope_sha256: *record.envelope_sha256(),
            operation_id,
            contacts_command_message_id: identity.command_message_id,
            reject_code,
            occurred_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map(|_| ())
        .map_err(MailContactsSyncProviderLinkResultErrorV1::Persistence)
}

fn decode_result(
    record: &OutboxRecordV1,
    expected: &ContractReferenceV1,
    expected_outcome: ResultOutcomeV1,
    context: &MailContactsSyncProviderLinkResultContextV1<'_>,
) -> Result<makosh_events_protocol::v1::DurableEnvelopeV1, MailContactsSyncProviderLinkResultErrorV1>
{
    if context.now_unix_millis <= 0 {
        return Err(MailContactsSyncProviderLinkResultErrorV1::InvalidPayload);
    }
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailContactsSyncProviderLinkResultErrorV1::InvalidEnvelope)?;
    validate_contract(envelope.contract.as_ref(), expected)?;
    if envelope.source.as_ref().is_none_or(|source| {
        source.module_id != CONTACTS_MODULE_ID_V1 || source.runtime_generation == 0
    }) || !matches!(
        envelope.semantics,
        Some(Semantics::Result(ResultMetadataV1 { outcome, .. })) if outcome == expected_outcome as i32
    ) {
        return Err(MailContactsSyncProviderLinkResultErrorV1::InvalidEnvelope);
    }
    Ok(envelope)
}

fn result_identity(
    record: &OutboxRecordV1,
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
    payload_command_id: &[u8],
) -> Result<ResultIdentityV1, MailContactsSyncProviderLinkResultErrorV1> {
    let command_id = id16(payload_command_id)?;
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailContactsSyncProviderLinkResultErrorV1::InvalidEnvelope);
    };
    let command_message_id = id16(&metadata.command_message_id)?;
    if metadata.command_id.as_slice() != command_id
        || envelope.causation_message_id.as_slice() != command_message_id
        || record.message_id() == &command_message_id
    {
        return Err(MailContactsSyncProviderLinkResultErrorV1::InvalidEnvelope);
    }
    Ok(ResultIdentityV1 { command_message_id })
}

fn validate_contract(
    actual: Option<&ContractRefV1>,
    expected: &ContractReferenceV1,
) -> Result<(), MailContactsSyncProviderLinkResultErrorV1> {
    if actual.is_none_or(|actual| {
        actual.owner != expected.owner
            || actual.name != expected.name
            || actual.major != expected.major
            || actual.revision != expected.revision
            || actual.schema_sha256 != expected.schema_sha256
    }) {
        return Err(MailContactsSyncProviderLinkResultErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailContactsSyncProviderLinkResultErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(MailContactsSyncProviderLinkResultErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> MailContactsSyncProviderLinkResultErrorV1 {
    MailContactsSyncProviderLinkResultErrorV1::EventUnavailable
}
