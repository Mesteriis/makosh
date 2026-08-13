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
use makosh_search_core::{SearchProjectionDocumentV1, search_query_token_digests_v1};
use makosh_search_persistence::{
    ApplySearchDocumentV1, SearchEnvelopeRecordV1, SearchPersistenceErrorV1, SearchPersistenceV1,
    SearchReplayOutcomeV1,
};
use makosh_tasks_command_api::{
    TASKS_MODULE_ID_V1,
    client_wire::{TaskChangedV1, TaskStateV1},
    tasks_lifecycle_event_contract_reference_v1,
};
use prost::Message;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchSourceV1 {
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
pub struct SearchExecutionContextV1 {
    pub logical_owner_id: String,
    pub projection_generation: u64,
    pub owner_derived_key: [u8; 32],
    pub now_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchExecutionOutcomeV1 {
    Applied,
    Replayed,
    Ignored,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchExecutionErrorV1 {
    InvalidContext,
    InvalidEnvelope,
    InvalidPayload,
    Persistence(SearchPersistenceErrorV1),
}

struct NormalizedEventV1 {
    message_id: [u8; 16],
    entity_id: [u8; 16],
    logical_owner_id: String,
    source_owner: &'static str,
    source_module_id: &'static str,
    entity_kind: &'static str,
    source_revision: u64,
    lifecycle_state: String,
    occurred_seconds: i64,
    occurred_nanos: i32,
    deleted: bool,
    partition_key: [u8; 16],
    contract: ContractReferenceV1,
}

pub async fn process_search_source_event_v1(
    persistence: &SearchPersistenceV1,
    record: &OutboxRecordV1,
    source: SearchSourceV1,
    context: &SearchExecutionContextV1,
) -> Result<SearchExecutionOutcomeV1, SearchExecutionErrorV1> {
    validate_context(context)?;
    let envelope: DurableEnvelopeV1 = exact_decode(
        record.exact_bytes(),
        SearchExecutionErrorV1::InvalidEnvelope,
    )?;
    let Some(event) = normalize(source, &envelope)? else {
        return Ok(SearchExecutionOutcomeV1::Ignored);
    };
    validate_envelope(record, &envelope, &event, context)?;
    let lifecycle_state = if event.deleted {
        String::new()
    } else {
        event.lifecycle_state.clone()
    };
    let token_digests = if event.deleted {
        Vec::new()
    } else {
        search_query_token_digests_v1(
            &context.owner_derived_key,
            &format!(
                "{} {} {}",
                event.source_owner, event.entity_kind, event.lifecycle_state
            ),
        )
        .map_err(|_| SearchExecutionErrorV1::InvalidPayload)?
    };
    let outcome = persistence
        .apply_document_once(&ApplySearchDocumentV1 {
            input: SearchEnvelopeRecordV1 {
                message_id: event.message_id,
                envelope_sha256: *record.envelope_sha256(),
                envelope_bytes: record.exact_bytes().to_vec(),
            },
            projection_generation: context.projection_generation,
            document: SearchProjectionDocumentV1 {
                logical_owner_id: event.logical_owner_id,
                source_owner: event.source_owner.to_owned(),
                entity_kind: event.entity_kind.to_owned(),
                entity_id: event.entity_id,
                source_revision: event.source_revision,
                lifecycle_state,
                occurred_at_unix_millis: millis(event.occurred_seconds, event.occurred_nanos)?,
                deleted: event.deleted,
            },
            token_digests,
            completed_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map_err(SearchExecutionErrorV1::Persistence)?;
    Ok(match outcome {
        SearchReplayOutcomeV1::Applied => SearchExecutionOutcomeV1::Applied,
        SearchReplayOutcomeV1::Replayed => SearchExecutionOutcomeV1::Replayed,
    })
}

fn normalize(
    source: SearchSourceV1,
    envelope: &DurableEnvelopeV1,
) -> Result<Option<NormalizedEventV1>, SearchExecutionErrorV1> {
    macro_rules! lifecycle {
        ($type:ty, $contract:expr, $module:expr, $owner:expr, $kind:expr, $id:ident, $revision:ident, $state:ident, $time:ident, $enum:ty) => {{
            let value: $type =
                exact_decode(&envelope.payload, SearchExecutionErrorV1::InvalidPayload)?;
            let state = <$enum>::try_from(value.$state)
                .map_err(|_| SearchExecutionErrorV1::InvalidPayload)?;
            if state as i32 == 0 {
                return Err(SearchExecutionErrorV1::InvalidPayload);
            }
            let time = value.$time.ok_or(SearchExecutionErrorV1::InvalidPayload)?;
            let name = state.as_str_name().to_ascii_lowercase();
            Ok(Some(NormalizedEventV1 {
                message_id: id16(&value.event_id)?,
                entity_id: id16(&value.$id)?,
                logical_owner_id: value.logical_owner_id,
                source_owner: $owner,
                source_module_id: $module,
                entity_kind: $kind,
                source_revision: positive(value.$revision)?,
                lifecycle_state: name.clone(),
                occurred_seconds: time.unix_seconds,
                occurred_nanos: time.nanos,
                deleted: name.ends_with("_archived") || name.ends_with("_deleted"),
                partition_key: id16(&value.$id)?,
                contract: $contract,
            }))
        }};
    }
    match source {
        SearchSourceV1::Organizations => lifecycle!(
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
        SearchSourceV1::Relationships => lifecycle!(
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
        SearchSourceV1::Projects => lifecycle!(
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
        SearchSourceV1::Tasks => lifecycle!(
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
        SearchSourceV1::Obligations => lifecycle!(
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
        SearchSourceV1::Decisions => lifecycle!(
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
        SearchSourceV1::Calendar => lifecycle!(
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
        SearchSourceV1::Documents => lifecycle!(
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
        SearchSourceV1::Knowledge => lifecycle!(
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
        SearchSourceV1::Persons => normalize_persons(envelope),
    }
}

fn normalize_persons(
    envelope: &DurableEnvelopeV1,
) -> Result<Option<NormalizedEventV1>, SearchExecutionErrorV1> {
    let value: PersonsOwnerEventV1 =
        exact_decode(&envelope.payload, SearchExecutionErrorV1::InvalidPayload)?;
    let Some(PersonEvent::PersonChanged(value)) = value.event else {
        return Ok(None);
    };
    let state = PersonLifecycleV1::try_from(value.lifecycle)
        .map_err(|_| SearchExecutionErrorV1::InvalidPayload)?;
    if state as i32 == 0 {
        return Err(SearchExecutionErrorV1::InvalidPayload);
    }
    let time = value
        .changed_at
        .ok_or(SearchExecutionErrorV1::InvalidPayload)?;
    let name = state.as_str_name().to_ascii_lowercase();
    Ok(Some(NormalizedEventV1 {
        message_id: id16(&value.event_id)?,
        entity_id: id16(&value.person_id)?,
        partition_key: persons_owner_partition_id_v1(&value.logical_owner_id)
            .map_err(|_| SearchExecutionErrorV1::InvalidPayload)?,
        logical_owner_id: value.logical_owner_id,
        source_owner: "persons",
        source_module_id: PERSONS_MODULE_ID_V1,
        entity_kind: "person",
        source_revision: positive(value.person_revision)?,
        lifecycle_state: name.clone(),
        occurred_seconds: time.unix_seconds,
        occurred_nanos: time.nanos,
        deleted: name.ends_with("_archived"),
        contract: persons_owner_event_contract_reference_v1(),
    }))
}

fn validate_envelope(
    record: &OutboxRecordV1,
    envelope: &DurableEnvelopeV1,
    event: &NormalizedEventV1,
    context: &SearchExecutionContextV1,
) -> Result<(), SearchExecutionErrorV1> {
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(SearchExecutionErrorV1::InvalidEnvelope)?;
    let source = envelope
        .source
        .as_ref()
        .ok_or(SearchExecutionErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(SearchExecutionErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(SearchExecutionErrorV1::InvalidEnvelope)?;
    let recorded = envelope
        .recorded_at
        .as_ref()
        .ok_or(SearchExecutionErrorV1::InvalidEnvelope)?;
    let occurred = match envelope.semantics.as_ref() {
        Some(Semantics::Event(value)) => value.occurred_at.as_ref(),
        _ => None,
    }
    .ok_or(SearchExecutionErrorV1::InvalidEnvelope)?;
    let recorded_millis = millis(recorded.seconds, recorded.nanos)?;
    let occurred_millis = millis(event.occurred_seconds, event.occurred_nanos)?;
    if contract.owner != event.contract.owner
        || contract.name != event.contract.name
        || contract.major != event.contract.major
        || contract.revision != event.contract.revision
        || contract.schema_sha256 != event.contract.schema_sha256
        || source.module_id != event.source_module_id
        || source.runtime_instance_id.len() != 16
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id != event.source_module_id.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != event.source_module_id.as_bytes()
        || fence.epoch != source.runtime_generation
        || envelope.message_id != event.message_id
        || record.message_id() != &event.message_id
        || envelope.partition_key != event.partition_key
        || envelope.correlation_id != event.partition_key
        || envelope.causation_message_id.len() != 16
        || occurred.seconds != event.occurred_seconds
        || occurred.nanos != event.occurred_nanos
        || occurred_millis > recorded_millis
        || recorded_millis > context.now_unix_millis
        || event.logical_owner_id != context.logical_owner_id
    {
        return Err(SearchExecutionErrorV1::InvalidEnvelope);
    }
    Ok(())
}

fn validate_context(value: &SearchExecutionContextV1) -> Result<(), SearchExecutionErrorV1> {
    if value.logical_owner_id.is_empty()
        || value.projection_generation == 0
        || value.owner_derived_key.iter().all(|byte| *byte == 0)
        || value.now_unix_millis <= 0
    {
        Err(SearchExecutionErrorV1::InvalidContext)
    } else {
        Ok(())
    }
}

fn exact_decode<M: Message + Default>(
    bytes: &[u8],
    error: SearchExecutionErrorV1,
) -> Result<M, SearchExecutionErrorV1> {
    let value = M::decode(bytes).map_err(|_| error)?;
    (value.encode_to_vec() == bytes)
        .then_some(value)
        .ok_or(error)
}

fn id16(value: &[u8]) -> Result<[u8; 16], SearchExecutionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(SearchExecutionErrorV1::InvalidPayload)
}

fn positive(value: u64) -> Result<u64, SearchExecutionErrorV1> {
    (value > 0)
        .then_some(value)
        .ok_or(SearchExecutionErrorV1::InvalidPayload)
}

fn millis(seconds: i64, nanos: i32) -> Result<i64, SearchExecutionErrorV1> {
    if seconds <= 0 || !(0..1_000_000_000).contains(&nanos) || nanos % 1_000_000 != 0 {
        return Err(SearchExecutionErrorV1::InvalidEnvelope);
    }
    seconds
        .checked_mul(1_000)
        .and_then(|value| value.checked_add(i64::from(nanos / 1_000_000)))
        .ok_or(SearchExecutionErrorV1::InvalidEnvelope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_organizations_api::{
        OrganizationsEnvelopeContextV1, build_organization_changed_outbox_record_v1,
        client_wire::{OrganizationChangedV1, OrganizationStateV1, TimestampV1},
    };

    #[test]
    fn canonical_lifecycle_envelope_normalizes_without_private_content() {
        let record = build_organization_changed_outbox_record_v1(
            [1; 16],
            OrganizationChangedV1 {
                event_id: vec![2; 16],
                organization_id: vec![3; 16],
                logical_owner_id: "owner-1".to_owned(),
                organization_revision: 2,
                state: OrganizationStateV1::OrganizationStateActive as i32,
                occurred_at: Some(TimestampV1 {
                    unix_seconds: 10,
                    nanos: 0,
                }),
            },
            &OrganizationsEnvelopeContextV1 {
                module_id: ORGANIZATIONS_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "organization-runtime".to_owned(),
                runtime_generation: 3,
                recorded_at_unix_seconds: 10,
                recorded_at_nanos: 0,
            },
        )
        .expect("event");
        let envelope: DurableEnvelopeV1 = exact_decode(
            record.exact_bytes(),
            SearchExecutionErrorV1::InvalidEnvelope,
        )
        .unwrap();
        let normalized = normalize(SearchSourceV1::Organizations, &envelope)
            .unwrap()
            .unwrap();
        assert_eq!(normalized.entity_kind, "organization");
        assert_eq!(normalized.source_revision, 2);
        assert!(!normalized.deleted);
    }

    #[test]
    fn unknown_top_level_or_payload_fields_are_rejected() {
        let mut private = vec![0xA2, 0x06, 0x08];
        private.extend_from_slice(b"private!");
        assert_eq!(
            exact_decode::<DurableEnvelopeV1>(&private, SearchExecutionErrorV1::InvalidEnvelope),
            Err(SearchExecutionErrorV1::InvalidEnvelope)
        );
    }
}
