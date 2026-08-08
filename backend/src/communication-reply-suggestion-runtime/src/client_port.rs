use makosh_communication_reply_suggestion_api::{
    COMMUNICATION_REPLY_SUGGESTION_CONTRACT_MAJOR_V1,
    wire::{
        GetReplySuggestionRequestV1, GetReplySuggestionResponseV1, ReplySuggestionCandidateV1,
        ReplySuggestionCompletenessV1 as WireCompleteness, ReplySuggestionErrorCodeV1 as WireError,
        ReplySuggestionLanguageV1 as WireLanguage, ReplySuggestionStateV1 as WireState,
        ReplySuggestionToneV1 as WireTone, StartReplySuggestionRequestV1,
        StartReplySuggestionResponseV1,
    },
};
use makosh_communication_reply_suggestion_core::{
    ReplySuggestionCompletenessV1, ReplySuggestionDraftV1, ReplySuggestionLanguageV1,
    ReplySuggestionRejectionCodeV1, ReplySuggestionStateV1, ReplySuggestionToneV1,
};
use makosh_communication_reply_suggestion_persistence::{
    CommunicationReplySuggestionPersistenceV1, CreateReplySuggestionOutcomeV1,
    CreateReplySuggestionRunV1, PersistedReplySuggestionRunV1, ReplySuggestionPersistenceErrorV1,
};
use makosh_communications_ai_source_api::{
    CommunicationReplySourceEnvelopeContextV1,
    build_communication_reply_source_prepare_outbox_record_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub struct ReplySuggestionClientRuntimeContextV1<'a> {
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
}

pub async fn start_reply_suggestion_payload_v1(
    persistence: &CommunicationReplySuggestionPersistenceV1,
    logical_owner_id: &str,
    runtime: &ReplySuggestionClientRuntimeContextV1<'_>,
    payload: &[u8],
    now_unix_millis: i64,
) -> Vec<u8> {
    let Ok(request) = StartReplySuggestionRequestV1::decode(payload) else {
        return start_error(
            Vec::new(),
            WireError::ReplySuggestionErrorCodeInvalidRequest,
        );
    };
    let response_operation_id = request.operation_id.clone();
    let Some(draft) = start_draft(logical_owner_id, request) else {
        return start_error(
            response_operation_id,
            WireError::ReplySuggestionErrorCodeInvalidRequest,
        );
    };
    let Some(record) = source_prepare_record(logical_owner_id, runtime, &draft, now_unix_millis)
    else {
        return start_error(
            draft.run_id.to_vec(),
            WireError::ReplySuggestionErrorCodeUnavailable,
        );
    };
    let created = persistence
        .create_run(CreateReplySuggestionRunV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            draft,
            source_prepare_message_id: *record.message_id(),
            source_prepare_envelope_sha256: *record.envelope_sha256(),
            source_prepare_envelope_bytes: record.exact_bytes().to_vec(),
            created_at_unix_millis: now_unix_millis,
        })
        .await;
    let persisted = match created {
        Ok(CreateReplySuggestionOutcomeV1::Created(value))
        | Ok(CreateReplySuggestionOutcomeV1::Existing(value)) => value,
        Err(ReplySuggestionPersistenceErrorV1::RequestConflict) => {
            return start_error(
                response_operation_id,
                WireError::ReplySuggestionErrorCodeInvalidRequest,
            );
        }
        Err(_) => {
            return start_error(
                response_operation_id,
                WireError::ReplySuggestionErrorCodeUnavailable,
            );
        }
    };
    let persisted = if persisted.status.state == ReplySuggestionStateV1::Accepted {
        match persistence
            .begin_source_preparation(logical_owner_id, &persisted.draft.run_id, now_unix_millis)
            .await
        {
            Ok(value) => value,
            Err(_) => {
                return start_error(
                    persisted.draft.run_id.to_vec(),
                    WireError::ReplySuggestionErrorCodeUnavailable,
                );
            }
        }
    } else {
        persisted
    };
    StartReplySuggestionResponseV1 {
        run_id: persisted.draft.run_id.to_vec(),
        state: wire_state(persisted.status.state) as i32,
        error: rejection_error(persisted.status.rejection) as i32,
    }
    .encode_to_vec()
}

pub async fn get_reply_suggestion_payload_v1(
    persistence: &CommunicationReplySuggestionPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetReplySuggestionRequestV1::decode(payload) else {
        return get_error(
            Vec::new(),
            WireError::ReplySuggestionErrorCodeInvalidRequest,
        );
    };
    let response_run_id = request.run_id.clone();
    let Ok(run_id) = id16(&request.run_id) else {
        return get_error(
            response_run_id,
            WireError::ReplySuggestionErrorCodeInvalidRequest,
        );
    };
    if request.protocol_major != COMMUNICATION_REPLY_SUGGESTION_CONTRACT_MAJOR_V1 {
        return get_error(
            response_run_id,
            WireError::ReplySuggestionErrorCodeInvalidRequest,
        );
    }
    match persistence.load_run(logical_owner_id, &run_id).await {
        Ok(run) => get_response(run),
        Err(ReplySuggestionPersistenceErrorV1::NotFound) => {
            get_error(response_run_id, WireError::ReplySuggestionErrorCodeNotFound)
        }
        Err(_) => get_error(
            response_run_id,
            WireError::ReplySuggestionErrorCodeUnavailable,
        ),
    }
}

fn start_draft(
    logical_owner_id: &str,
    request: StartReplySuggestionRequestV1,
) -> Option<ReplySuggestionDraftV1> {
    if request.protocol_major != COMMUNICATION_REPLY_SUGGESTION_CONTRACT_MAJOR_V1 {
        return None;
    }
    let operation_id = id16(&request.operation_id).ok()?;
    Some(ReplySuggestionDraftV1 {
        run_id: run_id(logical_owner_id, &operation_id),
        operation_id,
        source_message_id: id16(&request.source_message_id).ok()?,
        expected_source_revision: (request.expected_source_revision > 0)
            .then_some(request.expected_source_revision)?,
        tone: tone(request.tone)?,
        language: language(request.language)?,
    })
}

fn source_prepare_record(
    logical_owner_id: &str,
    runtime: &ReplySuggestionClientRuntimeContextV1<'_>,
    draft: &ReplySuggestionDraftV1,
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
    build_communication_reply_source_prepare_outbox_record_v1(
        draft.run_id,
        draft.source_message_id,
        draft.expected_source_revision,
        logical_owner_id,
        deadline,
        &CommunicationReplySourceEnvelopeContextV1 {
            module_id:
                makosh_communication_reply_suggestion_api::COMMUNICATION_REPLY_SUGGESTION_MODULE_ID_V1
                    .to_owned(),
            runtime_instance_id: runtime.runtime_instance_id.to_owned(),
            runtime_generation: runtime.runtime_generation,
            recorded_at_unix_seconds: seconds,
            recorded_at_nanos: nanos,
        },
    )
    .ok()
}

fn get_response(run: PersistedReplySuggestionRunV1) -> Vec<u8> {
    let candidate = run
        .status
        .candidate
        .map(|value| ReplySuggestionCandidateV1 {
            subject_utf8: value.subject_utf8,
            body_utf8: value.body_utf8,
            resolved_tone: wire_tone(value.resolved_tone) as i32,
            resolved_language: wire_language(value.resolved_language) as i32,
            completeness: wire_completeness(value.completeness) as i32,
            confidence_basis_points: value.confidence_basis_points,
        });
    GetReplySuggestionResponseV1 {
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
    digest.update(b"makosh.communication_reply_suggestion.run.v1\0");
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

fn tone(value: i32) -> Option<ReplySuggestionToneV1> {
    match WireTone::try_from(value).ok()? {
        WireTone::ReplySuggestionToneProfessional => Some(ReplySuggestionToneV1::Professional),
        WireTone::ReplySuggestionToneFriendly => Some(ReplySuggestionToneV1::Friendly),
        WireTone::ReplySuggestionToneConcise => Some(ReplySuggestionToneV1::Concise),
        WireTone::ReplySuggestionToneFormal => Some(ReplySuggestionToneV1::Formal),
        WireTone::ReplySuggestionToneUnspecified => None,
    }
}

fn language(value: i32) -> Option<ReplySuggestionLanguageV1> {
    match WireLanguage::try_from(value).ok()? {
        WireLanguage::ReplySuggestionLanguageSource => Some(ReplySuggestionLanguageV1::Source),
        WireLanguage::ReplySuggestionLanguageEnglish => Some(ReplySuggestionLanguageV1::English),
        WireLanguage::ReplySuggestionLanguageRussian => Some(ReplySuggestionLanguageV1::Russian),
        WireLanguage::ReplySuggestionLanguageSpanish => Some(ReplySuggestionLanguageV1::Spanish),
        WireLanguage::ReplySuggestionLanguageUnspecified => None,
    }
}

pub(crate) const fn wire_state(value: ReplySuggestionStateV1) -> WireState {
    match value {
        ReplySuggestionStateV1::Accepted => WireState::ReplySuggestionStateAccepted,
        ReplySuggestionStateV1::PreparingSource => WireState::ReplySuggestionStatePreparingSource,
        ReplySuggestionStateV1::AwaitingInference => {
            WireState::ReplySuggestionStateAwaitingInference
        }
        ReplySuggestionStateV1::Ready => WireState::ReplySuggestionStateReady,
        ReplySuggestionStateV1::Rejected => WireState::ReplySuggestionStateRejected,
    }
}

const fn wire_tone(value: ReplySuggestionToneV1) -> WireTone {
    match value {
        ReplySuggestionToneV1::Professional => WireTone::ReplySuggestionToneProfessional,
        ReplySuggestionToneV1::Friendly => WireTone::ReplySuggestionToneFriendly,
        ReplySuggestionToneV1::Concise => WireTone::ReplySuggestionToneConcise,
        ReplySuggestionToneV1::Formal => WireTone::ReplySuggestionToneFormal,
    }
}

const fn wire_language(value: ReplySuggestionLanguageV1) -> WireLanguage {
    match value {
        ReplySuggestionLanguageV1::Source => WireLanguage::ReplySuggestionLanguageSource,
        ReplySuggestionLanguageV1::English => WireLanguage::ReplySuggestionLanguageEnglish,
        ReplySuggestionLanguageV1::Russian => WireLanguage::ReplySuggestionLanguageRussian,
        ReplySuggestionLanguageV1::Spanish => WireLanguage::ReplySuggestionLanguageSpanish,
    }
}

const fn wire_completeness(value: ReplySuggestionCompletenessV1) -> WireCompleteness {
    match value {
        ReplySuggestionCompletenessV1::Complete => {
            WireCompleteness::ReplySuggestionCompletenessComplete
        }
        ReplySuggestionCompletenessV1::Partial => {
            WireCompleteness::ReplySuggestionCompletenessPartial
        }
    }
}

pub(crate) const fn rejection_error(value: Option<ReplySuggestionRejectionCodeV1>) -> WireError {
    match value {
        None => WireError::ReplySuggestionErrorCodeUnspecified,
        Some(ReplySuggestionRejectionCodeV1::InvalidRequest) => {
            WireError::ReplySuggestionErrorCodeInvalidRequest
        }
        Some(ReplySuggestionRejectionCodeV1::SourceRejected) => {
            WireError::ReplySuggestionErrorCodeSourceRejected
        }
        Some(ReplySuggestionRejectionCodeV1::InferenceRejected) => {
            WireError::ReplySuggestionErrorCodeInferenceRejected
        }
        Some(ReplySuggestionRejectionCodeV1::Policy) => WireError::ReplySuggestionErrorCodePolicy,
    }
}

fn start_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    StartReplySuggestionResponseV1 {
        run_id,
        state: WireState::ReplySuggestionStateUnspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn get_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    GetReplySuggestionResponseV1 {
        run_id,
        source_message_id: Vec::new(),
        expected_source_revision: 0,
        state: WireState::ReplySuggestionStateUnspecified as i32,
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
            StartReplySuggestionRequestV1 {
                protocol_major: 1,
                operation_id: vec![1; 16],
                source_message_id: vec![2; 16],
                expected_source_revision: 3,
                tone: WireTone::ReplySuggestionToneProfessional as i32,
                language: WireLanguage::ReplySuggestionLanguageRussian as i32,
            },
        )
        .expect("draft");
        assert_eq!(draft.source_message_id, [2; 16]);
        assert_eq!(draft.expected_source_revision, 3);
        assert_eq!(draft.tone, ReplySuggestionToneV1::Professional);
        assert_eq!(draft.language, ReplySuggestionLanguageV1::Russian);
    }
}
