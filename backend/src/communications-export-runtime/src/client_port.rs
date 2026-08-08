//! Generated client contract adapter for export command, status, ticket and
//! descriptor-declared client_blob authorization.

use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use makosh_communications_evidence_export_source_api::{
    EvidenceExportEnvelopeContextV1, build_evidence_export_prepare_outbox_record_v1,
};
use makosh_communications_export_api::{
    COMMUNICATIONS_EXPORT_CONTRACT_MAJOR_V1, COMMUNICATIONS_EXPORT_MAX_MESSAGES_V1,
    COMMUNICATIONS_EXPORT_MODULE_ID_V1, COMMUNICATIONS_EXPORT_OWNER_V1,
    COMMUNICATIONS_EXPORT_READ_TICKET_BYTES_V1,
    wire::{
        CommunicationsExportErrorCodeV1, EvidenceExportArtifactReadRequestV1,
        EvidenceExportStatusV1, GetEvidenceExportStatusRequestV1,
        GetEvidenceExportStatusResponseV1, IssueEvidenceExportReadRequestV1,
        IssueEvidenceExportReadResponseV1, StartEvidenceExportRequestV1,
        StartEvidenceExportResponseV1,
    },
};
use makosh_communications_export_persistence::{
    CommunicationsExportPersistenceErrorV1, CommunicationsExportPersistenceV1,
};
use makosh_runtime_protocol::v1::{
    ModuleClientBlobAuthorizationV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;

use crate::{
    admission::{
        communications_export_command_contract_reference_v1,
        communications_export_query_contract_reference_v1,
        communications_export_read_contract_reference_v1,
        communications_export_ticket_contract_reference_v1,
    },
    ticket_store::CommunicationsExportTicketStoreV1,
};

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsExportClientPortErrorV1 {
    Protocol,
    NotFound,
    Unavailable,
}

pub async fn dispatch_communications_export_client_request_v1(
    persistence: &CommunicationsExportPersistenceV1,
    tickets: &Arc<CommunicationsExportTicketStoreV1>,
    runtime_instance_id: &str,
    runtime_generation: u64,
    grant_epoch: u64,
    request: &ModuleClientRequestV1,
) -> ModuleClientResponseV1 {
    let result = if request.contract.as_ref()
        == Some(&communications_export_command_contract_reference_v1())
    {
        start_export(
            persistence,
            runtime_instance_id,
            runtime_generation,
            request,
        )
        .await
    } else if request.contract.as_ref()
        == Some(&communications_export_query_contract_reference_v1())
    {
        get_status(persistence, request).await
    } else if request.contract.as_ref()
        == Some(&communications_export_ticket_contract_reference_v1())
    {
        issue_read_ticket(persistence, tickets, request).await
    } else if request.contract.as_ref() == Some(&communications_export_read_contract_reference_v1())
    {
        authorize_blob_read(
            persistence,
            tickets,
            runtime_generation,
            grant_epoch,
            request,
        )
        .await
    } else {
        Err(CommunicationsExportClientPortErrorV1::Protocol)
    };
    match result {
        Ok(response_payload) => module_response(request.request_id, response_payload),
        Err(CommunicationsExportClientPortErrorV1::Protocol) => {
            module_error(request.request_id, "REJECTED")
        }
        Err(CommunicationsExportClientPortErrorV1::NotFound) => {
            module_error(request.request_id, "NOT_FOUND")
        }
        Err(CommunicationsExportClientPortErrorV1::Unavailable) => {
            module_error(request.request_id, "UNAVAILABLE")
        }
    }
}

async fn start_export(
    persistence: &CommunicationsExportPersistenceV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    request: &ModuleClientRequestV1,
) -> Result<Vec<u8>, CommunicationsExportClientPortErrorV1> {
    validate_request(request)?;
    let payload = StartEvidenceExportRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| CommunicationsExportClientPortErrorV1::Protocol)?;
    let export_id = id16(&payload.operation_id)?;
    let message_ids = bounded_message_ids(&payload.message_ids)?;
    if payload.protocol_major != COMMUNICATIONS_EXPORT_CONTRACT_MAJOR_V1 {
        return Err(CommunicationsExportClientPortErrorV1::Protocol);
    }
    let now = now_unix_seconds()?;
    let outbox = build_evidence_export_prepare_outbox_record_v1(
        export_id,
        &request.logical_owner_id,
        &message_ids,
        now.checked_add(30)
            .ok_or(CommunicationsExportClientPortErrorV1::Unavailable)?,
        &EvidenceExportEnvelopeContextV1 {
            module_id: COMMUNICATIONS_EXPORT_MODULE_ID_V1.to_owned(),
            runtime_instance_id: runtime_instance_id.to_owned(),
            runtime_generation,
            recorded_at_unix_seconds: now,
            recorded_at_nanos: 0,
        },
    )
    .map_err(|_| CommunicationsExportClientPortErrorV1::Protocol)?;
    let response = match persistence
        .create_export_with_outbox(
            export_id,
            &request.logical_owner_id,
            &message_ids,
            &outbox,
            now,
        )
        .await
    {
        Ok(()) => StartEvidenceExportResponseV1 {
            export_id: export_id.to_vec(),
            error: CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodeUnspecified as i32,
        },
        Err(CommunicationsExportPersistenceErrorV1::Conflict)
        | Err(CommunicationsExportPersistenceErrorV1::InvalidInput) => {
            StartEvidenceExportResponseV1 {
                export_id: Vec::new(),
                error: CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodeInvalidRequest
                    as i32,
            }
        }
        Err(_) => return Err(CommunicationsExportClientPortErrorV1::Unavailable),
    };
    Ok(response.encode_to_vec())
}

async fn get_status(
    persistence: &CommunicationsExportPersistenceV1,
    request: &ModuleClientRequestV1,
) -> Result<Vec<u8>, CommunicationsExportClientPortErrorV1> {
    validate_request(request)?;
    let payload = GetEvidenceExportStatusRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| CommunicationsExportClientPortErrorV1::Protocol)?;
    let export_id = id16(&payload.export_id)?;
    if payload.protocol_major != COMMUNICATIONS_EXPORT_CONTRACT_MAJOR_V1 {
        return Err(CommunicationsExportClientPortErrorV1::Protocol);
    }
    let response = match persistence
        .job_status(&request.logical_owner_id, export_id)
        .await
        .map_err(|_| CommunicationsExportClientPortErrorV1::Unavailable)?
    {
        Some(status) => GetEvidenceExportStatusResponseV1 {
            export_id: export_id.to_vec(),
            status: status_wire(status.state)? as i32,
            requested_items: status.requested_items,
            completed_items: status.completed_items,
            artifact_bytes: status
                .artifact
                .map_or(0, |artifact| artifact.declared_bytes),
            error: if status.rejection_code.is_some() {
                CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodePolicyRejected as i32
            } else {
                CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodeUnspecified as i32
            },
        },
        None => GetEvidenceExportStatusResponseV1 {
            export_id: Vec::new(),
            status: EvidenceExportStatusV1::EvidenceExportStatusUnspecified as i32,
            requested_items: 0,
            completed_items: 0,
            artifact_bytes: 0,
            error: CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodeNotFound as i32,
        },
    };
    Ok(response.encode_to_vec())
}

async fn issue_read_ticket(
    persistence: &CommunicationsExportPersistenceV1,
    tickets: &Arc<CommunicationsExportTicketStoreV1>,
    request: &ModuleClientRequestV1,
) -> Result<Vec<u8>, CommunicationsExportClientPortErrorV1> {
    validate_request(request)?;
    let payload = IssueEvidenceExportReadRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| CommunicationsExportClientPortErrorV1::Protocol)?;
    let export_id = id16(&payload.export_id)?;
    if payload.protocol_major != COMMUNICATIONS_EXPORT_CONTRACT_MAJOR_V1 {
        return Err(CommunicationsExportClientPortErrorV1::Protocol);
    }
    let Some(status) = persistence
        .job_status(&request.logical_owner_id, export_id)
        .await
        .map_err(|_| CommunicationsExportClientPortErrorV1::Unavailable)?
    else {
        return Ok(ticket_error(
            CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodeNotFound,
        )
        .encode_to_vec());
    };
    let Some(artifact) = status.artifact else {
        return Ok(ticket_error(
            CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodeNotReady,
        )
        .encode_to_vec());
    };
    let issued = tickets
        .issue(
            &request.logical_owner_id,
            export_id,
            artifact,
            now_unix_seconds()?,
        )
        .map_err(|_| CommunicationsExportClientPortErrorV1::Unavailable)?;
    Ok(IssueEvidenceExportReadResponseV1 {
        opaque_read_capability: issued.opaque_read_capability.to_vec(),
        declared_bytes: issued.declared_bytes,
        expires_at_unix_seconds: issued.expires_at_unix_seconds,
        error: CommunicationsExportErrorCodeV1::CommunicationsExportErrorCodeUnspecified as i32,
    }
    .encode_to_vec())
}

async fn authorize_blob_read(
    persistence: &CommunicationsExportPersistenceV1,
    tickets: &Arc<CommunicationsExportTicketStoreV1>,
    runtime_generation: u64,
    grant_epoch: u64,
    request: &ModuleClientRequestV1,
) -> Result<Vec<u8>, CommunicationsExportClientPortErrorV1> {
    validate_request(request)?;
    let payload = EvidenceExportArtifactReadRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| CommunicationsExportClientPortErrorV1::Protocol)?;
    let capability: [u8; COMMUNICATIONS_EXPORT_READ_TICKET_BYTES_V1] = payload
        .opaque_read_capability
        .try_into()
        .map_err(|_| CommunicationsExportClientPortErrorV1::Protocol)?;
    let Some(consumed) = tickets
        .consume(
            capability,
            &request.logical_owner_id,
            runtime_generation,
            grant_epoch,
            now_unix_seconds()?,
        )
        .map_err(|_| CommunicationsExportClientPortErrorV1::Unavailable)?
    else {
        return Err(CommunicationsExportClientPortErrorV1::NotFound);
    };
    let current = persistence
        .job_status(&request.logical_owner_id, consumed.export_id)
        .await
        .map_err(|_| CommunicationsExportClientPortErrorV1::Unavailable)?
        .and_then(|status| status.artifact);
    if current != Some(consumed.artifact) {
        return Err(CommunicationsExportClientPortErrorV1::NotFound);
    }
    Ok(ModuleClientBlobAuthorizationV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        reference_id: consumed.artifact.reference_id.to_vec(),
        declared_size: consumed.artifact.declared_bytes,
        expected_plaintext_sha256: consumed.artifact.sha256.to_vec(),
        backup_class: 1,
    }
    .encode_to_vec())
}

fn validate_request(
    request: &ModuleClientRequestV1,
) -> Result<(), CommunicationsExportClientPortErrorV1> {
    if request.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || request.module_id != COMMUNICATIONS_EXPORT_MODULE_ID_V1
        || request.owner_id != COMMUNICATIONS_EXPORT_OWNER_V1
        || request.request_id == 0
        || request.request_payload.is_empty()
        || request.logical_owner_id.is_empty()
        || request.logical_owner_id.len() > 128
        || !request.logical_owner_id.is_ascii()
    {
        return Err(CommunicationsExportClientPortErrorV1::Protocol);
    }
    Ok(())
}

fn bounded_message_ids(
    values: &[Vec<u8>],
) -> Result<Vec<[u8; 16]>, CommunicationsExportClientPortErrorV1> {
    if values.is_empty() || values.len() > COMMUNICATIONS_EXPORT_MAX_MESSAGES_V1 {
        return Err(CommunicationsExportClientPortErrorV1::Protocol);
    }
    let ids = values
        .iter()
        .map(|value| id16(value))
        .collect::<Result<Vec<_>, _>>()?;
    if ids
        .iter()
        .enumerate()
        .any(|(index, value)| ids[..index].contains(value))
    {
        return Err(CommunicationsExportClientPortErrorV1::Protocol);
    }
    Ok(ids)
}

fn status_wire(state: u8) -> Result<EvidenceExportStatusV1, CommunicationsExportClientPortErrorV1> {
    match state {
        1 => Ok(EvidenceExportStatusV1::EvidenceExportStatusPendingSource),
        2 => Ok(EvidenceExportStatusV1::EvidenceExportStatusMaterializing),
        3 => Ok(EvidenceExportStatusV1::EvidenceExportStatusReady),
        4 => Ok(EvidenceExportStatusV1::EvidenceExportStatusRejected),
        _ => Err(CommunicationsExportClientPortErrorV1::Unavailable),
    }
}

fn ticket_error(error: CommunicationsExportErrorCodeV1) -> IssueEvidenceExportReadResponseV1 {
    IssueEvidenceExportReadResponseV1 {
        opaque_read_capability: Vec::new(),
        declared_bytes: 0,
        expires_at_unix_seconds: 0,
        error: error as i32,
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsExportClientPortErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(CommunicationsExportClientPortErrorV1::Protocol)
}

fn now_unix_seconds() -> Result<i64, CommunicationsExportClientPortErrorV1> {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CommunicationsExportClientPortErrorV1::Unavailable)?
            .as_secs(),
    )
    .map_err(|_| CommunicationsExportClientPortErrorV1::Unavailable)
}

fn module_response(request_id: u64, response_payload: Vec<u8>) -> ModuleClientResponseV1 {
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload,
        error_code: String::new(),
    }
}

fn module_error(request_id: u64, error_code: &str) -> ModuleClientResponseV1 {
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload: Vec::new(),
        error_code: error_code.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_exact_unique_and_bounded() {
        assert!(bounded_message_ids(&[]).is_err());
        assert!(bounded_message_ids(&[vec![1; 16], vec![2; 16]]).is_ok());
        assert!(bounded_message_ids(&[vec![1; 16], vec![1; 16]]).is_err());
    }
}
