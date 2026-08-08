//! Ciphertext-only relay from Kernel Event Hub to the managed authority child.

use makosh_runtime_protocol::{
    v1::{
        EventsAuthorityRuntimeControlRequestV1, EventsAuthorityRuntimeControlResponseV1,
        EventsRuntimeCredentialDeliveryV1, IssueEventsRuntimeCredentialRequestV1,
        events_authority_runtime_control_request_v1::Operation as RequestOperation,
        events_authority_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::events_authority::{
        validate_credential_request, validate_events_authority_runtime_control_response,
    },
};
use prost::Message;

use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelay;

pub(crate) const EVENTS_AUTHORITY_REGISTRATION_ID: &str = "events_authority";

/// Uses the one bounded managed-child relay; Kernel never decodes credential plaintext.
pub(crate) struct EventAuthorityCredentialRelayV1<R> {
    authority_registration_id: String,
    relay: R,
}

impl<R> EventAuthorityCredentialRelayV1<R>
where
    R: ManagedRuntimeRelay,
{
    pub(crate) fn new(authority_registration_id: String, relay: R) -> Result<Self, String> {
        valid_id(&authority_registration_id)
            .then_some(Self {
                authority_registration_id,
                relay,
            })
            .ok_or_else(|| "Events authority registration is invalid".to_owned())
    }

    pub(crate) fn issue(
        &self,
        request: IssueEventsRuntimeCredentialRequestV1,
    ) -> Result<EventsRuntimeCredentialDeliveryV1, EventAuthorityCredentialRelayErrorV1> {
        validate_credential_request(&request)
            .map_err(|_| EventAuthorityCredentialRelayErrorV1::Rejected)?;
        let payload = EventsAuthorityRuntimeControlRequestV1 {
            operation: Some(RequestOperation::IssueRuntimeCredential(request)),
        }
        .encode_to_vec();
        let response = self
            .relay
            .relay(&self.authority_registration_id, payload)
            .map_err(|_| EventAuthorityCredentialRelayErrorV1::Unavailable)?;
        decode_delivery(&response)
    }
}

fn decode_delivery(
    bytes: &[u8],
) -> Result<EventsRuntimeCredentialDeliveryV1, EventAuthorityCredentialRelayErrorV1> {
    let response = EventsAuthorityRuntimeControlResponseV1::decode(bytes)
        .map_err(|_| EventAuthorityCredentialRelayErrorV1::Rejected)?;
    validate_events_authority_runtime_control_response(&response)
        .map_err(|_| EventAuthorityCredentialRelayErrorV1::Rejected)?;
    match response.result {
        Some(ResponseResult::CredentialDelivery(delivery)) if response.error_code.is_empty() => {
            Ok(delivery)
        }
        _ => Err(EventAuthorityCredentialRelayErrorV1::Rejected),
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EventAuthorityCredentialRelayErrorV1 {
    Rejected,
    Unavailable,
}
