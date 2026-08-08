use makosh_hub_backend::application::review_transitions::TaskCandidateReviewApplicationService;
use makosh_hub_backend::domains::tasks::candidates::models::TaskCandidateReviewState;

use super::support::{
    build_review_event, live_task_candidate_context, seed_message, unique_suffix,
};

#[tokio::test]
async fn task_candidate_review_event_rebuilds_state_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let message_id = seed_message(
        &context,
        suffix,
        &format!("event-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-event-{suffix}"),
        &format!("Task event {suffix}"),
        "Action: verify integration",
    )
    .await;

    let _ = context
        .store
        .refresh_deterministic_candidates(100)
        .await
        .expect("refresh");
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("message observation id");
    let task_candidate_id: String = sqlx::query_scalar(
        "SELECT task_candidate_id FROM task_candidates WHERE source_id = $1 AND source_kind = 'observation'",
    )
    .bind(&message_observation_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate id");

    let confirm_event = build_review_event(
        &task_candidate_id,
        TaskCandidateReviewState::UserConfirmed,
        "event-reviewer",
        &format!("task-candidate-event-confirm-{suffix}"),
    );
    let reject_event = build_review_event(
        &task_candidate_id,
        TaskCandidateReviewState::UserRejected,
        "event-reviewer",
        &format!("task-candidate-event-reject-{suffix}"),
    );

    context
        .event_store
        .append(&confirm_event)
        .await
        .expect("append confirm event");
    context
        .event_store
        .append(&reject_event)
        .await
        .expect("append reject event");

    let confirm_event = context
        .event_store
        .get_by_id(&confirm_event.event_id)
        .await
        .expect("load confirm event")
        .expect("confirm event exists");
    TaskCandidateReviewApplicationService::new(context.pool.clone())
        .apply_review_event(&confirm_event)
        .await
        .expect("apply confirm event");
    let reject_event = context
        .event_store
        .get_by_id(&reject_event.event_id)
        .await
        .expect("load reject event")
        .expect("reject event exists");
    TaskCandidateReviewApplicationService::new(context.pool.clone())
        .apply_review_event(&reject_event)
        .await
        .expect("apply reject event");

    let state: String =
        sqlx::query_scalar("SELECT review_state FROM task_candidates WHERE task_candidate_id = $1")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("load state");
    assert_eq!(state, "user_rejected");

    let event_id: String =
        sqlx::query_scalar("SELECT event_id FROM task_candidates WHERE task_candidate_id = $1")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("load event id");
    assert_eq!(
        event_id,
        format!("task_candidate_review:task-candidate-event-reject-{suffix}")
    );
}
