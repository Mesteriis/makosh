use makosh_events_api::EventEnvelope;
mod list;
mod refresh;
mod review;
mod task_activation;

use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};

use super::errors::TaskCandidateError;
use super::models::{
    StoredCandidateRow, TaskCandidate, TaskCandidateReviewCommand,
    TaskCandidateReviewCommandResult, TaskCandidateReviewState,
};

use serde_json::Value;

#[derive(Clone)]
pub struct TaskCandidateStore {
    pool: PgPool,
}

impl TaskCandidateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn refresh_deterministic_candidates(
        &self,
        limit: i64,
    ) -> Result<usize, TaskCandidateError> {
        refresh::refresh_deterministic_candidates(&self.pool, limit).await
    }

    pub(crate) async fn set_review_state_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        command: &TaskCandidateReviewCommand,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<(TaskCandidateReviewCommandResult, StoredCandidateRow), TaskCandidateError> {
        review::set_candidate_review_state_in_transaction(
            transaction,
            command,
            observation_id,
            metadata,
        )
        .await
    }

    pub(crate) async fn apply_review_event_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event: &EventEnvelope,
    ) -> Result<(String, TaskCandidateReviewState, StoredCandidateRow), TaskCandidateError> {
        review::apply_candidate_review_event_in_transaction(transaction, event).await
    }

    pub(crate) async fn delete_task_for_candidate_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        task_candidate_id: &str,
    ) -> Result<(), TaskCandidateError> {
        review::delete_task_for_candidate_in_transaction(transaction, task_candidate_id).await
    }

    pub async fn list_candidates(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<TaskCandidate>, TaskCandidateError> {
        list::list_candidates(&self.pool, limit).await
    }

    pub async fn refresh_message_candidates_for_ids(
        &self,
        message_ids: &[String],
    ) -> Result<usize, TaskCandidateError> {
        refresh::refresh_message_candidates_for_ids(&self.pool, message_ids).await
    }
}
