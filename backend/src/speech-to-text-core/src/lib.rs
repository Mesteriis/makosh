#![forbid(unsafe_code)]

mod lifecycle;

pub use lifecycle::{
    SpeechToTextCoreErrorV1, SpeechToTextRunStateV1, SpeechToTextRunV1, accept_speech_to_text_v1,
    begin_speech_to_text_v1, complete_speech_to_text_v1, reject_speech_to_text_v1,
    validate_speech_to_text_run_v1,
};

pub const PACKAGE: &str = "makosh-speech-to-text-core";
pub const SPEECH_TO_TEXT_MAX_AUDIO_BYTES_V1: u64 = 512 * 1024 * 1024;
pub const SPEECH_TO_TEXT_MAX_DURATION_MILLIS_V1: u64 = 4 * 60 * 60 * 1_000;
pub const SPEECH_TO_TEXT_MAX_TRANSCRIPT_BYTES_V1: u32 = 4 * 1024 * 1024;
pub const SPEECH_TO_TEXT_MAX_SEGMENTS_V1: u32 = 100_000;
pub const SPEECH_TO_TEXT_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechAudioFormatV1 {
    WavPcmS16LeMono16Khz,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechLanguageV1 {
    Auto,
    English,
    Russian,
    Spanish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechTranscriptCompletenessV1 {
    Complete,
    Partial,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechToTextRejectionV1 {
    InvalidRequest,
    ConsentRejected,
    UnsupportedAudio,
    ProviderUnavailable,
    ProviderRejected,
    Policy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpeechBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpeechToTextRequestV1 {
    pub request_id: [u8; 16],
    pub logical_owner_id: String,
    pub source: SpeechBlobReceiptV1,
    pub audio_format: SpeechAudioFormatV1,
    pub duration_millis: u64,
    pub requested_language: SpeechLanguageV1,
    pub consent_receipt_id: [u8; 16],
    pub consent_policy_revision: u32,
    pub maximum_transcript_bytes: u32,
    pub maximum_segments: u32,
    pub request_digest: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpeechToTextExecutionReceiptV1 {
    pub provider_contract_schema_sha256: [u8; 32],
    pub model_revision_sha256: [u8; 32],
    pub provider_settings_revision: u64,
    pub provider_policy_revision: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpeechTranscriptArtifactV1 {
    pub receipt: SpeechBlobReceiptV1,
    pub detected_language: SpeechLanguageV1,
    pub segment_count: u32,
    pub completeness: SpeechTranscriptCompletenessV1,
    pub confidence_basis_points: u32,
    pub execution_receipt: SpeechToTextExecutionReceiptV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpeechToTextTerminalV1 {
    Ready(SpeechTranscriptArtifactV1),
    Rejected(SpeechToTextRejectionV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpeechToTextResultV1 {
    pub request_id: [u8; 16],
    pub request_digest: [u8; 32],
    pub source_sha256: [u8; 32],
    pub terminal: SpeechToTextTerminalV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechToTextValidationErrorV1 {
    InvalidIdentity,
    InvalidOwner,
    InvalidAudio,
    InvalidConsent,
    InvalidLimits,
    InvalidDigest,
    InvalidArtifact,
    InvalidExecutionReceipt,
    RequestResultMismatch,
}

pub fn validate_speech_to_text_request_v1(
    request: &SpeechToTextRequestV1,
) -> Result<(), SpeechToTextValidationErrorV1> {
    if zero(&request.request_id) {
        return Err(SpeechToTextValidationErrorV1::InvalidIdentity);
    }
    if request.logical_owner_id.is_empty()
        || request.logical_owner_id.len() > 128
        || !request.logical_owner_id.is_ascii()
        || request
            .logical_owner_id
            .bytes()
            .any(|byte| byte.is_ascii_control())
    {
        return Err(SpeechToTextValidationErrorV1::InvalidOwner);
    }
    validate_blob_receipt(&request.source, SPEECH_TO_TEXT_MAX_AUDIO_BYTES_V1)
        .map_err(|_| SpeechToTextValidationErrorV1::InvalidAudio)?;
    if request.duration_millis == 0
        || request.duration_millis > SPEECH_TO_TEXT_MAX_DURATION_MILLIS_V1
    {
        return Err(SpeechToTextValidationErrorV1::InvalidAudio);
    }
    if zero(&request.consent_receipt_id) || request.consent_policy_revision == 0 {
        return Err(SpeechToTextValidationErrorV1::InvalidConsent);
    }
    if request.maximum_transcript_bytes == 0
        || request.maximum_transcript_bytes > SPEECH_TO_TEXT_MAX_TRANSCRIPT_BYTES_V1
        || request.maximum_segments == 0
        || request.maximum_segments > SPEECH_TO_TEXT_MAX_SEGMENTS_V1
    {
        return Err(SpeechToTextValidationErrorV1::InvalidLimits);
    }
    if zero(&request.request_digest) {
        return Err(SpeechToTextValidationErrorV1::InvalidDigest);
    }
    Ok(())
}

pub fn validate_speech_to_text_result_v1(
    request: &SpeechToTextRequestV1,
    result: &SpeechToTextResultV1,
) -> Result<(), SpeechToTextValidationErrorV1> {
    validate_speech_to_text_request_v1(request)?;
    if result.request_id != request.request_id
        || result.request_digest != request.request_digest
        || result.source_sha256 != request.source.sha256
    {
        return Err(SpeechToTextValidationErrorV1::RequestResultMismatch);
    }
    if let SpeechToTextTerminalV1::Ready(artifact) = &result.terminal {
        validate_blob_receipt(
            &artifact.receipt,
            u64::from(request.maximum_transcript_bytes),
        )
        .map_err(|_| SpeechToTextValidationErrorV1::InvalidArtifact)?;
        if artifact.segment_count > request.maximum_segments
            || artifact.confidence_basis_points > 10_000
        {
            return Err(SpeechToTextValidationErrorV1::InvalidArtifact);
        }
        validate_execution_receipt(&artifact.execution_receipt)?;
    }
    Ok(())
}

fn validate_blob_receipt(
    receipt: &SpeechBlobReceiptV1,
    maximum_bytes: u64,
) -> Result<(), SpeechToTextValidationErrorV1> {
    if zero(&receipt.reference_id)
        || receipt.declared_bytes == 0
        || receipt.declared_bytes > maximum_bytes
        || zero(&receipt.sha256)
        || receipt.custody_proof.is_empty()
        || receipt.custody_proof.len() > SPEECH_TO_TEXT_MAX_CUSTODY_PROOF_BYTES_V1
    {
        return Err(SpeechToTextValidationErrorV1::InvalidArtifact);
    }
    Ok(())
}

fn validate_execution_receipt(
    receipt: &SpeechToTextExecutionReceiptV1,
) -> Result<(), SpeechToTextValidationErrorV1> {
    if zero(&receipt.provider_contract_schema_sha256)
        || zero(&receipt.model_revision_sha256)
        || receipt.provider_settings_revision == 0
        || receipt.provider_policy_revision == 0
    {
        return Err(SpeechToTextValidationErrorV1::InvalidExecutionReceipt);
    }
    Ok(())
}

fn zero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blob(marker: u8, declared_bytes: u64) -> SpeechBlobReceiptV1 {
        SpeechBlobReceiptV1 {
            reference_id: [marker; 16],
            declared_bytes,
            sha256: [marker; 32],
            custody_proof: vec![marker; 32],
        }
    }

    fn request() -> SpeechToTextRequestV1 {
        SpeechToTextRequestV1 {
            request_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            source: blob(2, 32_044),
            audio_format: SpeechAudioFormatV1::WavPcmS16LeMono16Khz,
            duration_millis: 1_000,
            requested_language: SpeechLanguageV1::Auto,
            consent_receipt_id: [3; 16],
            consent_policy_revision: 1,
            maximum_transcript_bytes: 64 * 1024,
            maximum_segments: 128,
            request_digest: [4; 32],
        }
    }

    fn ready_result() -> SpeechToTextResultV1 {
        SpeechToTextResultV1 {
            request_id: [1; 16],
            request_digest: [4; 32],
            source_sha256: [2; 32],
            terminal: SpeechToTextTerminalV1::Ready(SpeechTranscriptArtifactV1 {
                receipt: blob(5, 4_096),
                detected_language: SpeechLanguageV1::Russian,
                segment_count: 12,
                completeness: SpeechTranscriptCompletenessV1::Complete,
                confidence_basis_points: 9_100,
                execution_receipt: SpeechToTextExecutionReceiptV1 {
                    provider_contract_schema_sha256: [6; 32],
                    model_revision_sha256: [7; 32],
                    provider_settings_revision: 2,
                    provider_policy_revision: 1,
                },
            }),
        }
    }

    #[test]
    fn request_requires_audio_consent_limits_and_digest() {
        assert_eq!(validate_speech_to_text_request_v1(&request()), Ok(()));
        let mut invalid = request();
        invalid.consent_policy_revision = 0;
        assert_eq!(
            validate_speech_to_text_request_v1(&invalid),
            Err(SpeechToTextValidationErrorV1::InvalidConsent)
        );
        invalid = request();
        invalid.source.declared_bytes = SPEECH_TO_TEXT_MAX_AUDIO_BYTES_V1 + 1;
        assert_eq!(
            validate_speech_to_text_request_v1(&invalid),
            Err(SpeechToTextValidationErrorV1::InvalidAudio)
        );
    }

    #[test]
    fn ready_result_is_bound_to_request_source_and_bounded_artifact() {
        let request = request();
        assert_eq!(
            validate_speech_to_text_result_v1(&request, &ready_result()),
            Ok(())
        );
        let mut mismatched = ready_result();
        mismatched.source_sha256 = [9; 32];
        assert_eq!(
            validate_speech_to_text_result_v1(&request, &mismatched),
            Err(SpeechToTextValidationErrorV1::RequestResultMismatch)
        );
        let mut oversized = ready_result();
        let SpeechToTextTerminalV1::Ready(artifact) = &mut oversized.terminal else {
            unreachable!()
        };
        artifact.receipt.declared_bytes = u64::from(request.maximum_transcript_bytes) + 1;
        assert_eq!(
            validate_speech_to_text_result_v1(&request, &oversized),
            Err(SpeechToTextValidationErrorV1::InvalidArtifact)
        );
    }

    #[test]
    fn rejected_result_carries_no_artifact_or_provider_receipt() {
        let request = request();
        let rejected = SpeechToTextResultV1 {
            request_id: request.request_id,
            request_digest: request.request_digest,
            source_sha256: request.source.sha256,
            terminal: SpeechToTextTerminalV1::Rejected(SpeechToTextRejectionV1::Policy),
        };
        assert_eq!(
            validate_speech_to_text_result_v1(&request, &rejected),
            Ok(())
        );
    }
}
