//! Event-only Communications source preparation for cross-channel forwarding.
//! Provider identity and plaintext never enter the durable event spine.

use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobSessionRequestV1,
    request_managed_blob_session_v2,
};
use makosh_communications_cross_channel_forward_source_api::{
    CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
    CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_MODULE_ID_V1,
    CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_OWNER_ID_V1,
    CROSS_CHANNEL_FORWARD_SOURCE_COMMAND_CAPABILITY_ID_V1,
    CROSS_CHANNEL_FORWARD_SOURCE_MAX_BYTES_V1, CrossChannelForwardSourceEnvelopeContextV1,
    build_cross_channel_forward_source_prepared_outbox_record_v1,
    build_cross_channel_forward_source_rejected_outbox_record_v1,
    cross_channel_forward_source_prepare_contract_reference_v1,
    wire::{
        CrossChannelForwardBodySourceReceiptV1, CrossChannelForwardSourcePreparedV1,
        CrossChannelForwardSourceRejectCodeV1, CrossChannelForwardSourceRejectedV1,
        PrepareCrossChannelForwardSourceCommandV1,
    },
};
use makosh_communications_persistence::{
    CommunicationsConsumeOutcomeV1, CommunicationsCrossChannelForwardBodyReceiptV1,
    CommunicationsCrossChannelForwardSourceErrorV1,
    CommunicationsCrossChannelForwardSourceSnapshotV1, CommunicationsDurablePersistence,
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
        COMMUNICATIONS_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_BLOB_CAPABILITY_ID, COMMUNICATIONS_MODULE_ID,
    },
    canonical_outbox::CanonicalEventContextV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsCrossChannelForwardSourceDeliveryErrorV1 {
    Unavailable,
    InvalidEnvelope,
    InvalidPayload,
    Persistence,
}

pub async fn consume_next_cross_channel_forward_source_prepare_v1(
    persistence: &CommunicationsDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    context: &CanonicalEventContextV1,
) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsCrossChannelForwardSourceDeliveryErrorV1>
{
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(delivery_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationsCrossChannelForwardSourceDeliveryErrorV1::InvalidEnvelope)?;
    let decoded = decode_prepare_command(&record)?;
    let snapshot = match persistence
        .cross_channel_forward_source_snapshot(
            decoded.source_message_id,
            decoded.target_conversation_id,
        )
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
        Err(BodyMaterializationErrorV1::Policy) => {
            let outcome = persist_rejection(
                persistence,
                &record,
                &decoded,
                CrossChannelForwardSourceRejectCodeV1::CrossChannelForwardSourceRejectCodePolicy,
                context,
            )
            .await?;
            delivery.acknowledge().await.map_err(delivery_error)?;
            return Ok(outcome);
        }
        Err(BodyMaterializationErrorV1::Unavailable) => {
            return Err(CommunicationsCrossChannelForwardSourceDeliveryErrorV1::Unavailable);
        }
    };
    let body_source = match write_target_bound_source(
        control_channel,
        dispatcher,
        decoded.forward_id,
        &snapshot,
        plaintext,
    ) {
        Ok(body_source) => body_source,
        Err(BodyMaterializationErrorV1::Policy) => {
            let outcome = persist_rejection(
                persistence,
                &record,
                &decoded,
                CrossChannelForwardSourceRejectCodeV1::CrossChannelForwardSourceRejectCodePolicy,
                context,
            )
            .await?;
            delivery.acknowledge().await.map_err(delivery_error)?;
            return Ok(outcome);
        }
        Err(BodyMaterializationErrorV1::Unavailable) => {
            return Err(CommunicationsCrossChannelForwardSourceDeliveryErrorV1::Unavailable);
        }
    };
    let prepared = CrossChannelForwardSourcePreparedV1 {
        forward_id: decoded.forward_id.to_vec(),
        source_message_id: snapshot.source_message_id.to_vec(),
        target_conversation_id: snapshot.target_conversation_id.to_vec(),
        source_evidence_id: snapshot.evidence_id.to_vec(),
        source_evidence_revision: snapshot.evidence_revision,
        body_source: Some(body_source),
        logical_owner_id: decoded.logical_owner_id.clone(),
    };
    let outbox = build_cross_channel_forward_source_prepared_outbox_record_v1(
        *record.message_id(),
        prepared,
        &forward_source_envelope_context(context),
    )
    .map_err(|_| CommunicationsCrossChannelForwardSourceDeliveryErrorV1::InvalidPayload)?;
    let outcome = match persistence
        .persist_cross_channel_forward_source_result(
            *record.message_id(),
            *record.envelope_sha256(),
            Some(&snapshot),
            &outbox,
            context.recorded_at_unix_seconds,
        )
        .await
    {
        Ok(outcome) => outcome,
        Err(CommunicationsCrossChannelForwardSourceErrorV1::StaleRevision) => {
            persist_rejection(
                persistence,
                &record,
                &decoded,
                CrossChannelForwardSourceRejectCodeV1::CrossChannelForwardSourceRejectCodePolicy,
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
    forward_id: [u8; 16],
    source_message_id: [u8; 16],
    target_conversation_id: [u8; 16],
    logical_owner_id: String,
}

fn decode_prepare_command(
    record: &OutboxRecordV1,
) -> Result<DecodedPrepareCommandV1, CommunicationsCrossChannelForwardSourceDeliveryErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationsCrossChannelForwardSourceDeliveryErrorV1::InvalidEnvelope)?;
    if !exact_contract(
        envelope.contract.as_ref(),
        &cross_channel_forward_source_prepare_contract_reference_v1(),
    ) || envelope.source.as_ref().is_none_or(|source| {
        source.module_id != CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_MODULE_ID_V1
            || source.runtime_generation == 0
    }) {
        return Err(CommunicationsCrossChannelForwardSourceDeliveryErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Command(command)) = envelope.semantics else {
        return Err(CommunicationsCrossChannelForwardSourceDeliveryErrorV1::InvalidEnvelope);
    };
    if command.target_capability != CROSS_CHANNEL_FORWARD_SOURCE_COMMAND_CAPABILITY_ID_V1 {
        return Err(CommunicationsCrossChannelForwardSourceDeliveryErrorV1::InvalidEnvelope);
    }
    let payload = PrepareCrossChannelForwardSourceCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| CommunicationsCrossChannelForwardSourceDeliveryErrorV1::InvalidPayload)?;
    let forward_id = id16(&payload.forward_id)?;
    if command.command_id.as_slice() != forward_id
        || record.message_id() != &forward_id
        || !valid_logical_owner_id(&payload.logical_owner_id)
    {
        return Err(CommunicationsCrossChannelForwardSourceDeliveryErrorV1::InvalidPayload);
    }
    Ok(DecodedPrepareCommandV1 {
        forward_id,
        source_message_id: id16(&payload.source_message_id)?,
        target_conversation_id: id16(&payload.target_conversation_id)?,
        logical_owner_id: payload.logical_owner_id,
    })
}

fn read_and_validate_body(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    receipt: &CommunicationsCrossChannelForwardBodyReceiptV1,
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

fn write_target_bound_source(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    forward_id: [u8; 16],
    snapshot: &CommunicationsCrossChannelForwardSourceSnapshotV1,
    bytes: Vec<u8>,
) -> Result<CrossChannelForwardBodySourceReceiptV1, BodyMaterializationErrorV1> {
    let declared_bytes =
        u64::try_from(bytes.len()).map_err(|_| BodyMaterializationErrorV1::Policy)?;
    if declared_bytes == 0 || declared_bytes > CROSS_CHANNEL_FORWARD_SOURCE_MAX_BYTES_V1 {
        return Err(BodyMaterializationErrorV1::Policy);
    }
    let sha256: [u8; 32] = Sha256::digest(&bytes).into();
    let reference_id = source_reference_id(forward_id, snapshot, sha256);
    let session = request_managed_blob_session_v2(
        control_channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_BLOB_CAPABILITY_ID,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_OWNER_ID_V1,
                module_id: CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_MODULE_ID_V1,
                capability_id: CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
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
    Ok(CrossChannelForwardBodySourceReceiptV1 {
        reference_id: reference_id.to_vec(),
        declared_bytes,
        sha256: sha256.to_vec(),
        custody_transfer_source_proof: source_proof,
    })
}

fn source_reference_id(
    forward_id: [u8; 16],
    snapshot: &CommunicationsCrossChannelForwardSourceSnapshotV1,
    sha256: [u8; 32],
) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communications-cross-channel-forward-source-copy-v1");
    hasher.update(forward_id);
    hasher.update(snapshot.source_message_id);
    hasher.update(snapshot.target_conversation_id);
    hasher.update(snapshot.evidence_id);
    hasher.update(snapshot.evidence_revision.to_be_bytes());
    hasher.update(sha256);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

async fn persist_rejection(
    persistence: &CommunicationsDurablePersistence,
    record: &OutboxRecordV1,
    decoded: &DecodedPrepareCommandV1,
    code: CrossChannelForwardSourceRejectCodeV1,
    context: &CanonicalEventContextV1,
) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsCrossChannelForwardSourceDeliveryErrorV1>
{
    let outbox = build_cross_channel_forward_source_rejected_outbox_record_v1(
        *record.message_id(),
        CrossChannelForwardSourceRejectedV1 {
            forward_id: decoded.forward_id.to_vec(),
            code: code as i32,
            logical_owner_id: decoded.logical_owner_id.clone(),
        },
        &forward_source_envelope_context(context),
    )
    .map_err(|_| CommunicationsCrossChannelForwardSourceDeliveryErrorV1::InvalidPayload)?;
    persistence
        .persist_cross_channel_forward_source_result(
            *record.message_id(),
            *record.envelope_sha256(),
            None,
            &outbox,
            context.recorded_at_unix_seconds,
        )
        .await
        .map_err(persistence_error)
}

fn forward_source_envelope_context(
    context: &CanonicalEventContextV1,
) -> CrossChannelForwardSourceEnvelopeContextV1 {
    CrossChannelForwardSourceEnvelopeContextV1 {
        module_id: COMMUNICATIONS_MODULE_ID.to_owned(),
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.recorded_at_unix_seconds,
        recorded_at_nanos: context.recorded_at_nanos,
    }
}

fn snapshot_rejection_code(
    error: CommunicationsCrossChannelForwardSourceErrorV1,
) -> Result<
    CrossChannelForwardSourceRejectCodeV1,
    CommunicationsCrossChannelForwardSourceDeliveryErrorV1,
> {
    match error {
        CommunicationsCrossChannelForwardSourceErrorV1::InvalidRequest => Ok(
            CrossChannelForwardSourceRejectCodeV1::CrossChannelForwardSourceRejectCodeInvalidRequest,
        ),
        CommunicationsCrossChannelForwardSourceErrorV1::SourceMissingOrInactive => Ok(
            CrossChannelForwardSourceRejectCodeV1::CrossChannelForwardSourceRejectCodeSourceMissingOrInactive,
        ),
        CommunicationsCrossChannelForwardSourceErrorV1::TargetMissing => Ok(
            CrossChannelForwardSourceRejectCodeV1::CrossChannelForwardSourceRejectCodeTargetMissing,
        ),
        CommunicationsCrossChannelForwardSourceErrorV1::SameChannel => Ok(
            CrossChannelForwardSourceRejectCodeV1::CrossChannelForwardSourceRejectCodeSameChannel,
        ),
        CommunicationsCrossChannelForwardSourceErrorV1::ContentUnavailable => Ok(
            CrossChannelForwardSourceRejectCodeV1::CrossChannelForwardSourceRejectCodeContentUnavailable,
        ),
        CommunicationsCrossChannelForwardSourceErrorV1::ContentLimit => Ok(
            CrossChannelForwardSourceRejectCodeV1::CrossChannelForwardSourceRejectCodeContentLimit,
        ),
        CommunicationsCrossChannelForwardSourceErrorV1::StaleRevision
        | CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow
        | CommunicationsCrossChannelForwardSourceErrorV1::StorageUnavailable
        | CommunicationsCrossChannelForwardSourceErrorV1::InboxHashConflict
        | CommunicationsCrossChannelForwardSourceErrorV1::OutboxConflict => {
            Err(CommunicationsCrossChannelForwardSourceDeliveryErrorV1::Persistence)
        }
    }
}

fn persistence_error(
    _: CommunicationsCrossChannelForwardSourceErrorV1,
) -> CommunicationsCrossChannelForwardSourceDeliveryErrorV1 {
    CommunicationsCrossChannelForwardSourceDeliveryErrorV1::Persistence
}

fn delivery_error(
    _: RuntimePullDeliveryErrorV1,
) -> CommunicationsCrossChannelForwardSourceDeliveryErrorV1 {
    CommunicationsCrossChannelForwardSourceDeliveryErrorV1::Unavailable
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsCrossChannelForwardSourceDeliveryErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(CommunicationsCrossChannelForwardSourceDeliveryErrorV1::InvalidPayload)
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
enum BodyMaterializationErrorV1 {
    Policy,
    Unavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> CommunicationsCrossChannelForwardSourceSnapshotV1 {
        CommunicationsCrossChannelForwardSourceSnapshotV1 {
            source_message_id: [1; 16],
            target_conversation_id: [2; 16],
            evidence_id: [3; 16],
            evidence_revision: 4,
            body: CommunicationsCrossChannelForwardBodyReceiptV1 {
                reference_id: [5; 16],
                declared_bytes: 6,
                sha256: [7; 32],
            },
        }
    }

    #[test]
    fn source_reference_is_deterministic_and_snapshot_bound() {
        let first = source_reference_id([8; 16], &snapshot(), [9; 32]);
        let second = source_reference_id([8; 16], &snapshot(), [9; 32]);
        assert_eq!(first, second);
        let mut changed = snapshot();
        changed.evidence_revision += 1;
        assert_ne!(first, source_reference_id([8; 16], &changed, [9; 32]));
    }

    #[test]
    fn policy_rejections_are_typed_but_storage_failures_are_retried() {
        assert_eq!(
            snapshot_rejection_code(
                CommunicationsCrossChannelForwardSourceErrorV1::SameChannel
            ),
            Ok(
                CrossChannelForwardSourceRejectCodeV1::CrossChannelForwardSourceRejectCodeSameChannel
            )
        );
        assert_eq!(
            snapshot_rejection_code(
                CommunicationsCrossChannelForwardSourceErrorV1::StorageUnavailable
            ),
            Err(CommunicationsCrossChannelForwardSourceDeliveryErrorV1::Persistence)
        );
    }
}
