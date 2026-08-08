//! Generated client contracts and private `client_blob` authorization.

use makosh_attachment_preview_api::{
    ATTACHMENT_PREVIEW_CONTRACT_MAJOR_V1, ATTACHMENT_PREVIEW_MODULE_ID_V1,
    ATTACHMENT_PREVIEW_OWNER_V1, ATTACHMENT_PREVIEW_READ_TICKET_BYTES_V1,
    read_wire::ReadAttachmentPreviewRequestV1,
    wire::{
        AttachmentPreviewContentTypeV1, AttachmentPreviewErrorCodeV1, AttachmentPreviewKindV1,
        AttachmentPreviewStateV1, GetAttachmentPreviewRequestV1, GetAttachmentPreviewResponseV1,
        IssueAttachmentPreviewReadRequestV1, IssueAttachmentPreviewReadResponseV1,
        StartAttachmentPreviewRequestV1, StartAttachmentPreviewResponseV1,
    },
};
use makosh_attachment_preview_persistence::{
    AttachmentPreviewPersistenceErrorV1, AttachmentPreviewPersistenceV1,
    CreateAttachmentPreviewRunOutcomeV1, CreateAttachmentPreviewRunV1,
    IssueAttachmentPreviewTicketV1, PersistedAttachmentPreviewRunV1,
};
use makosh_runtime_protocol::v1::{
    ModuleClientBlobAuthorizationV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::contracts::{
    command_contract_v1, query_contract_v1, read_contract_v1, ticket_contract_v1,
};

const MODULE_CLIENT_PROTOCOL_MAJOR_V1: u32 = 1;

pub async fn dispatch_attachment_preview_client_request_v1(
    persistence: &AttachmentPreviewPersistenceV1,
    runtime_generation: u64,
    grant_epoch: u64,
    request: &ModuleClientRequestV1,
    now_unix_millis: i64,
) -> ModuleClientResponseV1 {
    let result =
        if validate_request(request, runtime_generation, grant_epoch, now_unix_millis).is_err() {
            Err(AttachmentPreviewClientPortErrorV1::Protocol)
        } else if request.contract.as_ref() == Some(&command_contract_v1()) {
            start(persistence, request, now_unix_millis).await
        } else if request.contract.as_ref() == Some(&query_contract_v1()) {
            get(persistence, request).await
        } else if request.contract.as_ref() == Some(&ticket_contract_v1()) {
            issue_read(
                persistence,
                runtime_generation,
                grant_epoch,
                request,
                now_unix_millis / 1_000,
            )
            .await
        } else if request.contract.as_ref() == Some(&read_contract_v1()) {
            authorize_read(
                persistence,
                runtime_generation,
                grant_epoch,
                request,
                now_unix_millis / 1_000,
            )
            .await
        } else {
            Err(AttachmentPreviewClientPortErrorV1::Protocol)
        };
    match result {
        Ok(payload) => response(request.request_id, payload),
        Err(AttachmentPreviewClientPortErrorV1::Protocol) => error(request.request_id, "REJECTED"),
        Err(AttachmentPreviewClientPortErrorV1::NotFound) => error(request.request_id, "NOT_FOUND"),
        Err(AttachmentPreviewClientPortErrorV1::Unavailable) => {
            error(request.request_id, "UNAVAILABLE")
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentPreviewClientPortErrorV1 {
    Protocol,
    NotFound,
    Unavailable,
}

async fn start(
    persistence: &AttachmentPreviewPersistenceV1,
    request: &ModuleClientRequestV1,
    now_unix_millis: i64,
) -> Result<Vec<u8>, AttachmentPreviewClientPortErrorV1> {
    let payload = StartAttachmentPreviewRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| AttachmentPreviewClientPortErrorV1::Protocol)?;
    if payload.protocol_major != ATTACHMENT_PREVIEW_CONTRACT_MAJOR_V1 {
        return Err(AttachmentPreviewClientPortErrorV1::Protocol);
    }
    let create = CreateAttachmentPreviewRunV1 {
        logical_owner_id: request.logical_owner_id.clone(),
        operation_id: id16(&payload.operation_id)?,
        attachment_anchor_id: id16(&payload.attachment_anchor_id)?,
        created_at_unix_millis: now_unix_millis,
    };
    let response = match persistence.create_run(&create).await {
        Ok(CreateAttachmentPreviewRunOutcomeV1::Created(run))
        | Ok(CreateAttachmentPreviewRunOutcomeV1::Replayed(run)) => {
            StartAttachmentPreviewResponseV1 {
                run_id: run.run_id.to_vec(),
                state: run.status.state as i32,
                error: AttachmentPreviewErrorCodeV1::Unspecified as i32,
            }
        }
        Ok(CreateAttachmentPreviewRunOutcomeV1::OperationCollision)
        | Err(AttachmentPreviewPersistenceErrorV1::InvalidInput) => {
            start_error(AttachmentPreviewErrorCodeV1::InvalidRequest)
        }
        Err(_) => start_error(AttachmentPreviewErrorCodeV1::Unavailable),
    };
    Ok(response.encode_to_vec())
}

async fn get(
    persistence: &AttachmentPreviewPersistenceV1,
    request: &ModuleClientRequestV1,
) -> Result<Vec<u8>, AttachmentPreviewClientPortErrorV1> {
    let payload = GetAttachmentPreviewRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| AttachmentPreviewClientPortErrorV1::Protocol)?;
    if payload.protocol_major != ATTACHMENT_PREVIEW_CONTRACT_MAJOR_V1 {
        return Err(AttachmentPreviewClientPortErrorV1::Protocol);
    }
    let run_id = id16(&payload.run_id)?;
    let response = match persistence
        .find_run(&request.logical_owner_id, run_id)
        .await
        .map_err(|_| AttachmentPreviewClientPortErrorV1::Unavailable)?
    {
        Some(run) => get_response(run),
        None => GetAttachmentPreviewResponseV1 {
            run_id: payload.run_id,
            error: AttachmentPreviewErrorCodeV1::NotFound as i32,
            ..Default::default()
        },
    };
    Ok(response.encode_to_vec())
}

async fn issue_read(
    persistence: &AttachmentPreviewPersistenceV1,
    runtime_generation: u64,
    grant_epoch: u64,
    request: &ModuleClientRequestV1,
    now_unix_seconds: i64,
) -> Result<Vec<u8>, AttachmentPreviewClientPortErrorV1> {
    let payload = IssueAttachmentPreviewReadRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| AttachmentPreviewClientPortErrorV1::Protocol)?;
    if payload.protocol_major != ATTACHMENT_PREVIEW_CONTRACT_MAJOR_V1 {
        return Err(AttachmentPreviewClientPortErrorV1::Protocol);
    }
    let run_id = id16(&payload.run_id)?;
    let mut opaque_ticket = [0_u8; ATTACHMENT_PREVIEW_READ_TICKET_BYTES_V1];
    getrandom::fill(&mut opaque_ticket)
        .map_err(|_| AttachmentPreviewClientPortErrorV1::Unavailable)?;
    let ticket_sha256 = Sha256::digest(opaque_ticket).into();
    let device_actor_sha256 =
        device_actor_sha256_v1(&request.logical_owner_id, &request.authenticated_device_id);
    let response = match persistence
        .issue_read_ticket(
            &request.logical_owner_id,
            IssueAttachmentPreviewTicketV1 {
                ticket_sha256,
                device_actor_sha256,
                run_id,
                runtime_generation,
                grant_epoch,
                now_unix_seconds,
            },
        )
        .await
    {
        Ok(issued) => IssueAttachmentPreviewReadResponseV1 {
            run_id: issued.run_id.to_vec(),
            opaque_read_ticket: opaque_ticket.to_vec(),
            expires_at_unix_seconds: u64::try_from(issued.expires_at_unix_seconds)
                .map_err(|_| AttachmentPreviewClientPortErrorV1::Unavailable)?,
            content_type: issued.content_type as i32,
            preview_size_bytes: issued.preview_size_bytes,
            error: AttachmentPreviewErrorCodeV1::Unspecified as i32,
        },
        Err(AttachmentPreviewPersistenceErrorV1::NotFound) => {
            ticket_error(run_id, AttachmentPreviewErrorCodeV1::NotFound)
        }
        Err(AttachmentPreviewPersistenceErrorV1::StaleFence) => {
            ticket_error(run_id, AttachmentPreviewErrorCodeV1::Unavailable)
        }
        Err(AttachmentPreviewPersistenceErrorV1::InvalidInput) => {
            ticket_error(run_id, AttachmentPreviewErrorCodeV1::InvalidRequest)
        }
        Err(_) => ticket_error(run_id, AttachmentPreviewErrorCodeV1::Unavailable),
    };
    Ok(response.encode_to_vec())
}

async fn authorize_read(
    persistence: &AttachmentPreviewPersistenceV1,
    runtime_generation: u64,
    grant_epoch: u64,
    request: &ModuleClientRequestV1,
    now_unix_seconds: i64,
) -> Result<Vec<u8>, AttachmentPreviewClientPortErrorV1> {
    let payload = ReadAttachmentPreviewRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| AttachmentPreviewClientPortErrorV1::Protocol)?;
    if payload.protocol_major != ATTACHMENT_PREVIEW_CONTRACT_MAJOR_V1 {
        return Err(AttachmentPreviewClientPortErrorV1::Protocol);
    }
    let ticket: [u8; ATTACHMENT_PREVIEW_READ_TICKET_BYTES_V1] = payload
        .opaque_read_ticket
        .try_into()
        .map_err(|_| AttachmentPreviewClientPortErrorV1::Protocol)?;
    let redeemed = persistence
        .redeem_read_ticket(
            &request.logical_owner_id,
            Sha256::digest(ticket).into(),
            device_actor_sha256_v1(&request.logical_owner_id, &request.authenticated_device_id),
            runtime_generation,
            grant_epoch,
            now_unix_seconds,
        )
        .await
        .map_err(|error| match error {
            AttachmentPreviewPersistenceErrorV1::NotFound
            | AttachmentPreviewPersistenceErrorV1::TicketExpired
            | AttachmentPreviewPersistenceErrorV1::TicketUsed
            | AttachmentPreviewPersistenceErrorV1::StaleFence => {
                AttachmentPreviewClientPortErrorV1::NotFound
            }
            _ => AttachmentPreviewClientPortErrorV1::Unavailable,
        })?;
    Ok(ModuleClientBlobAuthorizationV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR_V1,
        reference_id: redeemed.derived_reference_id.to_vec(),
        declared_size: redeemed.preview_size_bytes,
        expected_plaintext_sha256: redeemed.derived_receipt_sha256.to_vec(),
        backup_class: 1,
    }
    .encode_to_vec())
}

fn validate_request(
    request: &ModuleClientRequestV1,
    runtime_generation: u64,
    grant_epoch: u64,
    now_unix_millis: i64,
) -> Result<(), AttachmentPreviewClientPortErrorV1> {
    if request.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR_V1
        || request.module_id != ATTACHMENT_PREVIEW_MODULE_ID_V1
        || request.owner_id != ATTACHMENT_PREVIEW_OWNER_V1
        || request.request_id == 0
        || request.request_payload.is_empty()
        || !valid_identity(&request.logical_owner_id)
        || !valid_identity(&request.authenticated_device_id)
        || runtime_generation == 0
        || grant_epoch == 0
        || now_unix_millis <= 0
    {
        Err(AttachmentPreviewClientPortErrorV1::Protocol)
    } else {
        Ok(())
    }
}

fn get_response(run: PersistedAttachmentPreviewRunV1) -> GetAttachmentPreviewResponseV1 {
    GetAttachmentPreviewResponseV1 {
        run_id: run.run_id.to_vec(),
        attachment_anchor_id: run.attachment_anchor_id.to_vec(),
        state: run.status.state as i32,
        state_revision: run.status.state_revision,
        preview_kind: run
            .status
            .preview_kind
            .unwrap_or(AttachmentPreviewKindV1::Unspecified) as i32,
        content_type: run
            .status
            .content_type
            .unwrap_or(AttachmentPreviewContentTypeV1::Unspecified) as i32,
        preview_size_bytes: run.status.preview_size_bytes,
        truncated: run.status.truncated,
        error: run
            .status
            .error
            .unwrap_or(AttachmentPreviewErrorCodeV1::Unspecified) as i32,
    }
}

fn start_error(error: AttachmentPreviewErrorCodeV1) -> StartAttachmentPreviewResponseV1 {
    StartAttachmentPreviewResponseV1 {
        run_id: Vec::new(),
        state: AttachmentPreviewStateV1::Unspecified as i32,
        error: error as i32,
    }
}

fn ticket_error(
    run_id: [u8; 16],
    error: AttachmentPreviewErrorCodeV1,
) -> IssueAttachmentPreviewReadResponseV1 {
    IssueAttachmentPreviewReadResponseV1 {
        run_id: run_id.to_vec(),
        opaque_read_ticket: Vec::new(),
        expires_at_unix_seconds: 0,
        content_type: AttachmentPreviewContentTypeV1::Unspecified as i32,
        preview_size_bytes: 0,
        error: error as i32,
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], AttachmentPreviewClientPortErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(AttachmentPreviewClientPortErrorV1::Protocol)
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn device_actor_sha256_v1(logical_owner_id: &str, authenticated_device_id: &str) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.attachment-preview.device-actor.v1\0");
    digest.update(logical_owner_id.as_bytes());
    digest.update([0]);
    digest.update(authenticated_device_id.as_bytes());
    digest.finalize().into()
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
    fn device_actor_binding_is_owner_and_device_specific() {
        assert_eq!(
            device_actor_sha256_v1("owner-a", "device-a"),
            device_actor_sha256_v1("owner-a", "device-a")
        );
        assert_ne!(
            device_actor_sha256_v1("owner-a", "device-a"),
            device_actor_sha256_v1("owner-a", "device-b")
        );
        assert_ne!(
            device_actor_sha256_v1("owner-a", "device-a"),
            device_actor_sha256_v1("owner-b", "device-a")
        );
    }

    #[test]
    fn exact_ids_are_required() {
        assert_eq!(id16(&[1; 16]), Ok([1; 16]));
        assert_eq!(
            id16(&[0; 16]),
            Err(AttachmentPreviewClientPortErrorV1::Protocol)
        );
        assert_eq!(
            id16(&[1; 15]),
            Err(AttachmentPreviewClientPortErrorV1::Protocol)
        );
    }
}
