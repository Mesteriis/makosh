use makosh_communication_delivery_intent_api::{
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
    wire::{
        DeliveryIntentErrorCodeV1, DeliveryIntentStatusV1, SubmitDeliveryIntentRequestV1,
        SubmitDeliveryIntentResponseV1,
    },
};
use prost::Message;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryIntentRequestV1 {
    pub operation_id: [u8; 16],
    pub conversation_id: [u8; 16],
    pub reply_to_message_id: Option<[u8; 16]>,
    pub body_utf8: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentResponseV1 {
    Accepted { intent_id: [u8; 16] },
    Rejected,
    Retryable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryIntentRequestErrorV1 {
    Unavailable,
    Protocol,
}

#[allow(async_fn_in_trait)]
pub trait DeliveryIntentRequestPortV1 {
    async fn request(
        &mut self,
        request_id: [u8; 16],
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, DeliveryIntentRequestErrorV1>;
}

impl DeliveryIntentRequestV1 {
    #[must_use]
    pub fn encode(self) -> Vec<u8> {
        SubmitDeliveryIntentRequestV1 {
            protocol_major: COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
            operation_id: self.operation_id.to_vec(),
            conversation_id: self.conversation_id.to_vec(),
            reply_to_message_id: self.reply_to_message_id.map(|id| id.to_vec()),
            body_utf8: self.body_utf8,
        }
        .encode_to_vec()
    }
}

pub fn decode_delivery_intent_response_v1(
    expected_operation_id: [u8; 16],
    payload: &[u8],
) -> Result<DeliveryIntentResponseV1, DeliveryIntentRequestErrorV1> {
    let response = SubmitDeliveryIntentResponseV1::decode(payload)
        .map_err(|_| DeliveryIntentRequestErrorV1::Protocol)?;
    let error = DeliveryIntentErrorCodeV1::try_from(response.error)
        .map_err(|_| DeliveryIntentRequestErrorV1::Protocol)?;
    if error == DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeUnavailable {
        return Ok(DeliveryIntentResponseV1::Retryable);
    }
    if error != DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeUnspecified {
        return Ok(DeliveryIntentResponseV1::Rejected);
    }
    let intent_id: [u8; 16] = response
        .intent_id
        .try_into()
        .map_err(|_| DeliveryIntentRequestErrorV1::Protocol)?;
    let status = DeliveryIntentStatusV1::try_from(response.status)
        .map_err(|_| DeliveryIntentRequestErrorV1::Protocol)?;
    if intent_id != expected_operation_id
        || status == DeliveryIntentStatusV1::DeliveryIntentStatusUnspecified
    {
        return Err(DeliveryIntentRequestErrorV1::Protocol);
    }
    Ok(DeliveryIntentResponseV1::Accepted { intent_id })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_a_mismatched_receipt_and_retries_only_unavailable() {
        let mismatched = SubmitDeliveryIntentResponseV1 {
            intent_id: vec![8; 16],
            status: DeliveryIntentStatusV1::DeliveryIntentStatusAccepted as i32,
            error: DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeUnspecified as i32,
        }
        .encode_to_vec();
        assert_eq!(
            decode_delivery_intent_response_v1([7; 16], &mismatched),
            Err(DeliveryIntentRequestErrorV1::Protocol)
        );
        let unavailable = SubmitDeliveryIntentResponseV1 {
            intent_id: vec![7; 16],
            status: DeliveryIntentStatusV1::DeliveryIntentStatusUnspecified as i32,
            error: DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeUnavailable as i32,
        }
        .encode_to_vec();
        assert_eq!(
            decode_delivery_intent_response_v1([7; 16], &unavailable),
            Ok(DeliveryIntentResponseV1::Retryable)
        );
    }
}
