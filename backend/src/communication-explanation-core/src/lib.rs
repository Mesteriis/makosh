#![forbid(unsafe_code)]

use std::collections::BTreeSet;

pub const PACKAGE: &str = "makosh-communication-explanation-core";
pub const COMMUNICATION_EXPLANATION_MAX_REASONS_V1: usize = 8;
pub const COMMUNICATION_EXPLANATION_MAX_REASON_TEXT_BYTES_V1: usize = 512;
pub const COMMUNICATION_EXPLANATION_MAX_CONFIDENCE_BASIS_POINTS_V1: u32 = 10_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CommunicationExplanationReasonKindV1 {
    Urgency,
    FinancialAttention,
    LegalOrContractual,
    ReplyRequested,
    Deadline,
    AttachmentReference,
    MarketingOrBulk,
    OtherAttention,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationExplanationSourceBasisV1 {
    Subject,
    Body,
    CanonicalMetadata,
    Combined,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationExplanationCompletenessV1 {
    Complete,
    Partial,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationExplanationReasonV1 {
    pub kind: CommunicationExplanationReasonKindV1,
    pub explanation_utf8: Vec<u8>,
    pub source_basis: CommunicationExplanationSourceBasisV1,
    pub confidence_basis_points: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationExplanationDraftV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub source_message_id: [u8; 16],
    pub expected_source_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationExplanationCandidateV1 {
    pub reasons: Vec<CommunicationExplanationReasonV1>,
    pub completeness: CommunicationExplanationCompletenessV1,
    pub confidence_basis_points: u32,
    pub request_digest: [u8; 32],
    pub source_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationExplanationStateV1 {
    Accepted,
    PreparingSource,
    AwaitingInference,
    Ready,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationExplanationRejectionCodeV1 {
    InvalidRequest,
    SourceRejected,
    InferenceRejected,
    Policy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationExplanationStatusV1 {
    pub state: CommunicationExplanationStateV1,
    pub state_revision: u64,
    pub source_evidence_id: Option<[u8; 16]>,
    pub source_evidence_revision: Option<u64>,
    pub source_sha256: Option<[u8; 32]>,
    pub inference_request_digest: Option<[u8; 32]>,
    pub candidate: Option<CommunicationExplanationCandidateV1>,
    pub rejection: Option<CommunicationExplanationRejectionCodeV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationExplanationTransitionV1 {
    BeginSourcePreparation,
    SourcePrepared {
        source_evidence_id: [u8; 16],
        source_evidence_revision: u64,
        source_sha256: [u8; 32],
        inference_request_digest: [u8; 32],
    },
    Complete(CommunicationExplanationCandidateV1),
    Reject(CommunicationExplanationRejectionCodeV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationExplanationValidationErrorV1 {
    InvalidRunId,
    InvalidOperationId,
    InvalidSourceMessageId,
    InvalidSourceRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationExplanationTransitionErrorV1 {
    InvalidTransition,
    InvalidSourceReceipt,
    InvalidCandidate,
    DuplicateReasonKind,
    DigestMismatch,
    InvalidStatus,
    RevisionExhausted,
}

pub fn validate_communication_explanation_draft_v1(
    draft: &CommunicationExplanationDraftV1,
) -> Result<(), CommunicationExplanationValidationErrorV1> {
    if zero(&draft.run_id) {
        return Err(CommunicationExplanationValidationErrorV1::InvalidRunId);
    }
    if zero(&draft.operation_id) {
        return Err(CommunicationExplanationValidationErrorV1::InvalidOperationId);
    }
    if zero(&draft.source_message_id) {
        return Err(CommunicationExplanationValidationErrorV1::InvalidSourceMessageId);
    }
    if draft.expected_source_revision == 0 {
        return Err(CommunicationExplanationValidationErrorV1::InvalidSourceRevision);
    }
    Ok(())
}

#[must_use]
pub fn accepted_communication_explanation_status_v1() -> CommunicationExplanationStatusV1 {
    CommunicationExplanationStatusV1 {
        state: CommunicationExplanationStateV1::Accepted,
        state_revision: 1,
        source_evidence_id: None,
        source_evidence_revision: None,
        source_sha256: None,
        inference_request_digest: None,
        candidate: None,
        rejection: None,
    }
}

pub fn transition_communication_explanation_v1(
    current: &CommunicationExplanationStatusV1,
    transition: CommunicationExplanationTransitionV1,
) -> Result<CommunicationExplanationStatusV1, CommunicationExplanationTransitionErrorV1> {
    let next_revision = current
        .state_revision
        .checked_add(1)
        .ok_or(CommunicationExplanationTransitionErrorV1::RevisionExhausted)?;
    match (current.state, transition) {
        (
            CommunicationExplanationStateV1::Accepted,
            CommunicationExplanationTransitionV1::BeginSourcePreparation,
        ) => Ok(CommunicationExplanationStatusV1 {
            state: CommunicationExplanationStateV1::PreparingSource,
            state_revision: next_revision,
            ..current.clone()
        }),
        (
            CommunicationExplanationStateV1::PreparingSource,
            CommunicationExplanationTransitionV1::SourcePrepared {
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
                return Err(CommunicationExplanationTransitionErrorV1::InvalidSourceReceipt);
            }
            Ok(CommunicationExplanationStatusV1 {
                state: CommunicationExplanationStateV1::AwaitingInference,
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
            CommunicationExplanationStateV1::AwaitingInference,
            CommunicationExplanationTransitionV1::Complete(candidate),
        ) => {
            validate_candidate(&candidate)?;
            if current.inference_request_digest != Some(candidate.request_digest)
                || current.source_sha256 != Some(candidate.source_sha256)
            {
                return Err(CommunicationExplanationTransitionErrorV1::DigestMismatch);
            }
            Ok(CommunicationExplanationStatusV1 {
                state: CommunicationExplanationStateV1::Ready,
                state_revision: next_revision,
                candidate: Some(candidate),
                rejection: None,
                ..current.clone()
            })
        }
        (
            CommunicationExplanationStateV1::Accepted
            | CommunicationExplanationStateV1::PreparingSource
            | CommunicationExplanationStateV1::AwaitingInference,
            CommunicationExplanationTransitionV1::Reject(rejection),
        ) => Ok(CommunicationExplanationStatusV1 {
            state: CommunicationExplanationStateV1::Rejected,
            state_revision: next_revision,
            candidate: None,
            rejection: Some(rejection),
            ..current.clone()
        }),
        _ => Err(CommunicationExplanationTransitionErrorV1::InvalidTransition),
    }
}

pub fn validate_communication_explanation_status_v1(
    status: &CommunicationExplanationStatusV1,
) -> Result<(), CommunicationExplanationTransitionErrorV1> {
    if status.state_revision == 0 {
        return Err(CommunicationExplanationTransitionErrorV1::InvalidStatus);
    }
    match status.state {
        CommunicationExplanationStateV1::Accepted
        | CommunicationExplanationStateV1::PreparingSource => {
            if status.source_evidence_id.is_some()
                || status.source_evidence_revision.is_some()
                || status.source_sha256.is_some()
                || status.inference_request_digest.is_some()
                || status.candidate.is_some()
                || status.rejection.is_some()
            {
                return Err(CommunicationExplanationTransitionErrorV1::InvalidStatus);
            }
        }
        CommunicationExplanationStateV1::AwaitingInference => {
            validate_source_state(status)?;
            if status.candidate.is_some() || status.rejection.is_some() {
                return Err(CommunicationExplanationTransitionErrorV1::InvalidStatus);
            }
        }
        CommunicationExplanationStateV1::Ready => {
            validate_source_state(status)?;
            let candidate = status
                .candidate
                .as_ref()
                .ok_or(CommunicationExplanationTransitionErrorV1::InvalidStatus)?;
            validate_candidate(candidate)?;
            if status.rejection.is_some()
                || status.inference_request_digest != Some(candidate.request_digest)
                || status.source_sha256 != Some(candidate.source_sha256)
            {
                return Err(CommunicationExplanationTransitionErrorV1::InvalidStatus);
            }
        }
        CommunicationExplanationStateV1::Rejected => {
            if status.candidate.is_some() || status.rejection.is_none() {
                return Err(CommunicationExplanationTransitionErrorV1::InvalidStatus);
            }
            let has_source = status.source_evidence_id.is_some()
                || status.source_evidence_revision.is_some()
                || status.source_sha256.is_some()
                || status.inference_request_digest.is_some();
            if has_source {
                validate_source_state(status)?;
            }
        }
    }
    Ok(())
}

fn validate_source_state(
    status: &CommunicationExplanationStatusV1,
) -> Result<(), CommunicationExplanationTransitionErrorV1> {
    if status.source_evidence_id.is_none_or(|value| zero(&value))
        || status
            .source_evidence_revision
            .is_none_or(|value| value == 0)
        || status.source_sha256.is_none_or(|value| zero(&value))
        || status
            .inference_request_digest
            .is_none_or(|value| zero(&value))
    {
        return Err(CommunicationExplanationTransitionErrorV1::InvalidStatus);
    }
    Ok(())
}

fn validate_candidate(
    candidate: &CommunicationExplanationCandidateV1,
) -> Result<(), CommunicationExplanationTransitionErrorV1> {
    if candidate.reasons.len() > COMMUNICATION_EXPLANATION_MAX_REASONS_V1
        || candidate.confidence_basis_points
            > COMMUNICATION_EXPLANATION_MAX_CONFIDENCE_BASIS_POINTS_V1
        || zero(&candidate.request_digest)
        || zero(&candidate.source_sha256)
    {
        return Err(CommunicationExplanationTransitionErrorV1::InvalidCandidate);
    }
    let mut kinds = BTreeSet::new();
    for reason in &candidate.reasons {
        if reason.explanation_utf8.is_empty()
            || reason.explanation_utf8.len() > COMMUNICATION_EXPLANATION_MAX_REASON_TEXT_BYTES_V1
            || std::str::from_utf8(&reason.explanation_utf8).is_err()
            || reason.confidence_basis_points
                > COMMUNICATION_EXPLANATION_MAX_CONFIDENCE_BASIS_POINTS_V1
        {
            return Err(CommunicationExplanationTransitionErrorV1::InvalidCandidate);
        }
        if !kinds.insert(reason.kind) {
            return Err(CommunicationExplanationTransitionErrorV1::DuplicateReasonKind);
        }
    }
    Ok(())
}

fn zero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> CommunicationExplanationDraftV1 {
        CommunicationExplanationDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            source_message_id: [3; 16],
            expected_source_revision: 7,
        }
    }

    fn candidate() -> CommunicationExplanationCandidateV1 {
        CommunicationExplanationCandidateV1 {
            reasons: vec![CommunicationExplanationReasonV1 {
                kind: CommunicationExplanationReasonKindV1::Deadline,
                explanation_utf8: b"A concrete deadline is mentioned.".to_vec(),
                source_basis: CommunicationExplanationSourceBasisV1::Body,
                confidence_basis_points: 8_500,
            }],
            completeness: CommunicationExplanationCompletenessV1::Complete,
            confidence_basis_points: 8_500,
            request_digest: [5; 32],
            source_sha256: [6; 32],
        }
    }

    #[test]
    fn validates_exact_draft_identity() {
        assert_eq!(
            validate_communication_explanation_draft_v1(&draft()),
            Ok(())
        );
        let mut invalid = draft();
        invalid.operation_id = [0; 16];
        assert_eq!(
            validate_communication_explanation_draft_v1(&invalid),
            Err(CommunicationExplanationValidationErrorV1::InvalidOperationId)
        );
    }

    #[test]
    fn transitions_through_source_and_inference_with_matching_digests() {
        let accepted = accepted_communication_explanation_status_v1();
        let preparing = transition_communication_explanation_v1(
            &accepted,
            CommunicationExplanationTransitionV1::BeginSourcePreparation,
        )
        .expect("accepted transition");
        let awaiting = transition_communication_explanation_v1(
            &preparing,
            CommunicationExplanationTransitionV1::SourcePrepared {
                source_evidence_id: [4; 16],
                source_evidence_revision: 7,
                source_sha256: [6; 32],
                inference_request_digest: [5; 32],
            },
        )
        .expect("source transition");
        let ready = transition_communication_explanation_v1(
            &awaiting,
            CommunicationExplanationTransitionV1::Complete(candidate()),
        )
        .expect("inference transition");
        assert_eq!(ready.state, CommunicationExplanationStateV1::Ready);
        assert_eq!(ready.state_revision, 4);
    }

    #[test]
    fn rejects_duplicate_reason_taxonomy() {
        let mut duplicate = candidate();
        duplicate.reasons.push(duplicate.reasons[0].clone());
        assert_eq!(
            validate_candidate(&duplicate),
            Err(CommunicationExplanationTransitionErrorV1::DuplicateReasonKind)
        );
    }

    #[test]
    fn allows_empty_reason_list_without_fabricating_a_reason() {
        let mut empty = candidate();
        empty.reasons.clear();
        empty.confidence_basis_points = 10_000;
        assert_eq!(validate_candidate(&empty), Ok(()));
    }

    #[test]
    fn rejects_invalid_utf8_and_overconfidence() {
        let mut invalid = candidate();
        invalid.reasons[0].explanation_utf8 = vec![0xff];
        assert_eq!(
            validate_candidate(&invalid),
            Err(CommunicationExplanationTransitionErrorV1::InvalidCandidate)
        );
        let mut overconfident = candidate();
        overconfident.confidence_basis_points = 10_001;
        assert_eq!(
            validate_candidate(&overconfident),
            Err(CommunicationExplanationTransitionErrorV1::InvalidCandidate)
        );
    }

    #[test]
    fn status_validation_rejects_partial_source_coordinates() {
        let mut invalid = accepted_communication_explanation_status_v1();
        invalid.state = CommunicationExplanationStateV1::AwaitingInference;
        invalid.source_evidence_id = Some([1; 16]);
        assert_eq!(
            validate_communication_explanation_status_v1(&invalid),
            Err(CommunicationExplanationTransitionErrorV1::InvalidStatus)
        );
    }
}
