use makosh_communication_cross_channel_forward_persistence::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardDeliveryRejectedEventV1,
    CrossChannelForwardDeliverySubmittedEventV1, CrossChannelForwardPersistenceErrorV1,
};
use makosh_communication_delivery_intent_ingress_api::{
    COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_MODULE_ID_V1,
    communication_delivery_intent_rejected_contract_reference_v1,
    communication_delivery_intent_rejected_message_id_v1,
    communication_delivery_intent_submit_message_id_v1,
    communication_delivery_intent_submitted_contract_reference_v1,
    communication_delivery_intent_submitted_message_id_v1,
    wire::{
        CommunicationDeliveryIntentIngressRejectCodeV1, CommunicationDeliveryIntentRejectedV1,
        CommunicationDeliveryIntentSubmittedV1,
    },
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        ActorKindV1, ContractRefV1, FenceKindV1, ResultOutcomeV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardDeliveryResultErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(CrossChannelForwardPersistenceErrorV1),
    EventUnavailable,
}

pub async fn consume_delivery_submitted_once_v1(
    persistence: &CommunicationCrossChannelForwardPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    expected_logical_owner_id: &str,
    consumed_at_unix_millis: i64,
) -> Result<bool, CrossChannelForwardDeliveryResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope)?;
    let event = decode_submitted(&record, expected_logical_owner_id)?;
    persistence
        .persist_delivery_submitted(&event, consumed_at_unix_millis)
        .await
        .map_err(CrossChannelForwardDeliveryResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

pub async fn consume_delivery_rejected_once_v1(
    persistence: &CommunicationCrossChannelForwardPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    expected_logical_owner_id: &str,
    consumed_at_unix_millis: i64,
) -> Result<bool, CrossChannelForwardDeliveryResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope)?;
    let event = decode_rejected(&record, expected_logical_owner_id)?;
    persistence
        .persist_delivery_rejected(&event, consumed_at_unix_millis)
        .await
        .map_err(CrossChannelForwardDeliveryResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

fn decode_submitted(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<CrossChannelForwardDeliverySubmittedEventV1, CrossChannelForwardDeliveryResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope)?;
    let (intent_id, command_message_id) = validate_result_envelope(
        record,
        &envelope,
        &communication_delivery_intent_submitted_contract_reference_v1(),
        ResultOutcomeV1::Succeeded,
    )?;
    let payload = CommunicationDeliveryIntentSubmittedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CrossChannelForwardDeliveryResultErrorV1::InvalidPayload)?;
    if payload.intent_id.as_slice() != intent_id
        || payload.logical_owner_id != expected_logical_owner_id
    {
        return Err(CrossChannelForwardDeliveryResultErrorV1::InvalidPayload);
    }
    Ok(CrossChannelForwardDeliverySubmittedEventV1 {
        result_message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        logical_owner_id: payload.logical_owner_id,
        delivery_intent_id: intent_id,
        delivery_submit_message_id: command_message_id,
    })
}

fn decode_rejected(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<CrossChannelForwardDeliveryRejectedEventV1, CrossChannelForwardDeliveryResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope)?;
    let (intent_id, command_message_id) = validate_result_envelope(
        record,
        &envelope,
        &communication_delivery_intent_rejected_contract_reference_v1(),
        ResultOutcomeV1::Rejected,
    )?;
    let payload = CommunicationDeliveryIntentRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CrossChannelForwardDeliveryResultErrorV1::InvalidPayload)?;
    let rejection_code = CommunicationDeliveryIntentIngressRejectCodeV1::try_from(payload.code)
        .ok()
        .filter(|code| {
            *code
                != CommunicationDeliveryIntentIngressRejectCodeV1::
                    CommunicationDeliveryIntentIngressRejectCodeUnspecified
        })
        .and_then(|code| u16::try_from(code as i32).ok())
        .ok_or(CrossChannelForwardDeliveryResultErrorV1::InvalidPayload)?;
    if payload.intent_id.as_slice() != intent_id
        || payload.logical_owner_id != expected_logical_owner_id
    {
        return Err(CrossChannelForwardDeliveryResultErrorV1::InvalidPayload);
    }
    Ok(CrossChannelForwardDeliveryRejectedEventV1 {
        result_message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        logical_owner_id: payload.logical_owner_id,
        delivery_intent_id: intent_id,
        delivery_submit_message_id: command_message_id,
        rejection_code,
    })
}

fn validate_result_envelope(
    record: &OutboxRecordV1,
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
    expected_contract: &ContractReferenceV1,
    expected_outcome: ResultOutcomeV1,
) -> Result<([u8; 16], [u8; 16]), CrossChannelForwardDeliveryResultErrorV1> {
    let source = envelope
        .source
        .as_ref()
        .ok_or(CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope)?;
    let recorded_at = envelope
        .recorded_at
        .as_ref()
        .ok_or(CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope)?;
    let Some(Semantics::Result(result)) = envelope.semantics.as_ref() else {
        return Err(CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope);
    };
    let intent_id = id16(&result.command_id)?;
    let command_message_id = communication_delivery_intent_submit_message_id_v1(&intent_id);
    let result_message_id = match expected_outcome {
        ResultOutcomeV1::Succeeded => {
            communication_delivery_intent_submitted_message_id_v1(&intent_id)
        }
        ResultOutcomeV1::Rejected => {
            communication_delivery_intent_rejected_message_id_v1(&intent_id)
        }
        _ => return Err(CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope),
    };
    let completed_at = result
        .completed_at
        .as_ref()
        .ok_or(CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope)?;
    if !exact_contract(envelope.contract.as_ref(), expected_contract)
        || source.module_id != COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_MODULE_ID_V1
        || source.runtime_instance_id.len() != 16
        || source.runtime_instance_id.iter().all(|byte| *byte == 0)
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id.as_slice()
            != COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_MODULE_ID_V1.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id.as_slice()
            != COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
        || recorded_at.seconds <= 0
        || !(0..1_000_000_000).contains(&recorded_at.nanos)
        || completed_at != recorded_at
        || result.command_message_id.as_slice() != command_message_id
        || result.outcome != expected_outcome as i32
        || result.execution_attempt != 1
        || envelope.partition_key.as_slice() != intent_id
        || envelope.correlation_id.as_slice() != intent_id
        || envelope.causation_message_id.as_slice() != command_message_id
        || envelope.message_id.as_slice() != result_message_id
        || envelope.message_id.as_slice() != record.message_id()
    {
        return Err(CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope);
    }
    Ok((intent_id, command_message_id))
}

fn exact_contract(actual: Option<&ContractRefV1>, expected: &ContractReferenceV1) -> bool {
    actual.is_some_and(|actual| {
        actual.owner == expected.owner
            && actual.name == expected.name
            && actual.major == expected.major
            && actual.revision == expected.revision
            && actual.schema_sha256 == expected.schema_sha256
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], CrossChannelForwardDeliveryResultErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(CrossChannelForwardDeliveryResultErrorV1::InvalidEnvelope)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> CrossChannelForwardDeliveryResultErrorV1 {
    CrossChannelForwardDeliveryResultErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_communication_delivery_intent_ingress_api::{
        CommunicationDeliveryIntentIngressEnvelopeContextV1,
        build_communication_delivery_intent_rejected_outbox_record_v1,
        build_communication_delivery_intent_submitted_outbox_record_v1,
    };

    fn context(module_id: &str) -> CommunicationDeliveryIntentIngressEnvelopeContextV1 {
        CommunicationDeliveryIntentIngressEnvelopeContextV1 {
            module_id: module_id.to_owned(),
            runtime_instance_id: "delivery-runtime-1".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 0,
        }
    }

    #[test]
    fn delivery_results_require_exact_runtime_owner_and_closed_rejection() {
        let submitted = build_communication_delivery_intent_submitted_outbox_record_v1(
            communication_delivery_intent_submit_message_id_v1(&[1; 16]),
            CommunicationDeliveryIntentSubmittedV1 {
                intent_id: vec![1; 16],
                logical_owner_id: "owner-1".to_owned(),
            },
            &context(COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_MODULE_ID_V1),
        )
        .expect("submitted");
        assert!(decode_submitted(&submitted, "owner-1").is_ok());

        let wrong_source = build_communication_delivery_intent_rejected_outbox_record_v1(
            communication_delivery_intent_submit_message_id_v1(&[2; 16]),
            CommunicationDeliveryIntentRejectedV1 {
                intent_id: vec![2; 16],
                code: CommunicationDeliveryIntentIngressRejectCodeV1::
                    CommunicationDeliveryIntentIngressRejectCodePolicy as i32,
                logical_owner_id: "owner-1".to_owned(),
            },
            &context("makosh-not-delivery-intent-runtime"),
        )
        .expect("rejected");
        assert!(decode_rejected(&wrong_source, "owner-1").is_err());
    }
}
