use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Duration, Utc};
use makosh_hub_backend::application::task_relationship::TaskRelationshipApplicationService;
use makosh_hub_backend::domains::decisions::models::{
    decision::NewDecision, evidence::NewDecisionEvidence, source_kind::DecisionEvidenceSourceKind,
    states::DecisionReviewState,
};
use makosh_hub_backend::domains::obligations::models::{
    entity_kind::ObligationEntityKind, evidence::NewObligationEvidence, obligation::NewObligation,
    source_kind::ObligationEvidenceSourceKind, states::ObligationReviewState,
};
use makosh_hub_backend::domains::relationships::{
    models::{RelationshipEntityKind, RelationshipEvidenceSourceKind, RelationshipReviewState},
    store::RelationshipStore,
};
use makosh_hub_backend::domains::review::{
    models::{NewReviewItem, NewReviewItemEvidence, ReviewItemKind},
    store::ReviewInboxStore,
};
use makosh_hub_backend::domains::tasks::api::{NewTask, TaskListQuery, TaskStore, TaskUpdate};
use makosh_hub_backend::domains::tasks::brain::TaskBrainService;
use makosh_hub_backend::domains::tasks::core::{
    checklists::TaskChecklistStore, context_packs::TaskContextPackStore,
    evidence::TaskEvidenceStore, provider_store::TaskProviderStore, relations::TaskRelationStore,
    subtasks::TaskSubtaskStore,
};
use makosh_hub_backend::domains::tasks::health::TaskWatchtowerService;
use makosh_hub_backend::domains::tasks::intelligence::TaskIntelligenceService;
use makosh_hub_backend::domains::tasks::rules::{TaskRuleStore, TaskTemplateStore};
use makosh_hub_backend::platform::storage::database::Database;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}

async fn live_pool() -> Option<PgPool> {
    let test_context = TestContext::new().await;
    let url = test_context.connection_string();
    let db = Database::connect(Some(&url)).await.expect("connect");
    Box::leak(Box::new(test_context));
    Some(db.pool().expect("pool").clone())
}

fn disconnected_pool() -> PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgres://x:x@127.0.0.1:1/db")
        .expect("lazy")
}

fn assert_float_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.0001,
        "expected {expected}, got {actual}"
    );
}

// ── Task CRUD ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_crud_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("Test {suffix}"),
            description: Some("desc".into()),
            source_type: Some("manual".into()),
            makosh_status: Some("new".into()),
            priority_score: Some(0.8),
            ..Default::default()
        })
        .await
        .expect("create");
    assert!(task.task_id.starts_with("task:v1:"));
    assert_eq!(task.makosh_status, "new");
    assert_eq!(task.provenance_kind, "observation");
    assert!(task.provenance_id.starts_with("observation:v1:"));
    assert_eq!(task.source_kind, "observation");
    assert_eq!(task.source_type, "observation");
    assert_eq!(task.source_id, task.provenance_id);

    let fetched = store
        .get(&task.task_id)
        .await
        .expect("get")
        .expect("exists");
    assert_float_eq(fetched.priority_score.expect("priority score"), 0.8);

    let updated = store
        .update(
            &task.task_id,
            &TaskUpdate {
                makosh_status: Some("in_progress".into()),
                priority_score: Some(0.9),
                ..Default::default()
            },
        )
        .await
        .expect("update");
    assert_eq!(updated.makosh_status, "in_progress");
    assert_float_eq(updated.priority_score.expect("updated priority score"), 0.9);

    store
        .set_status(&task.task_id, "done")
        .await
        .expect("set status");
    let done = store
        .get(&task.task_id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(done.makosh_status, "done");
    assert!(done.completed_at.is_some());

    store.archive(&task.task_id).await.expect("archive");
    let archived = store
        .get(&task.task_id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(archived.makosh_status, "archived");
}

#[tokio::test]
async fn task_manual_creation_materializes_explicit_observation_provenance_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool.clone());
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("Manual provenance {suffix}"),
            description: Some("Task created directly from UI".to_owned()),
            ..Default::default()
        })
        .await
        .expect("create manual task");

    assert_eq!(task.provenance_kind, "observation");
    assert_eq!(task.source_kind, "observation");
    assert_eq!(task.source_type, "observation");
    assert_eq!(task.source_id, task.provenance_id);

    let row = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, observation.payload
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&task.provenance_id)
    .fetch_one(&pool)
    .await
    .expect("task provenance observation");
    let kind_code: String = row.try_get("kind_code").expect("kind code");
    let payload: serde_json::Value = row.try_get("payload").expect("payload");

    assert_eq!(kind_code, "TASK_MUTATION");
    assert_eq!(payload["task_title"], json!(task.title));
    assert_eq!(payload["captured_from"], json!("task_create"));
}

#[tokio::test]
async fn task_store_update_with_observation_materializes_task_link_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let observation_store = ObservationStore::new(pool.clone());
    let store = TaskStore::new(pool.clone());
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("Task update source {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");
    let observation = observation_store
        .capture(
            &NewObservation::new(
                "TASK_MUTATION",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "task_id": task.task_id,
                    "title": format!("Task update applied {suffix}"),
                }),
                format!("manual://tasks/update/{suffix}"),
            )
            .confidence(0.9),
        )
        .await
        .expect("capture observation");

    let updated = store
        .update_with_observation(
            &task.task_id,
            &TaskUpdate {
                title: Some(format!("Task update applied {suffix}")),
                ..Default::default()
            },
            &observation.observation_id,
            "task_update",
            json!({
                "operation": "update",
            }),
        )
        .await
        .expect("update");
    assert_eq!(updated.title, format!("Task update applied {suffix}"));

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'tasks'
           AND entity_kind = 'task'
           AND entity_id = $2
           AND relationship_kind = 'task_update'",
    )
    .bind(&observation.observation_id)
    .bind(&task.task_id)
    .fetch_one(&pool)
    .await
    .expect("observation link count");
    assert_eq!(link_count, 1);
}

#[tokio::test]
async fn task_creation_rejects_missing_review_item_provenance_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();

    let result = store
        .create(&NewTask {
            title: format!("Orphan review task {suffix}"),
            provenance_kind: Some("review_item".to_owned()),
            provenance_id: Some("review_item:v1:missing".to_owned()),
            source_type: Some("manual".to_owned()),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_err(),
        "task must not be created without existing review item"
    );
}

#[tokio::test]
async fn task_creation_rejects_missing_observation_provenance_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();

    let result = store
        .create(&NewTask {
            title: format!("Orphan observation task {suffix}"),
            provenance_kind: Some("observation".to_owned()),
            provenance_id: Some("observation:v1:missing".to_owned()),
            source_type: Some("manual".to_owned()),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_err(),
        "task must not be created without existing observation"
    );
}

#[tokio::test]
async fn task_creation_from_explicit_observation_provenance_uses_observation_source_against_postgres()
 {
    let Some(pool) = live_pool().await else {
        return;
    };
    let observation_store = ObservationStore::new(pool.clone());
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();
    let observation = observation_store
        .capture(
            &NewObservation::new(
                "DOCUMENT",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "title": format!("Explicit observation source {suffix}"),
                    "body": "Task should inherit observation-backed source fields."
                }),
                format!("manual://task-provenance/{suffix}"),
            )
            .confidence(0.9),
        )
        .await
        .expect("capture observation");

    let task = store
        .create(&NewTask {
            title: format!("Observation provenance task {suffix}"),
            provenance_kind: Some("observation".to_owned()),
            provenance_id: Some(observation.observation_id.clone()),
            ..Default::default()
        })
        .await
        .expect("create task");

    assert_eq!(task.provenance_kind, "observation");
    assert_eq!(task.provenance_id, observation.observation_id);
    assert_eq!(task.source_kind, "observation");
    assert_eq!(task.source_type, "observation");
    assert_eq!(task.source_id, task.provenance_id);
}

#[tokio::test]
async fn task_creation_rejects_missing_decision_provenance_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();

    let result = store
        .create(&NewTask {
            title: format!("Orphan decision task {suffix}"),
            provenance_kind: Some("decision".to_owned()),
            provenance_id: Some("decision:v1:missing".to_owned()),
            source_type: Some("manual".to_owned()),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_err(),
        "task must not be created without existing decision"
    );
}

#[tokio::test]
async fn task_creation_rejects_missing_obligation_provenance_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();

    let result = store
        .create(&NewTask {
            title: format!("Orphan obligation task {suffix}"),
            provenance_kind: Some("obligation".to_owned()),
            provenance_id: Some("obligation:v1:missing".to_owned()),
            source_type: Some("manual".to_owned()),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_err(),
        "task must not be created without existing obligation"
    );
}

#[tokio::test]
async fn task_creation_rejects_review_item_without_observation_evidence_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let review_store = ReviewInboxStore::new(pool.clone());
    let observation_store = ObservationStore::new(pool.clone());
    let store = TaskStore::new(pool.clone());
    let suffix = unique_suffix();
    let observation = observation_store
        .capture(&NewObservation::new(
            "DOCUMENT",
            ObservationOriginKind::Manual,
            Utc::now(),
            json!({
                "title": format!("Review evidence {suffix}"),
                "body": "Temporary evidence that will be detached."
            }),
            format!("manual://review-item/{suffix}"),
        ))
        .await
        .expect("capture review observation");
    let review_item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialTask,
                format!("Detached review evidence {suffix}"),
                "Review item exists but no longer has evidence.".to_owned(),
                0.84,
            ),
            &[NewReviewItemEvidence::new(
                observation.observation_id.clone(),
            )],
        )
        .await
        .expect("create review item");
    sqlx::query("DELETE FROM review_item_evidence WHERE review_item_id = $1")
        .bind(&review_item.review_item_id)
        .execute(&pool)
        .await
        .expect("detach review evidence");

    let result = store
        .create(&NewTask {
            title: format!("Review evidence gap {suffix}"),
            provenance_kind: Some("review_item".to_owned()),
            provenance_id: Some(review_item.review_item_id),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_err(),
        "task must not be created from review item without observation evidence"
    );
}

#[tokio::test]
async fn task_creation_rejects_decision_without_observation_evidence_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let decision_store = DecisionStore::new(pool.clone());
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();
    let decision = decision_store
        .upsert_with_evidence(
            &NewDecision::new(
                format!("Decision without observation evidence {suffix}"),
                "This decision was sourced from an event, not an observation.",
                0.71,
                DecisionReviewState::Suggested,
            ),
            &[NewDecisionEvidence::new(
                DecisionEvidenceSourceKind::Event,
                format!("event:task-provenance:{suffix}"),
            )
            .quote("Decision event without canonical observation evidence.")],
            &[],
        )
        .await
        .expect("create decision");

    let result = store
        .create(&NewTask {
            title: format!("Decision evidence gap {suffix}"),
            provenance_kind: Some("decision".to_owned()),
            provenance_id: Some(decision.decision_id),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_err(),
        "task must not be created from decision without observation evidence"
    );
}

#[tokio::test]
async fn task_creation_rejects_obligation_without_observation_evidence_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let obligation_store = ObligationStore::new(pool.clone());
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();
    let obligation = obligation_store
        .upsert_with_evidence(
            &NewObligation::new(
                ObligationEntityKind::Persona,
                format!("persona:task-gap:{suffix}"),
                format!("Deliver obligation gap proof {suffix}"),
                0.73,
                ObligationReviewState::Suggested,
            ),
            &[NewObligationEvidence::new(
                ObligationEvidenceSourceKind::Communication,
                format!("message:task-provenance:{suffix}"),
            )
            .quote("Obligation evidence exists without canonical observation.")],
        )
        .await
        .expect("create obligation");

    let result = store
        .create(&NewTask {
            title: format!("Obligation evidence gap {suffix}"),
            provenance_kind: Some("obligation".to_owned()),
            provenance_id: Some(obligation.obligation_id),
            ..Default::default()
        })
        .await;

    assert!(
        result.is_err(),
        "task must not be created from obligation without observation evidence"
    );
}

#[tokio::test]
async fn task_creation_bypass_via_direct_insert_violates_provenance_guard_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let task_id = format!("task:v1:direct-guard-check-{}", unique_suffix());
    let result = sqlx::query(
        "INSERT INTO tasks (task_id, title, provenance_kind, provenance_id, source_kind, source_id, source_type, makosh_status, created_from_event_id, created_by_actor_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
    )
    .bind(&task_id)
    .bind("Direct insert provenance guard check")
    .bind("observation")
    .bind("observation:v1:missing")
    .bind("manual")
    .bind("manual")
    .bind("manual")
    .bind("new")
    .bind::<Option<String>>(None)
    .bind::<Option<String>>(None)
    .execute(&pool)
    .await;

    assert!(
        result.is_err(),
        "db trigger must block provenance-less direct insert"
    );
}

// ── Task List ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_list_filtering_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool);
    let suffix = unique_suffix();
    store
        .create(&NewTask {
            title: format!("Active {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");
    store
        .create(&NewTask {
            title: format!("Done {suffix}"),
            source_type: Some("manual".into()),
            makosh_status: Some("done".into()),
            ..Default::default()
        })
        .await
        .expect("create");

    let all = store
        .list(&TaskListQuery {
            limit: Some(100),
            ..Default::default()
        })
        .await
        .expect("list");
    assert!(all.len() >= 2);

    let active = store
        .list(&TaskListQuery {
            status: Some("new".into()),
            limit: Some(100),
            ..Default::default()
        })
        .await
        .expect("list");
    assert!(
        active
            .iter()
            .any(|t| t.title.contains(&format!("Active {suffix}")))
    );
}

// ── Context Pack ──────────────────────────────────────────────────────────

#[tokio::test]
async fn task_context_pack_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool.clone());
    let ctx = TaskContextPackStore::new(pool);
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("Ctx {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");

    let pack = ctx
        .upsert(
            &task.task_id,
            Some("summary"),
            json!(["Q1"]),
            json!(["blocker"]),
            json!(["risk"]),
            Some("next step"),
        )
        .await
        .expect("upsert");
    assert_eq!(pack.summary.as_deref(), Some("summary"));

    let fetched = ctx.get(&task.task_id).await.expect("get").expect("exists");
    assert_eq!(fetched.summary.as_deref(), Some("summary"));
}

// ── Evidence ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_evidence_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool.clone());
    let ev = TaskEvidenceStore::new(pool);
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("Ev {suffix}"),
            source_type: Some("email".into()),
            ..Default::default()
        })
        .await
        .expect("create");

    let evidence = ev
        .add(
            &task.task_id,
            "email",
            "msg-1",
            Some("Please do this"),
            Some(0.9),
        )
        .await
        .expect("add");
    assert_eq!(evidence.source_type, "email");
    assert_float_eq(evidence.confidence, 0.9);

    let list = ev.list(&task.task_id).await.expect("list");
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn task_evidence_materializes_support_links_for_observation_source_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool.clone());
    let ev = TaskEvidenceStore::new(pool.clone());
    let observation = ObservationStore::new(pool.clone())
        .capture(
            &NewObservation::new(
                "TASK_MUTATION",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "quote": "Direct evidence for a task."
                }),
                "manual://tasks/evidence-support",
            )
            .confidence(0.87),
        )
        .await
        .expect("capture observation");
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("EvObs {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");

    let evidence = ev
        .add(
            &task.task_id,
            "observation",
            &observation.observation_id,
            Some("Please do this"),
            Some(0.91),
        )
        .await
        .expect("add observation evidence");

    let evidence_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'tasks'
          AND entity_kind = 'task_evidence'
          AND entity_id = $2
        "#,
    )
    .bind(&observation.observation_id)
    .bind(&evidence.id)
    .fetch_one(&pool)
    .await
    .expect("task evidence link count");
    assert_eq!(evidence_link_count, 1);

    let task_support_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'tasks'
          AND entity_kind = 'task'
          AND entity_id = $2
          AND relationship_kind = 'supports'
        "#,
    )
    .bind(&observation.observation_id)
    .bind(&task.task_id)
    .fetch_one(&pool)
    .await
    .expect("task support link count");
    assert_eq!(task_support_link_count, 1);
}

// ── Relations ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_relations_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool.clone());
    let rel = TaskRelationStore::new(pool);
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("Rel {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");

    rel.link(&task.task_id, "person", "p1", "blocks", "manual")
        .await
        .expect("link");
    let list = rel.list(&task.task_id).await.expect("list");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].relation_type, "blocks");
}

#[tokio::test]
async fn task_relation_materializes_first_class_relationship_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool.clone());
    let relationship_store = RelationshipStore::new(pool.clone());
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("Relationship task {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");
    let project_id = format!("project:v1:task-relation:{suffix}");

    let relation = TaskRelationshipApplicationService::new(pool.clone())
        .link(
            &task.task_id,
            "project",
            &project_id,
            "depends_on",
            "manual",
        )
        .await
        .expect("link");

    let relationships = relationship_store
        .list_for_entity(RelationshipEntityKind::Task, &task.task_id, 20)
        .await
        .expect("task relationships");
    let relationship = relationships
        .iter()
        .find(|item| {
            item.source_entity_kind == RelationshipEntityKind::Task
                && item.source_entity_id == task.task_id
                && item.target_entity_kind == RelationshipEntityKind::Project
                && item.target_entity_id == project_id
                && item.relationship_type == "depends_on"
        })
        .expect("task relation should create first-class Relationship");

    assert_eq!(
        relationship.review_state,
        RelationshipReviewState::UserConfirmed
    );
    assert_eq!(relationship.confidence, relation.confidence);
    assert_eq!(
        relationship.metadata["compatibility_table"],
        json!("task_relations")
    );
    assert_eq!(
        relationship.metadata["compatibility_record_id"],
        json!(relation.id)
    );
    assert_eq!(relationship.metadata["source"], json!("manual"));

    let evidence = sqlx::query(
        r#"
        SELECT source_kind, source_id, excerpt, metadata
        FROM relationship_evidence
        WHERE relationship_id = $1
        "#,
    )
    .bind(&relationship.relationship_id)
    .fetch_one(&pool)
    .await
    .expect("relationship evidence");
    let source_kind: String = evidence.try_get("source_kind").expect("source kind");
    let source_id: String = evidence.try_get("source_id").expect("source id");
    let excerpt: Option<String> = evidence.try_get("excerpt").expect("excerpt");
    let metadata: serde_json::Value = evidence.try_get("metadata").expect("metadata");

    assert_eq!(
        source_kind,
        RelationshipEvidenceSourceKind::Observation.as_str()
    );
    assert!(!source_id.is_empty());
    assert_eq!(
        excerpt.as_deref(),
        Some("Task relation was recorded through compatibility task relation data.")
    );
    assert_eq!(metadata["task_id"], json!(task.task_id));
    assert_eq!(metadata["entity_type"], json!("project"));
    assert_eq!(metadata["entity_id"], json!(project_id));
}

#[tokio::test]
async fn task_relation_store_materializes_observation_link_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool.clone());
    let observation = ObservationStore::new(pool.clone())
        .capture(
            &NewObservation::new(
                "TASK_MUTATION",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "entity_type": "project",
                    "relation_type": "depends_on",
                }),
                "manual://tasks/relation-observation",
            )
            .confidence(0.8),
        )
        .await
        .expect("capture observation");
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("RelationObs {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");
    let project_id = format!("project:v1:task-relation-observation:{suffix}");

    let relation = TaskRelationshipApplicationService::new(pool.clone())
        .link(
            &task.task_id,
            "project",
            &project_id,
            "depends_on",
            &format!("observation:{}", observation.observation_id),
        )
        .await
        .expect("link");

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'tasks'
           AND entity_kind = 'task_relation'
           AND entity_id = $2",
    )
    .bind(&observation.observation_id)
    .bind(&relation.id)
    .fetch_one(&pool)
    .await
    .expect("observation link count");
    assert_eq!(link_count, 1);
}

// ── Checklist ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_checklist_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool.clone());
    let cl = TaskChecklistStore::new(pool);
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("Cl {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");

    cl.set(
        &task.task_id,
        json!([{"text":"Step 1","done":false}]),
        "manual",
    )
    .await
    .expect("set");
    let fetched = cl.get(&task.task_id).await.expect("get").expect("exists");
    assert_eq!(fetched.source, "manual");
}

#[tokio::test]
async fn task_checklist_store_materializes_observation_link_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool.clone());
    let cl = TaskChecklistStore::new(pool.clone());
    let observation = ObservationStore::new(pool.clone())
        .capture(
            &NewObservation::new(
                "TASK_MUTATION",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "items": [{"text":"Step 1","done":false}]
                }),
                "manual://tasks/checklist-observation",
            )
            .confidence(0.84),
        )
        .await
        .expect("capture observation");
    let suffix = unique_suffix();
    let task = store
        .create(&NewTask {
            title: format!("ClObs {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");

    let checklist = cl
        .set(
            &task.task_id,
            json!([{"text":"Step 1","done":false}]),
            &format!("observation:{}", observation.observation_id),
        )
        .await
        .expect("set");

    let link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'tasks'
          AND entity_kind = 'task_checklist'
          AND entity_id = $2
        "#,
    )
    .bind(&observation.observation_id)
    .bind(&checklist.id)
    .fetch_one(&pool)
    .await
    .expect("task checklist link count");
    assert_eq!(link_count, 1);
}

// ── Subtasks ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_subtasks_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool.clone());
    let sub = TaskSubtaskStore::new(pool);
    let suffix = unique_suffix();
    let parent = store
        .create(&NewTask {
            title: format!("Parent {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");
    let child = store
        .create(&NewTask {
            title: format!("Child {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");

    sub.add(&parent.task_id, &child.task_id, 0)
        .await
        .expect("add");
    let list = sub.list(&parent.task_id).await.expect("list");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].child_task_id, child.task_id);
    assert_eq!(list[0].source, "manual");
}

#[tokio::test]
async fn task_subtask_store_materializes_observation_link_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let store = TaskStore::new(pool.clone());
    let sub = TaskSubtaskStore::new(pool.clone());
    let observation = ObservationStore::new(pool.clone())
        .capture(
            &NewObservation::new(
                "TASK_MUTATION",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "sort_order": 0
                }),
                "manual://tasks/subtask-observation",
            )
            .confidence(0.82),
        )
        .await
        .expect("capture observation");
    let suffix = unique_suffix();
    let parent = store
        .create(&NewTask {
            title: format!("ParentObs {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");
    let child = store
        .create(&NewTask {
            title: format!("ChildObs {suffix}"),
            source_type: Some("manual".into()),
            ..Default::default()
        })
        .await
        .expect("create");

    let subtask = sub
        .add_with_source(
            &parent.task_id,
            &child.task_id,
            0,
            &format!("observation:{}", observation.observation_id),
        )
        .await
        .expect("add");

    let link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'tasks'
          AND entity_kind = 'task_subtask'
          AND entity_id = $2
        "#,
    )
    .bind(&observation.observation_id)
    .bind(&subtask.id)
    .fetch_one(&pool)
    .await
    .expect("task subtask link count");
    assert_eq!(link_count, 1);
}

// ── Providers ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_providers_against_postgres() {
    let Some(pool) = live_pool().await else {
        eprintln!("skip: no DB");
        return;
    };
    let store = TaskProviderStore::new(pool.clone());
    let suffix = unique_suffix();
    let provider = store
        .create("jira", &format!("Jira {suffix}"))
        .await
        .expect("create");
    let list = store.list().await.expect("list");
    assert!(list.iter().any(|p| p.provider == "jira"));
    let observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'tasks'
           AND entity_kind = 'task_provider_account'
           AND entity_id = $1
           AND relationship_kind = 'create'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&provider.account_id)
    .fetch_one(&pool)
    .await
    .expect("task provider observation link");
    let row = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("task provider observation");
    assert_eq!(
        row.try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "local_runtime"
    );
    assert_eq!(
        row.try_get::<String, _>("kind_code").expect("kind code"),
        "TASK_PROVIDER_ACCOUNT"
    );
}

// ── Rules and Templates ───────────────────────────────────────────────────

#[tokio::test]
async fn task_rules_and_templates_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let rules = TaskRuleStore::new(pool.clone());
    let tmpl = TaskTemplateStore::new(pool);
    let suffix = unique_suffix();

    let rule = rules
        .create(
            &format!("Rule {suffix}"),
            None,
            json!({"action":"auto_prioritize"}),
            Some("suggest_only"),
        )
        .await
        .expect("create");
    assert!(rule.rule_id.starts_with("taskrule:v1:"));

    rules.delete(&rule.rule_id).await.expect("delete");

    let templates = tmpl.list().await.expect("list");
    assert!(templates.iter().any(|t| t.template_id == "bug"));
}

// ── Intelligence ──────────────────────────────────────────────────────────

#[test]
fn task_intelligence_priority() {
    let now = Utc::now();
    let high = TaskIntelligenceService::calculate_priority(
        Some(now + Duration::hours(2)),
        true,
        true,
        true,
        true,
        false,
        false,
    );
    let low = TaskIntelligenceService::calculate_priority(
        Some(now + Duration::days(30)),
        false,
        false,
        false,
        false,
        false,
        false,
    );
    assert!(high > low);
    assert!(high > 0.5);
}

#[test]
fn task_intelligence_risk() {
    let high = TaskIntelligenceService::calculate_risk(true, true, true, true, true, "urgent fix");
    let low =
        TaskIntelligenceService::calculate_risk(false, false, false, false, false, "update docs");
    assert!(high > low);
    assert!(high > 0.5);
}

#[test]
fn task_intelligence_readiness() {
    let full = TaskIntelligenceService::calculate_readiness(true, true, true, true, true, true);
    assert!((full - 1.0).abs() < 0.01);
    let none =
        TaskIntelligenceService::calculate_readiness(false, false, false, false, false, false);
    assert!((none - 0.0).abs() < 0.01);
}

#[test]
fn task_intelligence_next_action() {
    assert!(
        TaskIntelligenceService::suggest_next_action("new", false, false, None).contains("Review")
    );
    assert!(
        TaskIntelligenceService::suggest_next_action("waiting", false, false, Some("John"))
            .contains("Follow")
    );
    assert!(
        TaskIntelligenceService::suggest_next_action("done", false, false, None)
            .contains("Archive")
    );
}

// ── Health ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_health_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let overdue = TaskWatchtowerService::overdue(&pool)
        .await
        .expect("overdue");
    assert!(overdue.is_object());

    let stale = TaskWatchtowerService::stale_tasks(&pool, 14)
        .await
        .expect("stale");
    assert!(stale.is_object());

    let no_ctx = TaskWatchtowerService::without_context(&pool)
        .await
        .expect("no ctx");
    assert!(no_ctx.is_object());

    let wl = TaskWatchtowerService::workload(&pool)
        .await
        .expect("workload");
    assert!(wl.is_object());
}

// ── Brain ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_brain_against_postgres() {
    let Some(pool) = live_pool().await else {
        return;
    };
    let brief = TaskBrainService::daily_brief(&pool).await.expect("brief");
    assert!(brief.is_object());

    let search = TaskBrainService::search_tasks(&pool, "test")
        .await
        .expect("search");
    assert!(search.is_object());
}

// ── Sync Export ───────────────────────────────────────────────────────────

#[test]
fn task_export_markdown() {
    let md = makosh_hub_backend::domains::tasks::sync::export_task_md(
        "Test",
        Some("desc"),
        "in_progress",
        Some("because"),
        Some("done"),
    );
    assert!(md.contains("# Test"));
    assert!(md.contains("in_progress"));
    assert!(md.contains("because"));
}

#[test]
fn task_export_json() {
    let json = makosh_hub_backend::domains::tasks::sync::export_task_json(
        "Test",
        Some("desc"),
        "done",
        Some(0.8),
        Some("2026-01-01"),
    );
    assert_eq!(json["title"], "Test");
    assert_eq!(json["priority"], 0.8);
}

// ── Disconnected pool smoke ───────────────────────────────────────────────

#[tokio::test]
async fn disconnected_task_stores() {
    let pool = disconnected_pool();
    let _ = TaskStore::new(pool.clone());
    let _ = TaskContextPackStore::new(pool.clone());
    let _ = TaskEvidenceStore::new(pool.clone());
    let _ = TaskRelationStore::new(pool.clone());
    let _ = TaskChecklistStore::new(pool.clone());
    let _ = TaskSubtaskStore::new(pool);
}
use makosh_hub_backend::domains::decisions::store::DecisionStore;
use makosh_hub_backend::domains::obligations::store::ObligationStore;
