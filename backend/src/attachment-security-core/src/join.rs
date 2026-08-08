use makosh_attachment_security_contract::ATTACHMENT_SECURITY_MAX_SCAN_CANDIDATE_BYTES_V1;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentSecurityJoinPolicyV1 {
    max_scan_bytes: u64,
}

impl AttachmentSecurityJoinPolicyV1 {
    pub fn new(max_scan_bytes: u64) -> Result<Self, AttachmentSecurityJoinPolicyErrorV1> {
        if max_scan_bytes == 0 || max_scan_bytes > ATTACHMENT_SECURITY_MAX_SCAN_CANDIDATE_BYTES_V1 {
            return Err(AttachmentSecurityJoinPolicyErrorV1::InvalidMaximum);
        }
        Ok(Self { max_scan_bytes })
    }

    #[must_use]
    pub const fn max_scan_bytes(self) -> u64 {
        self.max_scan_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityJoinPolicyErrorV1 {
    InvalidMaximum,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalAttachmentSafetyStateV1 {
    DescriptorOnly,
    BlobPending,
    BlobAdmitted,
    Quarantined,
    SafeForDelivery,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentSecurityScanCandidateV1 {
    pub message_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub blob_reference_id: [u8; 16],
    pub declared_size: u64,
    pub blob_receipt_sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
    pub causation_message_id: [u8; 16],
    pub correlation_id: [u8; 16],
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentSecurityCanonicalStateFactV1 {
    pub message_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub expected_state: CanonicalAttachmentSafetyStateV1,
    pub next_state: CanonicalAttachmentSafetyStateV1,
    pub evidence_id: [u8; 16],
    pub correlation_id: [u8; 16],
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityQuarantineReasonV1 {
    InvalidCandidate,
    InvalidCanonicalState,
    CandidateConflict,
    CanonicalStateConflict,
    AnchorMismatch,
    CorrelationMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentSecurityQuarantineEvidenceV1 {
    pub evidence_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub correlation_id: [u8; 16],
    pub reason: AttachmentSecurityQuarantineReasonV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentSecurityScanJobV1 {
    pub candidate_message_id: [u8; 16],
    pub canonical_state_message_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub blob_reference_id: [u8; 16],
    pub declared_size: u64,
    pub blob_receipt_sha256: [u8; 32],
    pub causation_message_id: [u8; 16],
    pub correlation_id: [u8; 16],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityRecordDecisionV1 {
    Insert,
    Duplicate,
    Quarantine(AttachmentSecurityQuarantineEvidenceV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityJoinDecisionV1 {
    Waiting,
    Runnable(AttachmentSecurityScanJobV1),
    Quarantine(AttachmentSecurityQuarantineEvidenceV1),
}

#[must_use]
pub fn decide_candidate_record_v1(
    existing: Option<&AttachmentSecurityScanCandidateV1>,
    incoming: &AttachmentSecurityScanCandidateV1,
    policy: AttachmentSecurityJoinPolicyV1,
) -> AttachmentSecurityRecordDecisionV1 {
    if !valid_candidate(incoming, policy) {
        return AttachmentSecurityRecordDecisionV1::Quarantine(
            attachment_security_quarantine_evidence_v1(
                incoming.attachment_anchor_id,
                incoming.correlation_id,
                AttachmentSecurityQuarantineReasonV1::InvalidCandidate,
            ),
        );
    }
    match existing {
        None => AttachmentSecurityRecordDecisionV1::Insert,
        Some(current) if current == incoming => AttachmentSecurityRecordDecisionV1::Duplicate,
        Some(_) => AttachmentSecurityRecordDecisionV1::Quarantine(
            attachment_security_quarantine_evidence_v1(
                incoming.attachment_anchor_id,
                incoming.correlation_id,
                AttachmentSecurityQuarantineReasonV1::CandidateConflict,
            ),
        ),
    }
}

#[must_use]
pub fn decide_canonical_state_record_v1(
    existing: Option<&AttachmentSecurityCanonicalStateFactV1>,
    incoming: &AttachmentSecurityCanonicalStateFactV1,
) -> AttachmentSecurityRecordDecisionV1 {
    if !valid_canonical_state(incoming) {
        return AttachmentSecurityRecordDecisionV1::Quarantine(
            attachment_security_quarantine_evidence_v1(
                incoming.attachment_anchor_id,
                incoming.correlation_id,
                AttachmentSecurityQuarantineReasonV1::InvalidCanonicalState,
            ),
        );
    }
    match existing {
        None => AttachmentSecurityRecordDecisionV1::Insert,
        Some(current) if current == incoming => AttachmentSecurityRecordDecisionV1::Duplicate,
        Some(_) => AttachmentSecurityRecordDecisionV1::Quarantine(
            attachment_security_quarantine_evidence_v1(
                incoming.attachment_anchor_id,
                incoming.correlation_id,
                AttachmentSecurityQuarantineReasonV1::CanonicalStateConflict,
            ),
        ),
    }
}

#[must_use]
pub fn decide_scan_join_v1(
    candidate: Option<&AttachmentSecurityScanCandidateV1>,
    canonical_state: Option<&AttachmentSecurityCanonicalStateFactV1>,
    policy: AttachmentSecurityJoinPolicyV1,
) -> AttachmentSecurityJoinDecisionV1 {
    if let Some(value) = candidate
        && !valid_candidate(value, policy)
    {
        return AttachmentSecurityJoinDecisionV1::Quarantine(
            attachment_security_quarantine_evidence_v1(
                value.attachment_anchor_id,
                value.correlation_id,
                AttachmentSecurityQuarantineReasonV1::InvalidCandidate,
            ),
        );
    }
    if let Some(value) = canonical_state
        && !valid_canonical_state(value)
    {
        return AttachmentSecurityJoinDecisionV1::Quarantine(
            attachment_security_quarantine_evidence_v1(
                value.attachment_anchor_id,
                value.correlation_id,
                AttachmentSecurityQuarantineReasonV1::InvalidCanonicalState,
            ),
        );
    }
    let (Some(candidate), Some(canonical_state)) = (candidate, canonical_state) else {
        return AttachmentSecurityJoinDecisionV1::Waiting;
    };
    if candidate.attachment_anchor_id != canonical_state.attachment_anchor_id {
        return AttachmentSecurityJoinDecisionV1::Quarantine(
            attachment_security_quarantine_evidence_v1(
                candidate.attachment_anchor_id,
                candidate.correlation_id,
                AttachmentSecurityQuarantineReasonV1::AnchorMismatch,
            ),
        );
    }
    if candidate.correlation_id != canonical_state.correlation_id {
        return AttachmentSecurityJoinDecisionV1::Quarantine(
            attachment_security_quarantine_evidence_v1(
                candidate.attachment_anchor_id,
                candidate.correlation_id,
                AttachmentSecurityQuarantineReasonV1::CorrelationMismatch,
            ),
        );
    }

    AttachmentSecurityJoinDecisionV1::Runnable(AttachmentSecurityScanJobV1 {
        candidate_message_id: candidate.message_id,
        canonical_state_message_id: canonical_state.message_id,
        attachment_anchor_id: candidate.attachment_anchor_id,
        blob_reference_id: candidate.blob_reference_id,
        declared_size: candidate.declared_size,
        blob_receipt_sha256: candidate.blob_receipt_sha256,
        causation_message_id: canonical_state.message_id,
        correlation_id: candidate.correlation_id,
    })
}

fn valid_candidate(
    candidate: &AttachmentSecurityScanCandidateV1,
    policy: AttachmentSecurityJoinPolicyV1,
) -> bool {
    valid_identifier(&candidate.message_id)
        && valid_identifier(&candidate.attachment_anchor_id)
        && valid_identifier(&candidate.blob_reference_id)
        && candidate.declared_size > 0
        && candidate.declared_size <= policy.max_scan_bytes
        && valid_sha256(&candidate.blob_receipt_sha256)
        && (1..=2_048).contains(&candidate.custody_transfer_source_proof.len())
        && valid_identifier(&candidate.causation_message_id)
        && valid_identifier(&candidate.correlation_id)
        && valid_timestamp(candidate.observed_at_unix_seconds)
}

fn valid_canonical_state(state: &AttachmentSecurityCanonicalStateFactV1) -> bool {
    valid_identifier(&state.message_id)
        && valid_identifier(&state.attachment_anchor_id)
        && state.expected_state == CanonicalAttachmentSafetyStateV1::BlobPending
        && state.next_state == CanonicalAttachmentSafetyStateV1::BlobAdmitted
        && valid_identifier(&state.evidence_id)
        && valid_identifier(&state.correlation_id)
        && valid_timestamp(state.observed_at_unix_seconds)
}

#[must_use]
pub fn attachment_security_quarantine_evidence_v1(
    attachment_anchor_id: [u8; 16],
    correlation_id: [u8; 16],
    reason: AttachmentSecurityQuarantineReasonV1,
) -> AttachmentSecurityQuarantineEvidenceV1 {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-security.join-quarantine.v1\0");
    hasher.update(attachment_anchor_id);
    hasher.update(correlation_id);
    hasher.update([quarantine_reason_value(reason)]);
    let digest: [u8; 32] = hasher.finalize().into();
    AttachmentSecurityQuarantineEvidenceV1 {
        evidence_id: digest[..16]
            .try_into()
            .expect("fixed SHA-256 prefix length"),
        attachment_anchor_id,
        correlation_id,
        reason,
    }
}

const fn quarantine_reason_value(reason: AttachmentSecurityQuarantineReasonV1) -> u8 {
    match reason {
        AttachmentSecurityQuarantineReasonV1::InvalidCandidate => 1,
        AttachmentSecurityQuarantineReasonV1::InvalidCanonicalState => 2,
        AttachmentSecurityQuarantineReasonV1::CandidateConflict => 3,
        AttachmentSecurityQuarantineReasonV1::CanonicalStateConflict => 4,
        AttachmentSecurityQuarantineReasonV1::AnchorMismatch => 5,
        AttachmentSecurityQuarantineReasonV1::CorrelationMismatch => 6,
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
    fn scan_becomes_runnable_only_after_an_order_independent_exact_join() {
        let policy = policy();
        let candidate = candidate();
        let canonical = canonical();

        assert_eq!(
            decide_scan_join_v1(Some(&candidate), None, policy),
            AttachmentSecurityJoinDecisionV1::Waiting
        );
        assert_eq!(
            decide_scan_join_v1(None, Some(&canonical), policy),
            AttachmentSecurityJoinDecisionV1::Waiting
        );
        let AttachmentSecurityJoinDecisionV1::Runnable(job) =
            decide_scan_join_v1(Some(&candidate), Some(&canonical), policy)
        else {
            panic!("runnable");
        };
        assert_eq!(job.attachment_anchor_id, candidate.attachment_anchor_id);
        assert_eq!(job.causation_message_id, canonical.message_id);
        assert_eq!(job.correlation_id, candidate.correlation_id);
    }

    #[test]
    fn mismatched_correlation_and_non_admission_transition_quarantine_without_scan() {
        let candidate = candidate();
        let mut correlation_mismatch = canonical();
        correlation_mismatch.correlation_id = [9; 16];
        assert!(matches!(
            decide_scan_join_v1(Some(&candidate), Some(&correlation_mismatch), policy()),
            AttachmentSecurityJoinDecisionV1::Quarantine(AttachmentSecurityQuarantineEvidenceV1 {
                reason: AttachmentSecurityQuarantineReasonV1::CorrelationMismatch,
                ..
            })
        ));

        let mut wrong_transition = canonical();
        wrong_transition.expected_state = CanonicalAttachmentSafetyStateV1::DescriptorOnly;
        assert!(matches!(
            decide_scan_join_v1(Some(&candidate), Some(&wrong_transition), policy()),
            AttachmentSecurityJoinDecisionV1::Quarantine(AttachmentSecurityQuarantineEvidenceV1 {
                reason: AttachmentSecurityQuarantineReasonV1::InvalidCanonicalState,
                ..
            })
        ));
    }

    #[test]
    fn exact_record_is_idempotent_and_changed_blob_identity_is_quarantined() {
        let candidate = candidate();
        assert_eq!(
            decide_candidate_record_v1(Some(&candidate), &candidate, policy()),
            AttachmentSecurityRecordDecisionV1::Duplicate
        );
        let mut changed = candidate.clone();
        changed.blob_reference_id = [8; 16];
        assert!(matches!(
            decide_candidate_record_v1(Some(&candidate), &changed, policy()),
            AttachmentSecurityRecordDecisionV1::Quarantine(
                AttachmentSecurityQuarantineEvidenceV1 {
                    reason: AttachmentSecurityQuarantineReasonV1::CandidateConflict,
                    ..
                }
            )
        ));
    }

    #[test]
    fn payload_cannot_expand_the_hard_scan_limit() {
        let mut oversized = candidate();
        oversized.declared_size = policy().max_scan_bytes() + 1;
        assert!(matches!(
            decide_scan_join_v1(Some(&oversized), Some(&canonical()), policy()),
            AttachmentSecurityJoinDecisionV1::Quarantine(AttachmentSecurityQuarantineEvidenceV1 {
                reason: AttachmentSecurityQuarantineReasonV1::InvalidCandidate,
                ..
            })
        ));
    }

    #[test]
    fn quarantine_evidence_is_deterministic_and_reason_scoped() {
        let first = attachment_security_quarantine_evidence_v1(
            [2; 16],
            [6; 16],
            AttachmentSecurityQuarantineReasonV1::CandidateConflict,
        );
        let duplicate = attachment_security_quarantine_evidence_v1(
            [2; 16],
            [6; 16],
            AttachmentSecurityQuarantineReasonV1::CandidateConflict,
        );
        let other_reason = attachment_security_quarantine_evidence_v1(
            [2; 16],
            [6; 16],
            AttachmentSecurityQuarantineReasonV1::CanonicalStateConflict,
        );

        assert_eq!(first, duplicate);
        assert_ne!(first.evidence_id, other_reason.evidence_id);
    }

    fn policy() -> AttachmentSecurityJoinPolicyV1 {
        AttachmentSecurityJoinPolicyV1::new(1024).expect("policy")
    }

    fn candidate() -> AttachmentSecurityScanCandidateV1 {
        AttachmentSecurityScanCandidateV1 {
            message_id: [1; 16],
            attachment_anchor_id: [2; 16],
            blob_reference_id: [3; 16],
            declared_size: 42,
            blob_receipt_sha256: [4; 32],
            custody_transfer_source_proof: vec![9; 64],
            causation_message_id: [5; 16],
            correlation_id: [6; 16],
            observed_at_unix_seconds: 1_700_000_000,
        }
    }

    fn canonical() -> AttachmentSecurityCanonicalStateFactV1 {
        AttachmentSecurityCanonicalStateFactV1 {
            message_id: [7; 16],
            attachment_anchor_id: [2; 16],
            expected_state: CanonicalAttachmentSafetyStateV1::BlobPending,
            next_state: CanonicalAttachmentSafetyStateV1::BlobAdmitted,
            evidence_id: [8; 16],
            correlation_id: [6; 16],
            observed_at_unix_seconds: 1_700_000_001,
        }
    }
}
