use sha2::{Digest, Sha256};

use crate::AttachmentSecurityScanJobV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScannerOutcomeV1 {
    Clean,
    ThreatFound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityVerdictV1 {
    SafeForDelivery,
    Quarantined,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentSecurityVerdictDecisionV1 {
    pub attachment_anchor_id: [u8; 16],
    pub verdict: AttachmentSecurityVerdictV1,
    pub evidence_id: [u8; 16],
    pub causation_message_id: [u8; 16],
    pub correlation_id: [u8; 16],
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityVerdictErrorV1 {
    InvalidObservedTime,
}

pub fn decide_attachment_security_verdict_v1(
    job: &AttachmentSecurityScanJobV1,
    scanner_outcome: ScannerOutcomeV1,
    observed_at_unix_seconds: i64,
) -> Result<AttachmentSecurityVerdictDecisionV1, AttachmentSecurityVerdictErrorV1> {
    if !(-62_135_596_800..=253_402_300_799).contains(&observed_at_unix_seconds) {
        return Err(AttachmentSecurityVerdictErrorV1::InvalidObservedTime);
    }
    let verdict = match scanner_outcome {
        ScannerOutcomeV1::Clean => AttachmentSecurityVerdictV1::SafeForDelivery,
        ScannerOutcomeV1::ThreatFound => AttachmentSecurityVerdictV1::Quarantined,
    };
    Ok(AttachmentSecurityVerdictDecisionV1 {
        attachment_anchor_id: job.attachment_anchor_id,
        verdict,
        evidence_id: verdict_evidence_id(job, verdict),
        causation_message_id: job.causation_message_id,
        correlation_id: job.correlation_id,
        observed_at_unix_seconds,
    })
}

fn verdict_evidence_id(
    job: &AttachmentSecurityScanJobV1,
    verdict: AttachmentSecurityVerdictV1,
) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-security.closed-verdict.v1\0");
    hasher.update(job.candidate_message_id);
    hasher.update(job.canonical_state_message_id);
    hasher.update(job.blob_receipt_sha256);
    hasher.update([verdict_value(verdict)]);
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16]
        .try_into()
        .expect("fixed SHA-256 prefix length")
}

const fn verdict_value(verdict: AttachmentSecurityVerdictV1) -> u8 {
    match verdict {
        AttachmentSecurityVerdictV1::SafeForDelivery => 1,
        AttachmentSecurityVerdictV1::Quarantined => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_scanner_outcomes_map_only_to_clean_or_quarantine() {
        let job = job();
        let clean =
            decide_attachment_security_verdict_v1(&job, ScannerOutcomeV1::Clean, 1_700_000_002)
                .expect("clean");
        let threat = decide_attachment_security_verdict_v1(
            &job,
            ScannerOutcomeV1::ThreatFound,
            1_700_000_002,
        )
        .expect("threat");

        assert_eq!(clean.verdict, AttachmentSecurityVerdictV1::SafeForDelivery);
        assert_eq!(threat.verdict, AttachmentSecurityVerdictV1::Quarantined);
        assert_ne!(clean.evidence_id, threat.evidence_id);
        assert_eq!(clean.causation_message_id, job.causation_message_id);
    }

    fn job() -> AttachmentSecurityScanJobV1 {
        AttachmentSecurityScanJobV1 {
            candidate_message_id: [1; 16],
            canonical_state_message_id: [2; 16],
            attachment_anchor_id: [3; 16],
            blob_reference_id: [4; 16],
            declared_size: 42,
            blob_receipt_sha256: [5; 32],
            causation_message_id: [2; 16],
            correlation_id: [6; 16],
        }
    }
}
