//! Exact managed-module transport adapter for the Communications query contract.

use makosh_communications_api::COMMUNICATIONS_QUERY_SCHEMA_SHA256;
use makosh_communications_persistence::CommunicationsDurablePersistence;
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;
use std::os::unix::net::UnixStream;

use crate::query_port::{CommunicationsQueryPortErrorV1, handle_query_request_v1};
use crate::search_access::CommunicationsSearchAccessV1;
use makosh_communications_ingress::COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID;

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;
pub const COMMUNICATIONS_MODULE_ID: &str = COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID;
const MODULE_ID: &str = COMMUNICATIONS_MODULE_ID;
const OWNER_ID: &str = "communications";
const CONTRACT_NAME: &str = "communications.query";
const CONTRACT_MAJOR: u32 = 1;
const CONTRACT_REVISION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsQueryClientPortErrorV1 {
    Protocol,
    Unavailable,
}

pub fn encode_module_query_request_v1(
    request_id: u64,
    query_payload: &[u8],
) -> Result<Vec<u8>, CommunicationsQueryClientPortErrorV1> {
    if request_id == 0 || query_payload.is_empty() {
        return Err(CommunicationsQueryClientPortErrorV1::Protocol);
    }
    Ok(ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: MODULE_ID.to_owned(),
        owner_id: OWNER_ID.to_owned(),
        contract: Some(query_contract()),
        request_id,
        request_payload: query_payload.to_vec(),
        logical_owner_id: String::new(),
        authenticated_device_id: String::new(),
        authenticated_client_session_id: String::new(),
    }
    .encode_to_vec())
}

pub fn decode_module_query_request_v1(
    bytes: &[u8],
) -> Result<(u64, Vec<u8>), CommunicationsQueryClientPortErrorV1> {
    let envelope = ModuleClientRequestV1::decode(bytes)
        .map_err(|_| CommunicationsQueryClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != MODULE_ID
        || envelope.owner_id != OWNER_ID
        || envelope.contract.as_ref() != Some(&query_contract())
        || envelope.request_id == 0
        || envelope.request_payload.is_empty()
    {
        return Err(CommunicationsQueryClientPortErrorV1::Protocol);
    }
    Ok((envelope.request_id, envelope.request_payload))
}

pub async fn handle_module_query_request_v1(
    persistence: &CommunicationsDurablePersistence,
    search_access: &mut CommunicationsSearchAccessV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    bytes: &[u8],
) -> Result<Vec<u8>, CommunicationsQueryClientPortErrorV1> {
    let (request_id, query_payload) = decode_module_query_request_v1(bytes)?;
    let response_payload = handle_query_request_v1(
        persistence,
        search_access,
        control_channel,
        dispatcher,
        &query_payload,
    )
    .await
    .map_err(map_query_error)?;
    Ok(ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload,
        error_code: String::new(),
    }
    .encode_to_vec())
}

fn query_contract() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: OWNER_ID.to_owned(),
        name: CONTRACT_NAME.to_owned(),
        major: CONTRACT_MAJOR,
        revision: CONTRACT_REVISION,
        schema_sha256: COMMUNICATIONS_QUERY_SCHEMA_SHA256.to_vec(),
    }
}

const fn map_query_error(
    error: CommunicationsQueryPortErrorV1,
) -> CommunicationsQueryClientPortErrorV1 {
    match error {
        CommunicationsQueryPortErrorV1::Protocol => CommunicationsQueryClientPortErrorV1::Protocol,
        CommunicationsQueryPortErrorV1::Unavailable => {
            CommunicationsQueryClientPortErrorV1::Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CommunicationsQueryClientPortErrorV1, decode_module_query_request_v1,
        encode_module_query_request_v1,
    };

    #[test]
    fn accepts_only_the_exact_communications_query_module_contract() {
        let request = encode_module_query_request_v1(7, &[1, 2, 3]).expect("query request");
        assert_eq!(
            decode_module_query_request_v1(&request),
            Ok((7, vec![1, 2, 3])),
        );
        assert_eq!(
            encode_module_query_request_v1(0, &[1]),
            Err(CommunicationsQueryClientPortErrorV1::Protocol),
        );
    }
}
