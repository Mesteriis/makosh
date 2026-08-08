//! Event-only Communications source preparation for communication explanations.
//! This owner boundary emits only target-bound Blob custody evidence; provider
//! identity and plaintext never enter the durable event spine.

use std::os::unix::net::UnixStream;

use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobSessionRequestV1,
    request_managed_blob_session_v2,
};
use makosh_communications_ai_source_api::{
    COMMUNICATION_EXPLANATION_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
    COMMUNICATION_EXPLANATION_SOURCE_BLOB_TARGET_MODULE_ID_V1,
    COMMUNICATION_EXPLANATION_SOURCE_BLOB_TARGET_OWNER_ID_V1,
    COMMUNICATION_EXPLANATION_SOURCE_MAX_BYTES_V1,
    COMMUNICATION_EXPLANATION_SOURCE_MAX_PROOF_BYTES_V1,
    COMMUNICATIONS_EXPLANATION_SOURCE_CAPABILITY_ID_V1,
    CommunicationExplanationSourceContentErrorV1, CommunicationExplanationSourceEnvelopeContextV1,
    build_communication_explanation_source_prepared_outbox_record_v1,
    build_communication_explanation_source_rejected_outbox_record_v1,
    communication_explanation_source_prepare_contract_reference_v1,
    encode_communication_explanation_source_content_v1,
    wire::{
        CommunicationExplanationSourceContentReceiptV1, CommunicationExplanationSourceContentV1,
        CommunicationExplanationSourcePreparedV1, CommunicationExplanationSourceRejectCodeV1,
        CommunicationExplanationSourceRejectedV1, PrepareCommunicationExplanationSourceCommandV1,
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
        COMMUNICATIONS_BLOB_CAPABILITY_ID, COMMUNICATIONS_EXPLANATION_SOURCE_BLOB_CAPABILITY_ID,
        COMMUNICATIONS_MODULE_ID,
    },
    canonical_outbox::CanonicalEventContextV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsExplanationSourceDeliveryErrorV1 {
    Unavailable,
    InvalidEnvelope,
    InvalidPayload,
    Persistence,
}

pub async fn consume_next_explanation_source_prepare_v1(
    persistence: &CommunicationsDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    logical_human_owner_id: &str,
    context: &CanonicalEventContextV1,
) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsExplanationSourceDeliveryErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(delivery_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CommunicationsExplanationSourceDeliveryErrorV1::InvalidEnvelope)?;
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
        Err(BodyMaterializationErrorV1::Policy(code)) => {
            let outcome = persist_rejection(persistence, &record, &decoded, code, context).await?;
            delivery.acknowledge().await.map_err(delivery_error)?;
            return Ok(outcome);
        }
        Err(BodyMaterializationErrorV1::Unavailable) => {
            return Err(CommunicationsExplanationSourceDeliveryErrorV1::Unavailable);
        }
    };
    let source_content = match write_target_bound_source(
        control_channel,
        dispatcher,
        decoded.run_id,
        &snapshot,
        plaintext,
    ) {
        Ok(source_content) => source_content,
        Err(BodyMaterializationErrorV1::Policy(code)) => {
            let outcome = persist_rejection(persistence, &record, &decoded, code, context).await?;
            delivery.acknowledge().await.map_err(delivery_error)?;
            return Ok(outcome);
        }
        Err(BodyMaterializationErrorV1::Unavailable) => {
            return Err(CommunicationsExplanationSourceDeliveryErrorV1::Unavailable);
        }
    };
    let outbox = build_communication_explanation_source_prepared_outbox_record_v1(
        *record.message_id(),
        CommunicationExplanationSourcePreparedV1 {
            run_id: decoded.run_id.to_vec(),
            source_message_id: snapshot.source_message_id.to_vec(),
            source_evidence_id: snapshot.evidence_id.to_vec(),
            source_evidence_revision: snapshot.evidence_revision,
            source_content: Some(source_content),
            logical_owner_id: decoded.logical_owner_id.clone(),
        },
        &explanation_source_envelope_context(context),
    )
    .map_err(|_| CommunicationsExplanationSourceDeliveryErrorV1::InvalidPayload)?;
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
        Err(CommunicationsSourceErrorV1::StaleRevision) => persist_rejection(
            persistence,
            &record,
            &decoded,
            CommunicationExplanationSourceRejectCodeV1::CommunicationExplanationSourceRejectCodeStaleRevision,
            context,
        )
        .await?,
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
) -> Result<DecodedPrepareCommandV1, CommunicationsExplanationSourceDeliveryErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CommunicationsExplanationSourceDeliveryErrorV1::InvalidEnvelope)?;
    if !exact_contract(
        envelope.contract.as_ref(),
        &communication_explanation_source_prepare_contract_reference_v1(),
    ) || envelope.source.as_ref().is_none_or(|source| {
        source.module_id != COMMUNICATION_EXPLANATION_SOURCE_BLOB_TARGET_MODULE_ID_V1
            || source.runtime_generation == 0
    }) {
        return Err(CommunicationsExplanationSourceDeliveryErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Command(command)) = envelope.semantics else {
        return Err(CommunicationsExplanationSourceDeliveryErrorV1::InvalidEnvelope);
    };
    if command.target_capability != COMMUNICATIONS_EXPLANATION_SOURCE_CAPABILITY_ID_V1 {
        return Err(CommunicationsExplanationSourceDeliveryErrorV1::InvalidEnvelope);
    }
    let payload =
        PrepareCommunicationExplanationSourceCommandV1::decode(envelope.payload.as_slice())
            .map_err(|_| CommunicationsExplanationSourceDeliveryErrorV1::InvalidPayload)?;
    let run_id = id16(&payload.run_id)?;
    if command.command_id.as_slice() != run_id
        || record.message_id() != &run_id
        || payload.expected_source_revision == 0
        || payload.logical_owner_id != logical_human_owner_id
        || !valid_logical_owner_id(&payload.logical_owner_id)
    {
        return Err(CommunicationsExplanationSourceDeliveryErrorV1::InvalidPayload);
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
    {
        return Err(policy_error());
    }
    if std::str::from_utf8(&bytes).is_err() {
        return Err(BodyMaterializationErrorV1::Policy(
            CommunicationExplanationSourceRejectCodeV1::CommunicationExplanationSourceRejectCodeInvalidUtf8,
        ));
    }
    Ok(bytes)
}

fn write_target_bound_source(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run_id: [u8; 16],
    snapshot: &CommunicationsSourceSnapshotV1,
    bytes: Vec<u8>,
) -> Result<CommunicationExplanationSourceContentReceiptV1, BodyMaterializationErrorV1> {
    let bytes = encode_communication_explanation_source_content_v1(
        &CommunicationExplanationSourceContentV1 {
            sender_utf8: snapshot.sender_utf8.clone(),
            subject_utf8: snapshot.subject_utf8.clone(),
            body_utf8: bytes,
        },
    )
    .map_err(|error| match error {
        CommunicationExplanationSourceContentErrorV1::Invalid => policy_error(),
        CommunicationExplanationSourceContentErrorV1::Limit => content_limit_error(),
    })?;
    let declared_bytes = u64::try_from(bytes.len()).map_err(|_| content_limit_error())?;
    if declared_bytes == 0 || declared_bytes > COMMUNICATION_EXPLANATION_SOURCE_MAX_BYTES_V1 {
        return Err(content_limit_error());
    }
    let sha256: [u8; 32] = Sha256::digest(&bytes).into();
    let reference_id = source_reference_id(run_id, snapshot, sha256);
    let session = request_managed_blob_session_v2(
        control_channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: COMMUNICATIONS_EXPLANATION_SOURCE_BLOB_CAPABILITY_ID,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: COMMUNICATION_EXPLANATION_SOURCE_BLOB_TARGET_OWNER_ID_V1,
                module_id: COMMUNICATION_EXPLANATION_SOURCE_BLOB_TARGET_MODULE_ID_V1,
                capability_id: COMMUNICATION_EXPLANATION_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
            }),
        },
    )
    .map_err(|_| BodyMaterializationErrorV1::Unavailable)?;
    let source_proof = session.custody_transfer_source_proof;
    if source_proof.is_empty()
        || source_proof.len() > COMMUNICATION_EXPLANATION_SOURCE_MAX_PROOF_BYTES_V1
    {
        return Err(policy_error());
    }
    BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, bytes))
        .map_err(|_| BodyMaterializationErrorV1::Unavailable)?;
    Ok(CommunicationExplanationSourceContentReceiptV1 {
        reference_id: reference_id.to_vec(),
        declared_bytes,
        sha256: sha256.to_vec(),
        custody_transfer_source_proof: source_proof,
    })
}

fn source_reference_id(
    run_id: [u8; 16],
    snapshot: &CommunicationsSourceSnapshotV1,
    sha256: [u8; 32],
) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communications-ai-explanation-source-copy-v1");
    hasher.update(run_id);
    hasher.update(snapshot.source_message_id);
    hasher.update(snapshot.evidence_id);
    hasher.update(snapshot.evidence_revision.to_be_bytes());
    hasher.update(sha256);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

async fn persist_rejection(
    persistence: &CommunicationsDurablePersistence,
    record: &OutboxRecordV1,
    decoded: &DecodedPrepareCommandV1,
    code: CommunicationExplanationSourceRejectCodeV1,
    context: &CanonicalEventContextV1,
) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsExplanationSourceDeliveryErrorV1> {
    let outbox = build_communication_explanation_source_rejected_outbox_record_v1(
        *record.message_id(),
        CommunicationExplanationSourceRejectedV1 {
            run_id: decoded.run_id.to_vec(),
            code: code as i32,
            logical_owner_id: decoded.logical_owner_id.clone(),
        },
        &explanation_source_envelope_context(context),
    )
    .map_err(|_| CommunicationsExplanationSourceDeliveryErrorV1::InvalidPayload)?;
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

fn explanation_source_envelope_context(
    context: &CanonicalEventContextV1,
) -> CommunicationExplanationSourceEnvelopeContextV1 {
    CommunicationExplanationSourceEnvelopeContextV1 {
        module_id: COMMUNICATIONS_MODULE_ID.to_owned(),
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.recorded_at_unix_seconds,
        recorded_at_nanos: context.recorded_at_nanos,
    }
}

fn snapshot_rejection_code(
    error: CommunicationsSourceErrorV1,
) -> Result<
    CommunicationExplanationSourceRejectCodeV1,
    CommunicationsExplanationSourceDeliveryErrorV1,
> {
    match error {
        CommunicationsSourceErrorV1::InvalidRequest => Ok(
            CommunicationExplanationSourceRejectCodeV1::CommunicationExplanationSourceRejectCodeInvalidRequest,
        ),
        CommunicationsSourceErrorV1::SourceMissingOrInactive => Ok(
            CommunicationExplanationSourceRejectCodeV1::CommunicationExplanationSourceRejectCodeSourceMissingOrInactive,
        ),
        CommunicationsSourceErrorV1::ContentUnavailable => Ok(
            CommunicationExplanationSourceRejectCodeV1::CommunicationExplanationSourceRejectCodeContentUnavailable,
        ),
        CommunicationsSourceErrorV1::ContentLimit => Ok(
            CommunicationExplanationSourceRejectCodeV1::CommunicationExplanationSourceRejectCodeContentLimit,
        ),
        CommunicationsSourceErrorV1::StaleRevision => Ok(
            CommunicationExplanationSourceRejectCodeV1::CommunicationExplanationSourceRejectCodeStaleRevision,
        ),
        CommunicationsSourceErrorV1::InvalidRow
        | CommunicationsSourceErrorV1::StorageUnavailable
        | CommunicationsSourceErrorV1::InboxHashConflict
        | CommunicationsSourceErrorV1::OutboxConflict => {
            Err(CommunicationsExplanationSourceDeliveryErrorV1::Persistence)
        }
    }
}

fn policy_error() -> BodyMaterializationErrorV1 {
    BodyMaterializationErrorV1::Policy(
        CommunicationExplanationSourceRejectCodeV1::CommunicationExplanationSourceRejectCodePolicy,
    )
}

fn content_limit_error() -> BodyMaterializationErrorV1 {
    BodyMaterializationErrorV1::Policy(
        CommunicationExplanationSourceRejectCodeV1::CommunicationExplanationSourceRejectCodeContentLimit,
    )
}

fn persistence_error(
    _: CommunicationsSourceErrorV1,
) -> CommunicationsExplanationSourceDeliveryErrorV1 {
    CommunicationsExplanationSourceDeliveryErrorV1::Persistence
}

fn delivery_error(_: RuntimePullDeliveryErrorV1) -> CommunicationsExplanationSourceDeliveryErrorV1 {
    CommunicationsExplanationSourceDeliveryErrorV1::Unavailable
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsExplanationSourceDeliveryErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(CommunicationsExplanationSourceDeliveryErrorV1::InvalidPayload)
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
    Policy(CommunicationExplanationSourceRejectCodeV1),
    Unavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> CommunicationsSourceSnapshotV1 {
        CommunicationsSourceSnapshotV1 {
            source_message_id: [1; 16],
            evidence_id: [2; 16],
            evidence_revision: 3,
            sender_utf8: b"Ada <ada@example.test>".to_vec(),
            subject_utf8: b"Quarterly update".to_vec(),
            body: CommunicationsBodyReceiptV1 {
                reference_id: [4; 16],
                declared_bytes: 5,
                sha256: [6; 32],
            },
        }
    }

    #[test]
    fn explanation_reference_is_deterministic_revision_bound_and_distinct_from_reply() {
        let left = source_reference_id([7; 16], &snapshot(), [8; 32]);
        assert_eq!(left, source_reference_id([7; 16], &snapshot(), [8; 32]));
        let mut changed = snapshot();
        changed.evidence_revision += 1;
        assert_ne!(left, source_reference_id([7; 16], &changed, [8; 32]));

        let mut reply_hasher = Sha256::new();
        reply_hasher.update(b"communications-ai-reply-source-copy-v1");
        reply_hasher.update([7; 16]);
        reply_hasher.update(snapshot().source_message_id);
        reply_hasher.update(snapshot().evidence_id);
        reply_hasher.update(snapshot().evidence_revision.to_be_bytes());
        reply_hasher.update([8; 32]);
        let reply_reference: [u8; 16] = reply_hasher.finalize()[..16]
            .try_into()
            .expect("digest prefix");
        assert_ne!(left, reply_reference);
    }

    #[test]
    fn wrong_human_owner_is_rejected_before_source_read() {
        let record =
            makosh_communications_ai_source_api::build_communication_explanation_source_prepare_outbox_record_v1(
                [7; 16],
                [8; 16],
                3,
                "owner-2",
                1_800_000_030,
                &CommunicationExplanationSourceEnvelopeContextV1 {
                    module_id: COMMUNICATION_EXPLANATION_SOURCE_BLOB_TARGET_MODULE_ID_V1.to_owned(),
                    runtime_instance_id: "explanation-source-runtime-1".to_owned(),
                    runtime_generation: 1,
                    recorded_at_unix_seconds: 1_800_000_000,
                    recorded_at_nanos: 0,
                },
            )
            .expect("valid producer command");

        assert_eq!(
            decode_prepare_command(&record, "owner-1"),
            Err(CommunicationsExplanationSourceDeliveryErrorV1::InvalidPayload)
        );
    }
}
