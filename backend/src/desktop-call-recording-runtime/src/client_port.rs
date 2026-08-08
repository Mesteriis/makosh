use makosh_desktop_call_recording_api::{
    GET_CONTRACT_NAME_V1, MODULE_ID_V1, OWNER_ID_V1, START_CONTRACT_NAME_V1, STOP_CONTRACT_NAME_V1,
    contract_reference_v1,
    wire::{
        DesktopCallRecordingStatusChangedV1, DesktopRecordingStateV1,
        GetDesktopCallRecordingRequestV1, GetDesktopCallRecordingResponseV1,
        RecordingTranscriptionAuthorityV1, StartDesktopCallRecordingRequestV1,
        StartDesktopCallRecordingResponseV1, StopDesktopCallRecordingRequestV1,
        StopDesktopCallRecordingResponseV1,
    },
};
use makosh_desktop_call_recording_core::{
    RecordingStateV1, StartRecordingV1, device_actor_sha256_v1, validate_start_v1,
};
use makosh_desktop_call_recording_persistence::{
    DesktopCallRecordingRepositoryV1, NewRecordingRunV1, RealtimeTransitionV1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;
use sha2::{Digest, Sha256};

const MODULE_CLIENT_PROTOCOL_MAJOR_V1: u32 = 1;
const CONSENT_CHALLENGE_TTL_MILLIS_V1: i64 = 60_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopRecordingClientPortErrorV1 {
    Protocol,
    Conflict,
    NotFound,
    Unavailable,
}

pub async fn dispatch_client_request_v1(
    persistence: &DesktopCallRecordingRepositoryV1,
    request: &ModuleClientRequestV1,
    now_unix_ms: i64,
) -> ModuleClientResponseV1 {
    let result =
        if request.contract.as_ref() == Some(&contract_reference_v1(START_CONTRACT_NAME_V1)) {
            start(persistence, request, now_unix_ms).await
        } else if request.contract.as_ref() == Some(&contract_reference_v1(STOP_CONTRACT_NAME_V1)) {
            stop(persistence, request).await
        } else if request.contract.as_ref() == Some(&contract_reference_v1(GET_CONTRACT_NAME_V1)) {
            get(persistence, request).await
        } else {
            Err(DesktopRecordingClientPortErrorV1::Protocol)
        };
    match result {
        Ok(payload) => response(request.request_id, payload, ""),
        Err(DesktopRecordingClientPortErrorV1::Protocol) => {
            response(request.request_id, Vec::new(), "INVALID_ARGUMENT")
        }
        Err(DesktopRecordingClientPortErrorV1::Conflict) => {
            response(request.request_id, Vec::new(), "CONFLICT")
        }
        Err(DesktopRecordingClientPortErrorV1::NotFound) => {
            response(request.request_id, Vec::new(), "NOT_FOUND")
        }
        Err(DesktopRecordingClientPortErrorV1::Unavailable) => {
            response(request.request_id, Vec::new(), "UNAVAILABLE")
        }
    }
}

async fn start(
    persistence: &DesktopCallRecordingRepositoryV1,
    request: &ModuleClientRequestV1,
    now_unix_ms: i64,
) -> Result<Vec<u8>, DesktopRecordingClientPortErrorV1> {
    validate_request(request)?;
    if now_unix_ms <= 0 {
        return Err(DesktopRecordingClientPortErrorV1::Unavailable);
    }
    let payload = StartDesktopCallRecordingRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| DesktopRecordingClientPortErrorV1::Protocol)?;
    let operation_id = id16(&payload.operation_id)?;
    let call_evidence_id = id16(&payload.call_evidence_id)?;
    let start = StartRecordingV1 {
        operation_id,
        call_evidence_id,
        expected_call_revision: payload.expected_call_revision,
        maximum_duration_millis: payload.maximum_duration_millis,
        consent_policy_revision: payload.consent_policy_revision,
        logical_owner_id: request.logical_owner_id.clone(),
        authenticated_device_id: request.authenticated_device_id.clone(),
    };
    validate_start_v1(&start).map_err(|_| DesktopRecordingClientPortErrorV1::Protocol)?;
    let recording_id = derived_id(b"recording", &request.logical_owner_id, operation_id);
    let challenge_id = derived_id(b"challenge", &request.logical_owner_id, operation_id);
    let command_id = begin_command_id_v1(&request.logical_owner_id, operation_id);
    let realtime = realtime_transition(
        recording_id,
        1,
        RecordingStateV1::AwaitingConsent,
        0,
        now_unix_ms,
        "",
    );
    let (run, _) = persistence
        .accept_or_replay(
            &NewRecordingRunV1 {
                logical_owner_id: request.logical_owner_id.clone(),
                operation_id,
                request_sha256: request_digest(request),
                call_evidence_id,
                call_evidence_revision: payload.expected_call_revision,
                recording_evidence_id: recording_id,
                device_actor_sha256: device_actor_sha256_v1(
                    &request.logical_owner_id,
                    &request.authenticated_device_id,
                )
                .map_err(|_| DesktopRecordingClientPortErrorV1::Protocol)?,
                challenge_id,
                challenge_expires_at_unix_ms: now_unix_ms
                    .checked_add(CONSENT_CHALLENGE_TTL_MILLIS_V1)
                    .ok_or(DesktopRecordingClientPortErrorV1::Unavailable)?,
                maximum_duration_millis: payload.maximum_duration_millis,
                consent_policy_revision: payload.consent_policy_revision,
            },
            command_id,
            &realtime,
        )
        .await
        .map_err(persistence_error)?;
    Ok(StartDesktopCallRecordingResponseV1 {
        recording_evidence_id: run.recording_evidence_id.to_vec(),
        recording_revision: run.recording_revision,
        state: wire_state(run.state) as i32,
    }
    .encode_to_vec())
}

async fn stop(
    persistence: &DesktopCallRecordingRepositoryV1,
    request: &ModuleClientRequestV1,
) -> Result<Vec<u8>, DesktopRecordingClientPortErrorV1> {
    validate_request(request)?;
    let payload = StopDesktopCallRecordingRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| DesktopRecordingClientPortErrorV1::Protocol)?;
    let recording_id = id16(&payload.recording_evidence_id)?;
    let command_id = stop_command_id_v1(&request.logical_owner_id, recording_id);
    let run = persistence
        .request_stop(&request.logical_owner_id, &recording_id, command_id)
        .await
        .map_err(persistence_error)?;
    Ok(StopDesktopCallRecordingResponseV1 {
        recording_revision: run.recording_revision,
        state: wire_state(run.state) as i32,
    }
    .encode_to_vec())
}

async fn get(
    persistence: &DesktopCallRecordingRepositoryV1,
    request: &ModuleClientRequestV1,
) -> Result<Vec<u8>, DesktopRecordingClientPortErrorV1> {
    validate_request(request)?;
    let payload = GetDesktopCallRecordingRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| DesktopRecordingClientPortErrorV1::Protocol)?;
    let recording_id = id16(&payload.recording_evidence_id)?;
    let run = persistence
        .get(&request.logical_owner_id, &recording_id)
        .await
        .map_err(persistence_error)?
        .ok_or(DesktopRecordingClientPortErrorV1::NotFound)?;
    let transcription_authority = transcription_authority(&run);
    Ok(GetDesktopCallRecordingResponseV1 {
        recording_evidence_id: recording_id.to_vec(),
        recording_revision: run.recording_revision,
        state: wire_state(run.state) as i32,
        duration_millis: run.source_duration_millis.unwrap_or(0),
        public_error_code: run.public_error_code.unwrap_or_default(),
        transcription_authority,
    }
    .encode_to_vec())
}

fn transcription_authority(
    run: &makosh_desktop_call_recording_persistence::PersistedRecordingRunV1,
) -> Option<RecordingTranscriptionAuthorityV1> {
    if run.state != RecordingStateV1::Ready {
        return None;
    }
    Some(RecordingTranscriptionAuthorityV1 {
        operation_id: run.operation_id.to_vec(),
        call_evidence_id: run.call_evidence_id.to_vec(),
        call_evidence_revision: run.call_evidence_revision,
        recording_evidence_id: run.recording_evidence_id.to_vec(),
        recording_revision: run.recording_revision,
        consent_receipt_id: run.consent_receipt_id?.to_vec(),
        consent_policy_revision: run.consent_policy_revision,
    })
}

#[must_use]
pub fn realtime_transition(
    recording_id: [u8; 16],
    revision: u64,
    state: RecordingStateV1,
    duration_millis: u64,
    occurred_at_unix_ms: i64,
    public_error_code: &str,
) -> RealtimeTransitionV1 {
    let payload = DesktopCallRecordingStatusChangedV1 {
        recording_evidence_id: recording_id.to_vec(),
        recording_revision: revision,
        state: wire_state(state) as i32,
        duration_millis,
        occurred_at_unix_ms,
        public_error_code: public_error_code.to_owned(),
    }
    .encode_to_vec();
    RealtimeTransitionV1 {
        occurred_at_unix_ms,
        payload_sha256: Sha256::digest(&payload).into(),
        payload_bytes: payload,
    }
}

fn validate_request(
    request: &ModuleClientRequestV1,
) -> Result<(), DesktopRecordingClientPortErrorV1> {
    if request.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR_V1
        || request.module_id != MODULE_ID_V1
        || request.owner_id != OWNER_ID_V1
        || request.request_id == 0
        || !valid_identity(&request.logical_owner_id)
        || !valid_identity(&request.authenticated_device_id)
    {
        return Err(DesktopRecordingClientPortErrorV1::Protocol);
    }
    Ok(())
}

fn request_digest(request: &ModuleClientRequestV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.desktop-call-recording.start-request.v1\0");
    hash.update(request.logical_owner_id.as_bytes());
    hash.update([0]);
    hash.update(request.authenticated_device_id.as_bytes());
    hash.update([0]);
    hash.update(&request.request_payload);
    hash.finalize().into()
}

fn derived_id(label: &[u8], owner: &str, seed: [u8; 16]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.desktop-call-recording.v1\0");
    hash.update(label);
    hash.update([0]);
    hash.update(owner.as_bytes());
    hash.update([0]);
    hash.update(seed);
    hash.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

#[must_use]
pub(crate) fn begin_command_id_v1(owner: &str, operation_id: [u8; 16]) -> [u8; 16] {
    derived_id(b"begin-command", owner, operation_id)
}

#[must_use]
pub(crate) fn stop_command_id_v1(owner: &str, recording_id: [u8; 16]) -> [u8; 16] {
    derived_id(b"stop-command", owner, recording_id)
}

fn id16(value: &[u8]) -> Result<[u8; 16], DesktopRecordingClientPortErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(DesktopRecordingClientPortErrorV1::Protocol)
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn wire_state(value: RecordingStateV1) -> DesktopRecordingStateV1 {
    match value {
        RecordingStateV1::AwaitingConsent => {
            DesktopRecordingStateV1::DesktopRecordingStateAwaitingConsentV1
        }
        RecordingStateV1::Capturing => DesktopRecordingStateV1::DesktopRecordingStateCapturingV1,
        RecordingStateV1::Materializing => {
            DesktopRecordingStateV1::DesktopRecordingStateMaterializingV1
        }
        RecordingStateV1::Ready => DesktopRecordingStateV1::DesktopRecordingStateReadyV1,
        RecordingStateV1::Rejected => DesktopRecordingStateV1::DesktopRecordingStateRejectedV1,
    }
}

fn persistence_error(
    error: makosh_desktop_call_recording_persistence::PersistenceErrorV1,
) -> DesktopRecordingClientPortErrorV1 {
    match error {
        makosh_desktop_call_recording_persistence::PersistenceErrorV1::InvalidInput => {
            DesktopRecordingClientPortErrorV1::Protocol
        }
        makosh_desktop_call_recording_persistence::PersistenceErrorV1::Conflict => {
            DesktopRecordingClientPortErrorV1::Conflict
        }
        makosh_desktop_call_recording_persistence::PersistenceErrorV1::StorageUnavailable
        | makosh_desktop_call_recording_persistence::PersistenceErrorV1::InvalidRow => {
            DesktopRecordingClientPortErrorV1::Unavailable
        }
    }
}

fn response(request_id: u64, payload: Vec<u8>, error_code: &str) -> ModuleClientResponseV1 {
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR_V1,
        request_id,
        response_payload: payload,
        error_code: error_code.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use makosh_desktop_call_recording_persistence::PersistedRecordingRunV1;

    use super::*;

    #[test]
    fn operation_contracts_are_not_ambiguous() {
        assert_ne!(
            contract_reference_v1(START_CONTRACT_NAME_V1),
            contract_reference_v1(STOP_CONTRACT_NAME_V1)
        );
        assert_ne!(
            contract_reference_v1(STOP_CONTRACT_NAME_V1),
            contract_reference_v1(GET_CONTRACT_NAME_V1)
        );
    }

    #[test]
    fn ids_are_owner_scoped_and_stable() {
        assert_eq!(
            derived_id(b"recording", "owner-1", [7; 16]),
            derived_id(b"recording", "owner-1", [7; 16])
        );
        assert_ne!(
            derived_id(b"recording", "owner-1", [7; 16]),
            derived_id(b"recording", "owner-2", [7; 16])
        );
    }

    #[test]
    fn transcription_authority_exists_only_for_ready_persisted_evidence() {
        let mut run = persisted_run(RecordingStateV1::Capturing);
        assert!(transcription_authority(&run).is_none());
        run.state = RecordingStateV1::Ready;
        let authority = transcription_authority(&run).expect("ready authority");
        assert_eq!(authority.operation_id, vec![1; 16]);
        assert_eq!(authority.call_evidence_id, vec![2; 16]);
        assert_eq!(authority.recording_evidence_id, vec![3; 16]);
        assert_eq!(authority.consent_receipt_id, vec![4; 16]);
        run.consent_receipt_id = None;
        assert!(transcription_authority(&run).is_none());
    }

    fn persisted_run(state: RecordingStateV1) -> PersistedRecordingRunV1 {
        PersistedRecordingRunV1 {
            logical_owner_id: "owner-1".to_owned(),
            operation_id: [1; 16],
            request_sha256: [5; 32],
            call_evidence_id: [2; 16],
            call_evidence_revision: 7,
            recording_evidence_id: [3; 16],
            recording_revision: 4,
            state,
            device_actor_sha256: [6; 32],
            challenge_id: [7; 16],
            challenge_expires_at_unix_ms: 1,
            maximum_duration_millis: 10_000,
            consent_policy_revision: 3,
            started_at_unix_ms: Some(1),
            ended_at_unix_ms: Some(2),
            consent_receipt_id: Some([4; 16]),
            source_reference_id: Some([8; 16]),
            source_declared_bytes: Some(44),
            source_duration_millis: Some(10),
            source_sha256: Some([9; 32]),
            public_error_code: None,
        }
    }
}
