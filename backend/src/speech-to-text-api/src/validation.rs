use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    SPEECH_TO_TEXT_CONTRACT_MAJOR_V1, SPEECH_TO_TEXT_MAX_AUDIO_BYTES_V1,
    SPEECH_TO_TEXT_MAX_CUSTODY_PROOF_BYTES_V1, SPEECH_TO_TEXT_MAX_DURATION_MILLIS_V1,
    SPEECH_TO_TEXT_MAX_SEGMENTS_V1, SPEECH_TO_TEXT_MAX_TRANSCRIPT_BYTES_V1,
    wire::{
        SpeechAudioFormatV1, SpeechLanguageV1, SpeechToTextRejectCodeV1, SpeechToTextRequestV1,
        SpeechToTextResultV1, SpeechToTextTerminalStatusV1, SpeechTranscriptCompletenessV1,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechToTextContractErrorV1 {
    InvalidProtocol,
    InvalidIdentity,
    InvalidOwner,
    InvalidSource,
    InvalidConsent,
    InvalidLimits,
    InvalidDigest,
    InvalidResult,
    ResultMismatch,
}

pub fn seal_speech_to_text_request_v1(
    mut request: SpeechToTextRequestV1,
) -> Result<SpeechToTextRequestV1, SpeechToTextContractErrorV1> {
    request.protocol_major = SPEECH_TO_TEXT_CONTRACT_MAJOR_V1;
    request.request_digest.clear();
    request.request_digest = compute_speech_to_text_request_digest_v1(&request)?.to_vec();
    validate_speech_to_text_request_v1(&request)?;
    Ok(request)
}

pub fn compute_speech_to_text_request_digest_v1(
    request: &SpeechToTextRequestV1,
) -> Result<[u8; 32], SpeechToTextContractErrorV1> {
    validate_request_shape(request, false)?;
    let mut canonical = request.clone();
    canonical.request_digest.clear();
    if let Some(source) = canonical.source.as_mut() {
        source.custody_transfer_source_proof.clear();
    }
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.speech-to-text.request.v1\0");
    hasher.update(canonical.encode_to_vec());
    Ok(hasher.finalize().into())
}

pub fn validate_speech_to_text_request_v1(
    request: &SpeechToTextRequestV1,
) -> Result<(), SpeechToTextContractErrorV1> {
    validate_request_shape(request, true)?;
    let expected = compute_speech_to_text_request_digest_v1(request)?;
    if request.request_digest.as_slice() != expected {
        return Err(SpeechToTextContractErrorV1::InvalidDigest);
    }
    Ok(())
}

pub fn validate_speech_to_text_result_v1(
    request: &SpeechToTextRequestV1,
    result: &SpeechToTextResultV1,
) -> Result<(), SpeechToTextContractErrorV1> {
    validate_speech_to_text_request_v1(request)?;
    if result.request_id != request.request_id
        || result.request_digest != request.request_digest
        || result.source_sha256
            != request
                .source
                .as_ref()
                .ok_or(SpeechToTextContractErrorV1::InvalidSource)?
                .sha256
    {
        return Err(SpeechToTextContractErrorV1::ResultMismatch);
    }
    let status = SpeechToTextTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| SpeechToTextContractErrorV1::InvalidResult)?;
    match status {
        SpeechToTextTerminalStatusV1::Ready => validate_ready_result(request, result),
        SpeechToTextTerminalStatusV1::Rejected => validate_rejected_result(result),
        SpeechToTextTerminalStatusV1::Unspecified => {
            Err(SpeechToTextContractErrorV1::InvalidResult)
        }
    }
}

fn validate_request_shape(
    request: &SpeechToTextRequestV1,
    require_digest: bool,
) -> Result<(), SpeechToTextContractErrorV1> {
    if request.protocol_major != SPEECH_TO_TEXT_CONTRACT_MAJOR_V1 {
        return Err(SpeechToTextContractErrorV1::InvalidProtocol);
    }
    if !id(&request.request_id, 16) {
        return Err(SpeechToTextContractErrorV1::InvalidIdentity);
    }
    if !token(&request.logical_owner_id) {
        return Err(SpeechToTextContractErrorV1::InvalidOwner);
    }
    let source = request
        .source
        .as_ref()
        .ok_or(SpeechToTextContractErrorV1::InvalidSource)?;
    if !id(&source.reference_id, 16)
        || source.declared_bytes == 0
        || source.declared_bytes > SPEECH_TO_TEXT_MAX_AUDIO_BYTES_V1
        || !id(&source.sha256, 32)
        || source.custody_transfer_source_proof.is_empty()
        || source.custody_transfer_source_proof.len() > SPEECH_TO_TEXT_MAX_CUSTODY_PROOF_BYTES_V1
        || request.audio_format != SpeechAudioFormatV1::WavPcmS16leMono16000Hz as i32
        || request.duration_millis == 0
        || request.duration_millis > SPEECH_TO_TEXT_MAX_DURATION_MILLIS_V1
        || !matches!(
            SpeechLanguageV1::try_from(request.requested_language),
            Ok(SpeechLanguageV1::Auto
                | SpeechLanguageV1::English
                | SpeechLanguageV1::Russian
                | SpeechLanguageV1::Spanish)
        )
    {
        return Err(SpeechToTextContractErrorV1::InvalidSource);
    }
    if !id(&request.consent_receipt_id, 16) || request.consent_policy_revision == 0 {
        return Err(SpeechToTextContractErrorV1::InvalidConsent);
    }
    if request.maximum_transcript_bytes == 0
        || request.maximum_transcript_bytes > SPEECH_TO_TEXT_MAX_TRANSCRIPT_BYTES_V1
        || request.maximum_segments == 0
        || request.maximum_segments > SPEECH_TO_TEXT_MAX_SEGMENTS_V1
    {
        return Err(SpeechToTextContractErrorV1::InvalidLimits);
    }
    if require_digest && !id(&request.request_digest, 32) {
        return Err(SpeechToTextContractErrorV1::InvalidDigest);
    }
    Ok(())
}

fn validate_ready_result(
    request: &SpeechToTextRequestV1,
    result: &SpeechToTextResultV1,
) -> Result<(), SpeechToTextContractErrorV1> {
    let transcript = result
        .transcript
        .as_ref()
        .ok_or(SpeechToTextContractErrorV1::InvalidResult)?;
    let receipt = result
        .execution_receipt
        .as_ref()
        .ok_or(SpeechToTextContractErrorV1::InvalidResult)?;
    if !id(&transcript.reference_id, 16)
        || transcript.declared_bytes == 0
        || transcript.declared_bytes > u64::from(request.maximum_transcript_bytes)
        || !id(&transcript.sha256, 32)
        || transcript.custody_transfer_source_proof.is_empty()
        || transcript.custody_transfer_source_proof.len()
            > SPEECH_TO_TEXT_MAX_CUSTODY_PROOF_BYTES_V1
        || !matches!(
            SpeechLanguageV1::try_from(result.detected_language),
            Ok(SpeechLanguageV1::Auto
                | SpeechLanguageV1::English
                | SpeechLanguageV1::Russian
                | SpeechLanguageV1::Spanish)
        )
        || result.segment_count > request.maximum_segments
        || !matches!(
            SpeechTranscriptCompletenessV1::try_from(result.completeness),
            Ok(SpeechTranscriptCompletenessV1::Complete | SpeechTranscriptCompletenessV1::Partial)
        )
        || result.confidence_basis_points > 10_000
        || !id(&receipt.provider_contract_schema_sha256, 32)
        || !id(&receipt.model_revision_sha256, 32)
        || receipt.provider_settings_revision == 0
        || receipt.provider_policy_revision == 0
        || result.reject_code != SpeechToTextRejectCodeV1::Unspecified as i32
    {
        return Err(SpeechToTextContractErrorV1::InvalidResult);
    }
    Ok(())
}

fn validate_rejected_result(
    result: &SpeechToTextResultV1,
) -> Result<(), SpeechToTextContractErrorV1> {
    if result.transcript.is_some()
        || result.execution_receipt.is_some()
        || result.detected_language != SpeechLanguageV1::Unspecified as i32
        || result.segment_count != 0
        || result.completeness != SpeechTranscriptCompletenessV1::Unspecified as i32
        || result.confidence_basis_points != 0
        || !matches!(
            SpeechToTextRejectCodeV1::try_from(result.reject_code),
            Ok(SpeechToTextRejectCodeV1::InvalidRequest
                | SpeechToTextRejectCodeV1::ConsentRejected
                | SpeechToTextRejectCodeV1::UnsupportedAudio
                | SpeechToTextRejectCodeV1::ProviderUnavailable
                | SpeechToTextRejectCodeV1::ProviderRejected
                | SpeechToTextRejectCodeV1::Policy)
        )
    {
        return Err(SpeechToTextContractErrorV1::InvalidResult);
    }
    Ok(())
}

fn id(value: &[u8], exact: usize) -> bool {
    value.len() == exact && value.iter().any(|byte| *byte != 0)
}

fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.is_ascii()
        && !value.bytes().any(|byte| byte.is_ascii_control())
}

#[cfg(test)]
mod tests {
    use crate::wire::SpeechAudioSourceReceiptV1;

    use super::*;

    fn request(proof: u8) -> SpeechToTextRequestV1 {
        seal_speech_to_text_request_v1(SpeechToTextRequestV1 {
            protocol_major: 0,
            request_id: vec![1; 16],
            logical_owner_id: "owner-1".to_owned(),
            source: Some(SpeechAudioSourceReceiptV1 {
                reference_id: vec![2; 16],
                declared_bytes: 32_044,
                sha256: vec![3; 32],
                custody_transfer_source_proof: vec![proof; 32],
            }),
            audio_format: SpeechAudioFormatV1::WavPcmS16leMono16000Hz as i32,
            duration_millis: 1_000,
            requested_language: SpeechLanguageV1::Auto as i32,
            consent_receipt_id: vec![4; 16],
            consent_policy_revision: 1,
            maximum_transcript_bytes: 64 * 1024,
            maximum_segments: 128,
            request_digest: Vec::new(),
        })
        .expect("sealed request")
    }

    #[test]
    fn digest_binds_semantics_but_not_refreshable_custody_proof() {
        assert_eq!(request(5).request_digest, request(6).request_digest);
        let mut changed = request(5);
        changed.duration_millis += 1;
        assert_eq!(
            validate_speech_to_text_request_v1(&changed),
            Err(SpeechToTextContractErrorV1::InvalidDigest)
        );
    }
}
