use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ActorKindV1, DurableEnvelopeV1, FenceKindV1, durable_envelope_v1::Semantics},
};
use makosh_graph_core::{GraphEdgeV1, GraphNodeV1};
use makosh_graph_persistence::{
    ApplyGraphMutationV1, GraphEnvelopeRecordV1, GraphMutationV1, GraphPersistenceErrorV1,
    GraphPersistenceV1, GraphReplayOutcomeV1,
};
use makosh_persons_api::{
    PERSONS_MODULE_ID_V1, persons_owner_event_contract_reference_v1, persons_owner_partition_id_v1,
    wire::{
        LineageChangeKindV1, PersonLifecycleV1, PersonsOwnerEventV1,
        persons_owner_event_v1::Event as PersonEvent,
    },
};
use makosh_relationships_api::{
    RELATIONSHIPS_MODULE_ID_V1,
    client_wire::{
        RelationshipChangedV1, RelationshipParticipantKindV1, RelationshipParticipantV1,
        RelationshipStateV1, RelationshipTypeV1,
    },
    relationships_lifecycle_event_contract_reference_v1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;
use sha2::{Digest, Sha256};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphSourceV1 {
    Persons,
    Relationships,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphExecutionContextV1 {
    pub logical_owner_id: String,
    pub projection_generation: u64,
    pub now_unix_millis: i64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphExecutionOutcomeV1 {
    Applied,
    Replayed,
    Ignored,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphExecutionErrorV1 {
    InvalidContext,
    InvalidEnvelope,
    InvalidPayload,
    Persistence(GraphPersistenceErrorV1),
}
struct Normalized {
    message_id: [u8; 16],
    owner: String,
    source_owner: &'static str,
    module: &'static str,
    revision: u64,
    seconds: i64,
    nanos: i32,
    partition: [u8; 16],
    contract: ContractReferenceV1,
    mutation: GraphMutationV1,
}
pub async fn process_graph_source_event_v1(
    persistence: &GraphPersistenceV1,
    record: &OutboxRecordV1,
    source: GraphSourceV1,
    context: &GraphExecutionContextV1,
) -> Result<GraphExecutionOutcomeV1, GraphExecutionErrorV1> {
    validate_context(context)?;
    let envelope: DurableEnvelopeV1 =
        exact_decode(record.exact_bytes(), GraphExecutionErrorV1::InvalidEnvelope)?;
    let Some(event) = normalize(source, &envelope)? else {
        return Ok(GraphExecutionOutcomeV1::Ignored);
    };
    validate_envelope(record, &envelope, &event, context)?;
    let outcome = persistence
        .apply_once(&ApplyGraphMutationV1 {
            input: GraphEnvelopeRecordV1 {
                message_id: event.message_id,
                envelope_sha256: *record.envelope_sha256(),
                envelope_bytes: record.exact_bytes().to_vec(),
            },
            projection_generation: context.projection_generation,
            logical_owner_id: event.owner,
            source_owner: event.source_owner.into(),
            source_revision: event.revision,
            mutation: event.mutation,
            completed_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map_err(GraphExecutionErrorV1::Persistence)?;
    Ok(match outcome {
        GraphReplayOutcomeV1::Applied => GraphExecutionOutcomeV1::Applied,
        GraphReplayOutcomeV1::Replayed => GraphExecutionOutcomeV1::Replayed,
    })
}
fn normalize(
    source: GraphSourceV1,
    envelope: &DurableEnvelopeV1,
) -> Result<Option<Normalized>, GraphExecutionErrorV1> {
    match source {
        GraphSourceV1::Relationships => normalize_relationship(envelope),
        GraphSourceV1::Persons => normalize_persons(envelope),
    }
}
fn normalize_relationship(
    envelope: &DurableEnvelopeV1,
) -> Result<Option<Normalized>, GraphExecutionErrorV1> {
    let value: RelationshipChangedV1 =
        exact_decode(&envelope.payload, GraphExecutionErrorV1::InvalidPayload)?;
    let state = RelationshipStateV1::try_from(value.state)
        .map_err(|_| GraphExecutionErrorV1::InvalidPayload)?;
    let relation = RelationshipTypeV1::try_from(value.relationship_type)
        .map_err(|_| GraphExecutionErrorV1::InvalidPayload)?;
    if state as i32 == 0 || relation as i32 == 0 {
        return Err(GraphExecutionErrorV1::InvalidPayload);
    }
    let time = value
        .occurred_at
        .ok_or(GraphExecutionErrorV1::InvalidPayload)?;
    let revision = positive(value.relationship_revision)?;
    let edge = GraphEdgeV1 {
        edge_id: id16(&value.relationship_id)?,
        logical_owner_id: value.logical_owner_id.clone(),
        source: participant(value.source)?,
        target: participant(value.target)?,
        edge_kind: if state == RelationshipStateV1::RelationshipStateConfirmed {
            relation.as_str_name().to_ascii_lowercase()
        } else {
            String::new()
        },
        source_revision: revision,
        occurred_at_unix_millis: millis(time.unix_seconds, time.nanos)?,
        deleted: state == RelationshipStateV1::RelationshipStateEnded,
    };
    Ok(Some(Normalized {
        message_id: id16(&value.event_id)?,
        owner: value.logical_owner_id,
        source_owner: "relationships",
        module: RELATIONSHIPS_MODULE_ID_V1,
        revision,
        seconds: time.unix_seconds,
        nanos: time.nanos,
        partition: id16(&value.relationship_id)?,
        contract: relationships_lifecycle_event_contract_reference_v1(),
        mutation: GraphMutationV1::UpsertEdge(edge),
    }))
}
fn normalize_persons(
    envelope: &DurableEnvelopeV1,
) -> Result<Option<Normalized>, GraphExecutionErrorV1> {
    let value: PersonsOwnerEventV1 =
        exact_decode(&envelope.payload, GraphExecutionErrorV1::InvalidPayload)?;
    match value.event {
        Some(PersonEvent::PersonChanged(value)) => {
            let state = PersonLifecycleV1::try_from(value.lifecycle)
                .map_err(|_| GraphExecutionErrorV1::InvalidPayload)?;
            if state as i32 == 0 {
                return Err(GraphExecutionErrorV1::InvalidPayload);
            }
            let time = value
                .changed_at
                .ok_or(GraphExecutionErrorV1::InvalidPayload)?;
            let revision = positive(value.person_revision)?;
            let owner = value.logical_owner_id.clone();
            Ok(Some(Normalized {
                message_id: id16(&value.event_id)?,
                owner: value.logical_owner_id,
                source_owner: "persons",
                module: PERSONS_MODULE_ID_V1,
                revision,
                seconds: time.unix_seconds,
                nanos: time.nanos,
                partition: persons_owner_partition_id_v1(&owner)
                    .map_err(|_| GraphExecutionErrorV1::InvalidPayload)?,
                contract: persons_owner_event_contract_reference_v1(),
                mutation: GraphMutationV1::UpsertNode {
                    node: GraphNodeV1 {
                        owner: "persons".into(),
                        kind: "person".into(),
                        id: id16(&value.person_id)?,
                    },
                    source_revision: revision,
                    deleted: state == PersonLifecycleV1::PersonLifecycleArchived,
                },
            }))
        }
        Some(PersonEvent::LineageChanged(value)) => {
            let kind = LineageChangeKindV1::try_from(value.change_kind)
                .map_err(|_| GraphExecutionErrorV1::InvalidPayload)?;
            if kind as i32 == 0 {
                return Err(GraphExecutionErrorV1::InvalidPayload);
            }
            let time = value
                .changed_at
                .ok_or(GraphExecutionErrorV1::InvalidPayload)?;
            let revision = positive(value.resulting_owner_revision)?;
            let owner = value.logical_owner_id.clone();
            let source = id16(&value.source_person_id)?;
            let target = id16(&value.target_person_id)?;
            let edge = GraphEdgeV1 {
                edge_id: lineage_edge_id(source, target),
                logical_owner_id: value.logical_owner_id.clone(),
                source: GraphNodeV1 {
                    owner: "persons".into(),
                    kind: "person".into(),
                    id: source,
                },
                target: GraphNodeV1 {
                    owner: "persons".into(),
                    kind: "person".into(),
                    id: target,
                },
                edge_kind: if kind == LineageChangeKindV1::LineageChangeKindMerged {
                    "person_lineage".into()
                } else {
                    String::new()
                },
                source_revision: revision,
                occurred_at_unix_millis: millis(time.unix_seconds, time.nanos)?,
                deleted: kind == LineageChangeKindV1::LineageChangeKindSplit,
            };
            Ok(Some(Normalized {
                message_id: id16(&value.event_id)?,
                owner: value.logical_owner_id,
                source_owner: "persons",
                module: PERSONS_MODULE_ID_V1,
                revision,
                seconds: time.unix_seconds,
                nanos: time.nanos,
                partition: persons_owner_partition_id_v1(&owner)
                    .map_err(|_| GraphExecutionErrorV1::InvalidPayload)?,
                contract: persons_owner_event_contract_reference_v1(),
                mutation: GraphMutationV1::UpsertEdge(edge),
            }))
        }
        _ => Ok(None),
    }
}
fn participant(
    value: Option<RelationshipParticipantV1>,
) -> Result<GraphNodeV1, GraphExecutionErrorV1> {
    let value = value.ok_or(GraphExecutionErrorV1::InvalidPayload)?;
    let (kind, owner) = match RelationshipParticipantKindV1::try_from(value.kind) {
        Ok(RelationshipParticipantKindV1::RelationshipParticipantKindPerson) => {
            ("person", "persons")
        }
        Ok(RelationshipParticipantKindV1::RelationshipParticipantKindOrganization) => {
            ("organization", "organizations")
        }
        _ => return Err(GraphExecutionErrorV1::InvalidPayload),
    };
    Ok(GraphNodeV1 {
        owner: owner.into(),
        kind: kind.into(),
        id: id16(&value.public_id)?,
    })
}
fn validate_envelope(
    record: &OutboxRecordV1,
    envelope: &DurableEnvelopeV1,
    event: &Normalized,
    context: &GraphExecutionContextV1,
) -> Result<(), GraphExecutionErrorV1> {
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(GraphExecutionErrorV1::InvalidEnvelope)?;
    let source = envelope
        .source
        .as_ref()
        .ok_or(GraphExecutionErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(GraphExecutionErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(GraphExecutionErrorV1::InvalidEnvelope)?;
    let recorded = envelope
        .recorded_at
        .as_ref()
        .ok_or(GraphExecutionErrorV1::InvalidEnvelope)?;
    let occurred = match envelope.semantics.as_ref() {
        Some(Semantics::Event(value)) => value.occurred_at.as_ref(),
        _ => None,
    }
    .ok_or(GraphExecutionErrorV1::InvalidEnvelope)?;
    if contract.owner != event.contract.owner
        || contract.name != event.contract.name
        || contract.major != event.contract.major
        || contract.revision != event.contract.revision
        || contract.schema_sha256 != event.contract.schema_sha256
        || source.module_id != event.module
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id != event.module.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != event.module.as_bytes()
        || fence.epoch != source.runtime_generation
        || envelope.message_id != event.message_id
        || record.message_id() != &event.message_id
        || envelope.partition_key != event.partition
        || envelope.correlation_id != event.partition
        || envelope.causation_message_id.len() != 16
        || occurred.seconds != event.seconds
        || occurred.nanos != event.nanos
        || millis(event.seconds, event.nanos)? > millis(recorded.seconds, recorded.nanos)?
        || millis(recorded.seconds, recorded.nanos)? > context.now_unix_millis
        || event.owner != context.logical_owner_id
    {
        return Err(GraphExecutionErrorV1::InvalidEnvelope);
    }
    Ok(())
}
fn lineage_edge_id(mut first: [u8; 16], mut second: [u8; 16]) -> [u8; 16] {
    if second < first {
        std::mem::swap(&mut first, &mut second)
    }
    let mut hash = Sha256::new();
    hash.update(b"makosh.graph.person-lineage.v1");
    hash.update(first);
    hash.update(second);
    hash.finalize()[..16].try_into().expect("fixed")
}
fn validate_context(value: &GraphExecutionContextV1) -> Result<(), GraphExecutionErrorV1> {
    if value.logical_owner_id.is_empty()
        || value.projection_generation == 0
        || value.now_unix_millis <= 0
    {
        Err(GraphExecutionErrorV1::InvalidContext)
    } else {
        Ok(())
    }
}
fn exact_decode<M: Message + Default>(
    bytes: &[u8],
    error: GraphExecutionErrorV1,
) -> Result<M, GraphExecutionErrorV1> {
    let value = M::decode(bytes).map_err(|_| error)?;
    (value.encode_to_vec() == bytes)
        .then_some(value)
        .ok_or(error)
}
fn id16(value: &[u8]) -> Result<[u8; 16], GraphExecutionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(GraphExecutionErrorV1::InvalidPayload)
}
fn positive(value: u64) -> Result<u64, GraphExecutionErrorV1> {
    (value > 0)
        .then_some(value)
        .ok_or(GraphExecutionErrorV1::InvalidPayload)
}
fn millis(seconds: i64, nanos: i32) -> Result<i64, GraphExecutionErrorV1> {
    if seconds <= 0 || !(0..1_000_000_000).contains(&nanos) || nanos % 1_000_000 != 0 {
        return Err(GraphExecutionErrorV1::InvalidEnvelope);
    }
    seconds
        .checked_mul(1000)
        .and_then(|v| v.checked_add(i64::from(nanos / 1_000_000)))
        .ok_or(GraphExecutionErrorV1::InvalidEnvelope)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lineage_edge_identity_is_order_independent() {
        assert_eq!(
            lineage_edge_id([1; 16], [2; 16]),
            lineage_edge_id([2; 16], [1; 16])
        );
    }
}
