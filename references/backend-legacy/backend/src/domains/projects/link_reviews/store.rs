use makosh_events_api::EventEnvelope;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use makosh_events_postgres::store::EventStore;
use makosh_observations_postgres::review_links::materialize_review_transition_link_in_transaction;

use super::constants::PROJECT_LINK_REVIEW_EVENT_TYPE;
use super::errors::ProjectLinkReviewError;
use super::events::ReviewEvent;
use super::models::{
    ProjectLinkReview, ProjectLinkReviewCommand, ProjectLinkReviewCommandResult,
    ProjectLinkTargetKind, ProjectReviewedTarget, ReviewEventApplication,
};
use super::rows::{row_to_project_link_review, row_to_project_reviewed_target};
use super::validation::validate_non_empty;

#[derive(Clone)]
pub struct ProjectLinkReviewStore {
    pool: PgPool,
}

impl ProjectLinkReviewStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn set_review_state(
        &self,
        command: &ProjectLinkReviewCommand,
    ) -> Result<ProjectLinkReviewCommandResult, ProjectLinkReviewError> {
        self.set_review_state_with_observation(command, None, None)
            .await
    }

    pub async fn set_review_state_with_observation(
        &self,
        command: &ProjectLinkReviewCommand,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<ProjectLinkReviewCommandResult, ProjectLinkReviewError> {
        let command_id = validate_non_empty("command_id", &command.command_id)?;
        let project_id = validate_non_empty("project_id", &command.project_id)?;
        let target_id = validate_non_empty("target_id", &command.target_id)?;
        let actor_id = validate_non_empty("actor_id", &command.actor_id)?;

        let mut transaction = self.pool.begin().await?;

        self.ensure_project_exists(&mut transaction, &project_id)
            .await?;
        self.ensure_target_exists(&mut transaction, command.target_kind, &target_id)
            .await?;

        let event_id = format!("project_link_review:{command_id}");
        let event = ProjectLinkReviewCommand {
            command_id,
            project_id: project_id.clone(),
            target_kind: command.target_kind,
            target_id: target_id.clone(),
            review_state: command.review_state,
            actor_id: actor_id.clone(),
        }
        .to_review_event(&event_id)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        self.apply_review_event_in_transaction(
            &mut transaction,
            ReviewEventApplication {
                target_kind: command.target_kind,
                project_id: &project_id,
                target_id: &target_id,
                review_state: command.review_state,
                event_id: &event.event_id,
                actor_id: &actor_id,
                reviewed_at: event.occurred_at,
            },
        )
        .await?;
        let metadata = match metadata {
            Some(extra) => Some(json!({
                "event_id": event_id,
                "context": extra,
            })),
            None => Some(json!({
                "event_id": event_id,
            })),
        };
        materialize_review_transition_link_in_transaction(
            &mut transaction,
            observation_id,
            "projects",
            "project_link_review",
            &event_id,
            "review_state",
            command.review_state.as_str(),
            metadata,
        )
        .await?;

        transaction.commit().await?;

        Ok(ProjectLinkReviewCommandResult {
            project_id,
            target_kind: command.target_kind,
            target_id,
            review_state: command.review_state,
            event_id,
        })
    }

    pub async fn apply_review_event(
        &self,
        event: &EventEnvelope,
    ) -> Result<(), ProjectLinkReviewError> {
        let parsed = ReviewEvent::from_payload(&event.payload)?;
        if event.event_type != PROJECT_LINK_REVIEW_EVENT_TYPE {
            return Err(ProjectLinkReviewError::InvalidEventType);
        }

        let actor_id = event
            .actor
            .as_ref()
            .and_then(|value| value.get("actor_id"))
            .and_then(Value::as_str)
            .ok_or(ProjectLinkReviewError::MissingActorId)?;
        let actor_id = validate_non_empty("actor_id", actor_id)?;
        let mut transaction = self.pool.begin().await?;

        self.ensure_project_exists(&mut transaction, &parsed.project_id)
            .await?;
        self.ensure_target_exists(&mut transaction, parsed.target_kind, &parsed.target_id)
            .await?;
        self.apply_review_event_in_transaction(
            &mut transaction,
            ReviewEventApplication {
                target_kind: parsed.target_kind,
                project_id: &parsed.project_id,
                target_id: &parsed.target_id,
                review_state: parsed.review_state,
                event_id: &event.event_id,
                actor_id: &actor_id,
                reviewed_at: event.occurred_at,
            },
        )
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn explicit_review(
        &self,
        project_id: &str,
        target_kind: ProjectLinkTargetKind,
        target_id: &str,
    ) -> Result<Option<ProjectLinkReview>, ProjectLinkReviewError> {
        let project_id = validate_non_empty("project_id", project_id)?;
        let target_id = validate_non_empty("target_id", target_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                project_id,
                target_kind,
                target_id,
                review_state,
                event_id,
                actor_id,
                reviewed_at,
                created_at,
                updated_at
            FROM project_link_reviews
            WHERE project_id = $1 AND target_kind = $2 AND target_id = $3
            "#,
        )
        .bind(&project_id)
        .bind(target_kind.as_str())
        .bind(&target_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_project_link_review).transpose()
    }

    pub async fn active_message_ids_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectLinkReviewError> {
        let project_id = validate_non_empty("project_id", project_id)?;

        let rows = sqlx::query(
            r#"
            WITH keyword_matches AS (
                SELECT message_id AS target_id
                FROM communication_messages message
                WHERE EXISTS (
                    SELECT 1
                    FROM project_keywords keyword
                    WHERE keyword.project_id = $1
                      AND (
                          position(lower(keyword.keyword) in lower(message.subject)) > 0
                          OR position(lower(keyword.keyword) in lower(message.body_text)) > 0
                      )
                )
            ),
            confirmed AS (
                SELECT target_id
                FROM project_link_reviews
                WHERE project_id = $1
                  AND target_kind = 'message'
                  AND review_state = 'user_confirmed'
            ),
            rejected AS (
                SELECT target_id
                FROM project_link_reviews
                WHERE project_id = $1
                  AND target_kind = 'message'
                  AND review_state = 'user_rejected'
            ),
            active AS (
                SELECT target_id, 'suggested' AS review_state FROM keyword_matches
                UNION ALL
                SELECT target_id, 'user_confirmed' AS review_state FROM confirmed
            )
            SELECT active.target_id, max(active.review_state) AS review_state
            FROM active
            WHERE NOT EXISTS (
                SELECT 1
                FROM rejected
                WHERE rejected.target_id = active.target_id
            )
            GROUP BY active.target_id
            ORDER BY active.target_id
            "#,
        )
        .bind(&project_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_project_reviewed_target)
            .collect()
    }

    pub async fn active_document_ids_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<ProjectReviewedTarget>, ProjectLinkReviewError> {
        let project_id = validate_non_empty("project_id", project_id)?;

        let rows = sqlx::query(
            r#"
            WITH keyword_matches AS (
                SELECT document_id AS target_id
                FROM documents document
                WHERE EXISTS (
                    SELECT 1
                    FROM project_keywords keyword
                    WHERE keyword.project_id = $1
                      AND (
                          position(lower(keyword.keyword) in lower(document.title)) > 0
                          OR position(lower(keyword.keyword) in lower(document.extracted_text)) > 0
                      )
                )
            ),
            confirmed AS (
                SELECT target_id
                FROM project_link_reviews
                WHERE project_id = $1
                  AND target_kind = 'document'
                  AND review_state = 'user_confirmed'
            ),
            rejected AS (
                SELECT target_id
                FROM project_link_reviews
                WHERE project_id = $1
                  AND target_kind = 'document'
                  AND review_state = 'user_rejected'
            ),
            active AS (
                SELECT target_id, 'suggested' AS review_state FROM keyword_matches
                UNION ALL
                SELECT target_id, 'user_confirmed' AS review_state FROM confirmed
            )
            SELECT active.target_id, max(active.review_state) AS review_state
            FROM active
            WHERE NOT EXISTS (
                SELECT 1
                FROM rejected
                WHERE rejected.target_id = active.target_id
            )
            GROUP BY active.target_id
            ORDER BY active.target_id
            "#,
        )
        .bind(&project_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_project_reviewed_target)
            .collect()
    }
}
