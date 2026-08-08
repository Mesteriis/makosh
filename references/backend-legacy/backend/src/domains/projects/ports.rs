use makosh_projects_api::{
    ProjectGraphReadPort, ProjectMatchedDocument, ProjectMatchedMessage, ProjectProjectionSource,
    ProjectUpsert, ProjectWritePort,
};
use makosh_projects_postgres::ProjectPostgresReadQuery;
use sqlx::PgPool;

use super::core::models::{NewProject, Project};
use super::link_reviews::errors::ProjectLinkReviewError;
use super::link_reviews::models::ProjectLinkReviewCommand;
use super::link_reviews::store::ProjectLinkReviewStore;

#[derive(Clone)]
pub struct ProjectCommandPort {
    write: ProjectPostgresReadQuery,
}

impl ProjectCommandPort {
    pub fn new(pool: PgPool) -> Self {
        Self {
            write: ProjectPostgresReadQuery::new(pool),
        }
    }

    pub async fn upsert_project(
        &self,
        project: &NewProject,
    ) -> Result<Project, ProjectCommandPortError> {
        let value = self
            .write
            .upsert(&ProjectUpsert {
                project_id: project.project_id.clone(),
                name: project.name.clone(),
                kind: project.kind.clone(),
                status: project.status.clone(),
                description: project.description.clone(),
                owner_display_name: project.owner_display_name.clone(),
                progress_percent: project.progress_percent,
                start_date: project.start_date,
                target_date: project.target_date,
                keywords: project.keywords.clone(),
            })
            .await
            .map_err(|error| ProjectCommandPortError::Query(error.to_string()))?;
        Ok(Project {
            project_id: value.project_id,
            name: value.name,
            kind: value.kind,
            status: value.status,
            description: value.description,
            owner_display_name: value.owner_display_name,
            progress_percent: value.progress_percent,
            start_date: value.start_date,
            target_date: value.target_date,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }

    pub(crate) async fn graph_projection_projects(
        &self,
    ) -> Result<Vec<ProjectProjectionSource>, ProjectCommandPortError> {
        self.write
            .projection_projects()
            .await
            .map_err(|error| ProjectCommandPortError::Query(error.to_string()))
    }

    pub(crate) async fn matching_project_messages(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectMatchedMessage>, ProjectCommandPortError> {
        self.write
            .matching_messages(project_id)
            .await
            .map_err(|error| ProjectCommandPortError::Query(error.to_string()))
    }

    pub(crate) async fn matching_project_documents(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectMatchedDocument>, ProjectCommandPortError> {
        self.write
            .matching_documents(project_id)
            .await
            .map_err(|error| ProjectCommandPortError::Query(error.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectCommandPortError {
    #[error("project command query failed: {0}")]
    Query(String),
}

#[derive(Clone)]
pub struct ProjectLinkReviewPort(ProjectLinkReviewStore);

impl ProjectLinkReviewPort {
    pub fn new(pool: PgPool) -> Self {
        Self(ProjectLinkReviewStore::new(pool))
    }

    pub async fn set_review_state(
        &self,
        command: &ProjectLinkReviewCommand,
    ) -> Result<super::link_reviews::models::ProjectLinkReviewCommandResult, ProjectLinkReviewError>
    {
        self.0.set_review_state(command).await
    }
}
