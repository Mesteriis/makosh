//! Capability-routed query adapter from the delivery workflow to Communications.

use std::os::unix::net::UnixStream;

use makosh_communications_api::{
    COMMUNICATIONS_QUERY_SCHEMA_SHA256, CommunicationConversationIdV1,
    CommunicationConversationSummaryV1, CommunicationMessageIdV1, CommunicationMessageSummaryV1,
    CommunicationsQueryProjectionErrorV1,
    query_wire::{
        CommunicationsQueryRequestV1, CommunicationsQueryResponseV1, GetConversationRequestV1,
        GetMessageRequestV1, communications_query_request_v1::Operation,
        communications_query_response_v1::Result as QueryResult,
    },
};
use makosh_runtime_protocol::{
    managed_control::{
        ManagedControlChannelV2, ManagedControlRequestDispatcherV2, ManagedControlTransportErrorV2,
    },
    v1::{
        ContractReferenceV1, ManagedRuntimeControlRequestV1, ManagedRuntimeModuleQueryRequestV1,
        ManagedRuntimeModuleQueryResponseV1, managed_runtime_control_request_v1,
        managed_runtime_control_response_v1,
    },
    validation::module_query::{
        MODULE_QUERY_MAX_DEADLINE_MILLIS_V1, validate_module_query_response_v1,
    },
};
use prost::Message;
use sha2::{Digest, Sha256};

const QUERY_PROTOCOL_MAJOR: u32 = 1;
const QUERY_DEADLINE_MILLIS: u32 = MODULE_QUERY_MAX_DEADLINE_MILLIS_V1;

#[derive(Debug)]
pub enum CommunicationsQueryClientErrorV1 {
    Protocol,
    Unavailable,
}

impl From<ManagedControlTransportErrorV2> for CommunicationsQueryClientErrorV1 {
    fn from(_: ManagedControlTransportErrorV2) -> Self {
        Self::Unavailable
    }
}

impl From<CommunicationsQueryProjectionErrorV1> for CommunicationsQueryClientErrorV1 {
    fn from(_: CommunicationsQueryProjectionErrorV1) -> Self {
        Self::Protocol
    }
}

pub struct ManagedCommunicationsQueryClientV1<'a> {
    pub control_channel: &'a mut ManagedControlChannelV2<UnixStream>,
    pub dispatcher: &'a mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
}

impl ManagedCommunicationsQueryClientV1<'_> {
    pub fn resolve_route_sources(
        &mut self,
        operation_id: [u8; 16],
        conversation_id: CommunicationConversationIdV1,
        reply_to_message_id: Option<CommunicationMessageIdV1>,
    ) -> Result<
        (
            CommunicationConversationSummaryV1,
            Option<CommunicationMessageSummaryV1>,
        ),
        CommunicationsQueryClientErrorV1,
    > {
        let conversation = self.get_conversation(
            derived_request_id(operation_id, b"conversation"),
            conversation_id,
        )?;
        let reply = reply_to_message_id
            .map(|message_id| {
                self.get_message(derived_request_id(operation_id, b"reply"), message_id)
            })
            .transpose()?;
        Ok((conversation, reply))
    }

    pub fn get_conversation(
        &mut self,
        request_id: [u8; 16],
        conversation_id: CommunicationConversationIdV1,
    ) -> Result<CommunicationConversationSummaryV1, CommunicationsQueryClientErrorV1> {
        let response = self.query(
            request_id,
            Operation::GetConversation(GetConversationRequestV1 {
                conversation_id: conversation_id.bytes().to_vec(),
            }),
        )?;
        let QueryResult::GetConversation(response) = response
            .result
            .ok_or(CommunicationsQueryClientErrorV1::Protocol)?
        else {
            return Err(CommunicationsQueryClientErrorV1::Protocol);
        };
        response
            .conversation
            .ok_or(CommunicationsQueryClientErrorV1::Protocol)?
            .try_into()
            .map_err(Into::into)
    }

    pub fn get_message(
        &mut self,
        request_id: [u8; 16],
        message_id: CommunicationMessageIdV1,
    ) -> Result<CommunicationMessageSummaryV1, CommunicationsQueryClientErrorV1> {
        let response = self.query(
            request_id,
            Operation::GetMessage(GetMessageRequestV1 {
                message_id: message_id.bytes().to_vec(),
            }),
        )?;
        let QueryResult::GetMessage(response) = response
            .result
            .ok_or(CommunicationsQueryClientErrorV1::Protocol)?
        else {
            return Err(CommunicationsQueryClientErrorV1::Protocol);
        };
        response
            .message
            .ok_or(CommunicationsQueryClientErrorV1::Protocol)?
            .try_into()
            .map_err(Into::into)
    }

    fn query(
        &mut self,
        request_id: [u8; 16],
        operation: Operation,
    ) -> Result<CommunicationsQueryResponseV1, CommunicationsQueryClientErrorV1> {
        let query = CommunicationsQueryRequestV1 {
            protocol_major: QUERY_PROTOCOL_MAJOR,
            operation: Some(operation),
        };
        let request = module_query_request(request_id, query.encode_to_vec());
        let response = self.control_channel.request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(
                    managed_runtime_control_request_v1::Operation::RouteModuleQuery(request),
                ),
            },
            self.dispatcher,
        )?;
        if !response.error_code.is_empty() {
            developer_log_route_error(&response.error_code);
            return Err(CommunicationsQueryClientErrorV1::Unavailable);
        }
        let Some(managed_runtime_control_response_v1::Result::ModuleQueryRoute(response)) =
            response.result
        else {
            return Err(CommunicationsQueryClientErrorV1::Protocol);
        };
        decode_response(request_id, response)
    }
}

fn developer_log_route_error(error_code: &str) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_delivery_intent_query_route_error={error_code}");
    }
}

fn module_query_request(
    request_id: [u8; 16],
    request_payload: Vec<u8>,
) -> ManagedRuntimeModuleQueryRequestV1 {
    ManagedRuntimeModuleQueryRequestV1 {
        request_id: request_id.to_vec(),
        contract: Some(communications_query_contract()),
        request_payload,
        deadline_millis: QUERY_DEADLINE_MILLIS,
    }
}

fn communications_query_contract() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: "communications".to_owned(),
        name: "communications.query".to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: COMMUNICATIONS_QUERY_SCHEMA_SHA256.to_vec(),
    }
}

fn derived_request_id(operation_id: [u8; 16], purpose: &[u8]) -> [u8; 16] {
    let digest = Sha256::new()
        .chain_update(b"makosh.communication-delivery-intent.query-request.v1\0")
        .chain_update(operation_id)
        .chain_update(purpose)
        .finalize();
    let mut request_id = [0_u8; 16];
    request_id.copy_from_slice(&digest[..16]);
    request_id
}

fn decode_response(
    request_id: [u8; 16],
    response: ManagedRuntimeModuleQueryResponseV1,
) -> Result<CommunicationsQueryResponseV1, CommunicationsQueryClientErrorV1> {
    validate_module_query_response_v1(&response)
        .map_err(|_| CommunicationsQueryClientErrorV1::Protocol)?;
    if response.request_id != request_id || !response.error_code.is_empty() {
        return Err(CommunicationsQueryClientErrorV1::Unavailable);
    }
    let response = CommunicationsQueryResponseV1::decode(response.response_payload.as_slice())
        .map_err(|_| CommunicationsQueryClientErrorV1::Protocol)?;
    if !response.error_code.is_empty() || response.result.is_none() {
        return Err(CommunicationsQueryClientErrorV1::Unavailable);
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_request_uses_the_exact_public_contract_and_bound() {
        let request = module_query_request([1; 16], vec![9]);
        assert_eq!(request.request_id, vec![1; 16]);
        assert_eq!(request.contract, Some(communications_query_contract()));
        assert_eq!(request.deadline_millis, MODULE_QUERY_MAX_DEADLINE_MILLIS_V1);
    }

    #[test]
    fn per_operation_query_ids_are_stable_distinct_and_non_zero() {
        let conversation = derived_request_id([1; 16], b"conversation");
        let reply = derived_request_id([1; 16], b"reply");
        assert_ne!(conversation, reply);
        assert!(conversation.iter().any(|byte| *byte != 0));
        assert_eq!(conversation, derived_request_id([1; 16], b"conversation"));
    }

    #[test]
    fn response_decode_rejects_correlation_and_error_ambiguity() {
        let query_response = CommunicationsQueryResponseV1 {
            result: Some(QueryResult::GetConversation(
                makosh_communications_api::query_wire::GetConversationResponseV1 {
                    conversation: Some(
                        (&CommunicationConversationSummaryV1 {
                            conversation_id: CommunicationConversationIdV1::new([3; 16]),
                            account_cursor: makosh_communications_api::CommunicationSourceCursorV1::new(
                                [4; 32],
                            ),
                            conversation_cursor:
                                makosh_communications_api::CommunicationSourceCursorV1::new([5; 32]),
                            provider:
                                makosh_communications_api::CommunicationProviderProvenanceV1::Telegram,
                            first_observed_at_unix_seconds: 1,
                            last_observed_at_unix_seconds: 2,
                            last_evidence_id:
                                makosh_communications_api::CommunicationObservationIdV1::new([6; 16]),
                        })
                            .into(),
                    ),
                },
            )),
            error_code: String::new(),
        };
        let response = ManagedRuntimeModuleQueryResponseV1 {
            request_id: vec![1; 16],
            response_payload: query_response.encode_to_vec(),
            error_code: String::new(),
        };
        assert!(decode_response([1; 16], response.clone()).is_ok());
        assert!(matches!(
            decode_response([2; 16], response),
            Err(CommunicationsQueryClientErrorV1::Unavailable)
        ));
    }
}
