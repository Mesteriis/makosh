use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use super::command_service::{TaskCommandService, TaskCommandServiceError};
use super::core::errors::TaskCoreError;
use super::core::observation_links::materialize_task_observation_link_in_transaction;
use makosh_observations_postgres::errors::ObservationStoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub task_id: String,
    pub task_candidate_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub provenance_kind: String,
    pub provenance_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub source_type: String,
    pub project_id: Option<String>,
    pub status: String,
    pub makosh_status: String,
    pub priority_score: Option<f64>,
    pub risk_score: Option<f64>,
    pub readiness_score: Option<f64>,
    pub area: Option<String>,
    pub why: Option<String>,
    pub outcome: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub waiting_reason: Option<String>,
    pub energy_type: Option<String>,
    pub confidentiality: String,
    pub tags: Value,
    pub task_metadata: Value,
    pub linked_person_id: Option<String>,
    pub linked_organization_id: Option<String>,
    pub created_from_event_id: Option<String>,
    pub created_by_actor_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskStore {
    pool: PgPool,
}

impl TaskStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: &NewTask) -> Result<Task, TaskError> {
        TaskCommandService::new(self.pool.clone())
            .create_task_manual(req)
            .await
            .map_err(|error| match error {
                TaskCommandServiceError::ObservationCapture { source, .. } => {
                    TaskError::from(source)
                }
                TaskCommandServiceError::Task(inner) => inner,
                TaskCommandServiceError::Core(inner) => TaskError::from(inner),
                TaskCommandServiceError::Sqlx(inner) => TaskError::from(inner),
                TaskCommandServiceError::ObservationStore(inner) => TaskError::from(inner),
                TaskCommandServiceError::MissingEvidenceSourceId => {
                    TaskError::MissingSourceIdentifier
                }
            })
    }

    pub(crate) async fn create_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        req: &NewTask,
    ) -> Result<Task, TaskError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let task_id = format!("task:v1:{ts:x}");
        let tags = req.tags.clone().unwrap_or_else(|| json!([]));
        let row = sqlx::query(
            "INSERT INTO tasks (task_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, makosh_status, priority_score, area, why, due_at, energy_type, confidentiality, tags, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21)
             RETURNING task_id, task_candidate_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, status, makosh_status, priority_score::float8 AS priority_score, risk_score::float8 AS risk_score, readiness_score::float8 AS readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at"
        ).bind(&task_id).bind(&req.title).bind(req.description.as_deref())
         .bind(req.provenance_kind.as_deref().ok_or(TaskError::MissingProvenance)?)
         .bind(req.provenance_id.as_deref().ok_or(TaskError::MissingProvenance)?)
         .bind(req.source_kind.as_deref().unwrap_or("manual")).bind(req.source_id.as_deref().unwrap_or("manual"))
         .bind(req.source_type.as_deref().unwrap_or("manual")).bind(req.project_id.as_deref())
         .bind(req.makosh_status.as_deref().unwrap_or("new")).bind(req.priority_score)
         .bind(req.area.as_deref()).bind(req.why.as_deref()).bind(req.due_at)
         .bind(req.energy_type.as_deref()).bind(req.confidentiality.as_deref().unwrap_or("private_local"))
         .bind(&tags).bind(req.linked_person_id.as_deref()).bind(req.linked_organization_id.as_deref())
         .bind(req.created_from_event_id.as_deref()).bind(req.created_by_actor_id.as_deref())
         .fetch_one(&mut **transaction).await?;
        let task = row_to_task(row)?;

        if let Some(observation_id) = req
            .source_id
            .as_deref()
            .filter(|_| req.source_kind.as_deref() == Some("observation"))
        {
            materialize_task_observation_link_in_transaction(
                transaction,
                Some(observation_id),
                Some("task_create"),
                &task.task_id,
                Some(json!({
                    "source_kind": req.source_kind,
                    "source_type": req.source_type,
                    "provenance_kind": req.provenance_kind,
                    "provenance_id": req.provenance_id,
                })),
            )
            .await?;
        }

        Ok(task)
    }

    pub async fn get(&self, task_id: &str) -> Result<Option<Task>, TaskError> {
        let row = sqlx::query("SELECT task_id, task_candidate_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, status, makosh_status, priority_score::float8 AS priority_score, risk_score::float8 AS risk_score, readiness_score::float8 AS readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at FROM tasks WHERE task_id=$1")
            .bind(task_id).fetch_optional(&self.pool).await?;
        row.map(|r| row_to_task(r).map_err(TaskError::from))
            .transpose()
    }

    pub async fn list(&self, query: &TaskListQuery) -> Result<Vec<Task>, TaskError> {
        let limit = query.limit.unwrap_or(100).clamp(1, 500);
        let rows = sqlx::query(
            "SELECT task_id, task_candidate_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, status, makosh_status, priority_score::float8 AS priority_score, risk_score::float8 AS risk_score, readiness_score::float8 AS readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at FROM tasks WHERE ($1::text IS NULL OR makosh_status=$1) AND ($2::text IS NULL OR project_id=$2) AND ($3::text IS NULL OR source_type=$3) ORDER BY COALESCE(priority_score,0) DESC, due_at ASC NULLS LAST, created_at DESC LIMIT $4"
        ).bind(query.status.as_deref()).bind(query.project_id.as_deref()).bind(query.source_type.as_deref()).bind(limit)
         .fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(row_to_task)
            .collect::<Result<Vec<_>, _>>()
            .map_err(TaskError::from)
    }

    pub async fn update(&self, task_id: &str, update: &TaskUpdate) -> Result<Task, TaskError> {
        self.update_internal(task_id, update, None, None, None)
            .await
    }

    pub async fn update_with_observation(
        &self,
        task_id: &str,
        update: &TaskUpdate,
        observation_id: &str,
        relationship_kind: &str,
        metadata: Value,
    ) -> Result<Task, TaskError> {
        self.update_internal(
            task_id,
            update,
            Some(observation_id),
            Some(relationship_kind),
            Some(metadata),
        )
        .await
    }

    async fn update_internal(
        &self,
        task_id: &str,
        update: &TaskUpdate,
        observation_id: Option<&str>,
        relationship_kind: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<Task, TaskError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "UPDATE tasks SET title=COALESCE($2,title), description=COALESCE($3,description), makosh_status=COALESCE($4,makosh_status), priority_score=COALESCE($5,priority_score), risk_score=COALESCE($6,risk_score), readiness_score=COALESCE($7,readiness_score), area=COALESCE($8,area), why=COALESCE($9,why), outcome=COALESCE($10,outcome), due_at=COALESCE($11,due_at), waiting_reason=COALESCE($12,waiting_reason), energy_type=COALESCE($13,energy_type), confidentiality=COALESCE($14,confidentiality), tags=COALESCE($15,tags), task_metadata=COALESCE($16,task_metadata), linked_person_id=COALESCE($17,linked_person_id), linked_organization_id=COALESCE($18,linked_organization_id), completed_at=COALESCE($19,completed_at), updated_at=now() WHERE task_id=$1 RETURNING task_id, task_candidate_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, status, makosh_status, priority_score::float8 AS priority_score, risk_score::float8 AS risk_score, readiness_score::float8 AS readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at"
        ).bind(task_id).bind(update.title.as_deref()).bind(update.description.as_deref())
         .bind(update.makosh_status.as_deref()).bind(update.priority_score).bind(update.risk_score).bind(update.readiness_score)
         .bind(update.area.as_deref()).bind(update.why.as_deref()).bind(update.outcome.as_deref())
         .bind(update.due_at).bind(update.waiting_reason.as_deref()).bind(update.energy_type.as_deref())
         .bind(update.confidentiality.as_deref()).bind(update.tags.as_ref()).bind(update.task_metadata.as_ref())
         .bind(update.linked_person_id.as_deref()).bind(update.linked_organization_id.as_deref())
         .bind(update.completed_at)
         .fetch_one(&mut *transaction).await?;
        let task = row_to_task(row)?;

        materialize_task_observation_link_in_transaction(
            &mut transaction,
            observation_id,
            relationship_kind,
            task_id,
            metadata,
        )
        .await?;
        transaction.commit().await?;

        Ok(task)
    }

    pub async fn set_status(&self, task_id: &str, status: &str) -> Result<Task, TaskError> {
        self.set_status_internal(task_id, status, None, None, None)
            .await
    }

    pub async fn set_status_with_observation(
        &self,
        task_id: &str,
        status: &str,
        observation_id: &str,
        relationship_kind: &str,
        metadata: Value,
    ) -> Result<Task, TaskError> {
        self.set_status_internal(
            task_id,
            status,
            Some(observation_id),
            Some(relationship_kind),
            Some(metadata),
        )
        .await
    }

    async fn set_status_internal(
        &self,
        task_id: &str,
        status: &str,
        observation_id: Option<&str>,
        relationship_kind: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<Task, TaskError> {
        let stored_status = if status == "completed" {
            "done"
        } else {
            status
        };
        let sql = if stored_status == "done" {
            "UPDATE tasks
             SET makosh_status=$2, completed_at=now(), updated_at=now()
             WHERE task_id=$1
             RETURNING task_id, task_candidate_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, status, makosh_status, priority_score::float8 AS priority_score, risk_score::float8 AS risk_score, readiness_score::float8 AS readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at"
        } else if stored_status == "archived" {
            "UPDATE tasks
             SET makosh_status=$2, archived_at=now(), updated_at=now()
             WHERE task_id=$1
             RETURNING task_id, task_candidate_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, status, makosh_status, priority_score::float8 AS priority_score, risk_score::float8 AS risk_score, readiness_score::float8 AS readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at"
        } else {
            "UPDATE tasks
             SET makosh_status=$2, updated_at=now()
             WHERE task_id=$1
             RETURNING task_id, task_candidate_id, title, description, provenance_kind, provenance_id, source_kind, source_id, source_type, project_id, status, makosh_status, priority_score::float8 AS priority_score, risk_score::float8 AS risk_score, readiness_score::float8 AS readiness_score, area, why, outcome, due_at, completed_at, archived_at, waiting_reason, energy_type, confidentiality, tags, task_metadata, linked_person_id, linked_organization_id, created_from_event_id, created_by_actor_id, created_at, updated_at"
        };
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(sql)
            .bind(task_id)
            .bind(stored_status)
            .fetch_optional(&mut *transaction)
            .await?
            .ok_or(TaskError::NotFound)?;
        let task = row_to_task(row)?;

        materialize_task_observation_link_in_transaction(
            &mut transaction,
            observation_id,
            relationship_kind,
            task_id,
            metadata,
        )
        .await?;
        transaction.commit().await?;

        Ok(task)
    }

    pub async fn archive(&self, task_id: &str) -> Result<Task, TaskError> {
        self.set_status(task_id, "archived").await
    }

    pub async fn archive_with_observation(
        &self,
        task_id: &str,
        observation_id: &str,
        relationship_kind: &str,
        metadata: Value,
    ) -> Result<Task, TaskError> {
        self.set_status_with_observation(
            task_id,
            "archived",
            observation_id,
            relationship_kind,
            metadata,
        )
        .await
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct NewTask {
    pub title: String,
    pub description: Option<String>,
    pub provenance_kind: Option<String>,
    pub provenance_id: Option<String>,
    pub source_kind: Option<String>,
    pub source_id: Option<String>,
    pub source_type: Option<String>,
    pub project_id: Option<String>,
    pub makosh_status: Option<String>,
    pub priority_score: Option<f64>,
    pub area: Option<String>,
    pub why: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub energy_type: Option<String>,
    pub confidentiality: Option<String>,
    pub tags: Option<Value>,
    pub linked_person_id: Option<String>,
    pub linked_organization_id: Option<String>,
    pub created_from_event_id: Option<String>,
    pub created_by_actor_id: Option<String>,
}

impl NewTask {
    pub fn with_provenance(
        mut self,
        provenance_kind: impl Into<String>,
        provenance_id: impl Into<String>,
    ) -> Self {
        self.provenance_kind = Some(provenance_kind.into());
        self.provenance_id = Some(provenance_id.into());
        self
    }

    pub fn with_source_defaults(
        mut self,
        source_kind: impl Into<String>,
        source_id: impl Into<String>,
    ) -> Self {
        if self.source_kind.is_none() {
            self.source_kind = Some(source_kind.into());
        }
        if self.source_id.is_none() {
            self.source_id = Some(source_id.into());
        }
        if self.source_type.is_none() {
            self.source_type = Some("manual".to_owned());
        }
        self
    }

    pub fn with_observation_source(mut self, observation_id: impl Into<String>) -> Self {
        let observation_id = observation_id.into();
        self.source_kind = Some("observation".to_owned());
        self.source_id = Some(observation_id);
        self.source_type = Some("observation".to_owned());
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub makosh_status: Option<String>,
    pub priority_score: Option<f64>,
    pub risk_score: Option<f64>,
    pub readiness_score: Option<f64>,
    pub area: Option<String>,
    pub why: Option<String>,
    pub outcome: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub waiting_reason: Option<String>,
    pub energy_type: Option<String>,
    pub confidentiality: Option<String>,
    pub tags: Option<Value>,
    pub task_metadata: Option<Value>,
    pub linked_person_id: Option<String>,
    pub linked_organization_id: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TaskListQuery {
    pub status: Option<String>,
    pub project_id: Option<String>,
    pub source_type: Option<String>,
    pub limit: Option<i64>,
}

fn row_to_task(row: PgRow) -> Result<Task, sqlx::Error> {
    Ok(Task {
        task_id: row.try_get("task_id")?,
        task_candidate_id: row.try_get("task_candidate_id")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        provenance_kind: row.try_get("provenance_kind")?,
        provenance_id: row.try_get("provenance_id")?,
        source_kind: row.try_get("source_kind")?,
        source_id: row.try_get("source_id")?,
        source_type: row.try_get("source_type")?,
        project_id: row.try_get("project_id")?,
        status: row.try_get("status")?,
        makosh_status: row.try_get("makosh_status")?,
        priority_score: row.try_get("priority_score")?,
        risk_score: row.try_get("risk_score")?,
        readiness_score: row.try_get("readiness_score")?,
        area: row.try_get("area")?,
        why: row.try_get("why")?,
        outcome: row.try_get("outcome")?,
        due_at: row.try_get("due_at")?,
        completed_at: row.try_get("completed_at")?,
        archived_at: row.try_get("archived_at")?,
        waiting_reason: row.try_get("waiting_reason")?,
        energy_type: row.try_get("energy_type")?,
        confidentiality: row.try_get("confidentiality")?,
        tags: row.try_get("tags")?,
        task_metadata: row.try_get("task_metadata")?,
        linked_person_id: row.try_get("linked_person_id")?,
        linked_organization_id: row.try_get("linked_organization_id")?,
        created_from_event_id: row.try_get("created_from_event_id")?,
        created_by_actor_id: row.try_get("created_by_actor_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Debug, Error)]
pub enum TaskError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),
    #[error(transparent)]
    Core(#[from] TaskCoreError),
    #[error("task provenance is required")]
    MissingProvenance,
    #[error("invalid task provenance specification")]
    InvalidProvenanceSpec,
    #[error("unknown task provenance kind")]
    UnknownProvenanceKind,
    #[error("missing provenance observation")]
    MissingProvenanceObservation,
    #[error("missing provenance target")]
    MissingProvenanceReference,
    #[error("provenance target has no observation evidence")]
    MissingProvenanceEvidence,
    #[error("source identifier is required")]
    MissingSourceIdentifier,
    #[error("not found")]
    NotFound,
}
