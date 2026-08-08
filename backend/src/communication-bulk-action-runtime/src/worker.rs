use makosh_communication_bulk_action_persistence::{
    BulkDeliveryPersistenceErrorV1, CommunicationBulkActionPersistenceV1, CompleteTargetOutcomeV1,
};

use crate::delivery_port::{
    DeliveryIntentRequestErrorV1, DeliveryIntentRequestPortV1, DeliveryIntentRequestV1,
    DeliveryIntentResponseV1, decode_delivery_intent_response_v1,
};

const ERROR_DELIVERY_INTENT_REJECTED: u16 = 3;
const ERROR_UNAVAILABLE: u16 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulkDeliveryWorkerErrorV1 {
    InvalidInput,
    Persistence(BulkDeliveryPersistenceErrorV1),
}

pub async fn process_next_target_v1(
    persistence: &CommunicationBulkActionPersistenceV1,
    port: &mut impl DeliveryIntentRequestPortV1,
    logical_owner_id: &str,
    worker_id: &str,
    now_unix_seconds: i64,
) -> Result<Option<CompleteTargetOutcomeV1>, BulkDeliveryWorkerErrorV1> {
    let Some(claim) = persistence
        .claim_next_target(logical_owner_id, worker_id, now_unix_seconds)
        .await
        .map_err(BulkDeliveryWorkerErrorV1::Persistence)?
    else {
        return Ok(None);
    };
    let request_id = claim.target_operation_id;
    let payload = DeliveryIntentRequestV1 {
        operation_id: claim.target_operation_id,
        conversation_id: claim.conversation_id,
        reply_to_message_id: claim.reply_to_message_id,
        body_utf8: claim.body_utf8.clone(),
    }
    .encode();
    let response = match port.request(request_id, payload).await {
        Ok(payload) => decode_delivery_intent_response_v1(request_id, &payload),
        Err(error) => Err(error),
    };
    let outcome = match response {
        Ok(DeliveryIntentResponseV1::Accepted { intent_id }) => {
            persistence
                .mark_target_accepted(&claim, intent_id, now_unix_seconds)
                .await
        }
        Ok(DeliveryIntentResponseV1::Rejected) => {
            persistence
                .mark_target_rejected(&claim, ERROR_DELIVERY_INTENT_REJECTED, now_unix_seconds)
                .await
        }
        Ok(DeliveryIntentResponseV1::Retryable)
        | Err(DeliveryIntentRequestErrorV1::Unavailable)
        | Err(DeliveryIntentRequestErrorV1::Protocol) => {
            persistence
                .mark_target_retryable(&claim, ERROR_UNAVAILABLE, now_unix_seconds)
                .await
        }
    }
    .map_err(BulkDeliveryWorkerErrorV1::Persistence)?;
    Ok(Some(outcome))
}
