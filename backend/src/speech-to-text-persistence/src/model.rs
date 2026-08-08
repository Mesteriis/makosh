use makosh_speech_to_text_core::{
    SpeechAudioFormatV1, SpeechLanguageV1, SpeechToTextRejectionV1, SpeechToTextRequestV1,
    SpeechToTextRunStateV1, SpeechToTextRunV1, SpeechToTextTerminalV1,
    SpeechTranscriptCompletenessV1, validate_speech_to_text_run_v1,
};

pub const SPEECH_TO_TEXT_RECOVERY_LIMIT_V1: u32 = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedSpeechToTextRequestV1 {
    pub request_id: [u8; 16],
    pub logical_owner_id: String,
    pub source_reference_id: [u8; 16],
    pub source_declared_bytes: u64,
    pub source_sha256: [u8; 32],
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
pub struct PersistedSpeechTranscriptArtifactV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub detected_language: SpeechLanguageV1,
    pub segment_count: u32,
    pub completeness: SpeechTranscriptCompletenessV1,
    pub confidence_basis_points: u32,
    pub provider_contract_schema_sha256: [u8; 32],
    pub model_revision_sha256: [u8; 32],
    pub provider_settings_revision: u64,
    pub provider_policy_revision: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedSpeechToTextRunV1 {
    pub request: PersistedSpeechToTextRequestV1,
    pub state: SpeechToTextRunStateV1,
    pub revision: u64,
    pub artifact: Option<PersistedSpeechTranscriptArtifactV1>,
    pub rejection: Option<SpeechToTextRejectionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpeechToTextTransitionV1 {
    pub current_revision: u64,
    pub next_run: SpeechToTextRunV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpeechToTextPersistenceOutcomeV1 {
    pub persisted: PersistedSpeechToTextRunV1,
    pub replayed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpeechToTextPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    RevisionConflict,
    InvalidTransition,
}

pub(crate) fn persisted_request(request: &SpeechToTextRequestV1) -> PersistedSpeechToTextRequestV1 {
    PersistedSpeechToTextRequestV1 {
        request_id: request.request_id,
        logical_owner_id: request.logical_owner_id.clone(),
        source_reference_id: request.source.reference_id,
        source_declared_bytes: request.source.declared_bytes,
        source_sha256: request.source.sha256,
        audio_format: request.audio_format,
        duration_millis: request.duration_millis,
        requested_language: request.requested_language,
        consent_receipt_id: request.consent_receipt_id,
        consent_policy_revision: request.consent_policy_revision,
        maximum_transcript_bytes: request.maximum_transcript_bytes,
        maximum_segments: request.maximum_segments,
        request_digest: request.request_digest,
    }
}

pub(crate) fn request_matches(
    persisted: &PersistedSpeechToTextRequestV1,
    request: &SpeechToTextRequestV1,
) -> bool {
    persisted == &persisted_request(request)
}

pub(crate) fn validate_accepted(
    run: &SpeechToTextRunV1,
) -> Result<(), SpeechToTextPersistenceErrorV1> {
    validate_speech_to_text_run_v1(run)
        .map_err(|_| SpeechToTextPersistenceErrorV1::InvalidInput)?;
    if run.revision != 1
        || run.state != SpeechToTextRunStateV1::Accepted
        || run.terminal_result.is_some()
    {
        return Err(SpeechToTextPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

pub(crate) fn validate_transition(
    current: &PersistedSpeechToTextRunV1,
    transition: &SpeechToTextTransitionV1,
) -> Result<(), SpeechToTextPersistenceErrorV1> {
    validate_speech_to_text_run_v1(&transition.next_run)
        .map_err(|_| SpeechToTextPersistenceErrorV1::InvalidInput)?;
    if transition.current_revision != current.revision
        || transition.next_run.revision != current.revision + 1
        || !request_matches(&current.request, &transition.next_run.request)
    {
        return Err(SpeechToTextPersistenceErrorV1::RevisionConflict);
    }
    match (current.state, transition.next_run.state) {
        (SpeechToTextRunStateV1::Accepted, SpeechToTextRunStateV1::Executing)
        | (SpeechToTextRunStateV1::Accepted, SpeechToTextRunStateV1::Rejected)
        | (SpeechToTextRunStateV1::Executing, SpeechToTextRunStateV1::Ready)
        | (SpeechToTextRunStateV1::Executing, SpeechToTextRunStateV1::Rejected) => Ok(()),
        _ => Err(SpeechToTextPersistenceErrorV1::InvalidTransition),
    }
}

pub(crate) fn terminal_parts(
    run: &SpeechToTextRunV1,
) -> Result<
    (
        Option<PersistedSpeechTranscriptArtifactV1>,
        Option<SpeechToTextRejectionV1>,
    ),
    SpeechToTextPersistenceErrorV1,
> {
    match run.terminal_result.as_ref().map(|value| &value.terminal) {
        None => Ok((None, None)),
        Some(SpeechToTextTerminalV1::Ready(artifact)) => Ok((
            Some(PersistedSpeechTranscriptArtifactV1 {
                reference_id: artifact.receipt.reference_id,
                declared_bytes: artifact.receipt.declared_bytes,
                sha256: artifact.receipt.sha256,
                detected_language: artifact.detected_language,
                segment_count: artifact.segment_count,
                completeness: artifact.completeness,
                confidence_basis_points: artifact.confidence_basis_points,
                provider_contract_schema_sha256: artifact
                    .execution_receipt
                    .provider_contract_schema_sha256,
                model_revision_sha256: artifact.execution_receipt.model_revision_sha256,
                provider_settings_revision: artifact.execution_receipt.provider_settings_revision,
                provider_policy_revision: artifact.execution_receipt.provider_policy_revision,
            }),
            None,
        )),
        Some(SpeechToTextTerminalV1::Rejected(rejection)) => Ok((None, Some(*rejection))),
    }
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.is_ascii()
        && !value.bytes().any(|byte| byte.is_ascii_control())
}

#[cfg(test)]
mod tests {
    use makosh_speech_to_text_core::{
        SpeechBlobReceiptV1, accept_speech_to_text_v1, begin_speech_to_text_v1,
    };

    use super::*;

    fn request(proof: u8) -> SpeechToTextRequestV1 {
        SpeechToTextRequestV1 {
            request_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            source: SpeechBlobReceiptV1 {
                reference_id: [2; 16],
                declared_bytes: 32_044,
                sha256: [3; 32],
                custody_proof: vec![proof; 32],
            },
            audio_format: SpeechAudioFormatV1::WavPcmS16LeMono16Khz,
            duration_millis: 1_000,
            requested_language: SpeechLanguageV1::Auto,
            consent_receipt_id: [4; 16],
            consent_policy_revision: 1,
            maximum_transcript_bytes: 64 * 1024,
            maximum_segments: 128,
            request_digest: [5; 32],
        }
    }

    #[test]
    fn persisted_request_deliberately_omits_refreshable_custody_proof() {
        let safe = persisted_request(&request(6));
        assert!(request_matches(&safe, &request(7)));
    }

    #[test]
    fn only_exact_revision_and_forward_transition_are_accepted() {
        let accepted = accept_speech_to_text_v1(request(6)).expect("accepted");
        let current = PersistedSpeechToTextRunV1 {
            request: persisted_request(&accepted.request),
            state: accepted.state,
            revision: accepted.revision,
            artifact: None,
            rejection: None,
        };
        let executing = begin_speech_to_text_v1(&accepted, 1).expect("executing");
        assert_eq!(
            validate_transition(
                &current,
                &SpeechToTextTransitionV1 {
                    current_revision: 1,
                    next_run: executing,
                }
            ),
            Ok(())
        );
    }
}
