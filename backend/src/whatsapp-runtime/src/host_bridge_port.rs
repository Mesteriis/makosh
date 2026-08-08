//! Typed private host-bridge port for WhatsApp observations and command leases.

use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use makosh_whatsapp_api::{
    client_contract::{WHATSAPP_DESCRIPTOR_SET_V1, WHATSAPP_MODULE_ID, WHATSAPP_OWNER_ID},
    host_bridge::{
        HOST_BRIDGE_CONTRACT_MAJOR, HOST_BRIDGE_CONTRACT_NAME, HOST_BRIDGE_CONTRACT_REVISION,
        WhatsAppHostBridgeOperationV1, decode_host_bridge_operation,
        encode_host_bridge_command_lease, encode_host_bridge_observation_accepted,
    },
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::managed::WhatsAppAdmittedRuntime;

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

#[derive(Debug)]
pub enum WhatsAppHostBridgePortError {
    Protocol,
    HostBridge,
    Ingress,
}

fn decode_host_request(
    bytes: &[u8],
) -> Result<(u64, WhatsAppHostBridgeOperationV1), WhatsAppHostBridgePortError> {
    let request =
        ModuleClientRequestV1::decode(bytes).map_err(|_| WhatsAppHostBridgePortError::Protocol)?;
    if request.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || request.module_id != WHATSAPP_MODULE_ID
        || request.owner_id != WHATSAPP_OWNER_ID
        || request.request_id == 0
        || request.contract.as_ref() != Some(&client_contract())
        || request.request_payload.is_empty()
    {
        return Err(WhatsAppHostBridgePortError::Protocol);
    }
    let operation = decode_host_bridge_operation(&request.request_payload)
        .map_err(|_| WhatsAppHostBridgePortError::HostBridge)?;
    Ok((request.request_id, operation))
}

pub async fn handle_host_request(
    runtime: &WhatsAppAdmittedRuntime,
    bytes: &[u8],
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
) -> Result<Vec<u8>, WhatsAppHostBridgePortError> {
    let (request_id, request) = decode_host_request(bytes)?;
    let response_payload = match request {
        WhatsAppHostBridgeOperationV1::Observation(envelope) => {
            runtime
                .accept_host_observation(&envelope, recorded_at_unix_seconds, recorded_at_nanos)
                .await
                .map_err(|_| WhatsAppHostBridgePortError::Ingress)?;
            encode_host_bridge_observation_accepted(&envelope.provider_event_id)
                .map_err(|_| WhatsAppHostBridgePortError::HostBridge)?
        }
        WhatsAppHostBridgeOperationV1::ClaimCommands(claim) => {
            let commands = runtime
                .claim_host_commands(
                    &claim.account_id,
                    &claim.host_claim_id,
                    recorded_at_unix_seconds,
                    i64::from(claim.lease_seconds),
                    i64::from(claim.limit),
                )
                .await
                .map_err(|_| WhatsAppHostBridgePortError::Ingress)?;
            encode_host_bridge_command_lease(&commands)
        }
    };
    Ok(ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload,
        error_code: String::new(),
    }
    .encode_to_vec())
}

fn client_contract() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: WHATSAPP_OWNER_ID.to_owned(),
        name: HOST_BRIDGE_CONTRACT_NAME.to_owned(),
        major: HOST_BRIDGE_CONTRACT_MAJOR,
        revision: HOST_BRIDGE_CONTRACT_REVISION,
        schema_sha256: Sha256::digest(WHATSAPP_DESCRIPTOR_SET_V1).to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_whatsapp_api::host_bridge::{
        HOST_BRIDGE_PROTOCOL_MAJOR, HOST_BRIDGE_PROTOCOL_REVISION, WhatsAppHostBridgeEnvelopeV1,
        WhatsAppHostCommandClaimV1, WhatsAppHostObservationV1, encode_host_bridge_payload,
        encode_host_command_claim,
    };

    #[test]
    fn accepts_only_the_exact_whatsapp_host_bridge_contract() {
        let payload = encode_host_bridge_payload(&WhatsAppHostBridgeEnvelopeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            account_id: "wa-1".to_owned(),
            provider_event_id: "event-1".to_owned(),
            observed_at_unix_seconds: 1_782_504_000,
            observation: WhatsAppHostObservationV1::RuntimeState {
                state: "running".to_owned(),
            },
        })
        .expect("payload");
        let request = ModuleClientRequestV1 {
            protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
            module_id: WHATSAPP_MODULE_ID.to_owned(),
            owner_id: WHATSAPP_OWNER_ID.to_owned(),
            contract: Some(client_contract()),
            request_id: 7,
            request_payload: payload,
            logical_owner_id: String::new(),
            authenticated_device_id: String::new(),
            authenticated_client_session_id: String::new(),
        };

        let (request_id, request) =
            decode_host_request(&request.encode_to_vec()).expect("decoded host observation");

        assert_eq!(request_id, 7);
        assert!(
            matches!(request, WhatsAppHostBridgeOperationV1::Observation(envelope) if envelope.provider_event_id == "event-1")
        );

        let claim = encode_host_command_claim(&WhatsAppHostCommandClaimV1 {
            account_id: "wa-1".to_owned(),
            host_claim_id: "host-1".to_owned(),
            lease_seconds: 30,
            limit: 4,
        })
        .expect("claim");
        let request = ModuleClientRequestV1 {
            protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
            module_id: WHATSAPP_MODULE_ID.to_owned(),
            owner_id: WHATSAPP_OWNER_ID.to_owned(),
            contract: Some(client_contract()),
            request_id: 8,
            request_payload: claim,
            logical_owner_id: String::new(),
            authenticated_device_id: String::new(),
            authenticated_client_session_id: String::new(),
        };

        let (_, request) =
            decode_host_request(&request.encode_to_vec()).expect("decoded host claim");
        assert!(
            matches!(request, WhatsAppHostBridgeOperationV1::ClaimCommands(claim) if claim.host_claim_id == "host-1")
        );
    }
}
