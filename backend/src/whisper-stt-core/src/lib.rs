#![forbid(unsafe_code)]

use makosh_speech_to_text_api::{
    SPEECH_TO_TEXT_SCHEMA_SHA256, validate_speech_to_text_request_v1,
    validate_speech_to_text_result_v1,
    wire::{
        SpeechLanguageV1, SpeechToTextExecutionReceiptV1, SpeechToTextRejectCodeV1,
        SpeechToTextRequestV1, SpeechToTextResultV1, SpeechToTextTerminalStatusV1,
        SpeechTranscriptArtifactReceiptV1, SpeechTranscriptCompletenessV1,
    },
};
use makosh_speech_transcript_artifact::{
    SPEECH_TRANSCRIPT_ARTIFACT_PROTOCOL_MAJOR_V1, encode_speech_transcript_document_v1,
    speech_transcript_document_sha256_v1,
    wire::{
        SpeechTranscriptCompletenessV1 as ArtifactCompletenessV1, SpeechTranscriptDocumentV1,
        SpeechTranscriptLanguageV1 as ArtifactLanguageV1, SpeechTranscriptSegmentV1,
    },
};

pub const PACKAGE: &str = "makosh-whisper-stt-core";
pub const WHISPER_STT_POLICY_REVISION_V1: u32 = 1;
pub const WHISPER_STT_MIN_TIMEOUT_MILLIS_V1: u64 = 1_000;
pub const WHISPER_STT_MAX_TIMEOUT_MILLIS_V1: u64 = 30 * 60 * 1_000;
pub const WHISPER_STT_MAX_THREADS_V1: u32 = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhisperSttExecutionPlanV1 {
    pub request: SpeechToTextRequestV1,
    pub model_revision_sha256: [u8; 32],
    pub provider_settings_revision: u64,
    pub provider_policy_revision: u32,
    pub thread_count: u32,
    pub timeout_millis: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhisperSttTranscriptSegmentV1 {
    pub start_millis: u64,
    pub end_millis: u64,
    pub content_utf8: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhisperSttExecutionOutcomeV1 {
    pub detected_language: SpeechLanguageV1,
    pub segments: Vec<WhisperSttTranscriptSegmentV1>,
    pub completeness: SpeechTranscriptCompletenessV1,
    pub confidence_basis_points: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhisperSttArtifactV1 {
    pub encoded_document: Vec<u8>,
    pub sha256: [u8; 32],
    pub detected_language: SpeechLanguageV1,
    pub segment_count: u32,
    pub completeness: SpeechTranscriptCompletenessV1,
    pub confidence_basis_points: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhisperSttBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhisperSttCoreErrorV1 {
    InvalidRequest,
    InvalidPolicy,
    InvalidOutcome,
    InvalidArtifact,
}

pub fn plan_whisper_stt_execution_v1(
    request: SpeechToTextRequestV1,
    model_revision_sha256: [u8; 32],
    provider_settings_revision: u64,
    thread_count: u32,
    timeout_millis: u64,
) -> Result<WhisperSttExecutionPlanV1, WhisperSttCoreErrorV1> {
    validate_speech_to_text_request_v1(&request)
        .map_err(|_| WhisperSttCoreErrorV1::InvalidRequest)?;
    if model_revision_sha256 == [0; 32]
        || provider_settings_revision == 0
        || !(1..=WHISPER_STT_MAX_THREADS_V1).contains(&thread_count)
        || !(WHISPER_STT_MIN_TIMEOUT_MILLIS_V1..=WHISPER_STT_MAX_TIMEOUT_MILLIS_V1)
            .contains(&timeout_millis)
    {
        return Err(WhisperSttCoreErrorV1::InvalidPolicy);
    }
    Ok(WhisperSttExecutionPlanV1 {
        request,
        model_revision_sha256,
        provider_settings_revision,
        provider_policy_revision: WHISPER_STT_POLICY_REVISION_V1,
        thread_count,
        timeout_millis,
    })
}

pub fn build_whisper_stt_artifact_v1(
    plan: &WhisperSttExecutionPlanV1,
    outcome: WhisperSttExecutionOutcomeV1,
) -> Result<WhisperSttArtifactV1, WhisperSttCoreErrorV1> {
    if outcome.segments.len() > usize::try_from(plan.request.maximum_segments).unwrap_or(usize::MAX)
        || outcome.confidence_basis_points > 10_000
    {
        return Err(WhisperSttCoreErrorV1::InvalidOutcome);
    }
    let document = SpeechTranscriptDocumentV1 {
        protocol_major: SPEECH_TRANSCRIPT_ARTIFACT_PROTOCOL_MAJOR_V1,
        request_id: plan.request.request_id.clone(),
        detected_language: artifact_language(outcome.detected_language)? as i32,
        duration_millis: plan.request.duration_millis,
        segments: outcome
            .segments
            .into_iter()
            .enumerate()
            .map(|(index, segment)| {
                Ok(SpeechTranscriptSegmentV1 {
                    index: u32::try_from(index)
                        .map_err(|_| WhisperSttCoreErrorV1::InvalidOutcome)?,
                    start_millis: segment.start_millis,
                    end_millis: segment.end_millis,
                    content_utf8: segment.content_utf8,
                })
            })
            .collect::<Result<Vec<_>, WhisperSttCoreErrorV1>>()?,
        completeness: artifact_completeness(outcome.completeness)? as i32,
        confidence_basis_points: outcome.confidence_basis_points,
    };
    let encoded_document = encode_speech_transcript_document_v1(
        &document,
        plan.request.duration_millis,
        plan.request.maximum_segments,
        plan.request.maximum_transcript_bytes,
    )
    .map_err(|_| WhisperSttCoreErrorV1::InvalidOutcome)?;
    let sha256 = speech_transcript_document_sha256_v1(&encoded_document);
    Ok(WhisperSttArtifactV1 {
        encoded_document,
        sha256,
        detected_language: outcome.detected_language,
        segment_count: document.segments.len() as u32,
        completeness: outcome.completeness,
        confidence_basis_points: outcome.confidence_basis_points,
    })
}

pub fn complete_whisper_stt_result_v1(
    plan: &WhisperSttExecutionPlanV1,
    artifact: &WhisperSttArtifactV1,
    receipt: WhisperSttBlobReceiptV1,
) -> Result<SpeechToTextResultV1, WhisperSttCoreErrorV1> {
    if receipt.reference_id == [0; 16]
        || receipt.declared_bytes != artifact.encoded_document.len() as u64
        || receipt.sha256 != artifact.sha256
        || receipt.custody_transfer_source_proof.is_empty()
    {
        return Err(WhisperSttCoreErrorV1::InvalidArtifact);
    }
    let source_sha256 = plan
        .request
        .source
        .as_ref()
        .ok_or(WhisperSttCoreErrorV1::InvalidRequest)?
        .sha256
        .clone();
    let result = SpeechToTextResultV1 {
        request_id: plan.request.request_id.clone(),
        request_digest: plan.request.request_digest.clone(),
        source_sha256,
        terminal_status: SpeechToTextTerminalStatusV1::Ready as i32,
        transcript: Some(SpeechTranscriptArtifactReceiptV1 {
            reference_id: receipt.reference_id.to_vec(),
            declared_bytes: receipt.declared_bytes,
            sha256: receipt.sha256.to_vec(),
            custody_transfer_source_proof: receipt.custody_transfer_source_proof,
        }),
        detected_language: artifact.detected_language as i32,
        segment_count: artifact.segment_count,
        completeness: artifact.completeness as i32,
        confidence_basis_points: artifact.confidence_basis_points,
        execution_receipt: Some(SpeechToTextExecutionReceiptV1 {
            provider_contract_schema_sha256: SPEECH_TO_TEXT_SCHEMA_SHA256.to_vec(),
            model_revision_sha256: plan.model_revision_sha256.to_vec(),
            provider_settings_revision: plan.provider_settings_revision,
            provider_policy_revision: plan.provider_policy_revision,
        }),
        reject_code: SpeechToTextRejectCodeV1::Unspecified as i32,
    };
    validate_speech_to_text_result_v1(&plan.request, &result)
        .map_err(|_| WhisperSttCoreErrorV1::InvalidArtifact)?;
    Ok(result)
}

pub fn reject_whisper_stt_result_v1(
    request: &SpeechToTextRequestV1,
    code: SpeechToTextRejectCodeV1,
) -> Result<SpeechToTextResultV1, WhisperSttCoreErrorV1> {
    if !matches!(
        code,
        SpeechToTextRejectCodeV1::InvalidRequest
            | SpeechToTextRejectCodeV1::ConsentRejected
            | SpeechToTextRejectCodeV1::UnsupportedAudio
            | SpeechToTextRejectCodeV1::ProviderUnavailable
            | SpeechToTextRejectCodeV1::ProviderRejected
            | SpeechToTextRejectCodeV1::Policy
    ) {
        return Err(WhisperSttCoreErrorV1::InvalidOutcome);
    }
    let result = SpeechToTextResultV1 {
        request_id: request.request_id.clone(),
        request_digest: request.request_digest.clone(),
        source_sha256: request
            .source
            .as_ref()
            .ok_or(WhisperSttCoreErrorV1::InvalidRequest)?
            .sha256
            .clone(),
        terminal_status: SpeechToTextTerminalStatusV1::Rejected as i32,
        transcript: None,
        detected_language: SpeechLanguageV1::Unspecified as i32,
        segment_count: 0,
        completeness: SpeechTranscriptCompletenessV1::Unspecified as i32,
        confidence_basis_points: 0,
        execution_receipt: None,
        reject_code: code as i32,
    };
    validate_speech_to_text_result_v1(request, &result)
        .map_err(|_| WhisperSttCoreErrorV1::InvalidOutcome)?;
    Ok(result)
}

fn artifact_language(value: SpeechLanguageV1) -> Result<ArtifactLanguageV1, WhisperSttCoreErrorV1> {
    match value {
        SpeechLanguageV1::Auto => Ok(ArtifactLanguageV1::Auto),
        SpeechLanguageV1::English => Ok(ArtifactLanguageV1::English),
        SpeechLanguageV1::Russian => Ok(ArtifactLanguageV1::Russian),
        SpeechLanguageV1::Spanish => Ok(ArtifactLanguageV1::Spanish),
        SpeechLanguageV1::Unspecified => Err(WhisperSttCoreErrorV1::InvalidOutcome),
    }
}

fn artifact_completeness(
    value: SpeechTranscriptCompletenessV1,
) -> Result<ArtifactCompletenessV1, WhisperSttCoreErrorV1> {
    match value {
        SpeechTranscriptCompletenessV1::Complete => Ok(ArtifactCompletenessV1::Complete),
        SpeechTranscriptCompletenessV1::Partial => Ok(ArtifactCompletenessV1::Partial),
        SpeechTranscriptCompletenessV1::Unspecified => Err(WhisperSttCoreErrorV1::InvalidOutcome),
    }
}

#[cfg(test)]
mod tests {
    use makosh_speech_to_text_api::{
        seal_speech_to_text_request_v1,
        wire::{SpeechAudioFormatV1, SpeechAudioSourceReceiptV1},
    };

    use super::*;

    fn request() -> SpeechToTextRequestV1 {
        seal_speech_to_text_request_v1(SpeechToTextRequestV1 {
            protocol_major: 0,
            request_id: vec![1; 16],
            logical_owner_id: "owner-1".to_owned(),
            source: Some(SpeechAudioSourceReceiptV1 {
                reference_id: vec![2; 16],
                declared_bytes: 32_044,
                sha256: vec![3; 32],
                custody_transfer_source_proof: vec![4; 32],
            }),
            audio_format: SpeechAudioFormatV1::WavPcmS16leMono16000Hz as i32,
            duration_millis: 1_000,
            requested_language: SpeechLanguageV1::English as i32,
            consent_receipt_id: vec![5; 16],
            consent_policy_revision: 1,
            maximum_transcript_bytes: 64 * 1024,
            maximum_segments: 8,
            request_digest: Vec::new(),
        })
        .expect("sealed request")
    }

    #[test]
    fn builds_canonical_artifact_and_typed_ready_result() {
        let plan = plan_whisper_stt_execution_v1(request(), [6; 32], 2, 4, 30_000).expect("plan");
        let artifact = build_whisper_stt_artifact_v1(
            &plan,
            WhisperSttExecutionOutcomeV1 {
                detected_language: SpeechLanguageV1::English,
                segments: vec![WhisperSttTranscriptSegmentV1 {
                    start_millis: 0,
                    end_millis: 1_000,
                    content_utf8: b"hello".to_vec(),
                }],
                completeness: SpeechTranscriptCompletenessV1::Complete,
                confidence_basis_points: 9_000,
            },
        )
        .expect("artifact");
        let result = complete_whisper_stt_result_v1(
            &plan,
            &artifact,
            WhisperSttBlobReceiptV1 {
                reference_id: [7; 16],
                declared_bytes: artifact.encoded_document.len() as u64,
                sha256: artifact.sha256,
                custody_transfer_source_proof: vec![8; 32],
            },
        )
        .expect("result");
        assert_eq!(
            result.terminal_status,
            SpeechToTextTerminalStatusV1::Ready as i32
        );
        assert_eq!(result.segment_count, 1);
        assert_eq!(
            result
                .execution_receipt
                .expect("receipt")
                .model_revision_sha256,
            vec![6; 32]
        );
    }
}
