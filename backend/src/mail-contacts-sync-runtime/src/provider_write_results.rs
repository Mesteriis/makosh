//! Workflow-owned reconciliation of terminal Mail address-book write results.

use makosh_contacts_command_api::{
    ContactsCommandEnvelopeContextV1,
    build_bind_mail_address_book_provider_link_command_outbox_record_v1,
    wire::{
        BindMailAddressBookProviderLinkCommandV1,
        MailAddressBookProviderKindV1 as ContactsMailAddressBookProviderKindV1,
    },
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ResultMetadataV1, ResultOutcomeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_address_book_contract::{
    MAIL_RUNTIME_MODULE_ID_V1, MailAddressBookContractV1,
    validate_mail_address_book_entry_upsert_rejected_v1,
    validate_mail_address_book_entry_upserted_v1,
    wire::{
        MailAddressBookEntryUpsertRejectedV1, MailAddressBookEntryUpsertedV1,
        MailAddressBookProviderKindV1, MailAddressBookRejectCodeV1,
    },
};
use makosh_mail_contacts_sync_core::MailContactsSyncRejectCodeV1;
use makosh_mail_contacts_sync_persistence::{
    CompleteMailAddressBookUpsertV1, MailContactsSyncPersistenceErrorV1,
    MailContactsSyncPersistenceV1, MailContactsSyncProviderWriteOutcomeV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MailContactsSyncProviderWriteResultErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(MailContactsSyncPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) struct MailContactsSyncProviderWriteResultContextV1<'a> {
    pub logical_owner_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub(crate) async fn consume_mail_entry_upserted_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &MailContactsSyncProviderWriteResultContextV1<'_>,
) -> Result<bool, MailContactsSyncProviderWriteResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailContactsSyncProviderWriteResultErrorV1::InvalidEnvelope)?;
    let envelope = decode_result(&record, MailAddressBookContractV1::EntryUpserted, context)?;
    let payload = MailAddressBookEntryUpsertedV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailContactsSyncProviderWriteResultErrorV1::InvalidPayload)?;
    validate_mail_address_book_entry_upserted_v1(&payload)
        .map_err(|_| MailContactsSyncProviderWriteResultErrorV1::InvalidPayload)?;
    let identity = result_identity(&record, &envelope, &payload.command_id, &payload.run_id)?;
    let operation = persistence
        .load_reverse_operation(context.logical_owner_id, identity.operation_id)
        .await
        .map_err(MailContactsSyncProviderWriteResultErrorV1::Persistence)?;
    if operation.state != 2
        || operation.mail_command_message_id != Some(identity.command_message_id)
        || operation.contact_revision != payload.applied_contact_revision
    {
        return Err(MailContactsSyncProviderWriteResultErrorV1::InvalidPayload);
    }
    complete(
        persistence,
        &record,
        identity,
        MailContactsSyncProviderWriteOutcomeV1::Succeeded,
        Some(build_contacts_link_command(
            &record, &operation, &payload, context,
        )?),
        context,
    )
    .await?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

pub(crate) async fn consume_mail_entry_upsert_rejected_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &MailContactsSyncProviderWriteResultContextV1<'_>,
) -> Result<bool, MailContactsSyncProviderWriteResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailContactsSyncProviderWriteResultErrorV1::InvalidEnvelope)?;
    let envelope = decode_result(
        &record,
        MailAddressBookContractV1::EntryUpsertRejected,
        context,
    )?;
    let payload = MailAddressBookEntryUpsertRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailContactsSyncProviderWriteResultErrorV1::InvalidPayload)?;
    validate_mail_address_book_entry_upsert_rejected_v1(&payload)
        .map_err(|_| MailContactsSyncProviderWriteResultErrorV1::InvalidPayload)?;
    let identity = result_identity(&record, &envelope, &payload.command_id, &payload.run_id)?;
    let expected_outcome = if payload.outcome_unknown {
        ResultOutcomeV1::Failed
    } else {
        ResultOutcomeV1::Rejected
    };
    if result_outcome(&envelope) != Some(expected_outcome) {
        return Err(MailContactsSyncProviderWriteResultErrorV1::InvalidEnvelope);
    }
    let operation = persistence
        .load_reverse_operation(context.logical_owner_id, identity.operation_id)
        .await
        .map_err(MailContactsSyncProviderWriteResultErrorV1::Persistence)?;
    if operation.state != 2
        || operation.mail_command_message_id != Some(identity.command_message_id)
    {
        return Err(MailContactsSyncProviderWriteResultErrorV1::InvalidPayload);
    }
    let outcome = if payload.outcome_unknown {
        MailContactsSyncProviderWriteOutcomeV1::OutcomeUnknown
    } else {
        MailContactsSyncProviderWriteOutcomeV1::Rejected(map_reject_code(payload.code)?)
    };
    complete(persistence, &record, identity, outcome, None, context).await?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

#[derive(Clone, Copy)]
struct ProviderWriteResultIdentityV1 {
    operation_id: [u8; 16],
    command_message_id: [u8; 16],
}

async fn complete(
    persistence: &MailContactsSyncPersistenceV1,
    record: &OutboxRecordV1,
    identity: ProviderWriteResultIdentityV1,
    outcome: MailContactsSyncProviderWriteOutcomeV1,
    contacts_link_command: Option<makosh_mail_contacts_sync_persistence::OutboxEnvelopeV1>,
    context: &MailContactsSyncProviderWriteResultContextV1<'_>,
) -> Result<(), MailContactsSyncProviderWriteResultErrorV1> {
    persistence
        .complete_mail_address_book_upsert(&CompleteMailAddressBookUpsertV1 {
            logical_owner_id: context.logical_owner_id.to_owned(),
            result_message_id: *record.message_id(),
            result_envelope_sha256: *record.envelope_sha256(),
            operation_id: identity.operation_id,
            mail_command_message_id: identity.command_message_id,
            outcome,
            contacts_link_command,
            occurred_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map(|_| ())
        .map_err(MailContactsSyncProviderWriteResultErrorV1::Persistence)
}

fn build_contacts_link_command(
    record: &OutboxRecordV1,
    operation: &makosh_mail_contacts_sync_persistence::MailContactsSyncReverseOperationV1,
    payload: &MailAddressBookEntryUpsertedV1,
    context: &MailContactsSyncProviderWriteResultContextV1<'_>,
) -> Result<
    makosh_mail_contacts_sync_persistence::OutboxEnvelopeV1,
    MailContactsSyncProviderWriteResultErrorV1,
> {
    let provider_kind = match MailAddressBookProviderKindV1::try_from(payload.provider_kind) {
        Ok(MailAddressBookProviderKindV1::MailAddressBookProviderKindGooglePeople) => {
            ContactsMailAddressBookProviderKindV1::MailAddressBookProviderKindGmail
        }
        Ok(MailAddressBookProviderKindV1::MailAddressBookProviderKindIcloudCarddav) => {
            ContactsMailAddressBookProviderKindV1::MailAddressBookProviderKindIcloud
        }
        _ => return Err(MailContactsSyncProviderWriteResultErrorV1::InvalidPayload),
    };
    let command_id = link_command_id(operation.operation_id, *record.message_id());
    let command = build_bind_mail_address_book_provider_link_command_outbox_record_v1(
        *record.message_id(),
        BindMailAddressBookProviderLinkCommandV1 {
            command_id: command_id.to_vec(),
            logical_owner_id: context.logical_owner_id.to_owned(),
            contact_id: operation.contact_id.to_vec(),
            expected_contact_revision: operation.contact_revision,
            source_account_id: operation.account_id.clone(),
            provider_kind: provider_kind as i32,
            provider_entry_id: payload.provider_entry_id.clone(),
            provider_etag: Some(payload.provider_etag.clone()),
        },
        context.now_unix_millis / 1_000 + crate::MAIL_CONTACTS_SYNC_COMMAND_DEADLINE_SECONDS_V1,
        &ContactsCommandEnvelopeContextV1 {
            module_id: "makosh-mail-contacts-sync-runtime".to_owned(),
            runtime_instance_id: context.runtime_instance_id.to_owned(),
            runtime_generation: context.runtime_generation,
            recorded_at_unix_seconds: context.now_unix_millis / 1_000,
            recorded_at_nanos: i32::try_from((context.now_unix_millis % 1_000) * 1_000_000)
                .unwrap_or_default(),
        },
    )
    .map_err(|_| MailContactsSyncProviderWriteResultErrorV1::InvalidPayload)?;
    Ok(makosh_mail_contacts_sync_persistence::OutboxEnvelopeV1 {
        message_id: *command.message_id(),
        envelope_sha256: *command.envelope_sha256(),
        envelope_bytes: command.exact_bytes().to_vec(),
    })
}

fn link_command_id(operation_id: [u8; 16], result_message_id: [u8; 16]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"mail-contacts-sync-provider-link-command-v1");
    hash.update(operation_id);
    hash.update(result_message_id);
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn map_reject_code(
    code: i32,
) -> Result<MailContactsSyncRejectCodeV1, MailContactsSyncProviderWriteResultErrorV1> {
    let code = MailAddressBookRejectCodeV1::try_from(code)
        .map_err(|_| MailContactsSyncProviderWriteResultErrorV1::InvalidPayload)?;
    match code {
        MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest => {
            Ok(MailContactsSyncRejectCodeV1::InvalidRequest)
        }
        MailAddressBookRejectCodeV1::MailAddressBookRejectCodeAccountUnavailable
        | MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable => {
            Ok(MailContactsSyncRejectCodeV1::AccountUnavailable)
        }
        MailAddressBookRejectCodeV1::MailAddressBookRejectCodeWriteScopeRequired
        | MailAddressBookRejectCodeV1::MailAddressBookRejectCodeReadOnlyProvider => {
            Ok(MailContactsSyncRejectCodeV1::RemoteWriteBlocked)
        }
        MailAddressBookRejectCodeV1::MailAddressBookRejectCodeEtagConflict => {
            Ok(MailContactsSyncRejectCodeV1::EtagConflict)
        }
        MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable => {
            Ok(MailContactsSyncRejectCodeV1::ProviderUnavailable)
        }
        MailAddressBookRejectCodeV1::MailAddressBookRejectCodePolicy => {
            Ok(MailContactsSyncRejectCodeV1::Policy)
        }
        MailAddressBookRejectCodeV1::MailAddressBookRejectCodeOutcomeUnknown
        | MailAddressBookRejectCodeV1::MailAddressBookRejectCodeUnspecified => {
            Err(MailContactsSyncProviderWriteResultErrorV1::InvalidPayload)
        }
    }
}

fn decode_result(
    record: &OutboxRecordV1,
    contract: MailAddressBookContractV1,
    context: &MailContactsSyncProviderWriteResultContextV1<'_>,
) -> Result<makosh_events_protocol::v1::DurableEnvelopeV1, MailContactsSyncProviderWriteResultErrorV1>
{
    if context.now_unix_millis <= 0 {
        return Err(MailContactsSyncProviderWriteResultErrorV1::InvalidPayload);
    }
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailContactsSyncProviderWriteResultErrorV1::InvalidEnvelope)?;
    let expected = contract.reference();
    if envelope.contract.as_ref().is_none_or(|actual| {
        actual.owner != expected.owner
            || actual.name != expected.name
            || actual.major != expected.major
            || actual.revision != expected.revision
            || actual.schema_sha256 != expected.schema_sha256
    }) || envelope.source.as_ref().is_none_or(|source| {
        source.module_id != MAIL_RUNTIME_MODULE_ID_V1 || source.runtime_generation == 0
    }) || !matches!(envelope.semantics, Some(Semantics::Result(_)))
    {
        return Err(MailContactsSyncProviderWriteResultErrorV1::InvalidEnvelope);
    }
    Ok(envelope)
}

fn result_identity(
    record: &OutboxRecordV1,
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
    payload_command_id: &[u8],
    payload_run_id: &[u8],
) -> Result<ProviderWriteResultIdentityV1, MailContactsSyncProviderWriteResultErrorV1> {
    let command_message_id = id16(payload_command_id)?;
    let operation_id = id16(payload_run_id)?;
    let Some(Semantics::Result(result)) = envelope.semantics.as_ref() else {
        return Err(MailContactsSyncProviderWriteResultErrorV1::InvalidEnvelope);
    };
    if result.command_id.as_slice() != command_message_id
        || result.command_message_id.as_slice() != command_message_id
        || envelope.partition_key.as_slice() != operation_id
        || envelope.correlation_id.as_slice() != operation_id
        || envelope.causation_message_id.as_slice() != command_message_id
        || record.message_id() == &command_message_id
    {
        return Err(MailContactsSyncProviderWriteResultErrorV1::InvalidEnvelope);
    }
    Ok(ProviderWriteResultIdentityV1 {
        operation_id,
        command_message_id,
    })
}

fn result_outcome(
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
) -> Option<ResultOutcomeV1> {
    let Some(Semantics::Result(ResultMetadataV1 { outcome, .. })) = envelope.semantics.as_ref()
    else {
        return None;
    };
    ResultOutcomeV1::try_from(*outcome).ok()
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailContactsSyncProviderWriteResultErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(MailContactsSyncProviderWriteResultErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> MailContactsSyncProviderWriteResultErrorV1 {
    MailContactsSyncProviderWriteResultErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_rejections_map_to_bounded_workflow_reasons() {
        for (provider, workflow) in [
            (
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest,
                MailContactsSyncRejectCodeV1::InvalidRequest,
            ),
            (
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodeAccountUnavailable,
                MailContactsSyncRejectCodeV1::AccountUnavailable,
            ),
            (
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable,
                MailContactsSyncRejectCodeV1::AccountUnavailable,
            ),
            (
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodeWriteScopeRequired,
                MailContactsSyncRejectCodeV1::RemoteWriteBlocked,
            ),
            (
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodeReadOnlyProvider,
                MailContactsSyncRejectCodeV1::RemoteWriteBlocked,
            ),
            (
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodeEtagConflict,
                MailContactsSyncRejectCodeV1::EtagConflict,
            ),
            (
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable,
                MailContactsSyncRejectCodeV1::ProviderUnavailable,
            ),
            (
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodePolicy,
                MailContactsSyncRejectCodeV1::Policy,
            ),
        ] {
            assert_eq!(map_reject_code(provider as i32), Ok(workflow));
        }
    }

    #[test]
    fn outcome_unknown_cannot_be_smuggled_as_a_terminal_rejection() {
        for code in [
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeUnspecified,
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeOutcomeUnknown,
        ] {
            assert_eq!(
                map_reject_code(code as i32),
                Err(MailContactsSyncProviderWriteResultErrorV1::InvalidPayload)
            );
        }
        assert_eq!(
            map_reject_code(i32::MAX),
            Err(MailContactsSyncProviderWriteResultErrorV1::InvalidPayload)
        );
    }
}
