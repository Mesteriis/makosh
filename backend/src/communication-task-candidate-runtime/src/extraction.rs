use std::os::unix::net::UnixStream;

use makosh_communication_task_candidate_core::{
    CommunicationTaskCandidateRejectionCodeV1, CommunicationTaskCandidateStateV1,
    CommunicationTaskCandidateTransitionV1, CommunicationTaskSourceContentV1,
    extract_communication_task_candidates_v1,
};
use makosh_communication_task_candidate_persistence::{
    CommunicationTaskCandidatePersistenceErrorV1, CommunicationTaskCandidatePersistenceV1,
    PersistedCommunicationTaskCandidateRunV1,
};
use makosh_communications_task_source_api::wire::CommunicationTaskSourceContentV1 as WireSourceContent;
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use prost::Message;

use crate::blob_materialization::{
    CommunicationTaskCandidateBlobErrorV1, read_task_source_v1, release_task_source_v1,
};
use crate::review_submission::{
    CommunicationTaskCandidateReviewSubmissionContextV1,
    CommunicationTaskCandidateReviewSubmissionErrorV1, prepare_review_submissions_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTaskCandidateExtractionErrorV1 {
    Blob(CommunicationTaskCandidateBlobErrorV1),
    InvalidState,
    Persistence(CommunicationTaskCandidatePersistenceErrorV1),
    ReviewSubmission,
}

pub(crate) async fn complete_communication_task_candidate_extraction_v1(
    persistence: &CommunicationTaskCandidatePersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run: &PersistedCommunicationTaskCandidateRunV1,
    source_bytes: &[u8],
    submission_context: &CommunicationTaskCandidateReviewSubmissionContextV1<'_>,
    occurred_at_unix_millis: i64,
) -> Result<PersistedCommunicationTaskCandidateRunV1, CommunicationTaskCandidateExtractionErrorV1> {
    if run.status.state != CommunicationTaskCandidateStateV1::Extracting {
        return Err(CommunicationTaskCandidateExtractionErrorV1::InvalidState);
    }
    let source_sha256 = run
        .status
        .source_sha256
        .ok_or(CommunicationTaskCandidateExtractionErrorV1::InvalidState)?;
    let source_evidence_id = run
        .status
        .source_evidence_id
        .ok_or(CommunicationTaskCandidateExtractionErrorV1::InvalidState)?;
    let source_evidence_revision = run
        .status
        .source_evidence_revision
        .ok_or(CommunicationTaskCandidateExtractionErrorV1::InvalidState)?;
    let extracted = WireSourceContent::decode(source_bytes)
        .ok()
        .and_then(|source| {
            extract_communication_task_candidates_v1(
                CommunicationTaskSourceContentV1 {
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
                |_error: CommunicationTaskCandidateReviewSubmissionErrorV1| {
                    CommunicationTaskCandidateExtractionErrorV1::ReviewSubmission
                },
            )?;
            (
                CommunicationTaskCandidateTransitionV1::Complete {
                    source_sha256,
                    candidates,
                },
                review_submissions,
            )
        }
        None => (
            CommunicationTaskCandidateTransitionV1::Reject(
                CommunicationTaskCandidateRejectionCodeV1::ExtractionRejected,
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
        .map_err(CommunicationTaskCandidateExtractionErrorV1::Persistence)
}

pub(crate) async fn recover_accepted_communication_task_candidate_once_v1(
    persistence: &CommunicationTaskCandidatePersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    logical_owner_id: &str,
    submission_context: &CommunicationTaskCandidateReviewSubmissionContextV1<'_>,
    occurred_at_unix_millis: i64,
) -> Result<bool, CommunicationTaskCandidateExtractionErrorV1> {
    let Some(run) = persistence
        .load_recoverable_runs(logical_owner_id)
        .await
        .map_err(CommunicationTaskCandidateExtractionErrorV1::Persistence)?
        .into_iter()
        .find(|run| run.status.state != CommunicationTaskCandidateStateV1::PreparingSource)
    else {
        return Ok(false);
    };
    match run.status.state {
        CommunicationTaskCandidateStateV1::Accepted => {
            persistence
                .begin_source_preparation(
                    logical_owner_id,
                    &run.draft.run_id,
                    occurred_at_unix_millis,
                )
                .await
                .map_err(CommunicationTaskCandidateExtractionErrorV1::Persistence)?;
        }
        CommunicationTaskCandidateStateV1::PreparingSource => unreachable!("filtered above"),
        CommunicationTaskCandidateStateV1::Extracting => {
            let cleanup = run
                .source_cleanup
                .as_ref()
                .ok_or(CommunicationTaskCandidateExtractionErrorV1::InvalidState)?;
            let body = read_task_source_v1(channel, dispatcher, cleanup)
                .map_err(CommunicationTaskCandidateExtractionErrorV1::Blob)?;
            let terminal = complete_communication_task_candidate_extraction_v1(
                persistence,
                channel,
                dispatcher,
                &run,
                body.as_slice(),
                submission_context,
                occurred_at_unix_millis,
            )
            .await?;
            release_task_source_v1(
                channel,
                dispatcher,
                run.draft.run_id,
                cleanup,
                terminal.status.state == CommunicationTaskCandidateStateV1::Ready,
            )
            .map_err(CommunicationTaskCandidateExtractionErrorV1::Blob)?;
            persistence
                .complete_blob_cleanup(
                    logical_owner_id,
                    &run.draft.run_id,
                    cleanup,
                    occurred_at_unix_millis,
                )
                .await
                .map_err(CommunicationTaskCandidateExtractionErrorV1::Persistence)?;
        }
        CommunicationTaskCandidateStateV1::Ready | CommunicationTaskCandidateStateV1::Rejected => {
            return Err(CommunicationTaskCandidateExtractionErrorV1::InvalidState);
        }
    }
    Ok(true)
}
