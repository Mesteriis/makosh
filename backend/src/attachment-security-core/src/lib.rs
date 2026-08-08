//! Pure Attachment Security join and verdict policy.

mod join;
mod verdict;

pub use join::{
    AttachmentSecurityCanonicalStateFactV1, AttachmentSecurityJoinDecisionV1,
    AttachmentSecurityJoinPolicyErrorV1, AttachmentSecurityJoinPolicyV1,
    AttachmentSecurityQuarantineEvidenceV1, AttachmentSecurityQuarantineReasonV1,
    AttachmentSecurityRecordDecisionV1, AttachmentSecurityScanCandidateV1,
    AttachmentSecurityScanJobV1, CanonicalAttachmentSafetyStateV1,
    attachment_security_quarantine_evidence_v1, decide_candidate_record_v1,
    decide_canonical_state_record_v1, decide_scan_join_v1,
};
pub use verdict::{
    AttachmentSecurityVerdictDecisionV1, AttachmentSecurityVerdictErrorV1,
    AttachmentSecurityVerdictV1, ScannerOutcomeV1, decide_attachment_security_verdict_v1,
};

pub const PACKAGE: &str = "makosh-attachment-security-core";
