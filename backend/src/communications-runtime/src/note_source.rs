//! Event-only Communications source preparation for note candidate extraction.
//! Canonical subject and body are encoded into target-bound Blob custody;
//! sender, provider identity and plaintext never enter the durable event.

use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobSessionRequestV1,
    request_managed_blob_session_v2,
};
use makosh_communications_note_source_api::{
    COMMUNICATION_NOTE_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
    COMMUNICATION_NOTE_SOURCE_BLOB_TARGET_MODULE_ID_V1,
    COMMUNICATION_NOTE_SOURCE_BLOB_TARGET_OWNER_ID_V1, COMMUNICATION_NOTE_SOURCE_MAX_BYTES_V1,
    COMMUNICATION_NOTE_SOURCE_MAX_PROOF_BYTES_V1, COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID_V1,
    CommunicationNoteSourceEnvelopeContextV1,
    build_communication_note_source_prepared_outbox_record_v1,
    build_communication_note_source_rejected_outbox_record_v1,
    communication_note_source_prepare_contract_reference_v1,
    wire::{
        CommunicationNoteSourceContentReceiptV1, CommunicationNoteSourceContentV1,
        CommunicationNoteSourcePreparedV1, CommunicationNoteSourceRejectCodeV1,
        CommunicationNoteSourceRejectedV1, PrepareCommunicationNoteSourceCommandV1,
    },
};
use makosh_communications_persistence::{
    CommunicationsBodyReceiptV1, CommunicationsConsumeOutcomeV1, CommunicationsDurablePersistence,
    CommunicationsSourceErrorV1, CommunicationsSourceSnapshotV1,
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
        COMMUNICATIONS_BLOB_CAPABILITY_ID, COMMUNICATIONS_MODULE_ID,
        COMMUNICATIONS_NOTE_SOURCE_BLOB_CAPABILITY_ID,
    },
    canonical_outbox::CanonicalEventContextV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsNoteSourceDeliveryErrorV1 {
    Unavailable,
    InvalidEnvelope,
    InvalidPayload,
    Persistence,
}

pub async fn consume_next_note_source_prepare_v1(
    persistence: &CommunicationsDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    logical_human_owner_id: &str,
    context: &CanonicalEventContextV1,
) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsNoteSourceDeliveryErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(delivery_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationsNoteSourceDeliveryErrorV1::InvalidEnvelope)?;
    let decoded = decode_prepare_command(&record, logical_human_owner_id)?;
    let snapshot = match persistence
        .source_snapshot(decoded.source_message_id, decoded.expected_source_revision)
        .await
    {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let code = snapshot_rejection_code(error)?;
            let outcome = persist_rejection(persistence, &record, &decoded, code, context).await?;
            delivery.acknowledge().await.map_err(delivery_error)?;
            return Ok(outcome);
        }
    };
    let plaintext = match read_and_validate_body(control_channel, dispatcher, &snapshot.body) {
        Ok(plaintext) => plaintext,
        Err(SourceMaterializationErrorV1::Policy(code)) => {
            let outcome = persist_rejection(persistence, &record, &decoded, code, context).await?;
            delivery.acknowledge().await.map_err(delivery_error)?;
            return Ok(outcome);
        }
        Err(SourceMaterializationErrorV1::Unavailable) => {
            return Err(CommunicationsNoteSourceDeliveryErrorV1::Unavailable);
        }
    };
    let source_bytes = match encode_source_content(&snapshot, plaintext) {
        Ok(source_bytes) => source_bytes,
        Err(SourceMaterializationErrorV1::Policy(code)) => {
            let outcome = persist_rejection(persistence, &record, &decoded, code, context).await?;
            delivery.acknowledge().await.map_err(delivery_error)?;
            return Ok(outcome);
        }
        Err(SourceMaterializationErrorV1::Unavailable) => unreachable!("encoding is local"),
    };
    let source_content = match write_target_bound_source(
        control_channel,
        dispatcher,
        decoded.run_id,
        &snapshot,
        source_bytes,
    ) {
        Ok(source_content) => source_content,
        Err(SourceMaterializationErrorV1::Policy(code)) => {
            let outcome = persist_rejection(persistence, &record, &decoded, code, context).await?;
            delivery.acknowledge().await.map_err(delivery_error)?;
            return Ok(outcome);
        }
        Err(SourceMaterializationErrorV1::Unavailable) => {
            return Err(CommunicationsNoteSourceDeliveryErrorV1::Unavailable);
        }
    };
    let outbox = build_communication_note_source_prepared_outbox_record_v1(
        *record.message_id(),
        CommunicationNoteSourcePreparedV1 {
            run_id: decoded.run_id.to_vec(),
            source_message_id: snapshot.source_message_id.to_vec(),
            expected_source_revision: decoded.expected_source_revision,
            source_evidence_id: snapshot.evidence_id.to_vec(),
            source_evidence_revision: snapshot.evidence_revision,
            source_content: Some(source_content),
            logical_owner_id: decoded.logical_owner_id.clone(),
        },
        &note_source_envelope_context(context),
    )
    .map_err(|_| CommunicationsNoteSourceDeliveryErrorV1::InvalidPayload)?;
    let outcome = match persistence
        .persist_source_result(
            *record.message_id(),
            *record.envelope_sha256(),
            Some(&snapshot),
            &outbox,
            context.recorded_at_unix_seconds,
        )
        .await
    {
        Ok(outcome) => outcome,
        Err(CommunicationsSourceErrorV1::StaleRevision) => {
            persist_rejection(
                persistence,
                &record,
                &decoded,
                CommunicationNoteSourceRejectCodeV1::CommunicationNoteSourceRejectCodeStaleSourceRevision,
                context,
            )
            .await?
        }
        Err(error) => return Err(persistence_error(error)),
    };
    delivery.acknowledge().await.map_err(delivery_error)?;
    Ok(outcome)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DecodedPrepareCommandV1 {
    run_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
    logical_owner_id: String,
}

fn decode_prepare_command(
    record: &OutboxRecordV1,
    logical_human_owner_id: &str,
) -> Result<DecodedPrepareCommandV1, CommunicationsNoteSourceDeliveryErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationsNoteSourceDeliveryErrorV1::InvalidEnvelope)?;
    if !exact_contract(
        envelope.contract.as_ref(),
        &communication_note_source_prepare_contract_reference_v1(),
    ) || envelope.source.as_ref().is_none_or(|source| {
        source.module_id != COMMUNICATION_NOTE_SOURCE_BLOB_TARGET_MODULE_ID_V1
            || source.runtime_generation == 0
    }) {
        return Err(CommunicationsNoteSourceDeliveryErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Command(command)) = envelope.semantics else {
        return Err(CommunicationsNoteSourceDeliveryErrorV1::InvalidEnvelope);
    };
    if command.target_capability != COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID_V1 {
        return Err(CommunicationsNoteSourceDeliveryErrorV1::InvalidEnvelope);
    }
    let payload = PrepareCommunicationNoteSourceCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationsNoteSourceDeliveryErrorV1::InvalidPayload)?;
    let run_id = id16(&payload.run_id)?;
    if command.command_id.as_slice() != run_id
        || record.message_id() != &run_id
        || payload.expected_source_revision == 0
        || payload.logical_owner_id != logical_human_owner_id
        || !valid_logical_owner_id(&payload.logical_owner_id)
    {
        return Err(CommunicationsNoteSourceDeliveryErrorV1::InvalidPayload);
    }
    Ok(DecodedPrepareCommandV1 {
        run_id,
        source_message_id: id16(&payload.source_message_id)?,
        expected_source_revision: payload.expected_source_revision,
        logical_owner_id: payload.logical_owner_id,
    })
}

fn read_and_validate_body(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    receipt: &CommunicationsBodyReceiptV1,
) -> Result<Vec<u8>, SourceMaterializationErrorV1> {
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
    .map_err(|_| SourceMaterializationErrorV1::Unavailable)?;
    let bytes = BlobDataClient::new(session.data_socket_path)
        .and_then(|client| {
            client.read_range(
                session.grant,
                session.channel_binding,
                0,
                receipt.declared_bytes,
            )
        })
        .map_err(|_| SourceMaterializationErrorV1::Unavailable)?;
    if bytes.len() != usize::try_from(receipt.declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(&bytes).as_slice() != receipt.sha256
    {
        return Err(policy_error());
    }
    Ok(bytes)
}

fn encode_source_content(
    snapshot: &CommunicationsSourceSnapshotV1,
    body_utf8: Vec<u8>,
) -> Result<Vec<u8>, SourceMaterializationErrorV1> {
    if std::str::from_utf8(&snapshot.subject_utf8).is_err()
        || std::str::from_utf8(&body_utf8).is_err()
    {
        return Err(invalid_utf8_error());
    }
    let bytes = CommunicationNoteSourceContentV1 {
        subject_utf8: snapshot.subject_utf8.clone(),
        body_utf8,
    }
    .encode_to_vec();
    let declared_bytes = u64::try_from(bytes.len()).map_err(|_| content_limit_error())?;
    if !(1..=COMMUNICATION_NOTE_SOURCE_MAX_BYTES_V1).contains(&declared_bytes) {
        return Err(content_limit_error());
    }
    Ok(bytes)
}

fn write_target_bound_source(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run_id: [u8; 16],
    snapshot: &CommunicationsSourceSnapshotV1,
    bytes: Vec<u8>,
) -> Result<CommunicationNoteSourceContentReceiptV1, SourceMaterializationErrorV1> {
    let declared_bytes = u64::try_from(bytes.len()).map_err(|_| content_limit_error())?;
    if !(1..=COMMUNICATION_NOTE_SOURCE_MAX_BYTES_V1).contains(&declared_bytes) {
        return Err(content_limit_error());
    }
    let sha256: [u8; 32] = Sha256::digest(&bytes).into();
    let reference_id = source_reference_id(
        run_id,
        snapshot.source_message_id,
        snapshot.evidence_id,
        snapshot.evidence_revision,
        sha256,
    );
    let session = request_managed_blob_session_v2(
        control_channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: COMMUNICATIONS_NOTE_SOURCE_BLOB_CAPABILITY_ID,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: COMMUNICATION_NOTE_SOURCE_BLOB_TARGET_OWNER_ID_V1,
                module_id: COMMUNICATION_NOTE_SOURCE_BLOB_TARGET_MODULE_ID_V1,
                capability_id: COMMUNICATION_NOTE_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
            }),
        },
    )
    .map_err(|_| SourceMaterializationErrorV1::Unavailable)?;
    let source_proof = session.custody_transfer_source_proof;
    if source_proof.is_empty() || source_proof.len() > COMMUNICATION_NOTE_SOURCE_MAX_PROOF_BYTES_V1
    {
        return Err(policy_error());
    }
    BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, bytes))
        .map_err(|_| SourceMaterializationErrorV1::Unavailable)?;
    Ok(CommunicationNoteSourceContentReceiptV1 {
        reference_id: reference_id.to_vec(),
        declared_bytes,
        sha256: sha256.to_vec(),
        custody_transfer_source_proof: source_proof,
    })
}

fn source_reference_id(
    run_id: [u8; 16],
    source_message_id: [u8; 16],
    evidence_id: [u8; 16],
    evidence_revision: u64,
    sha256: [u8; 32],
) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communications-note-source-copy-v1");
    hasher.update(run_id);
    hasher.update(source_message_id);
    hasher.update(evidence_id);
    hasher.update(evidence_revision.to_be_bytes());
    hasher.update(sha256);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

async fn persist_rejection(
    persistence: &CommunicationsDurablePersistence,
    record: &OutboxRecordV1,
    decoded: &DecodedPrepareCommandV1,
    code: CommunicationNoteSourceRejectCodeV1,
    context: &CanonicalEventContextV1,
) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsNoteSourceDeliveryErrorV1> {
    let outbox = build_communication_note_source_rejected_outbox_record_v1(
        *record.message_id(),
        CommunicationNoteSourceRejectedV1 {
            run_id: decoded.run_id.to_vec(),
            code: code as i32,
            logical_owner_id: decoded.logical_owner_id.clone(),
        },
        &note_source_envelope_context(context),
    )
    .map_err(|_| CommunicationsNoteSourceDeliveryErrorV1::InvalidPayload)?;
    persistence
        .persist_source_result(
            *record.message_id(),
            *record.envelope_sha256(),
            None,
            &outbox,
            context.recorded_at_unix_seconds,
        )
        .await
        .map_err(persistence_error)
}

fn note_source_envelope_context(
    context: &CanonicalEventContextV1,
) -> CommunicationNoteSourceEnvelopeContextV1 {
    CommunicationNoteSourceEnvelopeContextV1 {
        module_id: COMMUNICATIONS_MODULE_ID.to_owned(),
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.recorded_at_unix_seconds,
        recorded_at_nanos: context.recorded_at_nanos,
    }
}

fn snapshot_rejection_code(
    error: CommunicationsSourceErrorV1,
) -> Result<CommunicationNoteSourceRejectCodeV1, CommunicationsNoteSourceDeliveryErrorV1> {
    match error {
        CommunicationsSourceErrorV1::InvalidRequest => Ok(
            CommunicationNoteSourceRejectCodeV1::CommunicationNoteSourceRejectCodeInvalidRequest,
        ),
        CommunicationsSourceErrorV1::SourceMissingOrInactive => Ok(
            CommunicationNoteSourceRejectCodeV1::CommunicationNoteSourceRejectCodeSourceMissingOrInactive,
        ),
        CommunicationsSourceErrorV1::ContentUnavailable => Ok(
            CommunicationNoteSourceRejectCodeV1::CommunicationNoteSourceRejectCodeContentUnavailable,
        ),
        CommunicationsSourceErrorV1::ContentLimit => Ok(
            CommunicationNoteSourceRejectCodeV1::CommunicationNoteSourceRejectCodeContentLimit,
        ),
        CommunicationsSourceErrorV1::StaleRevision => Ok(
            CommunicationNoteSourceRejectCodeV1::CommunicationNoteSourceRejectCodeStaleSourceRevision,
        ),
        CommunicationsSourceErrorV1::InvalidRow
        | CommunicationsSourceErrorV1::StorageUnavailable
        | CommunicationsSourceErrorV1::InboxHashConflict
        | CommunicationsSourceErrorV1::OutboxConflict => {
            Err(CommunicationsNoteSourceDeliveryErrorV1::Persistence)
        }
    }
}

fn policy_error() -> SourceMaterializationErrorV1 {
    SourceMaterializationErrorV1::Policy(
        CommunicationNoteSourceRejectCodeV1::CommunicationNoteSourceRejectCodePolicy,
    )
}

fn content_limit_error() -> SourceMaterializationErrorV1 {
    SourceMaterializationErrorV1::Policy(
        CommunicationNoteSourceRejectCodeV1::CommunicationNoteSourceRejectCodeContentLimit,
    )
}

fn invalid_utf8_error() -> SourceMaterializationErrorV1 {
    SourceMaterializationErrorV1::Policy(
        CommunicationNoteSourceRejectCodeV1::CommunicationNoteSourceRejectCodeInvalidUtf8,
    )
}

fn persistence_error(_: CommunicationsSourceErrorV1) -> CommunicationsNoteSourceDeliveryErrorV1 {
    CommunicationsNoteSourceDeliveryErrorV1::Persistence
}

fn delivery_error(_: RuntimePullDeliveryErrorV1) -> CommunicationsNoteSourceDeliveryErrorV1 {
    CommunicationsNoteSourceDeliveryErrorV1::Unavailable
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsNoteSourceDeliveryErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(CommunicationsNoteSourceDeliveryErrorV1::InvalidPayload)
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn exact_contract(
    value: Option<&ContractRefV1>,
    expected: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> bool {
    value.is_some_and(|value| {
        value.owner == expected.owner
            && value.name == expected.name
            && value.major == expected.major
            && value.revision == expected.revision
            && value.schema_sha256 == expected.schema_sha256
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceMaterializationErrorV1 {
    Policy(CommunicationNoteSourceRejectCodeV1),
    Unavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_is_bound_to_run_evidence_and_body() {
        let first = source_reference_id([7; 16], [1; 16], [2; 16], 3, [8; 32]);
        let second = source_reference_id([9; 16], [1; 16], [2; 16], 3, [8; 32]);
        assert_ne!(first, second);
        assert!(first.iter().any(|byte| *byte != 0));
    }

    #[test]
    fn persistence_failures_never_become_business_rejections() {
        assert_eq!(
            snapshot_rejection_code(CommunicationsSourceErrorV1::StorageUnavailable),
            Err(CommunicationsNoteSourceDeliveryErrorV1::Persistence),
        );
    }

    #[test]
    fn typed_source_contains_subject_and_body_without_sender() {
        let snapshot = CommunicationsSourceSnapshotV1 {
            source_message_id: [1; 16],
            evidence_id: [2; 16],
            evidence_revision: 3,
            sender_utf8: b"private@example.test".to_vec(),
            subject_utf8: b"Please prepare the report".to_vec(),
            body: CommunicationsBodyReceiptV1 {
                reference_id: [4; 16],
                declared_bytes: 4,
                sha256: [5; 32],
            },
        };
        let encoded = encode_source_content(&snapshot, b"body".to_vec()).expect("source");
        let decoded = CommunicationNoteSourceContentV1::decode(encoded.as_slice()).expect("typed");
        assert_eq!(decoded.subject_utf8, b"Please prepare the report");
        assert_eq!(decoded.body_utf8, b"body");
        assert!(
            !encoded
                .windows(snapshot.sender_utf8.len())
                .any(|window| window == snapshot.sender_utf8)
        );
    }
}
