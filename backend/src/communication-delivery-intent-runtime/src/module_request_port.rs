//! Managed-module Request RPC adapter for the public delivery-intent command.

use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::{
    managed_control::ManagedControlRequestDispatcherV2,
    v1::{ManagedRuntimeModuleRequestDeliveryV1, ManagedRuntimeModuleRequestResponseV1},
    validation::module_request::validate_module_request_delivery_v1,
};

use crate::{
    contracts::command_contract_v1, runtime::DeliveryIntentManagedRuntimeV1,
    submit_port::submit_delivery_intent_payload_v1,
};

pub(crate) async fn handle_module_request_delivery_v1(
    runtime: &mut DeliveryIntentManagedRuntimeV1,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    delivery: ManagedRuntimeModuleRequestDeliveryV1,
    now_unix_seconds: i64,
) -> ManagedRuntimeModuleRequestResponseV1 {
    let request_id = delivery.request_id.clone();
    if validate_module_request_delivery_v1(&delivery).is_err()
        || delivery.logical_owner_id != runtime.logical_owner_id
        || delivery.contract.as_ref() != Some(&command_contract_v1())
    {
        return rejected(request_id);
    }
    ManagedRuntimeModuleRequestResponseV1 {
        request_id,
        response_payload: submit_delivery_intent_payload_v1(
            runtime,
            dispatcher,
            &delivery.request_payload,
            now_unix_seconds,
        )
        .await,
        error_code: String::new(),
    }
}

fn rejected(request_id: Vec<u8>) -> ManagedRuntimeModuleRequestResponseV1 {
    ManagedRuntimeModuleRequestResponseV1 {
        request_id,
        response_payload: Vec::new(),
        error_code: "REJECTED".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::rejected;

    #[test]
    fn rejection_preserves_only_the_request_correlation_id() {
        let response = rejected(vec![7; 16]);
        assert_eq!(response.request_id, vec![7; 16]);
        assert_eq!(response.error_code, "REJECTED");
        assert!(response.response_payload.is_empty());
    }
}
