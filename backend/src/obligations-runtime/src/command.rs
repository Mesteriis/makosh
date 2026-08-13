use std::os::unix::net::UnixStream;

use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{CommandMetadataV1, ContractRefV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_obligations_api::{
    OBLIGATIONS_MODULE_ID_V1, OBLIGATIONS_REVIEWED_CANDIDATE_COMMAND_CAPABILITY_ID_V1,
    ObligationsCommandEnvelopeContextV1,
    build_obligation_created_from_reviewed_candidate_outbox_record_v1,
    build_obligation_creation_from_reviewed_candidate_rejected_outbox_record_v1,
    create_obligation_from_reviewed_candidate_contract_reference_v1,
    wire::{
        CreateObligationFromReviewedCandidateCommandV1, ObligationCreatedFromReviewedCandidateV1,
        ObligationCreationFromReviewedCandidateRejectedV1, ObligationCreationRejectCodeV1,
    },
};
use makosh_obligations_core::{
    ObligationEvidenceLinkV1, ObligationProvenanceV1, ObligationTimestampV1,
    ReviewedCandidateObligationDraftV1, create_obligation_from_reviewed_candidate_v1,
};
use makosh_obligations_persistence::{
    CompleteReviewedCandidateObligationV1, ObligationsBlobReceiptV1, ObligationsOutboxRecordV1,
    ObligationsPersistenceErrorV1, ObligationsPersistenceV1,
    PersistReviewedCandidateMaterializationV1, PersistedReviewedCandidateCommandV1,
    RejectReviewedCandidateObligationV1, ReserveReviewedCandidateCommandOutcomeV1,
    ReserveReviewedCandidateCommandV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::ContractReferenceV1,
};
use prost::Message;

use crate::blob::{
    ObligationsBlobErrorV1, decode_candidate_content_v1, read_candidate_v1, release_candidate_v1,
    transfer_candidate_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ObligationsCommandErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Blob(ObligationsBlobErrorV1),
    Persistence(ObligationsPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) struct ObligationsCommandRuntimeContextV1<'a> {
    pub logical_owner_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub(crate) async fn consume_obligation_command_once_v1(
    persistence: &ObligationsPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    runtime: &ObligationsCommandRuntimeContextV1<'_>,
) -> Result<bool, ObligationsCommandErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ObligationsCommandErrorV1::InvalidEnvelope)?;
    let decoded = decode_command(&record, runtime.logical_owner_id, runtime.now_unix_millis)?;
    let reservation = persistence
        .reserve_command(&decoded.reservation(&record, runtime.now_unix_millis))
        .await
        .map_err(ObligationsCommandErrorV1::Persistence)?;
    let persisted = match reservation {
        ReserveReviewedCandidateCommandOutcomeV1::Reserved(value)
        | ReserveReviewedCandidateCommandOutcomeV1::Existing(value) => value,
    };
    if decoded.expired && !persisted.completed {
        let cleanup = match materialize_candidate(persistence, channel, dispatcher, &persisted)
            .await
        {
            Ok(cleanup) => Some(cleanup),
            Err(ObligationsCommandErrorV1::Blob(ObligationsBlobErrorV1::InvalidReceipt)) => None,
            Err(error) => return Err(error),
        };
        reject_command(
            persistence,
            &persisted,
            runtime,
            ObligationCreationRejectCodeV1::ObligationCreationRejectCodeInvalidRequest,
        )
        .await?;
        if let Some(cleanup) = cleanup {
            cleanup_command_with_outcome(
                persistence,
                channel,
                dispatcher,
                &persisted,
                &cleanup,
                false,
                runtime.now_unix_millis,
            )
            .await?;
        }
    } else {
        process_persisted_command(persistence, channel, dispatcher, &persisted, runtime).await?;
    }
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

pub(crate) async fn recover_obligation_command_once_v1(
    persistence: &ObligationsPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    runtime: &ObligationsCommandRuntimeContextV1<'_>,
) -> Result<bool, ObligationsCommandErrorV1> {
    let Some(command) = persistence
        .load_recoverable_commands(runtime.logical_owner_id)
        .await
        .map_err(ObligationsCommandErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    process_persisted_command(persistence, channel, dispatcher, &command, runtime).await?;
    Ok(true)
}

async fn process_persisted_command(
    persistence: &ObligationsPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    command: &PersistedReviewedCandidateCommandV1,
    runtime: &ObligationsCommandRuntimeContextV1<'_>,
) -> Result<(), ObligationsCommandErrorV1> {
    let cleanup = match materialize_candidate(persistence, channel, dispatcher, command).await {
        Ok(cleanup) => cleanup,
        Err(ObligationsCommandErrorV1::Blob(ObligationsBlobErrorV1::InvalidReceipt)) => {
            reject_command(
                persistence,
                command,
                runtime,
                ObligationCreationRejectCodeV1::ObligationCreationRejectCodeBlobMismatch,
            )
            .await?;
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    if command.completed {
        cleanup_command(persistence, channel, dispatcher, command, &cleanup, runtime).await?;
        return Ok(());
    }
    let bytes = match read_candidate_v1(channel, dispatcher, &cleanup) {
        Ok(bytes) => bytes,
        Err(ObligationsBlobErrorV1::InvalidReceipt) => {
            reject_command(
                persistence,
                command,
                runtime,
                ObligationCreationRejectCodeV1::ObligationCreationRejectCodeBlobMismatch,
            )
            .await?;
            cleanup_command_with_outcome(
                persistence,
                channel,
                dispatcher,
                command,
                &cleanup,
                false,
                runtime.now_unix_millis,
            )
            .await?;
            return Ok(());
        }
        Err(error) => return Err(ObligationsCommandErrorV1::Blob(error)),
    };
    let content = match decode_candidate_content_v1(bytes.as_slice()) {
        Ok(content) => content,
        Err(ObligationsBlobErrorV1::InvalidReceipt) => {
            reject_command(
                persistence,
                command,
                runtime,
                ObligationCreationRejectCodeV1::ObligationCreationRejectCodeBlobMismatch,
            )
            .await?;
            cleanup_command_with_outcome(
                persistence,
                channel,
                dispatcher,
                command,
                &cleanup,
                false,
                runtime.now_unix_millis,
            )
            .await?;
            return Ok(());
        }
        Err(error) => return Err(ObligationsCommandErrorV1::Blob(error)),
    };
    let mut evidence_links = content
        .evidence_links
        .into_iter()
        .map(|value| {
            Ok(ObligationEvidenceLinkV1 {
                evidence_link_id: id16(&value.evidence_link_id)?,
                evidence_owner_id: value.evidence_owner_id,
                evidence_record_id: id16(&value.evidence_record_id)?,
                evidence_revision: positive_revision(value.evidence_revision)?,
                evidence_digest: id32(&value.evidence_digest)?,
            })
        })
        .collect::<Result<Vec<_>, ObligationsCommandErrorV1>>()?;
    evidence_links.sort_by_key(|value| value.evidence_link_id);
    let draft = ReviewedCandidateObligationDraftV1 {
        logical_owner_id: command.logical_owner_id.clone(),
        provenance: ObligationProvenanceV1 {
            approved_candidate_id: command.approved_candidate_id,
            candidate_digest: command.candidate_digest,
            source_evidence_id: command.source_evidence_id,
            source_evidence_revision: command.source_evidence_revision,
            review_id: command.review_id,
            decision_revision: command.decision_revision,
            decided_by_owner_device_id: command.decided_by_owner_device_id,
        },
        statement: content.statement,
        condition: content.condition,
        due_at: content.due_at.map(candidate_timestamp).transpose()?,
        obligated_party_id: id16(&content.obligated_party_id)?,
        beneficiary_party_id: content
            .beneficiary_party_id
            .map(|value| id16(&value))
            .transpose()?,
        evidence_links,
        created_at: timestamp(runtime.now_unix_millis),
    };
    let obligation = create_obligation_from_reviewed_candidate_v1(draft.clone())
        .map_err(|_| ObligationsCommandErrorV1::InvalidPayload)?;
    let created = build_obligation_created_from_reviewed_candidate_outbox_record_v1(
        command.command_message_id,
        ObligationCreatedFromReviewedCandidateV1 {
            command_id: command.command_id.to_vec(),
            approved_candidate_id: command.approved_candidate_id.to_vec(),
            obligation_id: obligation.obligation_id.to_vec(),
            obligation_revision: obligation.obligation_revision,
            logical_owner_id: command.logical_owner_id.clone(),
        },
        &envelope_context(runtime),
    )
    .map_err(|_| ObligationsCommandErrorV1::InvalidPayload)?;
    persistence
        .complete_obligation(CompleteReviewedCandidateObligationV1 {
            logical_owner_id: command.logical_owner_id.clone(),
            command_message_id: command.command_message_id,
            draft,
            created_result: outbox_record(&created),
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(ObligationsCommandErrorV1::Persistence)?;
    cleanup_command_with_outcome(
        persistence,
        channel,
        dispatcher,
        command,
        &cleanup,
        true,
        runtime.now_unix_millis,
    )
    .await
}

async fn materialize_candidate(
    persistence: &ObligationsPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    command: &PersistedReviewedCandidateCommandV1,
) -> Result<makosh_obligations_persistence::ObligationsBlobCleanupV1, ObligationsCommandErrorV1> {
    if let Some(materialization) = command.materialization.clone() {
        return Ok(materialization);
    }
    let materialization = transfer_candidate_v1(
        channel,
        dispatcher,
        command.command_message_id,
        command.command_envelope_sha256,
        &command.candidate_content,
    )
    .map_err(ObligationsCommandErrorV1::Blob)?;
    persistence
        .persist_materialization(&PersistReviewedCandidateMaterializationV1 {
            logical_owner_id: command.logical_owner_id.clone(),
            command_message_id: command.command_message_id,
            materialization: materialization.clone(),
        })
        .await
        .map_err(ObligationsCommandErrorV1::Persistence)?;
    Ok(materialization)
}

async fn reject_command(
    persistence: &ObligationsPersistenceV1,
    command: &PersistedReviewedCandidateCommandV1,
    runtime: &ObligationsCommandRuntimeContextV1<'_>,
    code: ObligationCreationRejectCodeV1,
) -> Result<(), ObligationsCommandErrorV1> {
    let rejected = build_obligation_creation_from_reviewed_candidate_rejected_outbox_record_v1(
        command.command_message_id,
        ObligationCreationFromReviewedCandidateRejectedV1 {
            command_id: command.command_id.to_vec(),
            approved_candidate_id: command.approved_candidate_id.to_vec(),
            code: code as i32,
            logical_owner_id: command.logical_owner_id.clone(),
        },
        &envelope_context(runtime),
    )
    .map_err(|_| ObligationsCommandErrorV1::InvalidPayload)?;
    persistence
        .reject_obligation(&RejectReviewedCandidateObligationV1 {
            logical_owner_id: command.logical_owner_id.clone(),
            command_message_id: command.command_message_id,
            rejected_result: outbox_record(&rejected),
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(ObligationsCommandErrorV1::Persistence)
}

async fn cleanup_command(
    persistence: &ObligationsPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    command: &PersistedReviewedCandidateCommandV1,
    cleanup: &makosh_obligations_persistence::ObligationsBlobCleanupV1,
    runtime: &ObligationsCommandRuntimeContextV1<'_>,
) -> Result<(), ObligationsCommandErrorV1> {
    if command.cleanup_completed_at_unix_millis.is_some() {
        return Ok(());
    }
    cleanup_command_with_outcome(
        persistence,
        channel,
        dispatcher,
        command,
        cleanup,
        !command.rejected,
        runtime.now_unix_millis,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn cleanup_command_with_outcome(
    persistence: &ObligationsPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    command: &PersistedReviewedCandidateCommandV1,
    cleanup: &makosh_obligations_persistence::ObligationsBlobCleanupV1,
    accepted: bool,
    now_unix_millis: i64,
) -> Result<(), ObligationsCommandErrorV1> {
    release_candidate_v1(channel, dispatcher, command.command_id, cleanup, accepted)
        .map_err(ObligationsCommandErrorV1::Blob)?;
    persistence
        .complete_blob_cleanup(
            &command.logical_owner_id,
            command.command_message_id,
            now_unix_millis,
        )
        .await
        .map_err(ObligationsCommandErrorV1::Persistence)
}

struct DecodedObligationCommandV1 {
    command_id: [u8; 16],
    approved_candidate_id: [u8; 16],
    candidate_digest: [u8; 32],
    source_evidence_id: [u8; 16],
    source_evidence_revision: u64,
    review_id: [u8; 16],
    decision_revision: u64,
    decided_by_owner_device_id: [u8; 16],
    candidate_content: ObligationsBlobReceiptV1,
    logical_owner_id: String,
    expired: bool,
}

impl DecodedObligationCommandV1 {
    fn reservation(
        &self,
        record: &OutboxRecordV1,
        now_unix_millis: i64,
    ) -> ReserveReviewedCandidateCommandV1 {
        ReserveReviewedCandidateCommandV1 {
            logical_owner_id: self.logical_owner_id.clone(),
            command_message_id: *record.message_id(),
            command_envelope_sha256: *record.envelope_sha256(),
            command_id: self.command_id,
            approved_candidate_id: self.approved_candidate_id,
            candidate_digest: self.candidate_digest,
            source_evidence_id: self.source_evidence_id,
            source_evidence_revision: self.source_evidence_revision,
            review_id: self.review_id,
            decision_revision: self.decision_revision,
            decided_by_owner_device_id: self.decided_by_owner_device_id,
            candidate_content: self.candidate_content.clone(),
            received_at_unix_millis: now_unix_millis,
        }
    }
}

fn decode_command(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
    now_unix_millis: i64,
) -> Result<DecodedObligationCommandV1, ObligationsCommandErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ObligationsCommandErrorV1::InvalidEnvelope)?;
    validate_contract(
        envelope.contract.as_ref(),
        &create_obligation_from_reviewed_candidate_contract_reference_v1(),
    )?;
    let Some(Semantics::Command(CommandMetadataV1 {
        command_id,
        target_capability,
        deadline,
        ..
    })) = envelope.semantics
    else {
        return Err(ObligationsCommandErrorV1::InvalidEnvelope);
    };
    if command_id.as_slice() != record.message_id()
        || target_capability != OBLIGATIONS_REVIEWED_CANDIDATE_COMMAND_CAPABILITY_ID_V1
    {
        return Err(ObligationsCommandErrorV1::InvalidEnvelope);
    }
    let payload =
        CreateObligationFromReviewedCandidateCommandV1::decode(envelope.payload.as_slice())
            .map_err(|_| ObligationsCommandErrorV1::InvalidPayload)?;
    let command_id = id16(&payload.command_id)?;
    let approved_candidate_id = id16(&payload.approved_candidate_id)?;
    if command_id != *record.message_id()
        || envelope.partition_key.as_slice() != approved_candidate_id
        || payload.logical_owner_id != expected_logical_owner_id
        || payload.source_evidence_revision == 0
        || payload.decision_revision == 0
        || now_unix_millis <= 0
    {
        return Err(ObligationsCommandErrorV1::InvalidPayload);
    }
    let receipt = payload
        .candidate_content
        .ok_or(ObligationsCommandErrorV1::InvalidPayload)?;
    let expired = deadline.is_none_or(|deadline| {
        deadline.seconds < now_unix_millis / 1_000
            || (deadline.seconds == now_unix_millis / 1_000
                && i64::from(deadline.nanos) <= (now_unix_millis % 1_000) * 1_000_000)
    });
    Ok(DecodedObligationCommandV1 {
        command_id,
        approved_candidate_id,
        candidate_digest: id32(&payload.candidate_digest)?,
        source_evidence_id: id16(&payload.source_evidence_id)?,
        source_evidence_revision: payload.source_evidence_revision,
        review_id: id16(&payload.review_id)?,
        decision_revision: payload.decision_revision,
        decided_by_owner_device_id: id16(&payload.decided_by_owner_device_id)?,
        candidate_content: ObligationsBlobReceiptV1 {
            reference_id: id16(&receipt.reference_id)?,
            declared_bytes: receipt.declared_bytes,
            sha256: id32(&receipt.sha256)?,
            custody_transfer_source_proof: receipt.custody_transfer_source_proof,
        },
        logical_owner_id: payload.logical_owner_id,
        expired,
    })
}

fn validate_contract(
    actual: Option<&ContractRefV1>,
    expected: &ContractReferenceV1,
) -> Result<(), ObligationsCommandErrorV1> {
    if actual.is_none_or(|actual| {
        actual.owner != expected.owner
            || actual.name != expected.name
            || actual.major != expected.major
            || actual.revision != expected.revision
            || actual.schema_sha256 != expected.schema_sha256
    }) {
        return Err(ObligationsCommandErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn outbox_record(record: &OutboxRecordV1) -> ObligationsOutboxRecordV1 {
    ObligationsOutboxRecordV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    }
}

fn envelope_context(
    runtime: &ObligationsCommandRuntimeContextV1<'_>,
) -> ObligationsCommandEnvelopeContextV1 {
    ObligationsCommandEnvelopeContextV1 {
        module_id: OBLIGATIONS_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.to_owned(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: runtime.now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((runtime.now_unix_millis % 1_000) * 1_000_000)
            .unwrap_or_default(),
    }
}

fn timestamp(now_unix_millis: i64) -> ObligationTimestampV1 {
    ObligationTimestampV1 {
        unix_seconds: now_unix_millis / 1_000,
        nanos: i32::try_from((now_unix_millis % 1_000) * 1_000_000).unwrap_or_default(),
    }
}

fn candidate_timestamp(
    value: makosh_obligations_api::wire::TimestampV1,
) -> Result<ObligationTimestampV1, ObligationsCommandErrorV1> {
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return Err(ObligationsCommandErrorV1::InvalidPayload);
    }
    Ok(ObligationTimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    })
}

fn positive_revision(value: u64) -> Result<u64, ObligationsCommandErrorV1> {
    (value > 0)
        .then_some(value)
        .ok_or(ObligationsCommandErrorV1::InvalidPayload)
}

fn id16(value: &[u8]) -> Result<[u8; 16], ObligationsCommandErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ObligationsCommandErrorV1::InvalidPayload)
}

fn id32(value: &[u8]) -> Result<[u8; 32], ObligationsCommandErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(ObligationsCommandErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> ObligationsCommandErrorV1 {
    ObligationsCommandErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use makosh_obligations_api::{
        ObligationsCommandEnvelopeContextV1,
        build_create_obligation_from_reviewed_candidate_outbox_record_v1,
        wire::ObligationsTargetBoundCandidateReceiptV1,
    };

    use super::*;

    #[test]
    fn decoder_requires_exact_obligations_owner_and_partition() {
        let record = build_create_obligation_from_reviewed_candidate_outbox_record_v1(
            CreateObligationFromReviewedCandidateCommandV1 {
                command_id: vec![1; 16],
                approved_candidate_id: vec![2; 16],
                candidate_digest: vec![3; 32],
                source_evidence_id: vec![4; 16],
                source_evidence_revision: 1,
                review_id: vec![5; 16],
                decision_revision: 1,
                decided_by_owner_device_id: vec![6; 16],
                candidate_content: Some(ObligationsTargetBoundCandidateReceiptV1 {
                    reference_id: vec![7; 16],
                    declared_bytes: 8,
                    sha256: vec![9; 32],
                    custody_transfer_source_proof: vec![10; 32],
                }),
                logical_owner_id: "owner-1".to_owned(),
            },
            1_800_000_100,
            &ObligationsCommandEnvelopeContextV1 {
                module_id: "promotion-workflow".to_owned(),
                runtime_instance_id: "runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("command");
        assert!(decode_command(&record, "owner-1", 1_800_000_001_000).is_ok());
        assert_eq!(
            decode_command(&record, "owner-2", 1_800_000_001_000).err(),
            Some(ObligationsCommandErrorV1::InvalidPayload)
        );
    }
}
