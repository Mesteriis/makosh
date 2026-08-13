use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, EventMetadataV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::{
    DECISIONS_MODULE_ID_V1, client_wire::DecisionChangedV1,
    decisions_lifecycle_event_contract_reference_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionsEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionsEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_decision_changed_outbox_record_v1(
    operation_id: [u8; 16],
    payload: DecisionChangedV1,
    context: &DecisionsEnvelopeContextV1,
) -> Result<OutboxRecordV1, DecisionsEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let event_id = id16(&payload.event_id)?;
    let decision_id = id16(&payload.decision_id)?;
    let occurred_at = payload
        .occurred_at
        .as_ref()
        .filter(|value| value.unix_seconds > 0 && (0..1_000_000_000).contains(&value.nanos))
        .ok_or(DecisionsEnvelopeBuildErrorV1::InvalidPayload)?;
    if !nonzero(&operation_id)
        || payload.decision_revision == 0
        || !(1..=4).contains(&payload.state)
        || !valid_owner(&payload.logical_owner_id)
    {
        return Err(DecisionsEnvelopeBuildErrorV1::InvalidPayload);
    }
    let contract = decisions_lifecycle_event_contract_reference_v1();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: event_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: contract.owner,
            name: contract.name,
            major: contract.major,
            revision: contract.revision,
            schema_sha256: contract.schema_sha256,
        }),
        source: Some(SourceRefV1 {
            module_id: DECISIONS_MODULE_ID_V1.to_owned(),
            runtime_instance_id: digest(
                b"decisions-runtime-instance-v1",
                context.runtime_instance_id.as_bytes(),
                b"source",
            )
            .to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(Timestamp {
            seconds: context.recorded_at_unix_seconds,
            nanos: context.recorded_at_nanos,
        }),
        partition_key: decision_id.to_vec(),
        causation_message_id: operation_id.to_vec(),
        correlation_id: decision_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: DECISIONS_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: DECISIONS_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Event(EventMetadataV1 {
            occurred_at: Some(Timestamp {
                seconds: occurred_at.unix_seconds,
                nanos: occurred_at.nanos,
            }),
        })),
        payload: payload.encode_to_vec(),
    };
    validate_envelope_v1(&envelope).map_err(|_| DecisionsEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(
    context: &DecisionsEnvelopeContextV1,
) -> Result<(), DecisionsEnvelopeBuildErrorV1> {
    if context.module_id != DECISIONS_MODULE_ID_V1
        || context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 128
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(DecisionsEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

fn id16(bytes: &[u8]) -> Result<[u8; 16], DecisionsEnvelopeBuildErrorV1> {
    let value: [u8; 16] = bytes
        .try_into()
        .map_err(|_| DecisionsEnvelopeBuildErrorV1::InvalidPayload)?;
    nonzero(&value)
        .then_some(value)
        .ok_or(DecisionsEnvelopeBuildErrorV1::InvalidPayload)
}

fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn digest(domain: &[u8], left: &[u8], right: &[u8]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update((left.len() as u64).to_be_bytes());
    hash.update(left);
    hash.update((right.len() as u64).to_be_bytes());
    hash.update(right);
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn outbox_error(_: OutboxRecordError) -> DecisionsEnvelopeBuildErrorV1 {
    DecisionsEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_wire::{DecisionChangedV1, DecisionStateV1, TimestampV1};

    #[test]
    fn lifecycle_event_excludes_private_decision_content() {
        let record = build_decision_changed_outbox_record_v1(
            [1; 16],
            DecisionChangedV1 {
                event_id: vec![2; 16],
                decision_id: vec![3; 16],
                logical_owner_id: "owner-1".to_owned(),
                decision_revision: 2,
                state: DecisionStateV1::DecisionStateDecided as i32,
                occurred_at: Some(TimestampV1 {
                    unix_seconds: 10,
                    nanos: 0,
                }),
            },
            &DecisionsEnvelopeContextV1 {
                module_id: DECISIONS_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "decisions-runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 10,
                recorded_at_nanos: 0,
            },
        )
        .expect("event");
        for private in [
            b"Private rationale".as_slice(),
            b"private-evidence".as_slice(),
        ] {
            assert!(
                !record
                    .exact_bytes()
                    .windows(private.len())
                    .any(|window| window == private)
            );
        }
    }
}
