use makosh_attachment_preview_evidence_replay_api::{
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CONTRACT_MAJOR_V1,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MODULE_ID_V1, ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_OWNER_V1,
    wire::{
        AttachmentPreviewEvidenceReplayErrorV1, AttachmentPreviewEvidenceReplayStateV1,
        StartAttachmentPreviewEvidenceReplayRequestV1,
        StartAttachmentPreviewEvidenceReplayResponseV1,
    },
};
use makosh_attachment_preview_evidence_replay_core::{
    AuthenticatedReplayOperationRequestV1, ReplayProducerV1,
};
use makosh_attachment_preview_evidence_replay_persistence::{
    AttachmentPreviewEvidenceReplayPersistenceV1, ReplayCommandOutboxRecordV1,
    ReplayOperationCreateOutcomeV1, ReplayPersistenceErrorV1,
};
use makosh_communications_retained_evidence_replay_contract::{
    CommunicationsReplayCommandEnvelopeContextV1, build_communications_replay_command_outbox_v1,
    wire::ReplayCommunicationsEvidenceCommandV1,
};
use makosh_mail_retained_evidence_replay_contract::{
    MailReplayCommandEnvelopeContextV1, build_mail_replay_command_outbox_v1,
    wire::ReplayMailEvidenceCommandV1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::contracts::client_command_contract_v1;

const MODULE_CLIENT_PROTOCOL_MAJOR_V1: u32 = 1;
const COMMAND_DEADLINE_SECONDS_V1: i64 = 300;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayClientRuntimeContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub now_unix_seconds: i64,
    pub now_nanos: i32,
}

pub async fn dispatch_replay_client_request_v1(
    persistence: &AttachmentPreviewEvidenceReplayPersistenceV1,
    request: &ModuleClientRequestV1,
    context: &ReplayClientRuntimeContextV1,
) -> ModuleClientResponseV1 {
    let payload = match start_request(request, context) {
        Ok(payload) => start(persistence, request, payload, context).await,
        Err(error) => start_error(error).encode_to_vec(),
    };
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR_V1,
        request_id: request.request_id,
        response_payload: payload,
        error_code: String::new(),
    }
}

async fn start(
    persistence: &AttachmentPreviewEvidenceReplayPersistenceV1,
    module_request: &ModuleClientRequestV1,
    payload: StartAttachmentPreviewEvidenceReplayRequestV1,
    context: &ReplayClientRuntimeContextV1,
) -> Vec<u8> {
    let Some(request) = authenticated_request(module_request, payload) else {
        return start_error(AttachmentPreviewEvidenceReplayErrorV1::InvalidRequest).encode_to_vec();
    };
    let commands = match command_records(&request, context) {
        Ok(commands) => commands,
        Err(()) => {
            return start_error(AttachmentPreviewEvidenceReplayErrorV1::Unavailable)
                .encode_to_vec();
        }
    };
    let response = match persistence
        .create_operation(&request, commands, context.now_unix_seconds)
        .await
    {
        Ok(ReplayOperationCreateOutcomeV1::Created(operation))
        | Ok(ReplayOperationCreateOutcomeV1::Replayed(operation)) => {
            StartAttachmentPreviewEvidenceReplayResponseV1 {
                operation_id: operation.request.operation_id.to_vec(),
                state: operation.state as i32,
                error: operation.error as i32,
            }
        }
        Ok(ReplayOperationCreateOutcomeV1::OperationCollision)
        | Err(ReplayPersistenceErrorV1::Conflict) => response_error(
            request.operation_id,
            AttachmentPreviewEvidenceReplayStateV1::Rejected,
            AttachmentPreviewEvidenceReplayErrorV1::Conflict,
        ),
        Err(ReplayPersistenceErrorV1::InvalidInput | ReplayPersistenceErrorV1::WrongContract) => {
            response_error(
                request.operation_id,
                AttachmentPreviewEvidenceReplayStateV1::Rejected,
                AttachmentPreviewEvidenceReplayErrorV1::InvalidRequest,
            )
        }
        Err(_) => response_error(
            request.operation_id,
            AttachmentPreviewEvidenceReplayStateV1::Unavailable,
            AttachmentPreviewEvidenceReplayErrorV1::Unavailable,
        ),
    };
    response.encode_to_vec()
}

fn start_request(
    request: &ModuleClientRequestV1,
    context: &ReplayClientRuntimeContextV1,
) -> Result<StartAttachmentPreviewEvidenceReplayRequestV1, AttachmentPreviewEvidenceReplayErrorV1> {
    if request.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR_V1
        || request.module_id != ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MODULE_ID_V1
        || request.owner_id != ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_OWNER_V1
        || request.contract.as_ref() != Some(&client_command_contract_v1())
        || request.request_id == 0
        || request.request_payload.is_empty()
        || !valid_identity(&request.logical_owner_id)
        || !valid_identity(&request.authenticated_device_id)
        || !valid_context(context)
    {
        return Err(AttachmentPreviewEvidenceReplayErrorV1::InvalidRequest);
    }
    let payload =
        StartAttachmentPreviewEvidenceReplayRequestV1::decode(request.request_payload.as_slice())
            .map_err(|_| AttachmentPreviewEvidenceReplayErrorV1::InvalidRequest)?;
    (payload.protocol_major == ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_CONTRACT_MAJOR_V1)
        .then_some(payload)
        .ok_or(AttachmentPreviewEvidenceReplayErrorV1::InvalidRequest)
}

fn authenticated_request(
    module_request: &ModuleClientRequestV1,
    payload: StartAttachmentPreviewEvidenceReplayRequestV1,
) -> Option<AuthenticatedReplayOperationRequestV1> {
    Some(AuthenticatedReplayOperationRequestV1 {
        operation_id: id16(&payload.operation_id)?,
        attachment_anchor_id: id16(&payload.attachment_anchor_id)?,
        logical_owner_id: module_request.logical_owner_id.clone(),
        owner_device_actor_sha256: device_actor_sha256_v1(
            &module_request.logical_owner_id,
            &module_request.authenticated_device_id,
        ),
    })
}

fn command_records(
    request: &AuthenticatedReplayOperationRequestV1,
    context: &ReplayClientRuntimeContextV1,
) -> Result<[ReplayCommandOutboxRecordV1; 2], ()> {
    let communications = build_communications_replay_command_outbox_v1(
        ReplayCommunicationsEvidenceCommandV1 {
            operation_id: request.operation_id.to_vec(),
            logical_owner_id: request.logical_owner_id.clone(),
            owner_device_actor_sha256: request.owner_device_actor_sha256.to_vec(),
            attachment_anchor_id: request.attachment_anchor_id.to_vec(),
        },
        &CommunicationsReplayCommandEnvelopeContextV1 {
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            recorded_at_unix_seconds: context.now_unix_seconds,
            recorded_at_nanos: context.now_nanos,
            deadline_unix_seconds: context.now_unix_seconds + COMMAND_DEADLINE_SECONDS_V1,
            logical_attempt: 1,
        },
    )
    .map_err(|_| ())?;
    let mail = build_mail_replay_command_outbox_v1(
        ReplayMailEvidenceCommandV1 {
            operation_id: request.operation_id.to_vec(),
            logical_owner_id: request.logical_owner_id.clone(),
            owner_device_actor_sha256: request.owner_device_actor_sha256.to_vec(),
            attachment_anchor_id: request.attachment_anchor_id.to_vec(),
        },
        &MailReplayCommandEnvelopeContextV1 {
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            recorded_at_unix_seconds: context.now_unix_seconds,
            recorded_at_nanos: context.now_nanos,
            deadline_unix_seconds: context.now_unix_seconds + COMMAND_DEADLINE_SECONDS_V1,
            logical_attempt: 1,
        },
    )
    .map_err(|_| ())?;
    Ok([
        ReplayCommandOutboxRecordV1::accept(
            ReplayProducerV1::Communications,
            communications.exact_bytes().to_vec(),
        )
        .map_err(|_| ())?,
        ReplayCommandOutboxRecordV1::accept(ReplayProducerV1::Mail, mail.exact_bytes().to_vec())
            .map_err(|_| ())?,
    ])
}

fn response_error(
    operation_id: [u8; 16],
    state: AttachmentPreviewEvidenceReplayStateV1,
    error: AttachmentPreviewEvidenceReplayErrorV1,
) -> StartAttachmentPreviewEvidenceReplayResponseV1 {
    StartAttachmentPreviewEvidenceReplayResponseV1 {
        operation_id: operation_id.to_vec(),
        state: state as i32,
        error: error as i32,
    }
}

fn start_error(
    error: AttachmentPreviewEvidenceReplayErrorV1,
) -> StartAttachmentPreviewEvidenceReplayResponseV1 {
    StartAttachmentPreviewEvidenceReplayResponseV1 {
        operation_id: Vec::new(),
        state: AttachmentPreviewEvidenceReplayStateV1::Rejected as i32,
        error: error as i32,
    }
}

fn device_actor_sha256_v1(logical_owner_id: &str, authenticated_device_id: &str) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.attachment-preview-evidence-replay.device-actor.v1\0");
    digest.update(logical_owner_id.as_bytes());
    digest.update([0]);
    digest.update(authenticated_device_id.as_bytes());
    digest.finalize().into()
}

fn id16(value: &[u8]) -> Option<[u8; 16]> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
}

fn valid_context(value: &ReplayClientRuntimeContextV1) -> bool {
    valid_identity(&value.runtime_instance_id)
        && value.runtime_generation > 0
        && value.grant_epoch > 0
        && value.now_unix_seconds > 0
        && (0..1_000_000_000).contains(&value.now_nanos)
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_actor_is_derived_only_from_authenticated_envelope_identity() {
        assert_eq!(
            device_actor_sha256_v1("owner-1", "device-1"),
            device_actor_sha256_v1("owner-1", "device-1")
        );
        assert_ne!(
            device_actor_sha256_v1("owner-1", "device-1"),
            device_actor_sha256_v1("owner-1", "device-2")
        );
    }
}
