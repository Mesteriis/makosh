use makosh_events_protocol::{delivery::OutboxRecordV1, v1::ResultOutcomeV1};
use makosh_telegram_delivery_intent_contract::{
    TELEGRAM_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1, TELEGRAM_DELIVERY_INTENT_TARGET_MODULE_ID_V1,
    telegram_delivery_intent_execute_contract_reference_v1,
    telegram_delivery_intent_rejected_contract_reference_v1,
    telegram_delivery_intent_succeeded_contract_reference_v1,
    validate_telegram_delivery_intent_execute_v1, validate_telegram_delivery_intent_rejected_v1,
    validate_telegram_delivery_intent_succeeded_v1,
    wire::{
        ExecuteTelegramDeliveryIntentCommandV1, TelegramDeliveryIntentBodySourceReceiptV1,
        TelegramDeliveryIntentRejectedV1, TelegramDeliveryIntentSucceededV1,
    },
};
use prost::Message;

use crate::{
    DecodedDeliveryIntentTerminalV1, DeliveryIntentBodySourceV1, DeliveryIntentCommandContextV1,
    DeliveryIntentEventAdapterErrorV1, DeliveryIntentTerminalOutcomeV1, build_command_outbox_v1,
    decode_result_envelope_v1, validate_result_identity_v1,
};

const MESSAGE_DOMAIN: &[u8] = b"makosh.telegram.delivery-intent.execute.v1";

pub fn build_execute_outbox_v1(
    intent_id: [u8; 16],
    logical_owner_id: &str,
    account_source_cursor: [u8; 32],
    conversation_source_cursor: [u8; 32],
    reply_to_source_cursor: Option<[u8; 32]>,
    body_source: &DeliveryIntentBodySourceV1,
    context: &DeliveryIntentCommandContextV1,
) -> Result<OutboxRecordV1, DeliveryIntentEventAdapterErrorV1> {
    let payload = ExecuteTelegramDeliveryIntentCommandV1 {
        intent_id: intent_id.to_vec(),
        logical_owner_id: logical_owner_id.to_owned(),
        account_source_cursor: account_source_cursor.to_vec(),
        conversation_source_cursor: conversation_source_cursor.to_vec(),
        reply_to_source_cursor: reply_to_source_cursor.map(|value| value.to_vec()),
        body_source: Some(TelegramDeliveryIntentBodySourceReceiptV1 {
            reference_id: body_source.reference_id.to_vec(),
            declared_bytes: body_source.declared_bytes,
            sha256: body_source.sha256.to_vec(),
            custody_transfer_source_proof: body_source.custody_transfer_source_proof.clone(),
        }),
    };
    validate_telegram_delivery_intent_execute_v1(&payload)
        .map_err(|_| DeliveryIntentEventAdapterErrorV1::InvalidPayload)?;
    build_command_outbox_v1(
        intent_id,
        logical_owner_id,
        telegram_delivery_intent_execute_contract_reference_v1(),
        TELEGRAM_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1,
        MESSAGE_DOMAIN,
        payload.encode_to_vec(),
        context,
    )
}

pub fn decode_succeeded_v1(
    exact_bytes: &[u8],
) -> Result<DecodedDeliveryIntentTerminalV1, DeliveryIntentEventAdapterErrorV1> {
    let (envelope, command_message_id, envelope_sha256) = decode_result_envelope_v1(
        exact_bytes,
        &telegram_delivery_intent_succeeded_contract_reference_v1(),
        TELEGRAM_DELIVERY_INTENT_TARGET_MODULE_ID_V1,
        ResultOutcomeV1::Succeeded,
    )?;
    let payload = TelegramDeliveryIntentSucceededV1::decode(envelope.payload.as_slice())
        .map_err(|_| DeliveryIntentEventAdapterErrorV1::InvalidPayload)?;
    validate_telegram_delivery_intent_succeeded_v1(&payload)
        .map_err(|_| DeliveryIntentEventAdapterErrorV1::InvalidPayload)?;
    let intent_id = validate_result_identity_v1(&envelope, &payload.intent_id)?;
    Ok(DecodedDeliveryIntentTerminalV1 {
        envelope_message_id: envelope
            .message_id
            .as_slice()
            .try_into()
            .expect("validated message ID"),
        envelope_sha256,
        command_message_id,
        intent_id,
        logical_owner_id: payload.logical_owner_id,
        outcome: DeliveryIntentTerminalOutcomeV1::Succeeded {
            provider_operation_id: payload.provider_operation_id,
        },
    })
}

pub fn decode_rejected_v1(
    exact_bytes: &[u8],
) -> Result<DecodedDeliveryIntentTerminalV1, DeliveryIntentEventAdapterErrorV1> {
    let (envelope, command_message_id, envelope_sha256) = decode_result_envelope_v1(
        exact_bytes,
        &telegram_delivery_intent_rejected_contract_reference_v1(),
        TELEGRAM_DELIVERY_INTENT_TARGET_MODULE_ID_V1,
        ResultOutcomeV1::Rejected,
    )?;
    let payload = TelegramDeliveryIntentRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| DeliveryIntentEventAdapterErrorV1::InvalidPayload)?;
    validate_telegram_delivery_intent_rejected_v1(&payload)
        .map_err(|_| DeliveryIntentEventAdapterErrorV1::InvalidPayload)?;
    let intent_id = validate_result_identity_v1(&envelope, &payload.intent_id)?;
    Ok(DecodedDeliveryIntentTerminalV1 {
        envelope_message_id: envelope
            .message_id
            .as_slice()
            .try_into()
            .expect("validated message ID"),
        envelope_sha256,
        command_message_id,
        intent_id,
        logical_owner_id: payload.logical_owner_id,
        outcome: DeliveryIntentTerminalOutcomeV1::Rejected {
            rejection_code: payload
                .code
                .try_into()
                .map_err(|_| DeliveryIntentEventAdapterErrorV1::InvalidPayload)?,
        },
    })
}
