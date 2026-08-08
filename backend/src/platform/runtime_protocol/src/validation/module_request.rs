//! Structural validation for capability-routed managed module requests.

use crate::v1::{
    ContractReferenceV1, ManagedRuntimeModuleRequestDeliveryV1,
    ManagedRuntimeModuleRequestRequestV1, ManagedRuntimeModuleRequestResponseV1,
};

pub const MODULE_REQUEST_ID_BYTES_V1: usize = 16;
pub const MODULE_REQUEST_MAX_PAYLOAD_BYTES_V1: usize = 64 * 1024;
pub const MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1: u32 = 30_000;
pub const MODULE_REQUEST_MAX_ERROR_CODE_BYTES_V1: usize = 128;

pub fn validate_module_request_request_v1(
    request: &ManagedRuntimeModuleRequestRequestV1,
) -> Result<(), ModuleRequestValidationErrorV1> {
    if !valid_request_id(&request.request_id)
        || !request
            .contract
            .as_ref()
            .is_some_and(valid_contract_reference)
        || request.request_payload.len() > MODULE_REQUEST_MAX_PAYLOAD_BYTES_V1
        || !(1..=MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1).contains(&request.deadline_millis)
        || (!request.response_blob_capability_id.is_empty()
            && !valid_identifier(&request.response_blob_capability_id))
    {
        return Err(ModuleRequestValidationErrorV1::InvalidRequest);
    }
    Ok(())
}

pub fn validate_module_request_delivery_v1(
    delivery: &ManagedRuntimeModuleRequestDeliveryV1,
) -> Result<(), ModuleRequestValidationErrorV1> {
    if !valid_request_id(&delivery.request_id)
        || !valid_identifier(&delivery.logical_owner_id)
        || !delivery
            .contract
            .as_ref()
            .is_some_and(valid_contract_reference)
        || delivery.request_payload.len() > MODULE_REQUEST_MAX_PAYLOAD_BYTES_V1
        || !valid_response_blob_target(delivery)
    {
        return Err(ModuleRequestValidationErrorV1::InvalidDelivery);
    }
    Ok(())
}

pub fn validate_module_request_response_v1(
    response: &ManagedRuntimeModuleRequestResponseV1,
) -> Result<(), ModuleRequestValidationErrorV1> {
    let successful = response.error_code.is_empty()
        && response.response_payload.len() <= MODULE_REQUEST_MAX_PAYLOAD_BYTES_V1;
    let failed = response.response_payload.is_empty() && valid_error_code(&response.error_code);
    if !valid_request_id(&response.request_id) || (!successful && !failed) {
        return Err(ModuleRequestValidationErrorV1::InvalidResponse);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleRequestValidationErrorV1 {
    InvalidRequest,
    InvalidDelivery,
    InvalidResponse,
}

fn valid_request_id(value: &[u8]) -> bool {
    value.len() == MODULE_REQUEST_ID_BYTES_V1 && value.iter().any(|byte| *byte != 0)
}

fn valid_contract_reference(value: &ContractReferenceV1) -> bool {
    valid_identifier(&value.owner)
        && valid_identifier(&value.name)
        && value.major > 0
        && value.revision > 0
        && value.schema_sha256.len() == 32
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_error_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MODULE_REQUEST_MAX_ERROR_CODE_BYTES_V1
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || matches!(byte, b'_'))
}

fn valid_response_blob_target(delivery: &ManagedRuntimeModuleRequestDeliveryV1) -> bool {
    let empty = delivery.response_blob_target_owner_id.is_empty()
        && delivery.response_blob_target_module_id.is_empty()
        && delivery.response_blob_target_capability_id.is_empty();
    let exact = valid_identifier(&delivery.response_blob_target_owner_id)
        && valid_identifier(&delivery.response_blob_target_module_id)
        && valid_identifier(&delivery.response_blob_target_capability_id);
    empty || exact
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract() -> ContractReferenceV1 {
        ContractReferenceV1 {
            owner: "communication_delivery_intent".to_owned(),
            name: "communication.delivery-intent.submit".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: vec![7; 32],
        }
    }

    #[test]
    fn request_delivery_and_response_are_bounded_and_correlated() {
        let request = ManagedRuntimeModuleRequestRequestV1 {
            request_id: vec![1; MODULE_REQUEST_ID_BYTES_V1],
            contract: Some(contract()),
            request_payload: vec![8],
            deadline_millis: MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1,
            response_blob_capability_id: "communication_delivery_intent.blob.v1".to_owned(),
        };
        assert_eq!(validate_module_request_request_v1(&request), Ok(()));

        let delivery = ManagedRuntimeModuleRequestDeliveryV1 {
            request_id: request.request_id.clone(),
            logical_owner_id: "owner_local".to_owned(),
            contract: request.contract.clone(),
            request_payload: request.request_payload,
            response_blob_target_owner_id: "communication_delivery_intent".to_owned(),
            response_blob_target_module_id: "makosh-communication-delivery-intent-runtime"
                .to_owned(),
            response_blob_target_capability_id: request.response_blob_capability_id,
        };
        assert_eq!(validate_module_request_delivery_v1(&delivery), Ok(()));

        let response = ManagedRuntimeModuleRequestResponseV1 {
            request_id: delivery.request_id,
            response_payload: vec![9],
            error_code: String::new(),
        };
        assert_eq!(validate_module_request_response_v1(&response), Ok(()));
    }

    #[test]
    fn malformed_oversized_or_ambiguous_results_fail_closed() {
        let mut request = ManagedRuntimeModuleRequestRequestV1 {
            request_id: vec![0; MODULE_REQUEST_ID_BYTES_V1],
            contract: Some(contract()),
            request_payload: Vec::new(),
            deadline_millis: 1,
            response_blob_capability_id: String::new(),
        };
        assert_eq!(
            validate_module_request_request_v1(&request),
            Err(ModuleRequestValidationErrorV1::InvalidRequest)
        );
        request.request_id.fill(1);
        request.request_payload = vec![1; MODULE_REQUEST_MAX_PAYLOAD_BYTES_V1 + 1];
        assert!(validate_module_request_request_v1(&request).is_err());

        let response = ManagedRuntimeModuleRequestResponseV1 {
            request_id: vec![1; MODULE_REQUEST_ID_BYTES_V1],
            response_payload: vec![1],
            error_code: "UNAVAILABLE".to_owned(),
        };
        assert_eq!(
            validate_module_request_response_v1(&response),
            Err(ModuleRequestValidationErrorV1::InvalidResponse)
        );
    }
}
