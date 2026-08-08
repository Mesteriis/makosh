use makosh_communication_cross_channel_forward_api::COMMUNICATION_CROSS_CHANNEL_FORWARD_MODULE_ID_V1;
use makosh_communication_cross_channel_forward_persistence::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardBlobReceiptV1,
    CrossChannelForwardPersistenceErrorV1, CrossChannelForwardPreparedEventV1,
    CrossChannelForwardRejectedEventV1,
};
use makosh_communication_delivery_intent_ingress_api::{
    CommunicationDeliveryIntentIngressEnvelopeContextV1,
    build_communication_delivery_intent_submit_outbox_record_v1,
    wire::DeliveryIntentBodySourceReceiptV1,
};
use makosh_communications_cross_channel_forward_source_api::{
    cross_channel_forward_source_prepared_contract_reference_v1,
    cross_channel_forward_source_rejected_contract_reference_v1,
    wire::{CrossChannelForwardSourcePreparedV1, CrossChannelForwardSourceRejectedV1},
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryErrorV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ContractRefV1, ResultOutcomeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use prost::Message;

use crate::{CrossChannelForwardBlobPortV1, CrossChannelForwardBlobTransferErrorV1};

const COMMUNICATIONS_RUNTIME_MODULE_ID_V1: &str = "makosh-communications-runtime";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardSourceResultErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Blob(CrossChannelForwardBlobTransferErrorV1),
    Persistence(CrossChannelForwardPersistenceErrorV1),
    EventUnavailable,
}

pub struct CrossChannelForwardSourceConsumerContextV1<'a> {
    pub expected_logical_owner_id: &'a str,
    pub runtime_instance_id: &'a str,
    pub runtime_generation: u64,
    pub consumed_at_unix_millis: i64,
}

pub async fn consume_source_prepared_once_v1(
    persistence: &CommunicationCrossChannelForwardPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    blob_port: &mut dyn CrossChannelForwardBlobPortV1,
    context: &CrossChannelForwardSourceConsumerContextV1<'_>,
) -> Result<bool, CrossChannelForwardSourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CrossChannelForwardSourceResultErrorV1::InvalidEnvelope)?;
    let prepared = decode_prepared(&record, context.expected_logical_owner_id)?;
    let status = persistence
        .status(context.expected_logical_owner_id, &prepared.forward_id)
        .await
        .map_err(CrossChannelForwardSourceResultErrorV1::Persistence)?;
    if status.source_message_id != prepared.source_message_id
        || status.target_conversation_id != prepared.target_conversation_id
    {
        return Err(CrossChannelForwardSourceResultErrorV1::InvalidPayload);
    }
    let materialized = blob_port
        .transfer_to_delivery_intent(&prepared)
        .map_err(CrossChannelForwardSourceResultErrorV1::Blob)?;
    let prepared = CrossChannelForwardPreparedEventV1 {
        source_body: materialized.source_body,
        ..prepared
    };
    let delivery_body = materialized.delivery_body;
    let envelope_context = delivery_context(
        context.runtime_instance_id,
        context.runtime_generation,
        context.consumed_at_unix_millis,
    )?;
    let deadline = envelope_context
        .recorded_at_unix_seconds
        .checked_add(300)
        .ok_or(CrossChannelForwardSourceResultErrorV1::InvalidPayload)?;
    let delivery_submit = build_communication_delivery_intent_submit_outbox_record_v1(
        prepared.forward_id,
        prepared.target_conversation_id,
        status.target_reply_to_message_id,
        DeliveryIntentBodySourceReceiptV1 {
            reference_id: delivery_body.reference_id.to_vec(),
            declared_bytes: delivery_body.declared_bytes,
            sha256: delivery_body.sha256.to_vec(),
            custody_transfer_source_proof: delivery_body.custody_transfer_source_proof.clone(),
        },
        &prepared.logical_owner_id,
        deadline,
        &envelope_context,
    )
    .map_err(|_| CrossChannelForwardSourceResultErrorV1::InvalidPayload)?;
    persistence
        .persist_source_prepared_and_delivery_submit(
            &prepared,
            &delivery_body,
            &delivery_submit,
            context.consumed_at_unix_millis,
        )
        .await
        .map_err(CrossChannelForwardSourceResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

pub async fn consume_source_rejected_once_v1(
    persistence: &CommunicationCrossChannelForwardPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    expected_logical_owner_id: &str,
    consumed_at_unix_millis: i64,
) -> Result<bool, CrossChannelForwardSourceResultErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(event_error)?;
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| CrossChannelForwardSourceResultErrorV1::InvalidEnvelope)?;
    let rejected = decode_rejected(&record, expected_logical_owner_id)?;
    persistence
        .persist_source_rejected(&rejected, consumed_at_unix_millis)
        .await
        .map_err(CrossChannelForwardSourceResultErrorV1::Persistence)?;
    delivery.acknowledge().await.map_err(event_error)?;
    Ok(true)
}

fn decode_prepared(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<CrossChannelForwardPreparedEventV1, CrossChannelForwardSourceResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CrossChannelForwardSourceResultErrorV1::InvalidEnvelope)?;
    let command_id = validate_result_envelope(
        &envelope.contract,
        &cross_channel_forward_source_prepared_contract_reference_v1(),
        envelope
            .source
            .as_ref()
            .map(|source| source.module_id.as_str()),
        envelope
            .source
            .as_ref()
            .map_or(0, |source| source.runtime_generation),
        envelope.semantics.as_ref(),
        ResultOutcomeV1::Succeeded,
    )?;
    let payload = CrossChannelForwardSourcePreparedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CrossChannelForwardSourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != expected_logical_owner_id
        || payload.forward_id.as_slice() != command_id
    {
        return Err(CrossChannelForwardSourceResultErrorV1::InvalidPayload);
    }
    let source_body = payload
        .body_source
        .ok_or(CrossChannelForwardSourceResultErrorV1::InvalidPayload)?;
    Ok(CrossChannelForwardPreparedEventV1 {
        result_message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        logical_owner_id: payload.logical_owner_id,
        forward_id: id16(&payload.forward_id)?,
        source_message_id: id16(&payload.source_message_id)?,
        target_conversation_id: id16(&payload.target_conversation_id)?,
        source_evidence_id: id16(&payload.source_evidence_id)?,
        source_evidence_revision: payload.source_evidence_revision,
        source_body: CrossChannelForwardBlobReceiptV1 {
            reference_id: id16(&source_body.reference_id)?,
            declared_bytes: source_body.declared_bytes,
            sha256: sha256(&source_body.sha256)?,
            custody_transfer_source_proof: source_body.custody_transfer_source_proof,
        },
    })
}

fn decode_rejected(
    record: &OutboxRecordV1,
    expected_logical_owner_id: &str,
) -> Result<CrossChannelForwardRejectedEventV1, CrossChannelForwardSourceResultErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| CrossChannelForwardSourceResultErrorV1::InvalidEnvelope)?;
    let command_id = validate_result_envelope(
        &envelope.contract,
        &cross_channel_forward_source_rejected_contract_reference_v1(),
        envelope
            .source
            .as_ref()
            .map(|source| source.module_id.as_str()),
        envelope
            .source
            .as_ref()
            .map_or(0, |source| source.runtime_generation),
        envelope.semantics.as_ref(),
        ResultOutcomeV1::Rejected,
    )?;
    let payload = CrossChannelForwardSourceRejectedV1::decode(envelope.payload.as_slice())
        .map_err(|_| CrossChannelForwardSourceResultErrorV1::InvalidPayload)?;
    let rejection_code = u16::try_from(payload.code)
        .map_err(|_| CrossChannelForwardSourceResultErrorV1::InvalidPayload)?;
    if payload.logical_owner_id != expected_logical_owner_id
        || payload.forward_id.as_slice() != command_id
        || !(1..=7).contains(&rejection_code)
    {
        return Err(CrossChannelForwardSourceResultErrorV1::InvalidPayload);
    }
    Ok(CrossChannelForwardRejectedEventV1 {
        result_message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        logical_owner_id: payload.logical_owner_id,
        forward_id: id16(&payload.forward_id)?,
        rejection_code,
    })
}

fn validate_result_envelope(
    actual_contract: &Option<ContractRefV1>,
    expected_contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    source_module_id: Option<&str>,
    source_runtime_generation: u64,
    semantics: Option<&Semantics>,
    expected_outcome: ResultOutcomeV1,
) -> Result<[u8; 16], CrossChannelForwardSourceResultErrorV1> {
    if !exact_contract(actual_contract.as_ref(), expected_contract)
        || source_module_id != Some(COMMUNICATIONS_RUNTIME_MODULE_ID_V1)
        || source_runtime_generation == 0
    {
        return Err(CrossChannelForwardSourceResultErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Result(result)) = semantics else {
        return Err(CrossChannelForwardSourceResultErrorV1::InvalidEnvelope);
    };
    if result.command_id.len() != 16
        || result.command_message_id.as_slice() != result.command_id
        || result.outcome != expected_outcome as i32
        || result.execution_attempt == 0
    {
        return Err(CrossChannelForwardSourceResultErrorV1::InvalidEnvelope);
    }
    id16(&result.command_id)
}

fn delivery_context(
    runtime_instance_id: &str,
    runtime_generation: u64,
    unix_millis: i64,
) -> Result<
    CommunicationDeliveryIntentIngressEnvelopeContextV1,
    CrossChannelForwardSourceResultErrorV1,
> {
    if runtime_instance_id.is_empty() || runtime_generation == 0 || unix_millis <= 0 {
        return Err(CrossChannelForwardSourceResultErrorV1::InvalidPayload);
    }
    Ok(CommunicationDeliveryIntentIngressEnvelopeContextV1 {
        module_id: COMMUNICATION_CROSS_CHANNEL_FORWARD_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime_instance_id.to_owned(),
        runtime_generation,
        recorded_at_unix_seconds: unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((unix_millis % 1_000) * 1_000_000)
            .map_err(|_| CrossChannelForwardSourceResultErrorV1::InvalidPayload)?,
    })
}

fn exact_contract(
    actual: Option<&ContractRefV1>,
    expected: &makosh_runtime_protocol::v1::ContractReferenceV1,
) -> bool {
    actual.is_some_and(|actual| {
        actual.owner == expected.owner
            && actual.name == expected.name
            && actual.major == expected.major
            && actual.revision == expected.revision
    })
}

fn id16(value: &[u8]) -> Result<[u8; 16], CrossChannelForwardSourceResultErrorV1> {
    value
        .try_into()
        .map_err(|_| CrossChannelForwardSourceResultErrorV1::InvalidPayload)
}

fn sha256(value: &[u8]) -> Result<[u8; 32], CrossChannelForwardSourceResultErrorV1> {
    value
        .try_into()
        .map_err(|_| CrossChannelForwardSourceResultErrorV1::InvalidPayload)
}

fn event_error(_: RuntimePullDeliveryErrorV1) -> CrossChannelForwardSourceResultErrorV1 {
    CrossChannelForwardSourceResultErrorV1::EventUnavailable
}

#[cfg(test)]
mod tests {
    use super::{decode_prepared, decode_rejected};
    use makosh_communications_cross_channel_forward_source_api::{
        CrossChannelForwardSourceEnvelopeContextV1,
        build_cross_channel_forward_source_prepared_outbox_record_v1,
        build_cross_channel_forward_source_rejected_outbox_record_v1,
        wire::{
            CrossChannelForwardBodySourceReceiptV1, CrossChannelForwardSourcePreparedV1,
            CrossChannelForwardSourceRejectCodeV1, CrossChannelForwardSourceRejectedV1,
        },
    };

    fn context(module_id: &str) -> CrossChannelForwardSourceEnvelopeContextV1 {
        CrossChannelForwardSourceEnvelopeContextV1 {
            module_id: module_id.to_owned(),
            runtime_instance_id: "communications-runtime-1".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 0,
        }
    }

    #[test]
    fn source_results_require_exact_communications_runtime_and_correlation() {
        let prepared = build_cross_channel_forward_source_prepared_outbox_record_v1(
            [1; 16],
            CrossChannelForwardSourcePreparedV1 {
                forward_id: vec![1; 16],
                source_message_id: vec![2; 16],
                target_conversation_id: vec![3; 16],
                source_evidence_id: vec![4; 16],
                source_evidence_revision: 1,
                body_source: Some(CrossChannelForwardBodySourceReceiptV1 {
                    reference_id: vec![5; 16],
                    declared_bytes: 5,
                    sha256: vec![6; 32],
                    custody_transfer_source_proof: vec![7; 48],
                }),
                logical_owner_id: "owner-1".to_owned(),
            },
            &context("makosh-communications-runtime"),
        )
        .expect("prepared");
        assert!(decode_prepared(&prepared, "owner-1").is_ok());

        let wrong_source = build_cross_channel_forward_source_rejected_outbox_record_v1(
            [8; 16],
            CrossChannelForwardSourceRejectedV1 {
                forward_id: vec![8; 16],
                code:
                    CrossChannelForwardSourceRejectCodeV1::CrossChannelForwardSourceRejectCodePolicy
                        as i32,
                logical_owner_id: "owner-1".to_owned(),
            },
            &context("makosh-not-communications-runtime"),
        )
        .expect("rejected");
        assert!(decode_rejected(&wrong_source, "owner-1").is_err());
    }
}
