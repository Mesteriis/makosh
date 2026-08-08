//! Durable Communications-owned source preparation for the evidence-export
//! workflow. Provider identity and body plaintext never enter the event spine.

use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobSessionRequestV1,
    request_managed_blob_session_v2,
};
use makosh_communications_api::CommunicationDirectionV1;
use makosh_communications_evidence_export_source_api::{
    COMMUNICATIONS_EXPORT_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
    COMMUNICATIONS_EXPORT_SOURCE_BLOB_TARGET_MODULE_ID_V1,
    COMMUNICATIONS_EXPORT_SOURCE_BLOB_TARGET_OWNER_ID_V1, EVIDENCE_EXPORT_MAX_SOURCE_BYTES_V1,
    EvidenceExportEnvelopeContextV1, build_evidence_export_prepared_outbox_record_v1,
    build_evidence_export_rejected_outbox_record_v1, evidence_export_prepare_contract_reference_v1,
    wire::{
        EvidenceExportBodySourceReceiptV1, EvidenceExportBodyStateV1, EvidenceExportDirectionV1,
        EvidenceExportPreparedV1, EvidenceExportRejectCodeV1, EvidenceExportRejectedV1,
        EvidenceExportSourceItemV1, PrepareEvidenceExportCommandV1,
    },
};
use makosh_communications_persistence::{
    CommunicationsConsumeOutcomeV1, CommunicationsDurablePersistence,
    CommunicationsEvidenceExportBodyReceiptV1, CommunicationsEvidenceExportSourceErrorV1,
    CommunicationsEvidenceExportSourceItemV1,
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
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::BlobDataOperationV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    admission::{
        COMMUNICATIONS_BLOB_CAPABILITY_ID, COMMUNICATIONS_EXPORT_SOURCE_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_MODULE_ID,
    },
    canonical_outbox::CanonicalEventContextV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsEvidenceExportDeliveryErrorV1 {
    Unavailable,
    InvalidEnvelope,
    InvalidPayload,
    Persistence,
}

pub async fn consume_next_evidence_export_prepare_v1(
    persistence: &CommunicationsDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    context: &CanonicalEventContextV1,
) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsEvidenceExportDeliveryErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(delivery_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationsEvidenceExportDeliveryErrorV1::InvalidEnvelope)?;
    let decoded = decode_prepare_command(&record)?;
    let snapshot = match persistence
        .evidence_export_source_snapshot(&decoded.message_ids)
        .await
    {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let code = snapshot_rejection_code(error)?;
            let outbox = rejected_outbox(
                record.message_id(),
                decoded.export_id,
                &decoded.logical_owner_id,
                code,
                context,
            )?;
            let outcome = persistence
                .persist_evidence_export_source_result(
                    *record.message_id(),
                    *record.envelope_sha256(),
                    None,
                    &outbox,
                    context.recorded_at_unix_seconds,
                )
                .await
                .map_err(persistence_error)?;
            delivery.acknowledge().await.map_err(delivery_error)?;
            return Ok(outcome);
        }
    };
    let plaintext = match read_and_validate_bodies(control_channel, dispatcher, &snapshot) {
        Ok(plaintext) => plaintext,
        Err(BodyMaterializationErrorV1::Policy) => {
            let outbox = rejected_outbox(
                record.message_id(),
                decoded.export_id,
                &decoded.logical_owner_id,
                EvidenceExportRejectCodeV1::EvidenceExportRejectCodePolicy,
                context,
            )?;
            let outcome = persistence
                .persist_evidence_export_source_result(
                    *record.message_id(),
                    *record.envelope_sha256(),
                    None,
                    &outbox,
                    context.recorded_at_unix_seconds,
                )
                .await
                .map_err(persistence_error)?;
            delivery.acknowledge().await.map_err(delivery_error)?;
            return Ok(outcome);
        }
        Err(BodyMaterializationErrorV1::Unavailable) => {
            return Err(CommunicationsEvidenceExportDeliveryErrorV1::Unavailable);
        }
    };
    let prepared = match write_target_bound_sources(
        control_channel,
        dispatcher,
        decoded.export_id,
        &snapshot,
        plaintext,
    ) {
        Ok(items) => EvidenceExportPreparedV1 {
            export_id: decoded.export_id.to_vec(),
            items,
            logical_owner_id: decoded.logical_owner_id.clone(),
        },
        Err(BodyMaterializationErrorV1::Policy) => {
            let outbox = rejected_outbox(
                record.message_id(),
                decoded.export_id,
                &decoded.logical_owner_id,
                EvidenceExportRejectCodeV1::EvidenceExportRejectCodePolicy,
                context,
            )?;
            let outcome = persistence
                .persist_evidence_export_source_result(
                    *record.message_id(),
                    *record.envelope_sha256(),
                    None,
                    &outbox,
                    context.recorded_at_unix_seconds,
                )
                .await
                .map_err(persistence_error)?;
            delivery.acknowledge().await.map_err(delivery_error)?;
            return Ok(outcome);
        }
        Err(BodyMaterializationErrorV1::Unavailable) => {
            return Err(CommunicationsEvidenceExportDeliveryErrorV1::Unavailable);
        }
    };
    let envelope_context = evidence_export_envelope_context(context);
    let outbox = build_evidence_export_prepared_outbox_record_v1(
        *record.message_id(),
        prepared,
        &envelope_context,
    )
    .map_err(|_| CommunicationsEvidenceExportDeliveryErrorV1::InvalidPayload)?;
    let outcome = match persistence
        .persist_evidence_export_source_result(
            *record.message_id(),
            *record.envelope_sha256(),
            Some(&snapshot),
            &outbox,
            context.recorded_at_unix_seconds,
        )
        .await
    {
        Ok(outcome) => outcome,
        Err(CommunicationsEvidenceExportSourceErrorV1::StaleRevision) => {
            let outbox = rejected_outbox(
                record.message_id(),
                decoded.export_id,
                &decoded.logical_owner_id,
                EvidenceExportRejectCodeV1::EvidenceExportRejectCodeStaleRevision,
                context,
            )?;
            persistence
                .persist_evidence_export_source_result(
                    *record.message_id(),
                    *record.envelope_sha256(),
                    None,
                    &outbox,
                    context.recorded_at_unix_seconds,
                )
                .await
                .map_err(persistence_error)?
        }
        Err(error) => return Err(persistence_error(error)),
    };
    delivery.acknowledge().await.map_err(delivery_error)?;
    Ok(outcome)
}

struct DecodedPrepareCommandV1 {
    export_id: [u8; 16],
    logical_owner_id: String,
    message_ids: Vec<[u8; 16]>,
}

fn decode_prepare_command(
    record: &OutboxRecordV1,
) -> Result<DecodedPrepareCommandV1, CommunicationsEvidenceExportDeliveryErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationsEvidenceExportDeliveryErrorV1::InvalidEnvelope)?;
    let expected = evidence_export_prepare_contract_reference_v1();
    if !exact_contract(envelope.contract.as_ref(), &expected) {
        return Err(CommunicationsEvidenceExportDeliveryErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Command(command)) = envelope.semantics.as_ref() else {
        return Err(CommunicationsEvidenceExportDeliveryErrorV1::InvalidEnvelope);
    };
    if command.target_capability != "communications.export-source.v1"
        || command.command_id != envelope.message_id
    {
        return Err(CommunicationsEvidenceExportDeliveryErrorV1::InvalidEnvelope);
    }
    let payload = PrepareEvidenceExportCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationsEvidenceExportDeliveryErrorV1::InvalidPayload)?;
    let export_id = id16(&payload.export_id)?;
    if !valid_logical_owner_id(&payload.logical_owner_id) {
        return Err(CommunicationsEvidenceExportDeliveryErrorV1::InvalidPayload);
    }
    if envelope.message_id.as_slice() != export_id {
        return Err(CommunicationsEvidenceExportDeliveryErrorV1::InvalidPayload);
    }
    if payload.message_ids.is_empty() || payload.message_ids.len() > 64 {
        return Err(CommunicationsEvidenceExportDeliveryErrorV1::InvalidPayload);
    }
    let message_ids = payload
        .message_ids
        .iter()
        .map(|value| id16(value))
        .collect::<Result<Vec<_>, _>>()?;
    if message_ids
        .iter()
        .enumerate()
        .any(|(index, value)| message_ids[..index].contains(value))
    {
        return Err(CommunicationsEvidenceExportDeliveryErrorV1::InvalidPayload);
    }
    Ok(DecodedPrepareCommandV1 {
        export_id,
        logical_owner_id: payload.logical_owner_id,
        message_ids,
    })
}

fn read_and_validate_bodies(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    snapshot: &[CommunicationsEvidenceExportSourceItemV1],
) -> Result<Vec<Option<Vec<u8>>>, BodyMaterializationErrorV1> {
    snapshot
        .iter()
        .map(|item| {
            item.body
                .as_ref()
                .map(|body| read_and_validate_body(control_channel, dispatcher, body))
                .transpose()
        })
        .collect()
}

fn read_and_validate_body(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    receipt: &CommunicationsEvidenceExportBodyReceiptV1,
) -> Result<Vec<u8>, BodyMaterializationErrorV1> {
    let session = request_managed_blob_session_v2(
        control_channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: COMMUNICATIONS_BLOB_CAPABILITY_ID,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id: &receipt.reference_id,
            declared_size: receipt.declared_bytes,
            backup_class: 1,
            receipt_sha256: None,
            custody_target: None,
        },
    )
    .map_err(|_| BodyMaterializationErrorV1::Unavailable)?;
    let bytes = BlobDataClient::new(session.data_socket_path)
        .and_then(|client| {
            client.read_range(
                session.grant,
                session.channel_binding,
                0,
                receipt.declared_bytes,
            )
        })
        .map_err(|_| BodyMaterializationErrorV1::Unavailable)?;
    if bytes.len() != usize::try_from(receipt.declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(&bytes).as_slice() != receipt.sha256
        || std::str::from_utf8(&bytes).is_err()
    {
        return Err(BodyMaterializationErrorV1::Policy);
    }
    Ok(bytes)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BodyMaterializationErrorV1 {
    Policy,
    Unavailable,
}

fn write_target_bound_sources(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    export_id: [u8; 16],
    snapshot: &[CommunicationsEvidenceExportSourceItemV1],
    plaintext: Vec<Option<Vec<u8>>>,
) -> Result<Vec<EvidenceExportSourceItemV1>, BodyMaterializationErrorV1> {
    if snapshot.len() != plaintext.len() {
        return Err(BodyMaterializationErrorV1::Policy);
    }
    snapshot
        .iter()
        .zip(plaintext)
        .map(|(item, plaintext)| {
            let body_source = plaintext
                .map(|bytes| {
                    write_target_bound_source(control_channel, dispatcher, export_id, item, bytes)
                })
                .transpose()?;
            Ok(EvidenceExportSourceItemV1 {
                message_id: item.message_id.to_vec(),
                conversation_id: item.conversation_id.to_vec(),
                evidence_id: item.evidence_id.to_vec(),
                evidence_revision: item.evidence_revision,
                direction: wire_direction(item.direction) as i32,
                occurred_at_unix_seconds: item.occurred_at_unix_seconds,
                observed_at_unix_seconds: item.observed_at_unix_seconds,
                participant_display_label: item.participant_display_label.clone(),
                body_state: if body_source.is_some() {
                    EvidenceExportBodyStateV1::EvidenceExportBodyStateAdmittedUtf8 as i32
                } else {
                    EvidenceExportBodyStateV1::EvidenceExportBodyStateUnavailable as i32
                },
                body_source,
            })
        })
        .collect()
}

fn write_target_bound_source(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    export_id: [u8; 16],
    item: &CommunicationsEvidenceExportSourceItemV1,
    bytes: Vec<u8>,
) -> Result<EvidenceExportBodySourceReceiptV1, BodyMaterializationErrorV1> {
    let declared_bytes =
        u64::try_from(bytes.len()).map_err(|_| BodyMaterializationErrorV1::Policy)?;
    if declared_bytes == 0 || declared_bytes > EVIDENCE_EXPORT_MAX_SOURCE_BYTES_V1 {
        return Err(BodyMaterializationErrorV1::Policy);
    }
    let sha256: [u8; 32] = Sha256::digest(&bytes).into();
    let reference_id = source_reference_id(export_id, item, sha256);
    let session = request_managed_blob_session_v2(
        control_channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: COMMUNICATIONS_EXPORT_SOURCE_BLOB_CAPABILITY_ID,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: COMMUNICATIONS_EXPORT_SOURCE_BLOB_TARGET_OWNER_ID_V1,
                module_id: COMMUNICATIONS_EXPORT_SOURCE_BLOB_TARGET_MODULE_ID_V1,
                capability_id: COMMUNICATIONS_EXPORT_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
            }),
        },
    )
    .map_err(|_| BodyMaterializationErrorV1::Unavailable)?;
    let source_proof = session.custody_transfer_source_proof;
    if source_proof.is_empty() || source_proof.len() > 2_048 {
        return Err(BodyMaterializationErrorV1::Policy);
    }
    BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, bytes))
        .map_err(|_| BodyMaterializationErrorV1::Unavailable)?;
    Ok(EvidenceExportBodySourceReceiptV1 {
        reference_id: reference_id.to_vec(),
        declared_bytes,
        sha256: sha256.to_vec(),
        custody_transfer_source_proof: source_proof,
    })
}

fn source_reference_id(
    export_id: [u8; 16],
    item: &CommunicationsEvidenceExportSourceItemV1,
    sha256: [u8; 32],
) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communications-evidence-export-source-copy-v1");
    hasher.update(export_id);
    hasher.update(item.message_id);
    hasher.update(item.evidence_id);
    hasher.update(item.evidence_revision.to_be_bytes());
    hasher.update(sha256);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

fn rejected_outbox(
    command_message_id: &[u8; 16],
    export_id: [u8; 16],
    logical_owner_id: &str,
    code: EvidenceExportRejectCodeV1,
    context: &CanonicalEventContextV1,
) -> Result<OutboxRecordV1, CommunicationsEvidenceExportDeliveryErrorV1> {
    build_evidence_export_rejected_outbox_record_v1(
        *command_message_id,
        EvidenceExportRejectedV1 {
            export_id: export_id.to_vec(),
            code: code as i32,
            logical_owner_id: logical_owner_id.to_owned(),
        },
        &evidence_export_envelope_context(context),
    )
    .map_err(|_| CommunicationsEvidenceExportDeliveryErrorV1::InvalidPayload)
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn evidence_export_envelope_context(
    context: &CanonicalEventContextV1,
) -> EvidenceExportEnvelopeContextV1 {
    EvidenceExportEnvelopeContextV1 {
        module_id: COMMUNICATIONS_MODULE_ID.to_owned(),
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.recorded_at_unix_seconds,
        recorded_at_nanos: context.recorded_at_nanos,
    }
}

fn snapshot_rejection_code(
    error: CommunicationsEvidenceExportSourceErrorV1,
) -> Result<EvidenceExportRejectCodeV1, CommunicationsEvidenceExportDeliveryErrorV1> {
    match error {
        CommunicationsEvidenceExportSourceErrorV1::InvalidRequest => {
            Ok(EvidenceExportRejectCodeV1::EvidenceExportRejectCodeInvalidRequest)
        }
        CommunicationsEvidenceExportSourceErrorV1::NotFound => {
            Ok(EvidenceExportRejectCodeV1::EvidenceExportRejectCodeNotFound)
        }
        CommunicationsEvidenceExportSourceErrorV1::StaleRevision => {
            Ok(EvidenceExportRejectCodeV1::EvidenceExportRejectCodeStaleRevision)
        }
        CommunicationsEvidenceExportSourceErrorV1::ContentLimit => {
            Ok(EvidenceExportRejectCodeV1::EvidenceExportRejectCodeContentLimit)
        }
        CommunicationsEvidenceExportSourceErrorV1::InvalidRow
        | CommunicationsEvidenceExportSourceErrorV1::StorageUnavailable => {
            Err(CommunicationsEvidenceExportDeliveryErrorV1::Unavailable)
        }
        CommunicationsEvidenceExportSourceErrorV1::InboxHashConflict
        | CommunicationsEvidenceExportSourceErrorV1::OutboxConflict => {
            Err(CommunicationsEvidenceExportDeliveryErrorV1::Persistence)
        }
    }
}

fn persistence_error(
    error: CommunicationsEvidenceExportSourceErrorV1,
) -> CommunicationsEvidenceExportDeliveryErrorV1 {
    match error {
        CommunicationsEvidenceExportSourceErrorV1::StorageUnavailable => {
            CommunicationsEvidenceExportDeliveryErrorV1::Unavailable
        }
        _ => CommunicationsEvidenceExportDeliveryErrorV1::Persistence,
    }
}

fn delivery_error(_: RuntimePullDeliveryErrorV1) -> CommunicationsEvidenceExportDeliveryErrorV1 {
    CommunicationsEvidenceExportDeliveryErrorV1::Unavailable
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsEvidenceExportDeliveryErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(CommunicationsEvidenceExportDeliveryErrorV1::InvalidPayload)
}

fn exact_contract(
    contract: Option<&ContractRefV1>,
    expected: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> bool {
    contract.is_some_and(|contract| {
        contract.owner == expected.owner
            && contract.name == expected.name
            && contract.major == expected.major
            && contract.revision == expected.revision
            && contract.schema_sha256 == expected.schema_sha256
    })
}

const fn wire_direction(value: CommunicationDirectionV1) -> EvidenceExportDirectionV1 {
    match value {
        CommunicationDirectionV1::Incoming => {
            EvidenceExportDirectionV1::EvidenceExportDirectionIncoming
        }
        CommunicationDirectionV1::Outgoing => {
            EvidenceExportDirectionV1::EvidenceExportDirectionOutgoing
        }
        CommunicationDirectionV1::Unknown => {
            EvidenceExportDirectionV1::EvidenceExportDirectionUnknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item() -> CommunicationsEvidenceExportSourceItemV1 {
        CommunicationsEvidenceExportSourceItemV1 {
            message_id: [2; 16],
            conversation_id: [3; 16],
            evidence_id: [4; 16],
            evidence_revision: 7,
            direction: CommunicationDirectionV1::Incoming,
            occurred_at_unix_seconds: 1_700_000_000,
            observed_at_unix_seconds: 1_700_000_001,
            participant_display_label: None,
            body: None,
        }
    }

    #[test]
    fn source_reference_is_deterministic_and_snapshot_bound() {
        let first = source_reference_id([1; 16], &item(), [5; 32]);
        let second = source_reference_id([1; 16], &item(), [5; 32]);
        assert_eq!(first, second);
        assert_ne!(first, source_reference_id([9; 16], &item(), [5; 32]));
        assert!(first.iter().any(|byte| *byte != 0));
    }

    #[test]
    fn snapshot_policy_failures_map_to_typed_terminal_rejections() {
        assert_eq!(
            snapshot_rejection_code(CommunicationsEvidenceExportSourceErrorV1::ContentLimit),
            Ok(EvidenceExportRejectCodeV1::EvidenceExportRejectCodeContentLimit)
        );
        assert_eq!(
            snapshot_rejection_code(CommunicationsEvidenceExportSourceErrorV1::NotFound),
            Ok(EvidenceExportRejectCodeV1::EvidenceExportRejectCodeNotFound)
        );
        assert_eq!(
            snapshot_rejection_code(CommunicationsEvidenceExportSourceErrorV1::StorageUnavailable),
            Err(CommunicationsEvidenceExportDeliveryErrorV1::Unavailable)
        );
    }
}
