#![forbid(unsafe_code)]

use makosh_call_transcription_api::{MAX_SEGMENTS_V1, MAX_TRANSCRIPT_BYTES_V1};
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-call-transcription-core";
pub const MAX_AUDIO_BYTES_V1: u64 = 64 * 1024 * 1024;
pub const MAX_DURATION_MILLIS_V1: u64 = 4 * 60 * 60 * 1_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionLanguageV1 {
    Auto,
    English,
    Russian,
    Spanish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionCompletenessV1 {
    Complete,
    Partial,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionStateV1 {
    Accepted,
    AwaitingRecording,
    AwaitingStt,
    MaterializingTranscript,
    Ready,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionRejectionV1 {
    RecordingRejected,
    SttRejected,
    ResultRejected,
    StaleAuthority,
    Policy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallTranscriptionDraftV1 {
    pub operation_id: [u8; 16],
    pub call_evidence_id: [u8; 16],
    pub call_evidence_revision: u64,
    pub recording_evidence_id: [u8; 16],
    pub recording_revision: u64,
    pub consent_receipt_id: [u8; 16],
    pub consent_policy_revision: u32,
    pub requested_language: CallTranscriptionLanguageV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingSourceV1 {
    pub recording_evidence_id: [u8; 16],
    pub recording_revision: u64,
    pub call_evidence_id: [u8; 16],
    pub call_evidence_revision: u64,
    pub consent_receipt_id: [u8; 16],
    pub consent_policy_revision: u32,
    pub audio_reference_id: [u8; 16],
    pub audio_sha256: [u8; 32],
    pub declared_bytes: u64,
    pub duration_millis: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingTranscriptV1 {
    pub transcript_reference_id: [u8; 16],
    pub transcript_sha256: [u8; 32],
    pub transcript_size_bytes: u64,
    pub detected_language: CallTranscriptionLanguageV1,
    pub duration_millis: u64,
    pub segment_count: u32,
    pub completeness: CallTranscriptionCompletenessV1,
    pub confidence_basis_points: u32,
    pub stt_request_digest: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptArtifactV1 {
    pub artifact_id: [u8; 16],
    pub transcript_sha256: [u8; 32],
    pub transcript_size_bytes: u64,
    pub detected_language: CallTranscriptionLanguageV1,
    pub duration_millis: u64,
    pub segment_count: u32,
    pub completeness: CallTranscriptionCompletenessV1,
    pub confidence_basis_points: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallTranscriptionStatusV1 {
    pub state: CallTranscriptionStateV1,
    pub state_revision: u64,
    pub source_sha256: Option<[u8; 32]>,
    pub stt_request_digest: Option<[u8; 32]>,
    pub pending_transcript: Option<PendingTranscriptV1>,
    pub artifact: Option<TranscriptArtifactV1>,
    pub rejection: Option<CallTranscriptionRejectionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallTranscriptionTransitionV1 {
    AwaitRecording,
    RecordingReady {
        source: RecordingSourceV1,
        stt_request_digest: [u8; 32],
    },
    SttCompleted(PendingTranscriptV1),
    TranscriptMaterialized {
        artifact_id: [u8; 16],
    },
    Reject(CallTranscriptionRejectionV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionCoreErrorV1 {
    InvalidIdentity,
    InvalidRevision,
    InvalidSource,
    SourceMismatch,
    InvalidResult,
    DigestMismatch,
    InvalidTransition,
    RevisionExhausted,
}

pub fn validate_draft_v1(
    draft: &CallTranscriptionDraftV1,
) -> Result<(), CallTranscriptionCoreErrorV1> {
    if zero(&draft.operation_id)
        || zero(&draft.call_evidence_id)
        || zero(&draft.recording_evidence_id)
        || zero(&draft.consent_receipt_id)
    {
        return Err(CallTranscriptionCoreErrorV1::InvalidIdentity);
    }
    if draft.call_evidence_revision == 0
        || draft.recording_revision == 0
        || draft.consent_policy_revision == 0
    {
        return Err(CallTranscriptionCoreErrorV1::InvalidRevision);
    }
    Ok(())
}

pub fn request_fingerprint_v1(
    draft: &CallTranscriptionDraftV1,
) -> Result<[u8; 32], CallTranscriptionCoreErrorV1> {
    validate_draft_v1(draft)?;
    let mut digest = Sha256::new();
    digest.update(b"makosh.call-transcription.request.v1\0");
    digest.update(draft.operation_id);
    digest.update(draft.call_evidence_id);
    digest.update(draft.call_evidence_revision.to_be_bytes());
    digest.update(draft.recording_evidence_id);
    digest.update(draft.recording_revision.to_be_bytes());
    digest.update(draft.consent_receipt_id);
    digest.update(draft.consent_policy_revision.to_be_bytes());
    digest.update([language_code(draft.requested_language)]);
    Ok(digest.finalize().into())
}

#[must_use]
pub fn accepted_status_v1() -> CallTranscriptionStatusV1 {
    CallTranscriptionStatusV1 {
        state: CallTranscriptionStateV1::Accepted,
        state_revision: 1,
        source_sha256: None,
        stt_request_digest: None,
        pending_transcript: None,
        artifact: None,
        rejection: None,
    }
}

pub fn transition_v1(
    draft: &CallTranscriptionDraftV1,
    current: &CallTranscriptionStatusV1,
    transition: CallTranscriptionTransitionV1,
) -> Result<CallTranscriptionStatusV1, CallTranscriptionCoreErrorV1> {
    validate_draft_v1(draft)?;
    let revision = current
        .state_revision
        .checked_add(1)
        .ok_or(CallTranscriptionCoreErrorV1::RevisionExhausted)?;
    match (current.state, transition) {
        (CallTranscriptionStateV1::Accepted, CallTranscriptionTransitionV1::AwaitRecording) => {
            Ok(CallTranscriptionStatusV1 {
                state: CallTranscriptionStateV1::AwaitingRecording,
                state_revision: revision,
                ..current.clone()
            })
        }
        (
            CallTranscriptionStateV1::AwaitingRecording,
            CallTranscriptionTransitionV1::RecordingReady {
                source,
                stt_request_digest,
            },
        ) => {
            validate_source(draft, &source)?;
            if zero(&stt_request_digest) {
                return Err(CallTranscriptionCoreErrorV1::InvalidSource);
            }
            Ok(CallTranscriptionStatusV1 {
                state: CallTranscriptionStateV1::AwaitingStt,
                state_revision: revision,
                source_sha256: Some(source.audio_sha256),
                stt_request_digest: Some(stt_request_digest),
                pending_transcript: None,
                artifact: None,
                rejection: None,
            })
        }
        (
            CallTranscriptionStateV1::AwaitingStt,
            CallTranscriptionTransitionV1::SttCompleted(result),
        ) => {
            validate_pending(&result)?;
            if current.stt_request_digest != Some(result.stt_request_digest) {
                return Err(CallTranscriptionCoreErrorV1::DigestMismatch);
            }
            Ok(CallTranscriptionStatusV1 {
                state: CallTranscriptionStateV1::MaterializingTranscript,
                state_revision: revision,
                pending_transcript: Some(result),
                artifact: None,
                rejection: None,
                ..current.clone()
            })
        }
        (
            CallTranscriptionStateV1::MaterializingTranscript,
            CallTranscriptionTransitionV1::TranscriptMaterialized { artifact_id },
        ) => {
            if zero(&artifact_id) {
                return Err(CallTranscriptionCoreErrorV1::InvalidResult);
            }
            let pending = current
                .pending_transcript
                .as_ref()
                .ok_or(CallTranscriptionCoreErrorV1::InvalidResult)?;
            Ok(CallTranscriptionStatusV1 {
                state: CallTranscriptionStateV1::Ready,
                state_revision: revision,
                pending_transcript: None,
                artifact: Some(TranscriptArtifactV1 {
                    artifact_id,
                    transcript_sha256: pending.transcript_sha256,
                    transcript_size_bytes: pending.transcript_size_bytes,
                    detected_language: pending.detected_language,
                    duration_millis: pending.duration_millis,
                    segment_count: pending.segment_count,
                    completeness: pending.completeness,
                    confidence_basis_points: pending.confidence_basis_points,
                }),
                rejection: None,
                ..current.clone()
            })
        }
        (
            CallTranscriptionStateV1::Accepted
            | CallTranscriptionStateV1::AwaitingRecording
            | CallTranscriptionStateV1::AwaitingStt
            | CallTranscriptionStateV1::MaterializingTranscript,
            CallTranscriptionTransitionV1::Reject(rejection),
        ) => Ok(CallTranscriptionStatusV1 {
            state: CallTranscriptionStateV1::Rejected,
            state_revision: revision,
            pending_transcript: None,
            artifact: None,
            rejection: Some(rejection),
            ..current.clone()
        }),
        _ => Err(CallTranscriptionCoreErrorV1::InvalidTransition),
    }
}

fn validate_source(
    draft: &CallTranscriptionDraftV1,
    source: &RecordingSourceV1,
) -> Result<(), CallTranscriptionCoreErrorV1> {
    if zero(&source.audio_reference_id)
        || zero(&source.audio_sha256)
        || source.declared_bytes == 0
        || source.declared_bytes > MAX_AUDIO_BYTES_V1
        || source.duration_millis == 0
        || source.duration_millis > MAX_DURATION_MILLIS_V1
    {
        return Err(CallTranscriptionCoreErrorV1::InvalidSource);
    }
    if source.recording_evidence_id != draft.recording_evidence_id
        || source.recording_revision != draft.recording_revision
        || source.call_evidence_id != draft.call_evidence_id
        || source.call_evidence_revision != draft.call_evidence_revision
        || source.consent_receipt_id != draft.consent_receipt_id
        || source.consent_policy_revision != draft.consent_policy_revision
    {
        return Err(CallTranscriptionCoreErrorV1::SourceMismatch);
    }
    Ok(())
}

fn validate_pending(result: &PendingTranscriptV1) -> Result<(), CallTranscriptionCoreErrorV1> {
    if zero(&result.transcript_reference_id)
        || zero(&result.transcript_sha256)
        || result.transcript_size_bytes == 0
        || result.transcript_size_bytes > MAX_TRANSCRIPT_BYTES_V1
        || result.duration_millis == 0
        || result.duration_millis > MAX_DURATION_MILLIS_V1
        || result.segment_count > MAX_SEGMENTS_V1
        || result.confidence_basis_points > 10_000
        || zero(&result.stt_request_digest)
    {
        return Err(CallTranscriptionCoreErrorV1::InvalidResult);
    }
    Ok(())
}

fn language_code(value: CallTranscriptionLanguageV1) -> u8 {
    match value {
        CallTranscriptionLanguageV1::Auto => 1,
        CallTranscriptionLanguageV1::English => 2,
        CallTranscriptionLanguageV1::Russian => 3,
        CallTranscriptionLanguageV1::Spanish => 4,
    }
}

fn zero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> CallTranscriptionDraftV1 {
        CallTranscriptionDraftV1 {
            operation_id: [1; 16],
            call_evidence_id: [2; 16],
            call_evidence_revision: 3,
            recording_evidence_id: [4; 16],
            recording_revision: 5,
            consent_receipt_id: [6; 16],
            consent_policy_revision: 7,
            requested_language: CallTranscriptionLanguageV1::Auto,
        }
    }

    fn source() -> RecordingSourceV1 {
        RecordingSourceV1 {
            recording_evidence_id: [4; 16],
            recording_revision: 5,
            call_evidence_id: [2; 16],
            call_evidence_revision: 3,
            consent_receipt_id: [6; 16],
            consent_policy_revision: 7,
            audio_reference_id: [8; 16],
            audio_sha256: [9; 32],
            declared_bytes: 32_044,
            duration_millis: 1_000,
        }
    }

    fn pending(digest: [u8; 32]) -> PendingTranscriptV1 {
        PendingTranscriptV1 {
            transcript_reference_id: [10; 16],
            transcript_sha256: [11; 32],
            transcript_size_bytes: 128,
            detected_language: CallTranscriptionLanguageV1::English,
            duration_millis: 1_000,
            segment_count: 1,
            completeness: CallTranscriptionCompletenessV1::Complete,
            confidence_basis_points: 9_000,
            stt_request_digest: digest,
        }
    }

    #[test]
    fn lifecycle_is_monotonic_and_terminal_is_immutable() {
        let draft = draft();
        let awaiting = transition_v1(
            &draft,
            &accepted_status_v1(),
            CallTranscriptionTransitionV1::AwaitRecording,
        )
        .expect("awaiting recording");
        let digest = [12; 32];
        let stt = transition_v1(
            &draft,
            &awaiting,
            CallTranscriptionTransitionV1::RecordingReady {
                source: source(),
                stt_request_digest: digest,
            },
        )
        .expect("awaiting stt");
        let materializing = transition_v1(
            &draft,
            &stt,
            CallTranscriptionTransitionV1::SttCompleted(pending(digest)),
        )
        .expect("materializing");
        let ready = transition_v1(
            &draft,
            &materializing,
            CallTranscriptionTransitionV1::TranscriptMaterialized {
                artifact_id: [13; 16],
            },
        )
        .expect("ready");
        assert_eq!(ready.state, CallTranscriptionStateV1::Ready);
        assert!(ready.artifact.is_some());
        assert_eq!(
            transition_v1(
                &draft,
                &ready,
                CallTranscriptionTransitionV1::Reject(CallTranscriptionRejectionV1::Policy),
            ),
            Err(CallTranscriptionCoreErrorV1::InvalidTransition)
        );
    }

    #[test]
    fn source_revisions_and_consent_must_match_exactly() {
        let draft = draft();
        let awaiting = transition_v1(
            &draft,
            &accepted_status_v1(),
            CallTranscriptionTransitionV1::AwaitRecording,
        )
        .expect("awaiting");
        let mut stale = source();
        stale.recording_revision += 1;
        assert_eq!(
            transition_v1(
                &draft,
                &awaiting,
                CallTranscriptionTransitionV1::RecordingReady {
                    source: stale,
                    stt_request_digest: [1; 32],
                },
            ),
            Err(CallTranscriptionCoreErrorV1::SourceMismatch)
        );
    }

    #[test]
    fn fingerprint_binds_operation_source_consent_and_language() {
        let first = request_fingerprint_v1(&draft()).expect("fingerprint");
        let mut changed = draft();
        changed.requested_language = CallTranscriptionLanguageV1::Spanish;
        assert_ne!(first, request_fingerprint_v1(&changed).expect("changed"));
    }
}
