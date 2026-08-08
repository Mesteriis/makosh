use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobSessionRequestV1,
    request_managed_blob_session_v2,
};
use makosh_contacts_mail_sync_source_api::{
    CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
    CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_MODULE_ID_V1,
    CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_OWNER_ID_V1, CONTACT_MAIL_SYNC_SOURCE_MAX_BYTES_V1,
    CONTACT_MAIL_SYNC_SOURCE_MAX_PROOF_BYTES_V1, CONTACT_MAIL_SYNC_SOURCE_REQUESTER_MODULE_ID_V1,
    CONTACTS_MAIL_SYNC_SOURCE_CAPABILITY_ID_V1, ContactsMailSyncSourceEnvelopeContextV1,
    build_contact_mail_sync_source_prepared_outbox_record_v1,
    build_contact_mail_sync_source_rejected_outbox_record_v1,
    contact_mail_sync_source_prepare_contract_reference_v1,
    wire::{
        ContactMailSyncSourceContentReceiptV1, ContactMailSyncSourceContentV1,
        ContactMailSyncSourcePreparedV1, ContactMailSyncSourceRejectCodeV1,
        ContactMailSyncSourceRejectedV1, MailAddressBookLinkV1,
        PrepareContactMailSyncSourceCommandV1,
    },
};
use makosh_contacts_persistence::{
    ContactMailSyncSourceRejectCodeV1 as PersistenceRejectCodeV1, ContactMailSyncSourceSnapshotV1,
    ContactsOutboxRecordV1, ContactsPersistenceErrorV1, ContactsPersistenceV1,
    PersistContactMailSyncSourceResultV1, ReserveContactMailSyncSourceV1,
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
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{BlobDataOperationV1, ContractReferenceV1},
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::admission::CONTACTS_MAIL_SYNC_SOURCE_BLOB_WRITER_CAPABILITY_ID_V1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContactsSourceErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(ContactsPersistenceErrorV1),
    EventUnavailable,
    BlobUnavailable,
}

pub(crate) struct ContactsSourceRuntimeContextV1<'a> {
    pub logical_owner_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DecodedSourceCommandV1 {
    command_message_id: [u8; 16],
    command_envelope_sha256: [u8; 32],
    operation_id: [u8; 16],
    contact_id: [u8; 16],
    expected_contact_revision: u64,
    target_mail_account_id: String,
    logical_owner_id: String,
}

pub(crate) async fn consume_contact_mail_sync_source_once_v1(
    persistence: &ContactsPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    runtime: &ContactsSourceRuntimeContextV1<'_>,
) -> Result<bool, ContactsSourceErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ContactsSourceErrorV1::InvalidEnvelope)?;
    let command = decode_command(&record, runtime)?;
    if persistence
        .reserve_contact_mail_sync_source(&source_reservation(&command, runtime))
        .await
        .map_err(ContactsSourceErrorV1::Persistence)?
        .is_some()
    {
        delivery.acknowledge().await.map_err(event_error)?;
        return Ok(true);
    }
    let snapshot = match persistence
        .contact_mail_sync_source_snapshot(
            &command.logical_owner_id,
            command.contact_id,
            command.expected_contact_revision,
            &command.target_mail_account_id,
        )
        .await
    {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let code =
                snapshot_reject_code(error).ok_or(ContactsSourceErrorV1::Persistence(error))?;
            persist_rejection(persistence, &command, code, runtime).await?;
            delivery.acknowledge().await.map_err(event_error)?;
            return Ok(true);
        }
    };
    let source_bytes = encode_source(&snapshot)?;
    let receipt = match write_target_bound_source(
        control_channel,
        dispatcher,
        &command,
        &snapshot,
        source_bytes,
    ) {
        Ok(receipt) => receipt,
        Err(SourceMaterializationErrorV1::ContentLimit) => {
            persist_rejection(
                persistence,
                &command,
                PersistenceRejectCodeV1::ContentLimit,
                runtime,
            )
            .await?;
            delivery.acknowledge().await.map_err(event_error)?;
            return Ok(true);
        }
        Err(SourceMaterializationErrorV1::Policy) => {
            persist_rejection(
                persistence,
                &command,
                PersistenceRejectCodeV1::Policy,
                runtime,
            )
            .await?;
            delivery.acknowledge().await.map_err(event_error)?;
            return Ok(true);
        }
        Err(SourceMaterializationErrorV1::Unavailable) => {
            return Err(ContactsSourceErrorV1::BlobUnavailable);
        }
    };
    let terminal = build_contact_mail_sync_source_prepared_outbox_record_v1(
        command.command_message_id,
        ContactMailSyncSourcePreparedV1 {
            operation_id: command.operation_id.to_vec(),
            contact_id: snapshot.contact_id.to_vec(),
            contact_revision: snapshot.contact_revision,
            source_content: Some(receipt),
            logical_owner_id: command.logical_owner_id.clone(),
        },
        &envelope_context(runtime),
    )
    .map_err(|_| ContactsSourceErrorV1::InvalidPayload)?;
    persist_result(
        persistence,
        &command,
        None,
        outbox_record(&terminal),
        runtime,
    )
    .await?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

fn decode_command(
    record: &OutboxRecordV1,
    runtime: &ContactsSourceRuntimeContextV1<'_>,
) -> Result<DecodedSourceCommandV1, ContactsSourceErrorV1> {
    if runtime.now_unix_millis <= 0 {
        return Err(ContactsSourceErrorV1::InvalidPayload);
    }
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ContactsSourceErrorV1::InvalidEnvelope)?;
    validate_contract(
        envelope.contract.as_ref(),
        &contact_mail_sync_source_prepare_contract_reference_v1(),
    )?;
    if envelope.source.as_ref().is_none_or(|source| {
        source.module_id != CONTACT_MAIL_SYNC_SOURCE_REQUESTER_MODULE_ID_V1
            || source.runtime_generation == 0
    }) {
        return Err(ContactsSourceErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Command(CommandMetadataV1 {
        command_id,
        target_capability,
        deadline,
        ..
    })) = envelope.semantics
    else {
        return Err(ContactsSourceErrorV1::InvalidEnvelope);
    };
    let payload = PrepareContactMailSyncSourceCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| ContactsSourceErrorV1::InvalidPayload)?;
    let operation_id = id16(&payload.operation_id)?;
    if command_id.as_slice() != operation_id
        || record.message_id() != &operation_id
        || target_capability != CONTACTS_MAIL_SYNC_SOURCE_CAPABILITY_ID_V1
        || payload.logical_owner_id != runtime.logical_owner_id
        || !valid_owner(&payload.logical_owner_id)
        || payload.expected_contact_revision == 0
        || !valid_text(&payload.target_mail_account_id, 256)
        || deadline.is_none_or(|deadline| {
            deadline.seconds < runtime.now_unix_millis / 1_000
                || (deadline.seconds == runtime.now_unix_millis / 1_000
                    && i64::from(deadline.nanos) <= (runtime.now_unix_millis % 1_000) * 1_000_000)
        })
    {
        return Err(ContactsSourceErrorV1::InvalidPayload);
    }
    Ok(DecodedSourceCommandV1 {
        command_message_id: *record.message_id(),
        command_envelope_sha256: *record.envelope_sha256(),
        operation_id,
        contact_id: id16(&payload.contact_id)?,
        expected_contact_revision: payload.expected_contact_revision,
        target_mail_account_id: payload.target_mail_account_id,
        logical_owner_id: payload.logical_owner_id,
    })
}

fn encode_source(
    snapshot: &ContactMailSyncSourceSnapshotV1,
) -> Result<Vec<u8>, ContactsSourceErrorV1> {
    let bytes = ContactMailSyncSourceContentV1 {
        display_name: snapshot.display_name.clone(),
        email_addresses: snapshot.email_addresses.clone(),
        phone_numbers: snapshot.phone_numbers.clone(),
        target_account_link: snapshot.target_account_link.as_ref().map(|link| {
            MailAddressBookLinkV1 {
                provider_entry_id: link.provider_entry_id.clone(),
                provider_etag: link.provider_etag.clone(),
            }
        }),
    }
    .encode_to_vec();
    if bytes.is_empty()
        || u64::try_from(bytes.len()).unwrap_or(u64::MAX) > CONTACT_MAIL_SYNC_SOURCE_MAX_BYTES_V1
    {
        return Err(ContactsSourceErrorV1::InvalidPayload);
    }
    Ok(bytes)
}

fn write_target_bound_source(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    command: &DecodedSourceCommandV1,
    snapshot: &ContactMailSyncSourceSnapshotV1,
    bytes: Vec<u8>,
) -> Result<ContactMailSyncSourceContentReceiptV1, SourceMaterializationErrorV1> {
    let declared_bytes =
        u64::try_from(bytes.len()).map_err(|_| SourceMaterializationErrorV1::ContentLimit)?;
    if !(1..=CONTACT_MAIL_SYNC_SOURCE_MAX_BYTES_V1).contains(&declared_bytes) {
        return Err(SourceMaterializationErrorV1::ContentLimit);
    }
    let sha256: [u8; 32] = Sha256::digest(&bytes).into();
    let reference_id = source_reference_id(command.operation_id, snapshot, sha256);
    let session = request_managed_blob_session_v2(
        control_channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: CONTACTS_MAIL_SYNC_SOURCE_BLOB_WRITER_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_OWNER_ID_V1,
                module_id: CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_MODULE_ID_V1,
                capability_id: CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
            }),
        },
    )
    .map_err(|_| SourceMaterializationErrorV1::Unavailable)?;
    if session.custody_transfer_source_proof.is_empty()
        || session.custody_transfer_source_proof.len() > CONTACT_MAIL_SYNC_SOURCE_MAX_PROOF_BYTES_V1
    {
        return Err(SourceMaterializationErrorV1::Policy);
    }
    let proof = session.custody_transfer_source_proof;
    BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, bytes))
        .map_err(|_| SourceMaterializationErrorV1::Unavailable)?;
    Ok(ContactMailSyncSourceContentReceiptV1 {
        reference_id: reference_id.to_vec(),
        declared_bytes,
        sha256: sha256.to_vec(),
        custody_transfer_source_proof: proof,
    })
}

async fn persist_rejection(
    persistence: &ContactsPersistenceV1,
    command: &DecodedSourceCommandV1,
    code: PersistenceRejectCodeV1,
    runtime: &ContactsSourceRuntimeContextV1<'_>,
) -> Result<(), ContactsSourceErrorV1> {
    let terminal = build_contact_mail_sync_source_rejected_outbox_record_v1(
        command.command_message_id,
        ContactMailSyncSourceRejectedV1 {
            operation_id: command.operation_id.to_vec(),
            code: wire_reject_code(code) as i32,
            logical_owner_id: command.logical_owner_id.clone(),
        },
        &envelope_context(runtime),
    )
    .map_err(|_| ContactsSourceErrorV1::InvalidPayload)?;
    persist_result(
        persistence,
        command,
        Some(code),
        outbox_record(&terminal),
        runtime,
    )
    .await
}

async fn persist_result(
    persistence: &ContactsPersistenceV1,
    command: &DecodedSourceCommandV1,
    reject_code: Option<PersistenceRejectCodeV1>,
    terminal_result: ContactsOutboxRecordV1,
    runtime: &ContactsSourceRuntimeContextV1<'_>,
) -> Result<(), ContactsSourceErrorV1> {
    persistence
        .persist_contact_mail_sync_source_result(&PersistContactMailSyncSourceResultV1 {
            command_message_id: command.command_message_id,
            command_envelope_sha256: command.command_envelope_sha256,
            operation_id: command.operation_id,
            contact_id: command.contact_id,
            expected_contact_revision: command.expected_contact_revision,
            target_mail_account_id: command.target_mail_account_id.clone(),
            logical_owner_id: command.logical_owner_id.clone(),
            reject_code,
            terminal_result,
            received_at_unix_millis: runtime.now_unix_millis,
            completed_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map(|_| ())
        .map_err(ContactsSourceErrorV1::Persistence)
}

fn source_reference_id(
    operation_id: [u8; 16],
    snapshot: &ContactMailSyncSourceSnapshotV1,
    sha256: [u8; 32],
) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"contacts-mail-sync-source-copy-v1");
    hash.update(operation_id);
    hash.update(snapshot.contact_id);
    hash.update(snapshot.contact_revision.to_be_bytes());
    hash.update(sha256);
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn source_reservation(
    command: &DecodedSourceCommandV1,
    runtime: &ContactsSourceRuntimeContextV1<'_>,
) -> ReserveContactMailSyncSourceV1 {
    ReserveContactMailSyncSourceV1 {
        command_message_id: command.command_message_id,
        command_envelope_sha256: command.command_envelope_sha256,
        operation_id: command.operation_id,
        contact_id: command.contact_id,
        expected_contact_revision: command.expected_contact_revision,
        target_mail_account_id: command.target_mail_account_id.clone(),
        logical_owner_id: command.logical_owner_id.clone(),
        received_at_unix_millis: runtime.now_unix_millis,
    }
}

fn snapshot_reject_code(error: ContactsPersistenceErrorV1) -> Option<PersistenceRejectCodeV1> {
    match error {
        ContactsPersistenceErrorV1::InvalidInput => Some(PersistenceRejectCodeV1::InvalidRequest),
        ContactsPersistenceErrorV1::NotFound => Some(PersistenceRejectCodeV1::ContactMissing),
        ContactsPersistenceErrorV1::StaleSource => {
            Some(PersistenceRejectCodeV1::StaleContactRevision)
        }
        ContactsPersistenceErrorV1::PolicyRejected => Some(PersistenceRejectCodeV1::Policy),
        _ => None,
    }
}

fn wire_reject_code(value: PersistenceRejectCodeV1) -> ContactMailSyncSourceRejectCodeV1 {
    match value {
        PersistenceRejectCodeV1::InvalidRequest => {
            ContactMailSyncSourceRejectCodeV1::ContactMailSyncSourceRejectCodeInvalidRequest
        }
        PersistenceRejectCodeV1::ContactMissing => {
            ContactMailSyncSourceRejectCodeV1::ContactMailSyncSourceRejectCodeContactMissing
        }
        PersistenceRejectCodeV1::StaleContactRevision => {
            ContactMailSyncSourceRejectCodeV1::ContactMailSyncSourceRejectCodeStaleContactRevision
        }
        PersistenceRejectCodeV1::ContentLimit => {
            ContactMailSyncSourceRejectCodeV1::ContactMailSyncSourceRejectCodeContentLimit
        }
        PersistenceRejectCodeV1::Policy => {
            ContactMailSyncSourceRejectCodeV1::ContactMailSyncSourceRejectCodePolicy
        }
    }
}

fn validate_contract(
    actual: Option<&ContractRefV1>,
    expected: &ContractReferenceV1,
) -> Result<(), ContactsSourceErrorV1> {
    if actual.is_none_or(|actual| {
        actual.owner != expected.owner
            || actual.name != expected.name
            || actual.major != expected.major
            || actual.revision != expected.revision
            || actual.schema_sha256 != expected.schema_sha256
    }) {
        return Err(ContactsSourceErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn envelope_context(
    runtime: &ContactsSourceRuntimeContextV1<'_>,
) -> ContactsMailSyncSourceEnvelopeContextV1 {
    ContactsMailSyncSourceEnvelopeContextV1 {
        module_id: "makosh-contacts-runtime".to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.to_owned(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
            .unwrap_or_default(),
    }
}

fn outbox_record(record: &OutboxRecordV1) -> ContactsOutboxRecordV1 {
    ContactsOutboxRecordV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], ContactsSourceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ContactsSourceErrorV1::InvalidPayload)
}

fn valid_owner(value: &str) -> bool {
    valid_text(value, 128)
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn valid_text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> ContactsSourceErrorV1 {
    ContactsSourceErrorV1::EventUnavailable
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceMaterializationErrorV1 {
    ContentLimit,
    Policy,
    Unavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_reference_binds_operation_contact_revision_and_content() {
        let snapshot = ContactMailSyncSourceSnapshotV1 {
            contact_id: [2; 16],
            contact_revision: 3,
            display_name: "Ada".to_owned(),
            email_addresses: vec!["ada@example.test".to_owned()],
            phone_numbers: Vec::new(),
            target_account_link: None,
        };
        let first = source_reference_id([1; 16], &snapshot, [4; 32]);
        let mut changed = snapshot;
        changed.contact_revision += 1;
        assert_ne!(first, source_reference_id([1; 16], &changed, [4; 32]));
    }
}
