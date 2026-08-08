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
use makosh_review_note_candidate_api::{
    REVIEW_NOTE_CANDIDATE_MODULE_ID_V1, REVIEW_NOTE_CANDIDATE_SUBMISSION_CAPABILITY_ID_V1,
    ReviewNoteCandidateEnvelopeContextV1,
    build_review_note_candidate_submission_rejected_outbox_record_v1,
    build_review_note_candidate_submitted_outbox_record_v1,
    review_note_candidate_submit_contract_reference_v1,
    wire::{
        NoteCandidateReviewSubmissionRejectedV1, NoteCandidateReviewSubmittedV1,
        ReviewNoteCandidateSubmissionRejectCodeV1, SubmitNoteCandidateForReviewCommandV1,
    },
};
use makosh_review_note_candidate_core::{
    ReviewNoteCandidateDraftV1, ReviewNoteCandidateTimestampV1, ReviewNoteSourceBasisV1,
    ReviewNoteTopicHintV1,
};
use makosh_review_note_candidate_persistence::{
    CompleteReviewNoteCandidateSubmissionV1, PersistReviewNoteCandidateMaterializationV1,
    RejectReviewNoteCandidateSubmissionV1, ReserveReviewNoteCandidateSubmissionOutcomeV1,
    ReserveReviewNoteCandidateSubmissionV1, ReviewNoteCandidateBlobReceiptV1,
    ReviewNoteCandidateOutboxRecordV1, ReviewNoteCandidatePersistenceErrorV1,
    ReviewNoteCandidatePersistenceV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::ContractReferenceV1,
};
use prost::Message;

use crate::blob_materialization::{
    ReviewNoteCandidateBlobErrorV1, decode_candidate_content_v1, read_materialized_candidate_v1,
    release_materialized_candidate_v1, transfer_review_candidate_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReviewNoteCandidateSubmissionErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Blob(ReviewNoteCandidateBlobErrorV1),
    Persistence(ReviewNoteCandidatePersistenceErrorV1),
    EventUnavailable,
}

pub(crate) struct ReviewNoteCandidateSubmissionRuntimeContextV1<'a> {
    pub logical_owner_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub(crate) async fn consume_review_note_candidate_submission_once_v1(
    persistence: &ReviewNoteCandidatePersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    runtime: &ReviewNoteCandidateSubmissionRuntimeContextV1<'_>,
) -> Result<bool, ReviewNoteCandidateSubmissionErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| ReviewNoteCandidateSubmissionErrorV1::InvalidEnvelope)?;
    let command = decode_submission(&record, runtime.logical_owner_id)?;
    let reservation = persistence
        .reserve_submission(ReserveReviewNoteCandidateSubmissionV1 {
            logical_owner_id: runtime.logical_owner_id.to_owned(),
            submission_message_id: *record.message_id(),
            submission_envelope_sha256: *record.envelope_sha256(),
            submission_id: command.submission_id,
            candidate_id: command.candidate_id,
            candidate_digest: command.candidate_digest,
            source_evidence_id: command.source_evidence_id,
            source_evidence_revision: command.source_evidence_revision,
            candidate_content: command.candidate_content.clone(),
            received_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(ReviewNoteCandidateSubmissionErrorV1::Persistence)?;
    let persisted = match reservation {
        ReserveReviewNoteCandidateSubmissionOutcomeV1::Reserved(value)
        | ReserveReviewNoteCandidateSubmissionOutcomeV1::Existing(value) => value,
    };
    if persisted.completed {
        cleanup_submission(
            persistence,
            channel,
            dispatcher,
            &persisted,
            !persisted.rejected,
            runtime.now_unix_millis,
        )
        .await?;
        delivery.acknowledge().await.map_err(event_error)?;
        return Ok(true);
    }
    let materialized = match materialize_or_read(
        persistence,
        channel,
        dispatcher,
        &persisted,
        runtime.now_unix_millis,
    )
    .await
    {
        Ok(MaterializationReadOutcomeV1::Ready(value)) => value,
        Ok(MaterializationReadOutcomeV1::Invalid(cleanup)) => {
            reject_submission(
                persistence,
                &record,
                &command,
                runtime,
                ReviewNoteCandidateSubmissionRejectCodeV1::ReviewNoteCandidateSubmissionRejectCodeBlobMismatch,
                runtime.now_unix_millis,
            )
            .await?;
            cleanup_materialization(
                persistence,
                channel,
                dispatcher,
                CleanupMaterializationV1 {
                    logical_owner_id: runtime.logical_owner_id,
                    submission_message_id: *record.message_id(),
                    submission_id: command.submission_id,
                    materialization: Some(&cleanup),
                    accepted: false,
                    now_unix_millis: runtime.now_unix_millis,
                },
            )
            .await?;
            delivery.acknowledge().await.map_err(event_error)?;
            return Ok(true);
        }
        Err(ReviewNoteCandidateSubmissionErrorV1::Blob(
            ReviewNoteCandidateBlobErrorV1::Unavailable,
        )) => {
            return Err(ReviewNoteCandidateSubmissionErrorV1::Blob(
                ReviewNoteCandidateBlobErrorV1::Unavailable,
            ));
        }
        Err(ReviewNoteCandidateSubmissionErrorV1::Blob(
            ReviewNoteCandidateBlobErrorV1::InvalidReceipt,
        )) => {
            reject_submission(
                persistence,
                &record,
                &command,
                runtime,
                ReviewNoteCandidateSubmissionRejectCodeV1::ReviewNoteCandidateSubmissionRejectCodeBlobMismatch,
                runtime.now_unix_millis,
            )
            .await?;
            cleanup_materialization(
                persistence,
                channel,
                dispatcher,
                CleanupMaterializationV1 {
                    logical_owner_id: runtime.logical_owner_id,
                    submission_message_id: *record.message_id(),
                    submission_id: command.submission_id,
                    materialization: persisted.materialization.as_ref(),
                    accepted: false,
                    now_unix_millis: runtime.now_unix_millis,
                },
            )
            .await?;
            delivery.acknowledge().await.map_err(event_error)?;
            return Ok(true);
        }
        Err(error) => return Err(error),
    };
    let content = match decode_candidate_content_v1(materialized.bytes.as_slice()) {
        Ok(value) => value,
        Err(ReviewNoteCandidateBlobErrorV1::InvalidReceipt) => {
            reject_submission(
                persistence,
                &record,
                &command,
                runtime,
                ReviewNoteCandidateSubmissionRejectCodeV1::ReviewNoteCandidateSubmissionRejectCodeBlobMismatch,
                runtime.now_unix_millis,
            )
            .await?;
            cleanup_materialization(
                persistence,
                channel,
                dispatcher,
                CleanupMaterializationV1 {
                    logical_owner_id: runtime.logical_owner_id,
                    submission_message_id: *record.message_id(),
                    submission_id: command.submission_id,
                    materialization: Some(&materialized.cleanup),
                    accepted: false,
                    now_unix_millis: runtime.now_unix_millis,
                },
            )
            .await?;
            delivery.acknowledge().await.map_err(event_error)?;
            return Ok(true);
        }
        Err(error) => return Err(ReviewNoteCandidateSubmissionErrorV1::Blob(error)),
    };
    let draft = ReviewNoteCandidateDraftV1 {
        logical_owner_id: runtime.logical_owner_id.to_owned(),
        candidate_id: command.candidate_id,
        candidate_digest: command.candidate_digest,
        source_evidence_id: command.source_evidence_id,
        source_evidence_revision: command.source_evidence_revision,
        title: content.title,
        excerpt: content.excerpt,
        topic_hints: content
            .topic_hints
            .into_iter()
            .map(topic_hint)
            .collect::<Result<_, _>>()?,
        source_basis: source_basis(content.source_basis)?,
        confidence_basis_points: content.confidence_basis_points,
        submitted_at: timestamp(runtime.now_unix_millis),
    };
    let review_id = makosh_review_note_candidate_core::derive_review_note_candidate_id_v1(
        runtime.logical_owner_id,
        &command.candidate_id,
        &command.candidate_digest,
    )
    .map_err(|_| ReviewNoteCandidateSubmissionErrorV1::InvalidPayload)?;
    let submitted = build_review_note_candidate_submitted_outbox_record_v1(
        *record.message_id(),
        NoteCandidateReviewSubmittedV1 {
            submission_id: command.submission_id.to_vec(),
            review_id: review_id.to_vec(),
            candidate_id: command.candidate_id.to_vec(),
            candidate_digest: command.candidate_digest.to_vec(),
            review_revision: 1,
            logical_owner_id: runtime.logical_owner_id.to_owned(),
        },
        &envelope_context(runtime),
    )
    .map_err(|_| ReviewNoteCandidateSubmissionErrorV1::InvalidPayload)?;
    persistence
        .complete_submission(CompleteReviewNoteCandidateSubmissionV1 {
            logical_owner_id: runtime.logical_owner_id.to_owned(),
            submission_message_id: *record.message_id(),
            draft,
            submitted_result: outbox_record(&submitted),
            occurred_at_unix_millis: runtime.now_unix_millis,
        })
        .await
        .map_err(ReviewNoteCandidateSubmissionErrorV1::Persistence)?;
    cleanup_materialization(
        persistence,
        channel,
        dispatcher,
        CleanupMaterializationV1 {
            logical_owner_id: runtime.logical_owner_id,
            submission_message_id: *record.message_id(),
            submission_id: command.submission_id,
            materialization: Some(&materialized.cleanup),
            accepted: true,
            now_unix_millis: runtime.now_unix_millis,
        },
    )
    .await?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

fn source_basis(
    value: i32,
) -> Result<ReviewNoteSourceBasisV1, ReviewNoteCandidateSubmissionErrorV1> {
    match value {
        1 => Ok(ReviewNoteSourceBasisV1::Subject),
        2 => Ok(ReviewNoteSourceBasisV1::Body),
        3 => Ok(ReviewNoteSourceBasisV1::Combined),
        _ => Err(ReviewNoteCandidateSubmissionErrorV1::InvalidPayload),
    }
}

fn topic_hint(value: i32) -> Result<ReviewNoteTopicHintV1, ReviewNoteCandidateSubmissionErrorV1> {
    match value {
        1 => Ok(ReviewNoteTopicHintV1::Financial),
        2 => Ok(ReviewNoteTopicHintV1::Legal),
        3 => Ok(ReviewNoteTopicHintV1::DecisionStatement),
        4 => Ok(ReviewNoteTopicHintV1::DeadlineStatement),
        _ => Err(ReviewNoteCandidateSubmissionErrorV1::InvalidPayload),
    }
}

struct MaterializedReviewNoteCandidateV1 {
    bytes: zeroize::Zeroizing<Vec<u8>>,
    cleanup: makosh_review_note_candidate_persistence::ReviewNoteCandidateBlobCleanupV1,
}

enum MaterializationReadOutcomeV1 {
    Ready(MaterializedReviewNoteCandidateV1),
    Invalid(makosh_review_note_candidate_persistence::ReviewNoteCandidateBlobCleanupV1),
}

async fn materialize_or_read(
    persistence: &ReviewNoteCandidatePersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    submission: &makosh_review_note_candidate_persistence::PersistedReviewNoteCandidateSubmissionV1,
    now_unix_millis: i64,
) -> Result<MaterializationReadOutcomeV1, ReviewNoteCandidateSubmissionErrorV1> {
    let cleanup = if let Some(cleanup) = &submission.materialization {
        cleanup.clone()
    } else {
        let cleanup = transfer_review_candidate_v1(
            channel,
            dispatcher,
            submission.submission_message_id,
            submission.submission_envelope_sha256,
            &submission.candidate_content,
        )
        .map_err(ReviewNoteCandidateSubmissionErrorV1::Blob)?;
        persistence
            .persist_materialization(PersistReviewNoteCandidateMaterializationV1 {
                logical_owner_id: submission.logical_owner_id.clone(),
                submission_message_id: submission.submission_message_id,
                materialization: cleanup.clone(),
                materialized_at_unix_millis: now_unix_millis,
            })
            .await
            .map_err(ReviewNoteCandidateSubmissionErrorV1::Persistence)?;
        cleanup
    };
    match read_materialized_candidate_v1(channel, dispatcher, &cleanup) {
        Ok(bytes) => Ok(MaterializationReadOutcomeV1::Ready(
            MaterializedReviewNoteCandidateV1 { bytes, cleanup },
        )),
        Err(ReviewNoteCandidateBlobErrorV1::InvalidReceipt) => {
            Ok(MaterializationReadOutcomeV1::Invalid(cleanup))
        }
        Err(error) => Err(ReviewNoteCandidateSubmissionErrorV1::Blob(error)),
    }
}

async fn cleanup_submission(
    persistence: &ReviewNoteCandidatePersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    submission: &makosh_review_note_candidate_persistence::PersistedReviewNoteCandidateSubmissionV1,
    accepted: bool,
    now_unix_millis: i64,
) -> Result<(), ReviewNoteCandidateSubmissionErrorV1> {
    if submission.cleanup_completed_at_unix_millis.is_some() {
        return Ok(());
    }
    cleanup_materialization(
        persistence,
        channel,
        dispatcher,
        CleanupMaterializationV1 {
            logical_owner_id: &submission.logical_owner_id,
            submission_message_id: submission.submission_message_id,
            submission_id: submission.submission_id,
            materialization: submission.materialization.as_ref(),
            accepted,
            now_unix_millis,
        },
    )
    .await
}

struct CleanupMaterializationV1<'a> {
    logical_owner_id: &'a str,
    submission_message_id: [u8; 16],
    submission_id: [u8; 16],
    materialization:
        Option<&'a makosh_review_note_candidate_persistence::ReviewNoteCandidateBlobCleanupV1>,
    accepted: bool,
    now_unix_millis: i64,
}

async fn cleanup_materialization(
    persistence: &ReviewNoteCandidatePersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: CleanupMaterializationV1<'_>,
) -> Result<(), ReviewNoteCandidateSubmissionErrorV1> {
    let Some(materialization) = request.materialization else {
        return Ok(());
    };
    release_materialized_candidate_v1(
        channel,
        dispatcher,
        request.submission_id,
        materialization,
        request.accepted,
    )
    .map_err(ReviewNoteCandidateSubmissionErrorV1::Blob)?;
    persistence
        .complete_blob_cleanup(
            request.logical_owner_id,
            &request.submission_message_id,
            materialization,
            request.now_unix_millis,
        )
        .await
        .map_err(ReviewNoteCandidateSubmissionErrorV1::Persistence)
}

async fn reject_submission(
    persistence: &ReviewNoteCandidatePersistenceV1,
    record: &OutboxRecordV1,
    command: &DecodedSubmissionV1,
    runtime: &ReviewNoteCandidateSubmissionRuntimeContextV1<'_>,
    code: ReviewNoteCandidateSubmissionRejectCodeV1,
    now_unix_millis: i64,
) -> Result<(), ReviewNoteCandidateSubmissionErrorV1> {
    let rejected = build_review_note_candidate_submission_rejected_outbox_record_v1(
        *record.message_id(),
        NoteCandidateReviewSubmissionRejectedV1 {
            submission_id: command.submission_id.to_vec(),
            candidate_id: command.candidate_id.to_vec(),
            code: code as i32,
            logical_owner_id: command.logical_owner_id.clone(),
        },
        &envelope_context_at(runtime, now_unix_millis),
    )
    .map_err(|_| ReviewNoteCandidateSubmissionErrorV1::InvalidPayload)?;
    persistence
        .reject_submission(RejectReviewNoteCandidateSubmissionV1 {
            logical_owner_id: command.logical_owner_id.clone(),
            submission_message_id: *record.message_id(),
            rejected_result: outbox_record(&rejected),
            occurred_at_unix_millis: now_unix_millis,
        })
        .await
        .map_err(ReviewNoteCandidateSubmissionErrorV1::Persistence)
}

struct DecodedSubmissionV1 {
    submission_id: [u8; 16],
    candidate_id: [u8; 16],
    candidate_digest: [u8; 32],
    source_evidence_id: [u8; 16],
    source_evidence_revision: u64,
    candidate_content: ReviewNoteCandidateBlobReceiptV1,
    logical_owner_id: String,
}

fn decode_submission(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<DecodedSubmissionV1, ReviewNoteCandidateSubmissionErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| ReviewNoteCandidateSubmissionErrorV1::InvalidEnvelope)?;
    validate_contract(
        envelope.contract.as_ref(),
        &review_note_candidate_submit_contract_reference_v1(),
    )?;
    let Some(Semantics::Command(CommandMetadataV1 {
        command_id,
        target_capability,
        ..
    })) = envelope.semantics
    else {
        return Err(ReviewNoteCandidateSubmissionErrorV1::InvalidEnvelope);
    };
    if command_id.as_slice() != record.message_id()
        || target_capability != REVIEW_NOTE_CANDIDATE_SUBMISSION_CAPABILITY_ID_V1
    {
        return Err(ReviewNoteCandidateSubmissionErrorV1::InvalidEnvelope);
    }
    let payload = SubmitNoteCandidateForReviewCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| ReviewNoteCandidateSubmissionErrorV1::InvalidPayload)?;
    let submission_id = id16(&payload.submission_id)?;
    if submission_id != *record.message_id()
        || payload.logical_owner_id != expected_logical_owner_id
        || payload.source_evidence_revision == 0
    {
        return Err(ReviewNoteCandidateSubmissionErrorV1::InvalidPayload);
    }
    let receipt = payload
        .candidate_content
        .ok_or(ReviewNoteCandidateSubmissionErrorV1::InvalidPayload)?;
    Ok(DecodedSubmissionV1 {
        submission_id,
        candidate_id: id16(&payload.candidate_id)?,
        candidate_digest: id32(&payload.candidate_digest)?,
        source_evidence_id: id16(&payload.source_evidence_id)?,
        source_evidence_revision: payload.source_evidence_revision,
        candidate_content: ReviewNoteCandidateBlobReceiptV1 {
            reference_id: id16(&receipt.reference_id)?,
            declared_bytes: receipt.declared_bytes,
            sha256: id32(&receipt.sha256)?,
            custody_transfer_source_proof: receipt.custody_transfer_source_proof,
        },
        logical_owner_id: payload.logical_owner_id,
    })
}

fn validate_contract(
    actual: Option<&ContractRefV1>,
    expected: &ContractReferenceV1,
) -> Result<(), ReviewNoteCandidateSubmissionErrorV1> {
    if actual.is_none_or(|actual| {
        actual.owner != expected.owner
            || actual.name != expected.name
            || actual.major != expected.major
            || actual.revision != expected.revision
            || actual.schema_sha256 != expected.schema_sha256
    }) {
        return Err(ReviewNoteCandidateSubmissionErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn outbox_record(record: &OutboxRecordV1) -> ReviewNoteCandidateOutboxRecordV1 {
    ReviewNoteCandidateOutboxRecordV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    }
}

fn envelope_context(
    runtime: &ReviewNoteCandidateSubmissionRuntimeContextV1<'_>,
) -> ReviewNoteCandidateEnvelopeContextV1 {
    envelope_context_at(runtime, runtime.now_unix_millis)
}

fn envelope_context_at(
    runtime: &ReviewNoteCandidateSubmissionRuntimeContextV1<'_>,
    now_unix_millis: i64,
) -> ReviewNoteCandidateEnvelopeContextV1 {
    ReviewNoteCandidateEnvelopeContextV1 {
        module_id: REVIEW_NOTE_CANDIDATE_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime.runtime_instance_id.to_owned(),
        runtime_generation: runtime.runtime_generation,
        recorded_at_unix_seconds: now_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((now_unix_millis % 1_000) * 1_000_000).unwrap_or_default(),
    }
}

fn timestamp(now_unix_millis: i64) -> ReviewNoteCandidateTimestampV1 {
    ReviewNoteCandidateTimestampV1 {
        unix_seconds: now_unix_millis / 1_000,
        nanos: i32::try_from((now_unix_millis % 1_000) * 1_000_000).unwrap_or_default(),
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], ReviewNoteCandidateSubmissionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewNoteCandidateSubmissionErrorV1::InvalidPayload)
}

fn id32(value: &[u8]) -> Result<[u8; 32], ReviewNoteCandidateSubmissionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewNoteCandidateSubmissionErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> ReviewNoteCandidateSubmissionErrorV1 {
    ReviewNoteCandidateSubmissionErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_review_note_candidate_api::{
        ReviewNoteCandidateEnvelopeContextV1, build_submit_review_note_candidate_outbox_record_v1,
        wire::ReviewTargetBoundCandidateReceiptV1,
    };

    #[test]
    fn decoder_requires_exact_command_and_owner() {
        let record = build_submit_review_note_candidate_outbox_record_v1(
            SubmitNoteCandidateForReviewCommandV1 {
                submission_id: vec![1; 16],
                candidate_id: vec![2; 16],
                candidate_digest: vec![3; 32],
                source_evidence_id: vec![4; 16],
                source_evidence_revision: 1,
                candidate_content: Some(ReviewTargetBoundCandidateReceiptV1 {
                    reference_id: vec![5; 16],
                    declared_bytes: 8,
                    sha256: vec![6; 32],
                    custody_transfer_source_proof: vec![7; 32],
                }),
                logical_owner_id: "owner-1".to_owned(),
            },
            1_800_000_100,
            &ReviewNoteCandidateEnvelopeContextV1 {
                module_id: "makosh-communication-note-candidate-runtime".to_owned(),
                runtime_instance_id: "runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .expect("record");
        assert!(decode_submission(&record, "owner-1").is_ok());
        assert_eq!(
            decode_submission(&record, "owner-2").err(),
            Some(ReviewNoteCandidateSubmissionErrorV1::InvalidPayload)
        );
    }
}
