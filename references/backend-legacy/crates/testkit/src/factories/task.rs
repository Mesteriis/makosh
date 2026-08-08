use chrono::Utc;
use makosh_hub_backend::domains::tasks::api::{NewTask, TaskStore};
use sqlx::postgres::PgPool;
use uuid::Uuid;

/// Factory for creating test tasks with sensible defaults.
pub struct TaskFactory<'a> {
    pool: &'a PgPool,
    title: String,
    description: Option<String>,
    status: Option<String>,
    priority_score: Option<f64>,
    area: Option<String>,
    due_at: Option<chrono::DateTime<Utc>>,
    project_id: Option<String>,
    linked_person_id: Option<String>,
    linked_organization_id: Option<String>,
}

impl<'a> TaskFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            title: format!("test-task-{}", Uuid::new_v4()),
            description: Some("Auto-generated test task".into()),
            status: Some("new".into()),
            priority_score: Some(0.5),
            area: Some("general".into()),
            due_at: None,
            project_id: None,
            linked_person_id: None,
            linked_organization_id: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_priority(mut self, score: f64) -> Self {
        self.priority_score = Some(score);
        self
    }

    pub fn with_area(mut self, area: impl Into<String>) -> Self {
        self.area = Some(area.into());
        self
    }

    pub fn with_due_date(mut self, due: chrono::DateTime<Utc>) -> Self {
        self.due_at = Some(due);
        self
    }

    pub fn with_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    pub async fn create(
        self,
    ) -> Result<
        makosh_hub_backend::domains::tasks::api::Task,
        makosh_hub_backend::domains::tasks::api::TaskError,
    > {
        let store = TaskStore::new(self.pool.clone());
        let new_task = NewTask {
            title: self.title,
            description: self.description,
            provenance_kind: Some("observation".into()),
            provenance_id: Some(format!("observation:v1:test-factory:{}", Uuid::new_v4())),
            source_kind: Some("import".into()),
            source_id: Some(format!("test-src-{}", Uuid::new_v4())),
            source_type: Some("import".into()),
            project_id: self.project_id,
            makosh_status: self.status,
            priority_score: self.priority_score,
            area: self.area,
            why: Some("Created by TaskFactory for integration testing".into()),
            due_at: self.due_at,
            energy_type: Some("medium".into()),
            confidentiality: Some("private_local".into()),
            tags: Some(serde_json::json!(["test", "factory"])),
            linked_person_id: self.linked_person_id,
            linked_organization_id: self.linked_organization_id,
            created_from_event_id: None,
            created_by_actor_id: Some("testkit".into()),
        };
        store.create(&new_task).await
    }
}
