//! Generated-contract client dispatcher for delivery intent submit and status.

use std::os::unix::net::UnixStream;

use makosh_communication_delivery_intent_api::{
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1, COMMUNICATION_DELIVERY_INTENT_MODULE_ID_V1,
    COMMUNICATION_DELIVERY_INTENT_OWNER_V1,
    wire::{
        DeliveryIntentErrorCodeV1, DeliveryIntentStatusV1, GetDeliveryIntentStatusRequestV1,
        GetDeliveryIntentStatusResponseV1,
    },
};
use makosh_communication_delivery_intent_persistence::DeliveryIntentStatusRecordV1;
use makosh_runtime_protocol::{
    managed_control::ManagedControlRequestDispatcherV2,
    v1::{ModuleClientRequestV1, ModuleClientResponseV1},
};
use prost::Message;

use crate::{
    client_status::{rejection_value, status_value},
    contracts::{command_contract_v1, query_contract_v1},
    runtime::{DeliveryIntentManagedRuntimeV1, DeliveryIntentRuntimeErrorV1},
    submit_port::submit_delivery_intent_payload_v1,
};

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

pub async fn dispatch_delivery_intent_client_request_v1(
    runtime: &mut DeliveryIntentManagedRuntimeV1,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: &ModuleClientRequestV1,
    now_unix_seconds: i64,
) -> ModuleClientResponseV1 {
    if request.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || request.module_id != COMMUNICATION_DELIVERY_INTENT_MODULE_ID_V1
        || request.owner_id != COMMUNICATION_DELIVERY_INTENT_OWNER_V1
    {
        return module_error(request.request_id, "REJECTED");
    }
    let response_payload = if request.contract.as_ref() == Some(&command_contract_v1()) {
        submit_delivery_intent_payload_v1(
            runtime,
            dispatcher,
            &request.request_payload,
            now_unix_seconds,
        )
        .await
    } else if request.contract.as_ref() == Some(&query_contract_v1()) {
        status(runtime, &request.request_payload).await
    } else {
        return module_error(request.request_id, "REJECTED");
    };
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id: request.request_id,
        response_payload,
        error_code: String::new(),
    }
}

async fn status(runtime: &DeliveryIntentManagedRuntimeV1, bytes: &[u8]) -> Vec<u8> {
    let Ok(request) = GetDeliveryIntentStatusRequestV1::decode(bytes) else {
        return status_error(
            Vec::new(),
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeInvalidRequest,
        );
    };
    let response_intent_id = request.intent_id.clone();
    let Ok(intent_id) = id16(&request.intent_id) else {
        return status_error(
            response_intent_id,
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeInvalidRequest,
        );
    };
    if request.protocol_major != COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1 {
        return status_error(
            response_intent_id,
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeInvalidRequest,
        );
    }
    let status = match runtime.delivery_intent_status_v1(intent_id).await {
        Ok(status) => status,
        Err(error) => return status_error(response_intent_id, runtime_error(error)),
    };
    let Some(status) = status else {
        return status_error(
            response_intent_id,
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeCanonicalSourceNotFound,
        );
    };
    status_response(status)
}

fn status_response(status: DeliveryIntentStatusRecordV1) -> Vec<u8> {
    GetDeliveryIntentStatusResponseV1 {
        intent_id: status.intent_id.to_vec(),
        status: status_value(status.state),
        provider_operation_id: status.provider_operation_id,
        error: rejection_value(status.rejection_code),
    }
    .encode_to_vec()
}

fn status_error(intent_id: Vec<u8>, error: DeliveryIntentErrorCodeV1) -> Vec<u8> {
    GetDeliveryIntentStatusResponseV1 {
        intent_id,
        status: DeliveryIntentStatusV1::DeliveryIntentStatusUnspecified as i32,
        provider_operation_id: None,
        error: error as i32,
    }
    .encode_to_vec()
}

const fn runtime_error(error: DeliveryIntentRuntimeErrorV1) -> DeliveryIntentErrorCodeV1 {
    match error {
        DeliveryIntentRuntimeErrorV1::InvalidRequest => {
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeInvalidRequest
        }
        DeliveryIntentRuntimeErrorV1::RouteUnavailable => {
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeRouteUnavailable
        }
        DeliveryIntentRuntimeErrorV1::Coordinator(_) => {
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodePolicyRejected
        }
        DeliveryIntentRuntimeErrorV1::Admission
        | DeliveryIntentRuntimeErrorV1::Persistence(_)
        | DeliveryIntentRuntimeErrorV1::EventContract
        | DeliveryIntentRuntimeErrorV1::Unavailable => {
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeUnavailable
        }
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], &'static str> {
    let value: [u8; 16] = value.try_into().map_err(|_| "INVALID_REQUEST")?;
    if value.iter().all(|byte| *byte == 0) {
        return Err("INVALID_REQUEST");
    }
    Ok(value)
}

fn module_error(request_id: u64, error_code: &str) -> ModuleClientResponseV1 {
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload: Vec::new(),
        error_code: error_code.to_owned(),
    }
}
