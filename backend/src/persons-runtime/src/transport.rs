use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1,
        EventMetadataV1, FenceKindV1, ResultMetadataV1, ResultOutcomeV1, SourceFenceV1,
        SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use makosh_persons_api::{
    PERSONS_COMMAND_CAPABILITY_ID_V1, PERSONS_OWNER_ID_V1, persons_command_contract_reference_v1,
    persons_command_rejected_contract_reference_v1,
    persons_command_succeeded_contract_reference_v1, persons_owner_event_contract_reference_v1,
    persons_review_candidate_contract_reference_v1,
    wire::{
        PersonCommandRejectedV1, PersonCommandSucceededV1, PersonReviewCandidateRaisedEventV1,
        PersonsCommandV1, PersonsOwnerEventV1,
    },
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::command::persons_wire_command_identity_v1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonsEnvelopeContextV1 {
    pub module_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonsEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_persons_command_outbox_record_v1(
    payload: PersonsCommandV1,
    deadline_unix_seconds: i64,
    context: &PersonsEnvelopeContextV1,
) -> Result<OutboxRecordV1, PersonsEnvelopeBuildErrorV1> {
    validate_context(context)?;
    let (command_id, logical_owner_id) = persons_wire_command_identity_v1(&payload)
        .map_err(|_| PersonsEnvelopeBuildErrorV1::InvalidPayload)?;
    if deadline_unix_seconds <= context.recorded_at_unix_seconds {
        return Err(PersonsEnvelopeBuildErrorV1::InvalidPayload);
    }
    let payload_bytes = payload.encode_to_vec();
    let fingerprint = Sha256::digest(&payload_bytes);
    let partition_key = digest16(
        b"persons-owner-partition-v1",
        logical_owner_id.as_bytes(),
        PERSONS_OWNER_ID_V1.as_bytes(),
    );
    build_envelope(
        EnvelopeBuildV1 {
            message_id: command_id,
            contract: persons_command_contract_reference_v1(),
            partition_key,
            correlation_id: partition_key,
            causation_message_id: Vec::new(),
            semantics: Semantics::Command(CommandMetadataV1 {
                command_id: command_id.to_vec(),
                target_capability: PERSONS_COMMAND_CAPABILITY_ID_V1.to_owned(),
                idempotency_key: fingerprint.to_vec(),
                deadline: Some(Timestamp {
                    seconds: deadline_unix_seconds,
                    nanos: 0,
                }),
                logical_attempt: 1,
            }),
            payload: payload_bytes,
        },
        context,
    )
}

pub fn build_persons_command_succeeded_outbox_record_v1(
    command_message_id: [u8; 16],
    correlation_id: [u8; 16],
    payload: PersonCommandSucceededV1,
    context: &PersonsEnvelopeContextV1,
) -> Result<OutboxRecordV1, PersonsEnvelopeBuildErrorV1> {
    build_result(
        b"persons-command-succeeded-v1",
        command_message_id,
        correlation_id,
        id16(&payload.command_id)?,
        ResultOutcomeV1::Succeeded,
        persons_command_succeeded_contract_reference_v1(),
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_persons_command_rejected_outbox_record_v1(
    command_message_id: [u8; 16],
    correlation_id: [u8; 16],
    payload: PersonCommandRejectedV1,
    context: &PersonsEnvelopeContextV1,
) -> Result<OutboxRecordV1, PersonsEnvelopeBuildErrorV1> {
    build_result(
        b"persons-command-rejected-v1",
        command_message_id,
        correlation_id,
        id16(&payload.command_id)?,
        ResultOutcomeV1::Rejected,
        persons_command_rejected_contract_reference_v1(),
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_persons_owner_event_outbox_record_v1(
    command_message_id: [u8; 16],
    correlation_id: [u8; 16],
    event_message_id: [u8; 16],
    partition_key: [u8; 16],
    occurred_at: Timestamp,
    payload: PersonsOwnerEventV1,
    context: &PersonsEnvelopeContextV1,
) -> Result<OutboxRecordV1, PersonsEnvelopeBuildErrorV1> {
    build_event(
        command_message_id,
        correlation_id,
        event_message_id,
        partition_key,
        occurred_at,
        persons_owner_event_contract_reference_v1(),
        payload.encode_to_vec(),
        context,
    )
}

pub fn build_persons_review_candidate_outbox_record_v1(
    command_message_id: [u8; 16],
    correlation_id: [u8; 16],
    event_message_id: [u8; 16],
    partition_key: [u8; 16],
    payload: PersonReviewCandidateRaisedEventV1,
    context: &PersonsEnvelopeContextV1,
) -> Result<OutboxRecordV1, PersonsEnvelopeBuildErrorV1> {
    let occurred_at = payload
        .observed_at
        .as_ref()
        .map(|value| Timestamp {
            seconds: value.unix_seconds,
            nanos: value.nanos,
        })
        .ok_or(PersonsEnvelopeBuildErrorV1::InvalidPayload)?;
    build_event(
        command_message_id,
        correlation_id,
        event_message_id,
        partition_key,
        occurred_at,
        persons_review_candidate_contract_reference_v1(),
        payload.encode_to_vec(),
        context,
    )
}

#[must_use]
pub fn persons_command_fingerprint_v1(payload: &PersonsCommandV1) -> [u8; 32] {
    Sha256::digest(payload.encode_to_vec()).into()
}

#[must_use]
pub fn persons_deterministic_public_id_v1(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    digest16(label, first, second)
}

struct EnvelopeBuildV1 {
    message_id: [u8; 16],
    contract: ContractReferenceV1,
    partition_key: [u8; 16],
    correlation_id: [u8; 16],
    causation_message_id: Vec<u8>,
    semantics: Semantics,
    payload: Vec<u8>,
}

#[allow(clippy::too_many_arguments)]
fn build_result(
    label: &[u8],
    command_message_id: [u8; 16],
    correlation_id: [u8; 16],
    command_id: [u8; 16],
    outcome: ResultOutcomeV1,
    contract: ContractReferenceV1,
    payload: Vec<u8>,
    context: &PersonsEnvelopeContextV1,
) -> Result<OutboxRecordV1, PersonsEnvelopeBuildErrorV1> {
    validate_context(context)?;
    id16(&command_message_id)?;
    id16(&correlation_id)?;
    let message_id = digest16(label, &command_message_id, &Sha256::digest(&payload));
    build_envelope(
        EnvelopeBuildV1 {
            message_id,
            contract,
            partition_key: correlation_id,
            correlation_id,
            causation_message_id: command_message_id.to_vec(),
            semantics: Semantics::Result(ResultMetadataV1 {
                command_id: command_id.to_vec(),
                command_message_id: command_message_id.to_vec(),
                outcome: outcome as i32,
                completed_at: Some(timestamp(context)),
                execution_attempt: 1,
            }),
            payload,
        },
        context,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_event(
    command_message_id: [u8; 16],
    correlation_id: [u8; 16],
    message_id: [u8; 16],
    partition_key: [u8; 16],
    occurred_at: Timestamp,
    contract: ContractReferenceV1,
    payload: Vec<u8>,
    context: &PersonsEnvelopeContextV1,
) -> Result<OutboxRecordV1, PersonsEnvelopeBuildErrorV1> {
    validate_context(context)?;
    id16(&command_message_id)?;
    id16(&correlation_id)?;
    id16(&message_id)?;
    id16(&partition_key)?;
    if occurred_at.seconds <= 0 || !(0..1_000_000_000).contains(&occurred_at.nanos) {
        return Err(PersonsEnvelopeBuildErrorV1::InvalidPayload);
    }
    build_envelope(
        EnvelopeBuildV1 {
            message_id,
            contract,
            partition_key,
            correlation_id,
            causation_message_id: command_message_id.to_vec(),
            semantics: Semantics::Event(EventMetadataV1 {
                occurred_at: Some(occurred_at),
            }),
            payload,
        },
        context,
    )
}

fn build_envelope(
    input: EnvelopeBuildV1,
    context: &PersonsEnvelopeContextV1,
) -> Result<OutboxRecordV1, PersonsEnvelopeBuildErrorV1> {
    let contract = input.contract;
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: input.message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: contract.owner,
            name: contract.name,
            major: contract.major,
            revision: contract.revision,
            schema_sha256: contract.schema_sha256,
        }),
        source: Some(SourceRefV1 {
            module_id: context.module_id.clone(),
            runtime_instance_id: source_runtime_public_id_v1(context).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp(context)),
        partition_key: input.partition_key.to_vec(),
        causation_message_id: input.causation_message_id,
        correlation_id: input.correlation_id.to_vec(),
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
        semantics: Some(input.semantics),
        payload: input.payload,
    };
    validate_envelope_v1(&envelope).map_err(|_| PersonsEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn validate_context(context: &PersonsEnvelopeContextV1) -> Result<(), PersonsEnvelopeBuildErrorV1> {
    if context.module_id.is_empty()
        || context.module_id.len() > 128
        || !context.module_id.is_ascii()
        || context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 128
        || !context.runtime_instance_id.is_ascii()
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(PersonsEnvelopeBuildErrorV1::InvalidContext);
    }
    Ok(())
}

#[must_use]
pub fn source_runtime_public_id_v1(context: &PersonsEnvelopeContextV1) -> [u8; 16] {
    digest16(
        b"persons-runtime-instance-v1",
        context.runtime_instance_id.as_bytes(),
        context.module_id.as_bytes(),
    )
}

fn timestamp(context: &PersonsEnvelopeContextV1) -> Timestamp {
    Timestamp {
        seconds: context.recorded_at_unix_seconds,
        nanos: context.recorded_at_nanos,
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], PersonsEnvelopeBuildErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(PersonsEnvelopeBuildErrorV1::InvalidPayload)
}

fn digest16(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(label);
    hasher.update((first.len() as u64).to_be_bytes());
    hasher.update(first);
    hasher.update((second.len() as u64).to_be_bytes());
    hasher.update(second);
    hasher.finalize()[..16].try_into().expect("fixed digest")
}

fn outbox_error(_: OutboxRecordError) -> PersonsEnvelopeBuildErrorV1 {
    PersonsEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::{
        v1::durable_envelope_v1::Semantics, validation::envelope::decode_envelope_v1,
    };
    use makosh_persons_api::wire::{
        ManualCreatePersonCommandV1, PersonProfileV1, TimestampV1, persons_command_v1::Command,
    };

    #[test]
    fn command_envelope_binds_exact_identity_partition_and_fence() {
        let payload = PersonsCommandV1 {
            command: Some(Command::ManualCreate(ManualCreatePersonCommandV1 {
                command_id: vec![1; 16],
                person_id: vec![2; 16],
                logical_owner_id: "owner-a".to_owned(),
                owner_profile: Some(PersonProfileV1 {
                    display_name: Some("Ada".to_owned()),
                    ..Default::default()
                }),
                created_at: Some(TimestampV1 {
                    unix_seconds: 1_800_000_000,
                    nanos: 0,
                }),
            })),
        };
        let record = build_persons_command_outbox_record_v1(
            payload.clone(),
            1_800_000_100,
            &context("producer-runtime", 7),
        )
        .expect("command envelope");
        let envelope = decode_envelope_v1(record.exact_bytes()).expect("decode envelope");
        let Some(Semantics::Command(command)) = envelope.semantics else {
            panic!("command semantics");
        };
        assert_eq!(record.message_id(), &[1; 16]);
        assert_eq!(command.command_id, vec![1; 16]);
        assert_eq!(
            command.idempotency_key,
            persons_command_fingerprint_v1(&payload)
        );
        assert_eq!(envelope.correlation_id, envelope.partition_key);
        assert_eq!(envelope.source_fence.expect("fence").epoch, 7);
    }

    fn context(module: &str, generation: u64) -> PersonsEnvelopeContextV1 {
        PersonsEnvelopeContextV1 {
            module_id: module.to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: generation,
            recorded_at_unix_seconds: 1_800_000_001,
            recorded_at_nanos: 0,
        }
    }
}
