use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{
        ManagedRuntimeControlRequestV1, ManagedRuntimeModuleRequestRequestV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::module_request::{
        validate_module_request_request_v1, validate_module_request_response_v1,
    },
};

use crate::{
    contracts::delivery_intent_command_contract_v1,
    delivery_port::{DeliveryIntentRequestErrorV1, DeliveryIntentRequestPortV1},
};

pub(crate) struct ManagedDeliveryIntentRequestPortV1<'a> {
    pub channel: &'a mut ManagedControlChannelV2<UnixStream>,
    pub dispatcher: &'a mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
}

impl DeliveryIntentRequestPortV1 for ManagedDeliveryIntentRequestPortV1<'_> {
    async fn request(
        &mut self,
        request_id: [u8; 16],
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, DeliveryIntentRequestErrorV1> {
        let request = ManagedRuntimeModuleRequestRequestV1 {
            request_id: request_id.to_vec(),
            contract: Some(delivery_intent_command_contract_v1()),
            request_payload: payload,
            deadline_millis: 30_000,
            response_blob_capability_id: String::new(),
        };
        validate_module_request_request_v1(&request)
            .map_err(|_| DeliveryIntentRequestErrorV1::Protocol)?;
        let response = self
            .channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::RouteModuleRequest(request)),
                },
                self.dispatcher,
            )
            .map_err(|_| DeliveryIntentRequestErrorV1::Unavailable)?;
        if !response.error_code.is_empty() {
            return Err(DeliveryIntentRequestErrorV1::Unavailable);
        }
        let Some(ControlResult::ModuleRequestRoute(response)) = response.result else {
            return Err(DeliveryIntentRequestErrorV1::Protocol);
        };
        validate_module_request_response_v1(&response)
            .map_err(|_| DeliveryIntentRequestErrorV1::Protocol)?;
        if response.request_id != request_id {
            return Err(DeliveryIntentRequestErrorV1::Protocol);
        }
        if !response.error_code.is_empty() {
            return Err(DeliveryIntentRequestErrorV1::Unavailable);
        }
        Ok(response.response_payload)
    }
}
