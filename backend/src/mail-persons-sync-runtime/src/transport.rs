use makosh_events_protocol::{
    delivery::{OutboxRecordError, OutboxRecordV1},
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use makosh_persons_api::{
    PERSONS_COMMAND_CAPABILITY_ID_V1, PERSONS_OWNER_ID_V1, persons_command_contract_reference_v1,
    wire::{PersonsCommandV1, persons_command_v1::Command},
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::MAIL_PERSONS_SYNC_MODULE_ID_V1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncEnvelopeContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncEnvelopeErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_persons_command_outbox_record_v1(
    payload: PersonsCommandV1,
    deadline_unix_seconds: i64,
    context: &MailPersonsSyncEnvelopeContextV1,
) -> Result<OutboxRecordV1, MailPersonsSyncEnvelopeErrorV1> {
    validate_context(context)?;
    let (command_id, owner) = source_command_identity(&payload)?;
    if deadline_unix_seconds <= context.recorded_at_unix_seconds {
        return Err(MailPersonsSyncEnvelopeErrorV1::InvalidPayload);
    }
    let payload_bytes = payload.encode_to_vec();
    let fingerprint = Sha256::digest(&payload_bytes);
    let partition_key = digest16(
        b"persons-owner-partition-v1",
        owner.as_bytes(),
        PERSONS_OWNER_ID_V1.as_bytes(),
    );
    let contract = persons_command_contract_reference_v1();
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: command_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: contract.owner,
            name: contract.name,
            major: contract.major,
            revision: contract.revision,
            schema_sha256: contract.schema_sha256,
        }),
        source: Some(SourceRefV1 {
            module_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.to_owned(),
            runtime_instance_id: source_runtime_public_id_v1(context).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(Timestamp {
            seconds: context.recorded_at_unix_seconds,
            nanos: context.recorded_at_nanos,
        }),
        partition_key: partition_key.to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: partition_key.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: command_id.to_vec(),
            target_capability: PERSONS_COMMAND_CAPABILITY_ID_V1.to_owned(),
            idempotency_key: fingerprint.to_vec(),
            deadline: Some(Timestamp {
                seconds: deadline_unix_seconds,
                nanos: 0,
            }),
            logical_attempt: 1,
        })),
        payload: payload_bytes,
    };
    validate_envelope_v1(&envelope).map_err(|_| MailPersonsSyncEnvelopeErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn source_command_identity(
    payload: &PersonsCommandV1,
) -> Result<([u8; 16], &str), MailPersonsSyncEnvelopeErrorV1> {
    let (id, owner) = match payload.command.as_ref() {
        Some(Command::SourceObserve(value)) => (&value.command_id, value.logical_owner_id.as_str()),
        Some(Command::SourceUpdate(value)) => (&value.command_id, value.logical_owner_id.as_str()),
        Some(Command::SourceRemove(value)) => (&value.command_id, value.logical_owner_id.as_str()),
        _ => return Err(MailPersonsSyncEnvelopeErrorV1::InvalidPayload),
    };
    let id = id
        .as_slice()
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(MailPersonsSyncEnvelopeErrorV1::InvalidPayload)?;
    if owner.is_empty()
        || owner.len() > 128
        || !owner.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        return Err(MailPersonsSyncEnvelopeErrorV1::InvalidPayload);
    }
    Ok((id, owner))
}

fn validate_context(
    context: &MailPersonsSyncEnvelopeContextV1,
) -> Result<(), MailPersonsSyncEnvelopeErrorV1> {
    if context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 128
        || !context.runtime_instance_id.is_ascii()
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        Err(MailPersonsSyncEnvelopeErrorV1::InvalidContext)
    } else {
        Ok(())
    }
}

#[must_use]
pub fn source_runtime_public_id_v1(context: &MailPersonsSyncEnvelopeContextV1) -> [u8; 16] {
    digest16(
        b"mail-persons-sync-runtime-instance-v1",
        context.runtime_instance_id.as_bytes(),
        MAIL_PERSONS_SYNC_MODULE_ID_V1.as_bytes(),
    )
}

fn digest16(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(label);
    for part in [first, second] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    digest.finalize()[..16].try_into().expect("SHA-256 prefix")
}

fn outbox_error(_: OutboxRecordError) -> MailPersonsSyncEnvelopeErrorV1 {
    MailPersonsSyncEnvelopeErrorV1::OutboxRejected
}
