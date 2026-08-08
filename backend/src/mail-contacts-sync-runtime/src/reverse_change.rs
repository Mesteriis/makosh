use makosh_contacts_mail_sync_source_api::{
    CONTACT_MAIL_SYNC_SOURCE_REQUESTER_MODULE_ID_V1, ContactsMailSyncSourceEnvelopeContextV1,
    build_contact_mail_sync_source_prepare_outbox_record_v1,
    contact_changed_for_mail_sync_contract_reference_v1,
    wire::{ContactChangedForMailSyncV1, PrepareContactMailSyncSourceCommandV1},
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ContractRefV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_contacts_sync_core::MailContactsSyncDirectionV1;
use makosh_mail_contacts_sync_persistence::{
    AcceptContactChangedForMailSyncV1, MailContactsSyncPersistenceErrorV1,
    MailContactsSyncPersistenceV1, MailContactsSyncReverseOperationSeedV1, OutboxEnvelopeV1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;
use sha2::{Digest, Sha256};

use crate::MailContactsSyncRuntimeSettingsV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MailContactsSyncReverseChangeErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(MailContactsSyncPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) struct MailContactsSyncReverseChangeContextV1<'a> {
    pub logical_owner_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub(crate) async fn consume_contact_changed_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    configurations: &[(String, MailContactsSyncRuntimeSettingsV1)],
    context: &MailContactsSyncReverseChangeContextV1<'_>,
) -> Result<bool, MailContactsSyncReverseChangeErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailContactsSyncReverseChangeErrorV1::InvalidEnvelope)?;
    let changed = decode_changed(&record, context)?;
    let origin_run_id = origin_run_id(persistence, &record, context.logical_owner_id).await?;
    let operations = configurations
        .iter()
        .filter(|(_, settings)| {
            settings.direction == MailContactsSyncDirectionV1::Bidirectional
                && settings.remote_write_enabled
        })
        .map(|(configuration_instance_id, settings)| {
            operation(
                &record,
                &changed,
                configuration_instance_id,
                settings,
                context,
                origin_run_id,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    persistence
        .accept_contact_changed_for_mail_sync(&AcceptContactChangedForMailSyncV1 {
            logical_owner_id: context.logical_owner_id.to_owned(),
            event_message_id: *record.message_id(),
            event_envelope_sha256: *record.envelope_sha256(),
            operations,
            occurred_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map_err(MailContactsSyncReverseChangeErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

fn operation(
    record: &OutboxRecordV1,
    changed: &ContactChangedForMailSyncV1,
    configuration_instance_id: &str,
    settings: &MailContactsSyncRuntimeSettingsV1,
    context: &MailContactsSyncReverseChangeContextV1<'_>,
    origin_run_id: Option<[u8; 16]>,
) -> Result<MailContactsSyncReverseOperationSeedV1, MailContactsSyncReverseChangeErrorV1> {
    let contact_id = id16(&changed.contact_id)?;
    let operation_id = operation_id(*record.message_id(), configuration_instance_id);
    let command = build_contact_mail_sync_source_prepare_outbox_record_v1(
        PrepareContactMailSyncSourceCommandV1 {
            operation_id: operation_id.to_vec(),
            contact_id: contact_id.to_vec(),
            expected_contact_revision: changed.contact_revision,
            target_mail_account_id: settings.account_id.clone(),
            logical_owner_id: context.logical_owner_id.to_owned(),
        },
        context.now_unix_millis / 1_000 + crate::MAIL_CONTACTS_SYNC_COMMAND_DEADLINE_SECONDS_V1,
        &ContactsMailSyncSourceEnvelopeContextV1 {
            module_id: CONTACT_MAIL_SYNC_SOURCE_REQUESTER_MODULE_ID_V1.to_owned(),
            runtime_instance_id: context.runtime_instance_id.to_owned(),
            runtime_generation: context.runtime_generation,
            recorded_at_unix_seconds: context.now_unix_millis / 1_000,
            recorded_at_nanos: i32::try_from((context.now_unix_millis % 1_000) * 1_000_000)
                .unwrap_or_default(),
        },
    )
    .map_err(|_| MailContactsSyncReverseChangeErrorV1::InvalidPayload)?;
    Ok(MailContactsSyncReverseOperationSeedV1 {
        operation_id,
        configuration_instance_id: configuration_instance_id.to_owned(),
        account_id: settings.account_id.clone(),
        contact_id,
        contact_revision: changed.contact_revision,
        origin_run_id,
        source_prepare_command: outbox(&command),
    })
}

async fn origin_run_id(
    persistence: &MailContactsSyncPersistenceV1,
    record: &OutboxRecordV1,
    logical_owner_id: &str,
) -> Result<Option<[u8; 16]>, MailContactsSyncReverseChangeErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailContactsSyncReverseChangeErrorV1::InvalidEnvelope)?;
    if envelope.causation_message_id.is_empty() {
        return Ok(None);
    }
    let command_message_id = id16(&envelope.causation_message_id)?;
    match persistence
        .run_id_for_contact_command(logical_owner_id, &command_message_id)
        .await
    {
        Ok(run_id) => Ok(Some(run_id)),
        Err(MailContactsSyncPersistenceErrorV1::NotFound) => Ok(None),
        Err(error) => Err(MailContactsSyncReverseChangeErrorV1::Persistence(error)),
    }
}

fn decode_changed(
    record: &OutboxRecordV1,
    context: &MailContactsSyncReverseChangeContextV1<'_>,
) -> Result<ContactChangedForMailSyncV1, MailContactsSyncReverseChangeErrorV1> {
    if context.now_unix_millis <= 0 {
        return Err(MailContactsSyncReverseChangeErrorV1::InvalidPayload);
    }
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailContactsSyncReverseChangeErrorV1::InvalidEnvelope)?;
    validate_contract(
        envelope.contract.as_ref(),
        &contact_changed_for_mail_sync_contract_reference_v1(),
    )?;
    if !matches!(envelope.semantics, Some(Semantics::Event(_)))
        || envelope.source.as_ref().is_none_or(|source| {
            source.module_id != "makosh-contacts-runtime" || source.runtime_generation == 0
        })
    {
        return Err(MailContactsSyncReverseChangeErrorV1::InvalidEnvelope);
    }
    let payload = ContactChangedForMailSyncV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailContactsSyncReverseChangeErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != context.logical_owner_id
        || payload.contact_revision == 0
        || id16(&payload.contact_id).is_err()
    {
        return Err(MailContactsSyncReverseChangeErrorV1::InvalidPayload);
    }
    Ok(payload)
}

fn operation_id(event_message_id: [u8; 16], configuration_instance_id: &str) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"mail-contacts-sync-reverse-operation-v1");
    hash.update(event_message_id);
    hash.update(configuration_instance_id.as_bytes());
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn outbox(record: &OutboxRecordV1) -> OutboxEnvelopeV1 {
    OutboxEnvelopeV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    }
}

fn validate_contract(
    actual: Option<&ContractRefV1>,
    expected: &ContractReferenceV1,
) -> Result<(), MailContactsSyncReverseChangeErrorV1> {
    if actual.is_none_or(|actual| {
        actual.owner != expected.owner
            || actual.name != expected.name
            || actual.major != expected.major
            || actual.revision != expected.revision
            || actual.schema_sha256 != expected.schema_sha256
    }) {
        return Err(MailContactsSyncReverseChangeErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], MailContactsSyncReverseChangeErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(MailContactsSyncReverseChangeErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> MailContactsSyncReverseChangeErrorV1 {
    MailContactsSyncReverseChangeErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_identity_is_configuration_scoped() {
        let first = operation_id([1; 16], "configuration-1");
        assert_eq!(first, operation_id([1; 16], "configuration-1"));
        assert_ne!(first, operation_id([1; 16], "configuration-2"));
    }
}
