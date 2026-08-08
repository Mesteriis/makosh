use sha2::{Digest, Sha256};

use crate::ATTACHMENT_TEXT_EXTRACTION_MAX_SOURCE_BYTES_V1;

const MAX_CUSTODY_SOURCE_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentTextExtractionRequestV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTextScanCandidateV1 {
    pub message_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub blob_reference_id: [u8; 16],
    pub declared_size: u64,
    pub blob_receipt_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextSafetyStateV1 {
    DescriptorOnly,
    BlobPending,
    BlobAdmitted,
    Quarantined,
    SafeForDelivery,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentTextCanonicalSafetyFactV1 {
    pub message_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub expected_state: AttachmentTextSafetyStateV1,
    pub next_state: AttachmentTextSafetyStateV1,
    pub evidence_id: [u8; 16],
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTextCustodyDelegationIntentV1 {
    pub run_id: [u8; 16],
    pub candidate_message_id: [u8; 16],
    pub safety_message_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub safety_evidence_id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextExtractionRejectionV1 {
    InvalidRequest,
    InvalidCandidate,
    InvalidSafetyState,
    CandidateConflict,
    SafetyStateConflict,
    AnchorMismatch,
    NotSafe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTextExtractionRecordDecisionV1 {
    Insert,
    Duplicate,
    Reject(AttachmentTextExtractionRejectionV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachmentTextExtractionJoinDecisionV1 {
    Waiting,
    CustodyDelegationRequired(AttachmentTextCustodyDelegationIntentV1),
    Reject(AttachmentTextExtractionRejectionV1),
}

#[must_use]
pub fn validate_attachment_text_request_v1(request: &AttachmentTextExtractionRequestV1) -> bool {
    valid_identifier(&request.run_id)
        && valid_identifier(&request.operation_id)
        && valid_identifier(&request.attachment_anchor_id)
}

#[must_use]
pub fn decide_attachment_text_scan_candidate_record_v1(
    existing: Option<&AttachmentTextScanCandidateV1>,
    incoming: &AttachmentTextScanCandidateV1,
) -> AttachmentTextExtractionRecordDecisionV1 {
    if !valid_candidate(incoming) {
        return AttachmentTextExtractionRecordDecisionV1::Reject(
            AttachmentTextExtractionRejectionV1::InvalidCandidate,
        );
    }
    match existing {
        None => AttachmentTextExtractionRecordDecisionV1::Insert,
        Some(current) if current == incoming => AttachmentTextExtractionRecordDecisionV1::Duplicate,
        Some(_) => AttachmentTextExtractionRecordDecisionV1::Reject(
            AttachmentTextExtractionRejectionV1::CandidateConflict,
        ),
    }
}

#[must_use]
pub fn decide_attachment_text_safety_record_v1(
    existing: Option<&AttachmentTextCanonicalSafetyFactV1>,
    incoming: &AttachmentTextCanonicalSafetyFactV1,
) -> AttachmentTextExtractionRecordDecisionV1 {
    if !valid_terminal_safety_fact(incoming) {
        return AttachmentTextExtractionRecordDecisionV1::Reject(
            AttachmentTextExtractionRejectionV1::InvalidSafetyState,
        );
    }
    match existing {
        None => AttachmentTextExtractionRecordDecisionV1::Insert,
        Some(current) if current == incoming => AttachmentTextExtractionRecordDecisionV1::Duplicate,
        Some(_) => AttachmentTextExtractionRecordDecisionV1::Reject(
            AttachmentTextExtractionRejectionV1::SafetyStateConflict,
        ),
    }
}

#[must_use]
pub fn decide_attachment_text_join_v1(
    request: &AttachmentTextExtractionRequestV1,
    candidate: Option<&AttachmentTextScanCandidateV1>,
    safety: Option<&AttachmentTextCanonicalSafetyFactV1>,
) -> AttachmentTextExtractionJoinDecisionV1 {
    if !validate_attachment_text_request_v1(request) {
        return AttachmentTextExtractionJoinDecisionV1::Reject(
            AttachmentTextExtractionRejectionV1::InvalidRequest,
        );
    }
    if let Some(value) = candidate
        && !valid_candidate(value)
    {
        return AttachmentTextExtractionJoinDecisionV1::Reject(
            AttachmentTextExtractionRejectionV1::InvalidCandidate,
        );
    }
    if let Some(value) = safety
        && !valid_terminal_safety_fact(value)
    {
        return AttachmentTextExtractionJoinDecisionV1::Reject(
            AttachmentTextExtractionRejectionV1::InvalidSafetyState,
        );
    }
    if candidate.is_some_and(|value| value.attachment_anchor_id != request.attachment_anchor_id)
        || safety.is_some_and(|value| value.attachment_anchor_id != request.attachment_anchor_id)
    {
        return AttachmentTextExtractionJoinDecisionV1::Reject(
            AttachmentTextExtractionRejectionV1::AnchorMismatch,
        );
    }
    if safety.is_some_and(|value| {
        matches!(
            value.next_state,
            AttachmentTextSafetyStateV1::Quarantined | AttachmentTextSafetyStateV1::Rejected
        )
    }) {
        return AttachmentTextExtractionJoinDecisionV1::Reject(
            AttachmentTextExtractionRejectionV1::NotSafe,
        );
    }
    let (Some(candidate), Some(safety)) = (candidate, safety) else {
        return AttachmentTextExtractionJoinDecisionV1::Waiting;
    };
    AttachmentTextExtractionJoinDecisionV1::CustodyDelegationRequired(
        AttachmentTextCustodyDelegationIntentV1 {
            run_id: request.run_id,
            candidate_message_id: candidate.message_id,
            safety_message_id: safety.message_id,
            attachment_anchor_id: request.attachment_anchor_id,
            safety_evidence_id: safety.evidence_id,
        },
    )
}

#[must_use]
pub fn attachment_text_rejection_evidence_id_v1(
    request: &AttachmentTextExtractionRequestV1,
    rejection: AttachmentTextExtractionRejectionV1,
) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-text-extraction.rejection.v1\0");
    hasher.update(request.run_id);
    hasher.update(request.attachment_anchor_id);
    hasher.update([rejection as u8]);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

fn valid_candidate(candidate: &AttachmentTextScanCandidateV1) -> bool {
    valid_identifier(&candidate.message_id)
        && valid_identifier(&candidate.attachment_anchor_id)
        && valid_identifier(&candidate.blob_reference_id)
        && (1..=ATTACHMENT_TEXT_EXTRACTION_MAX_SOURCE_BYTES_V1).contains(&candidate.declared_size)
        && valid_sha256(&candidate.blob_receipt_sha256)
        && (1..=MAX_CUSTODY_SOURCE_PROOF_BYTES_V1)
            .contains(&candidate.custody_transfer_source_proof.len())
        && valid_timestamp(candidate.observed_at_unix_seconds)
}

fn valid_terminal_safety_fact(fact: &AttachmentTextCanonicalSafetyFactV1) -> bool {
    valid_identifier(&fact.message_id)
        && valid_identifier(&fact.attachment_anchor_id)
        && valid_identifier(&fact.evidence_id)
        && valid_timestamp(fact.observed_at_unix_seconds)
        && match fact.next_state {
            AttachmentTextSafetyStateV1::SafeForDelivery => {
                fact.expected_state == AttachmentTextSafetyStateV1::BlobAdmitted
            }
            AttachmentTextSafetyStateV1::Quarantined | AttachmentTextSafetyStateV1::Rejected => {
                fact.expected_state != fact.next_state
            }
            _ => false,
        }
}

fn valid_identifier(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_timestamp(value: i64) -> bool {
    value > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_waits_for_both_facts_and_requires_exact_safe_anchor() {
        let request = request();
        let candidate = candidate();
        let safety = safety(AttachmentTextSafetyStateV1::SafeForDelivery);
        assert_eq!(
            decide_attachment_text_join_v1(&request, Some(&candidate), None),
            AttachmentTextExtractionJoinDecisionV1::Waiting
        );
        assert!(matches!(
            decide_attachment_text_join_v1(&request, Some(&candidate), Some(&safety)),
            AttachmentTextExtractionJoinDecisionV1::CustodyDelegationRequired(_)
        ));
        let mut wrong_anchor = safety;
        wrong_anchor.attachment_anchor_id = [8; 16];
        assert_eq!(
            decide_attachment_text_join_v1(&request, Some(&candidate), Some(&wrong_anchor)),
            AttachmentTextExtractionJoinDecisionV1::Reject(
                AttachmentTextExtractionRejectionV1::AnchorMismatch
            )
        );
    }

    #[test]
    fn quarantine_never_creates_custody_intent() {
        assert_eq!(
            decide_attachment_text_join_v1(
                &request(),
                Some(&candidate()),
                Some(&safety(AttachmentTextSafetyStateV1::Quarantined)),
            ),
            AttachmentTextExtractionJoinDecisionV1::Reject(
                AttachmentTextExtractionRejectionV1::NotSafe
            )
        );
    }

    #[test]
    fn replay_is_exact_and_collision_is_rejected() {
        let candidate = candidate();
        assert_eq!(
            decide_attachment_text_scan_candidate_record_v1(Some(&candidate), &candidate),
            AttachmentTextExtractionRecordDecisionV1::Duplicate
        );
        let mut collision = candidate.clone();
        collision.declared_size += 1;
        assert_eq!(
            decide_attachment_text_scan_candidate_record_v1(Some(&candidate), &collision),
            AttachmentTextExtractionRecordDecisionV1::Reject(
                AttachmentTextExtractionRejectionV1::CandidateConflict
            )
        );
    }

    #[test]
    fn candidate_bounds_fail_closed_before_custody_delegation() {
        for invalid in [
            AttachmentTextScanCandidateV1 {
                declared_size: 0,
                ..candidate()
            },
            AttachmentTextScanCandidateV1 {
                declared_size: ATTACHMENT_TEXT_EXTRACTION_MAX_SOURCE_BYTES_V1 + 1,
                ..candidate()
            },
            AttachmentTextScanCandidateV1 {
                blob_receipt_sha256: [0; 32],
                ..candidate()
            },
            AttachmentTextScanCandidateV1 {
                custody_transfer_source_proof: Vec::new(),
                ..candidate()
            },
            AttachmentTextScanCandidateV1 {
                custody_transfer_source_proof: vec![7; MAX_CUSTODY_SOURCE_PROOF_BYTES_V1 + 1],
                ..candidate()
            },
        ] {
            assert_eq!(
                decide_attachment_text_scan_candidate_record_v1(None, &invalid),
                AttachmentTextExtractionRecordDecisionV1::Reject(
                    AttachmentTextExtractionRejectionV1::InvalidCandidate
                )
            );
            assert_eq!(
                decide_attachment_text_join_v1(
                    &request(),
                    Some(&invalid),
                    Some(&safety(AttachmentTextSafetyStateV1::SafeForDelivery)),
                ),
                AttachmentTextExtractionJoinDecisionV1::Reject(
                    AttachmentTextExtractionRejectionV1::InvalidCandidate
                )
            );
        }
    }

    fn request() -> AttachmentTextExtractionRequestV1 {
        AttachmentTextExtractionRequestV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            attachment_anchor_id: [3; 16],
        }
    }

    fn candidate() -> AttachmentTextScanCandidateV1 {
        AttachmentTextScanCandidateV1 {
            message_id: [4; 16],
            attachment_anchor_id: [3; 16],
            blob_reference_id: [5; 16],
            declared_size: 42,
            blob_receipt_sha256: [6; 32],
            custody_transfer_source_proof: vec![7; 64],
            observed_at_unix_seconds: 1_800_000_000,
        }
    }

    fn safety(next_state: AttachmentTextSafetyStateV1) -> AttachmentTextCanonicalSafetyFactV1 {
        AttachmentTextCanonicalSafetyFactV1 {
            message_id: [8; 16],
            attachment_anchor_id: [3; 16],
            expected_state: AttachmentTextSafetyStateV1::BlobAdmitted,
            next_state,
            evidence_id: [9; 16],
            observed_at_unix_seconds: 1_800_000_001,
        }
    }
}
