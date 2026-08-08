//! Descriptor-declared client_blob authorization for canonical body bytes.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use makosh_communications_content_api::{CONTENT_CONTRACT_MAJOR_V1, ReadMessageBodyRequestV1};
use makosh_communications_persistence::CommunicationsDurablePersistence;
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientBlobAuthorizationV1, ModuleClientRequestV1,
    ModuleClientResponseV1,
};
use prost::Message;

use crate::admission::communications_content_read_contract_reference_v1;
use crate::content_ticket_store::CommunicationsContentTicketStoreV1;

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;
const MODULE_ID: &str = crate::admission::COMMUNICATIONS_MODULE_ID;
const OWNER_ID: &str = crate::admission::COMMUNICATIONS_OWNER_ID;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommunicationsContentBlobClientPortErrorV1 {
    Protocol,
    Unavailable,
}

pub async fn handle_module_content_blob_request_v1(
    persistence: &CommunicationsDurablePersistence,
    tickets: &Arc<CommunicationsContentTicketStoreV1>,
    bytes: &[u8],
) -> Result<Vec<u8>, CommunicationsContentBlobClientPortErrorV1> {
    let request =
        decode_module_request(bytes, &communications_content_read_contract_reference_v1())?;
    let payload = ReadMessageBodyRequestV1::decode(request.request_payload.as_slice())
        .map_err(|_| CommunicationsContentBlobClientPortErrorV1::Protocol)?;
    let capability: [u8; 32] = payload
        .opaque_read_capability
        .try_into()
        .map_err(|_| CommunicationsContentBlobClientPortErrorV1::Protocol)?;
    if payload.protocol_major != CONTENT_CONTRACT_MAJOR_V1
        || capability.iter().all(|byte| *byte == 0)
    {
        return Err(CommunicationsContentBlobClientPortErrorV1::Protocol);
    }
    let Some(consumed) = tickets
        .consume(capability, &request.logical_owner_id, now_unix_seconds()?)
        .map_err(|_| CommunicationsContentBlobClientPortErrorV1::Unavailable)?
    else {
        return Ok(module_error(request.request_id, "NOT_FOUND"));
    };
    let current = persistence
        .current_message_body_content_receipt(consumed.message_id)
        .await
        .map_err(|_| CommunicationsContentBlobClientPortErrorV1::Unavailable)?;
    let Some(current) = current_receipt_if_unchanged(current, &consumed.receipt) else {
        return Ok(module_error(request.request_id, "NOT_FOUND"));
    };
    Ok(module_response(
        request.request_id,
        ModuleClientBlobAuthorizationV1 {
            protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
            reference_id: current.reference_id.to_vec(),
            declared_size: current.declared_bytes,
            expected_plaintext_sha256: current.plaintext_sha256.to_vec(),
            backup_class: current.backup_class,
        }
        .encode_to_vec(),
    ))
}

fn current_receipt_if_unchanged(
    current: Option<makosh_communications_persistence::CommunicationsBodyContentReceiptV1>,
    ticket_receipt: &makosh_communications_persistence::CommunicationsBodyContentReceiptV1,
) -> Option<makosh_communications_persistence::CommunicationsBodyContentReceiptV1> {
    current.filter(|receipt| receipt == ticket_receipt)
}

fn decode_module_request(
    bytes: &[u8],
    contract: &ContractReferenceV1,
) -> Result<ModuleClientRequestV1, CommunicationsContentBlobClientPortErrorV1> {
    let request = ModuleClientRequestV1::decode(bytes)
        .map_err(|_| CommunicationsContentBlobClientPortErrorV1::Protocol)?;
    if request.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || request.module_id != MODULE_ID
        || request.owner_id != OWNER_ID
        || request.contract.as_ref() != Some(contract)
        || request.request_id == 0
        || request.request_payload.is_empty()
        || request.logical_owner_id.is_empty()
    {
        return Err(CommunicationsContentBlobClientPortErrorV1::Protocol);
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

fn module_error(request_id: u64, error_code: &str) -> Vec<u8> {
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload: Vec::new(),
        error_code: error_code.to_owned(),
    }
    .encode_to_vec()
}

fn now_unix_seconds() -> Result<i64, CommunicationsContentBlobClientPortErrorV1> {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CommunicationsContentBlobClientPortErrorV1::Unavailable)?
            .as_secs(),
    )
    .map_err(|_| CommunicationsContentBlobClientPortErrorV1::Unavailable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_communications_persistence::CommunicationsBodyContentReceiptV1;

    #[test]
    fn edit_delete_or_replaced_receipt_invalidates_the_ticket() {
        let receipt = CommunicationsBodyContentReceiptV1 {
            reference_id: [1; 16],
            declared_bytes: 32,
            plaintext_sha256: [2; 32],
            backup_class: 1,
            media_type: "text/plain".to_owned(),
        };
        assert_eq!(
            current_receipt_if_unchanged(Some(receipt.clone()), &receipt),
            Some(receipt.clone())
        );
        assert_eq!(current_receipt_if_unchanged(None, &receipt), None);
        assert_eq!(
            current_receipt_if_unchanged(
                Some(CommunicationsBodyContentReceiptV1 {
                    reference_id: [3; 16],
                    ..receipt.clone()
                }),
                &receipt,
            ),
            None
        );
    }
}
