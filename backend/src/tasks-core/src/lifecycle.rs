use sha2::{Digest, Sha256};

use crate::{
    MAX_LOGICAL_OWNER_ID_BYTES_V1, MAX_TITLE_CHARS_V1, STABLE_ID_BYTES_V1, TaskTimestampV1,
};

pub const MAX_DESCRIPTION_CHARS_V1: usize = 4_000;
pub const MAX_CHECKLIST_LABEL_CHARS_V1: usize = 240;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskLifecycleStateV1 {
    Open,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskPriorityV1 {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskDependencyV1 {
    pub dependency_id: [u8; STABLE_ID_BYTES_V1],
    pub depends_on_task_id: [u8; STABLE_ID_BYTES_V1],
    pub created_at_task_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskChecklistItemV1 {
    pub checklist_item_id: [u8; STABLE_ID_BYTES_V1],
    pub label: String,
    pub completed: bool,
    pub position: u32,
    pub updated_at_task_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualTaskDraftV1 {
    pub operation_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<TaskTimestampV1>,
    pub priority: TaskPriorityV1,
    pub created_at: TaskTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskRecordV1 {
    pub task_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<TaskTimestampV1>,
    pub state: TaskLifecycleStateV1,
    pub priority: TaskPriorityV1,
    pub task_revision: u64,
    pub dependencies: Vec<TaskDependencyV1>,
    pub checklist: Vec<TaskChecklistItemV1>,
    pub created_at: TaskTimestampV1,
    pub updated_at: TaskTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskLifecycleErrorV1 {
    InvalidOwner,
    InvalidOperationId,
    InvalidTaskId,
    InvalidTitle,
    InvalidDescription,
    InvalidTimestamp,
    InvalidDependency,
    InvalidChecklistItem,
    RevisionConflict,
    RevisionOverflow,
    DuplicateDependency,
    SelfDependency,
    DependencyCycle,
    DependencyNotFound,
    ChecklistItemExists,
    ChecklistItemNotFound,
    InvalidStateTransition,
}

pub fn derive_manual_task_id_v1(
    logical_owner_id: &str,
    operation_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], TaskLifecycleErrorV1> {
    if !valid_owner(logical_owner_id) {
        return Err(TaskLifecycleErrorV1::InvalidOwner);
    }
    if !nonzero(operation_id) {
        return Err(TaskLifecycleErrorV1::InvalidOperationId);
    }
    let mut hash = Sha256::new();
    hash.update(b"makosh.tasks.manual-task-id.v1\0");
    hash.update((logical_owner_id.len() as u64).to_be_bytes());
    hash.update(logical_owner_id.as_bytes());
    hash.update(operation_id);
    Ok(hash.finalize()[..STABLE_ID_BYTES_V1]
        .try_into()
        .expect("fixed digest"))
}

pub fn create_manual_task_v1(
    draft: ManualTaskDraftV1,
) -> Result<TaskRecordV1, TaskLifecycleErrorV1> {
    validate_content(&draft.title, draft.description.as_deref())?;
    validate_timestamp(draft.created_at)?;
    if let Some(due_at) = draft.due_at {
        validate_timestamp(due_at)?;
    }
    let task = TaskRecordV1 {
        task_id: derive_manual_task_id_v1(&draft.logical_owner_id, &draft.operation_id)?,
        logical_owner_id: draft.logical_owner_id,
        title: draft.title,
        description: draft.description,
        due_at: draft.due_at,
        state: TaskLifecycleStateV1::Open,
        priority: draft.priority,
        task_revision: 1,
        dependencies: Vec::new(),
        checklist: Vec::new(),
        created_at: draft.created_at,
        updated_at: draft.created_at,
    };
    validate_task_record_v1(&task)?;
    Ok(task)
}

pub fn update_task_content_v1(
    task: &mut TaskRecordV1,
    expected_revision: u64,
    title: Option<String>,
    description: Option<Option<String>>,
    due_at: Option<Option<TaskTimestampV1>>,
    changed_at: TaskTimestampV1,
) -> Result<(), TaskLifecycleErrorV1> {
    let next_title = title.as_deref().unwrap_or(&task.title);
    let next_description = description
        .as_ref()
        .map_or(task.description.as_deref(), |value| value.as_deref());
    validate_content(next_title, next_description)?;
    if let Some(Some(value)) = due_at {
        validate_timestamp(value)?;
    }
    let revision = next_revision(task, expected_revision, changed_at)?;
    if let Some(value) = title {
        task.title = value;
    }
    if let Some(value) = description {
        task.description = value;
    }
    if let Some(value) = due_at {
        task.due_at = value;
    }
    apply_revision(task, revision, changed_at);
    Ok(())
}

pub fn set_task_state_v1(
    task: &mut TaskRecordV1,
    expected_revision: u64,
    state: TaskLifecycleStateV1,
    changed_at: TaskTimestampV1,
) -> Result<(), TaskLifecycleErrorV1> {
    if task.state == state || !valid_transition(task.state, state) {
        return Err(TaskLifecycleErrorV1::InvalidStateTransition);
    }
    let revision = next_revision(task, expected_revision, changed_at)?;
    task.state = state;
    apply_revision(task, revision, changed_at);
    Ok(())
}

pub fn set_task_priority_v1(
    task: &mut TaskRecordV1,
    expected_revision: u64,
    priority: TaskPriorityV1,
    changed_at: TaskTimestampV1,
) -> Result<(), TaskLifecycleErrorV1> {
    let revision = next_revision(task, expected_revision, changed_at)?;
    task.priority = priority;
    apply_revision(task, revision, changed_at);
    Ok(())
}

pub fn add_task_dependency_v1(
    task: &mut TaskRecordV1,
    expected_revision: u64,
    dependency_id: [u8; STABLE_ID_BYTES_V1],
    depends_on_task_id: [u8; STABLE_ID_BYTES_V1],
    would_create_cycle: bool,
    changed_at: TaskTimestampV1,
) -> Result<(), TaskLifecycleErrorV1> {
    if !nonzero(&dependency_id) || !nonzero(&depends_on_task_id) {
        return Err(TaskLifecycleErrorV1::InvalidDependency);
    }
    if task.task_id == depends_on_task_id {
        return Err(TaskLifecycleErrorV1::SelfDependency);
    }
    if would_create_cycle {
        return Err(TaskLifecycleErrorV1::DependencyCycle);
    }
    if task.dependencies.iter().any(|value| {
        value.dependency_id == dependency_id || value.depends_on_task_id == depends_on_task_id
    }) {
        return Err(TaskLifecycleErrorV1::DuplicateDependency);
    }
    let revision = next_revision(task, expected_revision, changed_at)?;
    task.dependencies.push(TaskDependencyV1 {
        dependency_id,
        depends_on_task_id,
        created_at_task_revision: revision,
    });
    task.dependencies.sort_by_key(|value| value.dependency_id);
    apply_revision(task, revision, changed_at);
    Ok(())
}

pub fn remove_task_dependency_v1(
    task: &mut TaskRecordV1,
    expected_revision: u64,
    dependency_id: [u8; STABLE_ID_BYTES_V1],
    changed_at: TaskTimestampV1,
) -> Result<(), TaskLifecycleErrorV1> {
    let Some(index) = task
        .dependencies
        .iter()
        .position(|value| value.dependency_id == dependency_id)
    else {
        return Err(TaskLifecycleErrorV1::DependencyNotFound);
    };
    let revision = next_revision(task, expected_revision, changed_at)?;
    task.dependencies.remove(index);
    apply_revision(task, revision, changed_at);
    Ok(())
}

pub fn add_checklist_item_v1(
    task: &mut TaskRecordV1,
    expected_revision: u64,
    checklist_item_id: [u8; STABLE_ID_BYTES_V1],
    label: String,
    position: u32,
    changed_at: TaskTimestampV1,
) -> Result<(), TaskLifecycleErrorV1> {
    if !nonzero(&checklist_item_id) || !valid_text(&label, MAX_CHECKLIST_LABEL_CHARS_V1) {
        return Err(TaskLifecycleErrorV1::InvalidChecklistItem);
    }
    if task
        .checklist
        .iter()
        .any(|value| value.checklist_item_id == checklist_item_id)
    {
        return Err(TaskLifecycleErrorV1::ChecklistItemExists);
    }
    let revision = next_revision(task, expected_revision, changed_at)?;
    task.checklist.push(TaskChecklistItemV1 {
        checklist_item_id,
        label,
        completed: false,
        position,
        updated_at_task_revision: revision,
    });
    sort_checklist(&mut task.checklist);
    apply_revision(task, revision, changed_at);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn update_checklist_item_v1(
    task: &mut TaskRecordV1,
    expected_revision: u64,
    checklist_item_id: [u8; STABLE_ID_BYTES_V1],
    label: Option<String>,
    completed: Option<bool>,
    position: Option<u32>,
    changed_at: TaskTimestampV1,
) -> Result<(), TaskLifecycleErrorV1> {
    if label
        .as_deref()
        .is_some_and(|value| !valid_text(value, MAX_CHECKLIST_LABEL_CHARS_V1))
        || (label.is_none() && completed.is_none() && position.is_none())
    {
        return Err(TaskLifecycleErrorV1::InvalidChecklistItem);
    }
    let index = task
        .checklist
        .iter()
        .position(|value| value.checklist_item_id == checklist_item_id)
        .ok_or(TaskLifecycleErrorV1::ChecklistItemNotFound)?;
    let revision = next_revision(task, expected_revision, changed_at)?;
    let item = &mut task.checklist[index];
    if let Some(value) = label {
        item.label = value;
    }
    if let Some(value) = completed {
        item.completed = value;
    }
    if let Some(value) = position {
        item.position = value;
    }
    item.updated_at_task_revision = revision;
    sort_checklist(&mut task.checklist);
    apply_revision(task, revision, changed_at);
    Ok(())
}

pub fn remove_checklist_item_v1(
    task: &mut TaskRecordV1,
    expected_revision: u64,
    checklist_item_id: [u8; STABLE_ID_BYTES_V1],
    changed_at: TaskTimestampV1,
) -> Result<(), TaskLifecycleErrorV1> {
    let index = task
        .checklist
        .iter()
        .position(|value| value.checklist_item_id == checklist_item_id)
        .ok_or(TaskLifecycleErrorV1::ChecklistItemNotFound)?;
    let revision = next_revision(task, expected_revision, changed_at)?;
    task.checklist.remove(index);
    apply_revision(task, revision, changed_at);
    Ok(())
}

pub fn validate_task_record_v1(task: &TaskRecordV1) -> Result<(), TaskLifecycleErrorV1> {
    if !valid_owner(&task.logical_owner_id) {
        return Err(TaskLifecycleErrorV1::InvalidOwner);
    }
    if !nonzero(&task.task_id) || task.task_revision == 0 {
        return Err(TaskLifecycleErrorV1::InvalidTaskId);
    }
    validate_content(&task.title, task.description.as_deref())?;
    validate_timestamp(task.created_at)?;
    validate_timestamp(task.updated_at)?;
    if timestamp_key(task.updated_at) < timestamp_key(task.created_at) {
        return Err(TaskLifecycleErrorV1::InvalidTimestamp);
    }
    if task
        .due_at
        .is_some_and(|value| validate_timestamp(value).is_err())
    {
        return Err(TaskLifecycleErrorV1::InvalidTimestamp);
    }
    if task.dependencies.iter().any(|value| {
        !nonzero(&value.dependency_id)
            || !nonzero(&value.depends_on_task_id)
            || value.depends_on_task_id == task.task_id
            || value.created_at_task_revision == 0
            || value.created_at_task_revision > task.task_revision
    }) {
        return Err(TaskLifecycleErrorV1::InvalidDependency);
    }
    if task.checklist.iter().any(|value| {
        !nonzero(&value.checklist_item_id)
            || !valid_text(&value.label, MAX_CHECKLIST_LABEL_CHARS_V1)
            || value.updated_at_task_revision == 0
            || value.updated_at_task_revision > task.task_revision
    }) {
        return Err(TaskLifecycleErrorV1::InvalidChecklistItem);
    }
    Ok(())
}

fn next_revision(
    task: &TaskRecordV1,
    expected_revision: u64,
    changed_at: TaskTimestampV1,
) -> Result<u64, TaskLifecycleErrorV1> {
    if expected_revision == 0 || task.task_revision != expected_revision {
        return Err(TaskLifecycleErrorV1::RevisionConflict);
    }
    validate_timestamp(changed_at)?;
    if timestamp_key(changed_at) < timestamp_key(task.updated_at) {
        return Err(TaskLifecycleErrorV1::InvalidTimestamp);
    }
    task.task_revision
        .checked_add(1)
        .ok_or(TaskLifecycleErrorV1::RevisionOverflow)
}

fn apply_revision(task: &mut TaskRecordV1, revision: u64, changed_at: TaskTimestampV1) {
    task.task_revision = revision;
    task.updated_at = changed_at;
}

fn valid_transition(from: TaskLifecycleStateV1, to: TaskLifecycleStateV1) -> bool {
    matches!(
        (from, to),
        (
            TaskLifecycleStateV1::Open,
            TaskLifecycleStateV1::InProgress
                | TaskLifecycleStateV1::Completed
                | TaskLifecycleStateV1::Cancelled
        ) | (
            TaskLifecycleStateV1::InProgress,
            TaskLifecycleStateV1::Open
                | TaskLifecycleStateV1::Completed
                | TaskLifecycleStateV1::Cancelled
        ) | (
            TaskLifecycleStateV1::Completed | TaskLifecycleStateV1::Cancelled,
            TaskLifecycleStateV1::Open
        )
    )
}

fn validate_content(title: &str, description: Option<&str>) -> Result<(), TaskLifecycleErrorV1> {
    if !valid_text(title, MAX_TITLE_CHARS_V1) {
        return Err(TaskLifecycleErrorV1::InvalidTitle);
    }
    if description.is_some_and(|value| !valid_text(value, MAX_DESCRIPTION_CHARS_V1)) {
        return Err(TaskLifecycleErrorV1::InvalidDescription);
    }
    Ok(())
}

fn validate_timestamp(value: TaskTimestampV1) -> Result<(), TaskLifecycleErrorV1> {
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return Err(TaskLifecycleErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn timestamp_key(value: TaskTimestampV1) -> (i64, i32) {
    (value.unix_seconds, value.nanos)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_LOGICAL_OWNER_ID_BYTES_V1
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn sort_checklist(items: &mut [TaskChecklistItemV1]) {
    items.sort_by_key(|value| (value.position, value.checklist_item_id));
}
