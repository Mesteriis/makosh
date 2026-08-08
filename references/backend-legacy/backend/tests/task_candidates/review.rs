use makosh_hub_backend::application::review_transitions::TaskCandidateReviewApplicationService;
use makosh_hub_backend::domains::tasks::candidates::models::{
    TaskCandidateReviewCommand, TaskCandidateReviewState,
};
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;
use serde_json::json;
use sqlx::Row;

use super::support::{live_task_candidate_context, seed_message, unique_suffix};

#[tokio::test]
async fn task_candidate_review_confirm_creates_active_task_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let message_id = seed_message(
        &context,
        suffix,
        &format!("confirm-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-confirm-{suffix}"),
        &format!("Task confirm {suffix}"),
        "Action: review this item",
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
    let candidate_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM task_candidates WHERE task_candidate_id = $1",
    )
    .bind(&task_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate observation id");

    let result = context
        .review(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-confirm-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm");
    assert_eq!(result.review_state, TaskCandidateReviewState::UserConfirmed);
    assert_eq!(result.task_candidate_id, task_candidate_id);

    let task_row: (String, String, String, String, String, String) = sqlx::query_as(
        r#"
        SELECT task_id, provenance_kind, provenance_id, source_kind, source_id, source_type
        FROM tasks
        WHERE task_candidate_id = $1
        "#,
    )
    .bind(&task_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("task row");
    assert_eq!(task_row.1, "observation");
    assert_eq!(task_row.2, candidate_observation_id);
    assert_eq!(task_row.3, "observation");
    assert_eq!(task_row.4, task_row.2);
    assert_eq!(task_row.5, "observation");

    let observation_row: (String, String) = sqlx::query_as(
        r#"
        SELECT source_ref, kind.code
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&task_row.2)
    .fetch_one(&context.pool)
    .await
    .expect("provenance observation");
    assert!(!observation_row.0.trim().is_empty());
    assert_eq!(observation_row.1, "COMMUNICATION_MESSAGE");
}

#[tokio::test]
async fn task_candidate_store_review_with_observation_materializes_transition_link_against_postgres()
 {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let message_id = seed_message(
        &context,
        suffix,
        &format!("confirm-link-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-confirm-link-{suffix}"),
        &format!("Task confirm link {suffix}"),
        "Action: review this link owner path",
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
    let review_observation = ObservationStore::new(context.pool.clone())
        .capture(
            &NewObservation::new(
                "REVIEW_TRANSITION",
                ObservationOriginKind::Manual,
                chrono::Utc::now(),
                json!({
                    "task_candidate_id": task_candidate_id,
                    "operation": "task_candidate_review",
                }),
                format!("manual://task-candidate-review/{suffix}"),
            )
            .provenance(json!({
                "source": "task_candidates.review.test",
            })),
        )
        .await
        .expect("capture review observation");

    let result = TaskCandidateReviewApplicationService::new(context.pool.clone())
        .review_with_observation(
            &TaskCandidateReviewCommand {
                command_id: format!("task-candidate-confirm-link-{suffix}"),
                task_candidate_id: task_candidate_id.clone(),
                review_state: TaskCandidateReviewState::UserConfirmed,
                actor_id: "tests-reviewer".to_owned(),
            },
            &review_observation.observation_id,
            json!({
                "source": "task_candidates.review.test",
            }),
        )
        .await
        .expect("confirm");
    assert_eq!(result.review_state, TaskCandidateReviewState::UserConfirmed);

    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'tasks'
           AND entity_kind = 'task_candidate'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&task_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("task candidate review transition link");
    let linked_observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: serde_json::Value = link_row.try_get("metadata").expect("metadata");
    assert_eq!(linked_observation_id, review_observation.observation_id);
    assert_eq!(metadata["review_state"], json!("user_confirmed"));
    assert_eq!(
        metadata["event_id"],
        json!(format!(
            "task_candidate_review:task-candidate-confirm-link-{suffix}"
        ))
    );
}

#[tokio::test]
async fn task_candidate_review_confirm_materializes_obligation_candidate_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let statement = format!("send the countersigned agreement {suffix}");
    let quote = format!("I will {statement} by Friday 5pm.");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("obligation-confirm-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-obligation-confirm-{suffix}"),
        &format!("Obligation confirm {suffix}"),
        &quote,
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
    let candidate_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM task_candidates WHERE task_candidate_id = $1",
    )
    .bind(&task_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate observation id");

    let result = context
        .review(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-obligation-confirm-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm");
    assert_eq!(result.review_state, TaskCandidateReviewState::UserConfirmed);

    let task_id: String =
        sqlx::query_scalar("SELECT task_id FROM tasks WHERE task_candidate_id = $1")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("task id");

    let obligation_rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        r#"
        SELECT
            o.obligation_id,
            o.review_state,
            o.obligated_entity_kind,
            o.obligated_entity_id,
            l.link_kind
        FROM obligations o
        JOIN obligation_task_links l
          ON l.obligation_id = o.obligation_id
        WHERE l.task_id = $1
        ORDER BY o.obligation_id
        "#,
    )
    .bind(&task_id)
    .fetch_all(&context.pool)
    .await
    .expect("linked obligation rows");
    assert_eq!(
        obligation_rows.len(),
        1,
        "confirming an obligation-derived candidate should create one linked obligation"
    );
    assert_eq!(obligation_rows[0].1, "user_confirmed");
    assert_eq!(obligation_rows[0].2, "persona");
    assert_eq!(obligation_rows[0].3, "persona:owner");
    assert_eq!(obligation_rows[0].4, "fulfillment_task");

    let evidence_row: (String, String, Option<String>, Option<String>) = sqlx::query_as(
        r#"
        SELECT source_kind, source_id, quote, observation_id
        FROM obligation_evidence
        WHERE obligation_id = $1
        "#,
    )
    .bind(&obligation_rows[0].0)
    .fetch_one(&context.pool)
    .await
    .expect("obligation evidence");
    assert_eq!(evidence_row.0, "observation");
    assert_eq!(evidence_row.1, candidate_observation_id);
    assert_eq!(evidence_row.2.as_deref(), Some(quote.as_str()));
    assert_eq!(evidence_row.3.as_deref(), Some(evidence_row.1.as_str()));
}

#[tokio::test]
async fn obligation_task_candidate_reset_demotes_obligation_review_state_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let statement = format!("send the vendor approval memo {suffix}");
    let quote = format!("I will {statement} by Friday 5pm.");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("obligation-reset-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-obligation-reset-{suffix}"),
        &format!("Obligation reset {suffix}"),
        &quote,
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

    context
        .review(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-obligation-reset-confirm-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm");

    let reset = context
        .review(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-obligation-reset-reset-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::Suggested,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("reset");
    assert_eq!(reset.review_state, TaskCandidateReviewState::Suggested);

    let obligation_row: (String, String) = sqlx::query_as(
        r#"
        SELECT obligation_id, review_state
        FROM obligations
        WHERE statement = $1
        "#,
    )
    .bind(&statement)
    .fetch_one(&context.pool)
    .await
    .expect("obligation row");
    assert_eq!(
        obligation_row.1, "suggested",
        "resetting the candidate must demote the durable Obligation review state"
    );

    let task_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM tasks WHERE task_candidate_id = $1)")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("task exists");
    assert!(!task_exists);

    let link_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM obligation_task_links WHERE obligation_id = $1")
            .bind(&obligation_row.0)
            .fetch_one(&context.pool)
            .await
            .expect("obligation task link count");
    assert_eq!(link_count, 0);
}

#[tokio::test]
async fn task_candidate_review_reset_removes_active_task_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let message_id = seed_message(
        &context,
        suffix,
        &format!("reset-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-task-candidate-reset-{suffix}"),
        &format!("Task reset {suffix}"),
        "Please handle next step",
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

    let _ = context
        .review(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-reset-confirm-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm");

    let reset = context
        .review(&TaskCandidateReviewCommand {
            command_id: format!("task-candidate-reset-reset-{suffix}"),
            task_candidate_id: task_candidate_id.clone(),
            review_state: TaskCandidateReviewState::Suggested,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("reset");
    assert_eq!(reset.review_state, TaskCandidateReviewState::Suggested);

    let state: String =
        sqlx::query_scalar("SELECT review_state FROM task_candidates WHERE task_candidate_id = $1")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("candidate state");
    assert_eq!(state, "suggested");

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM tasks WHERE task_candidate_id = $1)")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("task exists");
    assert!(!exists);
}

#[tokio::test]
async fn task_candidate_schema_rejects_legacy_non_observation_candidate_against_postgres() {
    let Some(context) = live_task_candidate_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let task_candidate_id = format!("task_candidate:v1:legacy-non-observation:{suffix}");

    sqlx::query(
        r#"
        INSERT INTO task_candidates (
            task_candidate_id,
            source_kind,
            source_id,
            observation_id,
            candidate_kind,
            candidate_metadata,
            title,
            confidence,
            review_state,
            evidence_excerpt
        )
        VALUES (
            $1,
            'document',
            $2,
            NULL,
            'task',
            '{}'::jsonb,
            $3,
            0.81,
            'suggested',
            $4
        )
        "#,
    )
    .bind(&task_candidate_id)
    .bind(format!("legacy-document:{suffix}"))
    .bind(format!("Legacy activation candidate {suffix}"))
    .bind(format!("Legacy evidence {suffix}"))
    .execute(&context.pool)
    .await
    .expect_err("legacy non-observation candidate must violate current observation constraint");

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM tasks WHERE task_candidate_id = $1)")
            .bind(&task_candidate_id)
            .fetch_one(&context.pool)
            .await
            .expect("task exists");
    assert!(!exists);
}
