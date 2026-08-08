//! Managed-module Query RPC adapter for the canonical Communications query contract.

use std::os::unix::net::UnixStream;

use makosh_communications_persistence::CommunicationsDurablePersistence;
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{ManagedRuntimeModuleQueryDeliveryV1, ManagedRuntimeModuleQueryResponseV1},
    validation::module_query::validate_module_query_delivery_v1,
};

use crate::{
    admission::communications_query_contract_reference_v1,
    query_port::{CommunicationsQueryPortErrorV1, handle_query_request_v1},
    search_access::CommunicationsSearchAccessV1,
};

pub async fn handle_module_query_delivery_v1(
    persistence: &CommunicationsDurablePersistence,
    search_access: &mut CommunicationsSearchAccessV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    delivery: ManagedRuntimeModuleQueryDeliveryV1,
) -> ManagedRuntimeModuleQueryResponseV1 {
    let request_id = delivery.request_id.clone();
    if validate_module_query_delivery_v1(&delivery).is_err()
        || delivery.contract.as_ref() != Some(&communications_query_contract_reference_v1())
    {
        return rejected(request_id);
    }
    match handle_query_request_v1(
        persistence,
        search_access,
        control_channel,
        dispatcher,
        &delivery.request_payload,
    )
    .await
    {
        Ok(response_payload) => ManagedRuntimeModuleQueryResponseV1 {
            request_id,
            response_payload,
            error_code: String::new(),
        },
        Err(CommunicationsQueryPortErrorV1::Protocol) => rejected(request_id),
        Err(CommunicationsQueryPortErrorV1::Unavailable) => unavailable(request_id),
    }
}

fn rejected(request_id: Vec<u8>) -> ManagedRuntimeModuleQueryResponseV1 {
    ManagedRuntimeModuleQueryResponseV1 {
        request_id,
        response_payload: Vec::new(),
        error_code: "REJECTED".to_owned(),
    }
}

fn unavailable(request_id: Vec<u8>) -> ManagedRuntimeModuleQueryResponseV1 {
    ManagedRuntimeModuleQueryResponseV1 {
        request_id,
        response_payload: Vec::new(),
        error_code: "UNAVAILABLE".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::rejected;

    #[test]
    fn rejection_preserves_only_the_query_correlation_id() {
        let response = rejected(vec![7; 16]);
        assert_eq!(response.request_id, vec![7; 16]);
        assert_eq!(response.error_code, "REJECTED");
        assert!(response.response_payload.is_empty());
    }
}
