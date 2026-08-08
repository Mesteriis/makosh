use makosh_communication_translation_api::{
    COMMUNICATION_TRANSLATION_CONTRACT_MAJOR_V1,
    wire::{
        CommunicationTranslationCandidateV1,
        CommunicationTranslationCompletenessV1 as WireCompleteness,
        CommunicationTranslationDetectedLanguageV1 as WireDetectedLanguage,
        CommunicationTranslationErrorCodeV1 as WireError,
        CommunicationTranslationLanguageV1 as WireLanguage,
        CommunicationTranslationStateV1 as WireState, GetCommunicationTranslationRequestV1,
        GetCommunicationTranslationResponseV1, StartCommunicationTranslationRequestV1,
        StartCommunicationTranslationResponseV1,
    },
};
use makosh_communication_translation_core::{
    CommunicationTranslationCompletenessV1, CommunicationTranslationDetectedLanguageV1,
    CommunicationTranslationDraftV1, CommunicationTranslationLanguageV1,
    CommunicationTranslationRejectionCodeV1, CommunicationTranslationStateV1,
};
use makosh_communication_translation_persistence::{
    CommunicationTranslationPersistenceErrorV1, CommunicationTranslationPersistenceV1,
    CreateCommunicationTranslationOutcomeV1, CreateCommunicationTranslationRunV1,
    PersistedCommunicationTranslationRunV1,
};
use makosh_communications_ai_source_api::{
    CommunicationTranslationSourceEnvelopeContextV1,
    build_communication_translation_source_prepare_outbox_record_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub struct CommunicationTranslationClientRuntimeContextV1<'a> {
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
}

pub async fn start_communication_translation_payload_v1(
    persistence: &CommunicationTranslationPersistenceV1,
    logical_owner_id: &str,
    runtime: &CommunicationTranslationClientRuntimeContextV1<'_>,
    payload: &[u8],
    now_unix_millis: i64,
) -> Vec<u8> {
    let Ok(request) = StartCommunicationTranslationRequestV1::decode(payload) else {
        return start_error(
            Vec::new(),
            WireError::CommunicationTranslationErrorCodeInvalidRequest,
        );
    };
    let response_operation_id = request.operation_id.clone();
    let Some(draft) = start_draft(logical_owner_id, request) else {
        return start_error(
            response_operation_id,
            WireError::CommunicationTranslationErrorCodeInvalidRequest,
        );
    };
    let Some(record) = source_prepare_record(logical_owner_id, runtime, &draft, now_unix_millis)
    else {
        return start_error(
            draft.run_id.to_vec(),
            WireError::CommunicationTranslationErrorCodeUnavailable,
        );
    };
    let created = persistence
        .create_run(CreateCommunicationTranslationRunV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            draft,
            source_prepare_message_id: *record.message_id(),
            source_prepare_envelope_sha256: *record.envelope_sha256(),
            source_prepare_envelope_bytes: record.exact_bytes().to_vec(),
            created_at_unix_millis: now_unix_millis,
        })
        .await;
    let persisted = match created {
        Ok(CreateCommunicationTranslationOutcomeV1::Created(value))
        | Ok(CreateCommunicationTranslationOutcomeV1::Existing(value)) => value,
        Err(CommunicationTranslationPersistenceErrorV1::RequestConflict) => {
            return start_error(
                response_operation_id,
                WireError::CommunicationTranslationErrorCodeInvalidRequest,
            );
        }
        Err(_) => {
            return start_error(
                response_operation_id,
                WireError::CommunicationTranslationErrorCodeUnavailable,
            );
        }
    };
    let persisted = if persisted.status.state == CommunicationTranslationStateV1::Accepted {
        match persistence
            .begin_source_preparation(logical_owner_id, &persisted.draft.run_id, now_unix_millis)
            .await
        {
            Ok(value) => value,
            Err(_) => {
                return start_error(
                    persisted.draft.run_id.to_vec(),
                    WireError::CommunicationTranslationErrorCodeUnavailable,
                );
            }
        }
    } else {
        persisted
    };
    StartCommunicationTranslationResponseV1 {
        run_id: persisted.draft.run_id.to_vec(),
        state: wire_state(persisted.status.state) as i32,
        error: rejection_error(persisted.status.rejection) as i32,
    }
    .encode_to_vec()
}

pub async fn get_communication_translation_payload_v1(
    persistence: &CommunicationTranslationPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetCommunicationTranslationRequestV1::decode(payload) else {
        return get_error(
            Vec::new(),
            WireError::CommunicationTranslationErrorCodeInvalidRequest,
        );
    };
    let response_run_id = request.run_id.clone();
    let Ok(run_id) = id16(&request.run_id) else {
        return get_error(
            response_run_id,
            WireError::CommunicationTranslationErrorCodeInvalidRequest,
        );
    };
    if request.protocol_major != COMMUNICATION_TRANSLATION_CONTRACT_MAJOR_V1 {
        return get_error(
            response_run_id,
            WireError::CommunicationTranslationErrorCodeInvalidRequest,
        );
    }
    match persistence.load_run(logical_owner_id, &run_id).await {
        Ok(run) => get_response(run),
        Err(CommunicationTranslationPersistenceErrorV1::NotFound) => get_error(
            response_run_id,
            WireError::CommunicationTranslationErrorCodeNotFound,
        ),
        Err(_) => get_error(
            response_run_id,
            WireError::CommunicationTranslationErrorCodeUnavailable,
        ),
    }
}

fn start_draft(
    logical_owner_id: &str,
    request: StartCommunicationTranslationRequestV1,
) -> Option<CommunicationTranslationDraftV1> {
    if request.protocol_major != COMMUNICATION_TRANSLATION_CONTRACT_MAJOR_V1 {
        return None;
    }
    let operation_id = id16(&request.operation_id).ok()?;
    Some(CommunicationTranslationDraftV1 {
        run_id: run_id(logical_owner_id, &operation_id),
        operation_id,
        source_message_id: id16(&request.source_message_id).ok()?,
        expected_source_revision: (request.expected_source_revision > 0)
            .then_some(request.expected_source_revision)?,
        target_language: target_language(request.target_language)?,
    })
}

fn source_prepare_record(
    logical_owner_id: &str,
    runtime: &CommunicationTranslationClientRuntimeContextV1<'_>,
    draft: &CommunicationTranslationDraftV1,
    now_unix_millis: i64,
) -> Option<makosh_events_protocol::delivery::OutboxRecordV1> {
    if now_unix_millis <= 0
        || runtime.runtime_instance_id.is_empty()
        || runtime.runtime_generation == 0
    {
        return None;
    }
    let seconds = now_unix_millis / 1_000;
    let deadline = seconds.checked_add(300)?;
    let nanos = i32::try_from((now_unix_millis % 1_000) * 1_000_000).ok()?;
    build_communication_translation_source_prepare_outbox_record_v1(
        draft.run_id,
        draft.source_message_id,
        draft.expected_source_revision,
        logical_owner_id,
        deadline,
        &CommunicationTranslationSourceEnvelopeContextV1 {
            module_id: makosh_communication_translation_api::COMMUNICATION_TRANSLATION_MODULE_ID_V1
                .to_owned(),
            runtime_instance_id: runtime.runtime_instance_id.to_owned(),
            runtime_generation: runtime.runtime_generation,
            recorded_at_unix_seconds: seconds,
            recorded_at_nanos: nanos,
        },
    )
    .ok()
}

fn get_response(run: PersistedCommunicationTranslationRunV1) -> Vec<u8> {
    let candidate = run
        .status
        .candidate
        .map(|value| CommunicationTranslationCandidateV1 {
            translated_text_utf8: value.translated_text_utf8,
            detected_source_language: wire_detected_language(value.detected_source_language) as i32,
            target_language: wire_target_language(value.target_language) as i32,
            completeness: wire_completeness(value.completeness) as i32,
            confidence_basis_points: value.confidence_basis_points,
        });
    GetCommunicationTranslationResponseV1 {
        run_id: run.draft.run_id.to_vec(),
        source_message_id: run.draft.source_message_id.to_vec(),
        expected_source_revision: run.draft.expected_source_revision,
        state: wire_state(run.status.state) as i32,
        state_revision: run.status.state_revision,
        candidate,
        error: rejection_error(run.status.rejection) as i32,
    }
    .encode_to_vec()
}

fn run_id(logical_owner_id: &str, operation_id: &[u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication_translation.run.v1\0");
    digest.update(logical_owner_id.as_bytes());
    digest.update([0]);
    digest.update(operation_id);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn id16(value: &[u8]) -> Result<[u8; 16], ()> {
    let value: [u8; 16] = value.try_into().map_err(|_| ())?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(())
}

fn target_language(value: i32) -> Option<CommunicationTranslationLanguageV1> {
    match WireLanguage::try_from(value).ok()? {
        WireLanguage::CommunicationTranslationLanguageEnglish => {
            Some(CommunicationTranslationLanguageV1::English)
        }
        WireLanguage::CommunicationTranslationLanguageRussian => {
            Some(CommunicationTranslationLanguageV1::Russian)
        }
        WireLanguage::CommunicationTranslationLanguageSpanish => {
            Some(CommunicationTranslationLanguageV1::Spanish)
        }
        WireLanguage::CommunicationTranslationLanguageUnspecified => None,
    }
}

pub(crate) const fn wire_state(value: CommunicationTranslationStateV1) -> WireState {
    match value {
        CommunicationTranslationStateV1::Accepted => {
            WireState::CommunicationTranslationStateAccepted
        }
        CommunicationTranslationStateV1::PreparingSource => {
            WireState::CommunicationTranslationStatePreparingSource
        }
        CommunicationTranslationStateV1::AwaitingInference => {
            WireState::CommunicationTranslationStateAwaitingInference
        }
        CommunicationTranslationStateV1::Ready => WireState::CommunicationTranslationStateReady,
        CommunicationTranslationStateV1::Rejected => {
            WireState::CommunicationTranslationStateRejected
        }
    }
}

const fn wire_target_language(value: CommunicationTranslationLanguageV1) -> WireLanguage {
    match value {
        CommunicationTranslationLanguageV1::English => {
            WireLanguage::CommunicationTranslationLanguageEnglish
        }
        CommunicationTranslationLanguageV1::Russian => {
            WireLanguage::CommunicationTranslationLanguageRussian
        }
        CommunicationTranslationLanguageV1::Spanish => {
            WireLanguage::CommunicationTranslationLanguageSpanish
        }
    }
}

const fn wire_detected_language(
    value: CommunicationTranslationDetectedLanguageV1,
) -> WireDetectedLanguage {
    match value {
        CommunicationTranslationDetectedLanguageV1::Unknown => {
            WireDetectedLanguage::CommunicationTranslationDetectedLanguageUnknown
        }
        CommunicationTranslationDetectedLanguageV1::English => {
            WireDetectedLanguage::CommunicationTranslationDetectedLanguageEnglish
        }
        CommunicationTranslationDetectedLanguageV1::Russian => {
            WireDetectedLanguage::CommunicationTranslationDetectedLanguageRussian
        }
        CommunicationTranslationDetectedLanguageV1::Spanish => {
            WireDetectedLanguage::CommunicationTranslationDetectedLanguageSpanish
        }
    }
}

const fn wire_completeness(value: CommunicationTranslationCompletenessV1) -> WireCompleteness {
    match value {
        CommunicationTranslationCompletenessV1::Complete => {
            WireCompleteness::CommunicationTranslationCompletenessComplete
        }
        CommunicationTranslationCompletenessV1::Partial => {
            WireCompleteness::CommunicationTranslationCompletenessPartial
        }
    }
}

pub(crate) const fn rejection_error(
    value: Option<CommunicationTranslationRejectionCodeV1>,
) -> WireError {
    match value {
        None => WireError::CommunicationTranslationErrorCodeUnspecified,
        Some(CommunicationTranslationRejectionCodeV1::InvalidRequest) => {
            WireError::CommunicationTranslationErrorCodeInvalidRequest
        }
        Some(CommunicationTranslationRejectionCodeV1::SourceRejected) => {
            WireError::CommunicationTranslationErrorCodeSourceRejected
        }
        Some(CommunicationTranslationRejectionCodeV1::InferenceRejected) => {
            WireError::CommunicationTranslationErrorCodeInferenceRejected
        }
        Some(CommunicationTranslationRejectionCodeV1::Policy) => {
            WireError::CommunicationTranslationErrorCodePolicy
        }
    }
}

fn start_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    StartCommunicationTranslationResponseV1 {
        run_id,
        state: WireState::CommunicationTranslationStateUnspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn get_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    GetCommunicationTranslationResponseV1 {
        run_id,
        source_message_id: Vec::new(),
        expected_source_revision: 0,
        state: WireState::CommunicationTranslationStateUnspecified as i32,
        state_revision: 0,
        candidate: None,
        error: error as i32,
    }
    .encode_to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_identity_is_owner_and_operation_bound() {
        assert_eq!(run_id("owner-1", &[1; 16]), run_id("owner-1", &[1; 16]));
        assert_ne!(run_id("owner-1", &[1; 16]), run_id("owner-2", &[1; 16]));
        assert_ne!(run_id("owner-1", &[1; 16]), run_id("owner-1", &[2; 16]));
    }

    #[test]
    fn start_draft_is_concrete_and_provider_neutral() {
        let draft = start_draft(
            "owner-1",
            StartCommunicationTranslationRequestV1 {
                protocol_major: 1,
                operation_id: vec![1; 16],
                source_message_id: vec![2; 16],
                expected_source_revision: 3,
                target_language: WireLanguage::CommunicationTranslationLanguageRussian as i32,
            },
        )
        .expect("draft");
        assert_eq!(draft.source_message_id, [2; 16]);
        assert_eq!(draft.expected_source_revision, 3);
        assert_eq!(
            draft.target_language,
            CommunicationTranslationLanguageV1::Russian
        );
    }
}
