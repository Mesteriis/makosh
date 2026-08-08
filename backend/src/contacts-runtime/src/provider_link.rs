//! Contacts-owned reconciliation of provider links returned by Mail writes.

use makosh_contacts_command_api::{
    CONTACTS_MAIL_PROVIDER_LINK_COMMAND_CAPABILITY_ID_V1, ContactsCommandEnvelopeContextV1,
    bind_mail_address_book_provider_link_contract_reference_v1,
    build_bind_mail_address_book_provider_link_rejected_outbox_record_v1,
    build_mail_address_book_provider_link_bound_outbox_record_v1,
    wire::{
        BindMailAddressBookProviderLinkCommandV1, BindMailAddressBookProviderLinkRejectedV1,
        BindProviderLinkRejectCodeV1, MailAddressBookProviderKindV1,
        MailAddressBookProviderLinkBoundV1,
    },
};
use makosh_contacts_core::ContactProviderKindV1;
use makosh_contacts_persistence::{
    BindMailProviderLinkCommandV1, ContactProviderLinkBindOutcomeV1,
    ContactProviderLinkBindRejectCodeV1, ContactsOutboxRecordV1, ContactsPersistenceErrorV1,
    ContactsPersistenceV1,
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{CommandMetadataV1, ContractRefV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContactsProviderLinkErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(ContactsPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) struct ContactsProviderLinkRuntimeContextV1<'a> {
    pub logical_owner_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub(crate) async fn consume_bind_mail_provider_link_once_v1(
    persistence: &ContactsPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &ContactsProviderLinkRuntimeContextV1<'_>,
) -> Result<bool, ContactsProviderLinkErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ContactsProviderLinkErrorV1::InvalidEnvelope)?;
    let (identity, payload) = decode_command(&record, runtime)?;
    let input = BindMailProviderLinkCommandV1 {
        command_message_id: *record.message_id(),
        command_envelope_sha256: *record.envelope_sha256(),
        command_id: identity.command_id,
        logical_owner_id: payload.logical_owner_id,
        contact_id: identity.contact_id,
        expected_contact_revision: payload.expected_contact_revision,
        source_account_id: payload.source_account_id,
        provider_kind: provider_kind(payload.provider_kind)?,
        provider_entry_id: payload.provider_entry_id,
        provider_etag: payload.provider_etag,
        received_at_unix_millis: runtime.now_unix_millis,
        completed_at_unix_millis: runtime.now_unix_millis,
    };
    persistence
        .bind_mail_provider_link(&input, |outcome| {
            let context = envelope_context(runtime);
            let terminal = match outcome {
                ContactProviderLinkBindOutcomeV1::Bound { contact_revision } => {
                    build_mail_address_book_provider_link_bound_outbox_record_v1(
                        input.command_message_id,
                        MailAddressBookProviderLinkBoundV1 {
                            command_id: input.command_id.to_vec(),
                            contact_id: input.contact_id.to_vec(),
                            contact_revision,
                            logical_owner_id: input.logical_owner_id.clone(),
                        },
                        &context,
                    )
                }
                ContactProviderLinkBindOutcomeV1::Rejected(code) => {
                    build_bind_mail_address_book_provider_link_rejected_outbox_record_v1(
                        input.command_message_id,
                        BindMailAddressBookProviderLinkRejectedV1 {
                            command_id: input.command_id.to_vec(),
                            code: wire_reject(code) as i32,
                            logical_owner_id: input.logical_owner_id.clone(),
                        },
                        &context,
                    )
                }
            }
            .map_err(|_| ContactsPersistenceErrorV1::InvalidInput)?;
            Ok(outbox_record(&terminal))
        })
        .await
        .map_err(ContactsProviderLinkErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

struct CommandIdentityV1 {
    command_id: [u8; 16],
    contact_id: [u8; 16],
}

fn decode_command(
    record: &OutboxRecordV1,
    runtime: &ContactsProviderLinkRuntimeContextV1<'_>,
) -> Result<
    (CommandIdentityV1, BindMailAddressBookProviderLinkCommandV1),
    ContactsProviderLinkErrorV1,
> {
    if runtime.now_unix_millis <= 0 {
        return Err(ContactsProviderLinkErrorV1::InvalidPayload);
    }
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ContactsProviderLinkErrorV1::InvalidEnvelope)?;
    validate_contract(
        envelope.contract.as_ref(),
        &bind_mail_address_book_provider_link_contract_reference_v1(),
    )?;
    if envelope.source.as_ref().is_none_or(|source| {
        source.module_id != "makosh-mail-contacts-sync-runtime" || source.runtime_generation == 0
    }) {
        return Err(ContactsProviderLinkErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Command(CommandMetadataV1 {
        command_id,
        target_capability,
        deadline,
        ..
    })) = envelope.semantics
    else {
        return Err(ContactsProviderLinkErrorV1::InvalidEnvelope);
    };
    let payload = BindMailAddressBookProviderLinkCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| ContactsProviderLinkErrorV1::InvalidPayload)?;
    let command_id = id16(&command_id)?;
    let contact_id = id16(&payload.contact_id)?;
    if record.message_id() != &command_id
        || payload.command_id.as_slice() != command_id
        || payload.logical_owner_id != runtime.logical_owner_id
        || target_capability != CONTACTS_MAIL_PROVIDER_LINK_COMMAND_CAPABILITY_ID_V1
        || deadline.is_none_or(|deadline| deadline.seconds < runtime.now_unix_millis / 1_000)
    {
        return Err(ContactsProviderLinkErrorV1::InvalidPayload);
    }
    Ok((
        CommandIdentityV1 {
            command_id,
            contact_id,
        },
        payload,
    ))
}

fn validate_contract(
    actual: Option<&ContractRefV1>,
    expected: &ContractReferenceV1,
) -> Result<(), ContactsProviderLinkErrorV1> {
    if actual.is_none_or(|actual| {
        actual.owner != expected.owner
            || actual.name != expected.name
            || actual.major != expected.major
            || actual.revision != expected.revision
            || actual.schema_sha256 != expected.schema_sha256
    }) {
        return Err(ContactsProviderLinkErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn envelope_context(
    runtime: &ContactsProviderLinkRuntimeContextV1<'_>,
) -> ContactsCommandEnvelopeContextV1 {
    ContactsCommandEnvelopeContextV1 {
        module_id: "makosh-contacts-runtime".to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.to_owned(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
            .unwrap_or_default(),
    }
}

fn provider_kind(value: i32) -> Result<ContactProviderKindV1, ContactsProviderLinkErrorV1> {
    match MailAddressBookProviderKindV1::try_from(value) {
        Ok(MailAddressBookProviderKindV1::MailAddressBookProviderKindGmail) => {
            Ok(ContactProviderKindV1::Gmail)
        }
        Ok(MailAddressBookProviderKindV1::MailAddressBookProviderKindIcloud) => {
            Ok(ContactProviderKindV1::Icloud)
        }
        _ => Err(ContactsProviderLinkErrorV1::InvalidPayload),
    }
}

fn wire_reject(value: ContactProviderLinkBindRejectCodeV1) -> BindProviderLinkRejectCodeV1 {
    match value {
        ContactProviderLinkBindRejectCodeV1::InvalidRequest => {
            BindProviderLinkRejectCodeV1::BindProviderLinkRejectCodeInvalidRequest
        }
        ContactProviderLinkBindRejectCodeV1::ContactMissing => {
            BindProviderLinkRejectCodeV1::BindProviderLinkRejectCodeContactMissing
        }
        ContactProviderLinkBindRejectCodeV1::StaleContactRevision => {
            BindProviderLinkRejectCodeV1::BindProviderLinkRejectCodeStaleContactRevision
        }
        ContactProviderLinkBindRejectCodeV1::ProviderLinkConflict => {
            BindProviderLinkRejectCodeV1::BindProviderLinkRejectCodeProviderLinkConflict
        }
        ContactProviderLinkBindRejectCodeV1::Policy => {
            BindProviderLinkRejectCodeV1::BindProviderLinkRejectCodePolicy
        }
    }
}

fn outbox_record(record: &OutboxRecordV1) -> ContactsOutboxRecordV1 {
    ContactsOutboxRecordV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], ContactsProviderLinkErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ContactsProviderLinkErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> ContactsProviderLinkErrorV1 {
    ContactsProviderLinkErrorV1::EventUnavailable
}
