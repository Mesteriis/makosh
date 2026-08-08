use makosh_tasks_api::{
    TaskGetFuture, TaskListFuture, TaskListQuery, TaskQueryError, TaskRead, TaskReadPort,
};
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct TaskPostgresReadQuery {
    pool: PgPool,
}
impl TaskPostgresReadQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
impl TaskReadPort for TaskPostgresReadQuery {
    fn list<'a>(&'a self, query: TaskListQuery) -> TaskListFuture<'a> {
        Box::pin(async move {
            let limit = query.limit.unwrap_or(100).clamp(1, 500);
            let rows = sqlx::query(TASK_SQL)
                .bind(query.status.as_deref())
                .bind(query.project_id.as_deref())
                .bind(query.source_type.as_deref())
                .bind(limit)
                .fetch_all(&self.pool)
                .await
                .map_err(error)?;
            rows.into_iter().map(map_row).collect()
        })
    }

    fn get<'a>(&'a self, task_id: &'a str) -> TaskGetFuture<'a> {
        Box::pin(async move {
            sqlx::query(TASK_GET_SQL)
                .bind(task_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(error)?
                .map(map_row)
                .transpose()
        })
    }
}
const TASK_SQL: &str = "SELECT task_id, task_candidate_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, status, makosh_status, priority_score::float8 AS priority_score, risk_score::float8 AS risk_score, readiness_score::float8 AS readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at FROM tasks WHERE ($1::text IS NULL OR makosh_status=$1) AND ($2::text IS NULL OR project_id=$2) AND ($3::text IS NULL OR source_type=$3) ORDER BY COALESCE(priority_score,0) DESC, due_at ASC NULLS LAST, created_at DESC LIMIT $4";
const TASK_GET_SQL: &str = "SELECT task_id, task_candidate_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, status, makosh_status, priority_score::float8 AS priority_score, risk_score::float8 AS risk_score, readiness_score::float8 AS readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at FROM tasks WHERE task_id=$1";
fn error(e: sqlx::Error) -> TaskQueryError {
    TaskQueryError(e.to_string())
}
fn map_row(row: sqlx::postgres::PgRow) -> Result<TaskRead, TaskQueryError> {
    macro_rules! get {
        ($name:literal) => {
            row.try_get($name).map_err(error)?
        };
    }
    Ok(TaskRead {
        task_id: get!("task_id"),
        task_candidate_id: get!("task_candidate_id"),
        title: get!("title"),
        description: get!("description"),
        provenance_kind: get!("provenance_kind"),
        provenance_id: get!("provenance_id"),
        source_kind: get!("source_kind"),
        source_id: get!("source_id"),
        source_type: get!("source_type"),
        project_id: get!("project_id"),
        status: get!("status"),
        makosh_status: get!("makosh_status"),
        priority_score: get!("priority_score"),
        risk_score: get!("risk_score"),
        readiness_score: get!("readiness_score"),
        area: get!("area"),
        why: get!("why"),
        outcome: get!("outcome"),
        due_at: get!("due_at"),
        completed_at: get!("completed_at"),
        archived_at: get!("archived_at"),
        waiting_reason: get!("waiting_reason"),
        energy_type: get!("energy_type"),
        confidentiality: get!("confidentiality"),
        tags: get!("tags"),
        task_metadata: get!("task_metadata"),
        linked_person_id: get!("linked_person_id"),
        linked_organization_id: get!("linked_organization_id"),
        created_from_event_id: get!("created_from_event_id"),
        created_by_actor_id: get!("created_by_actor_id"),
        created_at: get!("created_at"),
        updated_at: get!("updated_at"),
    })
}
