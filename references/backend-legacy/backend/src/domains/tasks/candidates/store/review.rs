use chrono::{DateTime, Utc};
use makosh_events_api::EventEnvelope;
use serde_json::{Value, json};
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use makosh_events_postgres::store::EventStore;
use makosh_observations_postgres::review_links::materialize_review_transition_link_in_transaction;

use super::super::constants::{TASK_CANDIDATE_EVENT_PREFIX, TASK_CANDIDATE_REVIEW_EVENT_TYPE};
use super::super::errors::TaskCandidateError;
use super::super::events::{ReviewCommandEvent, ReviewEventPayload};
use super::super::models::{
    StoredCandidateRow, TaskCandidateReviewCommand, TaskCandidateReviewCommandResult,
    TaskCandidateReviewState,
};
use super::super::persistence::row_task_candidate;
use super::super::validation::validate_non_empty;
use super::task_activation::upsert_task_in_transaction;

pub(crate) async fn set_candidate_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    command: &TaskCandidateReviewCommand,
    observation_id: Option<&str>,
    metadata: Option<Value>,
) -> Result<(TaskCandidateReviewCommandResult, StoredCandidateRow), TaskCandidateError> {
    let command_id = validate_non_empty("command_id", &command.command_id)?;
    let task_candidate_id = validate_non_empty("task_candidate_id", &command.task_candidate_id)?;
    let actor_id = validate_non_empty("actor_id", &command.actor_id)?;
    let event_id = format!("{TASK_CANDIDATE_EVENT_PREFIX}{command_id}");
    let event = ReviewCommandEvent {
        command_id,
        task_candidate_id: task_candidate_id.clone(),
        review_state: command.review_state,
        actor_id: actor_id.clone(),
        event_id: event_id.clone(),
        occurred_at: Utc::now(),
    }
    .into_event()?;
    EventStore::append_in_transaction(transaction, &event).await?;
    let candidate = apply_candidate_review_state_in_transaction(
        transaction,
        &task_candidate_id,
        command.review_state,
        &event_id,
        &actor_id,
        event.occurred_at,
    )
    .await?;
    let metadata = metadata
        .map(|extra| json!({ "event_id": event_id, "context": extra }))
        .or_else(|| Some(json!({ "event_id": event_id })));
    materialize_review_transition_link_in_transaction(
        transaction,
        observation_id,
        "tasks",
        "task_candidate",
        &task_candidate_id,
        "review_state",
        command.review_state.as_str(),
        metadata,
    )
    .await?;
    Ok((
        TaskCandidateReviewCommandResult {
            task_candidate_id,
            review_state: command.review_state,
            event_id,
        },
        candidate,
    ))
}

pub(crate) async fn apply_candidate_review_event_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    event: &EventEnvelope,
) -> Result<(String, TaskCandidateReviewState, StoredCandidateRow), TaskCandidateError> {
    if event.event_type != TASK_CANDIDATE_REVIEW_EVENT_TYPE {
        return Err(TaskCandidateError::InvalidEventType);
    }

    let payload = ReviewEventPayload::from_payload(&event.payload)?;
    let actor_id = event
        .actor
        .as_ref()
        .and_then(|value| value.get("actor_id"))
        .and_then(Value::as_str)
        .ok_or(TaskCandidateError::MissingActorId)?;
    let actor_id = validate_non_empty("actor_id", actor_id)?;

    let candidate = apply_candidate_review_state_in_transaction(
        transaction,
        &payload.task_candidate_id,
        payload.review_state,
        &event.event_id,
        &actor_id,
        event.occurred_at,
    )
    .await?;
    Ok((payload.task_candidate_id, payload.review_state, candidate))
}

pub(crate) async fn apply_candidate_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    review_state: TaskCandidateReviewState,
    event_id: &str,
    actor_id: &str,
    reviewed_at: DateTime<Utc>,
) -> Result<StoredCandidateRow, TaskCandidateError> {
    let candidate = row_task_candidate(transaction, task_candidate_id).await?;
    match review_state {
        TaskCandidateReviewState::UserConfirmed => {
            upsert_task_in_transaction(
                transaction,
                task_candidate_id,
                &candidate,
                event_id,
                actor_id,
            )
            .await?;
            update_candidate_review_state(
                transaction,
                task_candidate_id,
                review_state,
                event_id,
                actor_id,
                reviewed_at,
            )
            .await?;
        }
        TaskCandidateReviewState::Suggested | TaskCandidateReviewState::UserRejected => {
            update_candidate_review_state(
                transaction,
                task_candidate_id,
                review_state,
                event_id,
                actor_id,
                reviewed_at,
            )
            .await?;
        }
    }
    Ok(candidate)
}

async fn update_candidate_review_state(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
    review_state: TaskCandidateReviewState,
    event_id: &str,
    actor_id: &str,
    reviewed_at: DateTime<Utc>,
) -> Result<(), TaskCandidateError> {
    sqlx::query(
        r#"
        UPDATE task_candidates
        SET
            review_state = $1,
            event_id = $2,
            actor_id = $3,
            reviewed_at = $4,
            updated_at = now()
        WHERE task_candidate_id = $5
        "#,
    )
    .bind(review_state.as_str())
    .bind(event_id)
    .bind(actor_id)
    .bind(reviewed_at)
    .bind(task_candidate_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub(crate) async fn delete_task_for_candidate_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    task_candidate_id: &str,
) -> Result<(), TaskCandidateError> {
    sqlx::query("DELETE FROM tasks WHERE task_candidate_id = $1")
        .bind(task_candidate_id)
        .execute(&mut **transaction)
        .await?;

    Ok(())
}
