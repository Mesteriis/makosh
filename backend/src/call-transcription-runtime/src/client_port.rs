use makosh_call_transcription_api::{
    CONTRACT_MAJOR_V1, GET_CONTRACT_NAME_V1, MODULE_ID_V1, OWNER_ID_V1, READ_CONTRACT_NAME_V1,
    READ_TICKET_BYTES_V1, START_CONTRACT_NAME_V1, TICKET_CONTRACT_NAME_V1, contract_reference_v1,
    wire::{
        CallTranscriptionArtifactV1 as WireArtifact,
        CallTranscriptionCompletenessV1 as WireCompleteness,
        CallTranscriptionErrorCodeV1 as WireError, CallTranscriptionLanguageV1 as WireLanguage,
        CallTranscriptionStateV1 as WireState, GetCallTranscriptionRequestV1,
        GetCallTranscriptionResponseV1, IssueCallTranscriptReadRequestV1,
        IssueCallTranscriptReadResponseV1, ReadCallTranscriptRequestV1,
        StartCallTranscriptionRequestV1, StartCallTranscriptionResponseV1,
    },
};
use makosh_call_transcription_core::{
    CallTranscriptionCompletenessV1, CallTranscriptionDraftV1, CallTranscriptionLanguageV1,
    CallTranscriptionRejectionV1, CallTranscriptionStateV1, TranscriptArtifactV1,
};
use makosh_call_transcription_persistence::{
    CallTranscriptionPersistenceErrorV1, CallTranscriptionPersistenceV1,
    CreateCallTranscriptionRunOutcomeV1, CreateCallTranscriptionRunV1, IssueCallTranscriptTicketV1,
    PersistedCallTranscriptionRunV1,
};
use makosh_runtime_protocol::v1::{
    ModuleClientBlobAuthorizationV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

const MODULE_CLIENT_PROTOCOL_MAJOR_V1: u32 = 1;

pub struct CallTranscriptionClientRuntimeContextV1<'a> {
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

pub async fn dispatch_client_request_v1(
    persistence: &CallTranscriptionPersistenceV1,
    runtime: &CallTranscriptionClientRuntimeContextV1<'_>,
    request: &ModuleClientRequestV1,
    now_unix_millis: i64,
) -> ModuleClientResponseV1 {
    let result = if validate_request(request, runtime, now_unix_millis).is_err() {
        Err(CallTranscriptionClientPortErrorV1::Protocol)
    } else if request.contract.as_ref() == Some(&contract_reference_v1(START_CONTRACT_NAME_V1)) {
        start(persistence, request, now_unix_millis).await
    } else if request.contract.as_ref() == Some(&contract_reference_v1(GET_CONTRACT_NAME_V1)) {
        get(persistence, request).await
    } else if request.contract.as_ref() == Some(&contract_reference_v1(TICKET_CONTRACT_NAME_V1)) {
        issue_read(persistence, runtime, request, now_unix_millis / 1_000).await
    } else if request.contract.as_ref() == Some(&contract_reference_v1(READ_CONTRACT_NAME_V1)) {
        authorize_read(persistence, runtime, request, now_unix_millis / 1_000).await
    } else {
        Err(CallTranscriptionClientPortErrorV1::Protocol)
    };
    match result {
        Ok(payload) => response(request.request_id, payload),
        Err(CallTranscriptionClientPortErrorV1::Protocol) => error(request.request_id, "REJECTED"),
        Err(CallTranscriptionClientPortErrorV1::NotFound) => error(request.request_id, "NOT_FOUND"),
        Err(CallTranscriptionClientPortErrorV1::Unavailable) => {
            error(request.request_id, "UNAVAILABLE")
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionClientPortErrorV1 {
    Protocol,
    NotFound,
    Unavailable,
}

async fn start(
    persistence: &CallTranscriptionPersistenceV1,
    request: &ModuleClientRequestV1,
    now_unix_millis: i64,
) -> Result<Vec<u8>, CallTranscriptionClientPortErrorV1> {
    let payload = StartCallTranscriptionRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| CallTranscriptionClientPortErrorV1::Protocol)?;
    if payload.protocol_major != CONTRACT_MAJOR_V1
        || payload.expected_call_evidence_revision == 0
        || payload.expected_recording_revision == 0
        || payload.consent_policy_revision == 0
    {
        return Err(CallTranscriptionClientPortErrorV1::Protocol);
    }
    let draft = CallTranscriptionDraftV1 {
        operation_id: id16(&payload.operation_id)?,
        call_evidence_id: id16(&payload.call_evidence_id)?,
        call_evidence_revision: payload.expected_call_evidence_revision,
        recording_evidence_id: id16(&payload.recording_evidence_id)?,
        recording_revision: payload.expected_recording_revision,
        consent_receipt_id: id16(&payload.consent_receipt_id)?,
        consent_policy_revision: payload.consent_policy_revision,
        requested_language: core_language(payload.requested_language)?,
    };
    let response = match persistence
        .create_run(CreateCallTranscriptionRunV1 {
            logical_owner_id: request.logical_owner_id.clone(),
            draft,
            created_at_unix_millis: now_unix_millis,
        })
        .await
    {
        Ok(CreateCallTranscriptionRunOutcomeV1::Created(run))
        | Ok(CreateCallTranscriptionRunOutcomeV1::Existing(run)) => {
            StartCallTranscriptionResponseV1 {
                run_id: run.run_id.to_vec(),
                state: wire_state(run.status.state) as i32,
                state_revision: run.status.state_revision,
                error: rejection_error(run.status.rejection) as i32,
            }
        }
        Err(
            CallTranscriptionPersistenceErrorV1::RequestConflict
            | CallTranscriptionPersistenceErrorV1::InvalidInput,
        ) => start_error(WireError::CallTranscriptionErrorCodeInvalidRequest),
        Err(_) => start_error(WireError::CallTranscriptionErrorCodeUnavailable),
    };
    Ok(response.encode_to_vec())
}

async fn get(
    persistence: &CallTranscriptionPersistenceV1,
    request: &ModuleClientRequestV1,
) -> Result<Vec<u8>, CallTranscriptionClientPortErrorV1> {
    let payload = GetCallTranscriptionRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| CallTranscriptionClientPortErrorV1::Protocol)?;
    if payload.protocol_major != CONTRACT_MAJOR_V1 {
        return Err(CallTranscriptionClientPortErrorV1::Protocol);
    }
    let run_id = id16(&payload.run_id)?;
    let response = match persistence
        .load_run(&request.logical_owner_id, run_id)
        .await
    {
        Ok(run) => get_response(run),
        Err(CallTranscriptionPersistenceErrorV1::NotFound) => GetCallTranscriptionResponseV1 {
            run_id: payload.run_id,
            error: WireError::CallTranscriptionErrorCodeNotFound as i32,
            ..Default::default()
        },
        Err(_) => return Err(CallTranscriptionClientPortErrorV1::Unavailable),
    };
    Ok(response.encode_to_vec())
}

async fn issue_read(
    persistence: &CallTranscriptionPersistenceV1,
    runtime: &CallTranscriptionClientRuntimeContextV1<'_>,
    request: &ModuleClientRequestV1,
    now_unix_seconds: i64,
) -> Result<Vec<u8>, CallTranscriptionClientPortErrorV1> {
    let payload = IssueCallTranscriptReadRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| CallTranscriptionClientPortErrorV1::Protocol)?;
    if payload.protocol_major != CONTRACT_MAJOR_V1 {
        return Err(CallTranscriptionClientPortErrorV1::Protocol);
    }
    let run_id = id16(&payload.run_id)?;
    let mut ticket = [0_u8; READ_TICKET_BYTES_V1];
    getrandom::fill(&mut ticket).map_err(|_| CallTranscriptionClientPortErrorV1::Unavailable)?;
    let response = match persistence
        .issue_read_ticket(
            &request.logical_owner_id,
            IssueCallTranscriptTicketV1 {
                ticket_sha256: Sha256::digest(ticket).into(),
                device_actor_sha256: device_actor_sha256(request),
                client_session_sha256: client_session_sha256(request),
                run_id,
                runtime_generation: runtime.runtime_generation,
                grant_epoch: runtime.grant_epoch,
                now_unix_seconds,
            },
        )
        .await
    {
        Ok(issued) => IssueCallTranscriptReadResponseV1 {
            run_id: issued.run_id.to_vec(),
            opaque_read_ticket: ticket.to_vec(),
            expires_at_unix_seconds: u64::try_from(issued.expires_at_unix_seconds)
                .map_err(|_| CallTranscriptionClientPortErrorV1::Unavailable)?,
            transcript_size_bytes: issued.transcript_size_bytes,
            error: WireError::CallTranscriptionErrorCodeUnspecified as i32,
        },
        Err(CallTranscriptionPersistenceErrorV1::NotFound) => {
            ticket_error(run_id, WireError::CallTranscriptionErrorCodeNotFound)
        }
        Err(CallTranscriptionPersistenceErrorV1::InvalidInput) => {
            ticket_error(run_id, WireError::CallTranscriptionErrorCodeInvalidRequest)
        }
        Err(CallTranscriptionPersistenceErrorV1::StaleFence) => {
            ticket_error(run_id, WireError::CallTranscriptionErrorCodeStaleAuthority)
        }
        Err(_) => ticket_error(run_id, WireError::CallTranscriptionErrorCodeUnavailable),
    };
    Ok(response.encode_to_vec())
}

async fn authorize_read(
    persistence: &CallTranscriptionPersistenceV1,
    runtime: &CallTranscriptionClientRuntimeContextV1<'_>,
    request: &ModuleClientRequestV1,
    now_unix_seconds: i64,
) -> Result<Vec<u8>, CallTranscriptionClientPortErrorV1> {
    let payload = ReadCallTranscriptRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| CallTranscriptionClientPortErrorV1::Protocol)?;
    if payload.protocol_major != CONTRACT_MAJOR_V1 {
        return Err(CallTranscriptionClientPortErrorV1::Protocol);
    }
    let ticket: [u8; READ_TICKET_BYTES_V1] = payload
        .opaque_read_ticket
        .try_into()
        .map_err(|_| CallTranscriptionClientPortErrorV1::Protocol)?;
    let redeemed = persistence
        .redeem_read_ticket(
            &request.logical_owner_id,
            Sha256::digest(ticket).into(),
            device_actor_sha256(request),
            client_session_sha256(request),
            runtime.runtime_generation,
            runtime.grant_epoch,
            now_unix_seconds,
        )
        .await
        .map_err(|error| match error {
            CallTranscriptionPersistenceErrorV1::NotFound
            | CallTranscriptionPersistenceErrorV1::TicketExpired
            | CallTranscriptionPersistenceErrorV1::TicketUsed
            | CallTranscriptionPersistenceErrorV1::StaleFence => {
                CallTranscriptionClientPortErrorV1::NotFound
            }
            _ => CallTranscriptionClientPortErrorV1::Unavailable,
        })?;
    Ok(ModuleClientBlobAuthorizationV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR_V1,
        reference_id: redeemed.artifact_reference_id.to_vec(),
        declared_size: redeemed.transcript_size_bytes,
        expected_plaintext_sha256: redeemed.artifact_receipt_sha256.to_vec(),
        backup_class: 1,
    }
    .encode_to_vec())
}

fn get_response(run: PersistedCallTranscriptionRunV1) -> GetCallTranscriptionResponseV1 {
    GetCallTranscriptionResponseV1 {
        run_id: run.run_id.to_vec(),
        call_evidence_id: run.draft.call_evidence_id.to_vec(),
        call_evidence_revision: run.draft.call_evidence_revision,
        recording_evidence_id: run.draft.recording_evidence_id.to_vec(),
        recording_revision: run.draft.recording_revision,
        state: wire_state(run.status.state) as i32,
        state_revision: run.status.state_revision,
        artifact: run.status.artifact.map(wire_artifact),
        error: rejection_error(run.status.rejection) as i32,
    }
}

pub(crate) fn wire_artifact(value: TranscriptArtifactV1) -> WireArtifact {
    WireArtifact {
        transcript_sha256: value.transcript_sha256.to_vec(),
        transcript_size_bytes: value.transcript_size_bytes,
        detected_language: wire_language(value.detected_language) as i32,
        duration_millis: value.duration_millis,
        segment_count: value.segment_count,
        completeness: wire_completeness(value.completeness) as i32,
        confidence_basis_points: value.confidence_basis_points,
    }
}

pub(crate) const fn wire_state(value: CallTranscriptionStateV1) -> WireState {
    match value {
        CallTranscriptionStateV1::Accepted => WireState::CallTranscriptionStateAccepted,
        CallTranscriptionStateV1::AwaitingRecording => {
            WireState::CallTranscriptionStateAwaitingRecording
        }
        CallTranscriptionStateV1::AwaitingStt => WireState::CallTranscriptionStateAwaitingStt,
        CallTranscriptionStateV1::MaterializingTranscript => {
            WireState::CallTranscriptionStateMaterializingTranscript
        }
        CallTranscriptionStateV1::Ready => WireState::CallTranscriptionStateReady,
        CallTranscriptionStateV1::Rejected => WireState::CallTranscriptionStateRejected,
    }
}

pub(crate) const fn rejection_error(value: Option<CallTranscriptionRejectionV1>) -> WireError {
    match value {
        None => WireError::CallTranscriptionErrorCodeUnspecified,
        Some(CallTranscriptionRejectionV1::RecordingRejected) => {
            WireError::CallTranscriptionErrorCodeRecordingRejected
        }
        Some(CallTranscriptionRejectionV1::SttRejected) => {
            WireError::CallTranscriptionErrorCodeSttRejected
        }
        Some(CallTranscriptionRejectionV1::ResultRejected) => {
            WireError::CallTranscriptionErrorCodeResultRejected
        }
        Some(CallTranscriptionRejectionV1::StaleAuthority) => {
            WireError::CallTranscriptionErrorCodeStaleAuthority
        }
        Some(CallTranscriptionRejectionV1::Policy) => WireError::CallTranscriptionErrorCodePolicy,
    }
}

fn core_language(
    value: i32,
) -> Result<CallTranscriptionLanguageV1, CallTranscriptionClientPortErrorV1> {
    match WireLanguage::try_from(value).map_err(|_| CallTranscriptionClientPortErrorV1::Protocol)? {
        WireLanguage::CallTranscriptionLanguageAuto => Ok(CallTranscriptionLanguageV1::Auto),
        WireLanguage::CallTranscriptionLanguageEnglish => Ok(CallTranscriptionLanguageV1::English),
        WireLanguage::CallTranscriptionLanguageRussian => Ok(CallTranscriptionLanguageV1::Russian),
        WireLanguage::CallTranscriptionLanguageSpanish => Ok(CallTranscriptionLanguageV1::Spanish),
        WireLanguage::CallTranscriptionLanguageUnspecified => {
            Err(CallTranscriptionClientPortErrorV1::Protocol)
        }
    }
}

const fn wire_language(value: CallTranscriptionLanguageV1) -> WireLanguage {
    match value {
        CallTranscriptionLanguageV1::Auto => WireLanguage::CallTranscriptionLanguageAuto,
        CallTranscriptionLanguageV1::English => WireLanguage::CallTranscriptionLanguageEnglish,
        CallTranscriptionLanguageV1::Russian => WireLanguage::CallTranscriptionLanguageRussian,
        CallTranscriptionLanguageV1::Spanish => WireLanguage::CallTranscriptionLanguageSpanish,
    }
}

const fn wire_completeness(value: CallTranscriptionCompletenessV1) -> WireCompleteness {
    match value {
        CallTranscriptionCompletenessV1::Complete => {
            WireCompleteness::CallTranscriptionCompletenessComplete
        }
        CallTranscriptionCompletenessV1::Partial => {
            WireCompleteness::CallTranscriptionCompletenessPartial
        }
    }
}

fn validate_request(
    request: &ModuleClientRequestV1,
    runtime: &CallTranscriptionClientRuntimeContextV1<'_>,
    now_unix_millis: i64,
) -> Result<(), CallTranscriptionClientPortErrorV1> {
    if request.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR_V1
        || request.module_id != MODULE_ID_V1
        || request.owner_id != OWNER_ID_V1
        || request.request_id == 0
        || request.request_payload.is_empty()
        || !valid_identity(&request.logical_owner_id)
        || !valid_identity(&request.authenticated_device_id)
        || !valid_identity(&request.authenticated_client_session_id)
        || runtime.runtime_instance_id.is_empty()
        || runtime.runtime_generation == 0
        || runtime.grant_epoch == 0
        || now_unix_millis <= 0
    {
        Err(CallTranscriptionClientPortErrorV1::Protocol)
    } else {
        Ok(())
    }
}

fn device_actor_sha256(request: &ModuleClientRequestV1) -> [u8; 32] {
    bound_identity_sha256(
        b"makosh.call-transcription.device-actor.v1\0",
        &request.logical_owner_id,
        &request.authenticated_device_id,
    )
}

fn client_session_sha256(request: &ModuleClientRequestV1) -> [u8; 32] {
    bound_identity_sha256(
        b"makosh.call-transcription.client-session.v1\0",
        &request.logical_owner_id,
        &request.authenticated_client_session_id,
    )
}

fn bound_identity_sha256(domain: &[u8], owner: &str, value: &str) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update(owner.as_bytes());
    digest.update([0]);
    digest.update(value.as_bytes());
    digest.finalize().into()
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn id16(value: &[u8]) -> Result<[u8; 16], CallTranscriptionClientPortErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(CallTranscriptionClientPortErrorV1::Protocol)
}

fn start_error(error: WireError) -> StartCallTranscriptionResponseV1 {
    StartCallTranscriptionResponseV1 {
        run_id: Vec::new(),
        state: WireState::CallTranscriptionStateUnspecified as i32,
        state_revision: 0,
        error: error as i32,
    }
}

fn ticket_error(run_id: [u8; 16], error: WireError) -> IssueCallTranscriptReadResponseV1 {
    IssueCallTranscriptReadResponseV1 {
        run_id: run_id.to_vec(),
        opaque_read_ticket: Vec::new(),
        expires_at_unix_seconds: 0,
        transcript_size_bytes: 0,
        error: error as i32,
    }
}

fn response(request_id: u64, response_payload: Vec<u8>) -> ModuleClientResponseV1 {
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR_V1,
        request_id,
        response_payload,
        error_code: String::new(),
    }
}

fn error(request_id: u64, error_code: &str) -> ModuleClientResponseV1 {
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR_V1,
        request_id,
        response_payload: Vec::new(),
        error_code: error_code.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(session: &str) -> ModuleClientRequestV1 {
        ModuleClientRequestV1 {
            protocol_major: 1,
            module_id: MODULE_ID_V1.to_owned(),
            owner_id: OWNER_ID_V1.to_owned(),
            contract: Some(contract_reference_v1(GET_CONTRACT_NAME_V1)),
            request_id: 1,
            request_payload: vec![1],
            logical_owner_id: "owner-1".to_owned(),
            authenticated_device_id: "device-1".to_owned(),
            authenticated_client_session_id: session.to_owned(),
        }
    }

    #[test]
    fn actor_and_session_bindings_are_distinct_and_session_specific() {
        let first = request("session-1");
        let second = request("session-2");
        assert_eq!(device_actor_sha256(&first), device_actor_sha256(&second));
        assert_ne!(
            client_session_sha256(&first),
            client_session_sha256(&second)
        );
        assert_ne!(device_actor_sha256(&first), client_session_sha256(&first));
    }
}
