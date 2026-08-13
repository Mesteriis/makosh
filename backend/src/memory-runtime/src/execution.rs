use makosh_decisions_api::{
    DECISIONS_MODULE_ID_V1,
    client_wire::{DecisionChangedV1, DecisionStateV1},
    decisions_lifecycle_event_contract_reference_v1,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ActorKindV1, DurableEnvelopeV1, FenceKindV1, durable_envelope_v1::Semantics},
};
use makosh_knowledge_command_api::{
    KNOWLEDGE_MODULE_ID_V1,
    client_wire::{KnowledgeNoteChangedV1, KnowledgeNoteStateV1},
    knowledge_lifecycle_event_contract_reference_v1,
};
use makosh_memory_core::MemoryProjectionEntryV1;
use makosh_memory_persistence::{
    ApplyMemoryEntryV1, MemoryEnvelopeRecordV1, MemoryPersistenceErrorV1, MemoryPersistenceV1,
    MemoryReplayOutcomeV1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use prost::Message;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemorySourceV1 {
    Decisions,
    Knowledge,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryExecutionContextV1 {
    pub logical_owner_id: String,
    pub projection_generation: u64,
    pub now_unix_millis: i64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryExecutionOutcomeV1 {
    Applied,
    Replayed,
    Ignored,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryExecutionErrorV1 {
    InvalidContext,
    InvalidEnvelope,
    InvalidPayload,
    Persistence(MemoryPersistenceErrorV1),
}
struct Event {
    event_id: [u8; 16],
    entity_id: [u8; 16],
    owner: String,
    source_owner: &'static str,
    module: &'static str,
    kind: &'static str,
    revision: u64,
    state: String,
    seconds: i64,
    nanos: i32,
    partition: [u8; 16],
    contract: ContractReferenceV1,
}

pub async fn process_memory_source_event_v1(
    persistence: &MemoryPersistenceV1,
    record: &OutboxRecordV1,
    source: MemorySourceV1,
    context: &MemoryExecutionContextV1,
) -> Result<MemoryExecutionOutcomeV1, MemoryExecutionErrorV1> {
    validate_context(context)?;
    let envelope: DurableEnvelopeV1 = exact_decode(
        record.exact_bytes(),
        MemoryExecutionErrorV1::InvalidEnvelope,
    )?;
    let Some(event) = normalize(source, &envelope)? else {
        return Ok(MemoryExecutionOutcomeV1::Ignored);
    };
    validate_envelope(record, &envelope, &event, context)?;
    let Some((memory_kind, tombstone)) = memory_kind_v1(event.source_owner, &event.state) else {
        return Ok(MemoryExecutionOutcomeV1::Ignored);
    };
    let outcome = persistence
        .apply_entry_once(&ApplyMemoryEntryV1 {
            input: MemoryEnvelopeRecordV1 {
                message_id: event.event_id,
                envelope_sha256: *record.envelope_sha256(),
                envelope_bytes: record.exact_bytes().to_vec(),
            },
            projection_generation: context.projection_generation,
            entry: MemoryProjectionEntryV1 {
                event_id: event.event_id,
                logical_owner_id: event.owner,
                source_owner: event.source_owner.into(),
                entity_kind: event.kind.into(),
                entity_id: event.entity_id,
                source_revision: event.revision,
                memory_kind: if tombstone {
                    String::new()
                } else {
                    memory_kind.into()
                },
                occurred_at_unix_millis: millis(event.seconds, event.nanos)?,
                tombstone,
            },
            completed_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map_err(MemoryExecutionErrorV1::Persistence)?;
    Ok(match outcome {
        MemoryReplayOutcomeV1::Applied => MemoryExecutionOutcomeV1::Applied,
        MemoryReplayOutcomeV1::Replayed => MemoryExecutionOutcomeV1::Replayed,
    })
}

fn memory_kind_v1(source_owner: &str, state: &str) -> Option<(&'static str, bool)> {
    match (source_owner, state) {
        ("knowledge", "knowledge_note_state_active") => Some(("verified_knowledge", false)),
        ("knowledge", "knowledge_note_state_archived") => Some(("", true)),
        ("decisions", "decision_state_decided") => Some(("accepted_decision", false)),
        ("decisions", "decision_state_superseded" | "decision_state_cancelled") => Some(("", true)),
        _ => None,
    }
}

fn normalize(
    source: MemorySourceV1,
    envelope: &DurableEnvelopeV1,
) -> Result<Option<Event>, MemoryExecutionErrorV1> {
    macro_rules! lifecycle {
        ($type:ty,$contract:expr,$module:expr,$owner:expr,$kind:expr,$id:ident,$revision:ident,$state:ident,$time:ident,$enum:ty) => {{
            let value: $type =
                exact_decode(&envelope.payload, MemoryExecutionErrorV1::InvalidPayload)?;
            let state = <$enum>::try_from(value.$state)
                .map_err(|_| MemoryExecutionErrorV1::InvalidPayload)?;
            if state as i32 == 0 {
                return Err(MemoryExecutionErrorV1::InvalidPayload);
            }
            let time = value.$time.ok_or(MemoryExecutionErrorV1::InvalidPayload)?;
            let name = state.as_str_name().to_ascii_lowercase();
            Ok(Some(Event {
                event_id: id16(&value.event_id)?,
                entity_id: id16(&value.$id)?,
                owner: value.logical_owner_id,
                source_owner: $owner,
                module: $module,
                kind: $kind,
                revision: positive(value.$revision)?,
                state: name.clone(),
                seconds: time.unix_seconds,
                nanos: time.nanos,
                partition: id16(&value.$id)?,
                contract: $contract,
            }))
        }};
    }
    match source {
        MemorySourceV1::Decisions => lifecycle!(
            DecisionChangedV1,
            decisions_lifecycle_event_contract_reference_v1(),
            DECISIONS_MODULE_ID_V1,
            "decisions",
            "decision",
            decision_id,
            decision_revision,
            state,
            occurred_at,
            DecisionStateV1
        ),
        MemorySourceV1::Knowledge => lifecycle!(
            KnowledgeNoteChangedV1,
            knowledge_lifecycle_event_contract_reference_v1(),
            KNOWLEDGE_MODULE_ID_V1,
            "knowledge",
            "knowledge_note",
            note_id,
            note_revision,
            state,
            occurred_at,
            KnowledgeNoteStateV1
        ),
    }
}
fn validate_envelope(
    record: &OutboxRecordV1,
    envelope: &DurableEnvelopeV1,
    event: &Event,
    context: &MemoryExecutionContextV1,
) -> Result<(), MemoryExecutionErrorV1> {
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(MemoryExecutionErrorV1::InvalidEnvelope)?;
    let source = envelope
        .source
        .as_ref()
        .ok_or(MemoryExecutionErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(MemoryExecutionErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(MemoryExecutionErrorV1::InvalidEnvelope)?;
    let recorded = envelope
        .recorded_at
        .as_ref()
        .ok_or(MemoryExecutionErrorV1::InvalidEnvelope)?;
    let occurred = match envelope.semantics.as_ref() {
        Some(Semantics::Event(value)) => value.occurred_at.as_ref(),
        _ => None,
    }
    .ok_or(MemoryExecutionErrorV1::InvalidEnvelope)?;
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
        || envelope.message_id != event.event_id
        || record.message_id() != &event.event_id
        || envelope.partition_key != event.partition
        || envelope.correlation_id != event.partition
        || envelope.causation_message_id.len() != 16
        || occurred.seconds != event.seconds
        || occurred.nanos != event.nanos
        || millis(event.seconds, event.nanos)? > millis(recorded.seconds, recorded.nanos)?
        || millis(recorded.seconds, recorded.nanos)? > context.now_unix_millis
        || event.owner != context.logical_owner_id
    {
        return Err(MemoryExecutionErrorV1::InvalidEnvelope);
    }
    Ok(())
}
fn validate_context(value: &MemoryExecutionContextV1) -> Result<(), MemoryExecutionErrorV1> {
    if value.logical_owner_id.is_empty()
        || value.projection_generation == 0
        || value.now_unix_millis <= 0
    {
        Err(MemoryExecutionErrorV1::InvalidContext)
    } else {
        Ok(())
    }
}
fn exact_decode<M: Message + Default>(
    bytes: &[u8],
    error: MemoryExecutionErrorV1,
) -> Result<M, MemoryExecutionErrorV1> {
    let value = M::decode(bytes).map_err(|_| error)?;
    (value.encode_to_vec() == bytes)
        .then_some(value)
        .ok_or(error)
}
fn id16(value: &[u8]) -> Result<[u8; 16], MemoryExecutionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(MemoryExecutionErrorV1::InvalidPayload)
}
fn positive(value: u64) -> Result<u64, MemoryExecutionErrorV1> {
    (value > 0)
        .then_some(value)
        .ok_or(MemoryExecutionErrorV1::InvalidPayload)
}
fn millis(seconds: i64, nanos: i32) -> Result<i64, MemoryExecutionErrorV1> {
    if seconds <= 0 || !(0..1_000_000_000).contains(&nanos) || nanos % 1_000_000 != 0 {
        return Err(MemoryExecutionErrorV1::InvalidEnvelope);
    }
    seconds
        .checked_mul(1000)
        .and_then(|v| v.checked_add(i64::from(nanos / 1_000_000)))
        .ok_or(MemoryExecutionErrorV1::InvalidEnvelope)
}
#[cfg(test)]
mod tests {
    use super::*;
    use makosh_decisions_api::{
        DecisionsEnvelopeContextV1, build_decision_changed_outbox_record_v1,
        client_wire::{DecisionChangedV1, TimestampV1},
    };
    #[test]
    fn canonical_decision_event_becomes_bounded_memory_entry() {
        let record = build_decision_changed_outbox_record_v1(
            [1; 16],
            DecisionChangedV1 {
                event_id: vec![2; 16],
                decision_id: vec![3; 16],
                logical_owner_id: "owner-1".into(),
                decision_revision: 2,
                state: DecisionStateV1::DecisionStateDecided as i32,
                occurred_at: Some(TimestampV1 {
                    unix_seconds: 10,
                    nanos: 0,
                }),
            },
            &DecisionsEnvelopeContextV1 {
                module_id: DECISIONS_MODULE_ID_V1.into(),
                runtime_instance_id: "runtime".into(),
                runtime_generation: 2,
                recorded_at_unix_seconds: 10,
                recorded_at_nanos: 0,
            },
        )
        .unwrap();
        let envelope: DurableEnvelopeV1 = exact_decode(
            record.exact_bytes(),
            MemoryExecutionErrorV1::InvalidEnvelope,
        )
        .unwrap();
        let event = normalize(MemorySourceV1::Decisions, &envelope)
            .unwrap()
            .unwrap();
        assert_eq!(event.kind, "decision");
        assert_eq!(event.revision, 2);
        assert_eq!(
            memory_kind_v1(event.source_owner, &event.state),
            Some(("accepted_decision", false))
        );
        assert_eq!(memory_kind_v1("decisions", "decision_state_draft"), None);
    }
}
