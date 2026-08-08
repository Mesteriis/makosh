use makosh_contacts_command_api::{
    CONTACTS_MAIL_IDENTITY_COMMAND_CAPABILITY_ID_V1, CONTACTS_MODULE_ID_V1,
    ContactsCommandEnvelopeContextV1, build_contact_upsert_rejected_outbox_record_v1,
    build_contact_upserted_outbox_record_v1, upsert_contact_command_contract_reference_v1,
    wire::{
        ContactUpsertFromMailAddressBookEntryRejectedV1,
        ContactUpsertOutcomeV1 as WireContactUpsertOutcomeV1,
        ContactUpsertRejectCodeV1 as WireContactUpsertRejectCodeV1,
        ContactUpsertedFromMailAddressBookEntryV1, MailAddressBookProviderKindV1,
        UpsertContactFromMailAddressBookEntryCommandV1,
    },
};
use makosh_contacts_core::{
    ContactProviderKindV1, ContactProviderProvenanceV1, ContactTimestampV1, ContactUpsertDraftV1,
    ContactUpsertOutcomeV1,
};
use makosh_contacts_mail_sync_source_api::{
    ContactsMailSyncSourceEnvelopeContextV1,
    build_contact_changed_for_mail_sync_outbox_record_caused_by_v1,
    wire::ContactChangedForMailSyncV1,
};
use makosh_contacts_persistence::{
    ApplyMailEntryCommandV1, ContactMailEntryRejectCodeV1, ContactMutationOutboxV1,
    ContactsOutboxRecordV1, ContactsPersistenceErrorV1, ContactsPersistenceV1,
    RejectMailEntryCommandV1,
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
pub(crate) enum ContactsCommandErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(ContactsPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) struct ContactsCommandRuntimeContextV1<'a> {
    pub logical_owner_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

struct CommandIdentityV1 {
    command_message_id: [u8; 16],
    command_envelope_sha256: [u8; 32],
    command_id: [u8; 16],
    logical_owner_id: String,
    entry_digest: [u8; 32],
}

enum DecodedCommandV1 {
    Apply {
        identity: CommandIdentityV1,
        draft: Box<ContactUpsertDraftV1>,
    },
    Reject(CommandIdentityV1),
}

pub(crate) async fn consume_contacts_command_once_v1(
    persistence: &ContactsPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &ContactsCommandRuntimeContextV1<'_>,
) -> Result<bool, ContactsCommandErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ContactsCommandErrorV1::InvalidEnvelope)?;
    match decode_command(&record, runtime)? {
        DecodedCommandV1::Reject(identity) => {
            persist_rejection(
                persistence,
                identity,
                ContactMailEntryRejectCodeV1::InvalidRequest,
                runtime,
            )
            .await?;
        }
        DecodedCommandV1::Apply { identity, draft } => {
            let input = ApplyMailEntryCommandV1 {
                command_message_id: identity.command_message_id,
                command_envelope_sha256: identity.command_envelope_sha256,
                command_id: identity.command_id,
                draft: *draft,
                received_at_unix_millis: runtime.now_unix_millis,
                completed_at_unix_millis: runtime.now_unix_millis,
            };
            let applied = persistence
                .apply_mail_entry(&input, |contact, outcome| {
                    let record = build_contact_upserted_outbox_record_v1(
                        identity.command_message_id,
                        ContactUpsertedFromMailAddressBookEntryV1 {
                            command_id: identity.command_id.to_vec(),
                            contact_id: contact.contact_id.to_vec(),
                            contact_revision: contact.contact_revision,
                            outcome: wire_outcome(outcome) as i32,
                            logical_owner_id: identity.logical_owner_id.clone(),
                        },
                        &envelope_context(runtime),
                    )
                    .map_err(|_| ContactsPersistenceErrorV1::InvalidInput)?;
                    let changed_event = if outcome == ContactUpsertOutcomeV1::Unchanged {
                        None
                    } else {
                        let changed =
                            build_contact_changed_for_mail_sync_outbox_record_caused_by_v1(
                                identity.command_message_id,
                                ContactChangedForMailSyncV1 {
                                    contact_id: contact.contact_id.to_vec(),
                                    contact_revision: contact.contact_revision,
                                    logical_owner_id: identity.logical_owner_id.clone(),
                                },
                                &source_envelope_context(runtime),
                            )
                            .map_err(|_| ContactsPersistenceErrorV1::InvalidInput)?;
                        Some(outbox_record(&changed))
                    };
                    Ok(ContactMutationOutboxV1 {
                        terminal_result: outbox_record(&record),
                        changed_event,
                    })
                })
                .await;
            if let Err(error) = applied {
                let code = business_reject_code(error)
                    .ok_or(ContactsCommandErrorV1::Persistence(error))?;
                persist_rejection(persistence, identity, code, runtime).await?;
            }
        }
    }
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

async fn persist_rejection(
    persistence: &ContactsPersistenceV1,
    identity: CommandIdentityV1,
    code: ContactMailEntryRejectCodeV1,
    runtime: &ContactsCommandRuntimeContextV1<'_>,
) -> Result<(), ContactsCommandErrorV1> {
    let terminal = build_contact_upsert_rejected_outbox_record_v1(
        identity.command_message_id,
        ContactUpsertFromMailAddressBookEntryRejectedV1 {
            command_id: identity.command_id.to_vec(),
            code: wire_reject_code(code) as i32,
            logical_owner_id: identity.logical_owner_id.clone(),
        },
        &envelope_context(runtime),
    )
    .map_err(|_| ContactsCommandErrorV1::InvalidPayload)?;
    persistence
        .reject_mail_entry(&RejectMailEntryCommandV1 {
            command_message_id: identity.command_message_id,
            command_envelope_sha256: identity.command_envelope_sha256,
            command_id: identity.command_id,
            logical_owner_id: identity.logical_owner_id,
            entry_digest: identity.entry_digest,
            received_at_unix_millis: runtime.now_unix_millis,
            completed_at_unix_millis: runtime.now_unix_millis,
            code,
            terminal_result: outbox_record(&terminal),
        })
        .await
        .map(|_| ())
        .map_err(ContactsCommandErrorV1::Persistence)
}

fn decode_command(
    record: &OutboxRecordV1,
    runtime: &ContactsCommandRuntimeContextV1<'_>,
) -> Result<DecodedCommandV1, ContactsCommandErrorV1> {
    if runtime.now_unix_millis <= 0 {
        return Err(ContactsCommandErrorV1::InvalidPayload);
    }
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ContactsCommandErrorV1::InvalidEnvelope)?;
    validate_contract(
        envelope.contract.as_ref(),
        &upsert_contact_command_contract_reference_v1(),
    )?;
    let Some(Semantics::Command(CommandMetadataV1 {
        command_id,
        target_capability,
        deadline,
        ..
    })) = envelope.semantics
    else {
        return Err(ContactsCommandErrorV1::InvalidEnvelope);
    };
    if command_id.as_slice() != record.message_id()
        || target_capability != CONTACTS_MAIL_IDENTITY_COMMAND_CAPABILITY_ID_V1
        || envelope.partition_key.len() != 16
    {
        return Err(ContactsCommandErrorV1::InvalidEnvelope);
    }
    let payload =
        UpsertContactFromMailAddressBookEntryCommandV1::decode(envelope.payload.as_slice())
            .map_err(|_| ContactsCommandErrorV1::InvalidPayload)?;
    let command_id = id16(&payload.command_id)?;
    if command_id != *record.message_id()
        || payload.logical_owner_id != runtime.logical_owner_id
        || payload.logical_owner_id.is_empty()
        || payload.logical_owner_id.len() > 128
    {
        return Err(ContactsCommandErrorV1::InvalidPayload);
    }
    let identity = CommandIdentityV1 {
        command_message_id: *record.message_id(),
        command_envelope_sha256: *record.envelope_sha256(),
        command_id,
        logical_owner_id: payload.logical_owner_id.clone(),
        entry_digest: id32(&payload.entry_digest).unwrap_or(*record.envelope_sha256()),
    };
    let expired = deadline.is_none_or(|deadline| {
        deadline.seconds < runtime.now_unix_millis / 1_000
            || (deadline.seconds == runtime.now_unix_millis / 1_000
                && i64::from(deadline.nanos) <= (runtime.now_unix_millis % 1_000) * 1_000_000)
    });
    let Some(observed_at) = payload.observed_at else {
        return Ok(DecodedCommandV1::Reject(identity));
    };
    let provider_kind = match MailAddressBookProviderKindV1::try_from(payload.provider_kind) {
        Ok(MailAddressBookProviderKindV1::MailAddressBookProviderKindGmail) => {
            ContactProviderKindV1::Gmail
        }
        Ok(MailAddressBookProviderKindV1::MailAddressBookProviderKindIcloud) => {
            ContactProviderKindV1::Icloud
        }
        _ => return Ok(DecodedCommandV1::Reject(identity)),
    };
    if expired || payload.source_revision == 0 || id32(&payload.entry_digest).is_err() {
        return Ok(DecodedCommandV1::Reject(identity));
    }
    Ok(DecodedCommandV1::Apply {
        identity,
        draft: Box::new(ContactUpsertDraftV1 {
            logical_owner_id: payload.logical_owner_id,
            display_name: payload.display_name,
            email_addresses: payload.email_addresses,
            phone_numbers: payload.phone_numbers,
            provenance: ContactProviderProvenanceV1 {
                source_account_id: payload.source_account_id,
                provider_kind,
                provider_entry_id: payload.provider_entry_id,
                provider_etag: payload.provider_etag,
                source_revision: payload.source_revision,
                entry_digest: id32(&payload.entry_digest)?,
                observed_at: ContactTimestampV1 {
                    unix_seconds: observed_at.seconds,
                    nanos: observed_at.nanos,
                },
            },
        }),
    })
}

fn validate_contract(
    actual: Option<&ContractRefV1>,
    expected: &ContractReferenceV1,
) -> Result<(), ContactsCommandErrorV1> {
    if actual.is_none_or(|actual| {
        actual.owner != expected.owner
            || actual.name != expected.name
            || actual.major != expected.major
            || actual.revision != expected.revision
            || actual.schema_sha256 != expected.schema_sha256
    }) {
        return Err(ContactsCommandErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn business_reject_code(error: ContactsPersistenceErrorV1) -> Option<ContactMailEntryRejectCodeV1> {
    match error {
        ContactsPersistenceErrorV1::InvalidInput => {
            Some(ContactMailEntryRejectCodeV1::InvalidRequest)
        }
        ContactsPersistenceErrorV1::IdentityAmbiguous => {
            Some(ContactMailEntryRejectCodeV1::IdentityAmbiguous)
        }
        ContactsPersistenceErrorV1::ProviderLinkConflict => {
            Some(ContactMailEntryRejectCodeV1::ProviderLinkConflict)
        }
        ContactsPersistenceErrorV1::StaleSource => Some(ContactMailEntryRejectCodeV1::StaleSource),
        ContactsPersistenceErrorV1::PolicyRejected => Some(ContactMailEntryRejectCodeV1::Policy),
        _ => None,
    }
}

fn wire_outcome(value: ContactUpsertOutcomeV1) -> WireContactUpsertOutcomeV1 {
    match value {
        ContactUpsertOutcomeV1::Created => WireContactUpsertOutcomeV1::ContactUpsertOutcomeCreated,
        ContactUpsertOutcomeV1::Updated => WireContactUpsertOutcomeV1::ContactUpsertOutcomeUpdated,
        ContactUpsertOutcomeV1::Unchanged => {
            WireContactUpsertOutcomeV1::ContactUpsertOutcomeUnchanged
        }
    }
}

fn wire_reject_code(value: ContactMailEntryRejectCodeV1) -> WireContactUpsertRejectCodeV1 {
    match value {
        ContactMailEntryRejectCodeV1::InvalidRequest => {
            WireContactUpsertRejectCodeV1::ContactUpsertRejectCodeInvalidRequest
        }
        ContactMailEntryRejectCodeV1::IdentityAmbiguous => {
            WireContactUpsertRejectCodeV1::ContactUpsertRejectCodeIdentityAmbiguous
        }
        ContactMailEntryRejectCodeV1::ProviderLinkConflict => {
            WireContactUpsertRejectCodeV1::ContactUpsertRejectCodeProviderLinkConflict
        }
        ContactMailEntryRejectCodeV1::StaleSource => {
            WireContactUpsertRejectCodeV1::ContactUpsertRejectCodeStaleSource
        }
        ContactMailEntryRejectCodeV1::Policy => {
            WireContactUpsertRejectCodeV1::ContactUpsertRejectCodePolicy
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

fn envelope_context(
    runtime: &ContactsCommandRuntimeContextV1<'_>,
) -> ContactsCommandEnvelopeContextV1 {
    ContactsCommandEnvelopeContextV1 {
        module_id: CONTACTS_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.to_owned(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
            .unwrap_or_default(),
    }
}

fn source_envelope_context(
    runtime: &ContactsCommandRuntimeContextV1<'_>,
) -> ContactsMailSyncSourceEnvelopeContextV1 {
    ContactsMailSyncSourceEnvelopeContextV1 {
        module_id: CONTACTS_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.to_owned(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
            .unwrap_or_default(),
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], ContactsCommandErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ContactsCommandErrorV1::InvalidPayload)
}

fn id32(value: &[u8]) -> Result<[u8; 32], ContactsCommandErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(ContactsCommandErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> ContactsCommandErrorV1 {
    ContactsCommandErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use makosh_contacts_command_api::{
        ContactsCommandEnvelopeContextV1, build_upsert_contact_command_outbox_record_v1,
    };
    use prost_types::Timestamp;

    use super::*;

    #[test]
    fn decoder_rejects_expired_command_without_cross_owner_mutation() {
        let record = build_upsert_contact_command_outbox_record_v1(
            UpsertContactFromMailAddressBookEntryCommandV1 {
                command_id: vec![1; 16],
                logical_owner_id: "owner-1".to_owned(),
                source_account_id: "mail-1".to_owned(),
                provider_kind: MailAddressBookProviderKindV1::MailAddressBookProviderKindGmail
                    as i32,
                provider_entry_id: "people/c1".to_owned(),
                provider_etag: None,
                display_name: "Ada".to_owned(),
                email_addresses: vec!["ada@example.test".to_owned()],
                phone_numbers: Vec::new(),
                observed_at: Some(Timestamp {
                    seconds: 1_800_000_000,
                    nanos: 0,
                }),
                source_revision: 1,
                entry_digest: vec![2; 32],
            },
            1_800_000_100,
            &ContactsCommandEnvelopeContextV1 {
                module_id: "mail-contacts-sync".to_owned(),
                runtime_instance_id: "runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("command");
        let context = ContactsCommandRuntimeContextV1 {
            logical_owner_id: "owner-1",
            runtime_instance_id: "contacts-runtime-1",
            runtime_generation: 1,
            now_unix_millis: 1_800_000_101_000,
        };
        assert!(matches!(
            decode_command(&record, &context),
            Ok(DecodedCommandV1::Reject(_))
        ));
        let other_owner = ContactsCommandRuntimeContextV1 {
            logical_owner_id: "owner-2",
            ..context
        };
        assert_eq!(
            decode_command(&record, &other_owner).err(),
            Some(ContactsCommandErrorV1::InvalidPayload)
        );
    }
}
