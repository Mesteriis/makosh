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
    RELATIONSHIPS_MODULE_ID_V1, client_wire::RelationshipChangedV1,
    relationships_lifecycle_event_contract_reference_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipsEnvelopeContextV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationshipsEnvelopeBuildErrorV1 {
    InvalidContext,
    InvalidPayload,
    InvalidEnvelope,
    OutboxRejected,
}

pub fn build_relationship_changed_outbox_record_v1(
    operation_id: [u8; 16],
    payload: RelationshipChangedV1,
    context: &RelationshipsEnvelopeContextV1,
) -> Result<OutboxRecordV1, RelationshipsEnvelopeBuildErrorV1> {
    if context.runtime_instance_id.is_empty()
        || context.runtime_instance_id.len() > 128
        || context.runtime_generation == 0
        || context.recorded_at_unix_seconds <= 0
        || !(0..1_000_000_000).contains(&context.recorded_at_nanos)
    {
        return Err(RelationshipsEnvelopeBuildErrorV1::InvalidContext);
    }
    let event_id = id16(&payload.event_id)?;
    let relationship_id = id16(&payload.relationship_id)?;
    let occurred_at = payload
        .occurred_at
        .as_ref()
        .filter(|value| valid_timestamp(value.unix_seconds, value.nanos))
        .ok_or(RelationshipsEnvelopeBuildErrorV1::InvalidPayload)?;
    if !nonzero(&operation_id)
        || !valid_owner(&payload.logical_owner_id)
        || payload
            .source
            .as_ref()
            .is_none_or(|value| value.kind == 0 || id16(&value.public_id).is_err())
        || payload
            .target
            .as_ref()
            .is_none_or(|value| value.kind == 0 || id16(&value.public_id).is_err())
        || payload.relationship_type == 0
        || payload.state == 0
        || payload.relationship_revision == 0
        || payload
            .valid_from
            .as_ref()
            .is_none_or(|value| !valid_timestamp(value.unix_seconds, value.nanos))
    {
        return Err(RelationshipsEnvelopeBuildErrorV1::InvalidPayload);
    }
    let contract = relationships_lifecycle_event_contract_reference_v1();
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
            module_id: RELATIONSHIPS_MODULE_ID_V1.to_owned(),
            runtime_instance_id: digest(context.runtime_instance_id.as_bytes()).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(Timestamp {
            seconds: context.recorded_at_unix_seconds,
            nanos: context.recorded_at_nanos,
        }),
        partition_key: relationship_id.to_vec(),
        causation_message_id: operation_id.to_vec(),
        correlation_id: relationship_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: RELATIONSHIPS_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: RELATIONSHIPS_MODULE_ID_V1.as_bytes().to_vec(),
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
    validate_envelope_v1(&envelope)
        .map_err(|_| RelationshipsEnvelopeBuildErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec()).map_err(outbox_error)
}

fn id16(value: &[u8]) -> Result<[u8; 16], RelationshipsEnvelopeBuildErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| RelationshipsEnvelopeBuildErrorV1::InvalidPayload)?;
    nonzero(&value)
        .then_some(value)
        .ok_or(RelationshipsEnvelopeBuildErrorV1::InvalidPayload)
}

fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}
fn valid_timestamp(seconds: i64, nanos: i32) -> bool {
    seconds > 0 && (0..1_000_000_000).contains(&nanos)
}
fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}
fn digest(value: &[u8]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.relationships.runtime-instance.v1\0");
    hash.update((value.len() as u64).to_be_bytes());
    hash.update(value);
    hash.finalize()[..16].try_into().expect("digest")
}
fn outbox_error(_: OutboxRecordError) -> RelationshipsEnvelopeBuildErrorV1 {
    RelationshipsEnvelopeBuildErrorV1::OutboxRejected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_wire::{
        RelationshipChangedV1, RelationshipParticipantKindV1, RelationshipParticipantV1,
        RelationshipStateV1, RelationshipTypeV1, TimestampV1,
    };

    #[test]
    fn event_excludes_evidence_and_private_markers() {
        let participant = RelationshipParticipantV1 {
            kind: RelationshipParticipantKindV1::RelationshipParticipantKindPerson as i32,
            public_id: vec![3; 16],
        };
        let record = build_relationship_changed_outbox_record_v1(
            [1; 16],
            RelationshipChangedV1 {
                event_id: vec![2; 16],
                relationship_id: vec![4; 16],
                logical_owner_id: "owner-1".to_owned(),
                source: Some(participant.clone()),
                target: Some(RelationshipParticipantV1 {
                    public_id: vec![5; 16],
                    ..participant
                }),
                relationship_type: RelationshipTypeV1::RelationshipTypeFriend as i32,
                state: RelationshipStateV1::RelationshipStateConfirmed as i32,
                valid_from: Some(TimestampV1 {
                    unix_seconds: 10,
                    nanos: 0,
                }),
                valid_until: None,
                relationship_revision: 1,
                occurred_at: Some(TimestampV1 {
                    unix_seconds: 10,
                    nanos: 0,
                }),
            },
            &RelationshipsEnvelopeContextV1 {
                runtime_instance_id: "relationships-runtime-1".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 10,
                recorded_at_nanos: 0,
            },
        )
        .expect("event");
        for private in [
            b"private-evidence-record".as_slice(),
            b"raw-provider-body".as_slice(),
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
