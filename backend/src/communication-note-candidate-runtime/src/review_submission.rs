use std::os::unix::net::UnixStream;

use makosh_communication_note_candidate_core::CommunicationNoteCandidateV1;
use makosh_communication_note_candidate_persistence::UnpublishedCommunicationNoteCandidateEventV1;
use makosh_review_note_candidate_api::{
    ReviewNoteCandidateEnvelopeContextV1, build_submit_review_note_candidate_outbox_record_v1,
    wire::SubmitNoteCandidateForReviewCommandV1,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use sha2::{Digest, Sha256};

use crate::blob_materialization::{
    CommunicationNoteCandidateBlobErrorV1, write_review_candidate_v1,
};

const SUBMISSION_DEADLINE_SECONDS_V1: i64 = 300;

pub(crate) struct CommunicationNoteCandidateReviewSubmissionContextV1<'a> {
    pub module_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CommunicationNoteCandidateReviewSubmissionErrorV1 {
    InvalidContext,
    Blob(CommunicationNoteCandidateBlobErrorV1),
    Envelope,
}

pub(crate) fn prepare_review_submissions_v1(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    logical_owner_id: &str,
    candidates: &[CommunicationNoteCandidateV1],
    context: &CommunicationNoteCandidateReviewSubmissionContextV1<'_>,
) -> Result<
    Vec<UnpublishedCommunicationNoteCandidateEventV1>,
    CommunicationNoteCandidateReviewSubmissionErrorV1,
> {
    let recorded_at_unix_seconds = context.now_unix_millis.div_euclid(1_000);
    let recorded_at_nanos = i32::try_from(context.now_unix_millis.rem_euclid(1_000) * 1_000_000)
        .map_err(|_| CommunicationNoteCandidateReviewSubmissionErrorV1::InvalidContext)?;
    if logical_owner_id.is_empty()
        || context.module_id.is_empty()
        || context.runtime_instance_id.is_empty()
        || context.runtime_generation == 0
        || recorded_at_unix_seconds <= 0
    {
        return Err(CommunicationNoteCandidateReviewSubmissionErrorV1::InvalidContext);
    }
    let envelope_context = ReviewNoteCandidateEnvelopeContextV1 {
        module_id: context.module_id.to_owned(),
        runtime_instance_id: context.runtime_instance_id.to_owned(),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds,
        recorded_at_nanos,
    };
    candidates
        .iter()
        .map(|candidate| {
            let receipt = write_review_candidate_v1(channel, dispatcher, candidate)
                .map_err(CommunicationNoteCandidateReviewSubmissionErrorV1::Blob)?;
            let submission_id = submission_id(logical_owner_id, candidate);
            let record = build_submit_review_note_candidate_outbox_record_v1(
                SubmitNoteCandidateForReviewCommandV1 {
                    submission_id: submission_id.to_vec(),
                    candidate_id: candidate.candidate_id.to_vec(),
                    candidate_digest: candidate.candidate_digest.to_vec(),
                    source_evidence_id: candidate.source_evidence_id.to_vec(),
                    source_evidence_revision: candidate.source_evidence_revision,
                    candidate_content: Some(receipt),
                    logical_owner_id: logical_owner_id.to_owned(),
                },
                recorded_at_unix_seconds.saturating_add(SUBMISSION_DEADLINE_SECONDS_V1),
                &envelope_context,
            )
            .map_err(|_| CommunicationNoteCandidateReviewSubmissionErrorV1::Envelope)?;
            Ok(UnpublishedCommunicationNoteCandidateEventV1 {
                message_id: *record.message_id(),
                envelope_sha256: *record.envelope_sha256(),
                envelope_bytes: record.exact_bytes().to_vec(),
            })
        })
        .collect()
}

fn submission_id(logical_owner_id: &str, candidate: &CommunicationNoteCandidateV1) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication_note_candidate.review-submission.v1\0");
    digest.update(logical_owner_id.as_bytes());
    digest.update([0]);
    digest.update(candidate.candidate_id);
    digest.update(candidate.candidate_digest);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_communication_note_candidate_core::{
        CommunicationNoteSourceBasisV1, CommunicationNoteTopicHintV1,
    };

    #[test]
    fn submission_identity_is_owner_and_candidate_bound() {
        let candidate = fixture_candidate();
        assert_eq!(
            submission_id("owner-1", &candidate),
            submission_id("owner-1", &candidate)
        );
        assert_ne!(
            submission_id("owner-1", &candidate),
            submission_id("owner-2", &candidate)
        );
    }

    fn fixture_candidate() -> CommunicationNoteCandidateV1 {
        CommunicationNoteCandidateV1 {
            candidate_id: [1; 16],
            candidate_digest: [2; 32],
            title: "Signed agreement".to_owned(),
            excerpt: "The agreement was approved.".to_owned(),
            topic_hints: vec![
                CommunicationNoteTopicHintV1::Legal,
                CommunicationNoteTopicHintV1::DecisionStatement,
            ],
            source_basis: CommunicationNoteSourceBasisV1::Body,
            confidence_basis_points: 9_000,
            source_evidence_id: [3; 16],
            source_evidence_revision: 1,
        }
    }
}
