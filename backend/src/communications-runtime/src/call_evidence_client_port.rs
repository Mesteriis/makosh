//! Exact module-query and client-RPC transport adapter for call evidence reads.

use makosh_communications_call_evidence_persistence::CommunicationsCallEvidencePersistenceV1;
use makosh_runtime_protocol::{
    v1::{
        ManagedRuntimeModuleQueryDeliveryV1, ManagedRuntimeModuleQueryResponseV1,
        ModuleClientRequestV1, ModuleClientResponseV1,
    },
    validation::{
        module_client::validate_module_client_request_v1,
        module_query::validate_module_query_delivery_v1,
    },
};
use prost::Message;

use crate::{
    admission::communications_call_evidence_query_contract_reference_v1,
    call_evidence_query_port::{CallEvidenceQueryPortErrorV1, handle_call_evidence_query_v1},
};

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallEvidenceClientPortErrorV1 {
    Protocol,
    Unavailable,
}

pub async fn handle_call_evidence_client_request_v1(
    persistence: &CommunicationsCallEvidencePersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, CallEvidenceClientPortErrorV1> {
    let request = ModuleClientRequestV1::decode(bytes)
        .map_err(|_| CallEvidenceClientPortErrorV1::Protocol)?;
    if validate_module_client_request_v1(&request).is_err()
        || request.contract.as_ref()
            != Some(&communications_call_evidence_query_contract_reference_v1())
    {
        return Err(CallEvidenceClientPortErrorV1::Protocol);
    }
    let response_payload =
        handle_call_evidence_query_v1(persistence, logical_owner_id, &request.request_payload)
            .await
            .map_err(map_query_error)?;
    Ok(ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id: request.request_id,
        response_payload,
        error_code: String::new(),
    }
    .encode_to_vec())
}

pub async fn handle_call_evidence_module_query_delivery_v1(
    persistence: &CommunicationsCallEvidencePersistenceV1,
    logical_owner_id: &str,
    delivery: ManagedRuntimeModuleQueryDeliveryV1,
) -> ManagedRuntimeModuleQueryResponseV1 {
    let request_id = delivery.request_id.clone();
    if validate_module_query_delivery_v1(&delivery).is_err()
        || delivery.contract.as_ref()
            != Some(&communications_call_evidence_query_contract_reference_v1())
    {
        return module_query_error(request_id, "REJECTED");
    }
    match handle_call_evidence_query_v1(persistence, logical_owner_id, &delivery.request_payload)
        .await
    {
        Ok(response_payload) => ManagedRuntimeModuleQueryResponseV1 {
            request_id,
            response_payload,
            error_code: String::new(),
        },
        Err(CallEvidenceQueryPortErrorV1::Protocol) => module_query_error(request_id, "REJECTED"),
        Err(CallEvidenceQueryPortErrorV1::Unavailable) => {
            module_query_error(request_id, "UNAVAILABLE")
        }
    }
}

const fn map_query_error(error: CallEvidenceQueryPortErrorV1) -> CallEvidenceClientPortErrorV1 {
    match error {
        CallEvidenceQueryPortErrorV1::Protocol => CallEvidenceClientPortErrorV1::Protocol,
        CallEvidenceQueryPortErrorV1::Unavailable => CallEvidenceClientPortErrorV1::Unavailable,
    }
}

fn module_query_error(
    request_id: Vec<u8>,
    error_code: &str,
) -> ManagedRuntimeModuleQueryResponseV1 {
    ManagedRuntimeModuleQueryResponseV1 {
        request_id,
        response_payload: Vec::new(),
        error_code: error_code.to_owned(),
    }
}
