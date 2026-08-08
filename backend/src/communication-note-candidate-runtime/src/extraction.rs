use std::os::unix::net::UnixStream;

use makosh_communication_note_candidate_core::{
    CommunicationNoteCandidateRejectionCodeV1, CommunicationNoteCandidateStateV1,
    CommunicationNoteCandidateTransitionV1, CommunicationNoteSourceContentV1,
    extract_communication_note_candidates_v1,
};
use makosh_communication_note_candidate_persistence::{
    CommunicationNoteCandidatePersistenceErrorV1, CommunicationNoteCandidatePersistenceV1,
    PersistedCommunicationNoteCandidateRunV1,
};
use makosh_communications_note_source_api::wire::CommunicationNoteSourceContentV1 as WireSourceContent;
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use prost::Message;

use crate::blob_materialization::{
    CommunicationNoteCandidateBlobErrorV1, read_note_source_v1, release_note_source_v1,
};
use crate::review_submission::{
    CommunicationNoteCandidateReviewSubmissionContextV1,
    CommunicationNoteCandidateReviewSubmissionErrorV1, prepare_review_submissions_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationNoteCandidateExtractionErrorV1 {
    Blob(CommunicationNoteCandidateBlobErrorV1),
    InvalidState,
    Persistence(CommunicationNoteCandidatePersistenceErrorV1),
    ReviewSubmission,
}

pub(crate) async fn complete_communication_note_candidate_extraction_v1(
    persistence: &CommunicationNoteCandidatePersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run: &PersistedCommunicationNoteCandidateRunV1,
    source_bytes: &[u8],
    submission_context: &CommunicationNoteCandidateReviewSubmissionContextV1<'_>,
    occurred_at_unix_millis: i64,
) -> Result<PersistedCommunicationNoteCandidateRunV1, CommunicationNoteCandidateExtractionErrorV1> {
    if run.status.state != CommunicationNoteCandidateStateV1::Extracting {
        return Err(CommunicationNoteCandidateExtractionErrorV1::InvalidState);
    }
    let source_sha256 = run
        .status
        .source_sha256
        .ok_or(CommunicationNoteCandidateExtractionErrorV1::InvalidState)?;
    let source_evidence_id = run
        .status
        .source_evidence_id
        .ok_or(CommunicationNoteCandidateExtractionErrorV1::InvalidState)?;
    let source_evidence_revision = run
        .status
        .source_evidence_revision
        .ok_or(CommunicationNoteCandidateExtractionErrorV1::InvalidState)?;
    let extracted = WireSourceContent::decode(source_bytes)
        .ok()
        .and_then(|source| {
            extract_communication_note_candidates_v1(
                CommunicationNoteSourceContentV1 {
                    subject_utf8: &source.subject_utf8,
                    body_utf8: &source.body_utf8,
                },
                source_evidence_id,
                source_evidence_revision,
            )
            .ok()
        });
    let (transition, review_submissions) = match extracted {
        Some(candidates) => {
            let review_submissions = prepare_review_submissions_v1(
                channel,
                dispatcher,
                &run.logical_owner_id,
                &candidates,
                submission_context,
            )
            .map_err(
                |_error: CommunicationNoteCandidateReviewSubmissionErrorV1| {
                    CommunicationNoteCandidateExtractionErrorV1::ReviewSubmission
                },
            )?;
            (
                CommunicationNoteCandidateTransitionV1::Complete {
                    source_sha256,
                    candidates,
                },
                review_submissions,
            )
        }
        None => (
            CommunicationNoteCandidateTransitionV1::Reject(
                CommunicationNoteCandidateRejectionCodeV1::ExtractionRejected,
            ),
            Vec::new(),
        ),
    };
    persistence
        .persist_extraction_transition(
            &run.logical_owner_id,
            &run.draft.run_id,
            transition,
            &review_submissions,
            occurred_at_unix_millis,
        )
        .await
        .map_err(CommunicationNoteCandidateExtractionErrorV1::Persistence)
}

pub(crate) async fn recover_accepted_communication_note_candidate_once_v1(
    persistence: &CommunicationNoteCandidatePersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    logical_owner_id: &str,
    submission_context: &CommunicationNoteCandidateReviewSubmissionContextV1<'_>,
    occurred_at_unix_millis: i64,
) -> Result<bool, CommunicationNoteCandidateExtractionErrorV1> {
    let Some(run) = persistence
        .load_recoverable_runs(logical_owner_id)
        .await
        .map_err(CommunicationNoteCandidateExtractionErrorV1::Persistence)?
        .into_iter()
        .find(|run| run.status.state != CommunicationNoteCandidateStateV1::PreparingSource)
    else {
        return Ok(false);
    };
    match run.status.state {
        CommunicationNoteCandidateStateV1::Accepted => {
            persistence
                .begin_source_preparation(
                    logical_owner_id,
                    &run.draft.run_id,
                    occurred_at_unix_millis,
                )
                .await
                .map_err(CommunicationNoteCandidateExtractionErrorV1::Persistence)?;
        }
        CommunicationNoteCandidateStateV1::PreparingSource => unreachable!("filtered above"),
        CommunicationNoteCandidateStateV1::Extracting => {
            let cleanup = run
                .source_cleanup
                .as_ref()
                .ok_or(CommunicationNoteCandidateExtractionErrorV1::InvalidState)?;
            let body = read_note_source_v1(channel, dispatcher, cleanup)
                .map_err(CommunicationNoteCandidateExtractionErrorV1::Blob)?;
            let terminal = complete_communication_note_candidate_extraction_v1(
                persistence,
                channel,
                dispatcher,
                &run,
                body.as_slice(),
                submission_context,
                occurred_at_unix_millis,
            )
            .await?;
            release_note_source_v1(
                channel,
                dispatcher,
                run.draft.run_id,
                cleanup,
                terminal.status.state == CommunicationNoteCandidateStateV1::Ready,
            )
            .map_err(CommunicationNoteCandidateExtractionErrorV1::Blob)?;
            persistence
                .complete_blob_cleanup(
                    logical_owner_id,
                    &run.draft.run_id,
                    cleanup,
                    occurred_at_unix_millis,
                )
                .await
                .map_err(CommunicationNoteCandidateExtractionErrorV1::Persistence)?;
        }
        CommunicationNoteCandidateStateV1::Ready | CommunicationNoteCandidateStateV1::Rejected => {
            return Err(CommunicationNoteCandidateExtractionErrorV1::InvalidState);
        }
    }
    Ok(true)
}
