use makosh_tasks_core::{
    TaskChecklistItemV1, TaskDependencyV1, TaskLifecycleErrorV1, TaskLifecycleStateV1,
    TaskPriorityV1, TaskRecordV1, TaskTimestampV1, add_checklist_item_v1, add_task_dependency_v1,
    create_manual_task_v1, remove_checklist_item_v1, remove_task_dependency_v1,
    set_task_priority_v1, set_task_state_v1, update_checklist_item_v1, update_task_content_v1,
    validate_task_record_v1,
};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    TasksLifecycleCommitV1, TasksLifecycleMutationV1, TasksLifecycleOperationOutcomeV1,
    TasksLifecycleOperationV1, TasksPersistenceErrorV1, TasksPersistenceV1,
    model::{valid_lifecycle_commit, valid_lifecycle_operation},
};

impl TasksPersistenceV1 {
    pub async fn load_lifecycle_operation_replay(
        &self,
        logical_owner_id: &str,
        operation_id: [u8; 16],
        request_sha256: [u8; 32],
        request_bytes: &[u8],
    ) -> Result<Option<Vec<u8>>, TasksPersistenceErrorV1> {
        if operation_id.iter().all(|byte| *byte == 0)
            || request_sha256.iter().all(|byte| *byte == 0)
            || request_bytes.is_empty()
            || request_bytes.len() > crate::model::TASKS_MAX_CLIENT_MESSAGE_BYTES_V1
            || Sha256::digest(request_bytes).as_slice() != request_sha256
        {
            return Err(TasksPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT request_sha256, request_bytes, response_sha256, response_bytes \
             FROM makosh_data.tasks_client_operations \
             WHERE logical_owner_id=$1 AND operation_id=$2",
        )
        .bind(logical_owner_id)
        .bind(operation_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let response = row
            .map(|row| {
                let stored_request_sha =
                    fixed::<32>(row.try_get("request_sha256").map_err(storage)?)?;
                let stored_request_bytes: Vec<u8> =
                    row.try_get("request_bytes").map_err(storage)?;
                let response_sha = fixed::<32>(row.try_get("response_sha256").map_err(storage)?)?;
                let response_bytes: Vec<u8> = row.try_get("response_bytes").map_err(storage)?;
                if stored_request_sha != request_sha256
                    || stored_request_bytes != request_bytes
                    || Sha256::digest(&response_bytes).as_slice() != response_sha
                {
                    return Err(TasksPersistenceErrorV1::OperationConflict);
                }
                Ok(response_bytes)
            })
            .transpose()?;
        transaction.commit().await.map_err(storage)?;
        Ok(response)
    }

    pub async fn apply_lifecycle_operation<F>(
        &self,
        input: TasksLifecycleOperationV1,
        build_commit: F,
    ) -> Result<TasksLifecycleOperationOutcomeV1, TasksPersistenceErrorV1>
    where
        F: FnOnce(&TaskRecordV1) -> Result<TasksLifecycleCommitV1, TasksPersistenceErrorV1>,
    {
        if !valid_lifecycle_operation(&input) {
            return Err(TasksPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1 || encode($2, 'hex'), 0))")
            .bind(&input.logical_owner_id)
            .bind(input.operation_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        if matches!(
            &input.mutation,
            TasksLifecycleMutationV1::AddDependency { .. }
                | TasksLifecycleMutationV1::RemoveDependency { .. }
        ) {
            sqlx::query(
                "SELECT pg_advisory_xact_lock(hashtextextended('tasks.dependency-graph.v1:' || $1, 0))",
            )
            .bind(&input.logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        }
        if let Some(response) = load_operation_replay(&mut transaction, &input).await? {
            transaction.commit().await.map_err(storage)?;
            return Ok(TasksLifecycleOperationOutcomeV1::Replayed {
                response_bytes: response,
            });
        }

        let mut task = match &input.mutation {
            TasksLifecycleMutationV1::Create(draft) => {
                if draft.logical_owner_id != input.logical_owner_id
                    || draft.operation_id != input.operation_id
                {
                    return Err(TasksPersistenceErrorV1::InvalidInput);
                }
                create_manual_task_v1(draft.clone()).map_err(core_error)?
            }
            mutation => {
                let task_id = mutation_task_id(mutation);
                load_task(&mut transaction, &input.logical_owner_id, task_id, true)
                    .await?
                    .ok_or(TasksPersistenceErrorV1::NotFound)?
            }
        };

        match &input.mutation {
            TasksLifecycleMutationV1::Create(_) => {}
            TasksLifecycleMutationV1::Update {
                expected_revision,
                title,
                description,
                due_at,
                changed_at,
                ..
            } => update_task_content_v1(
                &mut task,
                *expected_revision,
                title.clone(),
                description.clone(),
                *due_at,
                *changed_at,
            )
            .map_err(core_error)?,
            TasksLifecycleMutationV1::SetState {
                expected_revision,
                state,
                changed_at,
                ..
            } => set_task_state_v1(&mut task, *expected_revision, *state, *changed_at)
                .map_err(core_error)?,
            TasksLifecycleMutationV1::SetPriority {
                expected_revision,
                priority,
                changed_at,
                ..
            } => set_task_priority_v1(&mut task, *expected_revision, *priority, *changed_at)
                .map_err(core_error)?,
            TasksLifecycleMutationV1::AddDependency {
                expected_revision,
                dependency_id,
                depends_on_task_id,
                changed_at,
                ..
            } => {
                if load_task(
                    &mut transaction,
                    &input.logical_owner_id,
                    *depends_on_task_id,
                    false,
                )
                .await?
                .is_none()
                {
                    return Err(TasksPersistenceErrorV1::NotFound);
                }
                let cycle = dependency_reaches(
                    &mut transaction,
                    &input.logical_owner_id,
                    *depends_on_task_id,
                    task.task_id,
                )
                .await?;
                add_task_dependency_v1(
                    &mut task,
                    *expected_revision,
                    *dependency_id,
                    *depends_on_task_id,
                    cycle,
                    *changed_at,
                )
                .map_err(core_error)?;
            }
            TasksLifecycleMutationV1::RemoveDependency {
                expected_revision,
                dependency_id,
                changed_at,
                ..
            } => remove_task_dependency_v1(
                &mut task,
                *expected_revision,
                *dependency_id,
                *changed_at,
            )
            .map_err(core_error)?,
            TasksLifecycleMutationV1::AddChecklistItem {
                expected_revision,
                checklist_item_id,
                label,
                position,
                changed_at,
                ..
            } => add_checklist_item_v1(
                &mut task,
                *expected_revision,
                *checklist_item_id,
                label.clone(),
                *position,
                *changed_at,
            )
            .map_err(core_error)?,
            TasksLifecycleMutationV1::UpdateChecklistItem {
                expected_revision,
                checklist_item_id,
                label,
                completed,
                position,
                changed_at,
                ..
            } => update_checklist_item_v1(
                &mut task,
                *expected_revision,
                *checklist_item_id,
                label.clone(),
                *completed,
                *position,
                *changed_at,
            )
            .map_err(core_error)?,
            TasksLifecycleMutationV1::RemoveChecklistItem {
                expected_revision,
                checklist_item_id,
                changed_at,
                ..
            } => remove_checklist_item_v1(
                &mut task,
                *expected_revision,
                *checklist_item_id,
                *changed_at,
            )
            .map_err(core_error)?,
        }
        validate_task_record_v1(&task).map_err(core_error)?;

        persist_task(
            &mut transaction,
            &task,
            matches!(input.mutation, TasksLifecycleMutationV1::Create(_)),
        )
        .await?;
        let commit = build_commit(&task)?;
        if !valid_lifecycle_commit(&commit) {
            return Err(TasksPersistenceErrorV1::InvalidInput);
        }
        insert_event(
            &mut transaction,
            &input.logical_owner_id,
            &commit,
            input.received_at_unix_millis,
        )
        .await?;
        insert_operation(&mut transaction, &input, &task, &commit).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(TasksLifecycleOperationOutcomeV1::Applied {
            task: Box::new(task),
            response_bytes: commit.response_bytes,
        })
    }

    pub async fn get_lifecycle_task(
        &self,
        logical_owner_id: &str,
        task_id: [u8; 16],
    ) -> Result<Option<TaskRecordV1>, TasksPersistenceErrorV1> {
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let task = load_task(&mut transaction, logical_owner_id, task_id, false).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(task)
    }

    pub async fn list_lifecycle_tasks(
        &self,
        logical_owner_id: &str,
        after_task_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<TaskRecordV1>, TasksPersistenceErrorV1> {
        if limit == 0 || limit > 201 {
            return Err(TasksPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let rows = sqlx::query(
            "SELECT task_id FROM makosh_data.tasks_state \
             WHERE logical_owner_id = $1 AND ($2::bytea IS NULL OR task_id > $2) \
             ORDER BY task_id LIMIT $3",
        )
        .bind(logical_owner_id)
        .bind(after_task_id.map(|value| value.to_vec()))
        .bind(i64::from(limit))
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage)?;
        let mut tasks = Vec::with_capacity(rows.len());
        for row in rows {
            let task_id = fixed::<16>(row.try_get("task_id").map_err(storage)?)?;
            tasks.push(
                load_task(&mut transaction, logical_owner_id, task_id, false)
                    .await?
                    .ok_or(TasksPersistenceErrorV1::InvalidRow)?,
            );
        }
        transaction.commit().await.map_err(storage)?;
        Ok(tasks)
    }
}

async fn load_operation_replay(
    transaction: &mut Transaction<'_, Postgres>,
    input: &TasksLifecycleOperationV1,
) -> Result<Option<Vec<u8>>, TasksPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT operation_kind, request_sha256, request_bytes, response_sha256, response_bytes \
         FROM makosh_data.tasks_client_operations \
         WHERE logical_owner_id = $1 AND operation_id = $2 FOR UPDATE",
    )
    .bind(&input.logical_owner_id)
    .bind(input.operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let request_sha = fixed::<32>(row.try_get("request_sha256").map_err(storage)?)?;
    let request_bytes: Vec<u8> = row.try_get("request_bytes").map_err(storage)?;
    let response_sha = fixed::<32>(row.try_get("response_sha256").map_err(storage)?)?;
    let response_bytes: Vec<u8> = row.try_get("response_bytes").map_err(storage)?;
    if row.try_get::<i16, _>("operation_kind").map_err(storage)? != input.mutation.operation_kind()
        || request_sha != input.request_sha256
        || request_bytes != input.request_bytes
        || Sha256::digest(&response_bytes).as_slice() != response_sha
    {
        return Err(TasksPersistenceErrorV1::OperationConflict);
    }
    Ok(Some(response_bytes))
}

async fn load_task(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    task_id: [u8; 16],
    lock: bool,
) -> Result<Option<TaskRecordV1>, TasksPersistenceErrorV1> {
    let sql = if lock {
        "SELECT task_id, logical_owner_id, title, description, due_at_unix_seconds, due_at_nanos, \
         status, priority, task_revision, created_at_unix_seconds, created_at_nanos, \
         updated_at_unix_seconds, updated_at_nanos FROM makosh_data.tasks_state \
         WHERE logical_owner_id = $1 AND task_id = $2 FOR UPDATE"
    } else {
        "SELECT task_id, logical_owner_id, title, description, due_at_unix_seconds, due_at_nanos, \
         status, priority, task_revision, created_at_unix_seconds, created_at_nanos, \
         updated_at_unix_seconds, updated_at_nanos FROM makosh_data.tasks_state \
         WHERE logical_owner_id = $1 AND task_id = $2"
    };
    let Some(row) = sqlx::query(sql)
        .bind(logical_owner_id)
        .bind(task_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?
    else {
        return Ok(None);
    };
    let dependencies = sqlx::query(
        "SELECT dependency_id, depends_on_task_id, created_at_task_revision \
         FROM makosh_data.tasks_dependencies WHERE logical_owner_id = $1 AND task_id = $2 \
         ORDER BY dependency_id",
    )
    .bind(logical_owner_id)
    .bind(task_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?
    .into_iter()
    .map(|row| {
        Ok(TaskDependencyV1 {
            dependency_id: fixed(row.try_get("dependency_id").map_err(storage)?)?,
            depends_on_task_id: fixed(row.try_get("depends_on_task_id").map_err(storage)?)?,
            created_at_task_revision: positive_u64(
                row.try_get("created_at_task_revision").map_err(storage)?,
            )?,
        })
    })
    .collect::<Result<Vec<_>, TasksPersistenceErrorV1>>()?;
    let checklist = sqlx::query(
        "SELECT checklist_item_id, label, completed, position, updated_at_task_revision \
         FROM makosh_data.tasks_checklist WHERE logical_owner_id = $1 AND task_id = $2 \
         ORDER BY position, checklist_item_id",
    )
    .bind(logical_owner_id)
    .bind(task_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?
    .into_iter()
    .map(|row| {
        Ok(TaskChecklistItemV1 {
            checklist_item_id: fixed(row.try_get("checklist_item_id").map_err(storage)?)?,
            label: row.try_get("label").map_err(storage)?,
            completed: row.try_get("completed").map_err(storage)?,
            position: u32::try_from(row.try_get::<i32, _>("position").map_err(storage)?)
                .map_err(|_| TasksPersistenceErrorV1::InvalidRow)?,
            updated_at_task_revision: positive_u64(
                row.try_get("updated_at_task_revision").map_err(storage)?,
            )?,
        })
    })
    .collect::<Result<Vec<_>, TasksPersistenceErrorV1>>()?;
    let due_seconds: Option<i64> = row.try_get("due_at_unix_seconds").map_err(storage)?;
    let due_nanos: Option<i32> = row.try_get("due_at_nanos").map_err(storage)?;
    let due_at = match (due_seconds, due_nanos) {
        (None, None) => None,
        (Some(unix_seconds), Some(nanos)) => Some(TaskTimestampV1 {
            unix_seconds,
            nanos,
        }),
        _ => return Err(TasksPersistenceErrorV1::InvalidRow),
    };
    let task = TaskRecordV1 {
        task_id: fixed(row.try_get("task_id").map_err(storage)?)?,
        logical_owner_id: row.try_get("logical_owner_id").map_err(storage)?,
        title: row.try_get("title").map_err(storage)?,
        description: row.try_get("description").map_err(storage)?,
        due_at,
        state: decode_state(row.try_get("status").map_err(storage)?)?,
        priority: decode_priority(row.try_get("priority").map_err(storage)?)?,
        task_revision: positive_u64(row.try_get("task_revision").map_err(storage)?)?,
        dependencies,
        checklist,
        created_at: TaskTimestampV1 {
            unix_seconds: row.try_get("created_at_unix_seconds").map_err(storage)?,
            nanos: row.try_get("created_at_nanos").map_err(storage)?,
        },
        updated_at: TaskTimestampV1 {
            unix_seconds: row.try_get("updated_at_unix_seconds").map_err(storage)?,
            nanos: row.try_get("updated_at_nanos").map_err(storage)?,
        },
    };
    validate_task_record_v1(&task).map_err(core_error)?;
    Ok(Some(task))
}

async fn persist_task(
    transaction: &mut Transaction<'_, Postgres>,
    task: &TaskRecordV1,
    create: bool,
) -> Result<(), TasksPersistenceErrorV1> {
    if create {
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.tasks_state (logical_owner_id, task_id, title, description, \
             due_at_unix_seconds, due_at_nanos, status, priority, task_revision, \
             created_at_unix_seconds, created_at_nanos, updated_at_unix_seconds, updated_at_nanos) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) ON CONFLICT DO NOTHING",
        )
        .bind(&task.logical_owner_id)
        .bind(task.task_id.as_slice())
        .bind(&task.title)
        .bind(&task.description)
        .bind(task.due_at.map(|value| value.unix_seconds))
        .bind(task.due_at.map(|value| value.nanos))
        .bind(encode_state(task.state))
        .bind(encode_priority(task.priority))
        .bind(i64_value(task.task_revision)?)
        .bind(task.created_at.unix_seconds)
        .bind(task.created_at.nanos)
        .bind(task.updated_at.unix_seconds)
        .bind(task.updated_at.nanos)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
        if inserted.rows_affected() != 1 {
            return Err(TasksPersistenceErrorV1::TaskConflict);
        }
    } else {
        let updated = sqlx::query(
            "UPDATE makosh_data.tasks_state SET title=$3, description=$4, due_at_unix_seconds=$5, \
             due_at_nanos=$6, status=$7, priority=$8, task_revision=$9, \
             updated_at_unix_seconds=$10, updated_at_nanos=$11 \
             WHERE logical_owner_id=$1 AND task_id=$2 AND task_revision=$12",
        )
        .bind(&task.logical_owner_id)
        .bind(task.task_id.as_slice())
        .bind(&task.title)
        .bind(&task.description)
        .bind(task.due_at.map(|value| value.unix_seconds))
        .bind(task.due_at.map(|value| value.nanos))
        .bind(encode_state(task.state))
        .bind(encode_priority(task.priority))
        .bind(i64_value(task.task_revision)?)
        .bind(task.updated_at.unix_seconds)
        .bind(task.updated_at.nanos)
        .bind(i64_value(task.task_revision - 1)?)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
        if updated.rows_affected() != 1 {
            return Err(TasksPersistenceErrorV1::RevisionConflict);
        }
    }
    sqlx::query(
        "DELETE FROM makosh_data.tasks_dependencies WHERE logical_owner_id=$1 AND task_id=$2",
    )
    .bind(&task.logical_owner_id)
    .bind(task.task_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    for dependency in &task.dependencies {
        sqlx::query(
            "INSERT INTO makosh_data.tasks_dependencies (logical_owner_id, task_id, dependency_id, \
             depends_on_task_id, created_at_task_revision) VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(&task.logical_owner_id)
        .bind(task.task_id.as_slice())
        .bind(dependency.dependency_id.as_slice())
        .bind(dependency.depends_on_task_id.as_slice())
        .bind(i64_value(dependency.created_at_task_revision)?)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
    }
    sqlx::query("DELETE FROM makosh_data.tasks_checklist WHERE logical_owner_id=$1 AND task_id=$2")
        .bind(&task.logical_owner_id)
        .bind(task.task_id.as_slice())
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
    for item in &task.checklist {
        sqlx::query(
            "INSERT INTO makosh_data.tasks_checklist (logical_owner_id, task_id, checklist_item_id, \
             label, completed, position, updated_at_task_revision) VALUES ($1,$2,$3,$4,$5,$6,$7)",
        )
        .bind(&task.logical_owner_id)
        .bind(task.task_id.as_slice())
        .bind(item.checklist_item_id.as_slice())
        .bind(&item.label)
        .bind(item.completed)
        .bind(i32::try_from(item.position).map_err(|_| TasksPersistenceErrorV1::InvalidInput)?)
        .bind(i64_value(item.updated_at_task_revision)?)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
    }
    Ok(())
}

async fn insert_event(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    commit: &TasksLifecycleCommitV1,
    created_at_unix_millis: i64,
) -> Result<(), TasksPersistenceErrorV1> {
    let result = sqlx::query(
        "INSERT INTO makosh_data.tasks_outbox (logical_owner_id, message_id, envelope_sha256, \
         envelope_bytes, created_at_unix_millis) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(commit.lifecycle_event.message_id.as_slice())
    .bind(commit.lifecycle_event.envelope_sha256.as_slice())
    .bind(&commit.lifecycle_event.envelope_bytes)
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(TasksPersistenceErrorV1::InboxConflict);
    }
    Ok(())
}

async fn insert_operation(
    transaction: &mut Transaction<'_, Postgres>,
    input: &TasksLifecycleOperationV1,
    task: &TaskRecordV1,
    commit: &TasksLifecycleCommitV1,
) -> Result<(), TasksPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.tasks_client_operations (logical_owner_id, operation_id, \
         operation_kind, request_sha256, request_bytes, task_id, task_revision, response_sha256, \
         response_bytes, received_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
    )
    .bind(&input.logical_owner_id)
    .bind(input.operation_id.as_slice())
    .bind(input.mutation.operation_kind())
    .bind(input.request_sha256.as_slice())
    .bind(&input.request_bytes)
    .bind(task.task_id.as_slice())
    .bind(i64_value(task.task_revision)?)
    .bind(commit.response_sha256.as_slice())
    .bind(&commit.response_bytes)
    .bind(input.received_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn dependency_reaches(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    from_task_id: [u8; 16],
    target_task_id: [u8; 16],
) -> Result<bool, TasksPersistenceErrorV1> {
    sqlx::query_scalar::<_, bool>(
        "WITH RECURSIVE reachable(task_id) AS ( \
            SELECT depends_on_task_id FROM makosh_data.tasks_dependencies \
            WHERE logical_owner_id=$1 AND task_id=$2 \
            UNION \
            SELECT dependency.depends_on_task_id FROM makosh_data.tasks_dependencies dependency \
            JOIN reachable ON dependency.task_id=reachable.task_id \
            WHERE dependency.logical_owner_id=$1 \
         ) SELECT EXISTS(SELECT 1 FROM reachable WHERE task_id=$3)",
    )
    .bind(logical_owner_id)
    .bind(from_task_id.as_slice())
    .bind(target_task_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage)
}

fn mutation_task_id(value: &TasksLifecycleMutationV1) -> [u8; 16] {
    match value {
        TasksLifecycleMutationV1::Create(_) => [0; 16],
        TasksLifecycleMutationV1::Update { task_id, .. }
        | TasksLifecycleMutationV1::SetState { task_id, .. }
        | TasksLifecycleMutationV1::SetPriority { task_id, .. }
        | TasksLifecycleMutationV1::AddDependency { task_id, .. }
        | TasksLifecycleMutationV1::RemoveDependency { task_id, .. }
        | TasksLifecycleMutationV1::AddChecklistItem { task_id, .. }
        | TasksLifecycleMutationV1::UpdateChecklistItem { task_id, .. }
        | TasksLifecycleMutationV1::RemoveChecklistItem { task_id, .. } => *task_id,
    }
}

fn decode_state(value: i16) -> Result<TaskLifecycleStateV1, TasksPersistenceErrorV1> {
    match value {
        1 => Ok(TaskLifecycleStateV1::Open),
        2 => Ok(TaskLifecycleStateV1::InProgress),
        3 => Ok(TaskLifecycleStateV1::Completed),
        4 => Ok(TaskLifecycleStateV1::Cancelled),
        _ => Err(TasksPersistenceErrorV1::InvalidRow),
    }
}

fn encode_state(value: TaskLifecycleStateV1) -> i16 {
    match value {
        TaskLifecycleStateV1::Open => 1,
        TaskLifecycleStateV1::InProgress => 2,
        TaskLifecycleStateV1::Completed => 3,
        TaskLifecycleStateV1::Cancelled => 4,
    }
}

fn decode_priority(value: i16) -> Result<TaskPriorityV1, TasksPersistenceErrorV1> {
    match value {
        1 => Ok(TaskPriorityV1::Low),
        2 => Ok(TaskPriorityV1::Normal),
        3 => Ok(TaskPriorityV1::High),
        4 => Ok(TaskPriorityV1::Urgent),
        _ => Err(TasksPersistenceErrorV1::InvalidRow),
    }
}

fn encode_priority(value: TaskPriorityV1) -> i16 {
    match value {
        TaskPriorityV1::Low => 1,
        TaskPriorityV1::Normal => 2,
        TaskPriorityV1::High => 3,
        TaskPriorityV1::Urgent => 4,
    }
}

fn core_error(value: TaskLifecycleErrorV1) -> TasksPersistenceErrorV1 {
    match value {
        TaskLifecycleErrorV1::RevisionConflict | TaskLifecycleErrorV1::RevisionOverflow => {
            TasksPersistenceErrorV1::RevisionConflict
        }
        TaskLifecycleErrorV1::DependencyCycle | TaskLifecycleErrorV1::SelfDependency => {
            TasksPersistenceErrorV1::DependencyCycle
        }
        TaskLifecycleErrorV1::DependencyNotFound | TaskLifecycleErrorV1::ChecklistItemNotFound => {
            TasksPersistenceErrorV1::NotFound
        }
        TaskLifecycleErrorV1::DuplicateDependency
        | TaskLifecycleErrorV1::ChecklistItemExists
        | TaskLifecycleErrorV1::InvalidStateTransition => TasksPersistenceErrorV1::TaskConflict,
        _ => TasksPersistenceErrorV1::InvalidInput,
    }
}

fn fixed<const N: usize>(value: Vec<u8>) -> Result<[u8; N], TasksPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| TasksPersistenceErrorV1::InvalidRow)
}

fn positive_u64(value: i64) -> Result<u64, TasksPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(TasksPersistenceErrorV1::InvalidRow)
}

fn i64_value(value: u64) -> Result<i64, TasksPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| TasksPersistenceErrorV1::InvalidInput)
}

fn storage(_: sqlx::Error) -> TasksPersistenceErrorV1 {
    TasksPersistenceErrorV1::StorageUnavailable
}
