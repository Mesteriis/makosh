use makosh_communication_note_candidate_api::{
    COMMUNICATION_NOTE_CANDIDATE_CONTRACT_MAJOR_V1,
    wire::{
        CommunicationNoteCandidateCompletenessV1 as WireCompleteness,
        CommunicationNoteCandidateErrorCodeV1 as WireError,
        CommunicationNoteCandidateStateV1 as WireState,
        CommunicationNoteCandidateV1 as WireCandidate,
        CommunicationNoteSourceBasisV1 as WireSourceBasis,
        CommunicationNoteTopicHintV1 as WireTopicHint, GetCommunicationNoteCandidateRequestV1,
        GetCommunicationNoteCandidateResponseV1, StartCommunicationNoteCandidateRequestV1,
        StartCommunicationNoteCandidateResponseV1,
    },
};
use makosh_communication_note_candidate_core::{
    CommunicationNoteCandidateCompletenessV1, CommunicationNoteCandidateDraftV1,
    CommunicationNoteCandidateRejectionCodeV1, CommunicationNoteCandidateStateV1,
    CommunicationNoteCandidateV1, CommunicationNoteSourceBasisV1, CommunicationNoteTopicHintV1,
};
use makosh_communication_note_candidate_persistence::{
    CommunicationNoteCandidatePersistenceErrorV1, CommunicationNoteCandidatePersistenceV1,
    CreateCommunicationNoteCandidateOutcomeV1, CreateCommunicationNoteCandidateRunV1,
    PersistedCommunicationNoteCandidateRunV1,
};
use makosh_communications_note_source_api::{
    CommunicationNoteSourceEnvelopeContextV1,
    build_communication_note_source_prepare_outbox_record_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub struct CommunicationNoteCandidateClientRuntimeContextV1<'a> {
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
}

pub async fn start_communication_note_candidate_payload_v1(
    persistence: &CommunicationNoteCandidatePersistenceV1,
    logical_owner_id: &str,
    runtime: &CommunicationNoteCandidateClientRuntimeContextV1<'_>,
    payload: &[u8],
    now_unix_millis: i64,
) -> Vec<u8> {
    let Ok(request) = StartCommunicationNoteCandidateRequestV1::decode(payload) else {
        return start_error(
            Vec::new(),
            WireError::CommunicationNoteCandidateErrorCodeInvalidRequest,
        );
    };
    let response_operation_id = request.operation_id.clone();
    let Some(draft) = start_draft(logical_owner_id, request) else {
        return start_error(
            response_operation_id,
            WireError::CommunicationNoteCandidateErrorCodeInvalidRequest,
        );
    };
    let Some(record) = source_prepare_record(logical_owner_id, runtime, &draft, now_unix_millis)
    else {
        return start_error(
            draft.run_id.to_vec(),
            WireError::CommunicationNoteCandidateErrorCodeUnavailable,
        );
    };
    let created = persistence
        .create_run(CreateCommunicationNoteCandidateRunV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            draft,
            source_prepare_message_id: *record.message_id(),
            source_prepare_envelope_sha256: *record.envelope_sha256(),
            source_prepare_envelope_bytes: record.exact_bytes().to_vec(),
            created_at_unix_millis: now_unix_millis,
        })
        .await;
    let persisted = match created {
        Ok(CreateCommunicationNoteCandidateOutcomeV1::Created(value))
        | Ok(CreateCommunicationNoteCandidateOutcomeV1::Existing(value)) => value,
        Err(CommunicationNoteCandidatePersistenceErrorV1::RequestConflict) => {
            return start_error(
                response_operation_id,
                WireError::CommunicationNoteCandidateErrorCodeInvalidRequest,
            );
        }
        Err(_) => {
            return start_error(
                response_operation_id,
                WireError::CommunicationNoteCandidateErrorCodeUnavailable,
            );
        }
    };
    let persisted = if persisted.status.state == CommunicationNoteCandidateStateV1::Accepted {
        match persistence
            .begin_source_preparation(logical_owner_id, &persisted.draft.run_id, now_unix_millis)
            .await
        {
            Ok(value) => value,
            Err(_) => {
                return start_error(
                    persisted.draft.run_id.to_vec(),
                    WireError::CommunicationNoteCandidateErrorCodeUnavailable,
                );
            }
        }
    } else {
        persisted
    };
    StartCommunicationNoteCandidateResponseV1 {
        run_id: persisted.draft.run_id.to_vec(),
        state: wire_state(persisted.status.state) as i32,
        error: rejection_error(persisted.status.rejection) as i32,
    }
    .encode_to_vec()
}

pub async fn get_communication_note_candidate_payload_v1(
    persistence: &CommunicationNoteCandidatePersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetCommunicationNoteCandidateRequestV1::decode(payload) else {
        return get_error(
            Vec::new(),
            WireError::CommunicationNoteCandidateErrorCodeInvalidRequest,
        );
    };
    let response_run_id = request.run_id.clone();
    let Ok(run_id) = id16(&request.run_id) else {
        return get_error(
            response_run_id,
            WireError::CommunicationNoteCandidateErrorCodeInvalidRequest,
        );
    };
    if request.protocol_major != COMMUNICATION_NOTE_CANDIDATE_CONTRACT_MAJOR_V1 {
        return get_error(
            response_run_id,
            WireError::CommunicationNoteCandidateErrorCodeInvalidRequest,
        );
    }
    match persistence.load_run(logical_owner_id, &run_id).await {
        Ok(run) => get_response(run),
        Err(CommunicationNoteCandidatePersistenceErrorV1::NotFound) => get_error(
            response_run_id,
            WireError::CommunicationNoteCandidateErrorCodeNotFound,
        ),
        Err(_) => get_error(
            response_run_id,
            WireError::CommunicationNoteCandidateErrorCodeUnavailable,
        ),
    }
}

fn start_draft(
    logical_owner_id: &str,
    request: StartCommunicationNoteCandidateRequestV1,
) -> Option<CommunicationNoteCandidateDraftV1> {
    if request.protocol_major != COMMUNICATION_NOTE_CANDIDATE_CONTRACT_MAJOR_V1 {
        return None;
    }
    let operation_id = id16(&request.operation_id).ok()?;
    Some(CommunicationNoteCandidateDraftV1 {
        run_id: run_id(logical_owner_id, &operation_id),
        operation_id,
        source_message_id: id16(&request.source_message_id).ok()?,
        expected_source_revision: (request.expected_source_revision > 0)
            .then_some(request.expected_source_revision)?,
    })
}

fn source_prepare_record(
    logical_owner_id: &str,
    runtime: &CommunicationNoteCandidateClientRuntimeContextV1<'_>,
    draft: &CommunicationNoteCandidateDraftV1,
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
    build_communication_note_source_prepare_outbox_record_v1(
        draft.run_id,
        draft.source_message_id,
        draft.expected_source_revision,
        logical_owner_id,
        deadline,
        &CommunicationNoteSourceEnvelopeContextV1 {
            module_id:
                makosh_communication_note_candidate_api::COMMUNICATION_NOTE_CANDIDATE_MODULE_ID_V1
                    .to_owned(),
            runtime_instance_id: runtime.runtime_instance_id.to_owned(),
            runtime_generation: runtime.runtime_generation,
            recorded_at_unix_seconds: seconds,
            recorded_at_nanos: nanos,
        },
    )
    .ok()
}

fn get_response(run: PersistedCommunicationNoteCandidateRunV1) -> Vec<u8> {
    let candidates = run
        .status
        .candidates
        .unwrap_or_default()
        .into_iter()
        .map(wire_candidate)
        .collect();
    GetCommunicationNoteCandidateResponseV1 {
        run_id: run.draft.run_id.to_vec(),
        source_message_id: run.draft.source_message_id.to_vec(),
        expected_source_revision: run.draft.expected_source_revision,
        state: wire_state(run.status.state) as i32,
        state_revision: run.status.state_revision,
        candidates,
        completeness: wire_completeness(run.status.completeness) as i32,
        error: rejection_error(run.status.rejection) as i32,
    }
    .encode_to_vec()
}

fn run_id(logical_owner_id: &str, operation_id: &[u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communication_note_candidate.run.v1\0");
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

pub(crate) const fn wire_state(value: CommunicationNoteCandidateStateV1) -> WireState {
    match value {
        CommunicationNoteCandidateStateV1::Accepted => {
            WireState::CommunicationNoteCandidateStateAccepted
        }
        CommunicationNoteCandidateStateV1::PreparingSource => {
            WireState::CommunicationNoteCandidateStatePreparingSource
        }
        CommunicationNoteCandidateStateV1::Extracting => {
            WireState::CommunicationNoteCandidateStateExtracting
        }
        CommunicationNoteCandidateStateV1::Ready => WireState::CommunicationNoteCandidateStateReady,
        CommunicationNoteCandidateStateV1::Rejected => {
            WireState::CommunicationNoteCandidateStateRejected
        }
    }
}

fn wire_candidate(value: CommunicationNoteCandidateV1) -> WireCandidate {
    WireCandidate {
        candidate_id: value.candidate_id.to_vec(),
        candidate_digest: value.candidate_digest.to_vec(),
        title: value.title,
        excerpt: value.excerpt,
        topic_hints: value
            .topic_hints
            .into_iter()
            .map(wire_topic_hint)
            .map(|hint| hint as i32)
            .collect(),
        source_basis: wire_source_basis(value.source_basis) as i32,
        confidence_basis_points: value.confidence_basis_points,
        source_evidence_id: value.source_evidence_id.to_vec(),
        source_evidence_revision: value.source_evidence_revision,
    }
}

const fn wire_source_basis(value: CommunicationNoteSourceBasisV1) -> WireSourceBasis {
    match value {
        CommunicationNoteSourceBasisV1::Subject => {
            WireSourceBasis::CommunicationNoteSourceBasisSubject
        }
        CommunicationNoteSourceBasisV1::Body => WireSourceBasis::CommunicationNoteSourceBasisBody,
        CommunicationNoteSourceBasisV1::Combined => {
            WireSourceBasis::CommunicationNoteSourceBasisCombined
        }
    }
}

const fn wire_topic_hint(value: CommunicationNoteTopicHintV1) -> WireTopicHint {
    match value {
        CommunicationNoteTopicHintV1::Financial => {
            WireTopicHint::CommunicationNoteTopicHintFinancial
        }
        CommunicationNoteTopicHintV1::Legal => WireTopicHint::CommunicationNoteTopicHintLegal,
        CommunicationNoteTopicHintV1::DecisionStatement => {
            WireTopicHint::CommunicationNoteTopicHintDecisionStatement
        }
        CommunicationNoteTopicHintV1::DeadlineStatement => {
            WireTopicHint::CommunicationNoteTopicHintDeadlineStatement
        }
    }
}

const fn wire_completeness(
    value: Option<CommunicationNoteCandidateCompletenessV1>,
) -> WireCompleteness {
    match value {
        None => WireCompleteness::CommunicationNoteCandidateCompletenessUnspecified,
        Some(CommunicationNoteCandidateCompletenessV1::Complete) => {
            WireCompleteness::CommunicationNoteCandidateCompletenessComplete
        }
    }
}

pub(crate) const fn rejection_error(
    value: Option<CommunicationNoteCandidateRejectionCodeV1>,
) -> WireError {
    match value {
        None => WireError::CommunicationNoteCandidateErrorCodeUnspecified,
        Some(CommunicationNoteCandidateRejectionCodeV1::InvalidRequest) => {
            WireError::CommunicationNoteCandidateErrorCodeInvalidRequest
        }
        Some(CommunicationNoteCandidateRejectionCodeV1::SourceRejected) => {
            WireError::CommunicationNoteCandidateErrorCodeSourceRejected
        }
        Some(CommunicationNoteCandidateRejectionCodeV1::ExtractionRejected) => {
            WireError::CommunicationNoteCandidateErrorCodeExtractionRejected
        }
        Some(CommunicationNoteCandidateRejectionCodeV1::Policy) => {
            WireError::CommunicationNoteCandidateErrorCodePolicy
        }
    }
}

fn start_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    StartCommunicationNoteCandidateResponseV1 {
        run_id,
        state: WireState::CommunicationNoteCandidateStateUnspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn get_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    GetCommunicationNoteCandidateResponseV1 {
        run_id,
        source_message_id: Vec::new(),
        expected_source_revision: 0,
        state: WireState::CommunicationNoteCandidateStateUnspecified as i32,
        state_revision: 0,
        candidates: Vec::new(),
        completeness: WireCompleteness::CommunicationNoteCandidateCompletenessUnspecified as i32,
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
            StartCommunicationNoteCandidateRequestV1 {
                protocol_major: 1,
                operation_id: vec![1; 16],
                source_message_id: vec![2; 16],
                expected_source_revision: 3,
            },
        )
        .expect("draft");
        assert_eq!(draft.source_message_id, [2; 16]);
        assert_eq!(draft.expected_source_revision, 3);
        assert_ne!(draft.run_id, [0; 16]);
    }
}
