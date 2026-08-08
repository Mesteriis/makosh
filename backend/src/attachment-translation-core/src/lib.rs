#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-attachment-translation-core";
pub const ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1: u64 = 1024 * 1024;
pub const ATTACHMENT_TRANSLATION_MAX_RESULT_BYTES_V1: u64 = 64 * 1024;
pub const ATTACHMENT_TRANSLATION_MAX_CONFIDENCE_BASIS_POINTS_V1: u32 = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationLanguageV1 {
    English,
    Russian,
    Spanish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationDetectedLanguageV1 {
    Unknown,
    English,
    Russian,
    Spanish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationCompletenessV1 {
    Complete,
    Partial,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationRejectionCodeV1 {
    InvalidRequest,
    SourceRejected,
    InferenceRejected,
    ResultRejected,
    Policy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTranslationDraftV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub source_extraction_run_id: [u8; 16],
    pub expected_source_revision: u64,
    pub target_language: AttachmentTranslationLanguageV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTranslationArtifactV1 {
    pub artifact_id: [u8; 16],
    pub translated_sha256: [u8; 32],
    pub translated_size_bytes: u64,
    pub detected_source_language: AttachmentTranslationDetectedLanguageV1,
    pub target_language: AttachmentTranslationLanguageV1,
    pub completeness: AttachmentTranslationCompletenessV1,
    pub confidence_basis_points: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTranslationPendingResultV1 {
    pub translated_sha256: [u8; 32],
    pub translated_size_bytes: u64,
    pub detected_source_language: AttachmentTranslationDetectedLanguageV1,
    pub target_language: AttachmentTranslationLanguageV1,
    pub completeness: AttachmentTranslationCompletenessV1,
    pub confidence_basis_points: u32,
    pub inference_request_digest: [u8; 32],
    pub source_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationStateV1 {
    Accepted,
    AwaitingSource,
    AwaitingInference,
    MaterializingResult,
    Ready,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTranslationStatusV1 {
    pub state: AttachmentTranslationStateV1,
    pub state_revision: u64,
    pub source_sha256: Option<[u8; 32]>,
    pub inference_request_digest: Option<[u8; 32]>,
    pub pending_result: Option<AttachmentTranslationPendingResultV1>,
    pub artifact: Option<AttachmentTranslationArtifactV1>,
    pub rejection: Option<AttachmentTranslationRejectionCodeV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationTransitionV1 {
    RequestSource,
    SourcePrepared {
        source_sha256: [u8; 32],
        source_size_bytes: u64,
        inference_request_digest: [u8; 32],
    },
    InferenceCompleted(AttachmentTranslationPendingResultV1),
    ResultMaterialized {
        artifact_id: [u8; 16],
    },
    Reject(AttachmentTranslationRejectionCodeV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationValidationErrorV1 {
    InvalidRunId,
    InvalidOperationId,
    InvalidSourceExtractionRunId,
    InvalidSourceRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationTransitionErrorV1 {
    InvalidTransition,
    InvalidSourceReceipt,
    InvalidResult,
    DigestMismatch,
    RevisionExhausted,
}

pub fn validate_attachment_translation_draft_v1(
    draft: &AttachmentTranslationDraftV1,
) -> Result<(), AttachmentTranslationValidationErrorV1> {
    if zero(&draft.run_id) {
        return Err(AttachmentTranslationValidationErrorV1::InvalidRunId);
    }
    if zero(&draft.operation_id) {
        return Err(AttachmentTranslationValidationErrorV1::InvalidOperationId);
    }
    if zero(&draft.source_extraction_run_id) {
        return Err(AttachmentTranslationValidationErrorV1::InvalidSourceExtractionRunId);
    }
    if draft.expected_source_revision == 0 {
        return Err(AttachmentTranslationValidationErrorV1::InvalidSourceRevision);
    }
    Ok(())
}

#[must_use]
pub fn accepted_attachment_translation_status_v1() -> AttachmentTranslationStatusV1 {
    AttachmentTranslationStatusV1 {
        state: AttachmentTranslationStateV1::Accepted,
        state_revision: 1,
        source_sha256: None,
        inference_request_digest: None,
        pending_result: None,
        artifact: None,
        rejection: None,
    }
}

pub fn transition_attachment_translation_v1(
    current: &AttachmentTranslationStatusV1,
    transition: AttachmentTranslationTransitionV1,
) -> Result<AttachmentTranslationStatusV1, AttachmentTranslationTransitionErrorV1> {
    let next_revision = current
        .state_revision
        .checked_add(1)
        .ok_or(AttachmentTranslationTransitionErrorV1::RevisionExhausted)?;
    match (current.state, transition) {
        (
            AttachmentTranslationStateV1::Accepted,
            AttachmentTranslationTransitionV1::RequestSource,
        ) => Ok(AttachmentTranslationStatusV1 {
            state: AttachmentTranslationStateV1::AwaitingSource,
            state_revision: next_revision,
            ..current.clone()
        }),
        (
            AttachmentTranslationStateV1::AwaitingSource,
            AttachmentTranslationTransitionV1::SourcePrepared {
                source_sha256,
                source_size_bytes,
                inference_request_digest,
            },
        ) => {
            if zero(&source_sha256)
                || source_size_bytes == 0
                || source_size_bytes > ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1
                || zero(&inference_request_digest)
            {
                return Err(AttachmentTranslationTransitionErrorV1::InvalidSourceReceipt);
            }
            Ok(AttachmentTranslationStatusV1 {
                state: AttachmentTranslationStateV1::AwaitingInference,
                state_revision: next_revision,
                source_sha256: Some(source_sha256),
                inference_request_digest: Some(inference_request_digest),
                pending_result: None,
                artifact: None,
                rejection: None,
            })
        }
        (
            AttachmentTranslationStateV1::AwaitingInference,
            AttachmentTranslationTransitionV1::InferenceCompleted(result),
        ) => {
            validate_pending_result(&result)?;
            if current.inference_request_digest != Some(result.inference_request_digest)
                || current.source_sha256 != Some(result.source_sha256)
            {
                return Err(AttachmentTranslationTransitionErrorV1::DigestMismatch);
            }
            Ok(AttachmentTranslationStatusV1 {
                state: AttachmentTranslationStateV1::MaterializingResult,
                state_revision: next_revision,
                pending_result: Some(result),
                artifact: None,
                rejection: None,
                ..current.clone()
            })
        }
        (
            AttachmentTranslationStateV1::MaterializingResult,
            AttachmentTranslationTransitionV1::ResultMaterialized { artifact_id },
        ) => {
            if zero(&artifact_id) {
                return Err(AttachmentTranslationTransitionErrorV1::InvalidResult);
            }
            let pending = current
                .pending_result
                .as_ref()
                .ok_or(AttachmentTranslationTransitionErrorV1::InvalidResult)?;
            Ok(AttachmentTranslationStatusV1 {
                state: AttachmentTranslationStateV1::Ready,
                state_revision: next_revision,
                pending_result: None,
                artifact: Some(AttachmentTranslationArtifactV1 {
                    artifact_id,
                    translated_sha256: pending.translated_sha256,
                    translated_size_bytes: pending.translated_size_bytes,
                    detected_source_language: pending.detected_source_language,
                    target_language: pending.target_language,
                    completeness: pending.completeness,
                    confidence_basis_points: pending.confidence_basis_points,
                }),
                rejection: None,
                ..current.clone()
            })
        }
        (
            AttachmentTranslationStateV1::Accepted
            | AttachmentTranslationStateV1::AwaitingSource
            | AttachmentTranslationStateV1::AwaitingInference
            | AttachmentTranslationStateV1::MaterializingResult,
            AttachmentTranslationTransitionV1::Reject(rejection),
        ) => Ok(AttachmentTranslationStatusV1 {
            state: AttachmentTranslationStateV1::Rejected,
            state_revision: next_revision,
            pending_result: None,
            artifact: None,
            rejection: Some(rejection),
            ..current.clone()
        }),
        _ => Err(AttachmentTranslationTransitionErrorV1::InvalidTransition),
    }
}

fn validate_pending_result(
    result: &AttachmentTranslationPendingResultV1,
) -> Result<(), AttachmentTranslationTransitionErrorV1> {
    if zero(&result.translated_sha256)
        || result.translated_size_bytes == 0
        || result.translated_size_bytes > ATTACHMENT_TRANSLATION_MAX_RESULT_BYTES_V1
        || result.confidence_basis_points > ATTACHMENT_TRANSLATION_MAX_CONFIDENCE_BASIS_POINTS_V1
        || zero(&result.inference_request_digest)
        || zero(&result.source_sha256)
    {
        return Err(AttachmentTranslationTransitionErrorV1::InvalidResult);
    }
    Ok(())
}

fn zero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> AttachmentTranslationDraftV1 {
        AttachmentTranslationDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            source_extraction_run_id: [3; 16],
            expected_source_revision: 7,
            target_language: AttachmentTranslationLanguageV1::Russian,
        }
    }

    fn result() -> AttachmentTranslationPendingResultV1 {
        AttachmentTranslationPendingResultV1 {
            translated_sha256: [4; 32],
            translated_size_bytes: 512,
            detected_source_language: AttachmentTranslationDetectedLanguageV1::English,
            target_language: AttachmentTranslationLanguageV1::Russian,
            completeness: AttachmentTranslationCompletenessV1::Complete,
            confidence_basis_points: 8_500,
            inference_request_digest: [5; 32],
            source_sha256: [6; 32],
        }
    }

    #[test]
    fn validates_exact_provider_neutral_source_identity() {
        assert_eq!(validate_attachment_translation_draft_v1(&draft()), Ok(()));
        let mut invalid = draft();
        invalid.source_extraction_run_id = [0; 16];
        assert_eq!(
            validate_attachment_translation_draft_v1(&invalid),
            Err(AttachmentTranslationValidationErrorV1::InvalidSourceExtractionRunId)
        );
    }

    #[test]
    fn reaches_ready_without_storing_private_translation_bytes() {
        let accepted = accepted_attachment_translation_status_v1();
        let awaiting_source = transition_attachment_translation_v1(
            &accepted,
            AttachmentTranslationTransitionV1::RequestSource,
        )
        .expect("request source");
        let awaiting_inference = transition_attachment_translation_v1(
            &awaiting_source,
            AttachmentTranslationTransitionV1::SourcePrepared {
                source_sha256: [6; 32],
                source_size_bytes: 4096,
                inference_request_digest: [5; 32],
            },
        )
        .expect("source prepared");
        let materializing = transition_attachment_translation_v1(
            &awaiting_inference,
            AttachmentTranslationTransitionV1::InferenceCompleted(result()),
        )
        .expect("inference complete");
        let ready = transition_attachment_translation_v1(
            &materializing,
            AttachmentTranslationTransitionV1::ResultMaterialized {
                artifact_id: [7; 16],
            },
        )
        .expect("result materialized");
        assert_eq!(ready.state, AttachmentTranslationStateV1::Ready);
        assert_eq!(ready.artifact.expect("artifact").translated_size_bytes, 512);
        assert!(ready.pending_result.is_none());
    }

    #[test]
    fn rejects_digest_drift_and_oversized_source() {
        let awaiting_source = transition_attachment_translation_v1(
            &accepted_attachment_translation_status_v1(),
            AttachmentTranslationTransitionV1::RequestSource,
        )
        .expect("request source");
        assert_eq!(
            transition_attachment_translation_v1(
                &awaiting_source,
                AttachmentTranslationTransitionV1::SourcePrepared {
                    source_sha256: [6; 32],
                    source_size_bytes: ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1 + 1,
                    inference_request_digest: [5; 32],
                },
            ),
            Err(AttachmentTranslationTransitionErrorV1::InvalidSourceReceipt)
        );
        let awaiting_inference = transition_attachment_translation_v1(
            &awaiting_source,
            AttachmentTranslationTransitionV1::SourcePrepared {
                source_sha256: [6; 32],
                source_size_bytes: 1024,
                inference_request_digest: [5; 32],
            },
        )
        .expect("source prepared");
        let mut drifted = result();
        drifted.source_sha256 = [9; 32];
        assert_eq!(
            transition_attachment_translation_v1(
                &awaiting_inference,
                AttachmentTranslationTransitionV1::InferenceCompleted(drifted),
            ),
            Err(AttachmentTranslationTransitionErrorV1::DigestMismatch)
        );
    }
}
