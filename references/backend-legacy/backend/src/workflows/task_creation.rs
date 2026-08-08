use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::postgres::Postgres;
use sqlx::{PgPool, Transaction};

use crate::app::error::types::ApiError;
use crate::domains::tasks::api::{NewTask, Task};
use crate::domains::tasks::core::observation_links::materialize_task_observation_link_in_transaction;
use crate::domains::tasks::workflow_commands::TaskWorkflowCommands;

pub(crate) struct WorkflowTaskCreateInput {
    pub title: String,
    pub description: Option<String>,
    pub provenance_kind: Option<String>,
    pub provenance_id: Option<String>,
    pub source_kind: String,
    pub source_id: String,
    pub source_type: String,
    pub due_at: Option<DateTime<Utc>>,
    pub created_from_event_id: String,
    pub created_by_actor_id: String,
    pub projection_observation_id: Option<String>,
    pub projection_metadata: Option<Value>,
}

pub(crate) async fn create_task_from_workflow_input(
    pool: &PgPool,
    transaction: &mut Transaction<'_, Postgres>,
    input: WorkflowTaskCreateInput,
) -> Result<Task, ApiError> {
    let task = TaskWorkflowCommands::new(pool.clone())
        .create_in_transaction(
            transaction,
            &NewTask {
                title: input.title,
                description: input.description,
                provenance_kind: input.provenance_kind,
                provenance_id: input.provenance_id,
                source_kind: Some(input.source_kind),
                source_id: Some(input.source_id),
                source_type: Some(input.source_type),
                project_id: None,
                makosh_status: Some("new".to_owned()),
                priority_score: None,
                area: None,
                why: None,
                due_at: input.due_at,
                energy_type: None,
                confidentiality: Some("private_local".to_owned()),
                tags: None,
                linked_person_id: None,
                linked_organization_id: None,
                created_from_event_id: Some(input.created_from_event_id),
                created_by_actor_id: Some(input.created_by_actor_id),
            },
        )
        .await
        .map_err(ApiError::from)?;

    materialize_task_observation_link_in_transaction(
        transaction,
        input.projection_observation_id.as_deref(),
        Some("workflow_action_projection"),
        &task.task_id,
        input.projection_metadata,
    )
    .await
    .map_err(ApiError::from)?;

    Ok(task)
}
