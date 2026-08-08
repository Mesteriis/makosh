use makosh_hub_backend::domains::projects::core::errors::ProjectStoreError;
use makosh_hub_backend::domains::projects::core::models::{NewProject, Project};
use makosh_hub_backend::domains::projects::core::store::ProjectStore;
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct ProjectFactory<'a> {
    pool: &'a PgPool,
    name: String,
    kind: String,
    status: String,
    description: String,
}

impl<'a> ProjectFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            name: format!("test-project-{}", Uuid::new_v4()),
            kind: "software".into(),
            status: "active".into(),
            description: "Auto-generated test project".into(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = kind.into();
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub async fn create(self) -> Result<Project, ProjectStoreError> {
        let store = ProjectStore::new(self.pool.clone());
        let project_id = format!("proj:v1:test:{}", Uuid::new_v4());
        let new_project = NewProject {
            project_id,
            name: self.name,
            kind: self.kind,
            status: self.status,
            description: self.description,
            owner_display_name: "Test Owner".into(),
            progress_percent: 0,
            start_date: None,
            target_date: None,
            keywords: vec!["test".into(), "factory".into()],
        };
        store.upsert_project(&new_project).await
    }
}
