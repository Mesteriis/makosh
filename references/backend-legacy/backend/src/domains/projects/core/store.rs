use makosh_projects_api::ProjectWritePort;
use sqlx::postgres::PgPool;

use super::constants::{DEFAULT_PROJECT_LIMIT, PROJECT_DETAIL_ITEM_LIMIT};
use super::errors::ProjectStoreError;
use super::ids::project_graph_node_id;
use super::models::{NewProject, Project, ProjectDetail, ProjectSummary};
use super::rows::row_to_project;
use super::validation::{validate_limit, validate_non_empty};

#[derive(Clone)]
pub struct ProjectStore {
    pub(super) pool: PgPool,
}

impl ProjectStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_project(&self, project: &NewProject) -> Result<Project, ProjectStoreError> {
        let value = makosh_projects_postgres::ProjectPostgresReadQuery::new(self.pool.clone())
            .upsert(&makosh_projects_api::ProjectUpsert {
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
            .map_err(|error| {
                ProjectStoreError::Sqlx(sqlx::Error::Protocol(error.to_string().into()))
            })?;
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

    pub async fn list_projects(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<ProjectSummary>, ProjectStoreError> {
        let limit = validate_limit(limit.unwrap_or(DEFAULT_PROJECT_LIMIT))?;
        let rows = sqlx::query(
            r#"
            SELECT
                project_id,
                name,
                kind,
                status,
                description,
                owner_display_name,
                progress_percent,
                start_date,
                target_date,
                created_at,
                updated_at
            FROM projects
            ORDER BY updated_at DESC, name, project_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let projects = rows
            .into_iter()
            .map(row_to_project)
            .collect::<Result<Vec<_>, _>>()?;
        let mut summaries = Vec::with_capacity(projects.len());
        for project in projects {
            summaries.push(ProjectSummary {
                graph_node_id: project_graph_node_id(&project.project_id),
                stats: self.project_stats(&project.project_id).await?,
                project,
            });
        }

        Ok(summaries)
    }

    pub async fn project_detail(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectDetail>, ProjectStoreError> {
        let project_id = validate_non_empty("project_id", project_id)?;
        let Some(project) = self.project_by_id(&project_id).await? else {
            return Ok(None);
        };

        let key_personas = self
            .project_personas(&project.project_id, PROJECT_DETAIL_ITEM_LIMIT)
            .await?;

        Ok(Some(ProjectDetail {
            graph_node_id: project_graph_node_id(&project.project_id),
            stats: self.project_stats(&project.project_id).await?,
            timeline: self
                .project_timeline(&project.project_id, PROJECT_DETAIL_ITEM_LIMIT)
                .await?,
            key_personas: key_personas.clone(),
            #[allow(deprecated)]
            key_people: key_personas,
            recent_messages: self
                .project_messages(&project.project_id, PROJECT_DETAIL_ITEM_LIMIT)
                .await?,
            documents: self
                .project_documents(&project.project_id, PROJECT_DETAIL_ITEM_LIMIT)
                .await?,
            project,
        }))
    }
}
