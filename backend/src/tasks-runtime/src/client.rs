use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use makosh_tasks_command_api::{
    TASKS_MODULE_ID_V1, TASKS_OWNER_ID_V1, TasksCommandEnvelopeContextV1,
    build_task_changed_outbox_record_v1,
    client_wire::{
        AddChecklistItemRequestV1, AddTaskDependencyRequestV1, CreateTaskRequestV1,
        GetTaskRequestV1, ListTasksRequestV1, ListTasksResultV1, RemoveChecklistItemRequestV1,
        RemoveTaskDependencyRequestV1, SetTaskPriorityRequestV1, SetTaskStateRequestV1,
        TaskChangedV1, TaskChecklistItemV1 as WireChecklistItem,
        TaskDependencyV1 as WireDependency, TaskMutationResultV1, TaskPriorityV1 as WirePriority,
        TaskStateV1 as WireState, TaskSummaryV1, TimestampV1 as WireTimestamp,
        UpdateChecklistItemRequestV1, UpdateTaskRequestV1,
    },
    tasks_client_add_checklist_item_contract_reference_v1,
    tasks_client_add_dependency_contract_reference_v1, tasks_client_create_contract_reference_v1,
    tasks_client_get_contract_reference_v1, tasks_client_list_contract_reference_v1,
    tasks_client_remove_checklist_item_contract_reference_v1,
    tasks_client_remove_dependency_contract_reference_v1,
    tasks_client_set_priority_contract_reference_v1, tasks_client_set_state_contract_reference_v1,
    tasks_client_update_checklist_item_contract_reference_v1,
    tasks_client_update_contract_reference_v1,
};
use makosh_tasks_core::{
    ManualTaskDraftV1, TaskLifecycleStateV1, TaskPriorityV1, TaskRecordV1, TaskTimestampV1,
    derive_manual_task_id_v1,
};
use makosh_tasks_persistence::{
    TasksLifecycleCommitV1, TasksLifecycleMutationV1, TasksLifecycleOperationOutcomeV1,
    TasksLifecycleOperationV1, TasksOutboxRecordV1, TasksPersistenceErrorV1, TasksPersistenceV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub async fn dispatch_tasks_client_request_v1(
    persistence: &TasksPersistenceV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    now_unix_millis: i64,
) -> ModuleClientResponseV1 {
    let accepted_identity = request.protocol_major == 1
        && request.module_id == TASKS_MODULE_ID_V1
        && request.owner_id == TASKS_OWNER_ID_V1
        && request.logical_owner_id == logical_owner_id
        && !request.authenticated_device_id.is_empty()
        && !runtime_instance_id.is_empty()
        && runtime_generation > 0
        && now_unix_millis > 0;
    let response = if accepted_identity {
        dispatch(
            persistence,
            runtime_instance_id,
            runtime_generation,
            logical_owner_id,
            &request,
            now_unix_millis,
        )
        .await
    } else {
        Err("REJECTED")
    };
    match response {
        Ok(response_payload) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload,
            error_code: String::new(),
        },
        Err(error_code) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload: Vec::new(),
            error_code: error_code.to_owned(),
        },
    }
}

async fn dispatch(
    persistence: &TasksPersistenceV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    logical_owner_id: &str,
    request: &ModuleClientRequestV1,
    now_unix_millis: i64,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    if contract == &tasks_client_get_contract_reference_v1() {
        return get(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &tasks_client_list_contract_reference_v1() {
        return list(persistence, logical_owner_id, &request.request_payload).await;
    }

    let operation_id = decode_operation_id(contract, &request.request_payload)?;
    let request_sha256: [u8; 32] = Sha256::digest(&request.request_payload).into();
    if let Some(response) = persistence
        .load_lifecycle_operation_replay(
            logical_owner_id,
            operation_id,
            request_sha256,
            &request.request_payload,
        )
        .await
        .map_err(persistence_error)?
    {
        return Ok(response);
    }
    let mutation = decode_mutation(
        contract,
        logical_owner_id,
        &request.request_payload,
        now_unix_millis,
    )?;
    debug_assert_eq!(operation_id, mutation_operation_id(&mutation));
    let operation = TasksLifecycleOperationV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        operation_id,
        request_sha256,
        request_bytes: request.request_payload.clone(),
        received_at_unix_millis: now_unix_millis,
        mutation,
    };
    let context = TasksCommandEnvelopeContextV1 {
        module_id: TASKS_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime_instance_id.to_owned(),
        runtime_generation,
        recorded_at_unix_seconds: now_unix_millis / 1_000,
        recorded_at_nanos: ((now_unix_millis % 1_000) * 1_000_000) as i32,
    };
    let outcome = persistence
        .apply_lifecycle_operation(operation, |task| {
            let response = TaskMutationResultV1 {
                operation_id: operation_id.to_vec(),
                task: Some(summary(task)),
            }
            .encode_to_vec();
            let event_id = lifecycle_event_id(operation_id, task.task_id, task.task_revision);
            let event = build_task_changed_outbox_record_v1(
                operation_id,
                TaskChangedV1 {
                    event_id: event_id.to_vec(),
                    task_id: task.task_id.to_vec(),
                    logical_owner_id: task.logical_owner_id.clone(),
                    task_revision: task.task_revision,
                    state: encode_state(task.state),
                    priority: encode_priority(task.priority),
                    occurred_at: Some(timestamp(task.updated_at)),
                },
                &context,
            )
            .map_err(|_| TasksPersistenceErrorV1::InvalidInput)?;
            Ok(TasksLifecycleCommitV1 {
                response_sha256: Sha256::digest(&response).into(),
                response_bytes: response,
                lifecycle_event: TasksOutboxRecordV1 {
                    message_id: *event.message_id(),
                    envelope_sha256: *event.envelope_sha256(),
                    envelope_bytes: event.exact_bytes().to_vec(),
                },
            })
        })
        .await
        .map_err(persistence_error)?;
    Ok(match outcome {
        TasksLifecycleOperationOutcomeV1::Applied { response_bytes, .. }
        | TasksLifecycleOperationOutcomeV1::Replayed { response_bytes } => response_bytes,
    })
}

fn decode_operation_id(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    bytes: &[u8],
) -> Result<[u8; 16], &'static str> {
    macro_rules! operation_id {
        ($contract:expr, $type:ty) => {
            if contract == &$contract {
                let value = <$type>::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
                if value.encode_to_vec() != bytes {
                    return Err("INVALID_ARGUMENT");
                }
                return id16(&value.operation_id);
            }
        };
    }
    operation_id!(
        tasks_client_create_contract_reference_v1(),
        CreateTaskRequestV1
    );
    operation_id!(
        tasks_client_update_contract_reference_v1(),
        UpdateTaskRequestV1
    );
    operation_id!(
        tasks_client_set_state_contract_reference_v1(),
        SetTaskStateRequestV1
    );
    operation_id!(
        tasks_client_set_priority_contract_reference_v1(),
        SetTaskPriorityRequestV1
    );
    operation_id!(
        tasks_client_add_dependency_contract_reference_v1(),
        AddTaskDependencyRequestV1
    );
    operation_id!(
        tasks_client_remove_dependency_contract_reference_v1(),
        RemoveTaskDependencyRequestV1
    );
    operation_id!(
        tasks_client_add_checklist_item_contract_reference_v1(),
        AddChecklistItemRequestV1
    );
    operation_id!(
        tasks_client_update_checklist_item_contract_reference_v1(),
        UpdateChecklistItemRequestV1
    );
    operation_id!(
        tasks_client_remove_checklist_item_contract_reference_v1(),
        RemoveChecklistItemRequestV1
    );
    Err("REJECTED")
}

fn decode_mutation(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
    now_unix_millis: i64,
) -> Result<TasksLifecycleMutationV1, &'static str> {
    macro_rules! decode {
        ($type:ty) => {{
            let value = <$type>::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
            if value.encode_to_vec() != bytes {
                return Err("INVALID_ARGUMENT");
            }
            value
        }};
    }
    if contract == &tasks_client_create_contract_reference_v1() {
        let mut value = decode!(CreateTaskRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        let operation_id = id16(&value.operation_id)?;
        let created_at = checked_timestamp(value.created_at, now_unix_millis)?;
        let task_id = derive_manual_task_id_v1(logical_owner_id, &operation_id)
            .map_err(|_| "INVALID_ARGUMENT")?;
        if !value.task_id.is_empty() && value.task_id != task_id {
            return Err("INVALID_ARGUMENT");
        }
        Ok(TasksLifecycleMutationV1::Create(ManualTaskDraftV1 {
            operation_id,
            logical_owner_id: logical_owner_id.to_owned(),
            title: value.title,
            description: value.description,
            due_at: optional_due_timestamp(value.due_at)?,
            priority: decode_priority(value.priority)?,
            created_at,
        }))
    } else if contract == &tasks_client_update_contract_reference_v1() {
        let mut value = decode!(UpdateTaskRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        if value.clear_description && value.description.is_some()
            || value.clear_due_at && value.due_at.is_some()
        {
            return Err("INVALID_ARGUMENT");
        }
        Ok(TasksLifecycleMutationV1::Update {
            operation_id: id16(&value.operation_id)?,
            task_id: id16(&value.task_id)?,
            expected_revision: positive_revision(value.expected_task_revision)?,
            title: value.title,
            description: if value.clear_description {
                Some(None)
            } else {
                value.description.map(Some)
            },
            due_at: if value.clear_due_at {
                Some(None)
            } else {
                value
                    .due_at
                    .map(|time| decode_timestamp(Some(time)).map(Some))
                    .transpose()?
            },
            changed_at: checked_timestamp(value.updated_at, now_unix_millis)?,
        })
    } else if contract == &tasks_client_set_state_contract_reference_v1() {
        let mut value = decode!(SetTaskStateRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(TasksLifecycleMutationV1::SetState {
            operation_id: id16(&value.operation_id)?,
            task_id: id16(&value.task_id)?,
            expected_revision: positive_revision(value.expected_task_revision)?,
            state: decode_state(value.state)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &tasks_client_set_priority_contract_reference_v1() {
        let mut value = decode!(SetTaskPriorityRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(TasksLifecycleMutationV1::SetPriority {
            operation_id: id16(&value.operation_id)?,
            task_id: id16(&value.task_id)?,
            expected_revision: positive_revision(value.expected_task_revision)?,
            priority: decode_priority(value.priority)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &tasks_client_add_dependency_contract_reference_v1() {
        let mut value = decode!(AddTaskDependencyRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(TasksLifecycleMutationV1::AddDependency {
            operation_id: id16(&value.operation_id)?,
            task_id: id16(&value.task_id)?,
            expected_revision: positive_revision(value.expected_task_revision)?,
            dependency_id: id16(&value.dependency_id)?,
            depends_on_task_id: id16(&value.depends_on_task_id)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &tasks_client_remove_dependency_contract_reference_v1() {
        let mut value = decode!(RemoveTaskDependencyRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(TasksLifecycleMutationV1::RemoveDependency {
            operation_id: id16(&value.operation_id)?,
            task_id: id16(&value.task_id)?,
            expected_revision: positive_revision(value.expected_task_revision)?,
            dependency_id: id16(&value.dependency_id)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &tasks_client_add_checklist_item_contract_reference_v1() {
        let mut value = decode!(AddChecklistItemRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(TasksLifecycleMutationV1::AddChecklistItem {
            operation_id: id16(&value.operation_id)?,
            task_id: id16(&value.task_id)?,
            expected_revision: positive_revision(value.expected_task_revision)?,
            checklist_item_id: id16(&value.checklist_item_id)?,
            label: value.label,
            position: value.position,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &tasks_client_update_checklist_item_contract_reference_v1() {
        let mut value = decode!(UpdateChecklistItemRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(TasksLifecycleMutationV1::UpdateChecklistItem {
            operation_id: id16(&value.operation_id)?,
            task_id: id16(&value.task_id)?,
            expected_revision: positive_revision(value.expected_task_revision)?,
            checklist_item_id: id16(&value.checklist_item_id)?,
            label: value.label,
            completed: value.completed,
            position: value.position,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &tasks_client_remove_checklist_item_contract_reference_v1() {
        let mut value = decode!(RemoveChecklistItemRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(TasksLifecycleMutationV1::RemoveChecklistItem {
            operation_id: id16(&value.operation_id)?,
            task_id: id16(&value.task_id)?,
            expected_revision: positive_revision(value.expected_task_revision)?,
            checklist_item_id: id16(&value.checklist_item_id)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else {
        Err("REJECTED")
    }
}

async fn get(
    persistence: &TasksPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut request = GetTaskRequestV1::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
    if request.encode_to_vec() != bytes {
        return Err("INVALID_ARGUMENT");
    }
    accept_owner(&mut request.logical_owner_id, logical_owner_id)?;
    let task = persistence
        .get_lifecycle_task(logical_owner_id, id16(&request.task_id)?)
        .await
        .map_err(persistence_error)?
        .ok_or("NOT_FOUND")?;
    Ok(summary(&task).encode_to_vec())
}

async fn list(
    persistence: &TasksPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut request = ListTasksRequestV1::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
    if request.encode_to_vec() != bytes {
        return Err("INVALID_ARGUMENT");
    }
    accept_owner(&mut request.logical_owner_id, logical_owner_id)?;
    if !(1..=200).contains(&request.limit) {
        return Err("INVALID_ARGUMENT");
    }
    let after = if request.after_task_id.is_empty() {
        None
    } else {
        Some(id16(&request.after_task_id)?)
    };
    let mut tasks = persistence
        .list_lifecycle_tasks(
            logical_owner_id,
            after,
            u16::try_from(request.limit + 1).map_err(|_| "INVALID_ARGUMENT")?,
        )
        .await
        .map_err(persistence_error)?;
    let has_more = tasks.len() > request.limit as usize;
    tasks.truncate(request.limit as usize);
    let next = has_more
        .then(|| tasks.last().map(|task| task.task_id.to_vec()))
        .flatten()
        .unwrap_or_default();
    Ok(ListTasksResultV1 {
        tasks: tasks.iter().map(summary).collect(),
        next_after_task_id: next,
    }
    .encode_to_vec())
}

fn summary(task: &TaskRecordV1) -> TaskSummaryV1 {
    TaskSummaryV1 {
        task_id: task.task_id.to_vec(),
        logical_owner_id: task.logical_owner_id.clone(),
        title: task.title.clone(),
        description: task.description.clone(),
        due_at: task.due_at.map(timestamp),
        state: encode_state(task.state),
        priority: encode_priority(task.priority),
        task_revision: task.task_revision,
        dependencies: task
            .dependencies
            .iter()
            .map(|value| WireDependency {
                dependency_id: value.dependency_id.to_vec(),
                depends_on_task_id: value.depends_on_task_id.to_vec(),
                created_at_task_revision: value.created_at_task_revision,
            })
            .collect(),
        checklist: task
            .checklist
            .iter()
            .map(|value| WireChecklistItem {
                checklist_item_id: value.checklist_item_id.to_vec(),
                label: value.label.clone(),
                completed: value.completed,
                position: value.position,
                updated_at_task_revision: value.updated_at_task_revision,
            })
            .collect(),
        created_at: Some(timestamp(task.created_at)),
        updated_at: Some(timestamp(task.updated_at)),
    }
}

fn mutation_operation_id(value: &TasksLifecycleMutationV1) -> [u8; 16] {
    match value {
        TasksLifecycleMutationV1::Create(value) => value.operation_id,
        TasksLifecycleMutationV1::Update { operation_id, .. }
        | TasksLifecycleMutationV1::SetState { operation_id, .. }
        | TasksLifecycleMutationV1::SetPriority { operation_id, .. }
        | TasksLifecycleMutationV1::AddDependency { operation_id, .. }
        | TasksLifecycleMutationV1::RemoveDependency { operation_id, .. }
        | TasksLifecycleMutationV1::AddChecklistItem { operation_id, .. }
        | TasksLifecycleMutationV1::UpdateChecklistItem { operation_id, .. }
        | TasksLifecycleMutationV1::RemoveChecklistItem { operation_id, .. } => *operation_id,
    }
}

fn accept_owner(value: &mut String, logical_owner_id: &str) -> Result<(), &'static str> {
    if !value.is_empty() && value != logical_owner_id {
        return Err("REJECTED");
    }
    *value = logical_owner_id.to_owned();
    Ok(())
}

fn checked_timestamp(
    value: Option<WireTimestamp>,
    now_unix_millis: i64,
) -> Result<TaskTimestampV1, &'static str> {
    let value = value.ok_or("INVALID_ARGUMENT")?;
    if value.unix_seconds <= 0
        || !(0..1_000_000_000).contains(&value.nanos)
        || value.unix_seconds > now_unix_millis / 1_000
        || (value.unix_seconds == now_unix_millis / 1_000
            && i64::from(value.nanos) > (now_unix_millis % 1_000) * 1_000_000)
    {
        return Err("INVALID_ARGUMENT");
    }
    Ok(TaskTimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    })
}

fn optional_due_timestamp(
    value: Option<WireTimestamp>,
) -> Result<Option<TaskTimestampV1>, &'static str> {
    value.map(|value| decode_timestamp(Some(value))).transpose()
}

fn decode_timestamp(value: Option<WireTimestamp>) -> Result<TaskTimestampV1, &'static str> {
    let value = value.ok_or("INVALID_ARGUMENT")?;
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return Err("INVALID_ARGUMENT");
    }
    Ok(TaskTimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    })
}

fn timestamp(value: TaskTimestampV1) -> WireTimestamp {
    WireTimestamp {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    }
}

fn decode_state(value: i32) -> Result<TaskLifecycleStateV1, &'static str> {
    match WireState::try_from(value).map_err(|_| "INVALID_ARGUMENT")? {
        WireState::TaskStateOpen => Ok(TaskLifecycleStateV1::Open),
        WireState::TaskStateInProgress => Ok(TaskLifecycleStateV1::InProgress),
        WireState::TaskStateCompleted => Ok(TaskLifecycleStateV1::Completed),
        WireState::TaskStateCancelled => Ok(TaskLifecycleStateV1::Cancelled),
        WireState::TaskStateUnspecified => Err("INVALID_ARGUMENT"),
    }
}

fn encode_state(value: TaskLifecycleStateV1) -> i32 {
    match value {
        TaskLifecycleStateV1::Open => WireState::TaskStateOpen as i32,
        TaskLifecycleStateV1::InProgress => WireState::TaskStateInProgress as i32,
        TaskLifecycleStateV1::Completed => WireState::TaskStateCompleted as i32,
        TaskLifecycleStateV1::Cancelled => WireState::TaskStateCancelled as i32,
    }
}

fn decode_priority(value: i32) -> Result<TaskPriorityV1, &'static str> {
    match WirePriority::try_from(value).map_err(|_| "INVALID_ARGUMENT")? {
        WirePriority::TaskPriorityLow => Ok(TaskPriorityV1::Low),
        WirePriority::TaskPriorityNormal => Ok(TaskPriorityV1::Normal),
        WirePriority::TaskPriorityHigh => Ok(TaskPriorityV1::High),
        WirePriority::TaskPriorityUrgent => Ok(TaskPriorityV1::Urgent),
        WirePriority::TaskPriorityUnspecified => Err("INVALID_ARGUMENT"),
    }
}

fn encode_priority(value: TaskPriorityV1) -> i32 {
    match value {
        TaskPriorityV1::Low => WirePriority::TaskPriorityLow as i32,
        TaskPriorityV1::Normal => WirePriority::TaskPriorityNormal as i32,
        TaskPriorityV1::High => WirePriority::TaskPriorityHigh as i32,
        TaskPriorityV1::Urgent => WirePriority::TaskPriorityUrgent as i32,
    }
}

fn positive_revision(value: u64) -> Result<u64, &'static str> {
    (value > 0).then_some(value).ok_or("INVALID_ARGUMENT")
}

fn id16(value: &[u8]) -> Result<[u8; 16], &'static str> {
    let id: [u8; 16] = value.try_into().map_err(|_| "INVALID_ARGUMENT")?;
    id.iter()
        .any(|byte| *byte != 0)
        .then_some(id)
        .ok_or("INVALID_ARGUMENT")
}

fn lifecycle_event_id(operation_id: [u8; 16], task_id: [u8; 16], task_revision: u64) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.tasks.lifecycle-event-id.v1\0");
    hash.update(operation_id);
    hash.update(task_id);
    hash.update(task_revision.to_be_bytes());
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn persistence_error(value: TasksPersistenceErrorV1) -> &'static str {
    match value {
        TasksPersistenceErrorV1::NotFound => "NOT_FOUND",
        TasksPersistenceErrorV1::InvalidInput | TasksPersistenceErrorV1::InvalidRow => {
            "INVALID_ARGUMENT"
        }
        TasksPersistenceErrorV1::OperationConflict
        | TasksPersistenceErrorV1::RevisionConflict
        | TasksPersistenceErrorV1::DependencyCycle
        | TasksPersistenceErrorV1::CommandConflict
        | TasksPersistenceErrorV1::InboxConflict
        | TasksPersistenceErrorV1::TaskConflict => "FAILED_PRECONDITION",
        TasksPersistenceErrorV1::StorageUnavailable => "UNAVAILABLE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_cursor_is_last_returned_not_overflow() {
        let tasks = (1_u8..=3)
            .map(|id| TaskRecordV1 {
                task_id: [id; 16],
                logical_owner_id: "owner-1".to_owned(),
                title: format!("Task {id}"),
                description: None,
                due_at: None,
                state: TaskLifecycleStateV1::Open,
                priority: TaskPriorityV1::Normal,
                task_revision: 1,
                dependencies: Vec::new(),
                checklist: Vec::new(),
                created_at: TaskTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
                updated_at: TaskTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
            })
            .collect::<Vec<_>>();
        let mut page = tasks;
        let has_more = page.len() > 2;
        page.truncate(2);
        assert!(has_more);
        assert_eq!(page.last().expect("last").task_id, [2; 16]);
    }
}
