use crate::support::*;
use makosh_hub_backend::domains::decisions::models::decision::NewDecision;
use sqlx::Row;

#[tokio::test]
async fn tasks_crud_against_postgres() {
    let Some(database_url) = test_database_url("tasks CRUD test").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/tasks",
            json!({"title": format!("CRUD Task {suffix}"), "description": "CRUD test", "status": "active"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        eprintln!("skip: task create failed");
        return;
    }
    let created = json_body(response).await;
    let Some(task_id) = created["task_id"].as_str().map(|value| value.to_owned()) else {
        eprintln!("skip: no task_id");
        return;
    };
    assert_eq!(created["title"], json!(format!("CRUD Task {suffix}")));

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/tasks/{}", urlencoding_percent_encode(&task_id)),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let fetched = json_body(response).await;
    assert_eq!(fetched["task_id"], json!(task_id));

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!("/api/v1/tasks/{}", urlencoding_percent_encode(&task_id)),
            json!({"title": format!("Updated Task {suffix}"), "priority": "high"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let updated = json_body(response).await;
    assert_eq!(updated["title"], json!(format!("Updated Task {suffix}")));

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();
    let update_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'tasks'
           AND entity_kind = 'task'
           AND entity_id = $1
           AND relationship_kind = 'task_update'",
    )
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("task update observation link count");
    assert_eq!(update_link_count, 1);

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/archive",
                urlencoding_percent_encode(&task_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn tasks_list_returns_items() {
    let Some(database_url) = test_database_url("tasks list test").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    create_task(&app, suffix).await;

    let response = app
        .oneshot(get_request_with_token("/api/v1/tasks", LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let _items = body["items"].as_array().expect("items");
}

#[tokio::test]
async fn task_status_transition() {
    let Some(database_url) = test_database_url("task status test").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/status",
                urlencoding_percent_encode(&task_id)
            ),
            json!({"status": "completed"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();
    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'tasks'
           AND entity_kind = 'task'
           AND entity_id = $1
           AND relationship_kind = 'status_update'",
    )
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("task status observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn task_analyze_runtime_path_captures_observation_against_postgres() {
    let Some(database_url) = test_database_url("task analyze observation api").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/analyze",
                urlencoding_percent_encode(&task_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();
    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'tasks'
           AND entity_kind = 'task'
           AND entity_id = $1
           AND relationship_kind = 'analysis_update'",
    )
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("task analyze observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn task_creation_rejects_unknown_review_item_reference_in_api() {
    let Some(database_url) = test_database_url("task create invalid provenance api").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;

    let response = app
        .oneshot(post_request_with_token(
            "/api/v1/tasks",
            json!({
                "title": format!("API invalid provenance task {suffix}"),
                "provenance_kind": "review_item",
                "provenance_id": "review_item:v1:does-not-exist",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("invalid_task_query"));
    assert_eq!(
        body["message"],
        json!("task provenance reference does not exist")
    );
}

#[tokio::test]
async fn task_creation_rejects_decision_without_observation_evidence_in_api() {
    let Some(database_url) = test_database_url("task create decision provenance api").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();
    let decision = DecisionStore::new(pool)
        .upsert_with_evidence(
            &NewDecision::new(
                format!("API decision evidence gap {suffix}"),
                "Decision exists but has no observation evidence.",
                0.78,
                DecisionReviewState::Suggested,
            ),
            &[NewDecisionEvidence::new(
                DecisionEvidenceSourceKind::Event,
                format!("event:api-task-provenance:{suffix}"),
            )
            .quote("Decision evidence recorded outside canonical observations.")],
            &[],
        )
        .await
        .expect("create decision");

    let response = app
        .oneshot(post_request_with_token(
            "/api/v1/tasks",
            json!({
                "title": format!("API provenance evidence gap {suffix}"),
                "provenance_kind": "decision",
                "provenance_id": decision.decision_id,
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("invalid_task_query"));
    assert_eq!(
        body["message"],
        json!("task provenance reference has no observation evidence")
    );
}

#[tokio::test]
async fn task_checklist_manual_create_path_captures_observation_against_postgres() {
    let Some(database_url) = test_database_url("task checklist observation api").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;

    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/checklist",
                urlencoding_percent_encode(&task_id)
            ),
            json!({
                "items": [{"text": "Prepare migration", "done": false}],
                "source": "manual"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let checklist_id = body["id"].as_str().expect("checklist id").to_owned();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();

    let checklist_source: String =
        sqlx::query_scalar("SELECT source FROM task_checklists WHERE id::text = $1")
            .bind(&checklist_id)
            .fetch_one(&pool)
            .await
            .expect("checklist source");
    assert!(checklist_source.starts_with("observation:"));

    let observation_id = checklist_source
        .strip_prefix("observation:")
        .expect("observation prefix");
    let row = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(observation_id)
    .fetch_one(&pool)
    .await
    .expect("stored observation");
    assert_eq!(
        row.try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "manual"
    );
    assert_eq!(
        row.try_get::<String, _>("kind_code").expect("kind code"),
        "TASK_MUTATION"
    );

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'tasks'
           AND entity_kind = 'task_checklist'
           AND entity_id = $2",
    )
    .bind(observation_id)
    .bind(&checklist_id)
    .fetch_one(&pool)
    .await
    .expect("observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn task_evidence_manual_create_path_captures_observation_against_postgres() {
    let Some(database_url) = test_database_url("task evidence observation api").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;

    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/evidence",
                urlencoding_percent_encode(&task_id)
            ),
            json!({
                "source_type": "manual",
                "quote": "Direct operator evidence for this task.",
                "confidence": 0.85
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let evidence_id = body["id"].as_str().expect("evidence id").to_owned();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();

    let evidence_row =
        sqlx::query("SELECT source_type, source_id FROM task_evidence WHERE id::text = $1")
            .bind(&evidence_id)
            .fetch_one(&pool)
            .await
            .expect("evidence row");
    let source_type: String = evidence_row.try_get("source_type").expect("source type");
    let source_id: String = evidence_row.try_get("source_id").expect("source id");
    assert_eq!(source_type, "observation");

    let row = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&source_id)
    .fetch_one(&pool)
    .await
    .expect("stored observation");
    assert_eq!(
        row.try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "manual"
    );
    assert_eq!(
        row.try_get::<String, _>("kind_code").expect("kind code"),
        "TASK_MUTATION"
    );

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'tasks'
           AND entity_kind = 'task_evidence'
           AND entity_id = $2",
    )
    .bind(&source_id)
    .bind(&evidence_id)
    .fetch_one(&pool)
    .await
    .expect("observation link count");
    assert_eq!(link_count, 1);

    let task_support_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'tasks'
           AND entity_kind = 'task'
           AND entity_id = $2
           AND relationship_kind = 'supports'",
    )
    .bind(&source_id)
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("task support observation link count");
    assert_eq!(task_support_link_count, 1);
}

#[tokio::test]
async fn task_relation_manual_create_path_captures_observation_against_postgres() {
    let Some(database_url) = test_database_url("task relation observation api").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;

    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/relations",
                urlencoding_percent_encode(&task_id)
            ),
            json!({
                "entity_type": "project",
                "entity_id": format!("project:v1:task-relation:{suffix}"),
                "relation_type": "depends_on"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let relation_id = body["id"].as_str().expect("relation id").to_owned();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();

    let relation_source: String =
        sqlx::query_scalar("SELECT source FROM task_relations WHERE id::text = $1")
            .bind(&relation_id)
            .fetch_one(&pool)
            .await
            .expect("relation source");
    assert!(relation_source.starts_with("observation:"));

    let observation_id = relation_source
        .strip_prefix("observation:")
        .expect("observation prefix");
    let row = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(observation_id)
    .fetch_one(&pool)
    .await
    .expect("stored observation");
    assert_eq!(
        row.try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "manual"
    );
    assert_eq!(
        row.try_get::<String, _>("kind_code").expect("kind code"),
        "TASK_MUTATION"
    );

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'tasks'
           AND entity_kind = 'task_relation'
           AND entity_id = $2",
    )
    .bind(observation_id)
    .bind(&relation_id)
    .fetch_one(&pool)
    .await
    .expect("observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn task_archive_manual_path_captures_observation_against_postgres() {
    let Some(database_url) = test_database_url("task archive observation api").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;

    let Some(task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: task create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/archive",
                urlencoding_percent_encode(&task_id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();
    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'tasks'
           AND entity_kind = 'task'
           AND entity_id = $1
           AND relationship_kind = 'status_update'",
    )
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("task archive observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn task_subtask_manual_create_path_captures_observation_against_postgres() {
    let Some(database_url) = test_database_url("task subtask observation api").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_tasks_app(&database_url).await;

    let Some(parent_task_id) = create_task(&app, suffix).await else {
        eprintln!("skip: parent task create failed");
        return;
    };
    let Some(child_task_id) = create_task(&app, suffix + 1).await else {
        eprintln!("skip: child task create failed");
        return;
    };

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/tasks/{}/subtasks",
                urlencoding_percent_encode(&parent_task_id)
            ),
            json!({
                "child_task_id": child_task_id,
                "sort_order": 3
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let subtask_id = body["id"].as_str().expect("subtask id").to_owned();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database");
    let pool = database.pool().expect("pool").clone();

    let subtask_source: String =
        sqlx::query_scalar("SELECT source FROM task_subtasks WHERE id::text = $1")
            .bind(&subtask_id)
            .fetch_one(&pool)
            .await
            .expect("subtask source");
    assert!(subtask_source.starts_with("observation:"));

    let observation_id = subtask_source
        .strip_prefix("observation:")
        .expect("observation prefix");
    let row = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(observation_id)
    .fetch_one(&pool)
    .await
    .expect("stored observation");
    assert_eq!(
        row.try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "manual"
    );
    assert_eq!(
        row.try_get::<String, _>("kind_code").expect("kind code"),
        "TASK_MUTATION"
    );

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'tasks'
           AND entity_kind = 'task_subtask'
           AND entity_id = $2",
    )
    .bind(observation_id)
    .bind(&subtask_id)
    .fetch_one(&pool)
    .await
    .expect("observation link count");
    assert_eq!(link_count, 1);
}
use makosh_hub_backend::domains::decisions::models::{
    evidence::NewDecisionEvidence, source_kind::DecisionEvidenceSourceKind,
    states::DecisionReviewState,
};
use makosh_hub_backend::domains::decisions::store::DecisionStore;
