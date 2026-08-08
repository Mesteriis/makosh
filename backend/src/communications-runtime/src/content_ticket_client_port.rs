//! Generated client_rpc adapter for issuing canonical message body tickets.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use makosh_communications_api::CommunicationMessageIdV1;
use makosh_communications_content_api::{
    CONTENT_CONTRACT_MAJOR_V1, IssueMessageBodyReadRequestV1, IssueMessageBodyReadResponseV1,
};
use makosh_communications_persistence::CommunicationsDurablePersistence;
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;

use crate::admission::communications_content_ticket_contract_reference_v1;
use crate::content_ticket_store::{
    CommunicationsContentTicketStoreErrorV1, CommunicationsContentTicketStoreV1,
};

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;
const MODULE_ID: &str = crate::admission::COMMUNICATIONS_MODULE_ID;
const OWNER_ID: &str = crate::admission::COMMUNICATIONS_OWNER_ID;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommunicationsContentTicketClientPortErrorV1 {
    Protocol,
    Unavailable,
}

pub async fn handle_module_content_ticket_request_v1(
    persistence: &CommunicationsDurablePersistence,
    tickets: &Arc<CommunicationsContentTicketStoreV1>,
    bytes: &[u8],
) -> Result<Vec<u8>, CommunicationsContentTicketClientPortErrorV1> {
    let request = decode_module_request(
        bytes,
        &communications_content_ticket_contract_reference_v1(),
    )?;
    let payload = IssueMessageBodyReadRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| CommunicationsContentTicketClientPortErrorV1::Protocol)?;
    let message_id: [u8; 16] = payload
        .message_id
        .try_into()
        .map_err(|_| CommunicationsContentTicketClientPortErrorV1::Protocol)?;
    if payload.protocol_major != CONTENT_CONTRACT_MAJOR_V1
        || message_id.iter().all(|byte| *byte == 0)
    {
        return Err(CommunicationsContentTicketClientPortErrorV1::Protocol);
    }
    let Some(receipt) = persistence
        .current_message_body_content_receipt(CommunicationMessageIdV1::new(message_id))
        .await
        .map_err(|_| CommunicationsContentTicketClientPortErrorV1::Unavailable)?
    else {
        return Ok(module_response(
            request.request_id,
            IssueMessageBodyReadResponseV1 {
                opaque_read_capability: Vec::new(),
                declared_bytes: 0,
                expires_at_unix_seconds: 0,
                media_type: String::new(),
                error_code: "NOT_FOUND".to_owned(),
            }
            .encode_to_vec(),
        ));
    };
    let issued = tickets
        .issue(
            &request.logical_owner_id,
            CommunicationMessageIdV1::new(message_id),
            receipt,
            now_unix_seconds()?,
        )
        .map_err(map_ticket_error)?;
    Ok(module_response(
        request.request_id,
        IssueMessageBodyReadResponseV1 {
            opaque_read_capability: issued.opaque_read_capability.to_vec(),
            declared_bytes: issued.declared_bytes,
            expires_at_unix_seconds: issued.expires_at_unix_seconds,
            media_type: issued.media_type,
            error_code: String::new(),
        }
        .encode_to_vec(),
    ))
}

fn decode_module_request(
    bytes: &[u8],
    contract: &ContractReferenceV1,
) -> Result<ModuleClientRequestV1, CommunicationsContentTicketClientPortErrorV1> {
    let request = ModuleClientRequestV1::decode(bytes)
        .map_err(|_| CommunicationsContentTicketClientPortErrorV1::Protocol)?;
    if request.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || request.module_id != MODULE_ID
        || request.owner_id != OWNER_ID
        || request.contract.as_ref() != Some(contract)
        || request.request_id == 0
        || request.request_payload.is_empty()
        || request.logical_owner_id.is_empty()
    {
        return Err(CommunicationsContentTicketClientPortErrorV1::Protocol);
    }
    Ok(request)
}

fn module_response(request_id: u64, response_payload: Vec<u8>) -> Vec<u8> {
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload,
        error_code: String::new(),
    }
    .encode_to_vec()
}

fn now_unix_seconds() -> Result<i64, CommunicationsContentTicketClientPortErrorV1> {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CommunicationsContentTicketClientPortErrorV1::Unavailable)?
            .as_secs(),
    )
    .map_err(|_| CommunicationsContentTicketClientPortErrorV1::Unavailable)
}

const fn map_ticket_error(
    error: CommunicationsContentTicketStoreErrorV1,
) -> CommunicationsContentTicketClientPortErrorV1 {
    match error {
        CommunicationsContentTicketStoreErrorV1::InvalidRequest => {
            CommunicationsContentTicketClientPortErrorV1::Protocol
        }
        CommunicationsContentTicketStoreErrorV1::Unavailable => {
            CommunicationsContentTicketClientPortErrorV1::Unavailable
        }
    }
}
