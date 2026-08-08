//! Typed local client port for Zulip operational commands and operation status.

use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use makosh_zulip_api::client_contract::{
    ZULIP_CLIENT_CONTRACT_MAJOR, ZULIP_CLIENT_CONTRACT_REVISION, ZULIP_MODULE_ID, ZULIP_OWNER_ID,
    ZulipClientContractV1,
};
use makosh_zulip_api::{
    ZulipClientRequestV1, ZulipClientResponseV1, account_wire, client_wire, operational_wire,
    realtime_wire,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::managed::ZulipAdmittedRuntimeV1;

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZulipClientPortErrorV1 {
    Protocol,
    Runtime,
}

fn zulip_client_contract(contract: ZulipClientContractV1) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: ZULIP_OWNER_ID.to_owned(),
        name: contract.contract_name().to_owned(),
        major: ZULIP_CLIENT_CONTRACT_MAJOR,
        revision: ZULIP_CLIENT_CONTRACT_REVISION,
        schema_sha256: Sha256::digest(contract.descriptor_set()).to_vec(),
    }
}

fn validate_contract(
    reference: &ContractReferenceV1,
) -> Result<ZulipClientContractV1, ZulipClientPortErrorV1> {
    let contract = ZulipClientContractV1::from_contract_name(&reference.name)
        .ok_or(ZulipClientPortErrorV1::Protocol)?;
    if reference != &zulip_client_contract(contract) {
        return Err(ZulipClientPortErrorV1::Protocol);
    }
    Ok(contract)
}

fn request_contract(request: &ZulipClientRequestV1) -> ZulipClientContractV1 {
    match request {
        ZulipClientRequestV1::AccountLifecycle(_) => ZulipClientContractV1::AccountLifecycle,
        ZulipClientRequestV1::Command(_) => ZulipClientContractV1::Command,
        ZulipClientRequestV1::OperationStatus { .. } => ZulipClientContractV1::Query,
        ZulipClientRequestV1::OperationalQuery(_) => ZulipClientContractV1::OperationalQuery,
        ZulipClientRequestV1::OperationalReplay(_) => ZulipClientContractV1::OperationalRealtime,
    }
}

fn encode_request_payload(
    request: &ZulipClientRequestV1,
) -> Result<Vec<u8>, ZulipClientPortErrorV1> {
    Ok(match request {
        ZulipClientRequestV1::AccountLifecycle(command) => {
            account_wire::encode_account_lifecycle_command(command)
                .map_err(|_| ZulipClientPortErrorV1::Protocol)?
        }
        ZulipClientRequestV1::Command(command) => client_wire::encode_command_request(command),
        ZulipClientRequestV1::OperationStatus { operation_id } => {
            client_wire::encode_operation_status_query(operation_id)
        }
        ZulipClientRequestV1::OperationalQuery(query) => {
            operational_wire::encode_operational_query(query)
                .map_err(|_| ZulipClientPortErrorV1::Protocol)?
        }
        ZulipClientRequestV1::OperationalReplay(request) => {
            realtime_wire::encode_operational_replay_request(request)
                .map_err(|_| ZulipClientPortErrorV1::Protocol)?
        }
    })
}

fn decode_request_payload(
    contract: ZulipClientContractV1,
    bytes: &[u8],
) -> Result<ZulipClientRequestV1, ZulipClientPortErrorV1> {
    match contract {
        ZulipClientContractV1::AccountLifecycle => {
            account_wire::decode_account_lifecycle_command(bytes)
                .map(ZulipClientRequestV1::AccountLifecycle)
                .map_err(|_| ZulipClientPortErrorV1::Protocol)
        }
        ZulipClientContractV1::Command => client_wire::decode_command_request(bytes)
            .map(ZulipClientRequestV1::Command)
            .map_err(|_| ZulipClientPortErrorV1::Protocol),
        ZulipClientContractV1::Query => client_wire::decode_operation_status_query(bytes)
            .map(|operation_id| ZulipClientRequestV1::OperationStatus { operation_id })
            .map_err(|_| ZulipClientPortErrorV1::Protocol),
        ZulipClientContractV1::OperationalQuery => {
            operational_wire::decode_operational_query(bytes)
                .map(ZulipClientRequestV1::OperationalQuery)
                .map_err(|_| ZulipClientPortErrorV1::Protocol)
        }
        ZulipClientContractV1::OperationalRealtime => {
            realtime_wire::decode_operational_replay_request(bytes)
                .map(ZulipClientRequestV1::OperationalReplay)
                .map_err(|_| ZulipClientPortErrorV1::Protocol)
        }
    }
}

pub fn encode_module_request(
    request_id: u64,
    request: &ZulipClientRequestV1,
) -> Result<Vec<u8>, ZulipClientPortErrorV1> {
    if request_id == 0 {
        return Err(ZulipClientPortErrorV1::Protocol);
    }
    let contract = request_contract(request);
    Ok(ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: ZULIP_MODULE_ID.to_owned(),
        owner_id: ZULIP_OWNER_ID.to_owned(),
        contract: Some(zulip_client_contract(contract)),
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
) -> Result<(u64, ZulipClientContractV1, ZulipClientRequestV1), ZulipClientPortErrorV1> {
    let envelope =
        ModuleClientRequestV1::decode(bytes).map_err(|_| ZulipClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != ZULIP_MODULE_ID
        || envelope.owner_id != ZULIP_OWNER_ID
        || envelope.request_id == 0
        || envelope.request_payload.is_empty()
    {
        return Err(ZulipClientPortErrorV1::Protocol);
    }
    let contract = validate_contract(
        envelope
            .contract
            .as_ref()
            .ok_or(ZulipClientPortErrorV1::Protocol)?,
    )?;
    let request = decode_request_payload(contract, &envelope.request_payload)?;
    Ok((envelope.request_id, contract, request))
}

pub async fn handle_client_request(
    runtime: &ZulipAdmittedRuntimeV1,
    bytes: &[u8],
    requested_at_unix_seconds: i64,
) -> Result<Vec<u8>, ZulipClientPortErrorV1> {
    if requested_at_unix_seconds <= 0 {
        return Err(ZulipClientPortErrorV1::Protocol);
    }
    let (request_id, contract, request) = decode_module_request(bytes)?;
    let response = match request {
        ZulipClientRequestV1::AccountLifecycle(command) => runtime
            .apply_account_lifecycle(&command, requested_at_unix_seconds)
            .await
            .map(ZulipClientResponseV1::AccountLifecycle)
            .map_err(|_| ZulipClientPortErrorV1::Runtime)?,
        ZulipClientRequestV1::Command(command) => runtime
            .submit_command(&command, requested_at_unix_seconds)
            .await
            .map(ZulipClientResponseV1::CommandReceipt)
            .map_err(|_| ZulipClientPortErrorV1::Runtime)?,
        ZulipClientRequestV1::OperationStatus { operation_id } => runtime
            .command_operation_status(&operation_id)
            .await
            .map(ZulipClientResponseV1::OperationStatus)
            .map_err(|_| ZulipClientPortErrorV1::Runtime)?,
        ZulipClientRequestV1::OperationalQuery(query) => runtime
            .operational_query(&query)
            .await
            .map(ZulipClientResponseV1::OperationalQuery)
            .map_err(|_| ZulipClientPortErrorV1::Runtime)?,
        ZulipClientRequestV1::OperationalReplay(request) => runtime
            .operational_replay(&request)
            .await
            .map(ZulipClientResponseV1::OperationalReplay)
            .map_err(|_| ZulipClientPortErrorV1::Runtime)?,
    };
    encode_module_response(request_id, contract, &response)
}

fn encode_module_response(
    request_id: u64,
    contract: ZulipClientContractV1,
    response: &ZulipClientResponseV1,
) -> Result<Vec<u8>, ZulipClientPortErrorV1> {
    if request_id == 0 {
        return Err(ZulipClientPortErrorV1::Protocol);
    }
    let response_payload = match (contract, response) {
        (
            ZulipClientContractV1::AccountLifecycle,
            ZulipClientResponseV1::AccountLifecycle(receipt),
        ) => account_wire::encode_account_lifecycle_receipt(receipt)
            .map_err(|_| ZulipClientPortErrorV1::Protocol)?,
        (ZulipClientContractV1::Command, ZulipClientResponseV1::CommandReceipt(receipt)) => {
            client_wire::encode_command_response(receipt)
        }
        (ZulipClientContractV1::Query, ZulipClientResponseV1::OperationStatus(status)) => {
            client_wire::encode_operation_status_response(status.as_ref())
        }
        (
            ZulipClientContractV1::OperationalQuery,
            ZulipClientResponseV1::OperationalQuery(response),
        ) => operational_wire::encode_operational_query_response(response),
        (
            ZulipClientContractV1::OperationalRealtime,
            ZulipClientResponseV1::OperationalReplay(response),
        ) => realtime_wire::encode_operational_replay_response(response)
            .map_err(|_| ZulipClientPortErrorV1::Protocol)?,
        _ => return Err(ZulipClientPortErrorV1::Protocol),
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
    contract: ZulipClientContractV1,
    bytes: &[u8],
) -> Result<(u64, ZulipClientResponseV1), ZulipClientPortErrorV1> {
    let envelope =
        ModuleClientResponseV1::decode(bytes).map_err(|_| ZulipClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.request_id == 0
        || !envelope.error_code.is_empty()
    {
        return Err(ZulipClientPortErrorV1::Protocol);
    }
    let response = match contract {
        ZulipClientContractV1::AccountLifecycle => {
            account_wire::decode_account_lifecycle_receipt(&envelope.response_payload)
                .map(ZulipClientResponseV1::AccountLifecycle)
        }
        ZulipClientContractV1::Command => {
            client_wire::decode_command_response(&envelope.response_payload)
                .map(ZulipClientResponseV1::CommandReceipt)
        }
        ZulipClientContractV1::Query => {
            client_wire::decode_operation_status_response(&envelope.response_payload)
                .map(ZulipClientResponseV1::OperationStatus)
        }
        ZulipClientContractV1::OperationalQuery => {
            operational_wire::decode_operational_query_response(&envelope.response_payload)
                .map(ZulipClientResponseV1::OperationalQuery)
        }
        ZulipClientContractV1::OperationalRealtime => {
            realtime_wire::decode_operational_replay_response(&envelope.response_payload)
                .map(ZulipClientResponseV1::OperationalReplay)
        }
    }
    .map_err(|_| ZulipClientPortErrorV1::Protocol)?;
    Ok((envelope.request_id, response))
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::v1::ModuleClientRequestV1;
    use makosh_zulip_api::{
        ZulipCommandReceiptV1, ZulipCommandV1, account::ZulipAccountLifecycleCommandV1,
        client_contract::ZulipClientContractV1, operational::ZulipOperationalQueryV1,
        realtime::ZulipOperationalReplayRequestV1,
    };
    use prost::Message;

    use super::*;

    fn command_request() -> ZulipClientRequestV1 {
        ZulipClientRequestV1::Command(ZulipCommandV1::SendStream {
            operation_id: "command-operation".into(),
            account_id: "account".into(),
            stream: "stream".into(),
            topic: "topic".into(),
            content: "content".into(),
        })
    }

    fn account_request() -> ZulipClientRequestV1 {
        ZulipClientRequestV1::AccountLifecycle(ZulipAccountLifecycleCommandV1::BindCredential {
            account_id: "account".to_owned(),
            expected_binding_revision: 0,
            credential_revision: 1,
        })
    }

    fn query_request() -> ZulipClientRequestV1 {
        ZulipClientRequestV1::OperationStatus {
            operation_id: "query-operation".into(),
        }
    }

    fn operational_query_request() -> ZulipClientRequestV1 {
        ZulipClientRequestV1::OperationalQuery(ZulipOperationalQueryV1::GetAccountStatus {
            account_id: "account".into(),
        })
    }

    fn operational_replay_request() -> ZulipClientRequestV1 {
        ZulipClientRequestV1::OperationalReplay(ZulipOperationalReplayRequestV1 {
            account_id: "account".into(),
            after_sequence: 0,
            limit: 20,
        })
    }

    #[test]
    fn each_request_uses_only_its_exact_route_contract() {
        assert_eq!(
            decode_module_request(&encode_module_request(4, &account_request()).expect("encode"))
                .expect("decode")
                .1,
            ZulipClientContractV1::AccountLifecycle,
        );
        for (request, expected) in [
            (command_request(), ZulipClientContractV1::Command),
            (query_request(), ZulipClientContractV1::Query),
            (
                operational_query_request(),
                ZulipClientContractV1::OperationalQuery,
            ),
            (
                operational_replay_request(),
                ZulipClientContractV1::OperationalRealtime,
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
        envelope.contract = Some(zulip_client_contract(ZulipClientContractV1::Command));

        assert_eq!(
            decode_module_request(&envelope.encode_to_vec()),
            Err(ZulipClientPortErrorV1::Protocol)
        );
    }

    #[test]
    fn umbrella_contract_is_not_admitted() {
        let encoded = encode_module_request(1, &query_request()).expect("query request");
        let mut envelope = ModuleClientRequestV1::decode(encoded.as_slice()).expect("envelope");
        envelope.contract.as_mut().expect("contract").name = "zulip.client".to_owned();

        assert_eq!(
            decode_module_request(&envelope.encode_to_vec()),
            Err(ZulipClientPortErrorV1::Protocol)
        );
    }

    #[test]
    fn command_response_decodes_only_with_command_contract() {
        let response = ZulipClientResponseV1::CommandReceipt(ZulipCommandReceiptV1 {
            operation_id: "operation".into(),
            account_id: "account".into(),
        });
        let encoded = encode_module_response(1, ZulipClientContractV1::Command, &response)
            .expect("command response");

        assert_eq!(
            decode_module_response(ZulipClientContractV1::Command, &encoded),
            Ok((1, response))
        );
        assert_eq!(
            decode_module_response(ZulipClientContractV1::Query, &encoded),
            Err(ZulipClientPortErrorV1::Protocol)
        );
    }

    #[test]
    fn absent_operation_status_preserves_the_canonical_empty_protobuf_payload() {
        let response = ZulipClientResponseV1::OperationStatus(None);
        let encoded = encode_module_response(1, ZulipClientContractV1::Query, &response)
            .expect("operation status response");

        assert_eq!(
            decode_module_response(ZulipClientContractV1::Query, &encoded),
            Ok((1, response))
        );
        assert_eq!(
            decode_module_response(ZulipClientContractV1::Command, &encoded),
            Err(ZulipClientPortErrorV1::Protocol)
        );
    }
}
