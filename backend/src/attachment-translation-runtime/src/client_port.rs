use makosh_attachment_translation_api::{
    ATTACHMENT_TRANSLATION_CONTRACT_MAJOR_V1, ATTACHMENT_TRANSLATION_MODULE_ID_V1,
    ATTACHMENT_TRANSLATION_OWNER_V1, ATTACHMENT_TRANSLATION_READ_TICKET_BYTES_V1,
    read_wire::ReadAttachmentTranslationRequestV1,
    wire::{
        AttachmentTranslationArtifactV1 as WireArtifact,
        AttachmentTranslationCompletenessV1 as WireCompleteness,
        AttachmentTranslationDetectedLanguageV1 as WireDetectedLanguage,
        AttachmentTranslationErrorCodeV1 as WireError,
        AttachmentTranslationLanguageV1 as WireLanguage, AttachmentTranslationStateV1 as WireState,
        GetAttachmentTranslationRequestV1, GetAttachmentTranslationResponseV1,
        IssueAttachmentTranslationReadRequestV1, IssueAttachmentTranslationReadResponseV1,
        StartAttachmentTranslationRequestV1, StartAttachmentTranslationResponseV1,
    },
};
use makosh_attachment_translation_core::{
    AttachmentTranslationArtifactV1, AttachmentTranslationCompletenessV1,
    AttachmentTranslationDetectedLanguageV1, AttachmentTranslationDraftV1,
    AttachmentTranslationLanguageV1, AttachmentTranslationRejectionCodeV1,
    AttachmentTranslationStateV1,
};
use makosh_attachment_translation_ingress::{
    AttachmentTranslationSourceEnvelopeContextV1, attachment_translation_source_request_id_v1,
    build_request_attachment_translation_source_outbox_record_v1,
    wire::RequestAttachmentTranslationSourceV1,
};
use makosh_attachment_translation_persistence::{
    AttachmentTranslationPersistenceErrorV1, AttachmentTranslationPersistenceV1,
    CreateAttachmentTranslationOutcomeV1, CreateAttachmentTranslationRunV1,
    IssueAttachmentTranslationTicketV1, PersistedAttachmentTranslationRunV1,
};
use makosh_runtime_protocol::v1::{
    ModuleClientBlobAuthorizationV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::contracts::{
    attachment_translation_command_contract_v1, attachment_translation_query_contract_v1,
    attachment_translation_read_contract_v1, attachment_translation_ticket_contract_v1,
};

const MODULE_CLIENT_PROTOCOL_MAJOR_V1: u32 = 1;

pub struct AttachmentTranslationClientRuntimeContextV1<'a> {
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

pub async fn dispatch_attachment_translation_client_request_v1(
    persistence: &AttachmentTranslationPersistenceV1,
    runtime: &AttachmentTranslationClientRuntimeContextV1<'_>,
    request: &ModuleClientRequestV1,
    now_unix_millis: i64,
) -> ModuleClientResponseV1 {
    let result = if validate_request(request, runtime, now_unix_millis).is_err() {
        Err(AttachmentTranslationClientPortErrorV1::Protocol)
    } else if request.contract.as_ref() == Some(&attachment_translation_command_contract_v1()) {
        start(persistence, runtime, request, now_unix_millis).await
    } else if request.contract.as_ref() == Some(&attachment_translation_query_contract_v1()) {
        get(persistence, request).await
    } else if request.contract.as_ref() == Some(&attachment_translation_ticket_contract_v1()) {
        issue_read(persistence, runtime, request, now_unix_millis / 1_000).await
    } else if request.contract.as_ref() == Some(&attachment_translation_read_contract_v1()) {
        authorize_read(persistence, runtime, request, now_unix_millis / 1_000).await
    } else {
        Err(AttachmentTranslationClientPortErrorV1::Protocol)
    };
    match result {
        Ok(payload) => response(request.request_id, payload),
        Err(AttachmentTranslationClientPortErrorV1::Protocol) => {
            error(request.request_id, "REJECTED")
        }
        Err(AttachmentTranslationClientPortErrorV1::NotFound) => {
            error(request.request_id, "NOT_FOUND")
        }
        Err(AttachmentTranslationClientPortErrorV1::Unavailable) => {
            error(request.request_id, "UNAVAILABLE")
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationClientPortErrorV1 {
    Protocol,
    NotFound,
    Unavailable,
}

async fn start(
    persistence: &AttachmentTranslationPersistenceV1,
    runtime: &AttachmentTranslationClientRuntimeContextV1<'_>,
    request: &ModuleClientRequestV1,
    now_unix_millis: i64,
) -> Result<Vec<u8>, AttachmentTranslationClientPortErrorV1> {
    let payload = StartAttachmentTranslationRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| AttachmentTranslationClientPortErrorV1::Protocol)?;
    let operation_id = id16(&payload.operation_id)?;
    let source_extraction_run_id = id16(&payload.source_extraction_run_id)?;
    let target_language = target_language(payload.target_language)?;
    if payload.protocol_major != ATTACHMENT_TRANSLATION_CONTRACT_MAJOR_V1
        || payload.expected_source_revision == 0
    {
        return Err(AttachmentTranslationClientPortErrorV1::Protocol);
    }
    let run_id = run_id(&request.logical_owner_id, &operation_id);
    let draft = AttachmentTranslationDraftV1 {
        run_id,
        operation_id,
        source_extraction_run_id,
        expected_source_revision: payload.expected_source_revision,
        target_language,
    };
    let record =
        source_request_record(&request.logical_owner_id, runtime, &draft, now_unix_millis)?;
    let response = match persistence
        .create_run(CreateAttachmentTranslationRunV1 {
            logical_owner_id: request.logical_owner_id.clone(),
            draft,
            source_prepare_message_id: *record.message_id(),
            source_prepare_envelope_sha256: *record.envelope_sha256(),
            source_prepare_envelope_bytes: record.exact_bytes().to_vec(),
            created_at_unix_millis: now_unix_millis,
        })
        .await
    {
        Ok(CreateAttachmentTranslationOutcomeV1::Created(run))
        | Ok(CreateAttachmentTranslationOutcomeV1::Existing(run)) => {
            StartAttachmentTranslationResponseV1 {
                run_id: run.draft.run_id.to_vec(),
                state: wire_state(run.status.state) as i32,
                error: rejection_error(run.status.rejection) as i32,
            }
        }
        Err(AttachmentTranslationPersistenceErrorV1::RequestConflict)
        | Err(AttachmentTranslationPersistenceErrorV1::InvalidInput) => {
            start_error(WireError::AttachmentTranslationErrorCodeInvalidRequest)
        }
        Err(_) => start_error(WireError::AttachmentTranslationErrorCodeUnavailable),
    };
    Ok(response.encode_to_vec())
}

async fn get(
    persistence: &AttachmentTranslationPersistenceV1,
    request: &ModuleClientRequestV1,
) -> Result<Vec<u8>, AttachmentTranslationClientPortErrorV1> {
    let payload = GetAttachmentTranslationRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| AttachmentTranslationClientPortErrorV1::Protocol)?;
    if payload.protocol_major != ATTACHMENT_TRANSLATION_CONTRACT_MAJOR_V1 {
        return Err(AttachmentTranslationClientPortErrorV1::Protocol);
    }
    let run_id = id16(&payload.run_id)?;
    let response = match persistence
        .load_run(&request.logical_owner_id, &run_id)
        .await
    {
        Ok(run) => get_response(run),
        Err(AttachmentTranslationPersistenceErrorV1::NotFound) => {
            GetAttachmentTranslationResponseV1 {
                run_id: payload.run_id,
                error: WireError::AttachmentTranslationErrorCodeNotFound as i32,
                ..Default::default()
            }
        }
        Err(_) => return Err(AttachmentTranslationClientPortErrorV1::Unavailable),
    };
    Ok(response.encode_to_vec())
}

async fn issue_read(
    persistence: &AttachmentTranslationPersistenceV1,
    runtime: &AttachmentTranslationClientRuntimeContextV1<'_>,
    request: &ModuleClientRequestV1,
    now_unix_seconds: i64,
) -> Result<Vec<u8>, AttachmentTranslationClientPortErrorV1> {
    let payload =
        IssueAttachmentTranslationReadRequestV1::decode(request.request_payload.as_slice())
            .map_err(|_| AttachmentTranslationClientPortErrorV1::Protocol)?;
    if payload.protocol_major != ATTACHMENT_TRANSLATION_CONTRACT_MAJOR_V1 {
        return Err(AttachmentTranslationClientPortErrorV1::Protocol);
    }
    let run_id = id16(&payload.run_id)?;
    let mut opaque_ticket = [0_u8; ATTACHMENT_TRANSLATION_READ_TICKET_BYTES_V1];
    getrandom::fill(&mut opaque_ticket)
        .map_err(|_| AttachmentTranslationClientPortErrorV1::Unavailable)?;
    let response = match persistence
        .issue_read_ticket(
            &request.logical_owner_id,
            IssueAttachmentTranslationTicketV1 {
                ticket_sha256: Sha256::digest(opaque_ticket).into(),
                device_actor_sha256: device_actor_sha256(
                    &request.logical_owner_id,
                    &request.authenticated_device_id,
                ),
                run_id,
                runtime_generation: runtime.runtime_generation,
                grant_epoch: runtime.grant_epoch,
                now_unix_seconds,
            },
        )
        .await
    {
        Ok(issued) => IssueAttachmentTranslationReadResponseV1 {
            run_id: issued.run_id.to_vec(),
            opaque_read_ticket: opaque_ticket.to_vec(),
            expires_at_unix_seconds: u64::try_from(issued.expires_at_unix_seconds)
                .map_err(|_| AttachmentTranslationClientPortErrorV1::Unavailable)?,
            translated_size_bytes: issued.translated_size_bytes,
            error: WireError::AttachmentTranslationErrorCodeUnspecified as i32,
        },
        Err(AttachmentTranslationPersistenceErrorV1::NotFound) => {
            ticket_error(run_id, WireError::AttachmentTranslationErrorCodeNotFound)
        }
        Err(AttachmentTranslationPersistenceErrorV1::InvalidInput) => ticket_error(
            run_id,
            WireError::AttachmentTranslationErrorCodeInvalidRequest,
        ),
        Err(AttachmentTranslationPersistenceErrorV1::StaleFence) => {
            ticket_error(run_id, WireError::AttachmentTranslationErrorCodeUnavailable)
        }
        Err(
            AttachmentTranslationPersistenceErrorV1::InvalidRow
            | AttachmentTranslationPersistenceErrorV1::InvalidTransition
            | AttachmentTranslationPersistenceErrorV1::RevisionConflict,
        ) => ticket_error(
            run_id,
            WireError::AttachmentTranslationErrorCodeResultRejected,
        ),
        Err(_) => ticket_error(run_id, WireError::AttachmentTranslationErrorCodeUnavailable),
    };
    Ok(response.encode_to_vec())
}

async fn authorize_read(
    persistence: &AttachmentTranslationPersistenceV1,
    runtime: &AttachmentTranslationClientRuntimeContextV1<'_>,
    request: &ModuleClientRequestV1,
    now_unix_seconds: i64,
) -> Result<Vec<u8>, AttachmentTranslationClientPortErrorV1> {
    let payload = ReadAttachmentTranslationRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| AttachmentTranslationClientPortErrorV1::Protocol)?;
    if payload.protocol_major != ATTACHMENT_TRANSLATION_CONTRACT_MAJOR_V1 {
        return Err(AttachmentTranslationClientPortErrorV1::Protocol);
    }
    let ticket: [u8; ATTACHMENT_TRANSLATION_READ_TICKET_BYTES_V1] = payload
        .opaque_read_ticket
        .try_into()
        .map_err(|_| AttachmentTranslationClientPortErrorV1::Protocol)?;
    let redeemed = persistence
        .redeem_read_ticket(
            &request.logical_owner_id,
            Sha256::digest(ticket).into(),
            device_actor_sha256(&request.logical_owner_id, &request.authenticated_device_id),
            runtime.runtime_generation,
            runtime.grant_epoch,
            now_unix_seconds,
        )
        .await
        .map_err(|error| match error {
            AttachmentTranslationPersistenceErrorV1::NotFound
            | AttachmentTranslationPersistenceErrorV1::TicketExpired
            | AttachmentTranslationPersistenceErrorV1::TicketUsed
            | AttachmentTranslationPersistenceErrorV1::StaleFence => {
                AttachmentTranslationClientPortErrorV1::NotFound
            }
            _ => AttachmentTranslationClientPortErrorV1::Unavailable,
        })?;
    Ok(ModuleClientBlobAuthorizationV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR_V1,
        reference_id: redeemed.artifact_reference_id.to_vec(),
        declared_size: redeemed.translated_size_bytes,
        expected_plaintext_sha256: redeemed.artifact_receipt_sha256.to_vec(),
        backup_class: 1,
    }
    .encode_to_vec())
}

fn source_request_record(
    logical_owner_id: &str,
    runtime: &AttachmentTranslationClientRuntimeContextV1<'_>,
    draft: &AttachmentTranslationDraftV1,
    now_unix_millis: i64,
) -> Result<makosh_events_protocol::delivery::OutboxRecordV1, AttachmentTranslationClientPortErrorV1>
{
    let seconds = now_unix_millis / 1_000;
    let nanos = i32::try_from((now_unix_millis % 1_000) * 1_000_000)
        .map_err(|_| AttachmentTranslationClientPortErrorV1::Protocol)?;
    let deadline = seconds
        .checked_add(300)
        .ok_or(AttachmentTranslationClientPortErrorV1::Protocol)?;
    let request_id = attachment_translation_source_request_id_v1(
        draft.run_id,
        draft.source_extraction_run_id,
        draft.expected_source_revision,
    );
    build_request_attachment_translation_source_outbox_record_v1(
        RequestAttachmentTranslationSourceV1 {
            request_id: request_id.to_vec(),
            translation_run_id: draft.run_id.to_vec(),
            source_extraction_run_id: draft.source_extraction_run_id.to_vec(),
            expected_source_revision: draft.expected_source_revision,
            logical_owner_id: logical_owner_id.to_owned(),
        },
        deadline,
        &AttachmentTranslationSourceEnvelopeContextV1 {
            module_id: ATTACHMENT_TRANSLATION_MODULE_ID_V1.to_owned(),
            runtime_instance_id: runtime.runtime_instance_id.to_owned(),
            runtime_generation: runtime.runtime_generation,
            recorded_at_unix_seconds: seconds,
            recorded_at_nanos: nanos,
        },
    )
    .map_err(|_| AttachmentTranslationClientPortErrorV1::Unavailable)
}

fn get_response(run: PersistedAttachmentTranslationRunV1) -> GetAttachmentTranslationResponseV1 {
    GetAttachmentTranslationResponseV1 {
        run_id: run.draft.run_id.to_vec(),
        source_extraction_run_id: run.draft.source_extraction_run_id.to_vec(),
        expected_source_revision: run.draft.expected_source_revision,
        state: wire_state(run.status.state) as i32,
        state_revision: run.status.state_revision,
        artifact: run.status.artifact.map(wire_artifact),
        error: rejection_error(run.status.rejection) as i32,
    }
}

pub(crate) fn wire_artifact(artifact: AttachmentTranslationArtifactV1) -> WireArtifact {
    WireArtifact {
        translated_sha256: artifact.translated_sha256.to_vec(),
        translated_size_bytes: artifact.translated_size_bytes,
        detected_source_language: wire_detected_language(artifact.detected_source_language) as i32,
        target_language: wire_target_language(artifact.target_language) as i32,
        completeness: wire_completeness(artifact.completeness) as i32,
        confidence_basis_points: artifact.confidence_basis_points,
    }
}

fn validate_request(
    request: &ModuleClientRequestV1,
    runtime: &AttachmentTranslationClientRuntimeContextV1<'_>,
    now_unix_millis: i64,
) -> Result<(), AttachmentTranslationClientPortErrorV1> {
    if request.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR_V1
        || request.module_id != ATTACHMENT_TRANSLATION_MODULE_ID_V1
        || request.owner_id != ATTACHMENT_TRANSLATION_OWNER_V1
        || request.request_id == 0
        || request.request_payload.is_empty()
        || !valid_identity(&request.logical_owner_id)
        || !valid_identity(&request.authenticated_device_id)
        || runtime.runtime_instance_id.is_empty()
        || runtime.runtime_generation == 0
        || runtime.grant_epoch == 0
        || now_unix_millis <= 0
    {
        Err(AttachmentTranslationClientPortErrorV1::Protocol)
    } else {
        Ok(())
    }
}

fn run_id(logical_owner_id: &str, operation_id: &[u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.attachment_translation.run.v1\0");
    digest.update(logical_owner_id.as_bytes());
    digest.update([0]);
    digest.update(operation_id);
    digest.finalize()[..16].try_into().expect("digest prefix")
}

fn device_actor_sha256(logical_owner_id: &str, authenticated_device_id: &str) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.attachment-translation.device-actor.v1\0");
    digest.update(logical_owner_id.as_bytes());
    digest.update([0]);
    digest.update(authenticated_device_id.as_bytes());
    digest.finalize().into()
}

fn id16(value: &[u8]) -> Result<[u8; 16], AttachmentTranslationClientPortErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(AttachmentTranslationClientPortErrorV1::Protocol)
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn target_language(
    value: i32,
) -> Result<AttachmentTranslationLanguageV1, AttachmentTranslationClientPortErrorV1> {
    match WireLanguage::try_from(value)
        .map_err(|_| AttachmentTranslationClientPortErrorV1::Protocol)?
    {
        WireLanguage::AttachmentTranslationLanguageEnglish => {
            Ok(AttachmentTranslationLanguageV1::English)
        }
        WireLanguage::AttachmentTranslationLanguageRussian => {
            Ok(AttachmentTranslationLanguageV1::Russian)
        }
        WireLanguage::AttachmentTranslationLanguageSpanish => {
            Ok(AttachmentTranslationLanguageV1::Spanish)
        }
        WireLanguage::AttachmentTranslationLanguageUnspecified => {
            Err(AttachmentTranslationClientPortErrorV1::Protocol)
        }
    }
}

pub(crate) const fn wire_state(value: AttachmentTranslationStateV1) -> WireState {
    match value {
        AttachmentTranslationStateV1::Accepted => WireState::AttachmentTranslationStateAccepted,
        AttachmentTranslationStateV1::AwaitingSource => {
            WireState::AttachmentTranslationStateAwaitingSource
        }
        AttachmentTranslationStateV1::AwaitingInference => {
            WireState::AttachmentTranslationStateAwaitingInference
        }
        AttachmentTranslationStateV1::MaterializingResult => {
            WireState::AttachmentTranslationStateMaterializingResult
        }
        AttachmentTranslationStateV1::Ready => WireState::AttachmentTranslationStateReady,
        AttachmentTranslationStateV1::Rejected => WireState::AttachmentTranslationStateRejected,
    }
}

const fn wire_target_language(value: AttachmentTranslationLanguageV1) -> WireLanguage {
    match value {
        AttachmentTranslationLanguageV1::English => {
            WireLanguage::AttachmentTranslationLanguageEnglish
        }
        AttachmentTranslationLanguageV1::Russian => {
            WireLanguage::AttachmentTranslationLanguageRussian
        }
        AttachmentTranslationLanguageV1::Spanish => {
            WireLanguage::AttachmentTranslationLanguageSpanish
        }
    }
}

const fn wire_detected_language(
    value: AttachmentTranslationDetectedLanguageV1,
) -> WireDetectedLanguage {
    match value {
        AttachmentTranslationDetectedLanguageV1::Unknown => {
            WireDetectedLanguage::AttachmentTranslationDetectedLanguageUnknown
        }
        AttachmentTranslationDetectedLanguageV1::English => {
            WireDetectedLanguage::AttachmentTranslationDetectedLanguageEnglish
        }
        AttachmentTranslationDetectedLanguageV1::Russian => {
            WireDetectedLanguage::AttachmentTranslationDetectedLanguageRussian
        }
        AttachmentTranslationDetectedLanguageV1::Spanish => {
            WireDetectedLanguage::AttachmentTranslationDetectedLanguageSpanish
        }
    }
}

const fn wire_completeness(value: AttachmentTranslationCompletenessV1) -> WireCompleteness {
    match value {
        AttachmentTranslationCompletenessV1::Complete => {
            WireCompleteness::AttachmentTranslationCompletenessComplete
        }
        AttachmentTranslationCompletenessV1::Partial => {
            WireCompleteness::AttachmentTranslationCompletenessPartial
        }
    }
}

pub(crate) const fn rejection_error(
    value: Option<AttachmentTranslationRejectionCodeV1>,
) -> WireError {
    match value {
        None => WireError::AttachmentTranslationErrorCodeUnspecified,
        Some(AttachmentTranslationRejectionCodeV1::InvalidRequest) => {
            WireError::AttachmentTranslationErrorCodeInvalidRequest
        }
        Some(AttachmentTranslationRejectionCodeV1::SourceRejected) => {
            WireError::AttachmentTranslationErrorCodeSourceRejected
        }
        Some(AttachmentTranslationRejectionCodeV1::InferenceRejected) => {
            WireError::AttachmentTranslationErrorCodeInferenceRejected
        }
        Some(AttachmentTranslationRejectionCodeV1::ResultRejected) => {
            WireError::AttachmentTranslationErrorCodeResultRejected
        }
        Some(AttachmentTranslationRejectionCodeV1::Policy) => {
            WireError::AttachmentTranslationErrorCodePolicy
        }
    }
}

fn start_error(error: WireError) -> StartAttachmentTranslationResponseV1 {
    StartAttachmentTranslationResponseV1 {
        run_id: Vec::new(),
        state: WireState::AttachmentTranslationStateUnspecified as i32,
        error: error as i32,
    }
}

fn ticket_error(run_id: [u8; 16], error: WireError) -> IssueAttachmentTranslationReadResponseV1 {
    IssueAttachmentTranslationReadResponseV1 {
        run_id: run_id.to_vec(),
        opaque_read_ticket: Vec::new(),
        expires_at_unix_seconds: 0,
        translated_size_bytes: 0,
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

    #[test]
    fn identities_are_owner_operation_and_device_bound() {
        assert_ne!(run_id("owner-a", &[1; 16]), run_id("owner-b", &[1; 16]));
        assert_ne!(
            device_actor_sha256("owner-a", "device-a"),
            device_actor_sha256("owner-a", "device-b")
        );
    }
}
