//! Owner-authorized relay of a signed Account JWT to the managed authority.

use makosh_runtime_protocol::{
    v1::{
        ApplyEventsAccountJwtUpdateRequestV1, EventsAuthorityRuntimeControlRequestV1,
        EventsAuthorityRuntimeControlResponseV1,
        events_authority_runtime_control_request_v1::Operation,
        events_authority_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::events_authority::{
        validate_account_jwt_update, validate_events_authority_runtime_control_response,
    },
};
use prost::Message;

use super::binding::EVENTS_AUTHORITY_PROCESS_ID;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelay;

pub(crate) fn apply<R>(
    relay: &R,
    resolver_credential_revision: u64,
    signed_account_jwt: String,
) -> Result<u64, String>
where
    R: ManagedRuntimeRelay,
{
    let request = ApplyEventsAccountJwtUpdateRequestV1 {
        resolver_credential_revision,
        signed_account_jwt,
    };
    validate_account_jwt_update(&request)
        .map_err(|_| "Events authority Account JWT update is invalid".to_owned())?;
    let payload = EventsAuthorityRuntimeControlRequestV1 {
        operation: Some(Operation::ApplyAccountJwtUpdate(request)),
    }
    .encode_to_vec();
    let response = relay
        .relay(EVENTS_AUTHORITY_PROCESS_ID, payload)
        .map_err(|_| "Events authority runtime is unavailable".to_owned())?;
    parse_response(response.as_slice(), resolver_credential_revision)
}

fn parse_response(bytes: &[u8], expected_revision: u64) -> Result<u64, String> {
    let response = EventsAuthorityRuntimeControlResponseV1::decode(bytes)
        .map_err(|_| "Events authority Account JWT response is invalid".to_owned())?;
    validate_events_authority_runtime_control_response(&response)
        .map_err(|_| "Events authority Account JWT response is invalid".to_owned())?;
    match response.result {
        Some(ResponseResult::AccountJwtUpdated(value))
            if response.error_code.is_empty()
                && value.resolver_credential_revision == expected_revision =>
        {
            Ok(value.resolver_credential_revision)
        }
        _ => Err("Events authority Account JWT update was denied".to_owned()),
    }
}
