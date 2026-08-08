use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

use super::errors::ProjectLinkReviewError;
use super::models::{ProjectLinkReviewCommand, ProjectLinkReviewCommandResult};
use super::store::ProjectLinkReviewStore;

#[derive(Clone)]
pub struct ProjectLinkReviewService {
    pool: PgPool,
}

impl ProjectLinkReviewService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn review_manual(
        &self,
        command: &ProjectLinkReviewCommand,
    ) -> Result<ProjectLinkReviewCommandResult, ProjectLinkReviewServiceError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "project_id": command.project_id,
                        "target_kind": command.target_kind.as_str(),
                        "target_id": command.target_id,
                        "review_state": command.review_state.as_str(),
                        "event_id": format!("project_link_review:{}", command.command_id),
                        "operation": "project_link_review",
                    }),
                    format!(
                        "project://{}/link-review/{}/{}",
                        command.project_id,
                        command.target_kind.as_str(),
                        command.target_id
                    ),
                )
                .provenance(json!({
                    "captured_by": "projects.link_review_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?;

        Ok(ProjectLinkReviewStore::new(self.pool.clone())
            .set_review_state_with_observation(
                command,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "projects.link_review_service.review_manual",
                    "operation": "review_manual",
                })),
            )
            .await?)
    }
}

#[derive(Debug, Error)]
pub enum ProjectLinkReviewServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error(transparent)]
    ProjectLinkReview(#[from] ProjectLinkReviewError),
}
