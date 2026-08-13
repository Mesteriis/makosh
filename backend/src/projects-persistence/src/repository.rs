use makosh_projects_core::{
    ProjectLifecycleErrorV1, ProjectOutcomeStateV1, ProjectOutcomeV1, ProjectRecordV1,
    ProjectReferenceKindV1, ProjectReferenceStateV1, ProjectReferenceV1, ProjectStateV1,
    ProjectTimestampV1, add_project_outcome_v1, add_project_reference_v1, create_project_v1,
    remove_project_outcome_v1, remove_project_reference_v1, set_project_outcome_state_v1,
    set_project_state_v1, update_project_outcome_v1, update_project_v1, validate_project_record_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    ProjectLifecycleCommitV1, ProjectLifecycleMutationV1, ProjectLifecycleOperationOutcomeV1,
    ProjectLifecycleOperationV1, ProjectOutboxRecordV1, ProjectsPersistenceErrorV1,
    model::{
        PROJECTS_MAX_CLIENT_MESSAGE_BYTES_V1, i64_value, valid_commit, valid_operation, valid_owner,
    },
};

#[derive(Clone)]
pub struct ProjectsPersistenceV1 {
    pool: PgPool,
}

pub struct ProjectOutboxPublishClaimV1 {
    transaction: Transaction<'static, Postgres>,
    logical_owner_id: String,
    record: ProjectOutboxRecordV1,
    created_at_unix_millis: i64,
}

impl ProjectOutboxPublishClaimV1 {
    #[must_use]
    pub fn record(&self) -> &ProjectOutboxRecordV1 {
        &self.record
    }

    pub async fn mark_published(
        mut self,
        expected_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), ProjectsPersistenceErrorV1> {
        if expected_sha256 != self.record.envelope_sha256
            || Sha256::digest(&self.record.envelope_bytes).as_slice() != expected_sha256
            || published_at_unix_millis < self.created_at_unix_millis
        {
            return Err(ProjectsPersistenceErrorV1::OutboxConflict);
        }
        let affected = sqlx::query(
            "UPDATE makosh_data.projects_outbox SET published_at_unix_millis=$3 \
             WHERE logical_owner_id=$1 AND message_id=$2 AND envelope_sha256=$4 \
             AND published_at_unix_millis IS NULL",
        )
        .bind(&self.logical_owner_id)
        .bind(self.record.message_id.as_slice())
        .bind(published_at_unix_millis)
        .bind(expected_sha256.as_slice())
        .execute(&mut *self.transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if affected != 1 {
            return Err(ProjectsPersistenceErrorV1::OutboxConflict);
        }
        self.transaction.commit().await.map_err(storage)
    }
}

impl ProjectsPersistenceV1 {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, ProjectsPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(ProjectsPersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(
                u16::try_from(pgbouncer_port)
                    .map_err(|_| ProjectsPersistenceErrorV1::StorageUnavailable)?,
            )
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(storage)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), ProjectsPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage)
    }

    async fn begin_owner(
        &self,
        logical_owner_id: &str,
    ) -> Result<Transaction<'_, Postgres>, ProjectsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(ProjectsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        Ok(transaction)
    }

    pub async fn load_operation_replay(
        &self,
        logical_owner_id: &str,
        operation_id: [u8; 16],
        request_sha256: [u8; 32],
        request_bytes: &[u8],
    ) -> Result<Option<Vec<u8>>, ProjectsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !nonzero(&operation_id)
            || !nonzero(&request_sha256)
            || request_bytes.is_empty()
            || request_bytes.len() > PROJECTS_MAX_CLIENT_MESSAGE_BYTES_V1
            || Sha256::digest(request_bytes).as_slice() != request_sha256
        {
            return Err(ProjectsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let replay = load_operation_replay_raw(
            &mut transaction,
            logical_owner_id,
            operation_id,
            request_sha256,
            request_bytes,
            None,
        )
        .await?;
        transaction.commit().await.map_err(storage)?;
        Ok(replay)
    }

    pub async fn apply_lifecycle_operation<F>(
        &self,
        input: ProjectLifecycleOperationV1,
        build_commit: F,
    ) -> Result<ProjectLifecycleOperationOutcomeV1, ProjectsPersistenceErrorV1>
    where
        F: FnOnce(&ProjectRecordV1) -> Result<ProjectLifecycleCommitV1, ProjectsPersistenceErrorV1>,
    {
        if !valid_operation(&input) {
            return Err(ProjectsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(&input.logical_owner_id).await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1 || encode($2, 'hex'), 0))")
            .bind(&input.logical_owner_id)
            .bind(input.operation_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        if let Some(response_bytes) = load_operation_replay_raw(
            &mut transaction,
            &input.logical_owner_id,
            input.operation_id,
            input.request_sha256,
            &input.request_bytes,
            Some(input.mutation.operation_kind()),
        )
        .await?
        {
            transaction.commit().await.map_err(storage)?;
            return Ok(ProjectLifecycleOperationOutcomeV1::Replayed { response_bytes });
        }

        let creating = matches!(&input.mutation, ProjectLifecycleMutationV1::Create(_));
        let mut project = match &input.mutation {
            ProjectLifecycleMutationV1::Create(draft) => {
                if draft.logical_owner_id != input.logical_owner_id
                    || draft.operation_id != input.operation_id
                {
                    return Err(ProjectsPersistenceErrorV1::InvalidInput);
                }
                create_project_v1(draft.clone()).map_err(core_error)?
            }
            mutation => load_project(
                &mut transaction,
                &input.logical_owner_id,
                mutation
                    .project_id()
                    .ok_or(ProjectsPersistenceErrorV1::InvalidInput)?,
                true,
            )
            .await?
            .ok_or(ProjectsPersistenceErrorV1::NotFound)?,
        };
        apply_mutation(&mut project, &input.mutation)?;
        validate_project_record_v1(&project).map_err(core_error)?;
        persist_project(&mut transaction, &project, creating).await?;
        persist_outcomes(&mut transaction, &project).await?;
        persist_references(&mut transaction, &project).await?;

        let commit = build_commit(&project)?;
        if !valid_commit(&commit) {
            return Err(ProjectsPersistenceErrorV1::InvalidInput);
        }
        let outbox_inserted = sqlx::query(
            "INSERT INTO makosh_data.projects_outbox (logical_owner_id,message_id,envelope_sha256, \
             envelope_bytes,created_at_unix_millis) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(commit.lifecycle_event.message_id.as_slice())
        .bind(commit.lifecycle_event.envelope_sha256.as_slice())
        .bind(&commit.lifecycle_event.envelope_bytes)
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if outbox_inserted != 1 {
            return Err(ProjectsPersistenceErrorV1::OutboxConflict);
        }
        let operation_inserted = sqlx::query(
            "INSERT INTO makosh_data.projects_client_operations (logical_owner_id,operation_id, \
             operation_kind,request_sha256,request_bytes,project_id,project_revision,response_sha256, \
             response_bytes,received_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .bind(input.mutation.operation_kind())
        .bind(input.request_sha256.as_slice())
        .bind(&input.request_bytes)
        .bind(project.project_id.as_slice())
        .bind(i64_value(project.project_revision)?)
        .bind(commit.response_sha256.as_slice())
        .bind(&commit.response_bytes)
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if operation_inserted != 1 {
            return Err(ProjectsPersistenceErrorV1::OperationConflict);
        }
        transaction.commit().await.map_err(storage)?;
        Ok(ProjectLifecycleOperationOutcomeV1::Applied {
            project: Box::new(project),
            response_bytes: commit.response_bytes,
        })
    }

    pub async fn get_project(
        &self,
        logical_owner_id: &str,
        project_id: [u8; 16],
    ) -> Result<Option<ProjectRecordV1>, ProjectsPersistenceErrorV1> {
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let value = load_project(&mut transaction, logical_owner_id, project_id, false).await?;
        transaction.commit().await.map_err(storage)?;
        Ok(value)
    }

    pub async fn list_projects(
        &self,
        logical_owner_id: &str,
        after_project_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<ProjectRecordV1>, ProjectsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || limit == 0 || limit > 201 {
            return Err(ProjectsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner(logical_owner_id).await?;
        let rows = sqlx::query(
            "SELECT project_id FROM makosh_data.projects_records \
             WHERE logical_owner_id=$1 AND ($2::bytea IS NULL OR project_id>$2) \
             ORDER BY project_id LIMIT $3",
        )
        .bind(logical_owner_id)
        .bind(after_project_id.map(|value| value.to_vec()))
        .bind(i64::from(limit))
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage)?;
        let mut values = Vec::with_capacity(rows.len());
        for row in rows {
            let id = fixed::<16>(row.try_get("project_id").map_err(storage)?)?;
            values.push(
                load_project(&mut transaction, logical_owner_id, id, false)
                    .await?
                    .ok_or(ProjectsPersistenceErrorV1::InvalidRow)?,
            );
        }
        transaction.commit().await.map_err(storage)?;
        Ok(values)
    }

    pub async fn list_project_outcomes(
        &self,
        logical_owner_id: &str,
        project_id: [u8; 16],
        after_outcome_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<ProjectOutcomeV1>, ProjectsPersistenceErrorV1> {
        let project = self
            .get_project(logical_owner_id, project_id)
            .await?
            .ok_or(ProjectsPersistenceErrorV1::NotFound)?;
        bounded_after(project.outcomes, after_outcome_id, limit, |value| {
            value.outcome_id
        })
    }

    pub async fn list_project_references(
        &self,
        logical_owner_id: &str,
        project_id: [u8; 16],
        after_reference_id: Option<[u8; 16]>,
        limit: u16,
    ) -> Result<Vec<ProjectReferenceV1>, ProjectsPersistenceErrorV1> {
        let project = self
            .get_project(logical_owner_id, project_id)
            .await?
            .ok_or(ProjectsPersistenceErrorV1::NotFound)?;
        bounded_after(project.references, after_reference_id, limit, |value| {
            value.reference_id
        })
    }

    pub async fn claim_next_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Option<ProjectOutboxPublishClaimV1>, ProjectsPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(ProjectsPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        let row = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes,created_at_unix_millis \
             FROM makosh_data.projects_outbox WHERE logical_owner_id=$1 \
             AND published_at_unix_millis IS NULL ORDER BY outbox_sequence \
             LIMIT 1 FOR UPDATE SKIP LOCKED",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let Some(row) = row else {
            transaction.commit().await.map_err(storage)?;
            return Ok(None);
        };
        let record = ProjectOutboxRecordV1 {
            message_id: fixed(row.try_get("message_id").map_err(storage)?)?,
            envelope_sha256: fixed(row.try_get("envelope_sha256").map_err(storage)?)?,
            envelope_bytes: row.try_get("envelope_bytes").map_err(storage)?,
        };
        if Sha256::digest(&record.envelope_bytes).as_slice() != record.envelope_sha256 {
            return Err(ProjectsPersistenceErrorV1::InvalidRow);
        }
        Ok(Some(ProjectOutboxPublishClaimV1 {
            transaction,
            logical_owner_id: logical_owner_id.to_owned(),
            record,
            created_at_unix_millis: row.try_get("created_at_unix_millis").map_err(storage)?,
        }))
    }
}

fn bounded_after<T, F>(
    mut values: Vec<T>,
    after: Option<[u8; 16]>,
    limit: u16,
    id: F,
) -> Result<Vec<T>, ProjectsPersistenceErrorV1>
where
    F: Fn(&T) -> [u8; 16],
{
    if limit == 0 || limit > 201 {
        return Err(ProjectsPersistenceErrorV1::InvalidInput);
    }
    values.sort_by_key(&id);
    Ok(values
        .into_iter()
        .filter(|value| after.is_none_or(|after| id(value) > after))
        .take(usize::from(limit))
        .collect())
}

fn apply_mutation(
    project: &mut ProjectRecordV1,
    mutation: &ProjectLifecycleMutationV1,
) -> Result<(), ProjectsPersistenceErrorV1> {
    let result = match mutation {
        ProjectLifecycleMutationV1::Create(_) => return Ok(()),
        ProjectLifecycleMutationV1::Update {
            expected_revision,
            name,
            description,
            start_at,
            target_at,
            changed_at,
            ..
        } => update_project_v1(
            project,
            *expected_revision,
            name.clone(),
            description.clone(),
            *start_at,
            *target_at,
            *changed_at,
        ),
        ProjectLifecycleMutationV1::SetState {
            expected_revision,
            state,
            changed_at,
            ..
        } => set_project_state_v1(project, *expected_revision, *state, *changed_at),
        ProjectLifecycleMutationV1::AddOutcome {
            operation_id,
            expected_revision,
            title,
            description,
            target_at,
            changed_at,
            ..
        } => add_project_outcome_v1(
            project,
            *operation_id,
            *expected_revision,
            title.clone(),
            description.clone(),
            *target_at,
            *changed_at,
        )
        .map(|_| ()),
        ProjectLifecycleMutationV1::UpdateOutcome {
            expected_revision,
            outcome_id,
            expected_outcome_revision,
            title,
            description,
            target_at,
            changed_at,
            ..
        } => update_project_outcome_v1(
            project,
            *expected_revision,
            *outcome_id,
            *expected_outcome_revision,
            title.clone(),
            description.clone(),
            *target_at,
            *changed_at,
        ),
        ProjectLifecycleMutationV1::SetOutcomeState {
            expected_revision,
            outcome_id,
            expected_outcome_revision,
            state,
            changed_at,
            ..
        } => set_project_outcome_state_v1(
            project,
            *expected_revision,
            *outcome_id,
            *expected_outcome_revision,
            *state,
            *changed_at,
        ),
        ProjectLifecycleMutationV1::RemoveOutcome {
            expected_revision,
            outcome_id,
            expected_outcome_revision,
            changed_at,
            ..
        } => remove_project_outcome_v1(
            project,
            *expected_revision,
            *outcome_id,
            *expected_outcome_revision,
            *changed_at,
        ),
        ProjectLifecycleMutationV1::AddReference {
            expected_revision,
            kind,
            public_id,
            label,
            changed_at,
            ..
        } => add_project_reference_v1(
            project,
            *expected_revision,
            *kind,
            *public_id,
            label.clone(),
            *changed_at,
        )
        .map(|_| ()),
        ProjectLifecycleMutationV1::RemoveReference {
            expected_revision,
            reference_id,
            changed_at,
            ..
        } => remove_project_reference_v1(project, *expected_revision, *reference_id, *changed_at),
    };
    result.map_err(core_error)
}

async fn load_operation_replay_raw(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    operation_id: [u8; 16],
    request_sha256: [u8; 32],
    request_bytes: &[u8],
    operation_kind: Option<i16>,
) -> Result<Option<Vec<u8>>, ProjectsPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT operation_kind,request_sha256,request_bytes,response_sha256,response_bytes \
         FROM makosh_data.projects_client_operations WHERE logical_owner_id=$1 AND operation_id=$2 FOR UPDATE",
    )
    .bind(owner)
    .bind(operation_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    let Some(row) = row else { return Ok(None) };
    let kind: i16 = row.try_get("operation_kind").map_err(storage)?;
    let stored_request_sha: [u8; 32] = fixed(row.try_get("request_sha256").map_err(storage)?)?;
    let stored_request: Vec<u8> = row.try_get("request_bytes").map_err(storage)?;
    let stored_response_sha: [u8; 32] = fixed(row.try_get("response_sha256").map_err(storage)?)?;
    let stored_response: Vec<u8> = row.try_get("response_bytes").map_err(storage)?;
    if operation_kind.is_some_and(|value| value != kind)
        || stored_request_sha != request_sha256
        || stored_request != request_bytes
        || Sha256::digest(&stored_request).as_slice() != stored_request_sha
        || Sha256::digest(&stored_response).as_slice() != stored_response_sha
    {
        return Err(ProjectsPersistenceErrorV1::OperationConflict);
    }
    Ok(Some(stored_response))
}

async fn load_project(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    project_id: [u8; 16],
    for_update: bool,
) -> Result<Option<ProjectRecordV1>, ProjectsPersistenceErrorV1> {
    let statement = if for_update {
        "SELECT name,description,project_state,start_at_unix_seconds,start_at_nanos,target_at_unix_seconds,target_at_nanos, \
         project_revision,created_at_unix_seconds,created_at_nanos,updated_at_unix_seconds,updated_at_nanos \
         FROM makosh_data.projects_records WHERE logical_owner_id=$1 AND project_id=$2 FOR UPDATE"
    } else {
        "SELECT name,description,project_state,start_at_unix_seconds,start_at_nanos,target_at_unix_seconds,target_at_nanos, \
         project_revision,created_at_unix_seconds,created_at_nanos,updated_at_unix_seconds,updated_at_nanos \
         FROM makosh_data.projects_records WHERE logical_owner_id=$1 AND project_id=$2"
    };
    let row = sqlx::query(statement)
        .bind(owner)
        .bind(project_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage)?;
    let Some(row) = row else { return Ok(None) };
    let outcomes = load_outcomes(transaction, owner, project_id).await?;
    let references = load_references(transaction, owner, project_id).await?;
    let value = ProjectRecordV1 {
        project_id,
        logical_owner_id: owner.to_owned(),
        name: row.try_get("name").map_err(storage)?,
        description: row.try_get("description").map_err(storage)?,
        state: decode_state(row.try_get("project_state").map_err(storage)?)?,
        start_at: optional_timestamp(&row, "start_at_unix_seconds", "start_at_nanos")?,
        target_at: optional_timestamp(&row, "target_at_unix_seconds", "target_at_nanos")?,
        project_revision: u64_value(row.try_get("project_revision").map_err(storage)?)?,
        outcomes,
        references,
        created_at: ProjectTimestampV1 {
            unix_seconds: row.try_get("created_at_unix_seconds").map_err(storage)?,
            nanos: row.try_get("created_at_nanos").map_err(storage)?,
        },
        updated_at: ProjectTimestampV1 {
            unix_seconds: row.try_get("updated_at_unix_seconds").map_err(storage)?,
            nanos: row.try_get("updated_at_nanos").map_err(storage)?,
        },
    };
    validate_project_record_v1(&value).map_err(core_error)?;
    Ok(Some(value))
}

async fn load_outcomes(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    project_id: [u8; 16],
) -> Result<Vec<ProjectOutcomeV1>, ProjectsPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT outcome_id,title,description,outcome_state,target_at_unix_seconds,target_at_nanos, \
         outcome_revision,updated_at_project_revision,created_at_unix_seconds,created_at_nanos, \
         updated_at_unix_seconds,updated_at_nanos FROM makosh_data.projects_outcomes \
         WHERE logical_owner_id=$1 AND project_id=$2 ORDER BY outcome_id",
    )
    .bind(owner)
    .bind(project_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?;
    rows.into_iter()
        .map(|row| {
            Ok(ProjectOutcomeV1 {
                outcome_id: fixed(row.try_get("outcome_id").map_err(storage)?)?,
                project_id,
                title: row.try_get("title").map_err(storage)?,
                description: row.try_get("description").map_err(storage)?,
                state: decode_outcome_state(row.try_get("outcome_state").map_err(storage)?)?,
                target_at: optional_timestamp(&row, "target_at_unix_seconds", "target_at_nanos")?,
                outcome_revision: u64_value(row.try_get("outcome_revision").map_err(storage)?)?,
                updated_at_project_revision: u64_value(
                    row.try_get("updated_at_project_revision")
                        .map_err(storage)?,
                )?,
                created_at: ProjectTimestampV1 {
                    unix_seconds: row.try_get("created_at_unix_seconds").map_err(storage)?,
                    nanos: row.try_get("created_at_nanos").map_err(storage)?,
                },
                updated_at: ProjectTimestampV1 {
                    unix_seconds: row.try_get("updated_at_unix_seconds").map_err(storage)?,
                    nanos: row.try_get("updated_at_nanos").map_err(storage)?,
                },
            })
        })
        .collect()
}

async fn load_references(
    transaction: &mut Transaction<'_, Postgres>,
    owner: &str,
    project_id: [u8; 16],
) -> Result<Vec<ProjectReferenceV1>, ProjectsPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT reference_id,reference_kind,public_id,label,reference_state,updated_at_project_revision \
         FROM makosh_data.projects_references WHERE logical_owner_id=$1 AND project_id=$2 ORDER BY reference_id",
    )
    .bind(owner)
    .bind(project_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?;
    rows.into_iter()
        .map(|row| {
            Ok(ProjectReferenceV1 {
                reference_id: fixed(row.try_get("reference_id").map_err(storage)?)?,
                kind: decode_reference_kind(row.try_get("reference_kind").map_err(storage)?)?,
                public_id: fixed(row.try_get("public_id").map_err(storage)?)?,
                label: row.try_get("label").map_err(storage)?,
                state: decode_reference_state(row.try_get("reference_state").map_err(storage)?)?,
                updated_at_project_revision: u64_value(
                    row.try_get("updated_at_project_revision")
                        .map_err(storage)?,
                )?,
            })
        })
        .collect()
}

async fn persist_project(
    transaction: &mut Transaction<'_, Postgres>,
    value: &ProjectRecordV1,
    creating: bool,
) -> Result<(), ProjectsPersistenceErrorV1> {
    let (start_seconds, start_nanos) = split_timestamp(value.start_at);
    let (target_seconds, target_nanos) = split_timestamp(value.target_at);
    let affected = if creating {
        sqlx::query(
            "INSERT INTO makosh_data.projects_records (logical_owner_id,project_id,name,description,project_state, \
             start_at_unix_seconds,start_at_nanos,target_at_unix_seconds,target_at_nanos,project_revision, \
             created_at_unix_seconds,created_at_nanos,updated_at_unix_seconds,updated_at_nanos) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
        )
        .bind(&value.logical_owner_id).bind(value.project_id.as_slice()).bind(&value.name)
        .bind(&value.description).bind(encode_state(value.state)).bind(start_seconds).bind(start_nanos)
        .bind(target_seconds).bind(target_nanos).bind(i64_value(value.project_revision)?)
        .bind(value.created_at.unix_seconds).bind(value.created_at.nanos)
        .bind(value.updated_at.unix_seconds).bind(value.updated_at.nanos)
        .execute(&mut **transaction).await.map_err(storage)?.rows_affected()
    } else {
        let previous = value
            .project_revision
            .checked_sub(1)
            .ok_or(ProjectsPersistenceErrorV1::RevisionConflict)?;
        sqlx::query(
            "UPDATE makosh_data.projects_records SET name=$3,description=$4,project_state=$5, \
             start_at_unix_seconds=$6,start_at_nanos=$7,target_at_unix_seconds=$8,target_at_nanos=$9, \
             project_revision=$10,updated_at_unix_seconds=$11,updated_at_nanos=$12 \
             WHERE logical_owner_id=$1 AND project_id=$2 AND project_revision=$13",
        )
        .bind(&value.logical_owner_id).bind(value.project_id.as_slice()).bind(&value.name)
        .bind(&value.description).bind(encode_state(value.state)).bind(start_seconds).bind(start_nanos)
        .bind(target_seconds).bind(target_nanos).bind(i64_value(value.project_revision)?)
        .bind(value.updated_at.unix_seconds).bind(value.updated_at.nanos).bind(i64_value(previous)?)
        .execute(&mut **transaction).await.map_err(storage)?.rows_affected()
    };
    if affected != 1 {
        return Err(if creating {
            ProjectsPersistenceErrorV1::OperationConflict
        } else {
            ProjectsPersistenceErrorV1::RevisionConflict
        });
    }
    Ok(())
}

async fn persist_outcomes(
    transaction: &mut Transaction<'_, Postgres>,
    value: &ProjectRecordV1,
) -> Result<(), ProjectsPersistenceErrorV1> {
    sqlx::query(
        "DELETE FROM makosh_data.projects_outcomes WHERE logical_owner_id=$1 AND project_id=$2",
    )
    .bind(&value.logical_owner_id)
    .bind(value.project_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    for outcome in &value.outcomes {
        let (target_seconds, target_nanos) = split_timestamp(outcome.target_at);
        sqlx::query(
            "INSERT INTO makosh_data.projects_outcomes (logical_owner_id,project_id,outcome_id,title,description, \
             outcome_state,target_at_unix_seconds,target_at_nanos,outcome_revision,updated_at_project_revision, \
             created_at_unix_seconds,created_at_nanos,updated_at_unix_seconds,updated_at_nanos) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
        )
        .bind(&value.logical_owner_id).bind(value.project_id.as_slice()).bind(outcome.outcome_id.as_slice())
        .bind(&outcome.title).bind(&outcome.description).bind(encode_outcome_state(outcome.state))
        .bind(target_seconds).bind(target_nanos).bind(i64_value(outcome.outcome_revision)?)
        .bind(i64_value(outcome.updated_at_project_revision)?).bind(outcome.created_at.unix_seconds)
        .bind(outcome.created_at.nanos).bind(outcome.updated_at.unix_seconds).bind(outcome.updated_at.nanos)
        .execute(&mut **transaction).await.map_err(storage)?;
    }
    Ok(())
}

async fn persist_references(
    transaction: &mut Transaction<'_, Postgres>,
    value: &ProjectRecordV1,
) -> Result<(), ProjectsPersistenceErrorV1> {
    sqlx::query(
        "DELETE FROM makosh_data.projects_references WHERE logical_owner_id=$1 AND project_id=$2",
    )
    .bind(&value.logical_owner_id)
    .bind(value.project_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(storage)?;
    for reference in &value.references {
        sqlx::query(
            "INSERT INTO makosh_data.projects_references (logical_owner_id,project_id,reference_id,reference_kind, \
             public_id,label,reference_state,updated_at_project_revision) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(&value.logical_owner_id).bind(value.project_id.as_slice()).bind(reference.reference_id.as_slice())
        .bind(encode_reference_kind(reference.kind)).bind(reference.public_id.as_slice()).bind(&reference.label)
        .bind(encode_reference_state(reference.state)).bind(i64_value(reference.updated_at_project_revision)?)
        .execute(&mut **transaction).await.map_err(storage)?;
    }
    Ok(())
}

fn split_timestamp(value: Option<ProjectTimestampV1>) -> (Option<i64>, Option<i32>) {
    value.map_or((None, None), |value| {
        (Some(value.unix_seconds), Some(value.nanos))
    })
}

fn optional_timestamp(
    row: &sqlx::postgres::PgRow,
    seconds: &str,
    nanos: &str,
) -> Result<Option<ProjectTimestampV1>, ProjectsPersistenceErrorV1> {
    let seconds: Option<i64> = row.try_get(seconds).map_err(storage)?;
    let nanos: Option<i32> = row.try_get(nanos).map_err(storage)?;
    match (seconds, nanos) {
        (None, None) => Ok(None),
        (Some(unix_seconds), Some(nanos)) => Ok(Some(ProjectTimestampV1 {
            unix_seconds,
            nanos,
        })),
        _ => Err(ProjectsPersistenceErrorV1::InvalidRow),
    }
}

fn encode_state(value: ProjectStateV1) -> i16 {
    match value {
        ProjectStateV1::Planning => 1,
        ProjectStateV1::Active => 2,
        ProjectStateV1::OnHold => 3,
        ProjectStateV1::Completed => 4,
        ProjectStateV1::Archived => 5,
    }
}
fn decode_state(value: i16) -> Result<ProjectStateV1, ProjectsPersistenceErrorV1> {
    match value {
        1 => Ok(ProjectStateV1::Planning),
        2 => Ok(ProjectStateV1::Active),
        3 => Ok(ProjectStateV1::OnHold),
        4 => Ok(ProjectStateV1::Completed),
        5 => Ok(ProjectStateV1::Archived),
        _ => Err(ProjectsPersistenceErrorV1::InvalidRow),
    }
}
fn encode_outcome_state(value: ProjectOutcomeStateV1) -> i16 {
    match value {
        ProjectOutcomeStateV1::Pending => 1,
        ProjectOutcomeStateV1::Achieved => 2,
        ProjectOutcomeStateV1::Missed => 3,
        ProjectOutcomeStateV1::Cancelled => 4,
    }
}
fn decode_outcome_state(value: i16) -> Result<ProjectOutcomeStateV1, ProjectsPersistenceErrorV1> {
    match value {
        1 => Ok(ProjectOutcomeStateV1::Pending),
        2 => Ok(ProjectOutcomeStateV1::Achieved),
        3 => Ok(ProjectOutcomeStateV1::Missed),
        4 => Ok(ProjectOutcomeStateV1::Cancelled),
        _ => Err(ProjectsPersistenceErrorV1::InvalidRow),
    }
}
fn encode_reference_kind(value: ProjectReferenceKindV1) -> i16 {
    match value {
        ProjectReferenceKindV1::Person => 1,
        ProjectReferenceKindV1::Organization => 2,
        ProjectReferenceKindV1::Relationship => 3,
        ProjectReferenceKindV1::Task => 4,
        ProjectReferenceKindV1::Document => 5,
        ProjectReferenceKindV1::CalendarEvent => 6,
    }
}
fn decode_reference_kind(value: i16) -> Result<ProjectReferenceKindV1, ProjectsPersistenceErrorV1> {
    match value {
        1 => Ok(ProjectReferenceKindV1::Person),
        2 => Ok(ProjectReferenceKindV1::Organization),
        3 => Ok(ProjectReferenceKindV1::Relationship),
        4 => Ok(ProjectReferenceKindV1::Task),
        5 => Ok(ProjectReferenceKindV1::Document),
        6 => Ok(ProjectReferenceKindV1::CalendarEvent),
        _ => Err(ProjectsPersistenceErrorV1::InvalidRow),
    }
}
fn encode_reference_state(value: ProjectReferenceStateV1) -> i16 {
    match value {
        ProjectReferenceStateV1::Active => 1,
        ProjectReferenceStateV1::Removed => 2,
    }
}
fn decode_reference_state(
    value: i16,
) -> Result<ProjectReferenceStateV1, ProjectsPersistenceErrorV1> {
    match value {
        1 => Ok(ProjectReferenceStateV1::Active),
        2 => Ok(ProjectReferenceStateV1::Removed),
        _ => Err(ProjectsPersistenceErrorV1::InvalidRow),
    }
}

fn u64_value(value: i64) -> Result<u64, ProjectsPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(ProjectsPersistenceErrorV1::InvalidRow)
}
fn fixed<const N: usize>(value: Vec<u8>) -> Result<[u8; N], ProjectsPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| ProjectsPersistenceErrorV1::InvalidRow)
}
fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}
fn core_error(value: ProjectLifecycleErrorV1) -> ProjectsPersistenceErrorV1 {
    match value {
        ProjectLifecycleErrorV1::InvalidRevision | ProjectLifecycleErrorV1::RevisionOverflow => {
            ProjectsPersistenceErrorV1::RevisionConflict
        }
        ProjectLifecycleErrorV1::InvalidProjectId
        | ProjectLifecycleErrorV1::OutcomeNotFound
        | ProjectLifecycleErrorV1::ReferenceNotFound => ProjectsPersistenceErrorV1::NotFound,
        _ => ProjectsPersistenceErrorV1::InvalidInput,
    }
}
fn storage(_: sqlx::Error) -> ProjectsPersistenceErrorV1 {
    ProjectsPersistenceErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_and_cursor_order_are_bounded() {
        assert_eq!(
            core_error(ProjectLifecycleErrorV1::InvalidRevision),
            ProjectsPersistenceErrorV1::RevisionConflict
        );
        assert_eq!(
            core_error(ProjectLifecycleErrorV1::OutcomeNotFound),
            ProjectsPersistenceErrorV1::NotFound
        );
        assert_eq!(
            bounded_after(vec![[2; 16], [1; 16]], None, 1, |value| *value),
            Ok(vec![[1; 16]])
        );
    }
}
