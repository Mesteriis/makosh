#![forbid(unsafe_code)]

use std::collections::BTreeSet;

pub const PACKAGE: &str = "makosh-communication-recipient-suggestion-core";
pub const COMMUNICATION_RECIPIENT_SOURCE_MAX_BYTES_V1: usize = 256 * 1024;
pub const COMMUNICATION_RECIPIENT_MAX_CANDIDATES_V1: usize = 3;
pub const COMMUNICATION_RECIPIENT_MAX_CONFIDENCE_BASIS_POINTS_V1: u32 = 10_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CommunicationRecipientRoleV1 {
    AccountingOrBookkeeping,
    LegalCounsel,
    ProjectStakeholder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientRationaleV1 {
    FinancialDocumentOrPayment,
    LegalOrContractualReview,
    ProjectStatusOrUpdate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientSourceBasisV1 {
    Body,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationRecipientCandidateV1 {
    pub role: CommunicationRecipientRoleV1,
    pub rationale: CommunicationRecipientRationaleV1,
    pub source_basis: CommunicationRecipientSourceBasisV1,
    pub confidence_basis_points: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationRecipientSuggestionDraftV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub source_message_id: [u8; 16],
    pub expected_source_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientSuggestionStateV1 {
    Accepted,
    PreparingSource,
    Evaluating,
    Ready,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientSuggestionRejectionCodeV1 {
    InvalidRequest,
    SourceRejected,
    EvaluationRejected,
    Policy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationRecipientSuggestionStatusV1 {
    pub state: CommunicationRecipientSuggestionStateV1,
    pub state_revision: u64,
    pub source_evidence_id: Option<[u8; 16]>,
    pub source_evidence_revision: Option<u64>,
    pub source_sha256: Option<[u8; 32]>,
    pub candidates: Option<Vec<CommunicationRecipientCandidateV1>>,
    pub rejection: Option<CommunicationRecipientSuggestionRejectionCodeV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientSuggestionTransitionV1 {
    BeginSourcePreparation,
    SourcePrepared {
        source_evidence_id: [u8; 16],
        source_evidence_revision: u64,
        source_sha256: [u8; 32],
    },
    Complete {
        source_sha256: [u8; 32],
        candidates: Vec<CommunicationRecipientCandidateV1>,
    },
    Reject(CommunicationRecipientSuggestionRejectionCodeV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientSuggestionValidationErrorV1 {
    InvalidRunId,
    InvalidOperationId,
    InvalidSourceMessageId,
    InvalidSourceRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientSuggestionEvaluationErrorV1 {
    EmptySourceDigest,
    SourceLimit,
    InvalidUtf8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientSuggestionTransitionErrorV1 {
    InvalidTransition,
    InvalidSourceReceipt,
    InvalidCandidates,
    DuplicateRole,
    SourceDigestMismatch,
    RevisionExhausted,
}

pub fn validate_communication_recipient_suggestion_draft_v1(
    draft: &CommunicationRecipientSuggestionDraftV1,
) -> Result<(), CommunicationRecipientSuggestionValidationErrorV1> {
    if zero(&draft.run_id) {
        return Err(CommunicationRecipientSuggestionValidationErrorV1::InvalidRunId);
    }
    if zero(&draft.operation_id) {
        return Err(CommunicationRecipientSuggestionValidationErrorV1::InvalidOperationId);
    }
    if zero(&draft.source_message_id) {
        return Err(CommunicationRecipientSuggestionValidationErrorV1::InvalidSourceMessageId);
    }
    if draft.expected_source_revision == 0 {
        return Err(CommunicationRecipientSuggestionValidationErrorV1::InvalidSourceRevision);
    }
    Ok(())
}

#[must_use]
pub fn accepted_communication_recipient_suggestion_status_v1()
-> CommunicationRecipientSuggestionStatusV1 {
    CommunicationRecipientSuggestionStatusV1 {
        state: CommunicationRecipientSuggestionStateV1::Accepted,
        state_revision: 1,
        source_evidence_id: None,
        source_evidence_revision: None,
        source_sha256: None,
        candidates: None,
        rejection: None,
    }
}

pub fn evaluate_communication_recipient_candidates_v1(
    body_utf8: &[u8],
    source_sha256: [u8; 32],
) -> Result<Vec<CommunicationRecipientCandidateV1>, CommunicationRecipientSuggestionEvaluationErrorV1>
{
    if zero(&source_sha256) {
        return Err(CommunicationRecipientSuggestionEvaluationErrorV1::EmptySourceDigest);
    }
    if body_utf8.len() > COMMUNICATION_RECIPIENT_SOURCE_MAX_BYTES_V1 {
        return Err(CommunicationRecipientSuggestionEvaluationErrorV1::SourceLimit);
    }
    let body = std::str::from_utf8(body_utf8)
        .map_err(|_| CommunicationRecipientSuggestionEvaluationErrorV1::InvalidUtf8)?
        .to_lowercase();
    let mut candidates = Vec::with_capacity(COMMUNICATION_RECIPIENT_MAX_CANDIDATES_V1);

    if contains_any(&body, &["invoice", "factura", "payment"]) {
        candidates.push(candidate(
            CommunicationRecipientRoleV1::AccountingOrBookkeeping,
            CommunicationRecipientRationaleV1::FinancialDocumentOrPayment,
        ));
    }
    if contains_any(&body, &["contract", "legal", "nda"]) {
        candidates.push(candidate(
            CommunicationRecipientRoleV1::LegalCounsel,
            CommunicationRecipientRationaleV1::LegalOrContractualReview,
        ));
    }
    if body.contains("project") && contains_any(&body, &["update", "status"]) {
        candidates.push(candidate(
            CommunicationRecipientRoleV1::ProjectStakeholder,
            CommunicationRecipientRationaleV1::ProjectStatusOrUpdate,
        ));
    }
    Ok(candidates)
}

pub fn transition_communication_recipient_suggestion_v1(
    current: &CommunicationRecipientSuggestionStatusV1,
    transition: CommunicationRecipientSuggestionTransitionV1,
) -> Result<
    CommunicationRecipientSuggestionStatusV1,
    CommunicationRecipientSuggestionTransitionErrorV1,
> {
    let next_revision = current
        .state_revision
        .checked_add(1)
        .ok_or(CommunicationRecipientSuggestionTransitionErrorV1::RevisionExhausted)?;
    match (current.state, transition) {
        (
            CommunicationRecipientSuggestionStateV1::Accepted,
            CommunicationRecipientSuggestionTransitionV1::BeginSourcePreparation,
        ) => Ok(CommunicationRecipientSuggestionStatusV1 {
            state: CommunicationRecipientSuggestionStateV1::PreparingSource,
            state_revision: next_revision,
            ..current.clone()
        }),
        (
            CommunicationRecipientSuggestionStateV1::PreparingSource,
            CommunicationRecipientSuggestionTransitionV1::SourcePrepared {
                source_evidence_id,
                source_evidence_revision,
                source_sha256,
            },
        ) => {
            if zero(&source_evidence_id) || source_evidence_revision == 0 || zero(&source_sha256) {
                return Err(
                    CommunicationRecipientSuggestionTransitionErrorV1::InvalidSourceReceipt,
                );
            }
            Ok(CommunicationRecipientSuggestionStatusV1 {
                state: CommunicationRecipientSuggestionStateV1::Evaluating,
                state_revision: next_revision,
                source_evidence_id: Some(source_evidence_id),
                source_evidence_revision: Some(source_evidence_revision),
                source_sha256: Some(source_sha256),
                candidates: None,
                rejection: None,
            })
        }
        (
            CommunicationRecipientSuggestionStateV1::Evaluating,
            CommunicationRecipientSuggestionTransitionV1::Complete {
                source_sha256,
                candidates,
            },
        ) => {
            validate_candidates(&candidates)?;
            if current.source_sha256 != Some(source_sha256) {
                return Err(
                    CommunicationRecipientSuggestionTransitionErrorV1::SourceDigestMismatch,
                );
            }
            Ok(CommunicationRecipientSuggestionStatusV1 {
                state: CommunicationRecipientSuggestionStateV1::Ready,
                state_revision: next_revision,
                candidates: Some(candidates),
                rejection: None,
                ..current.clone()
            })
        }
        (
            CommunicationRecipientSuggestionStateV1::Accepted
            | CommunicationRecipientSuggestionStateV1::PreparingSource
            | CommunicationRecipientSuggestionStateV1::Evaluating,
            CommunicationRecipientSuggestionTransitionV1::Reject(rejection),
        ) => Ok(CommunicationRecipientSuggestionStatusV1 {
            state: CommunicationRecipientSuggestionStateV1::Rejected,
            state_revision: next_revision,
            candidates: None,
            rejection: Some(rejection),
            ..current.clone()
        }),
        _ => Err(CommunicationRecipientSuggestionTransitionErrorV1::InvalidTransition),
    }
}

pub fn validate_communication_recipient_suggestion_status_v1(
    status: &CommunicationRecipientSuggestionStatusV1,
) -> Result<(), CommunicationRecipientSuggestionTransitionErrorV1> {
    if status.state_revision == 0 {
        return Err(CommunicationRecipientSuggestionTransitionErrorV1::InvalidTransition);
    }
    let source_absent = status.source_evidence_id.is_none()
        && status.source_evidence_revision.is_none()
        && status.source_sha256.is_none();
    let source_present = status.source_evidence_id.is_some_and(|value| !zero(&value))
        && status
            .source_evidence_revision
            .is_some_and(|value| value > 0)
        && status.source_sha256.is_some_and(|value| !zero(&value));
    match status.state {
        CommunicationRecipientSuggestionStateV1::Accepted
        | CommunicationRecipientSuggestionStateV1::PreparingSource => {
            if !source_absent || status.candidates.is_some() || status.rejection.is_some() {
                return Err(CommunicationRecipientSuggestionTransitionErrorV1::InvalidTransition);
            }
        }
        CommunicationRecipientSuggestionStateV1::Evaluating => {
            if !source_present || status.candidates.is_some() || status.rejection.is_some() {
                return Err(CommunicationRecipientSuggestionTransitionErrorV1::InvalidTransition);
            }
        }
        CommunicationRecipientSuggestionStateV1::Ready => {
            let candidates = status
                .candidates
                .as_deref()
                .ok_or(CommunicationRecipientSuggestionTransitionErrorV1::InvalidCandidates)?;
            if !source_present || status.rejection.is_some() {
                return Err(CommunicationRecipientSuggestionTransitionErrorV1::InvalidTransition);
            }
            validate_candidates(candidates)?;
        }
        CommunicationRecipientSuggestionStateV1::Rejected => {
            if (!source_absent && !source_present)
                || status.candidates.is_some()
                || status.rejection.is_none()
            {
                return Err(CommunicationRecipientSuggestionTransitionErrorV1::InvalidTransition);
            }
        }
    }
    Ok(())
}

fn validate_candidates(
    candidates: &[CommunicationRecipientCandidateV1],
) -> Result<(), CommunicationRecipientSuggestionTransitionErrorV1> {
    if candidates.len() > COMMUNICATION_RECIPIENT_MAX_CANDIDATES_V1 {
        return Err(CommunicationRecipientSuggestionTransitionErrorV1::InvalidCandidates);
    }
    let mut roles = BTreeSet::new();
    for candidate in candidates {
        if !roles.insert(candidate.role) {
            return Err(CommunicationRecipientSuggestionTransitionErrorV1::DuplicateRole);
        }
        let exact_pair = matches!(
            (candidate.role, candidate.rationale),
            (
                CommunicationRecipientRoleV1::AccountingOrBookkeeping,
                CommunicationRecipientRationaleV1::FinancialDocumentOrPayment
            ) | (
                CommunicationRecipientRoleV1::LegalCounsel,
                CommunicationRecipientRationaleV1::LegalOrContractualReview
            ) | (
                CommunicationRecipientRoleV1::ProjectStakeholder,
                CommunicationRecipientRationaleV1::ProjectStatusOrUpdate
            )
        );
        if !exact_pair
            || candidate.source_basis != CommunicationRecipientSourceBasisV1::Body
            || candidate.confidence_basis_points == 0
            || candidate.confidence_basis_points
                > COMMUNICATION_RECIPIENT_MAX_CONFIDENCE_BASIS_POINTS_V1
        {
            return Err(CommunicationRecipientSuggestionTransitionErrorV1::InvalidCandidates);
        }
    }
    Ok(())
}

fn candidate(
    role: CommunicationRecipientRoleV1,
    rationale: CommunicationRecipientRationaleV1,
) -> CommunicationRecipientCandidateV1 {
    CommunicationRecipientCandidateV1 {
        role,
        rationale,
        source_basis: CommunicationRecipientSourceBasisV1::Body,
        confidence_basis_points: COMMUNICATION_RECIPIENT_MAX_CONFIDENCE_BASIS_POINTS_V1,
    }
}

fn contains_any(value: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| value.contains(term))
}

fn zero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: [u8; 32] = [9; 32];

    #[test]
    fn evaluates_the_three_legacy_product_signals_as_typed_roles() {
        let candidates = evaluate_communication_recipient_candidates_v1(
            b"Project status: invoice payment and NDA legal review",
            DIGEST,
        )
        .expect("valid source");
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.role)
                .collect::<Vec<_>>(),
            vec![
                CommunicationRecipientRoleV1::AccountingOrBookkeeping,
                CommunicationRecipientRoleV1::LegalCounsel,
                CommunicationRecipientRoleV1::ProjectStakeholder,
            ]
        );
    }

    #[test]
    fn evaluates_accounting_signal_without_fabricating_other_roles() {
        let candidates =
            evaluate_communication_recipient_candidates_v1(b"Invoice payment received", DIGEST)
                .expect("accounting candidate");
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.role)
                .collect::<Vec<_>>(),
            vec![CommunicationRecipientRoleV1::AccountingOrBookkeeping]
        );
    }

    #[test]
    fn evaluates_legal_signal_without_fabricating_other_roles() {
        let candidates =
            evaluate_communication_recipient_candidates_v1(b"NDA legal review", DIGEST)
                .expect("legal candidate");
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.role)
                .collect::<Vec<_>>(),
            vec![CommunicationRecipientRoleV1::LegalCounsel]
        );
    }

    #[test]
    fn evaluates_project_signal_without_fabricating_other_roles() {
        let candidates =
            evaluate_communication_recipient_candidates_v1(b"Project status update", DIGEST)
                .expect("project candidate");
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.role)
                .collect::<Vec<_>>(),
            vec![CommunicationRecipientRoleV1::ProjectStakeholder]
        );
    }

    #[test]
    fn allows_empty_candidate_list_without_fabricating_a_recipient() {
        assert_eq!(
            evaluate_communication_recipient_candidates_v1(b"hello", DIGEST),
            Ok(Vec::new())
        );
    }

    #[test]
    fn rejects_invalid_private_source_before_evaluation() {
        assert_eq!(
            evaluate_communication_recipient_candidates_v1(&[0xff], DIGEST),
            Err(CommunicationRecipientSuggestionEvaluationErrorV1::InvalidUtf8)
        );
        assert_eq!(
            evaluate_communication_recipient_candidates_v1(b"invoice", [0; 32]),
            Err(CommunicationRecipientSuggestionEvaluationErrorV1::EmptySourceDigest)
        );
    }

    #[test]
    fn lifecycle_binds_terminal_candidates_to_the_prepared_source_digest() {
        let accepted = accepted_communication_recipient_suggestion_status_v1();
        let preparing = transition_communication_recipient_suggestion_v1(
            &accepted,
            CommunicationRecipientSuggestionTransitionV1::BeginSourcePreparation,
        )
        .expect("preparing");
        let evaluating = transition_communication_recipient_suggestion_v1(
            &preparing,
            CommunicationRecipientSuggestionTransitionV1::SourcePrepared {
                source_evidence_id: [3; 16],
                source_evidence_revision: 4,
                source_sha256: DIGEST,
            },
        )
        .expect("evaluating");
        let candidates =
            evaluate_communication_recipient_candidates_v1(b"invoice", DIGEST).expect("candidate");
        assert_eq!(
            transition_communication_recipient_suggestion_v1(
                &evaluating,
                CommunicationRecipientSuggestionTransitionV1::Complete {
                    source_sha256: [8; 32],
                    candidates: candidates.clone(),
                },
            ),
            Err(CommunicationRecipientSuggestionTransitionErrorV1::SourceDigestMismatch)
        );
        let ready = transition_communication_recipient_suggestion_v1(
            &evaluating,
            CommunicationRecipientSuggestionTransitionV1::Complete {
                source_sha256: DIGEST,
                candidates,
            },
        )
        .expect("ready");
        assert_eq!(ready.state, CommunicationRecipientSuggestionStateV1::Ready);
        assert_eq!(ready.state_revision, 4);
    }

    #[test]
    fn rejects_duplicate_roles_and_role_rationale_mismatch() {
        let accounting = candidate(
            CommunicationRecipientRoleV1::AccountingOrBookkeeping,
            CommunicationRecipientRationaleV1::FinancialDocumentOrPayment,
        );
        assert_eq!(
            validate_candidates(&[accounting.clone(), accounting]),
            Err(CommunicationRecipientSuggestionTransitionErrorV1::DuplicateRole)
        );
        assert_eq!(
            validate_candidates(&[CommunicationRecipientCandidateV1 {
                role: CommunicationRecipientRoleV1::LegalCounsel,
                rationale: CommunicationRecipientRationaleV1::ProjectStatusOrUpdate,
                source_basis: CommunicationRecipientSourceBasisV1::Body,
                confidence_basis_points: 10_000,
            }]),
            Err(CommunicationRecipientSuggestionTransitionErrorV1::InvalidCandidates)
        );
    }
}
