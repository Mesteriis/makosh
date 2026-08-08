use makosh_communication_summary_api::{
    COMMUNICATION_SUMMARY_CONTRACT_MAJOR_V1,
    wire::{
        CommunicationSummaryCandidateV1, CommunicationSummaryCompletenessV1 as WireCompleteness,
        CommunicationSummaryErrorCodeV1 as WireError,
        CommunicationSummaryLanguageV1 as WireLanguage, CommunicationSummaryLengthV1 as WireLength,
        CommunicationSummaryStateV1 as WireState, GetCommunicationSummaryRequestV1,
        GetCommunicationSummaryResponseV1, StartCommunicationSummaryRequestV1,
        StartCommunicationSummaryResponseV1,
    },
};
use makosh_communication_summary_core::{
    CommunicationSummaryCompletenessV1, CommunicationSummaryDraftV1,
    CommunicationSummaryLanguageV1, CommunicationSummaryLengthV1,
    CommunicationSummaryRejectionCodeV1, CommunicationSummaryStateV1,
};
use makosh_communication_summary_persistence::{
    CommunicationSummaryPersistenceErrorV1, CommunicationSummaryPersistenceV1,
    CreateCommunicationSummaryOutcomeV1, CreateCommunicationSummaryRunV1,
    PersistedCommunicationSummaryRunV1,
};
use makosh_communications_ai_source_api::{
    CommunicationSummarySourceEnvelopeContextV1,
    build_communication_summary_source_prepare_outbox_record_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub struct CommunicationSummaryClientRuntimeContextV1<'a> {
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
}

pub async fn start_communication_summary_payload_v1(
    persistence: &CommunicationSummaryPersistenceV1,
    logical_owner_id: &str,
    runtime: &CommunicationSummaryClientRuntimeContextV1<'_>,
    payload: &[u8],
    now_unix_millis: i64,
) -> Vec<u8> {
    let Ok(request) = StartCommunicationSummaryRequestV1::decode(payload) else {
        return start_error(
            Vec::new(),
            WireError::CommunicationSummaryErrorCodeInvalidRequest,
        );
    };
    let response_operation_id = request.operation_id.clone();
    let Some(draft) = start_draft(logical_owner_id, request) else {
        return start_error(
            response_operation_id,
            WireError::CommunicationSummaryErrorCodeInvalidRequest,
        );
    };
    let Some(record) = source_prepare_record(logical_owner_id, runtime, &draft, now_unix_millis)
    else {
        return start_error(
            draft.run_id.to_vec(),
            WireError::CommunicationSummaryErrorCodeUnavailable,
        );
    };
    let created = persistence
        .create_run(CreateCommunicationSummaryRunV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            draft,
            source_prepare_message_id: *record.message_id(),
            source_prepare_envelope_sha256: *record.envelope_sha256(),
            source_prepare_envelope_bytes: record.exact_bytes().to_vec(),
            created_at_unix_millis: now_unix_millis,
        })
        .await;
    let persisted = match created {
        Ok(CreateCommunicationSummaryOutcomeV1::Created(value))
        | Ok(CreateCommunicationSummaryOutcomeV1::Existing(value)) => value,
        Err(CommunicationSummaryPersistenceErrorV1::RequestConflict) => {
            return start_error(
                response_operation_id,
                WireError::CommunicationSummaryErrorCodeInvalidRequest,
            );
        }
        Err(_) => {
            return start_error(
                response_operation_id,
                WireError::CommunicationSummaryErrorCodeUnavailable,
            );
        }
    };
    let persisted = if persisted.status.state == CommunicationSummaryStateV1::Accepted {
        match persistence
            .begin_source_preparation(logical_owner_id, &persisted.draft.run_id, now_unix_millis)
            .await
        {
            Ok(value) => value,
            Err(_) => {
                return start_error(
                    persisted.draft.run_id.to_vec(),
                    WireError::CommunicationSummaryErrorCodeUnavailable,
                );
            }
        }
    } else {
        persisted
    };
    StartCommunicationSummaryResponseV1 {
        run_id: persisted.draft.run_id.to_vec(),
        state: wire_state(persisted.status.state) as i32,
        error: rejection_error(persisted.status.rejection) as i32,
    }
    .encode_to_vec()
}

pub async fn get_communication_summary_payload_v1(
    persistence: &CommunicationSummaryPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetCommunicationSummaryRequestV1::decode(payload) else {
        return get_error(
            Vec::new(),
            WireError::CommunicationSummaryErrorCodeInvalidRequest,
        );
    };
    let response_run_id = request.run_id.clone();
    let Ok(run_id) = id16(&request.run_id) else {
        return get_error(
            response_run_id,
            WireError::CommunicationSummaryErrorCodeInvalidRequest,
        );
    };
    if request.protocol_major != COMMUNICATION_SUMMARY_CONTRACT_MAJOR_V1 {
        return get_error(
            response_run_id,
            WireError::CommunicationSummaryErrorCodeInvalidRequest,
        );
    }
    match persistence.load_run(logical_owner_id, &run_id).await {
        Ok(run) => get_response(run),
        Err(CommunicationSummaryPersistenceErrorV1::NotFound) => get_error(
            response_run_id,
            WireError::CommunicationSummaryErrorCodeNotFound,
        ),
        Err(_) => get_error(
            response_run_id,
            WireError::CommunicationSummaryErrorCodeUnavailable,
        ),
    }
}

fn start_draft(
    logical_owner_id: &str,
    request: StartCommunicationSummaryRequestV1,
) -> Option<CommunicationSummaryDraftV1> {
    if request.protocol_major != COMMUNICATION_SUMMARY_CONTRACT_MAJOR_V1 {
        return None;
    }
    let operation_id = id16(&request.operation_id).ok()?;
    Some(CommunicationSummaryDraftV1 {
        run_id: run_id(logical_owner_id, &operation_id),
        operation_id,
        source_message_id: id16(&request.source_message_id).ok()?,
        expected_source_revision: (request.expected_source_revision > 0)
            .then_some(request.expected_source_revision)?,
        length: length(request.length)?,
        language: language(request.language)?,
    })
}

fn source_prepare_record(
    logical_owner_id: &str,
    runtime: &CommunicationSummaryClientRuntimeContextV1<'_>,
    draft: &CommunicationSummaryDraftV1,
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
    build_communication_summary_source_prepare_outbox_record_v1(
        draft.run_id,
        draft.source_message_id,
        draft.expected_source_revision,
        logical_owner_id,
        deadline,
        &CommunicationSummarySourceEnvelopeContextV1 {
            module_id: makosh_communication_summary_api::COMMUNICATION_SUMMARY_MODULE_ID_V1
                .to_owned(),
            runtime_instance_id: runtime.runtime_instance_id.to_owned(),
            runtime_generation: runtime.runtime_generation,
            recorded_at_unix_seconds: seconds,
            recorded_at_nanos: nanos,
        },
    )
    .ok()
}

fn get_response(run: PersistedCommunicationSummaryRunV1) -> Vec<u8> {
    let candidate = run
        .status
        .candidate
        .map(|value| CommunicationSummaryCandidateV1 {
            summary_utf8: value.summary_utf8,
            resolved_length: wire_length(value.resolved_length) as i32,
            resolved_language: wire_language(value.resolved_language) as i32,
            completeness: wire_completeness(value.completeness) as i32,
            confidence_basis_points: value.confidence_basis_points,
        });
    GetCommunicationSummaryResponseV1 {
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
    digest.update(b"makosh.communication_summary.run.v1\0");
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

fn length(value: i32) -> Option<CommunicationSummaryLengthV1> {
    match WireLength::try_from(value).ok()? {
        WireLength::CommunicationSummaryLengthShort => Some(CommunicationSummaryLengthV1::Short),
        WireLength::CommunicationSummaryLengthStandard => {
            Some(CommunicationSummaryLengthV1::Standard)
        }
        WireLength::CommunicationSummaryLengthDetailed => {
            Some(CommunicationSummaryLengthV1::Detailed)
        }
        WireLength::CommunicationSummaryLengthUnspecified => None,
    }
}

fn language(value: i32) -> Option<CommunicationSummaryLanguageV1> {
    match WireLanguage::try_from(value).ok()? {
        WireLanguage::CommunicationSummaryLanguageAuto => {
            Some(CommunicationSummaryLanguageV1::Auto)
        }
        WireLanguage::CommunicationSummaryLanguageEnglish => {
            Some(CommunicationSummaryLanguageV1::English)
        }
        WireLanguage::CommunicationSummaryLanguageRussian => {
            Some(CommunicationSummaryLanguageV1::Russian)
        }
        WireLanguage::CommunicationSummaryLanguageSpanish => {
            Some(CommunicationSummaryLanguageV1::Spanish)
        }
        WireLanguage::CommunicationSummaryLanguageUnspecified => None,
    }
}

pub(crate) const fn wire_state(value: CommunicationSummaryStateV1) -> WireState {
    match value {
        CommunicationSummaryStateV1::Accepted => WireState::CommunicationSummaryStateAccepted,
        CommunicationSummaryStateV1::PreparingSource => {
            WireState::CommunicationSummaryStatePreparingSource
        }
        CommunicationSummaryStateV1::AwaitingInference => {
            WireState::CommunicationSummaryStateAwaitingInference
        }
        CommunicationSummaryStateV1::Ready => WireState::CommunicationSummaryStateReady,
        CommunicationSummaryStateV1::Rejected => WireState::CommunicationSummaryStateRejected,
    }
}

const fn wire_length(value: CommunicationSummaryLengthV1) -> WireLength {
    match value {
        CommunicationSummaryLengthV1::Short => WireLength::CommunicationSummaryLengthShort,
        CommunicationSummaryLengthV1::Standard => WireLength::CommunicationSummaryLengthStandard,
        CommunicationSummaryLengthV1::Detailed => WireLength::CommunicationSummaryLengthDetailed,
    }
}

const fn wire_language(value: CommunicationSummaryLanguageV1) -> WireLanguage {
    match value {
        CommunicationSummaryLanguageV1::Auto => WireLanguage::CommunicationSummaryLanguageAuto,
        CommunicationSummaryLanguageV1::English => {
            WireLanguage::CommunicationSummaryLanguageEnglish
        }
        CommunicationSummaryLanguageV1::Russian => {
            WireLanguage::CommunicationSummaryLanguageRussian
        }
        CommunicationSummaryLanguageV1::Spanish => {
            WireLanguage::CommunicationSummaryLanguageSpanish
        }
    }
}

const fn wire_completeness(value: CommunicationSummaryCompletenessV1) -> WireCompleteness {
    match value {
        CommunicationSummaryCompletenessV1::Complete => {
            WireCompleteness::CommunicationSummaryCompletenessComplete
        }
        CommunicationSummaryCompletenessV1::Partial => {
            WireCompleteness::CommunicationSummaryCompletenessPartial
        }
    }
}

pub(crate) const fn rejection_error(
    value: Option<CommunicationSummaryRejectionCodeV1>,
) -> WireError {
    match value {
        None => WireError::CommunicationSummaryErrorCodeUnspecified,
        Some(CommunicationSummaryRejectionCodeV1::InvalidRequest) => {
            WireError::CommunicationSummaryErrorCodeInvalidRequest
        }
        Some(CommunicationSummaryRejectionCodeV1::SourceRejected) => {
            WireError::CommunicationSummaryErrorCodeSourceRejected
        }
        Some(CommunicationSummaryRejectionCodeV1::InferenceRejected) => {
            WireError::CommunicationSummaryErrorCodeInferenceRejected
        }
        Some(CommunicationSummaryRejectionCodeV1::Policy) => {
            WireError::CommunicationSummaryErrorCodePolicy
        }
    }
}

fn start_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    StartCommunicationSummaryResponseV1 {
        run_id,
        state: WireState::CommunicationSummaryStateUnspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn get_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    GetCommunicationSummaryResponseV1 {
        run_id,
        source_message_id: Vec::new(),
        expected_source_revision: 0,
        state: WireState::CommunicationSummaryStateUnspecified as i32,
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
            StartCommunicationSummaryRequestV1 {
                protocol_major: 1,
                operation_id: vec![1; 16],
                source_message_id: vec![2; 16],
                expected_source_revision: 3,
                language: WireLanguage::CommunicationSummaryLanguageRussian as i32,
                length: WireLength::CommunicationSummaryLengthShort as i32,
            },
        )
        .expect("draft");
        assert_eq!(draft.source_message_id, [2; 16]);
        assert_eq!(draft.expected_source_revision, 3);
        assert_eq!(draft.length, CommunicationSummaryLengthV1::Short);
        assert_eq!(draft.language, CommunicationSummaryLanguageV1::Russian);
    }
}
