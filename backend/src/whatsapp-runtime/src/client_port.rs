//! Route-specific public client port for WhatsApp commands and operation status.

use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use makosh_whatsapp_api::client_contract::{
    WHATSAPP_CLIENT_CONTRACT_MAJOR, WHATSAPP_CLIENT_CONTRACT_REVISION, WHATSAPP_MODULE_ID,
    WHATSAPP_OWNER_ID, WhatsAppClientContractV1,
};
use makosh_whatsapp_api::{
    WhatsAppPublicClientRequestV1, WhatsAppPublicClientResponseV1, client_wire, operational_wire,
    realtime_wire,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::managed::WhatsAppAdmittedRuntime;

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppClientPortErrorV1 {
    Protocol,
    Runtime,
}

fn whatsapp_client_contract(contract: WhatsAppClientContractV1) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: WHATSAPP_OWNER_ID.to_owned(),
        name: contract.contract_name().to_owned(),
        major: WHATSAPP_CLIENT_CONTRACT_MAJOR,
        revision: WHATSAPP_CLIENT_CONTRACT_REVISION,
        schema_sha256: Sha256::digest(contract.descriptor_set()).to_vec(),
    }
}

fn validate_contract(
    reference: &ContractReferenceV1,
) -> Result<WhatsAppClientContractV1, WhatsAppClientPortErrorV1> {
    let contract = WhatsAppClientContractV1::from_contract_name(&reference.name)
        .ok_or(WhatsAppClientPortErrorV1::Protocol)?;
    if reference != &whatsapp_client_contract(contract) {
        return Err(WhatsAppClientPortErrorV1::Protocol);
    }
    Ok(contract)
}

fn request_contract(request: &WhatsAppPublicClientRequestV1) -> WhatsAppClientContractV1 {
    match request {
        WhatsAppPublicClientRequestV1::Command(_) => WhatsAppClientContractV1::Command,
        WhatsAppPublicClientRequestV1::OperationStatus { .. } => WhatsAppClientContractV1::Query,
        WhatsAppPublicClientRequestV1::OperationalQuery(_) => {
            WhatsAppClientContractV1::OperationalQuery
        }
        WhatsAppPublicClientRequestV1::OperationalReplay(_) => {
            WhatsAppClientContractV1::OperationalRealtime
        }
    }
}

fn encode_request_payload(
    request: &WhatsAppPublicClientRequestV1,
) -> Result<Vec<u8>, WhatsAppClientPortErrorV1> {
    Ok(match request {
        WhatsAppPublicClientRequestV1::Command(command) => client_wire::encode_command(command),
        WhatsAppPublicClientRequestV1::OperationStatus { operation_id } => {
            client_wire::encode_operation_status_query(operation_id)
        }
        WhatsAppPublicClientRequestV1::OperationalQuery(query) => {
            operational_wire::encode_operational_query(query)
                .map_err(|_| WhatsAppClientPortErrorV1::Protocol)?
        }
        WhatsAppPublicClientRequestV1::OperationalReplay(request) => {
            realtime_wire::encode_operational_replay_request(request)
                .map_err(|_| WhatsAppClientPortErrorV1::Protocol)?
        }
    })
}

fn decode_request_payload(
    contract: WhatsAppClientContractV1,
    bytes: &[u8],
) -> Result<WhatsAppPublicClientRequestV1, WhatsAppClientPortErrorV1> {
    match contract {
        WhatsAppClientContractV1::Command => client_wire::decode_command(bytes)
            .map(WhatsAppPublicClientRequestV1::Command)
            .map_err(|_| WhatsAppClientPortErrorV1::Protocol),
        WhatsAppClientContractV1::Query => client_wire::decode_operation_status_query(bytes)
            .map(|operation_id| WhatsAppPublicClientRequestV1::OperationStatus { operation_id })
            .map_err(|_| WhatsAppClientPortErrorV1::Protocol),
        WhatsAppClientContractV1::OperationalQuery => {
            operational_wire::decode_operational_query(bytes)
                .map(WhatsAppPublicClientRequestV1::OperationalQuery)
                .map_err(|_| WhatsAppClientPortErrorV1::Protocol)
        }
        WhatsAppClientContractV1::OperationalRealtime => {
            realtime_wire::decode_operational_replay_request(bytes)
                .map(WhatsAppPublicClientRequestV1::OperationalReplay)
                .map_err(|_| WhatsAppClientPortErrorV1::Protocol)
        }
    }
}

pub fn encode_module_request(
    request_id: u64,
    request: &WhatsAppPublicClientRequestV1,
) -> Result<Vec<u8>, WhatsAppClientPortErrorV1> {
    if request_id == 0 {
        return Err(WhatsAppClientPortErrorV1::Protocol);
    }
    let contract = request_contract(request);
    Ok(ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: WHATSAPP_MODULE_ID.to_owned(),
        owner_id: WHATSAPP_OWNER_ID.to_owned(),
        contract: Some(whatsapp_client_contract(contract)),
        request_id,
        request_payload: encode_request_payload(request)?,
        logical_owner_id: String::new(),
        authenticated_device_id: String::new(),
        authenticated_client_session_id: String::new(),
    }
    .encode_to_vec())
}

pub fn decode_module_request(
    bytes: &[u8],
) -> Result<(u64, WhatsAppClientContractV1, WhatsAppPublicClientRequestV1), WhatsAppClientPortErrorV1>
{
    let envelope =
        ModuleClientRequestV1::decode(bytes).map_err(|_| WhatsAppClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != WHATSAPP_MODULE_ID
        || envelope.owner_id != WHATSAPP_OWNER_ID
        || envelope.request_id == 0
        || envelope.request_payload.is_empty()
    {
        return Err(WhatsAppClientPortErrorV1::Protocol);
    }
    let contract = validate_contract(
        envelope
            .contract
            .as_ref()
            .ok_or(WhatsAppClientPortErrorV1::Protocol)?,
    )?;
    let request = decode_request_payload(contract, &envelope.request_payload)?;
    Ok((envelope.request_id, contract, request))
}

pub async fn handle_client_request(
    runtime: &WhatsAppAdmittedRuntime,
    bytes: &[u8],
    requested_at_unix_seconds: i64,
) -> Result<Vec<u8>, WhatsAppClientPortErrorV1> {
    if requested_at_unix_seconds <= 0 {
        return Err(WhatsAppClientPortErrorV1::Protocol);
    }
    let (request_id, contract, request) = decode_module_request(bytes)?;
    let response = match request {
        WhatsAppPublicClientRequestV1::Command(command) => runtime
            .submit_command(&command, requested_at_unix_seconds)
            .await
            .map(|operation_id| WhatsAppPublicClientResponseV1::Accepted { operation_id })
            .map_err(|_| WhatsAppClientPortErrorV1::Runtime)?,
        WhatsAppPublicClientRequestV1::OperationStatus { operation_id } => runtime
            .command_operation_status(&operation_id)
            .await
            .map(WhatsAppPublicClientResponseV1::OperationStatus)
            .map_err(|_| WhatsAppClientPortErrorV1::Runtime)?,
        WhatsAppPublicClientRequestV1::OperationalQuery(query) => runtime
            .operational_query(&query)
            .await
            .map(WhatsAppPublicClientResponseV1::OperationalQuery)
            .map_err(|_| WhatsAppClientPortErrorV1::Runtime)?,
        WhatsAppPublicClientRequestV1::OperationalReplay(request) => runtime
            .operational_replay(&request)
            .await
            .map(WhatsAppPublicClientResponseV1::OperationalReplay)
            .map_err(|_| WhatsAppClientPortErrorV1::Runtime)?,
    };
    encode_module_response(request_id, contract, &response)
}

fn encode_module_response(
    request_id: u64,
    contract: WhatsAppClientContractV1,
    response: &WhatsAppPublicClientResponseV1,
) -> Result<Vec<u8>, WhatsAppClientPortErrorV1> {
    if request_id == 0 {
        return Err(WhatsAppClientPortErrorV1::Protocol);
    }
    let response_payload = match (contract, response) {
        (
            WhatsAppClientContractV1::Command,
            WhatsAppPublicClientResponseV1::Accepted { operation_id },
        ) => client_wire::encode_command_accepted(operation_id),
        (
            WhatsAppClientContractV1::Query,
            WhatsAppPublicClientResponseV1::OperationStatus(status),
        ) => client_wire::encode_operation_status_response(status.as_ref()),
        (
            WhatsAppClientContractV1::OperationalQuery,
            WhatsAppPublicClientResponseV1::OperationalQuery(response),
        ) => operational_wire::encode_operational_query_response(response),
        (
            WhatsAppClientContractV1::OperationalRealtime,
            WhatsAppPublicClientResponseV1::OperationalReplay(response),
        ) => realtime_wire::encode_operational_replay_response(response)
            .map_err(|_| WhatsAppClientPortErrorV1::Protocol)?,
        _ => return Err(WhatsAppClientPortErrorV1::Protocol),
    };
    Ok(ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload,
        error_code: String::new(),
    }
    .encode_to_vec())
}

pub fn decode_module_response(
    contract: WhatsAppClientContractV1,
    bytes: &[u8],
) -> Result<(u64, WhatsAppPublicClientResponseV1), WhatsAppClientPortErrorV1> {
    let envelope =
        ModuleClientResponseV1::decode(bytes).map_err(|_| WhatsAppClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.request_id == 0
        || !envelope.error_code.is_empty()
    {
        return Err(WhatsAppClientPortErrorV1::Protocol);
    }
    let response = match contract {
        WhatsAppClientContractV1::Command => {
            client_wire::decode_command_accepted(&envelope.response_payload)
                .map(|operation_id| WhatsAppPublicClientResponseV1::Accepted { operation_id })
        }
        WhatsAppClientContractV1::Query => {
            client_wire::decode_operation_status_response(&envelope.response_payload)
                .map(WhatsAppPublicClientResponseV1::OperationStatus)
        }
        WhatsAppClientContractV1::OperationalQuery => {
            operational_wire::decode_operational_query_response(&envelope.response_payload)
                .map(WhatsAppPublicClientResponseV1::OperationalQuery)
        }
        WhatsAppClientContractV1::OperationalRealtime => {
            realtime_wire::decode_operational_replay_response(&envelope.response_payload)
                .map(WhatsAppPublicClientResponseV1::OperationalReplay)
        }
    }
    .map_err(|_| WhatsAppClientPortErrorV1::Protocol)?;
    Ok((envelope.request_id, response))
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::v1::ModuleClientRequestV1;
    use makosh_whatsapp_api::{
        WhatsAppProviderCommand, WhatsAppProviderCommandStateV1, WhatsAppProviderCommandStatusV1,
        operational::{
            WhatsAppOperationalQueryResponseV1, WhatsAppOperationalQueryV1,
            WhatsAppOperationalRuntimeStatusV1,
        },
        realtime::{WhatsAppOperationalReplayRequestV1, WhatsAppOperationalReplayResponseV1},
    };

    use super::*;

    fn command_request() -> WhatsAppPublicClientRequestV1 {
        WhatsAppPublicClientRequestV1::Command(WhatsAppProviderCommand::SendText {
            operation_id: "command-operation".into(),
            account_id: "account".into(),
            provider_chat_id: "chat".into(),
            text: "message".into(),
        })
    }

    fn query_request() -> WhatsAppPublicClientRequestV1 {
        WhatsAppPublicClientRequestV1::OperationStatus {
            operation_id: "query-operation".into(),
        }
    }

    fn operational_query_request() -> WhatsAppPublicClientRequestV1 {
        WhatsAppPublicClientRequestV1::OperationalQuery(
            WhatsAppOperationalQueryV1::GetRuntimeStatus {
                account_id: "account".to_owned(),
            },
        )
    }

    fn operational_replay_request() -> WhatsAppPublicClientRequestV1 {
        WhatsAppPublicClientRequestV1::OperationalReplay(WhatsAppOperationalReplayRequestV1 {
            account_id: "account".to_owned(),
            after_sequence: 0,
            limit: 10,
        })
    }

    #[test]
    fn each_request_uses_only_its_exact_route_contract() {
        for (request, expected) in [
            (command_request(), WhatsAppClientContractV1::Command),
            (query_request(), WhatsAppClientContractV1::Query),
            (
                operational_query_request(),
                WhatsAppClientContractV1::OperationalQuery,
            ),
            (
                operational_replay_request(),
                WhatsAppClientContractV1::OperationalRealtime,
            ),
        ] {
            let encoded = encode_module_request(1, &request).expect("module request");
            let (request_id, contract, decoded) =
                decode_module_request(&encoded).expect("decode module request");

            assert_eq!(request_id, 1);
            assert_eq!(contract, expected);
            assert_eq!(decoded, request);
        }
    }

    #[test]
    fn query_payload_is_rejected_under_command_contract() {
        let encoded = encode_module_request(1, &query_request()).expect("query request");
        let mut envelope = ModuleClientRequestV1::decode(encoded.as_slice()).expect("envelope");
        envelope.contract = Some(whatsapp_client_contract(WhatsAppClientContractV1::Command));

        assert_eq!(
            decode_module_request(&envelope.encode_to_vec()),
            Err(WhatsAppClientPortErrorV1::Protocol)
        );
    }

    #[test]
    fn umbrella_contract_is_not_admitted() {
        let encoded = encode_module_request(1, &query_request()).expect("query request");
        let mut envelope = ModuleClientRequestV1::decode(encoded.as_slice()).expect("envelope");
        envelope.contract.as_mut().expect("contract").name = "whatsapp.client".to_owned();

        assert_eq!(
            decode_module_request(&envelope.encode_to_vec()),
            Err(WhatsAppClientPortErrorV1::Protocol)
        );
    }

    #[test]
    fn command_response_decodes_only_with_command_contract() {
        let response = WhatsAppPublicClientResponseV1::Accepted {
            operation_id: "operation".into(),
        };
        let encoded = encode_module_response(1, WhatsAppClientContractV1::Command, &response)
            .expect("command response");

        assert_eq!(
            decode_module_response(WhatsAppClientContractV1::Command, &encoded),
            Ok((1, response))
        );
        assert_eq!(
            decode_module_response(WhatsAppClientContractV1::Query, &encoded),
            Err(WhatsAppClientPortErrorV1::Protocol)
        );
    }

    #[test]
    fn operation_status_response_round_trips_only_with_query_contract() {
        let response = WhatsAppPublicClientResponseV1::OperationStatus(Some(
            WhatsAppProviderCommandStatusV1 {
                operation_id: "operation".into(),
                account_id: "account".into(),
                state: WhatsAppProviderCommandStateV1::Succeeded,
                requested_at_unix_seconds: 1,
                completed_at_unix_seconds: Some(2),
            },
        ));
        let encoded = encode_module_response(1, WhatsAppClientContractV1::Query, &response)
            .expect("query response");

        assert_eq!(
            decode_module_response(WhatsAppClientContractV1::Query, &encoded),
            Ok((1, response))
        );
        assert_eq!(
            decode_module_response(WhatsAppClientContractV1::Command, &encoded),
            Err(WhatsAppClientPortErrorV1::Protocol)
        );
    }

    #[test]
    fn operational_response_round_trips_only_with_operational_contract() {
        let response = WhatsAppPublicClientResponseV1::OperationalQuery(
            WhatsAppOperationalQueryResponseV1::RuntimeStatus(WhatsAppOperationalRuntimeStatusV1 {
                account_id: "account".to_owned(),
                runtime_state: Some("running".to_owned()),
                projection_ready: true,
                latest_event_sequence: 7,
            }),
        );
        let encoded =
            encode_module_response(1, WhatsAppClientContractV1::OperationalQuery, &response)
                .expect("operational response");

        assert_eq!(
            decode_module_response(WhatsAppClientContractV1::OperationalQuery, &encoded),
            Ok((1, response))
        );
        assert_eq!(
            decode_module_response(WhatsAppClientContractV1::Query, &encoded),
            Err(WhatsAppClientPortErrorV1::Protocol)
        );
    }

    #[test]
    fn realtime_response_round_trips_only_with_realtime_contract() {
        let response = WhatsAppPublicClientResponseV1::OperationalReplay(
            WhatsAppOperationalReplayResponseV1 {
                account_id: "account".to_owned(),
                earliest_available_sequence: None,
                latest_available_sequence: None,
                frames: Vec::new(),
                next_sequence: 0,
                reset_required: false,
            },
        );
        let encoded =
            encode_module_response(1, WhatsAppClientContractV1::OperationalRealtime, &response)
                .expect("realtime response");

        assert_eq!(
            decode_module_response(WhatsAppClientContractV1::OperationalRealtime, &encoded),
            Ok((1, response))
        );
        assert_eq!(
            decode_module_response(WhatsAppClientContractV1::OperationalQuery, &encoded),
            Err(WhatsAppClientPortErrorV1::Protocol)
        );
    }
}
