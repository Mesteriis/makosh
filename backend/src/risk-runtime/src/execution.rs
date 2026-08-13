use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ActorKindV1, DurableEnvelopeV1, FenceKindV1, durable_envelope_v1::Semantics},
};
use makosh_obligations_api::{
    OBLIGATIONS_MODULE_ID_V1,
    client_wire::{ObligationChangedV1, ObligationStateV1},
    obligations_lifecycle_event_contract_reference_v1,
};
use makosh_risk_core::RiskProjectionEntryV1;
use makosh_risk_persistence::{
    ApplyRiskEntryV1, RiskEnvelopeRecordV1, RiskPersistenceErrorV1, RiskPersistenceV1,
    RiskReplayOutcomeV1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
use makosh_tasks_command_api::{
    TASKS_MODULE_ID_V1,
    client_wire::{TaskChangedV1, TaskStateV1},
    tasks_lifecycle_event_contract_reference_v1,
};
use prost::Message;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskSourceV1 {
    Tasks,
    Obligations,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskExecutionContextV1 {
    pub logical_owner_id: String,
    pub projection_generation: u64,
    pub now_unix_millis: i64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskExecutionOutcomeV1 {
    Applied,
    Replayed,
    Ignored,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskExecutionErrorV1 {
    InvalidContext,
    InvalidEnvelope,
    InvalidPayload,
    Persistence(RiskPersistenceErrorV1),
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
    cleared: bool,
    partition: [u8; 16],
    contract: ContractReferenceV1,
}

pub async fn process_risk_source_event_v1(
    persistence: &RiskPersistenceV1,
    record: &OutboxRecordV1,
    source: RiskSourceV1,
    context: &RiskExecutionContextV1,
) -> Result<RiskExecutionOutcomeV1, RiskExecutionErrorV1> {
    validate_context(context)?;
    let envelope: DurableEnvelopeV1 =
        exact_decode(record.exact_bytes(), RiskExecutionErrorV1::InvalidEnvelope)?;
    let Some(event) = normalize(source, &envelope)? else {
        return Ok(RiskExecutionOutcomeV1::Ignored);
    };
    validate_envelope(record, &envelope, &event, context)?;
    let Some(signal) = risk_signal_v1(
        event.source_owner,
        &event.state,
        event.cleared,
        millis(event.seconds, event.nanos)?,
    ) else {
        return Ok(RiskExecutionOutcomeV1::Ignored);
    };
    let outcome = persistence
        .apply_entry_once(&ApplyRiskEntryV1 {
            input: RiskEnvelopeRecordV1 {
                message_id: event.event_id,
                envelope_sha256: *record.envelope_sha256(),
                envelope_bytes: record.exact_bytes().to_vec(),
            },
            projection_generation: context.projection_generation,
            entry: RiskProjectionEntryV1 {
                event_id: event.event_id,
                logical_owner_id: event.owner,
                source_owner: event.source_owner.into(),
                entity_kind: event.kind.into(),
                entity_id: event.entity_id,
                source_revision: event.revision,
                reason_code: signal.reason_code.into(),
                severity: signal.severity,
                occurred_at_unix_millis: signal.occurred_at_unix_millis,
                expires_at_unix_millis: signal.expires_at_unix_millis,
                cleared: signal.cleared,
            },
            completed_at_unix_millis: context.now_unix_millis,
        })
        .await
        .map_err(RiskExecutionErrorV1::Persistence)?;
    Ok(match outcome {
        RiskReplayOutcomeV1::Applied => RiskExecutionOutcomeV1::Applied,
        RiskReplayOutcomeV1::Replayed => RiskExecutionOutcomeV1::Replayed,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RiskSignalV1 {
    reason_code: &'static str,
    severity: u32,
    occurred_at_unix_millis: i64,
    expires_at_unix_millis: i64,
    cleared: bool,
}

fn risk_signal_v1(
    source_owner: &str,
    state: &str,
    cleared: bool,
    occurred_at_unix_millis: i64,
) -> Option<RiskSignalV1> {
    let (reason_code, severity, horizon_days) = match (source_owner, state) {
        ("tasks", "task_state_open") => ("open_task", 1, 30),
        ("tasks", "task_state_in_progress") => ("in_progress_task", 1, 30),
        ("obligations", "obligation_state_open") => ("open_obligation", 3, 30),
        ("obligations", "obligation_state_breached") => ("breached_obligation", 5, 365),
        ("tasks", "task_state_completed" | "task_state_cancelled")
        | (
            "obligations",
            "obligation_state_fulfilled" | "obligation_state_waived" | "obligation_state_cancelled",
        ) => ("", 0, 0),
        _ => return None,
    };
    let cleared = cleared || reason_code.is_empty();
    let expires_at_unix_millis = if cleared {
        0
    } else {
        occurred_at_unix_millis.checked_add(i64::from(horizon_days) * 86_400_000)?
    };
    Some(RiskSignalV1 {
        reason_code,
        severity,
        occurred_at_unix_millis,
        expires_at_unix_millis,
        cleared,
    })
}

fn normalize(
    source: RiskSourceV1,
    envelope: &DurableEnvelopeV1,
) -> Result<Option<Event>, RiskExecutionErrorV1> {
    macro_rules! lifecycle {
        ($type:ty,$contract:expr,$module:expr,$owner:expr,$kind:expr,$id:ident,$revision:ident,$state:ident,$time:ident,$enum:ty) => {{
            let value: $type =
                exact_decode(&envelope.payload, RiskExecutionErrorV1::InvalidPayload)?;
            let state = <$enum>::try_from(value.$state)
                .map_err(|_| RiskExecutionErrorV1::InvalidPayload)?;
            if state as i32 == 0 {
                return Err(RiskExecutionErrorV1::InvalidPayload);
            }
            let time = value.$time.ok_or(RiskExecutionErrorV1::InvalidPayload)?;
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
                cleared: name.ends_with("_archived") || name.ends_with("_deleted"),
                partition: id16(&value.$id)?,
                contract: $contract,
            }))
        }};
    }
    match source {
        RiskSourceV1::Tasks => lifecycle!(
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
        RiskSourceV1::Obligations => lifecycle!(
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
    }
}
fn validate_envelope(
    record: &OutboxRecordV1,
    envelope: &DurableEnvelopeV1,
    event: &Event,
    context: &RiskExecutionContextV1,
) -> Result<(), RiskExecutionErrorV1> {
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(RiskExecutionErrorV1::InvalidEnvelope)?;
    let source = envelope
        .source
        .as_ref()
        .ok_or(RiskExecutionErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(RiskExecutionErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(RiskExecutionErrorV1::InvalidEnvelope)?;
    let recorded = envelope
        .recorded_at
        .as_ref()
        .ok_or(RiskExecutionErrorV1::InvalidEnvelope)?;
    let occurred = match envelope.semantics.as_ref() {
        Some(Semantics::Event(value)) => value.occurred_at.as_ref(),
        _ => None,
    }
    .ok_or(RiskExecutionErrorV1::InvalidEnvelope)?;
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
        return Err(RiskExecutionErrorV1::InvalidEnvelope);
    }
    Ok(())
}
fn validate_context(value: &RiskExecutionContextV1) -> Result<(), RiskExecutionErrorV1> {
    if value.logical_owner_id.is_empty()
        || value.projection_generation == 0
        || value.now_unix_millis <= 0
    {
        Err(RiskExecutionErrorV1::InvalidContext)
    } else {
        Ok(())
    }
}
fn exact_decode<M: Message + Default>(
    bytes: &[u8],
    error: RiskExecutionErrorV1,
) -> Result<M, RiskExecutionErrorV1> {
    let value = M::decode(bytes).map_err(|_| error)?;
    (value.encode_to_vec() == bytes)
        .then_some(value)
        .ok_or(error)
}
fn id16(value: &[u8]) -> Result<[u8; 16], RiskExecutionErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(RiskExecutionErrorV1::InvalidPayload)
}
fn positive(value: u64) -> Result<u64, RiskExecutionErrorV1> {
    (value > 0)
        .then_some(value)
        .ok_or(RiskExecutionErrorV1::InvalidPayload)
}
fn millis(seconds: i64, nanos: i32) -> Result<i64, RiskExecutionErrorV1> {
    if seconds <= 0 || !(0..1_000_000_000).contains(&nanos) || nanos % 1_000_000 != 0 {
        return Err(RiskExecutionErrorV1::InvalidEnvelope);
    }
    seconds
        .checked_mul(1000)
        .and_then(|v| v.checked_add(i64::from(nanos / 1_000_000)))
        .ok_or(RiskExecutionErrorV1::InvalidEnvelope)
}
#[cfg(test)]
mod tests {
    use super::*;
    use makosh_tasks_command_api::{
        TasksCommandEnvelopeContextV1, build_task_changed_outbox_record_v1,
        client_wire::{TaskChangedV1, TaskPriorityV1, TimestampV1},
    };
    #[test]
    fn canonical_task_event_maps_to_bounded_risk_signal() {
        let record = build_task_changed_outbox_record_v1(
            [1; 16],
            TaskChangedV1 {
                event_id: vec![2; 16],
                task_id: vec![3; 16],
                logical_owner_id: "owner-1".into(),
                task_revision: 2,
                state: TaskStateV1::TaskStateOpen as i32,
                priority: TaskPriorityV1::TaskPriorityNormal as i32,
                occurred_at: Some(TimestampV1 {
                    unix_seconds: 10,
                    nanos: 0,
                }),
            },
            &TasksCommandEnvelopeContextV1 {
                module_id: TASKS_MODULE_ID_V1.into(),
                runtime_instance_id: "runtime".into(),
                runtime_generation: 2,
                recorded_at_unix_seconds: 10,
                recorded_at_nanos: 0,
            },
        )
        .unwrap();
        let envelope: DurableEnvelopeV1 =
            exact_decode(record.exact_bytes(), RiskExecutionErrorV1::InvalidEnvelope).unwrap();
        let event = normalize(RiskSourceV1::Tasks, &envelope).unwrap().unwrap();
        assert_eq!(event.kind, "task");
        assert_eq!(event.revision, 2);
        assert_eq!(
            risk_signal_v1(event.source_owner, &event.state, event.cleared, 10_000)
                .expect("risk")
                .reason_code,
            "open_task"
        );
    }

    #[test]
    fn risk_mapping_is_closed_bounded_and_expiring() {
        let open = risk_signal_v1("obligations", "obligation_state_open", false, 10_000)
            .expect("open obligation");
        assert_eq!(open.reason_code, "open_obligation");
        assert_eq!(open.severity, 3);
        assert!(open.expires_at_unix_millis > open.occurred_at_unix_millis);
        let cleared = risk_signal_v1("obligations", "obligation_state_fulfilled", false, 20_000)
            .expect("clear signal");
        assert!(cleared.cleared);
        assert_eq!(cleared.severity, 0);
        assert_eq!(
            risk_signal_v1("projects", "project_state_active", false, 10_000),
            None
        );
    }
}
