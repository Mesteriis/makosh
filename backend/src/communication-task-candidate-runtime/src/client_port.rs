use makosh_communication_task_candidate_api::{
    COMMUNICATION_TASK_CANDIDATE_CONTRACT_MAJOR_V1,
    wire::{
        CommunicationTaskCandidateCompletenessV1 as WireCompleteness,
        CommunicationTaskCandidateErrorCodeV1 as WireError,
        CommunicationTaskCandidateStateV1 as WireState,
        CommunicationTaskCandidateV1 as WireCandidate,
        CommunicationTaskSignalKindV1 as WireSignalKind,
        CommunicationTaskSourceBasisV1 as WireSourceBasis, GetCommunicationTaskCandidateRequestV1,
        GetCommunicationTaskCandidateResponseV1, StartCommunicationTaskCandidateRequestV1,
        StartCommunicationTaskCandidateResponseV1,
    },
};
use makosh_communication_task_candidate_core::{
    CommunicationTaskCandidateCompletenessV1, CommunicationTaskCandidateDraftV1,
    CommunicationTaskCandidateRejectionCodeV1, CommunicationTaskCandidateStateV1,
    CommunicationTaskCandidateV1, CommunicationTaskSignalKindV1, CommunicationTaskSourceBasisV1,
};
use makosh_communication_task_candidate_persistence::{
    CommunicationTaskCandidatePersistenceErrorV1, CommunicationTaskCandidatePersistenceV1,
    CreateCommunicationTaskCandidateOutcomeV1, CreateCommunicationTaskCandidateRunV1,
    PersistedCommunicationTaskCandidateRunV1,
};
use makosh_communications_task_source_api::{
    CommunicationTaskSourceEnvelopeContextV1,
    build_communication_task_source_prepare_outbox_record_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub struct CommunicationTaskCandidateClientRuntimeContextV1<'a> {
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
}

pub async fn start_communication_task_candidate_payload_v1(
    persistence: &CommunicationTaskCandidatePersistenceV1,
    logical_owner_id: &str,
    runtime: &CommunicationTaskCandidateClientRuntimeContextV1<'_>,
    payload: &[u8],
    now_unix_millis: i64,
) -> Vec<u8> {
    let Ok(request) = StartCommunicationTaskCandidateRequestV1::decode(payload) else {
        return start_error(
            Vec::new(),
            WireError::CommunicationTaskCandidateErrorCodeInvalidRequest,
        );
    };
    let response_operation_id = request.operation_id.clone();
    let Some(draft) = start_draft(logical_owner_id, request) else {
        return start_error(
            response_operation_id,
            WireError::CommunicationTaskCandidateErrorCodeInvalidRequest,
        );
    };
    let Some(record) = source_prepare_record(logical_owner_id, runtime, &draft, now_unix_millis)
    else {
        return start_error(
            draft.run_id.to_vec(),
            WireError::CommunicationTaskCandidateErrorCodeUnavailable,
        );
    };
    let created = persistence
        .create_run(CreateCommunicationTaskCandidateRunV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            draft,
            source_prepare_message_id: *record.message_id(),
            source_prepare_envelope_sha256: *record.envelope_sha256(),
            source_prepare_envelope_bytes: record.exact_bytes().to_vec(),
            created_at_unix_millis: now_unix_millis,
        })
        .await;
    let persisted = match created {
        Ok(CreateCommunicationTaskCandidateOutcomeV1::Created(value))
        | Ok(CreateCommunicationTaskCandidateOutcomeV1::Existing(value)) => value,
        Err(CommunicationTaskCandidatePersistenceErrorV1::RequestConflict) => {
            return start_error(
                response_operation_id,
                WireError::CommunicationTaskCandidateErrorCodeInvalidRequest,
            );
        }
        Err(_) => {
            return start_error(
                response_operation_id,
                WireError::CommunicationTaskCandidateErrorCodeUnavailable,
            );
        }
    };
    let persisted = if persisted.status.state == CommunicationTaskCandidateStateV1::Accepted {
        match persistence
            .begin_source_preparation(logical_owner_id, &persisted.draft.run_id, now_unix_millis)
            .await
        {
            Ok(value) => value,
            Err(_) => {
                return start_error(
                    persisted.draft.run_id.to_vec(),
                    WireError::CommunicationTaskCandidateErrorCodeUnavailable,
                );
            }
        }
    } else {
        persisted
    };
    StartCommunicationTaskCandidateResponseV1 {
        run_id: persisted.draft.run_id.to_vec(),
        state: wire_state(persisted.status.state) as i32,
        error: rejection_error(persisted.status.rejection) as i32,
    }
    .encode_to_vec()
}

pub async fn get_communication_task_candidate_payload_v1(
    persistence: &CommunicationTaskCandidatePersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetCommunicationTaskCandidateRequestV1::decode(payload) else {
        return get_error(
            Vec::new(),
            WireError::CommunicationTaskCandidateErrorCodeInvalidRequest,
        );
    };
    let response_run_id = request.run_id.clone();
    let Ok(run_id) = id16(&request.run_id) else {
        return get_error(
            response_run_id,
            WireError::CommunicationTaskCandidateErrorCodeInvalidRequest,
        );
    };
    if request.protocol_major != COMMUNICATION_TASK_CANDIDATE_CONTRACT_MAJOR_V1 {
        return get_error(
            response_run_id,
            WireError::CommunicationTaskCandidateErrorCodeInvalidRequest,
        );
    }
    match persistence.load_run(logical_owner_id, &run_id).await {
        Ok(run) => get_response(run),
        Err(CommunicationTaskCandidatePersistenceErrorV1::NotFound) => get_error(
            response_run_id,
            WireError::CommunicationTaskCandidateErrorCodeNotFound,
        ),
        Err(_) => get_error(
            response_run_id,
            WireError::CommunicationTaskCandidateErrorCodeUnavailable,
        ),
    }
}

fn start_draft(
    logical_owner_id: &str,
    request: StartCommunicationTaskCandidateRequestV1,
) -> Option<CommunicationTaskCandidateDraftV1> {
    if request.protocol_major != COMMUNICATION_TASK_CANDIDATE_CONTRACT_MAJOR_V1 {
        return None;
    }
    let operation_id = id16(&request.operation_id).ok()?;
    Some(CommunicationTaskCandidateDraftV1 {
        run_id: run_id(logical_owner_id, &operation_id),
        operation_id,
        source_message_id: id16(&request.source_message_id).ok()?,
        expected_source_revision: (request.expected_source_revision > 0)
            .then_some(request.expected_source_revision)?,
    })
}

fn source_prepare_record(
    logical_owner_id: &str,
    runtime: &CommunicationTaskCandidateClientRuntimeContextV1<'_>,
    draft: &CommunicationTaskCandidateDraftV1,
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
    build_communication_task_source_prepare_outbox_record_v1(
        draft.run_id,
        draft.source_message_id,
        draft.expected_source_revision,
        logical_owner_id,
        deadline,
        &CommunicationTaskSourceEnvelopeContextV1 {
            module_id:
                makosh_communication_task_candidate_api::COMMUNICATION_TASK_CANDIDATE_MODULE_ID_V1
                    .to_owned(),
            runtime_instance_id: runtime.runtime_instance_id.to_owned(),
            runtime_generation: runtime.runtime_generation,
            recorded_at_unix_seconds: seconds,
            recorded_at_nanos: nanos,
        },
    )
    .ok()
}

fn get_response(run: PersistedCommunicationTaskCandidateRunV1) -> Vec<u8> {
    let candidates = run
        .status
        .candidates
        .unwrap_or_default()
        .into_iter()
        .map(wire_candidate)
        .collect();
    GetCommunicationTaskCandidateResponseV1 {
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
    digest.update(b"makosh.communication_task_candidate.run.v1\0");
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

pub(crate) const fn wire_state(value: CommunicationTaskCandidateStateV1) -> WireState {
    match value {
        CommunicationTaskCandidateStateV1::Accepted => {
            WireState::CommunicationTaskCandidateStateAccepted
        }
        CommunicationTaskCandidateStateV1::PreparingSource => {
            WireState::CommunicationTaskCandidateStatePreparingSource
        }
        CommunicationTaskCandidateStateV1::Extracting => {
            WireState::CommunicationTaskCandidateStateExtracting
        }
        CommunicationTaskCandidateStateV1::Ready => WireState::CommunicationTaskCandidateStateReady,
        CommunicationTaskCandidateStateV1::Rejected => {
            WireState::CommunicationTaskCandidateStateRejected
        }
    }
}

fn wire_candidate(value: CommunicationTaskCandidateV1) -> WireCandidate {
    WireCandidate {
        candidate_id: value.candidate_id.to_vec(),
        candidate_digest: value.candidate_digest.to_vec(),
        title: value.title,
        due_text_hint: value.due_text_hint,
        assignee_label_hint: value.assignee_label_hint,
        source_basis: wire_source_basis(value.source_basis) as i32,
        signal_kind: wire_signal_kind(value.signal_kind) as i32,
        confidence_basis_points: value.confidence_basis_points,
        source_evidence_id: value.source_evidence_id.to_vec(),
        source_evidence_revision: value.source_evidence_revision,
    }
}

const fn wire_source_basis(value: CommunicationTaskSourceBasisV1) -> WireSourceBasis {
    match value {
        CommunicationTaskSourceBasisV1::Subject => {
            WireSourceBasis::CommunicationTaskSourceBasisSubject
        }
        CommunicationTaskSourceBasisV1::Body => WireSourceBasis::CommunicationTaskSourceBasisBody,
        CommunicationTaskSourceBasisV1::Combined => {
            WireSourceBasis::CommunicationTaskSourceBasisCombined
        }
    }
}

const fn wire_signal_kind(value: CommunicationTaskSignalKindV1) -> WireSignalKind {
    match value {
        CommunicationTaskSignalKindV1::ExplicitAction => {
            WireSignalKind::CommunicationTaskSignalKindExplicitAction
        }
        CommunicationTaskSignalKindV1::DirectRequest => {
            WireSignalKind::CommunicationTaskSignalKindDirectRequest
        }
        CommunicationTaskSignalKindV1::FollowUp => {
            WireSignalKind::CommunicationTaskSignalKindFollowUp
        }
    }
}

const fn wire_completeness(
    value: Option<CommunicationTaskCandidateCompletenessV1>,
) -> WireCompleteness {
    match value {
        None => WireCompleteness::CommunicationTaskCandidateCompletenessUnspecified,
        Some(CommunicationTaskCandidateCompletenessV1::Complete) => {
            WireCompleteness::CommunicationTaskCandidateCompletenessComplete
        }
    }
}

pub(crate) const fn rejection_error(
    value: Option<CommunicationTaskCandidateRejectionCodeV1>,
) -> WireError {
    match value {
        None => WireError::CommunicationTaskCandidateErrorCodeUnspecified,
        Some(CommunicationTaskCandidateRejectionCodeV1::InvalidRequest) => {
            WireError::CommunicationTaskCandidateErrorCodeInvalidRequest
        }
        Some(CommunicationTaskCandidateRejectionCodeV1::SourceRejected) => {
            WireError::CommunicationTaskCandidateErrorCodeSourceRejected
        }
        Some(CommunicationTaskCandidateRejectionCodeV1::ExtractionRejected) => {
            WireError::CommunicationTaskCandidateErrorCodeExtractionRejected
        }
        Some(CommunicationTaskCandidateRejectionCodeV1::Policy) => {
            WireError::CommunicationTaskCandidateErrorCodePolicy
        }
    }
}

fn start_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    StartCommunicationTaskCandidateResponseV1 {
        run_id,
        state: WireState::CommunicationTaskCandidateStateUnspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn get_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    GetCommunicationTaskCandidateResponseV1 {
        run_id,
        source_message_id: Vec::new(),
        expected_source_revision: 0,
        state: WireState::CommunicationTaskCandidateStateUnspecified as i32,
        state_revision: 0,
        candidates: Vec::new(),
        completeness: WireCompleteness::CommunicationTaskCandidateCompletenessUnspecified as i32,
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
            StartCommunicationTaskCandidateRequestV1 {
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
