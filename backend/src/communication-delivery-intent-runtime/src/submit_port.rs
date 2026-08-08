//! Provider-neutral submit use case shared by client and managed-module ports.

use std::os::unix::net::UnixStream;

use makosh_communication_delivery_intent_api::{
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
    wire::{
        DeliveryIntentErrorCodeV1, DeliveryIntentStatusV1, SubmitDeliveryIntentRequestV1,
        SubmitDeliveryIntentResponseV1,
    },
};
use makosh_communication_delivery_intent_core::{
    CommunicationConversationIdV1, CommunicationMessageIdV1, DeliveryIntentDraftV1,
};
use makosh_communication_delivery_intent_persistence::CreateDeliveryIntentOutcomeV1;
use makosh_runtime_protocol::managed_control::ManagedControlRequestDispatcherV2;
use prost::Message;

use crate::{
    client_status::status_value,
    runtime::{DeliveryIntentManagedRuntimeV1, DeliveryIntentRuntimeErrorV1},
};

pub(crate) async fn submit_delivery_intent_payload_v1(
    runtime: &mut DeliveryIntentManagedRuntimeV1,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    bytes: &[u8],
    now_unix_seconds: i64,
) -> Vec<u8> {
    let Ok(request) = SubmitDeliveryIntentRequestV1::decode(bytes) else {
        return submit_error(
            Vec::new(),
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeInvalidRequest,
        );
    };
    let intent_id = request.operation_id.clone();
    let Ok(operation_id) = id16(&request.operation_id) else {
        return submit_error(
            intent_id,
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeInvalidRequest,
        );
    };
    let Ok(conversation_id) = id16(&request.conversation_id) else {
        return submit_error(
            intent_id,
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeInvalidRequest,
        );
    };
    let reply_to_message_id = request.reply_to_message_id.as_deref().map(id16).transpose();
    if request.protocol_major != COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1 {
        return submit_error(
            intent_id,
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeInvalidRequest,
        );
    }
    let Ok(reply_to_message_id) = reply_to_message_id else {
        return submit_error(
            intent_id,
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeInvalidRequest,
        );
    };
    let outcome = match runtime
        .submit_delivery_intent_v1(
            DeliveryIntentDraftV1 {
                operation_id,
                conversation_id: CommunicationConversationIdV1::new(conversation_id),
                reply_to_message_id: reply_to_message_id.map(CommunicationMessageIdV1::new),
                body_utf8: request.body_utf8,
            },
            now_unix_seconds,
            dispatcher,
        )
        .await
    {
        Ok(outcome) => outcome,
        Err(error) => return submit_error(intent_id, runtime_error(error)),
    };
    let status = match outcome {
        CreateDeliveryIntentOutcomeV1::Created(status)
        | CreateDeliveryIntentOutcomeV1::Existing(status) => status,
    };
    SubmitDeliveryIntentResponseV1 {
        intent_id: status.intent_id.to_vec(),
        status: status_value(status.state),
        error: DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeUnspecified as i32,
    }
    .encode_to_vec()
}

fn submit_error(intent_id: Vec<u8>, error: DeliveryIntentErrorCodeV1) -> Vec<u8> {
    SubmitDeliveryIntentResponseV1 {
        intent_id,
        status: DeliveryIntentStatusV1::DeliveryIntentStatusUnspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn id16(value: &[u8]) -> Result<[u8; 16], &'static str> {
    let value: [u8; 16] = value.try_into().map_err(|_| "INVALID_REQUEST")?;
    if value.iter().all(|byte| *byte == 0) {
        return Err("INVALID_REQUEST");
    }
    Ok(value)
}

const fn runtime_error(error: DeliveryIntentRuntimeErrorV1) -> DeliveryIntentErrorCodeV1 {
    match error {
        DeliveryIntentRuntimeErrorV1::InvalidRequest => {
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeInvalidRequest
        }
        DeliveryIntentRuntimeErrorV1::RouteUnavailable => {
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeRouteUnavailable
        }
        DeliveryIntentRuntimeErrorV1::Coordinator(_) => {
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodePolicyRejected
        }
        DeliveryIntentRuntimeErrorV1::Admission
        | DeliveryIntentRuntimeErrorV1::Persistence(_)
        | DeliveryIntentRuntimeErrorV1::EventContract
        | DeliveryIntentRuntimeErrorV1::Unavailable => {
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeUnavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_submit_errors_stay_inside_the_public_response() {
        let response = SubmitDeliveryIntentResponseV1::decode(
            submit_error(
                vec![1; 16],
                DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeRouteUnavailable,
            )
            .as_slice(),
        )
        .expect("typed submit response");
        assert_eq!(response.intent_id, vec![1; 16]);
        assert_eq!(
            response.error,
            DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeRouteUnavailable as i32
        );
    }
}
