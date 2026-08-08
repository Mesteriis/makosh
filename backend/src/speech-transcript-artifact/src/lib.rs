#![forbid(unsafe_code)]

use prost::Message;
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-speech-transcript-artifact";
pub const SPEECH_TRANSCRIPT_ARTIFACT_PROTOCOL_MAJOR_V1: u32 = 1;
pub const SPEECH_TRANSCRIPT_ARTIFACT_MAX_BYTES_V1: usize = 4 * 1024 * 1024;
pub const SPEECH_TRANSCRIPT_ARTIFACT_MAX_SEGMENTS_V1: usize = 100_000;
pub const SPEECH_TRANSCRIPT_ARTIFACT_MAX_SEGMENT_BYTES_V1: usize = 64 * 1024;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.speech_transcript.v1.rs"));
}

include!(concat!(
    env!("OUT_DIR"),
    "/speech_transcript_artifact_schema.rs"
));

pub const SPEECH_TRANSCRIPT_ARTIFACT_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/speech-transcript-artifact-v1.bin"
));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechTranscriptArtifactErrorV1 {
    InvalidProtocol,
    InvalidIdentity,
    InvalidLanguage,
    InvalidDuration,
    InvalidCompleteness,
    InvalidConfidence,
    InvalidSegment,
    TooLarge,
}

pub fn validate_speech_transcript_document_v1(
    document: &wire::SpeechTranscriptDocumentV1,
    maximum_duration_millis: u64,
    maximum_segments: u32,
    maximum_encoded_bytes: u32,
) -> Result<(), SpeechTranscriptArtifactErrorV1> {
    use wire::{SpeechTranscriptCompletenessV1, SpeechTranscriptLanguageV1};

    if document.protocol_major != SPEECH_TRANSCRIPT_ARTIFACT_PROTOCOL_MAJOR_V1 {
        return Err(SpeechTranscriptArtifactErrorV1::InvalidProtocol);
    }
    if document.request_id.len() != 16 || document.request_id.iter().all(|byte| *byte == 0) {
        return Err(SpeechTranscriptArtifactErrorV1::InvalidIdentity);
    }
    if !matches!(
        SpeechTranscriptLanguageV1::try_from(document.detected_language),
        Ok(SpeechTranscriptLanguageV1::Auto
            | SpeechTranscriptLanguageV1::English
            | SpeechTranscriptLanguageV1::Russian
            | SpeechTranscriptLanguageV1::Spanish)
    ) {
        return Err(SpeechTranscriptArtifactErrorV1::InvalidLanguage);
    }
    if maximum_duration_millis == 0
        || document.duration_millis == 0
        || document.duration_millis > maximum_duration_millis
    {
        return Err(SpeechTranscriptArtifactErrorV1::InvalidDuration);
    }
    if !matches!(
        SpeechTranscriptCompletenessV1::try_from(document.completeness),
        Ok(SpeechTranscriptCompletenessV1::Complete | SpeechTranscriptCompletenessV1::Partial)
    ) {
        return Err(SpeechTranscriptArtifactErrorV1::InvalidCompleteness);
    }
    if document.confidence_basis_points > 10_000 {
        return Err(SpeechTranscriptArtifactErrorV1::InvalidConfidence);
    }
    let segment_limit = usize::try_from(maximum_segments)
        .unwrap_or(usize::MAX)
        .min(SPEECH_TRANSCRIPT_ARTIFACT_MAX_SEGMENTS_V1);
    if document.segments.is_empty() || document.segments.len() > segment_limit {
        return Err(SpeechTranscriptArtifactErrorV1::InvalidSegment);
    }
    let mut previous_end = 0_u64;
    for (position, segment) in document.segments.iter().enumerate() {
        if usize::try_from(segment.index).ok() != Some(position)
            || segment.start_millis < previous_end
            || segment.end_millis <= segment.start_millis
            || segment.end_millis > document.duration_millis
            || segment.content_utf8.is_empty()
            || segment.content_utf8.len() > SPEECH_TRANSCRIPT_ARTIFACT_MAX_SEGMENT_BYTES_V1
            || std::str::from_utf8(&segment.content_utf8).is_err()
            || segment
                .content_utf8
                .iter()
                .all(|byte| byte.is_ascii_whitespace())
        {
            return Err(SpeechTranscriptArtifactErrorV1::InvalidSegment);
        }
        previous_end = segment.end_millis;
    }
    let encoded_limit = usize::try_from(maximum_encoded_bytes)
        .unwrap_or(usize::MAX)
        .min(SPEECH_TRANSCRIPT_ARTIFACT_MAX_BYTES_V1);
    if encoded_limit == 0 || document.encoded_len() > encoded_limit {
        return Err(SpeechTranscriptArtifactErrorV1::TooLarge);
    }
    Ok(())
}

pub fn encode_speech_transcript_document_v1(
    document: &wire::SpeechTranscriptDocumentV1,
    maximum_duration_millis: u64,
    maximum_segments: u32,
    maximum_encoded_bytes: u32,
) -> Result<Vec<u8>, SpeechTranscriptArtifactErrorV1> {
    validate_speech_transcript_document_v1(
        document,
        maximum_duration_millis,
        maximum_segments,
        maximum_encoded_bytes,
    )?;
    Ok(document.encode_to_vec())
}

#[must_use]
pub fn speech_transcript_document_sha256_v1(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document() -> wire::SpeechTranscriptDocumentV1 {
        wire::SpeechTranscriptDocumentV1 {
            protocol_major: 1,
            request_id: vec![1; 16],
            detected_language: wire::SpeechTranscriptLanguageV1::English as i32,
            duration_millis: 2_000,
            segments: vec![
                wire::SpeechTranscriptSegmentV1 {
                    index: 0,
                    start_millis: 0,
                    end_millis: 900,
                    content_utf8: b"first".to_vec(),
                },
                wire::SpeechTranscriptSegmentV1 {
                    index: 1,
                    start_millis: 1_000,
                    end_millis: 2_000,
                    content_utf8: b"second".to_vec(),
                },
            ],
            completeness: wire::SpeechTranscriptCompletenessV1::Complete as i32,
            confidence_basis_points: 9_000,
        }
    }

    #[test]
    fn accepts_only_ordered_bounded_utf8_segments() {
        let valid = document();
        let bytes =
            encode_speech_transcript_document_v1(&valid, 2_000, 8, 4_096).expect("valid document");
        assert_eq!(speech_transcript_document_sha256_v1(&bytes).len(), 32);

        let mut overlap = valid.clone();
        overlap.segments[1].start_millis = 800;
        assert_eq!(
            validate_speech_transcript_document_v1(&overlap, 2_000, 8, 4_096),
            Err(SpeechTranscriptArtifactErrorV1::InvalidSegment)
        );
        let mut invalid_utf8 = valid;
        invalid_utf8.segments[0].content_utf8 = vec![0xff];
        assert_eq!(
            validate_speech_transcript_document_v1(&invalid_utf8, 2_000, 8, 4_096),
            Err(SpeechTranscriptArtifactErrorV1::InvalidSegment)
        );
    }

    #[test]
    fn schema_is_private_content_only_and_provider_neutral() {
        let source = include_str!("../proto/makosh/speech_transcript/v1/transcript.proto");
        assert!(source.contains("content_utf8"));
        for forbidden in [
            "provider_name",
            "model_name",
            "filesystem_path",
            "custody_proof",
            "map<",
        ] {
            assert!(!source.contains(forbidden), "forbidden {forbidden}");
        }
    }
}
