//! Structural validation for opaque Core-to-module client routing envelopes.

use crate::v1::{ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1};

const PROTOCOL_MAJOR: u32 = 1;
const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_PAYLOAD_BYTES: usize = 1024 * 1024;
const MAX_ERROR_CODE_BYTES: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleClientValidationErrorV1 {
    InvalidRequest,
    InvalidResponse,
}

pub fn validate_module_client_request_v1(
    request: &ModuleClientRequestV1,
) -> Result<(), ModuleClientValidationErrorV1> {
    if request.protocol_major != PROTOCOL_MAJOR
        || !valid_identifier(&request.module_id)
        || !valid_identifier(&request.owner_id)
        || (!request.logical_owner_id.is_empty() && !valid_identifier(&request.logical_owner_id))
        || (!request.authenticated_device_id.is_empty()
            && !valid_identifier(&request.authenticated_device_id))
        || (!request.authenticated_client_session_id.is_empty()
            && !valid_identifier(&request.authenticated_client_session_id))
        || request.logical_owner_id.is_empty() != request.authenticated_device_id.is_empty()
        || request.logical_owner_id.is_empty() != request.authenticated_client_session_id.is_empty()
        || request.request_id == 0
        || request.request_payload.len() > MAX_PAYLOAD_BYTES
        || !request.contract.as_ref().is_some_and(valid_contract)
    {
        return Err(ModuleClientValidationErrorV1::InvalidRequest);
    }
    Ok(())
}

pub fn validate_module_client_response_v1(
    response: &ModuleClientResponseV1,
) -> Result<(), ModuleClientValidationErrorV1> {
    let successful_response =
        response.response_payload.len() <= MAX_PAYLOAD_BYTES && response.error_code.is_empty();
    let failed_response =
        response.response_payload.is_empty() && valid_error_code(&response.error_code);
    if response.protocol_major != PROTOCOL_MAJOR
        || response.request_id == 0
        || (!successful_response && !failed_response)
    {
        return Err(ModuleClientValidationErrorV1::InvalidResponse);
    }
    Ok(())
}

fn valid_contract(contract: &ContractReferenceV1) -> bool {
    valid_identifier(&contract.owner)
        && valid_identifier(&contract.name)
        && contract.major > 0
        && contract.revision > 0
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_error_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ERROR_CODE_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract() -> ContractReferenceV1 {
        ContractReferenceV1 {
            owner: "mail".to_owned(),
            name: "operational".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: vec![1; 32],
        }
    }

    #[test]
    fn accepts_a_bounded_opaque_client_request() {
        let request = ModuleClientRequestV1 {
            protocol_major: 1,
            module_id: "mail".to_owned(),
            owner_id: "mail".to_owned(),
            contract: Some(contract()),
            request_id: 1,
            request_payload: vec![1],
            logical_owner_id: String::new(),
            authenticated_device_id: String::new(),
            authenticated_client_session_id: String::new(),
        };
        assert_eq!(validate_module_client_request_v1(&request), Ok(()));
    }

    #[test]
    fn accepts_an_empty_protobuf_request_payload() {
        let request = ModuleClientRequestV1 {
            protocol_major: 1,
            module_id: "makosh-mail-runtime".to_owned(),
            owner_id: "integration.mail".to_owned(),
            logical_owner_id: "owner-1".to_owned(),
            authenticated_device_id: "device-1".to_owned(),
            authenticated_client_session_id: "session-1".to_owned(),
            request_id: 1,
            contract: Some(contract()),
            request_payload: Vec::new(),
        };
        assert_eq!(validate_module_client_request_v1(&request), Ok(()));
    }

    #[test]
    fn authenticated_owner_device_and_session_context_are_atomic() {
        let mut request = ModuleClientRequestV1 {
            protocol_major: 1,
            module_id: "makosh-mail-runtime".to_owned(),
            owner_id: "integration.mail".to_owned(),
            logical_owner_id: "owner-1".to_owned(),
            authenticated_device_id: String::new(),
            authenticated_client_session_id: String::new(),
            request_id: 1,
            contract: Some(contract()),
            request_payload: Vec::new(),
        };
        assert_eq!(
            validate_module_client_request_v1(&request),
            Err(ModuleClientValidationErrorV1::InvalidRequest)
        );
        request.logical_owner_id.clear();
        request.authenticated_device_id = "device-1".to_owned();
        assert_eq!(
            validate_module_client_request_v1(&request),
            Err(ModuleClientValidationErrorV1::InvalidRequest)
        );
        request.authenticated_device_id.clear();
        request.authenticated_client_session_id = "session-1".to_owned();
        assert_eq!(
            validate_module_client_request_v1(&request),
            Err(ModuleClientValidationErrorV1::InvalidRequest)
        );
    }

    #[test]
    fn rejects_a_response_with_payload_and_error() {
        let response = ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: 1,
            response_payload: vec![1],
            error_code: "REJECTED".to_owned(),
        };
        assert_eq!(
            validate_module_client_response_v1(&response),
            Err(ModuleClientValidationErrorV1::InvalidResponse)
        );
    }

    #[test]
    fn accepts_an_empty_successful_protobuf_payload() {
        let response = ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: 1,
            response_payload: Vec::new(),
            error_code: String::new(),
        };
        assert_eq!(validate_module_client_response_v1(&response), Ok(()));
    }
}
