#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-summary-core";
pub const COMMUNICATION_SUMMARY_MAX_BYTES_V1: usize = 64 * 1024;
pub const COMMUNICATION_SUMMARY_MAX_CONFIDENCE_BASIS_POINTS_V1: u32 = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationSummaryLengthV1 {
    Short,
    Standard,
    Detailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationSummaryLanguageV1 {
    Auto,
    English,
    Russian,
    Spanish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationSummaryCompletenessV1 {
    Complete,
    Partial,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationSummaryRejectionCodeV1 {
    InvalidRequest,
    SourceRejected,
    InferenceRejected,
    Policy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationSummaryDraftV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub source_message_id: [u8; 16],
    pub expected_source_revision: u64,
    pub length: CommunicationSummaryLengthV1,
    pub language: CommunicationSummaryLanguageV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationSummaryCandidateV1 {
    pub summary_utf8: Vec<u8>,
    pub resolved_language: CommunicationSummaryLanguageV1,
    pub resolved_length: CommunicationSummaryLengthV1,
    pub completeness: CommunicationSummaryCompletenessV1,
    pub confidence_basis_points: u32,
    pub request_digest: [u8; 32],
    pub source_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationSummaryStateV1 {
    Accepted,
    PreparingSource,
    AwaitingInference,
    Ready,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationSummaryStatusV1 {
    pub state: CommunicationSummaryStateV1,
    pub state_revision: u64,
    pub source_evidence_id: Option<[u8; 16]>,
    pub source_evidence_revision: Option<u64>,
    pub source_sha256: Option<[u8; 32]>,
    pub inference_request_digest: Option<[u8; 32]>,
    pub candidate: Option<CommunicationSummaryCandidateV1>,
    pub rejection: Option<CommunicationSummaryRejectionCodeV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationSummaryTransitionV1 {
    BeginSourcePreparation,
    SourcePrepared {
        source_evidence_id: [u8; 16],
        source_evidence_revision: u64,
        source_sha256: [u8; 32],
        inference_request_digest: [u8; 32],
    },
    Complete(CommunicationSummaryCandidateV1),
    Reject(CommunicationSummaryRejectionCodeV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationSummaryValidationErrorV1 {
    InvalidRunId,
    InvalidOperationId,
    InvalidSourceMessageId,
    InvalidSourceRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationSummaryTransitionErrorV1 {
    InvalidTransition,
    InvalidSourceReceipt,
    InvalidCandidate,
    InvalidStatus,
    DigestMismatch,
    RevisionExhausted,
}

pub fn validate_communication_summary_draft_v1(
    draft: &CommunicationSummaryDraftV1,
) -> Result<(), CommunicationSummaryValidationErrorV1> {
    if zero(&draft.run_id) {
        return Err(CommunicationSummaryValidationErrorV1::InvalidRunId);
    }
    if zero(&draft.operation_id) {
        return Err(CommunicationSummaryValidationErrorV1::InvalidOperationId);
    }
    if zero(&draft.source_message_id) {
        return Err(CommunicationSummaryValidationErrorV1::InvalidSourceMessageId);
    }
    if draft.expected_source_revision == 0 {
        return Err(CommunicationSummaryValidationErrorV1::InvalidSourceRevision);
    }
    Ok(())
}

#[must_use]
pub fn accepted_communication_summary_status_v1() -> CommunicationSummaryStatusV1 {
    CommunicationSummaryStatusV1 {
        state: CommunicationSummaryStateV1::Accepted,
        state_revision: 1,
        source_evidence_id: None,
        source_evidence_revision: None,
        source_sha256: None,
        inference_request_digest: None,
        candidate: None,
        rejection: None,
    }
}

pub fn transition_communication_summary_v1(
    current: &CommunicationSummaryStatusV1,
    transition: CommunicationSummaryTransitionV1,
) -> Result<CommunicationSummaryStatusV1, CommunicationSummaryTransitionErrorV1> {
    let next_revision = current
        .state_revision
        .checked_add(1)
        .ok_or(CommunicationSummaryTransitionErrorV1::RevisionExhausted)?;
    match (current.state, transition) {
        (
            CommunicationSummaryStateV1::Accepted,
            CommunicationSummaryTransitionV1::BeginSourcePreparation,
        ) => Ok(CommunicationSummaryStatusV1 {
            state: CommunicationSummaryStateV1::PreparingSource,
            state_revision: next_revision,
            ..current.clone()
        }),
        (
            CommunicationSummaryStateV1::PreparingSource,
            CommunicationSummaryTransitionV1::SourcePrepared {
                source_evidence_id,
                source_evidence_revision,
                source_sha256,
                inference_request_digest,
            },
        ) => {
            if zero(&source_evidence_id)
                || source_evidence_revision == 0
                || zero(&source_sha256)
                || zero(&inference_request_digest)
            {
                return Err(CommunicationSummaryTransitionErrorV1::InvalidSourceReceipt);
            }
            Ok(CommunicationSummaryStatusV1 {
                state: CommunicationSummaryStateV1::AwaitingInference,
                state_revision: next_revision,
                source_evidence_id: Some(source_evidence_id),
                source_evidence_revision: Some(source_evidence_revision),
                source_sha256: Some(source_sha256),
                inference_request_digest: Some(inference_request_digest),
                candidate: None,
                rejection: None,
            })
        }
        (
            CommunicationSummaryStateV1::AwaitingInference,
            CommunicationSummaryTransitionV1::Complete(candidate),
        ) => {
            validate_candidate(&candidate)?;
            if current.inference_request_digest != Some(candidate.request_digest)
                || current.source_sha256 != Some(candidate.source_sha256)
            {
                return Err(CommunicationSummaryTransitionErrorV1::DigestMismatch);
            }
            Ok(CommunicationSummaryStatusV1 {
                state: CommunicationSummaryStateV1::Ready,
                state_revision: next_revision,
                candidate: Some(candidate),
                rejection: None,
                ..current.clone()
            })
        }
        (
            CommunicationSummaryStateV1::Accepted
            | CommunicationSummaryStateV1::PreparingSource
            | CommunicationSummaryStateV1::AwaitingInference,
            CommunicationSummaryTransitionV1::Reject(rejection),
        ) => Ok(CommunicationSummaryStatusV1 {
            state: CommunicationSummaryStateV1::Rejected,
            state_revision: next_revision,
            candidate: None,
            rejection: Some(rejection),
            ..current.clone()
        }),
        _ => Err(CommunicationSummaryTransitionErrorV1::InvalidTransition),
    }
}

pub fn validate_communication_summary_status_v1(
    status: &CommunicationSummaryStatusV1,
) -> Result<(), CommunicationSummaryTransitionErrorV1> {
    if status.state_revision == 0 {
        return Err(CommunicationSummaryTransitionErrorV1::InvalidStatus);
    }
    let has_source = status.source_evidence_id.is_some()
        && status
            .source_evidence_revision
            .is_some_and(|value| value > 0)
        && status.source_sha256.is_some()
        && status.inference_request_digest.is_some();
    let has_partial_source = status.source_evidence_id.is_some()
        || status.source_evidence_revision.is_some()
        || status.source_sha256.is_some()
        || status.inference_request_digest.is_some();
    match status.state {
        CommunicationSummaryStateV1::Accepted | CommunicationSummaryStateV1::PreparingSource => {
            if has_partial_source || status.candidate.is_some() || status.rejection.is_some() {
                return Err(CommunicationSummaryTransitionErrorV1::InvalidStatus);
            }
        }
        CommunicationSummaryStateV1::AwaitingInference => {
            if !has_source || status.candidate.is_some() || status.rejection.is_some() {
                return Err(CommunicationSummaryTransitionErrorV1::InvalidStatus);
            }
        }
        CommunicationSummaryStateV1::Ready => {
            let candidate = status
                .candidate
                .as_ref()
                .ok_or(CommunicationSummaryTransitionErrorV1::InvalidStatus)?;
            validate_candidate(candidate)?;
            if !has_source
                || status.rejection.is_some()
                || status.inference_request_digest != Some(candidate.request_digest)
                || status.source_sha256 != Some(candidate.source_sha256)
            {
                return Err(CommunicationSummaryTransitionErrorV1::InvalidStatus);
            }
        }
        CommunicationSummaryStateV1::Rejected => {
            if status.candidate.is_some() || status.rejection.is_none() {
                return Err(CommunicationSummaryTransitionErrorV1::InvalidStatus);
            }
            if has_partial_source && !has_source {
                return Err(CommunicationSummaryTransitionErrorV1::InvalidStatus);
            }
        }
    }
    Ok(())
}

fn validate_candidate(
    candidate: &CommunicationSummaryCandidateV1,
) -> Result<(), CommunicationSummaryTransitionErrorV1> {
    if candidate.summary_utf8.is_empty()
        || candidate.summary_utf8.len() > COMMUNICATION_SUMMARY_MAX_BYTES_V1
        || std::str::from_utf8(&candidate.summary_utf8).is_err()
        || candidate.confidence_basis_points > COMMUNICATION_SUMMARY_MAX_CONFIDENCE_BASIS_POINTS_V1
        || zero(&candidate.request_digest)
        || zero(&candidate.source_sha256)
    {
        return Err(CommunicationSummaryTransitionErrorV1::InvalidCandidate);
    }
    Ok(())
}

fn zero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> CommunicationSummaryDraftV1 {
        CommunicationSummaryDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            source_message_id: [3; 16],
            expected_source_revision: 7,
            length: CommunicationSummaryLengthV1::Standard,
            language: CommunicationSummaryLanguageV1::Auto,
        }
    }

    fn candidate() -> CommunicationSummaryCandidateV1 {
        CommunicationSummaryCandidateV1 {
            summary_utf8: "Краткое содержание сообщения.".as_bytes().to_vec(),
            resolved_language: CommunicationSummaryLanguageV1::Russian,
            resolved_length: CommunicationSummaryLengthV1::Standard,
            completeness: CommunicationSummaryCompletenessV1::Complete,
            confidence_basis_points: 8_500,
            request_digest: [7; 32],
            source_sha256: [8; 32],
        }
    }

    #[test]
    fn draft_requires_exact_non_zero_canonical_identities_and_revision() {
        assert_eq!(validate_communication_summary_draft_v1(&draft()), Ok(()));
        let mut invalid = draft();
        invalid.operation_id = [0; 16];
        assert_eq!(
            validate_communication_summary_draft_v1(&invalid),
            Err(CommunicationSummaryValidationErrorV1::InvalidOperationId)
        );
        invalid = draft();
        invalid.expected_source_revision = 0;
        assert_eq!(
            validate_communication_summary_draft_v1(&invalid),
            Err(CommunicationSummaryValidationErrorV1::InvalidSourceRevision)
        );
    }

    #[test]
    fn state_machine_reaches_ready_only_with_exact_source_and_request_digests() {
        let accepted = accepted_communication_summary_status_v1();
        let preparing = transition_communication_summary_v1(
            &accepted,
            CommunicationSummaryTransitionV1::BeginSourcePreparation,
        )
        .expect("begin source preparation");
        let awaiting = transition_communication_summary_v1(
            &preparing,
            CommunicationSummaryTransitionV1::SourcePrepared {
                source_evidence_id: [6; 16],
                source_evidence_revision: 9,
                source_sha256: [8; 32],
                inference_request_digest: [7; 32],
            },
        )
        .expect("source prepared");
        let ready = transition_communication_summary_v1(
            &awaiting,
            CommunicationSummaryTransitionV1::Complete(candidate()),
        )
        .expect("complete inference");
        assert_eq!(ready.state, CommunicationSummaryStateV1::Ready);
        assert_eq!(ready.state_revision, 4);
        assert_eq!(ready.candidate, Some(candidate()));

        let mut mismatched = candidate();
        mismatched.request_digest = [9; 32];
        assert_eq!(
            transition_communication_summary_v1(
                &awaiting,
                CommunicationSummaryTransitionV1::Complete(mismatched),
            ),
            Err(CommunicationSummaryTransitionErrorV1::DigestMismatch)
        );
    }

    #[test]
    fn invalid_candidate_and_terminal_reentry_fail_closed() {
        let accepted = accepted_communication_summary_status_v1();
        let rejected = transition_communication_summary_v1(
            &accepted,
            CommunicationSummaryTransitionV1::Reject(CommunicationSummaryRejectionCodeV1::Policy),
        )
        .expect("reject");
        assert_eq!(rejected.state, CommunicationSummaryStateV1::Rejected);
        assert_eq!(
            transition_communication_summary_v1(
                &rejected,
                CommunicationSummaryTransitionV1::BeginSourcePreparation,
            ),
            Err(CommunicationSummaryTransitionErrorV1::InvalidTransition)
        );

        let preparing = transition_communication_summary_v1(
            &accepted,
            CommunicationSummaryTransitionV1::BeginSourcePreparation,
        )
        .expect("begin");
        assert_eq!(
            transition_communication_summary_v1(
                &preparing,
                CommunicationSummaryTransitionV1::SourcePrepared {
                    source_evidence_id: [0; 16],
                    source_evidence_revision: 1,
                    source_sha256: [8; 32],
                    inference_request_digest: [7; 32],
                },
            ),
            Err(CommunicationSummaryTransitionErrorV1::InvalidSourceReceipt)
        );

        assert_eq!(validate_communication_summary_status_v1(&rejected), Ok(()));
        let mut invalid = accepted_communication_summary_status_v1();
        invalid.source_sha256 = Some([8; 32]);
        assert_eq!(
            validate_communication_summary_status_v1(&invalid),
            Err(CommunicationSummaryTransitionErrorV1::InvalidStatus)
        );
    }
}
