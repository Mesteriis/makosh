use makosh_calendar_api::{
    CALENDAR_MODULE_ID_V1, calendar_lifecycle_event_contract_reference_v1,
    client_wire::{CalendarEventChangedV1, CalendarEventStateV1},
};
use makosh_decisions_api::{
    DECISIONS_MODULE_ID_V1,
    client_wire::{DecisionChangedV1, DecisionStateV1},
    decisions_lifecycle_event_contract_reference_v1,
};
use makosh_documents_api::{
    DOCUMENTS_MODULE_ID_V1,
    client_wire::{DocumentChangedV1, DocumentStateV1},
    documents_lifecycle_event_contract_reference_v1,
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
use makosh_obligations_api::{
    OBLIGATIONS_MODULE_ID_V1,
    client_wire::{ObligationChangedV1, ObligationStateV1},
    obligations_lifecycle_event_contract_reference_v1,
};
use makosh_organizations_api::{
    ORGANIZATIONS_MODULE_ID_V1,
    client_wire::{OrganizationChangedV1, OrganizationStateV1},
    organizations_lifecycle_event_contract_reference_v1,
};
use makosh_persons_api::{
    PERSONS_MODULE_ID_V1, persons_owner_event_contract_reference_v1, persons_owner_partition_id_v1,
    wire::{PersonLifecycleV1, PersonsOwnerEventV1, persons_owner_event_v1::Event as PersonEvent},
};
use makosh_projects_api::{
    PROJECTS_MODULE_ID_V1,
    client_wire::{ProjectChangedV1, ProjectStateV1},
    projects_lifecycle_event_contract_reference_v1,
};
use makosh_relationships_api::{
    RELATIONSHIPS_MODULE_ID_V1,
    client_wire::{RelationshipChangedV1, RelationshipStateV1},
    relationships_lifecycle_event_contract_reference_v1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use makosh_tasks_command_api::{
    TASKS_MODULE_ID_V1,
    client_wire::{TaskChangedV1, TaskStateV1},
    tasks_lifecycle_event_contract_reference_v1,
};
use makosh_timeline_core::TimelineProjectionEntryV1;
use makosh_timeline_persistence::{
    ApplyTimelineEntryV1, TimelineEnvelopeRecordV1, TimelinePersistenceErrorV1,
    TimelinePersistenceV1, TimelineReplayOutcomeV1,
};
use prost::Message;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimelineSourceV1 {
    Persons,
    Organizations,
    Relationships,
    Projects,
    Tasks,
    Obligations,
    Decisions,
    Calendar,
    Documents,
    Knowledge,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineExecutionContextV1 {
    pub logical_owner_id: String,
    pub projection_generation: u64,
    pub now_unix_millis: i64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimelineExecutionOutcomeV1 {
    Applied,
    Replayed,
    Ignored,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimelineExecutionErrorV1 {
    InvalidContext,
    InvalidEnvelope,
    InvalidPayload,
    Persistence(TimelinePersistenceErrorV1),
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
    tombstone: bool,
    partition: [u8; 16],
    contract: ContractReferenceV1,
}

pub async fn process_timeline_source_event_v1(
    persistence: &TimelinePersistenceV1,
    record: &OutboxRecordV1,
    source: TimelineSourceV1,
    context: &TimelineExecutionContextV1,
) -> Result<TimelineExecutionOutcomeV1, TimelineExecutionErrorV1> {
    validate_context(context)?;
    let envelope: DurableEnvelopeV1 = exact_decode(
        record.exact_bytes(),
        TimelineExecutionErrorV1::InvalidEnvelope,
    )?;
    let Some(event) = normalize(source, &envelope)? else {
        return Ok(TimelineExecutionOutcomeV1::Ignored);
    };
    validate_envelope(record, &envelope, &event, context)?;
    let outcome = persistence
        .apply_entry_once(&ApplyTimelineEntryV1 {
            input: TimelineEnvelopeRecordV1 {
                message_id: event.event_id,
                envelope_sha256: *record.envelope_sha256(),
                envelope_bytes: record.exact_bytes().to_vec(),
            },
            projection_generation: context.projection_generation,
            entry: TimelineProjectionEntryV1 {
                event_id: event.event_id,
                logical_owner_id: event.owner,
                source_owner: event.source_owner.into(),
                entity_kind: event.kind.into(),
                entity_id: event.entity_id,
                source_revision: event.revision,
                lifecycle_state: if event.tombstone {
                    String::new()
                } else {
                    event.state
                },
                occurred_at_unix_millis: millis(event.seconds, event.nanos)?,
                tombstone: event.tombstone,
            },
            completed_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map_err(TimelineExecutionErrorV1::Persistence)?;
    Ok(match outcome {
        TimelineReplayOutcomeV1::Applied => TimelineExecutionOutcomeV1::Applied,
        TimelineReplayOutcomeV1::Replayed => TimelineExecutionOutcomeV1::Replayed,
    })
}

fn normalize(
    source: TimelineSourceV1,
    envelope: &DurableEnvelopeV1,
) -> Result<Option<Event>, TimelineExecutionErrorV1> {
    macro_rules! lifecycle {
        ($type:ty,$contract:expr,$module:expr,$owner:expr,$kind:expr,$id:ident,$revision:ident,$state:ident,$time:ident,$enum:ty) => {{
            let value: $type =
                exact_decode(&envelope.payload, TimelineExecutionErrorV1::InvalidPayload)?;
            let state = <$enum>::try_from(value.$state)
                .map_err(|_| TimelineExecutionErrorV1::InvalidPayload)?;
            if state as i32 == 0 {
                return Err(TimelineExecutionErrorV1::InvalidPayload);
            }
            let time = value
                .$time
                .ok_or(TimelineExecutionErrorV1::InvalidPayload)?;
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
                tombstone: name.ends_with("_archived") || name.ends_with("_deleted"),
                partition: id16(&value.$id)?,
                contract: $contract,
            }))
        }};
    }
    match source {
        TimelineSourceV1::Organizations => lifecycle!(
            OrganizationChangedV1,
            organizations_lifecycle_event_contract_reference_v1(),
            ORGANIZATIONS_MODULE_ID_V1,
            "organizations",
            "organization",
            organization_id,
            organization_revision,
            state,
            occurred_at,
            OrganizationStateV1
        ),
        TimelineSourceV1::Relationships => lifecycle!(
            RelationshipChangedV1,
            relationships_lifecycle_event_contract_reference_v1(),
            RELATIONSHIPS_MODULE_ID_V1,
            "relationships",
            "relationship",
            relationship_id,
            relationship_revision,
            state,
            occurred_at,
            RelationshipStateV1
        ),
        TimelineSourceV1::Projects => lifecycle!(
            ProjectChangedV1,
            projects_lifecycle_event_contract_reference_v1(),
            PROJECTS_MODULE_ID_V1,
            "projects",
            "project",
            project_id,
            project_revision,
            state,
            occurred_at,
            ProjectStateV1
        ),
        TimelineSourceV1::Tasks => lifecycle!(
            TaskChangedV1,
            tasks_lifecycle_event_contract_reference_v1(),
            TASKS_MODULE_ID_V1,
            "tasks",
            "task",
            task_id,
            task_revision,
            state,
            occurred_at,
            TaskStateV1
        ),
        TimelineSourceV1::Obligations => lifecycle!(
            ObligationChangedV1,
            obligations_lifecycle_event_contract_reference_v1(),
            OBLIGATIONS_MODULE_ID_V1,
            "obligations",
            "obligation",
            obligation_id,
            obligation_revision,
            state,
            occurred_at,
            ObligationStateV1
        ),
        TimelineSourceV1::Decisions => lifecycle!(
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
        TimelineSourceV1::Calendar => lifecycle!(
            CalendarEventChangedV1,
            calendar_lifecycle_event_contract_reference_v1(),
            CALENDAR_MODULE_ID_V1,
            "calendar",
            "calendar_event",
            calendar_event_id,
            event_revision,
            state,
            occurred_at,
            CalendarEventStateV1
        ),
        TimelineSourceV1::Documents => lifecycle!(
            DocumentChangedV1,
            documents_lifecycle_event_contract_reference_v1(),
            DOCUMENTS_MODULE_ID_V1,
            "documents",
            "document",
            document_id,
            document_revision,
            state,
            occurred_at,
            DocumentStateV1
        ),
        TimelineSourceV1::Knowledge => lifecycle!(
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
        TimelineSourceV1::Persons => normalize_person(envelope),
    }
}
fn normalize_person(
    envelope: &DurableEnvelopeV1,
) -> Result<Option<Event>, TimelineExecutionErrorV1> {
    let value: PersonsOwnerEventV1 =
        exact_decode(&envelope.payload, TimelineExecutionErrorV1::InvalidPayload)?;
    let Some(PersonEvent::PersonChanged(value)) = value.event else {
        return Ok(None);
    };
    let state = PersonLifecycleV1::try_from(value.lifecycle)
        .map_err(|_| TimelineExecutionErrorV1::InvalidPayload)?;
    if state as i32 == 0 {
        return Err(TimelineExecutionErrorV1::InvalidPayload);
    }
    let time = value
        .changed_at
        .ok_or(TimelineExecutionErrorV1::InvalidPayload)?;
    let name = state.as_str_name().to_ascii_lowercase();
    Ok(Some(Event {
        event_id: id16(&value.event_id)?,
        entity_id: id16(&value.person_id)?,
        partition: persons_owner_partition_id_v1(&value.logical_owner_id)
            .map_err(|_| TimelineExecutionErrorV1::InvalidPayload)?,
        owner: value.logical_owner_id,
        source_owner: "persons",
        module: PERSONS_MODULE_ID_V1,
        kind: "person",
        revision: positive(value.person_revision)?,
        state: name.clone(),
        seconds: time.unix_seconds,
        nanos: time.nanos,
        tombstone: name.ends_with("_archived"),
        contract: persons_owner_event_contract_reference_v1(),
    }))
}
fn validate_envelope(
    record: &OutboxRecordV1,
    envelope: &DurableEnvelopeV1,
    event: &Event,
    context: &TimelineExecutionContextV1,
) -> Result<(), TimelineExecutionErrorV1> {
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(TimelineExecutionErrorV1::InvalidEnvelope)?;
    let source = envelope
        .source
        .as_ref()
        .ok_or(TimelineExecutionErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(TimelineExecutionErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(TimelineExecutionErrorV1::InvalidEnvelope)?;
    let recorded = envelope
        .recorded_at
        .as_ref()
        .ok_or(TimelineExecutionErrorV1::InvalidEnvelope)?;
    let occurred = match envelope.semantics.as_ref() {
        Some(Semantics::Event(value)) => value.occurred_at.as_ref(),
        _ => None,
    }
    .ok_or(TimelineExecutionErrorV1::InvalidEnvelope)?;
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
        return Err(TimelineExecutionErrorV1::InvalidEnvelope);
    }
    Ok(())
}
fn validate_context(value: &TimelineExecutionContextV1) -> Result<(), TimelineExecutionErrorV1> {
    if value.logical_owner_id.is_empty()
        || value.projection_generation == 0
        || value.now_unix_millis <= 0
    {
        Err(TimelineExecutionErrorV1::InvalidContext)
    } else {
        Ok(())
    }
}
fn exact_decode<M: Message + Default>(
    bytes: &[u8],
    error: TimelineExecutionErrorV1,
) -> Result<M, TimelineExecutionErrorV1> {
    let value = M::decode(bytes).map_err(|_| error)?;
    (value.encode_to_vec() == bytes)
        .then_some(value)
        .ok_or(error)
}
fn id16(value: &[u8]) -> Result<[u8; 16], TimelineExecutionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(TimelineExecutionErrorV1::InvalidPayload)
}
fn positive(value: u64) -> Result<u64, TimelineExecutionErrorV1> {
    (value > 0)
        .then_some(value)
        .ok_or(TimelineExecutionErrorV1::InvalidPayload)
}
fn millis(seconds: i64, nanos: i32) -> Result<i64, TimelineExecutionErrorV1> {
    if seconds <= 0 || !(0..1_000_000_000).contains(&nanos) || nanos % 1_000_000 != 0 {
        return Err(TimelineExecutionErrorV1::InvalidEnvelope);
    }
    seconds
        .checked_mul(1000)
        .and_then(|v| v.checked_add(i64::from(nanos / 1_000_000)))
        .ok_or(TimelineExecutionErrorV1::InvalidEnvelope)
}
#[cfg(test)]
mod tests {
    use super::*;
    use makosh_organizations_api::{
        OrganizationsEnvelopeContextV1, build_organization_changed_outbox_record_v1,
        client_wire::{OrganizationChangedV1, OrganizationStateV1, TimestampV1},
    };
    #[test]
    fn canonical_event_becomes_structural_timeline_entry() {
        let record = build_organization_changed_outbox_record_v1(
            [1; 16],
            OrganizationChangedV1 {
                event_id: vec![2; 16],
                organization_id: vec![3; 16],
                logical_owner_id: "owner-1".into(),
                organization_revision: 2,
                state: OrganizationStateV1::OrganizationStateActive as i32,
                occurred_at: Some(TimestampV1 {
                    unix_seconds: 10,
                    nanos: 0,
                }),
            },
            &OrganizationsEnvelopeContextV1 {
                module_id: ORGANIZATIONS_MODULE_ID_V1.into(),
                runtime_instance_id: "runtime".into(),
                runtime_generation: 2,
                recorded_at_unix_seconds: 10,
                recorded_at_nanos: 0,
            },
        )
        .unwrap();
        let envelope: DurableEnvelopeV1 = exact_decode(
            record.exact_bytes(),
            TimelineExecutionErrorV1::InvalidEnvelope,
        )
        .unwrap();
        let event = normalize(TimelineSourceV1::Organizations, &envelope)
            .unwrap()
            .unwrap();
        assert_eq!(event.kind, "organization");
        assert_eq!(event.revision, 2);
    }
}
