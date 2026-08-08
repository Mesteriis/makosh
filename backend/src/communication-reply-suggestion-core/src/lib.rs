#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-reply-suggestion-core";
pub const REPLY_SUGGESTION_MAX_SUBJECT_BYTES_V1: usize = 998;
pub const REPLY_SUGGESTION_MAX_BODY_BYTES_V1: usize = 64 * 1024;
pub const REPLY_SUGGESTION_MAX_CONFIDENCE_BASIS_POINTS_V1: u32 = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplySuggestionToneV1 {
    Professional,
    Friendly,
    Concise,
    Formal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplySuggestionLanguageV1 {
    Source,
    English,
    Russian,
    Spanish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplySuggestionCompletenessV1 {
    Complete,
    Partial,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplySuggestionRejectionCodeV1 {
    InvalidRequest,
    SourceRejected,
    InferenceRejected,
    Policy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplySuggestionDraftV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub source_message_id: [u8; 16],
    pub expected_source_revision: u64,
    pub tone: ReplySuggestionToneV1,
    pub language: ReplySuggestionLanguageV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplySuggestionCandidateV1 {
    pub subject_utf8: Vec<u8>,
    pub body_utf8: Vec<u8>,
    pub resolved_tone: ReplySuggestionToneV1,
    pub resolved_language: ReplySuggestionLanguageV1,
    pub completeness: ReplySuggestionCompletenessV1,
    pub confidence_basis_points: u32,
    pub request_digest: [u8; 32],
    pub source_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplySuggestionStateV1 {
    Accepted,
    PreparingSource,
    AwaitingInference,
    Ready,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplySuggestionStatusV1 {
    pub state: ReplySuggestionStateV1,
    pub state_revision: u64,
    pub source_evidence_id: Option<[u8; 16]>,
    pub source_evidence_revision: Option<u64>,
    pub source_sha256: Option<[u8; 32]>,
    pub inference_request_digest: Option<[u8; 32]>,
    pub candidate: Option<ReplySuggestionCandidateV1>,
    pub rejection: Option<ReplySuggestionRejectionCodeV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplySuggestionTransitionV1 {
    BeginSourcePreparation,
    SourcePrepared {
        source_evidence_id: [u8; 16],
        source_evidence_revision: u64,
        source_sha256: [u8; 32],
        inference_request_digest: [u8; 32],
    },
    Complete(ReplySuggestionCandidateV1),
    Reject(ReplySuggestionRejectionCodeV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplySuggestionValidationErrorV1 {
    InvalidRunId,
    InvalidOperationId,
    InvalidSourceMessageId,
    InvalidSourceRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplySuggestionTransitionErrorV1 {
    InvalidTransition,
    InvalidSourceReceipt,
    InvalidCandidate,
    InvalidStatus,
    DigestMismatch,
    RevisionExhausted,
}

pub fn validate_reply_suggestion_draft_v1(
    draft: &ReplySuggestionDraftV1,
) -> Result<(), ReplySuggestionValidationErrorV1> {
    if zero(&draft.run_id) {
        return Err(ReplySuggestionValidationErrorV1::InvalidRunId);
    }
    if zero(&draft.operation_id) {
        return Err(ReplySuggestionValidationErrorV1::InvalidOperationId);
    }
    if zero(&draft.source_message_id) {
        return Err(ReplySuggestionValidationErrorV1::InvalidSourceMessageId);
    }
    if draft.expected_source_revision == 0 {
        return Err(ReplySuggestionValidationErrorV1::InvalidSourceRevision);
    }
    Ok(())
}

#[must_use]
pub fn accepted_reply_suggestion_status_v1() -> ReplySuggestionStatusV1 {
    ReplySuggestionStatusV1 {
        state: ReplySuggestionStateV1::Accepted,
        state_revision: 1,
        source_evidence_id: None,
        source_evidence_revision: None,
        source_sha256: None,
        inference_request_digest: None,
        candidate: None,
        rejection: None,
    }
}

pub fn transition_reply_suggestion_v1(
    current: &ReplySuggestionStatusV1,
    transition: ReplySuggestionTransitionV1,
) -> Result<ReplySuggestionStatusV1, ReplySuggestionTransitionErrorV1> {
    let next_revision = current
        .state_revision
        .checked_add(1)
        .ok_or(ReplySuggestionTransitionErrorV1::RevisionExhausted)?;
    match (current.state, transition) {
        (ReplySuggestionStateV1::Accepted, ReplySuggestionTransitionV1::BeginSourcePreparation) => {
            Ok(ReplySuggestionStatusV1 {
                state: ReplySuggestionStateV1::PreparingSource,
                state_revision: next_revision,
                ..current.clone()
            })
        }
        (
            ReplySuggestionStateV1::PreparingSource,
            ReplySuggestionTransitionV1::SourcePrepared {
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
                return Err(ReplySuggestionTransitionErrorV1::InvalidSourceReceipt);
            }
            Ok(ReplySuggestionStatusV1 {
                state: ReplySuggestionStateV1::AwaitingInference,
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
            ReplySuggestionStateV1::AwaitingInference,
            ReplySuggestionTransitionV1::Complete(candidate),
        ) => {
            validate_candidate(&candidate)?;
            if current.inference_request_digest != Some(candidate.request_digest)
                || current.source_sha256 != Some(candidate.source_sha256)
            {
                return Err(ReplySuggestionTransitionErrorV1::DigestMismatch);
            }
            Ok(ReplySuggestionStatusV1 {
                state: ReplySuggestionStateV1::Ready,
                state_revision: next_revision,
                candidate: Some(candidate),
                rejection: None,
                ..current.clone()
            })
        }
        (
            ReplySuggestionStateV1::Accepted
            | ReplySuggestionStateV1::PreparingSource
            | ReplySuggestionStateV1::AwaitingInference,
            ReplySuggestionTransitionV1::Reject(rejection),
        ) => Ok(ReplySuggestionStatusV1 {
            state: ReplySuggestionStateV1::Rejected,
            state_revision: next_revision,
            candidate: None,
            rejection: Some(rejection),
            ..current.clone()
        }),
        _ => Err(ReplySuggestionTransitionErrorV1::InvalidTransition),
    }
}

pub fn validate_reply_suggestion_status_v1(
    status: &ReplySuggestionStatusV1,
) -> Result<(), ReplySuggestionTransitionErrorV1> {
    if status.state_revision == 0 {
        return Err(ReplySuggestionTransitionErrorV1::InvalidStatus);
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
        ReplySuggestionStateV1::Accepted | ReplySuggestionStateV1::PreparingSource => {
            if has_partial_source || status.candidate.is_some() || status.rejection.is_some() {
                return Err(ReplySuggestionTransitionErrorV1::InvalidStatus);
            }
        }
        ReplySuggestionStateV1::AwaitingInference => {
            if !has_source || status.candidate.is_some() || status.rejection.is_some() {
                return Err(ReplySuggestionTransitionErrorV1::InvalidStatus);
            }
        }
        ReplySuggestionStateV1::Ready => {
            let candidate = status
                .candidate
                .as_ref()
                .ok_or(ReplySuggestionTransitionErrorV1::InvalidStatus)?;
            validate_candidate(candidate)?;
            if !has_source
                || status.rejection.is_some()
                || status.inference_request_digest != Some(candidate.request_digest)
                || status.source_sha256 != Some(candidate.source_sha256)
            {
                return Err(ReplySuggestionTransitionErrorV1::InvalidStatus);
            }
        }
        ReplySuggestionStateV1::Rejected => {
            if status.candidate.is_some() || status.rejection.is_none() {
                return Err(ReplySuggestionTransitionErrorV1::InvalidStatus);
            }
            if has_partial_source && !has_source {
                return Err(ReplySuggestionTransitionErrorV1::InvalidStatus);
            }
        }
    }
    Ok(())
}

fn validate_candidate(
    candidate: &ReplySuggestionCandidateV1,
) -> Result<(), ReplySuggestionTransitionErrorV1> {
    if candidate.subject_utf8.len() > REPLY_SUGGESTION_MAX_SUBJECT_BYTES_V1
        || candidate.body_utf8.is_empty()
        || candidate.body_utf8.len() > REPLY_SUGGESTION_MAX_BODY_BYTES_V1
        || std::str::from_utf8(&candidate.subject_utf8).is_err()
        || std::str::from_utf8(&candidate.body_utf8).is_err()
        || candidate.confidence_basis_points > REPLY_SUGGESTION_MAX_CONFIDENCE_BASIS_POINTS_V1
        || zero(&candidate.request_digest)
        || zero(&candidate.source_sha256)
    {
        return Err(ReplySuggestionTransitionErrorV1::InvalidCandidate);
    }
    Ok(())
}

fn zero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> ReplySuggestionDraftV1 {
        ReplySuggestionDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            source_message_id: [3; 16],
            expected_source_revision: 7,
            tone: ReplySuggestionToneV1::Professional,
            language: ReplySuggestionLanguageV1::Source,
        }
    }

    fn candidate() -> ReplySuggestionCandidateV1 {
        ReplySuggestionCandidateV1 {
            subject_utf8: b"Re: status".to_vec(),
            body_utf8: "Спасибо, получил.".as_bytes().to_vec(),
            resolved_tone: ReplySuggestionToneV1::Professional,
            resolved_language: ReplySuggestionLanguageV1::Russian,
            completeness: ReplySuggestionCompletenessV1::Complete,
            confidence_basis_points: 8_500,
            request_digest: [7; 32],
            source_sha256: [8; 32],
        }
    }

    #[test]
    fn draft_requires_exact_non_zero_canonical_identities_and_revision() {
        assert_eq!(validate_reply_suggestion_draft_v1(&draft()), Ok(()));
        let mut invalid = draft();
        invalid.operation_id = [0; 16];
        assert_eq!(
            validate_reply_suggestion_draft_v1(&invalid),
            Err(ReplySuggestionValidationErrorV1::InvalidOperationId)
        );
        invalid = draft();
        invalid.expected_source_revision = 0;
        assert_eq!(
            validate_reply_suggestion_draft_v1(&invalid),
            Err(ReplySuggestionValidationErrorV1::InvalidSourceRevision)
        );
    }

    #[test]
    fn state_machine_reaches_ready_only_with_exact_source_and_request_digests() {
        let accepted = accepted_reply_suggestion_status_v1();
        let preparing = transition_reply_suggestion_v1(
            &accepted,
            ReplySuggestionTransitionV1::BeginSourcePreparation,
        )
        .expect("begin source preparation");
        let awaiting = transition_reply_suggestion_v1(
            &preparing,
            ReplySuggestionTransitionV1::SourcePrepared {
                source_evidence_id: [6; 16],
                source_evidence_revision: 9,
                source_sha256: [8; 32],
                inference_request_digest: [7; 32],
            },
        )
        .expect("source prepared");
        let ready = transition_reply_suggestion_v1(
            &awaiting,
            ReplySuggestionTransitionV1::Complete(candidate()),
        )
        .expect("complete inference");
        assert_eq!(ready.state, ReplySuggestionStateV1::Ready);
        assert_eq!(ready.state_revision, 4);
        assert_eq!(ready.candidate, Some(candidate()));

        let mut mismatched = candidate();
        mismatched.request_digest = [9; 32];
        assert_eq!(
            transition_reply_suggestion_v1(
                &awaiting,
                ReplySuggestionTransitionV1::Complete(mismatched),
            ),
            Err(ReplySuggestionTransitionErrorV1::DigestMismatch)
        );
    }

    #[test]
    fn invalid_candidate_and_terminal_reentry_fail_closed() {
        let accepted = accepted_reply_suggestion_status_v1();
        let rejected = transition_reply_suggestion_v1(
            &accepted,
            ReplySuggestionTransitionV1::Reject(ReplySuggestionRejectionCodeV1::Policy),
        )
        .expect("reject");
        assert_eq!(rejected.state, ReplySuggestionStateV1::Rejected);
        assert_eq!(
            transition_reply_suggestion_v1(
                &rejected,
                ReplySuggestionTransitionV1::BeginSourcePreparation,
            ),
            Err(ReplySuggestionTransitionErrorV1::InvalidTransition)
        );

        let preparing = transition_reply_suggestion_v1(
            &accepted,
            ReplySuggestionTransitionV1::BeginSourcePreparation,
        )
        .expect("begin");
        assert_eq!(
            transition_reply_suggestion_v1(
                &preparing,
                ReplySuggestionTransitionV1::SourcePrepared {
                    source_evidence_id: [0; 16],
                    source_evidence_revision: 1,
                    source_sha256: [8; 32],
                    inference_request_digest: [7; 32],
                },
            ),
            Err(ReplySuggestionTransitionErrorV1::InvalidSourceReceipt)
        );

        assert_eq!(validate_reply_suggestion_status_v1(&rejected), Ok(()));
        let mut invalid = accepted_reply_suggestion_status_v1();
        invalid.source_sha256 = Some([8; 32]);
        assert_eq!(
            validate_reply_suggestion_status_v1(&invalid),
            Err(ReplySuggestionTransitionErrorV1::InvalidStatus)
        );
    }
}
