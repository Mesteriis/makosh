use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        ResultMetadataV1, ResultOutcomeV1, SourceFenceV1, SourceRefV1,
        durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_SCHEMA_SHA256,
    CROSS_CHANNEL_FORWARD_SOURCE_COMMAND_CAPABILITY_ID_V1,
    CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_MAJOR_V1,
    CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_REVISION_V1, CROSS_CHANNEL_FORWARD_SOURCE_MAX_BYTES_V1,
    CROSS_CHANNEL_FORWARD_SOURCE_MAX_PROOF_BYTES_V1, CROSS_CHANNEL_FORWARD_SOURCE_OWNER_V1,
    CROSS_CHANNEL_FORWARD_SOURCE_PREPARE_CONTRACT_NAME_V1,
    CROSS_CHANNEL_FORWARD_SOURCE_PREPARED_CONTRACT_NAME_V1,
    CROSS_CHANNEL_FORWARD_SOURCE_REJECTED_CONTRACT_NAME_V1,
    wire::{
        CrossChannelForwardBodySourceReceiptV1, CrossChannelForwardSourcePreparedV1,
        CrossChannelForwardSourceRejectedV1, PrepareCrossChannelForwardSourceCommandV1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardSourceEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChannelForwardSourceEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_cross_channel_forward_source_prepare_outbox_record_v1(
    forward_id: [u8; 16],
    source_message_id: [u8; 16],
    target_conversation_id: [u8; 16],
    logical_owner_id: &str,
    deadline_unix_seconds: i64,
    context: &CrossChannelForwardSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CrossChannelForwardSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    validate_common_payload(
        &forward_id,
        &source_message_id,
        &target_conversation_id,
        logical_owner_id,
    )?;
    if deadline_unix_seconds <= context.recorded_at_unix_seconds {
        return Err(CrossChannelForwardSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    let payload = PrepareCrossChannelForwardSourceCommandV1 {
        forward_id: forward_id.to_vec(),
        source_message_id: source_message_id.to_vec(),
        target_conversation_id: target_conversation_id.to_vec(),
        logical_owner_id: logical_owner_id.to_owned(),
    }
    .encode_to_vec();
    build_envelope(
        forward_id,
        &forward_id,
        &[],
        CROSS_CHANNEL_FORWARD_SOURCE_PREPARE_CONTRACT_NAME_V1,
        Semantics::Command(CommandMetadataV1 {
            command_id: forward_id.to_vec(),
            target_capability: CROSS_CHANNEL_FORWARD_SOURCE_COMMAND_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: Sha256::digest(
                [
                    b"communications-cross-channel-forward-source-prepare-v1".as_slice(),
                    &forward_id,
                ]
                .concat(),
            )
            .to_vec(),
            deadline: Some(Timestamp {
                seconds: deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        }),
        payload,
        context,
    )
}

pub fn build_cross_channel_forward_source_prepared_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CrossChannelForwardSourcePreparedV1,
    context: &CrossChannelForwardSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CrossChannelForwardSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let forward_id = validate_prepared_payload(&payload)?;
    if !valid_id(&command_message_id) {
        return Err(CrossChannelForwardSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_envelope(
        result_message_id(b"prepared", &forward_id),
        &forward_id,
        &command_message_id,
        CROSS_CHANNEL_FORWARD_SOURCE_PREPARED_CONTRACT_NAME_V1,
        result_semantics(
            &forward_id,
            &command_message_id,
            ResultOutcomeV1::Succeeded,
            context,
        ),
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_cross_channel_forward_source_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    payload: CrossChannelForwardSourceRejectedV1,
    context: &CrossChannelForwardSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CrossChannelForwardSourceEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let forward_id = id16(&payload.forward_id)?;
    if !valid_id(&command_message_id)
        || !valid_logical_owner_id(&payload.logical_owner_id)
        || payload.code == 0
    {
        return Err(CrossChannelForwardSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_envelope(
        result_message_id(b"rejected", &forward_id),
        &forward_id,
        &command_message_id,
        CROSS_CHANNEL_FORWARD_SOURCE_REJECTED_CONTRACT_NAME_V1,
        result_semantics(
            &forward_id,
            &command_message_id,
            ResultOutcomeV1::Rejected,
            context,
        ),
        payload.encode_to_vec(),
        context,
    )
}

fn build_envelope(
    message_id: [u8; 16],
    partition_key: &[u8; 16],
    causation_message_id: &[u8],
    contract_name: &str,
    semantics: Semantics,
    payload: Vec<u8>,
    context: &CrossChannelForwardSourceEnvelopeContextV1,
) -> Result<OutboxRecordV1, CrossChannelForwardSourceEnvelopeBuildErrorV1> {
    let timestamp = Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    };
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: CROSS_CHANNEL_FORWARD_SOURCE_OWNER_V1.to_owned(),
            name: contract_name.to_owned(),
            major: CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_MAJOR_V1,
            revision: CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_SCHEMA_SHA256.to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: runtime_source_reference(&context.runtime_instance_id).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp),
        partition_key: partition_key.to_vec(),
        causation_message_id: causation_message_id.to_vec(),
        correlation_id: partition_key.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: context.module_id.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: context.module_id.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(semantics),
        payload,
    };
    validate_envelope_v1(&envelope)
        .map_err(|_| CrossChannelForwardSourceEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn result_semantics(
    command_id: &[u8; 16],
    command_message_id: &[u8; 16],
    outcome: ResultOutcomeV1,
    context: &CrossChannelForwardSourceEnvelopeContextV1,
) -> Semantics {
    Semantics::Result(ResultMetadataV1 {
        command_id: command_id.to_vec(),
        command_message_id: command_message_id.to_vec(),
        outcome: outcome as i32,
        completed_at: Some(Timestamp {
            seconds: context.recorded_at_unix_seconds,
            nanos: context.recorded_at_nanos,
        }),
        execution_attempt: 1,
    })
}

fn validate_context(
    context: &CrossChannelForwardSourceEnvelopeContextV1,
) -> Result<(), CrossChannelForwardSourceEnvelopeBuildErrorV1> {
    if context.module_id.is_empty()
        || context.module_id.len() > 128
        || !context.module_id.is_ascii()
        || context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 256
        || !context.runtime_instance_id.is_ascii()
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(CrossChannelForwardSourceEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn validate_prepared_payload(
    payload: &CrossChannelForwardSourcePreparedV1,
) -> Result<[u8; 16], CrossChannelForwardSourceEnvelopeBuildErrorV1> {
    let forward_id = id16(&payload.forward_id)?;
    let source_message_id = id16(&payload.source_message_id)?;
    let target_conversation_id = id16(&payload.target_conversation_id)?;
    validate_common_payload(
        &forward_id,
        &source_message_id,
        &target_conversation_id,
        &payload.logical_owner_id,
    )?;
    id16(&payload.source_evidence_id)?;
    if payload.source_evidence_revision == 0
        || payload
            .body_source
            .as_ref()
            .is_none_or(|receipt| !valid_source_receipt(receipt))
    {
        return Err(CrossChannelForwardSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(forward_id)
}

fn validate_common_payload(
    forward_id: &[u8; 16],
    source_message_id: &[u8; 16],
    target_conversation_id: &[u8; 16],
    logical_owner_id: &str,
) -> Result<(), CrossChannelForwardSourceEnvelopeBuildErrorV1> {
    if !valid_id(forward_id)
        || !valid_id(source_message_id)
        || !valid_id(target_conversation_id)
        || !valid_logical_owner_id(logical_owner_id)
    {
        return Err(CrossChannelForwardSourceEnvelopeBuildErrorV1::InvalidPayload);
    }
    Ok(())
}

fn valid_source_receipt(receipt: &CrossChannelForwardBodySourceReceiptV1) -> bool {
    receipt.reference_id.len() == 16
        && receipt.reference_id.iter().any(|byte| *byte != 0)
        && (1..=CROSS_CHANNEL_FORWARD_SOURCE_MAX_BYTES_V1).contains(&receipt.declared_bytes)
        && receipt.sha256.len() == 32
        && receipt.sha256.iter().any(|byte| *byte != 0)
        && !receipt.custody_transfer_source_proof.is_empty()
        && receipt.custody_transfer_source_proof.len()
            <= CROSS_CHANNEL_FORWARD_SOURCE_MAX_PROOF_BYTES_V1
}

fn valid_id(id: &[u8; 16]) -> bool {
    id.iter().any(|byte| *byte != 0)
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn id16(bytes: &[u8]) -> Result<[u8; 16], CrossChannelForwardSourceEnvelopeBuildErrorV1> {
    bytes
        .try_into()
        .ok()
        .filter(valid_id)
        .ok_or(CrossChannelForwardSourceEnvelopeBuildErrorV1::InvalidPayload)
}

fn result_message_id(label: &[u8], forward_id: &[u8; 16]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"communications-cross-channel-forward-source-result-v1");
    hasher.update(label);
    hasher.update(forward_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    Sha256::digest(runtime_instance_id.as_bytes())[..16]
        .try_into()
        .expect("digest prefix")
}

fn outbox_error(_: OutboxRecordError) -> CrossChannelForwardSourceEnvelopeBuildErrorV1 {
    CrossChannelForwardSourceEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::DurableEnvelopeV1;

    fn context() -> CrossChannelForwardSourceEnvelopeContextV1 {
        CrossChannelForwardSourceEnvelopeContextV1 {
            module_id: "makosh-communication-cross-channel-forward-runtime".to_owned(),
            runtime_instance_id: "runtime-forward-1".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 12,
        }
    }

    fn prepared_payload() -> CrossChannelForwardSourcePreparedV1 {
        CrossChannelForwardSourcePreparedV1 {
            forward_id: vec![1; 16],
            source_message_id: vec![2; 16],
            target_conversation_id: vec![3; 16],
            source_evidence_id: vec![4; 16],
            source_evidence_revision: 9,
            body_source: Some(CrossChannelForwardBodySourceReceiptV1 {
                reference_id: vec![5; 16],
                declared_bytes: 42,
                sha256: vec![6; 32],
                custody_transfer_source_proof: vec![7; 64],
            }),
            logical_owner_id: "owner-1".to_owned(),
        }
    }

    #[test]
    fn command_contains_only_canonical_ids_and_exact_capability() {
        let record = build_cross_channel_forward_source_prepare_outbox_record_v1(
            [1; 16],
            [2; 16],
            [3; 16],
            "owner-1",
            1_800_000_030,
            &context(),
        )
        .expect("command");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        let Some(Semantics::Command(command)) = envelope.semantics else {
            panic!("command semantics");
        };
        assert_eq!(
            command.target_capability,
            CROSS_CHANNEL_FORWARD_SOURCE_COMMAND_CAPABILITY_ID_V1
        );
        let payload =
            PrepareCrossChannelForwardSourceCommandV1::decode(envelope.payload.as_slice())
                .expect("payload");
        assert_eq!(payload.source_message_id, vec![2; 16]);
        assert_eq!(payload.target_conversation_id, vec![3; 16]);
        assert_eq!(payload.logical_owner_id, "owner-1");
    }

    #[test]
    fn prepared_result_requires_a_bounded_target_bound_receipt() {
        let record = build_cross_channel_forward_source_prepared_outbox_record_v1(
            [8; 16],
            prepared_payload(),
            &context(),
        )
        .expect("prepared result");
        let envelope = DurableEnvelopeV1::decode(record.exact_bytes()).expect("envelope");
        assert!(matches!(envelope.semantics, Some(Semantics::Result(_))));

        let mut invalid = prepared_payload();
        invalid
            .body_source
            .as_mut()
            .expect("receipt")
            .declared_bytes = CROSS_CHANNEL_FORWARD_SOURCE_MAX_BYTES_V1 + 1;
        assert_eq!(
            build_cross_channel_forward_source_prepared_outbox_record_v1(
                [8; 16],
                invalid,
                &context(),
            ),
            Err(CrossChannelForwardSourceEnvelopeBuildErrorV1::InvalidPayload)
        );
    }
}
