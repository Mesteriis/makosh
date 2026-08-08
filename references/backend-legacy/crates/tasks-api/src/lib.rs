use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{future::Future, pin::Pin};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskRead {
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

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TaskListQuery {
    pub status: Option<String>,
    pub project_id: Option<String>,
    pub source_type: Option<String>,
    pub limit: Option<i64>,
}
pub type TaskListFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<TaskRead>, TaskQueryError>> + Send + 'a>>;
pub type TaskGetFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<TaskRead>, TaskQueryError>> + Send + 'a>>;
pub trait TaskReadPort: Send + Sync {
    fn list<'a>(&'a self, query: TaskListQuery) -> TaskListFuture<'a>;
    fn get<'a>(&'a self, task_id: &'a str) -> TaskGetFuture<'a>;
}
#[derive(Debug, thiserror::Error)]
#[error("task query failed: {0}")]
pub struct TaskQueryError(pub String);
