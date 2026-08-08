use sha2::{Digest, Sha256};

use crate::DEFAULT_MAX_ARCHIVE_BYTES_V1;

const MAX_CUSTODY_SOURCE_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveInspectionRequestV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveInspectionScanCandidateV1 {
    pub message_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub blob_reference_id: [u8; 16],
    pub declared_size: u64,
    pub blob_receipt_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionSafetyStateV1 {
    DescriptorOnly,
    BlobPending,
    BlobAdmitted,
    Quarantined,
    SafeForDelivery,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveInspectionCanonicalSafetyFactV1 {
    pub message_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub expected_state: ArchiveInspectionSafetyStateV1,
    pub next_state: ArchiveInspectionSafetyStateV1,
    pub evidence_id: [u8; 16],
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveInspectionCustodyDelegationIntentV1 {
    pub run_id: [u8; 16],
    pub candidate_message_id: [u8; 16],
    pub safety_message_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub safety_evidence_id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionRejectionV1 {
    InvalidRequest,
    InvalidCandidate,
    InvalidSafetyState,
    CandidateConflict,
    SafetyStateConflict,
    AnchorMismatch,
    NotSafe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionRecordDecisionV1 {
    Insert,
    Duplicate,
    Reject(ArchiveInspectionRejectionV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArchiveInspectionJoinDecisionV1 {
    Waiting,
    CustodyDelegationRequired(ArchiveInspectionCustodyDelegationIntentV1),
    Reject(ArchiveInspectionRejectionV1),
}

#[must_use]
pub fn validate_archive_inspection_request_v1(request: &ArchiveInspectionRequestV1) -> bool {
    valid_identifier(&request.run_id)
        && valid_identifier(&request.operation_id)
        && valid_identifier(&request.attachment_anchor_id)
}

#[must_use]
pub fn decide_archive_scan_candidate_record_v1(
    existing: Option<&ArchiveInspectionScanCandidateV1>,
    incoming: &ArchiveInspectionScanCandidateV1,
) -> ArchiveInspectionRecordDecisionV1 {
    if !valid_candidate(incoming) {
        return ArchiveInspectionRecordDecisionV1::Reject(
            ArchiveInspectionRejectionV1::InvalidCandidate,
        );
    }
    match existing {
        None => ArchiveInspectionRecordDecisionV1::Insert,
        Some(current) if current == incoming => ArchiveInspectionRecordDecisionV1::Duplicate,
        Some(_) => ArchiveInspectionRecordDecisionV1::Reject(
            ArchiveInspectionRejectionV1::CandidateConflict,
        ),
    }
}

#[must_use]
pub fn decide_archive_inspection_safety_record_v1(
    existing: Option<&ArchiveInspectionCanonicalSafetyFactV1>,
    incoming: &ArchiveInspectionCanonicalSafetyFactV1,
) -> ArchiveInspectionRecordDecisionV1 {
    if !valid_terminal_safety_fact(incoming) {
        return ArchiveInspectionRecordDecisionV1::Reject(
            ArchiveInspectionRejectionV1::InvalidSafetyState,
        );
    }
    match existing {
        None => ArchiveInspectionRecordDecisionV1::Insert,
        Some(current) if current == incoming => ArchiveInspectionRecordDecisionV1::Duplicate,
        Some(_) => ArchiveInspectionRecordDecisionV1::Reject(
            ArchiveInspectionRejectionV1::SafetyStateConflict,
        ),
    }
}

#[must_use]
pub fn decide_archive_inspection_join_v1(
    request: &ArchiveInspectionRequestV1,
    candidate: Option<&ArchiveInspectionScanCandidateV1>,
    safety: Option<&ArchiveInspectionCanonicalSafetyFactV1>,
) -> ArchiveInspectionJoinDecisionV1 {
    if !validate_archive_inspection_request_v1(request) {
        return ArchiveInspectionJoinDecisionV1::Reject(
            ArchiveInspectionRejectionV1::InvalidRequest,
        );
    }
    if let Some(value) = candidate
        && !valid_candidate(value)
    {
        return ArchiveInspectionJoinDecisionV1::Reject(
            ArchiveInspectionRejectionV1::InvalidCandidate,
        );
    }
    if let Some(value) = safety
        && !valid_terminal_safety_fact(value)
    {
        return ArchiveInspectionJoinDecisionV1::Reject(
            ArchiveInspectionRejectionV1::InvalidSafetyState,
        );
    }
    if let Some(candidate) = candidate
        && request.attachment_anchor_id != candidate.attachment_anchor_id
    {
        return ArchiveInspectionJoinDecisionV1::Reject(
            ArchiveInspectionRejectionV1::AnchorMismatch,
        );
    }
    if let Some(safety) = safety {
        if request.attachment_anchor_id != safety.attachment_anchor_id {
            return ArchiveInspectionJoinDecisionV1::Reject(
                ArchiveInspectionRejectionV1::AnchorMismatch,
            );
        }
        if matches!(
            safety.next_state,
            ArchiveInspectionSafetyStateV1::Quarantined | ArchiveInspectionSafetyStateV1::Rejected
        ) {
            return ArchiveInspectionJoinDecisionV1::Reject(ArchiveInspectionRejectionV1::NotSafe);
        }
    }
    let (Some(candidate), Some(safety)) = (candidate, safety) else {
        return ArchiveInspectionJoinDecisionV1::Waiting;
    };

    ArchiveInspectionJoinDecisionV1::CustodyDelegationRequired(
        ArchiveInspectionCustodyDelegationIntentV1 {
            run_id: request.run_id,
            candidate_message_id: candidate.message_id,
            safety_message_id: safety.message_id,
            attachment_anchor_id: request.attachment_anchor_id,
            safety_evidence_id: safety.evidence_id,
        },
    )
}

#[must_use]
pub fn archive_inspection_rejection_evidence_id_v1(
    request: &ArchiveInspectionRequestV1,
    rejection: ArchiveInspectionRejectionV1,
) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-archive-inspection.rejection.v1\0");
    hasher.update(request.run_id);
    hasher.update(request.attachment_anchor_id);
    hasher.update([rejection_code(rejection)]);
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16]
        .try_into()
        .expect("fixed SHA-256 prefix length")
}

fn valid_candidate(candidate: &ArchiveInspectionScanCandidateV1) -> bool {
    valid_identifier(&candidate.message_id)
        && valid_identifier(&candidate.attachment_anchor_id)
        && valid_identifier(&candidate.blob_reference_id)
        && (1..=DEFAULT_MAX_ARCHIVE_BYTES_V1).contains(&candidate.declared_size)
        && valid_sha256(&candidate.blob_receipt_sha256)
        && (1..=MAX_CUSTODY_SOURCE_PROOF_BYTES_V1)
            .contains(&candidate.custody_transfer_source_proof.len())
        && valid_timestamp(candidate.observed_at_unix_seconds)
}

fn valid_terminal_safety_fact(fact: &ArchiveInspectionCanonicalSafetyFactV1) -> bool {
    valid_identifier(&fact.message_id)
        && valid_identifier(&fact.attachment_anchor_id)
        && valid_identifier(&fact.evidence_id)
        && valid_timestamp(fact.observed_at_unix_seconds)
        && match fact.next_state {
            ArchiveInspectionSafetyStateV1::SafeForDelivery => {
                fact.expected_state == ArchiveInspectionSafetyStateV1::BlobAdmitted
            }
            ArchiveInspectionSafetyStateV1::Quarantined
            | ArchiveInspectionSafetyStateV1::Rejected => {
                fact.expected_state != fact.next_state
                    && fact.expected_state != ArchiveInspectionSafetyStateV1::SafeForDelivery
            }
            ArchiveInspectionSafetyStateV1::DescriptorOnly
            | ArchiveInspectionSafetyStateV1::BlobPending
            | ArchiveInspectionSafetyStateV1::BlobAdmitted => false,
        }
}

const fn rejection_code(rejection: ArchiveInspectionRejectionV1) -> u8 {
    match rejection {
        ArchiveInspectionRejectionV1::InvalidRequest => 1,
        ArchiveInspectionRejectionV1::InvalidCandidate => 2,
        ArchiveInspectionRejectionV1::InvalidSafetyState => 3,
        ArchiveInspectionRejectionV1::CandidateConflict => 4,
        ArchiveInspectionRejectionV1::SafetyStateConflict => 5,
        ArchiveInspectionRejectionV1::AnchorMismatch => 6,
        ArchiveInspectionRejectionV1::NotSafe => 7,
    }
}

fn valid_identifier(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_timestamp(seconds: i64) -> bool {
    (-62_135_596_800..=253_402_300_799).contains(&seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_three_way_join_is_order_independent_and_safe_only() {
        let request = request();
        let candidate = candidate();
        let safety = safety(
            ArchiveInspectionSafetyStateV1::BlobAdmitted,
            ArchiveInspectionSafetyStateV1::SafeForDelivery,
        );

        assert_eq!(
            decide_archive_inspection_join_v1(&request, None, None),
            ArchiveInspectionJoinDecisionV1::Waiting
        );
        assert_eq!(
            decide_archive_inspection_join_v1(&request, Some(&candidate), None),
            ArchiveInspectionJoinDecisionV1::Waiting
        );
        assert_eq!(
            decide_archive_inspection_join_v1(&request, None, Some(&safety)),
            ArchiveInspectionJoinDecisionV1::Waiting
        );
        let ArchiveInspectionJoinDecisionV1::CustodyDelegationRequired(intent) =
            decide_archive_inspection_join_v1(&request, Some(&candidate), Some(&safety))
        else {
            panic!("safe evidence must require custody delegation");
        };
        assert_eq!(intent.run_id, request.run_id);
        assert_eq!(intent.candidate_message_id, candidate.message_id);
        assert_eq!(intent.safety_evidence_id, safety.evidence_id);
    }

    #[test]
    fn unsafe_terminal_state_rejects_without_blob_work() {
        for terminal in [
            ArchiveInspectionSafetyStateV1::Quarantined,
            ArchiveInspectionSafetyStateV1::Rejected,
        ] {
            assert_eq!(
                decide_archive_inspection_join_v1(
                    &request(),
                    None,
                    Some(&safety(
                        ArchiveInspectionSafetyStateV1::BlobAdmitted,
                        terminal,
                    )),
                ),
                ArchiveInspectionJoinDecisionV1::Reject(ArchiveInspectionRejectionV1::NotSafe)
            );
            assert_eq!(
                decide_archive_inspection_join_v1(
                    &request(),
                    Some(&candidate()),
                    Some(&safety(
                        ArchiveInspectionSafetyStateV1::BlobAdmitted,
                        terminal,
                    )),
                ),
                ArchiveInspectionJoinDecisionV1::Reject(ArchiveInspectionRejectionV1::NotSafe)
            );
        }
    }

    #[test]
    fn invalid_or_conflicting_facts_fail_closed() {
        let candidate = candidate();
        let mut changed_candidate = candidate.clone();
        changed_candidate.blob_reference_id = [9; 16];
        assert_eq!(
            decide_archive_scan_candidate_record_v1(Some(&candidate), &changed_candidate),
            ArchiveInspectionRecordDecisionV1::Reject(
                ArchiveInspectionRejectionV1::CandidateConflict
            )
        );

        let safety = safety(
            ArchiveInspectionSafetyStateV1::BlobAdmitted,
            ArchiveInspectionSafetyStateV1::SafeForDelivery,
        );
        let mut changed_safety = safety;
        changed_safety.evidence_id = [8; 16];
        assert_eq!(
            decide_archive_inspection_safety_record_v1(Some(&safety), &changed_safety),
            ArchiveInspectionRecordDecisionV1::Reject(
                ArchiveInspectionRejectionV1::SafetyStateConflict
            )
        );

        let mut mismatched = candidate;
        mismatched.attachment_anchor_id = [7; 16];
        assert_eq!(
            decide_archive_inspection_join_v1(&request(), Some(&mismatched), Some(&safety)),
            ArchiveInspectionJoinDecisionV1::Reject(ArchiveInspectionRejectionV1::AnchorMismatch)
        );
    }

    #[test]
    fn candidate_cannot_expand_archive_or_custody_bounds() {
        let mut oversized = candidate();
        oversized.declared_size = DEFAULT_MAX_ARCHIVE_BYTES_V1 + 1;
        assert_eq!(
            decide_archive_scan_candidate_record_v1(None, &oversized),
            ArchiveInspectionRecordDecisionV1::Reject(
                ArchiveInspectionRejectionV1::InvalidCandidate
            )
        );
        oversized = candidate();
        oversized.custody_transfer_source_proof = vec![1; MAX_CUSTODY_SOURCE_PROOF_BYTES_V1 + 1];
        assert_eq!(
            decide_archive_scan_candidate_record_v1(None, &oversized),
            ArchiveInspectionRecordDecisionV1::Reject(
                ArchiveInspectionRejectionV1::InvalidCandidate
            )
        );
    }

    #[test]
    fn rejection_evidence_is_deterministic_and_reason_scoped() {
        let request = request();
        let first = archive_inspection_rejection_evidence_id_v1(
            &request,
            ArchiveInspectionRejectionV1::CandidateConflict,
        );
        let replay = archive_inspection_rejection_evidence_id_v1(
            &request,
            ArchiveInspectionRejectionV1::CandidateConflict,
        );
        let other = archive_inspection_rejection_evidence_id_v1(
            &request,
            ArchiveInspectionRejectionV1::SafetyStateConflict,
        );
        assert_eq!(first, replay);
        assert_ne!(first, other);
    }

    fn request() -> ArchiveInspectionRequestV1 {
        ArchiveInspectionRequestV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            attachment_anchor_id: [3; 16],
        }
    }

    fn candidate() -> ArchiveInspectionScanCandidateV1 {
        ArchiveInspectionScanCandidateV1 {
            message_id: [4; 16],
            attachment_anchor_id: [3; 16],
            blob_reference_id: [5; 16],
            declared_size: 512,
            blob_receipt_sha256: [6; 32],
            custody_transfer_source_proof: vec![7; 64],
            observed_at_unix_seconds: 1_700_000_000,
        }
    }

    fn safety(
        expected_state: ArchiveInspectionSafetyStateV1,
        next_state: ArchiveInspectionSafetyStateV1,
    ) -> ArchiveInspectionCanonicalSafetyFactV1 {
        ArchiveInspectionCanonicalSafetyFactV1 {
            message_id: [8; 16],
            attachment_anchor_id: [3; 16],
            expected_state,
            next_state,
            evidence_id: [9; 16],
            observed_at_unix_seconds: 1_700_000_001,
        }
    }
}
