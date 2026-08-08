#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-communication-translation-core";
pub const COMMUNICATION_TRANSLATION_MAX_BYTES_V1: usize = 64 * 1024;
pub const COMMUNICATION_TRANSLATION_MAX_CONFIDENCE_BASIS_POINTS_V1: u32 = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTranslationLanguageV1 {
    English,
    Russian,
    Spanish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTranslationDetectedLanguageV1 {
    Unknown,
    English,
    Russian,
    Spanish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTranslationCompletenessV1 {
    Complete,
    Partial,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTranslationRejectionCodeV1 {
    InvalidRequest,
    SourceRejected,
    InferenceRejected,
    Policy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationTranslationDraftV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub source_message_id: [u8; 16],
    pub expected_source_revision: u64,
    pub target_language: CommunicationTranslationLanguageV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationTranslationCandidateV1 {
    pub translated_text_utf8: Vec<u8>,
    pub detected_source_language: CommunicationTranslationDetectedLanguageV1,
    pub target_language: CommunicationTranslationLanguageV1,
    pub completeness: CommunicationTranslationCompletenessV1,
    pub confidence_basis_points: u32,
    pub request_digest: [u8; 32],
    pub source_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTranslationStateV1 {
    Accepted,
    PreparingSource,
    AwaitingInference,
    Ready,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationTranslationStatusV1 {
    pub state: CommunicationTranslationStateV1,
    pub state_revision: u64,
    pub source_evidence_id: Option<[u8; 16]>,
    pub source_evidence_revision: Option<u64>,
    pub source_sha256: Option<[u8; 32]>,
    pub inference_request_digest: Option<[u8; 32]>,
    pub candidate: Option<CommunicationTranslationCandidateV1>,
    pub rejection: Option<CommunicationTranslationRejectionCodeV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationTranslationTransitionV1 {
    BeginSourcePreparation,
    SourcePrepared {
        source_evidence_id: [u8; 16],
        source_evidence_revision: u64,
        source_sha256: [u8; 32],
        inference_request_digest: [u8; 32],
    },
    Complete(CommunicationTranslationCandidateV1),
    Reject(CommunicationTranslationRejectionCodeV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTranslationValidationErrorV1 {
    InvalidRunId,
    InvalidOperationId,
    InvalidSourceMessageId,
    InvalidSourceRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTranslationTransitionErrorV1 {
    InvalidTransition,
    InvalidSourceReceipt,
    InvalidCandidate,
    InvalidStatus,
    DigestMismatch,
    RevisionExhausted,
}

pub fn validate_communication_translation_draft_v1(
    draft: &CommunicationTranslationDraftV1,
) -> Result<(), CommunicationTranslationValidationErrorV1> {
    if zero(&draft.run_id) {
        return Err(CommunicationTranslationValidationErrorV1::InvalidRunId);
    }
    if zero(&draft.operation_id) {
        return Err(CommunicationTranslationValidationErrorV1::InvalidOperationId);
    }
    if zero(&draft.source_message_id) {
        return Err(CommunicationTranslationValidationErrorV1::InvalidSourceMessageId);
    }
    if draft.expected_source_revision == 0 {
        return Err(CommunicationTranslationValidationErrorV1::InvalidSourceRevision);
    }
    Ok(())
}

#[must_use]
pub fn accepted_communication_translation_status_v1() -> CommunicationTranslationStatusV1 {
    CommunicationTranslationStatusV1 {
        state: CommunicationTranslationStateV1::Accepted,
        state_revision: 1,
        source_evidence_id: None,
        source_evidence_revision: None,
        source_sha256: None,
        inference_request_digest: None,
        candidate: None,
        rejection: None,
    }
}

pub fn transition_communication_translation_v1(
    current: &CommunicationTranslationStatusV1,
    transition: CommunicationTranslationTransitionV1,
) -> Result<CommunicationTranslationStatusV1, CommunicationTranslationTransitionErrorV1> {
    let next_revision = current
        .state_revision
        .checked_add(1)
        .ok_or(CommunicationTranslationTransitionErrorV1::RevisionExhausted)?;
    match (current.state, transition) {
        (
            CommunicationTranslationStateV1::Accepted,
            CommunicationTranslationTransitionV1::BeginSourcePreparation,
        ) => Ok(CommunicationTranslationStatusV1 {
            state: CommunicationTranslationStateV1::PreparingSource,
            state_revision: next_revision,
            ..current.clone()
        }),
        (
            CommunicationTranslationStateV1::PreparingSource,
            CommunicationTranslationTransitionV1::SourcePrepared {
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
                return Err(CommunicationTranslationTransitionErrorV1::InvalidSourceReceipt);
            }
            Ok(CommunicationTranslationStatusV1 {
                state: CommunicationTranslationStateV1::AwaitingInference,
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
            CommunicationTranslationStateV1::AwaitingInference,
            CommunicationTranslationTransitionV1::Complete(candidate),
        ) => {
            validate_candidate(&candidate)?;
            if current.inference_request_digest != Some(candidate.request_digest)
                || current.source_sha256 != Some(candidate.source_sha256)
            {
                return Err(CommunicationTranslationTransitionErrorV1::DigestMismatch);
            }
            Ok(CommunicationTranslationStatusV1 {
                state: CommunicationTranslationStateV1::Ready,
                state_revision: next_revision,
                candidate: Some(candidate),
                rejection: None,
                ..current.clone()
            })
        }
        (
            CommunicationTranslationStateV1::Accepted
            | CommunicationTranslationStateV1::PreparingSource
            | CommunicationTranslationStateV1::AwaitingInference,
            CommunicationTranslationTransitionV1::Reject(rejection),
        ) => Ok(CommunicationTranslationStatusV1 {
            state: CommunicationTranslationStateV1::Rejected,
            state_revision: next_revision,
            candidate: None,
            rejection: Some(rejection),
            ..current.clone()
        }),
        _ => Err(CommunicationTranslationTransitionErrorV1::InvalidTransition),
    }
}

pub fn validate_communication_translation_status_v1(
    status: &CommunicationTranslationStatusV1,
) -> Result<(), CommunicationTranslationTransitionErrorV1> {
    if status.state_revision == 0 {
        return Err(CommunicationTranslationTransitionErrorV1::InvalidStatus);
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
        CommunicationTranslationStateV1::Accepted
        | CommunicationTranslationStateV1::PreparingSource => {
            if has_partial_source || status.candidate.is_some() || status.rejection.is_some() {
                return Err(CommunicationTranslationTransitionErrorV1::InvalidStatus);
            }
        }
        CommunicationTranslationStateV1::AwaitingInference => {
            if !has_source || status.candidate.is_some() || status.rejection.is_some() {
                return Err(CommunicationTranslationTransitionErrorV1::InvalidStatus);
            }
        }
        CommunicationTranslationStateV1::Ready => {
            let candidate = status
                .candidate
                .as_ref()
                .ok_or(CommunicationTranslationTransitionErrorV1::InvalidStatus)?;
            validate_candidate(candidate)?;
            if !has_source
                || status.rejection.is_some()
                || status.inference_request_digest != Some(candidate.request_digest)
                || status.source_sha256 != Some(candidate.source_sha256)
            {
                return Err(CommunicationTranslationTransitionErrorV1::InvalidStatus);
            }
        }
        CommunicationTranslationStateV1::Rejected => {
            if status.candidate.is_some() || status.rejection.is_none() {
                return Err(CommunicationTranslationTransitionErrorV1::InvalidStatus);
            }
            if has_partial_source && !has_source {
                return Err(CommunicationTranslationTransitionErrorV1::InvalidStatus);
            }
        }
    }
    Ok(())
}

fn validate_candidate(
    candidate: &CommunicationTranslationCandidateV1,
) -> Result<(), CommunicationTranslationTransitionErrorV1> {
    if candidate.translated_text_utf8.is_empty()
        || candidate.translated_text_utf8.len() > COMMUNICATION_TRANSLATION_MAX_BYTES_V1
        || std::str::from_utf8(&candidate.translated_text_utf8).is_err()
        || candidate.confidence_basis_points
            > COMMUNICATION_TRANSLATION_MAX_CONFIDENCE_BASIS_POINTS_V1
        || zero(&candidate.request_digest)
        || zero(&candidate.source_sha256)
    {
        return Err(CommunicationTranslationTransitionErrorV1::InvalidCandidate);
    }
    Ok(())
}

fn zero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> CommunicationTranslationDraftV1 {
        CommunicationTranslationDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            source_message_id: [3; 16],
            expected_source_revision: 7,
            target_language: CommunicationTranslationLanguageV1::Russian,
        }
    }

    fn candidate() -> CommunicationTranslationCandidateV1 {
        CommunicationTranslationCandidateV1 {
            translated_text_utf8: "Переведённое сообщение.".as_bytes().to_vec(),
            detected_source_language: CommunicationTranslationDetectedLanguageV1::English,
            target_language: CommunicationTranslationLanguageV1::Russian,
            completeness: CommunicationTranslationCompletenessV1::Complete,
            confidence_basis_points: 8_500,
            request_digest: [7; 32],
            source_sha256: [8; 32],
        }
    }

    #[test]
    fn draft_requires_exact_non_zero_canonical_identities_and_revision() {
        assert_eq!(
            validate_communication_translation_draft_v1(&draft()),
            Ok(())
        );
        let mut invalid = draft();
        invalid.operation_id = [0; 16];
        assert_eq!(
            validate_communication_translation_draft_v1(&invalid),
            Err(CommunicationTranslationValidationErrorV1::InvalidOperationId)
        );
        invalid = draft();
        invalid.expected_source_revision = 0;
        assert_eq!(
            validate_communication_translation_draft_v1(&invalid),
            Err(CommunicationTranslationValidationErrorV1::InvalidSourceRevision)
        );
    }

    #[test]
    fn state_machine_reaches_ready_only_with_exact_source_and_request_digests() {
        let accepted = accepted_communication_translation_status_v1();
        let preparing = transition_communication_translation_v1(
            &accepted,
            CommunicationTranslationTransitionV1::BeginSourcePreparation,
        )
        .expect("begin source preparation");
        let awaiting = transition_communication_translation_v1(
            &preparing,
            CommunicationTranslationTransitionV1::SourcePrepared {
                source_evidence_id: [6; 16],
                source_evidence_revision: 9,
                source_sha256: [8; 32],
                inference_request_digest: [7; 32],
            },
        )
        .expect("source prepared");
        let ready = transition_communication_translation_v1(
            &awaiting,
            CommunicationTranslationTransitionV1::Complete(candidate()),
        )
        .expect("complete inference");
        assert_eq!(ready.state, CommunicationTranslationStateV1::Ready);
        assert_eq!(ready.state_revision, 4);

        let mut mismatched = candidate();
        mismatched.request_digest = [9; 32];
        assert_eq!(
            transition_communication_translation_v1(
                &awaiting,
                CommunicationTranslationTransitionV1::Complete(mismatched),
            ),
            Err(CommunicationTranslationTransitionErrorV1::DigestMismatch)
        );
    }

    #[test]
    fn invalid_candidate_and_terminal_reentry_fail_closed() {
        let accepted = accepted_communication_translation_status_v1();
        let rejected = transition_communication_translation_v1(
            &accepted,
            CommunicationTranslationTransitionV1::Reject(
                CommunicationTranslationRejectionCodeV1::Policy,
            ),
        )
        .expect("reject");
        assert_eq!(
            transition_communication_translation_v1(
                &rejected,
                CommunicationTranslationTransitionV1::BeginSourcePreparation,
            ),
            Err(CommunicationTranslationTransitionErrorV1::InvalidTransition)
        );
        assert_eq!(
            validate_communication_translation_status_v1(&rejected),
            Ok(())
        );
    }
}
