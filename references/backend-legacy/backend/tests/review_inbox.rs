use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use chrono::{TimeZone, Utc};
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::decisions::store::DecisionStore;
use makosh_hub_backend::domains::documents::core::{
    models::NewDocumentImport, store::DocumentImportStore,
};
use makosh_hub_backend::domains::obligations::store::ObligationStore;
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::domains::personas::identity::store::PersonaIdentityReviewStore;
use makosh_hub_backend::domains::projects::core::store::ProjectStore;
use makosh_hub_backend::domains::relationships::{
    models::{
        NewRelationship, NewRelationshipEvidence, RelationshipEntityKind, RelationshipReviewState,
    },
    store::RelationshipStore,
};
use makosh_hub_backend::domains::review::{
    models::{
        NewReviewItem, NewReviewItemEvidence, ReviewItemKind, ReviewItemStatus,
        ReviewPromotionTarget,
    },
    store::ReviewInboxStore,
};
use makosh_hub_backend::domains::tasks::api::TaskStore;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::review_inbox::project_persona_identity_review_event;
use makosh_hub_backend::workflows::review_inbox::sync_decisions_to_review_for_observations;
use makosh_hub_backend::workflows::review_inbox::sync_obligations_to_review_for_observations;
use makosh_hub_backend::workflows::review_inbox::sync_relationships_to_review_for_observations;
use makosh_hub_backend::workflows::review_inbox::sync_task_candidates_to_review_for_observations;
use makosh_hub_backend::workflows::review_promotion::ReviewPromotionService;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

#[tokio::test]
async fn review_inbox_creates_evidence_backed_item_against_postgres() {
    let Some((pool, observation_store, review_store)) =
        live_review_context("evidence-backed review item").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observation_id = seed_manual_note(&observation_store, suffix).await;

    let item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialTask,
                "Send the Friday report",
                "Email evidence suggests a possible deadline-backed task.",
                0.84,
            )
            .metadata(json!({"detector": "contract-test"})),
            &[NewReviewItemEvidence::new(observation_id.clone()).role("primary")],
        )
        .await
        .expect("create review item");

    assert_eq!(item.item_kind, ReviewItemKind::PotentialTask);
    assert_eq!(item.status, ReviewItemStatus::New);
    assert_eq!(item.target_domain, None);
    assert_eq!(item.confidence, 0.84);

    let evidence_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM review_item_evidence
        WHERE review_item_id = $1
          AND observation_id = $2
        "#,
    )
    .bind(&item.review_item_id)
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("review evidence count");

    assert_eq!(evidence_count, 1);

    let open = review_store
        .list_by_status(ReviewItemStatus::New, 25)
        .await
        .expect("list new review items");
    assert!(
        open.iter()
            .any(|candidate| candidate.review_item_id == item.review_item_id)
    );

    let detected_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM event_log
        WHERE event_type = 'task.candidate.detected.v1'
          AND subject ->> 'review_item_id' = $1
        "#,
    )
    .bind(&item.review_item_id)
    .fetch_one(&pool)
    .await
    .expect("candidate detected event count");
    assert_eq!(detected_count, 1);

    let available_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM event_log
        WHERE event_type = 'review.item.available.v1'
          AND subject ->> 'review_item_id' = $1
        "#,
    )
    .bind(&item.review_item_id)
    .fetch_one(&pool)
    .await
    .expect("review available event count");
    assert_eq!(available_count, 1);
}

#[tokio::test]
async fn review_inbox_filters_active_and_all_lists_against_postgres() {
    let Some((_, observation_store, review_store)) =
        live_review_context("review list filters").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observation_id = seed_manual_note(&observation_store, suffix).await;

    let new_item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialTask,
                "Prepare migration notes",
                "New task candidate to review.",
                0.75,
            ),
            &[NewReviewItemEvidence::new(observation_id.clone())],
        )
        .await
        .expect("create new review item");

    let reviewed_item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialDecision,
                "Archive obsolete item",
                "Decision candidate to process.",
                0.76,
            ),
            &[NewReviewItemEvidence::new(observation_id.clone())],
        )
        .await
        .expect("create review item to move into review");

    let dismissed_item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialProject,
                "Retire old note",
                "Project candidate should be dismissed.",
                0.5,
            ),
            &[NewReviewItemEvidence::new(observation_id)],
        )
        .await
        .expect("create dismissible review item");
    let _dismissed_item = review_store
        .set_status(&dismissed_item.review_item_id, ReviewItemStatus::Dismissed)
        .await
        .expect("dismiss review item");

    let in_review_item = review_store
        .set_status(&reviewed_item.review_item_id, ReviewItemStatus::InReview)
        .await
        .expect("move item into review");

    let active = review_store
        .list_open(100)
        .await
        .expect("list open review items");
    let active_ids: Vec<&str> = active
        .iter()
        .map(|item| item.review_item_id.as_str())
        .collect();

    assert!(active_ids.contains(&new_item.review_item_id.as_str()));
    assert!(active_ids.contains(&in_review_item.review_item_id.as_str()));
    assert!(!active_ids.contains(&dismissed_item.review_item_id.as_str()));

    let all = review_store
        .list_all(100)
        .await
        .expect("list all review items");
    let all_ids: Vec<&str> = all
        .iter()
        .map(|item| item.review_item_id.as_str())
        .collect();

    assert!(all_ids.contains(&new_item.review_item_id.as_str()));
    assert!(all_ids.contains(&in_review_item.review_item_id.as_str()));
    assert!(all_ids.contains(&dismissed_item.review_item_id.as_str()));
}

#[tokio::test]
async fn review_inbox_lifecycle_approves_promotes_dismisses_and_archives_against_postgres() {
    let Some((_pool, observation_store, review_store)) =
        live_review_context("review lifecycle").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observation_id = seed_manual_note(&observation_store, suffix).await;

    let item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialDecision,
                "Buy a NAS",
                "A decision candidate needs explicit review.",
                0.91,
            ),
            &[NewReviewItemEvidence::new(observation_id.clone())],
        )
        .await
        .expect("create decision review item");

    let in_review = review_store
        .set_status(&item.review_item_id, ReviewItemStatus::InReview)
        .await
        .expect("move review item into review");
    assert_eq!(in_review.status, ReviewItemStatus::InReview);

    let approved = review_store
        .set_status(&item.review_item_id, ReviewItemStatus::Approved)
        .await
        .expect("approve review item");
    assert_eq!(approved.status, ReviewItemStatus::Approved);

    let approved_again = review_store
        .set_status(&item.review_item_id, ReviewItemStatus::Approved)
        .await
        .expect("repeat approve review item idempotently");
    assert_eq!(approved_again.status, ReviewItemStatus::Approved);

    let promoted = review_store
        .promote(
            &item.review_item_id,
            ReviewPromotionTarget::new("decisions", "decision", format!("decision:v1:{suffix}")),
        )
        .await
        .expect("promote review item");
    assert_eq!(promoted.status, ReviewItemStatus::Promoted);
    assert_eq!(promoted.target_domain.as_deref(), Some("decisions"));
    assert_eq!(promoted.target_entity_kind.as_deref(), Some("decision"));

    let dismissed_item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::KnowledgeCandidate,
                "Dismiss low-value note",
                "Candidate is not useful enough for promotion.",
                0.51,
            ),
            &[NewReviewItemEvidence::new(observation_id)],
        )
        .await
        .expect("create dismissable review item");
    let dismissed = review_store
        .set_status(&dismissed_item.review_item_id, ReviewItemStatus::Dismissed)
        .await
        .expect("dismiss review item");
    assert_eq!(dismissed.status, ReviewItemStatus::Dismissed);

    let archived = review_store
        .set_status(&dismissed_item.review_item_id, ReviewItemStatus::Archived)
        .await
        .expect("archive review item");
    assert_eq!(archived.status, ReviewItemStatus::Archived);

    for event_type in [
        "decision.candidate.detected.v1",
        "review.item.available.v1",
        "review.item.approved.v1",
        "review.item.promoted.v1",
    ] {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM event_log
            WHERE event_type = $1
              AND subject ->> 'review_item_id' = $2
            "#,
        )
        .bind(event_type)
        .bind(&item.review_item_id)
        .fetch_one(&_pool)
        .await
        .expect("review lifecycle event count");
        assert_eq!(count, 1, "missing lifecycle event {event_type}");
    }

    let dismissed_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM event_log
        WHERE event_type = 'review.item.dismissed.v1'
          AND subject ->> 'review_item_id' = $1
        "#,
    )
    .bind(&dismissed_item.review_item_id)
    .fetch_one(&_pool)
    .await
    .expect("dismissed event count");
    assert_eq!(dismissed_count, 1);
}

#[tokio::test]
async fn review_inbox_promotion_event_captures_trace_ids_against_postgres() {
    let Some((pool, observation_store, review_store)) =
        live_review_context("review promotion trace ids").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let source_observation_id = seed_manual_note(&observation_store, suffix).await;

    let item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialDecision,
                "Trace first",
                "Promotion trace test should persist causation and correlation.",
                0.77,
            ),
            &[NewReviewItemEvidence::new(source_observation_id)],
        )
        .await
        .expect("create review item");

    let transition_observation = observation_store
        .capture(
            &NewObservation::new(
                "REVIEW_TRANSITION",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "review_item_id": item.review_item_id,
                    "operation": "review_item_promote",
                }),
                format!("manual://review-item-promote-trace/{suffix}"),
            )
            .provenance(json!({
                "source": "review_inbox.test",
            })),
        )
        .await
        .expect("capture review transition observation");

    let promoted = review_store
        .promote_with_observation(
            &item.review_item_id,
            ReviewPromotionTarget::new(
                "decisions",
                "decision",
                format!("decision:v1:trace-{suffix}"),
            ),
            Some(&transition_observation.observation_id),
            Some(json!({"source": "review_inbox.test"})),
            Some(&transition_observation.observation_id),
            Some("trace-correlation"),
        )
        .await
        .expect("promote review item with trace ids");

    assert_eq!(promoted.status, ReviewItemStatus::Promoted);

    let event_row = sqlx::query(
        r#"
        SELECT causation_id, correlation_id
        FROM event_log
        WHERE event_type = 'review.item.promoted.v1'
          AND subject ->> 'review_item_id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&item.review_item_id)
    .fetch_one(&pool)
    .await
    .expect("load promoted review event");

    let causation: Option<String> = event_row
        .try_get("causation_id")
        .expect("promoted causation");
    let correlation: Option<String> = event_row
        .try_get("correlation_id")
        .expect("promoted correlation");

    assert_eq!(
        causation.as_deref(),
        Some(transition_observation.observation_id.as_str()),
        "promote event should carry causation",
    );
    assert_eq!(
        correlation.as_deref(),
        Some("trace-correlation"),
        "promote event should carry correlation",
    );
}

#[tokio::test]
async fn review_inbox_status_with_observation_materializes_transition_link_against_postgres() {
    let Some((pool, observation_store, review_store)) =
        live_review_context("review status observation link").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let source_observation_id = seed_manual_note(&observation_store, suffix).await;

    let item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialTask,
                "Status owner path",
                "Review status change should materialize its own review transition link.",
                0.73,
            ),
            &[NewReviewItemEvidence::new(source_observation_id)],
        )
        .await
        .expect("create review item");

    let review_observation = observation_store
        .capture(
            &NewObservation::new(
                "REVIEW_TRANSITION",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "review_item_id": item.review_item_id,
                    "operation": "review_item_status_transition",
                    "status": "approved",
                }),
                format!("manual://review-item-status/{suffix}"),
            )
            .provenance(json!({
                "source": "review_inbox.test",
            })),
        )
        .await
        .expect("capture review transition observation");

    let updated = review_store
        .set_status_with_observation(
            &item.review_item_id,
            ReviewItemStatus::Approved,
            Some(&review_observation.observation_id),
            Some(json!({
                "source": "review_inbox.test",
            })),
        )
        .await
        .expect("approve review item");
    assert_eq!(updated.status, ReviewItemStatus::Approved);

    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'review'
           AND entity_kind = 'review_item'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&item.review_item_id)
    .fetch_one(&pool)
    .await
    .expect("review item observation link");
    let observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: serde_json::Value = link_row.try_get("metadata").expect("metadata");
    assert_eq!(observation_id, review_observation.observation_id);
    assert_eq!(metadata["status"], json!("approved"));
}

#[tokio::test]
async fn review_promotion_service_with_observation_materializes_review_item_transition_link_against_postgres()
 {
    let Some((pool, observation_store, review_store)) =
        live_review_context("review promote observation link").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let source_observation_id = seed_manual_note(&observation_store, suffix).await;

    let item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialProject,
                "Promote project owner path",
                "Promotion should materialize review transition link inside promotion service.",
                0.79,
            ),
            &[NewReviewItemEvidence::new(source_observation_id)],
        )
        .await
        .expect("create review item");

    let review_observation = observation_store
        .capture(
            &NewObservation::new(
                "REVIEW_TRANSITION",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "review_item_id": item.review_item_id,
                    "operation": "review_item_promote",
                    "target_domain": "projects",
                    "target_entity_kind": "project",
                    "target_entity_id": format!("project:v1:promote-link:{suffix}"),
                }),
                format!("manual://review-item-promote/{suffix}"),
            )
            .provenance(json!({
                "source": "review_inbox.test",
            })),
        )
        .await
        .expect("capture promote observation");

    let promoted = ReviewPromotionService::new(pool.clone())
        .promote_with_observation(
            &item.review_item_id,
            ReviewPromotionTarget::new(
                "projects",
                "project",
                format!("project:v1:promote-link:{suffix}"),
            ),
            Some(&review_observation.observation_id),
            Some(json!({
                "source": "review_inbox.test",
            })),
            None,
            None,
        )
        .await
        .expect("promote review item");
    assert_eq!(promoted.status, ReviewItemStatus::Promoted);

    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'review'
           AND entity_kind = 'review_item'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&item.review_item_id)
    .fetch_one(&pool)
    .await
    .expect("review promotion observation link");
    let observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: serde_json::Value = link_row.try_get("metadata").expect("metadata");
    assert_eq!(observation_id, review_observation.observation_id);
    assert_eq!(metadata["status"], json!("promoted"));
}

#[tokio::test]
async fn review_can_materialize_promotions_for_core_target_domains_against_postgres() {
    let Some((pool, observation_store, review_store)) =
        live_review_context("review promotion targets").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observation_id = seed_manual_note(&observation_store, suffix).await;

    let targets = [
        (
            ReviewItemKind::NewPersona,
            "personas",
            "persona",
            "persona:v1:review",
        ),
        (
            ReviewItemKind::NewOrganization,
            "organizations",
            "organization",
            "org:v1:review",
        ),
        (
            ReviewItemKind::PotentialTask,
            "tasks",
            "task",
            "task:v1:review",
        ),
        (
            ReviewItemKind::PotentialObligation,
            "obligations",
            "obligation",
            "obligation:v1:review",
        ),
        (
            ReviewItemKind::PotentialDecision,
            "decisions",
            "decision",
            "decision:v1:review",
        ),
        (
            ReviewItemKind::PotentialRelationship,
            "relationships",
            "relationship",
            "relationship:v1:review",
        ),
        (
            ReviewItemKind::PotentialProject,
            "projects",
            "project",
            "project:v1:review",
        ),
        (
            ReviewItemKind::KnowledgeCandidate,
            "documents",
            "document",
            "document:v1:review",
        ),
    ];

    for (kind, domain, entity_kind, entity_id_prefix) in targets {
        let item = review_store
            .create_with_evidence(
                &NewReviewItem::new(
                    kind,
                    format!("Promote {domain} candidate {suffix}"),
                    "Review inbox owns promotion state and target references.",
                    0.8,
                ),
                &[NewReviewItemEvidence::new(observation_id.clone())],
            )
            .await
            .expect("create promotable item");
        let promoted = ReviewPromotionService::new(pool.clone())
            .promote(
                &item.review_item_id,
                ReviewPromotionTarget::new(
                    domain,
                    entity_kind,
                    format!("{entity_id_prefix}:{suffix}"),
                ),
            )
            .await
            .expect("promote item");

        assert_eq!(promoted.status, ReviewItemStatus::Promoted);
        assert_eq!(promoted.target_domain.as_deref(), Some(domain));
        assert_eq!(promoted.target_entity_kind.as_deref(), Some(entity_kind));

        let target_id = promoted
            .target_entity_id
            .clone()
            .expect("promotion target id");
        assert_materialized_target(
            &pool,
            domain,
            &target_id,
            &item.review_item_id,
            &observation_id,
        )
        .await;
    }
}

#[tokio::test]
async fn review_item_promotion_rejects_missing_evidence_with_bad_request() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let app = build_review_api_app(&database_url).await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let observation_store = ObservationStore::new(pool.clone());
    let review_store = ReviewInboxStore::new(pool.clone());

    let suffix = unique_suffix();

    let test_cases = [
        (
            ReviewItemKind::PotentialDecision,
            "decisions",
            "decision",
            "invalid_decision_query",
            "decision evidence is required",
        ),
        (
            ReviewItemKind::PotentialObligation,
            "obligations",
            "obligation",
            "invalid_obligation_query",
            "obligation evidence is required",
        ),
        (
            ReviewItemKind::PotentialRelationship,
            "relationships",
            "relationship",
            "invalid_relationship_query",
            "relationship evidence is required",
        ),
    ];

    for (index, (item_kind, target_domain, target_entity_kind, expected_error, expected_message)) in
        test_cases.iter().enumerate()
    {
        let observation_id = seed_manual_note(&observation_store, suffix + index as u128).await;
        let item = review_store
            .create_with_evidence(
                &NewReviewItem::new(
                    *item_kind,
                    format!("{item_kind:?} promotion requires evidence {suffix}"),
                    "No evidence should remain at promotion time.".to_owned(),
                    0.91,
                ),
                &[NewReviewItemEvidence::new(observation_id.clone())],
            )
            .await
            .expect("create orphanable review item");

        sqlx::query("DELETE FROM review_item_evidence WHERE review_item_id = $1")
            .bind(&item.review_item_id)
            .execute(&pool)
            .await
            .expect("delete review evidence");

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/api/v1/review/items/{}/promote",
                        path_segment(&item.review_item_id)
                    ))
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-makosh-secret", REVIEW_API_TOKEN)
                    .body(Body::from(
                        json!({
                            "target_domain": target_domain,
                            "target_entity_kind": target_entity_kind,
                            "target_entity_id": format!(
                                "{target_entity_kind}:review-orphan-{suffix}-{}",
                                index
                            ),
                        })
                        .to_string(),
                    ))
                    .expect("bad promote request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_response(response).await;
        assert_eq!(body["error"], json!(expected_error));
        assert_eq!(body["message"], json!(expected_message));
    }
}

#[tokio::test]
async fn review_item_creation_rejects_unknown_observation_with_bad_request() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let app = build_review_api_app(&database_url).await;
    let suffix = unique_suffix();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/review/items")
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-makosh-secret", REVIEW_API_TOKEN)
                .body(Body::from(
                    json!({
                        "item_kind": "potential_task",
                        "title": format!("Unknown evidence task {suffix}"),
                        "summary": "Review inbox must only accept canonical observations.",
                        "confidence": 0.77,
                        "evidence": [
                            {
                                "observation_id": format!("observation:v1:missing:{suffix}")
                            }
                        ]
                    })
                    .to_string(),
                ))
                .expect("bad create review request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_response(response).await;
    assert_eq!(body["error"], json!("invalid_review_query"));
    assert_eq!(
        body["message"],
        json!("review evidence observation was not found")
    );
}

#[tokio::test]
async fn review_item_api_lifecycle_captures_observation_trail_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let observation_store = ObservationStore::new(pool.clone());
    let review_store = ReviewInboxStore::new(pool.clone());
    let app = build_review_api_app(&database_url).await;
    let suffix = unique_suffix();
    let observation_id = seed_manual_note(&observation_store, suffix).await;

    let promotable = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialDecision,
                format!("Review API decision {suffix}"),
                "API lifecycle must create observation-backed review transitions.",
                0.93,
            ),
            &[NewReviewItemEvidence::new(observation_id.clone())],
        )
        .await
        .expect("create promotable review item");

    let take_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/api/v1/review/items/{}/take",
                    path_segment(&promotable.review_item_id)
                ))
                .header("x-makosh-secret", REVIEW_API_TOKEN)
                .body(Body::empty())
                .expect("take request"),
        )
        .await
        .expect("take response");
    assert_eq!(take_response.status(), StatusCode::OK);

    let approve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/api/v1/review/items/{}/approve",
                    path_segment(&promotable.review_item_id)
                ))
                .header("x-makosh-secret", REVIEW_API_TOKEN)
                .body(Body::empty())
                .expect("approve request"),
        )
        .await
        .expect("approve response");
    assert_eq!(approve_response.status(), StatusCode::OK);

    let promote_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/api/v1/review/items/{}/promote",
                    path_segment(&promotable.review_item_id)
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-makosh-secret", REVIEW_API_TOKEN)
                .body(Body::from(
                    json!({
                        "target_domain": "decisions",
                        "target_entity_kind": "decision",
                        "target_entity_id": format!("decision:v1:review-api:{suffix}")
                    })
                    .to_string(),
                ))
                .expect("promote request"),
        )
        .await
        .expect("promote response");
    assert_eq!(promote_response.status(), StatusCode::OK);

    let disposable = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::KnowledgeCandidate,
                format!("Review API dismiss {suffix}"),
                "Dismiss and archive should also leave observation trail.",
                0.51,
            ),
            &[NewReviewItemEvidence::new(observation_id)],
        )
        .await
        .expect("create disposable review item");

    let dismiss_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/api/v1/review/items/{}/dismiss",
                    path_segment(&disposable.review_item_id)
                ))
                .header("x-makosh-secret", REVIEW_API_TOKEN)
                .body(Body::empty())
                .expect("dismiss request"),
        )
        .await
        .expect("dismiss response");
    assert_eq!(dismiss_response.status(), StatusCode::OK);

    let archive_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/api/v1/review/items/{}/archive",
                    path_segment(&disposable.review_item_id)
                ))
                .header("x-makosh-secret", REVIEW_API_TOKEN)
                .body(Body::empty())
                .expect("archive request"),
        )
        .await
        .expect("archive response");
    assert_eq!(archive_response.status(), StatusCode::OK);

    let transition_rows = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'review'
           AND entity_kind = 'review_item'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at ASC",
    )
    .bind(&promotable.review_item_id)
    .fetch_all(&pool)
    .await
    .expect("promotable observation links");
    assert_eq!(transition_rows.len(), 3);
    let promotable_statuses: Vec<String> = transition_rows
        .iter()
        .map(|row| {
            row.try_get::<serde_json::Value, _>("metadata")
                .expect("metadata")["status"]
                .as_str()
                .expect("status")
                .to_owned()
        })
        .collect();
    assert_eq!(
        promotable_statuses,
        vec!["in_review", "approved", "promoted"]
    );

    let promote_observation_id: String = transition_rows[2]
        .try_get("observation_id")
        .expect("promote observation id");
    let promote_payload: serde_json::Value =
        sqlx::query_scalar("SELECT payload FROM observations WHERE observation_id = $1")
            .bind(&promote_observation_id)
            .fetch_one(&pool)
            .await
            .expect("promote observation payload");
    assert_eq!(promote_payload["operation"], "review_item_promote");

    let disposable_rows = sqlx::query(
        "SELECT metadata
         FROM observation_links
         WHERE domain = 'review'
           AND entity_kind = 'review_item'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at ASC",
    )
    .bind(&disposable.review_item_id)
    .fetch_all(&pool)
    .await
    .expect("disposable observation links");
    assert_eq!(disposable_rows.len(), 2);
    let disposable_statuses: Vec<String> = disposable_rows
        .iter()
        .map(|row| {
            row.try_get::<serde_json::Value, _>("metadata")
                .expect("metadata")["status"]
                .as_str()
                .expect("status")
                .to_owned()
        })
        .collect();
    assert_eq!(disposable_statuses, vec!["dismissed", "archived"]);
}

#[tokio::test]
async fn task_candidate_review_mirror_promotes_obligation_candidate_against_postgres() {
    let Some((pool, observation_store, review_store)) =
        live_review_context("task candidate mirrored review").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observation = observation_store
        .capture(
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc.with_ymd_and_hms(2026, 6, 18, 13, 15, 0).unwrap(),
                json!({
                    "subject": format!("Mirrored candidate {suffix}"),
                    "body": format!("I will send the mirrored agreement {suffix} by Friday 5pm.")
                }),
                format!("manual://review-mirror/{suffix}"),
            )
            .confidence(0.93),
        )
        .await
        .expect("manual message observation");

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
            due_text,
            confidence,
            review_state,
            evidence_excerpt
        )
        VALUES (
            $1,
            'observation',
            $2,
            $2,
            'obligation_task',
            $3,
            $4,
            'Friday 5pm',
            0.87,
            'suggested',
            $5
        )
        "#,
    )
    .bind(format!("task_candidate:v1:review_mirror:{suffix}"))
    .bind(&observation.observation_id)
    .bind(json!({
        "engine": "obligation",
        "obligation_candidate": {
            "statement": format!("send the mirrored agreement {suffix}"),
            "quote": format!("I will send the mirrored agreement {suffix} by Friday 5pm."),
            "due_text": "Friday 5pm",
            "confidence": 0.95,
            "kind": "promise"
        }
    }))
    .bind(format!("send the mirrored agreement {suffix}"))
    .bind(format!(
        "I will send the mirrored agreement {suffix} by Friday 5pm."
    ))
    .execute(&pool)
    .await
    .expect("insert task candidate");

    let mirrored = sync_task_candidates_to_review_for_observations(
        &pool,
        std::slice::from_ref(&observation.observation_id),
    )
    .await
    .expect("mirror task candidates to review");
    assert_eq!(mirrored, 1);

    let review_item = review_store
        .list_open(20)
        .await
        .expect("list review items")
        .into_iter()
        .find(|item| item.metadata["mirrored_from"] == json!("task_candidates"))
        .expect("mirrored review item");
    assert_eq!(review_item.item_kind, ReviewItemKind::PotentialTask);
    assert_eq!(
        review_item.metadata["candidate_kind"],
        json!("obligation_task")
    );
    assert_eq!(review_item.metadata["due_text"], json!("Friday 5pm"));

    let promoted =
        makosh_hub_backend::workflows::review_promotion::ReviewPromotionService::new(pool.clone())
            .promote(
                &review_item.review_item_id,
                ReviewPromotionTarget::new("tasks", "task", format!("task:v1:mirror:{suffix}")),
            )
            .await
            .expect("promote mirrored review item");
    assert_eq!(promoted.status, ReviewItemStatus::Promoted);

    let task_id = promoted.target_entity_id.clone().expect("promoted task id");
    let obligation_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM obligation_task_links
        WHERE task_id = $1
        "#,
    )
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("obligation task link count");
    assert_eq!(obligation_count, 1);
}

#[tokio::test]
async fn decision_review_mirror_promotes_existing_decision_against_postgres() {
    let Some((pool, observation_store, review_store)) =
        live_review_context("decision mirrored review").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observation = observation_store
        .capture(
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc.with_ymd_and_hms(2026, 6, 18, 13, 45, 0).unwrap(),
                json!({
                    "subject": format!("Decision mirror {suffix}"),
                    "body": format!("Decision: Buy mirrored NAS {suffix} because local evidence matters.")
                }),
                format!("manual://decision-mirror/{suffix}"),
            )
            .confidence(0.94),
        )
        .await
        .expect("manual decision observation");

    let decision = DecisionStore::new(pool.clone())
        .upsert_with_evidence(
            &makosh_hub_backend::domains::decisions::models::decision::NewDecision::new(
                format!("Buy mirrored NAS {suffix}"),
                "local evidence matters",
                0.83,
                makosh_hub_backend::domains::decisions::models::states::DecisionReviewState::Suggested,
            ),
            &[
                makosh_hub_backend::domains::decisions::models::evidence::NewDecisionEvidence::observation(
                    observation.observation_id.clone(),
                )
                .quote(format!(
                    "Decision: Buy mirrored NAS {suffix} because local evidence matters."
                )),
            ],
            &[],
        )
        .await
        .expect("seed suggested decision");

    let mirrored = sync_decisions_to_review_for_observations(
        &pool,
        std::slice::from_ref(&observation.observation_id),
    )
    .await
    .expect("mirror decisions to review");
    assert_eq!(mirrored, 1);

    let review_item = review_store
        .list_open(20)
        .await
        .expect("list review items")
        .into_iter()
        .find(|item| item.metadata["mirrored_from"] == json!("decisions"))
        .expect("mirrored decision review item");

    let promoted =
        makosh_hub_backend::workflows::review_promotion::ReviewPromotionService::new(pool.clone())
            .promote(
                &review_item.review_item_id,
                ReviewPromotionTarget::new(
                    "decisions",
                    "decision",
                    format!("decision:v1:mirror:{suffix}"),
                ),
            )
            .await
            .expect("promote mirrored decision review item");
    assert_eq!(promoted.status, ReviewItemStatus::Promoted);

    let decision_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM decisions WHERE decision_id = $1")
            .bind(&decision.decision_id)
            .fetch_one(&pool)
            .await
            .expect("decision count");
    assert_eq!(decision_count, 1);
    let review_state: String =
        sqlx::query_scalar("SELECT review_state FROM decisions WHERE decision_id = $1")
            .bind(&decision.decision_id)
            .fetch_one(&pool)
            .await
            .expect("decision review state");
    assert_eq!(review_state, "user_confirmed");

    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'decisions'
           AND entity_kind = 'decision'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&decision.decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision promotion observation link");
    let observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: serde_json::Value = link_row.try_get("metadata").expect("metadata");
    assert_eq!(
        metadata["review_item_id"],
        json!(review_item.review_item_id)
    );

    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("decision promotion observation origin");
    assert_eq!(origin_kind, "manual");
}

#[tokio::test]
async fn obligation_review_mirror_promotes_existing_obligation_against_postgres() {
    let Some((pool, observation_store, review_store)) =
        live_review_context("obligation mirrored review").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observation = observation_store
        .capture(
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc.with_ymd_and_hms(2026, 6, 18, 14, 10, 0).unwrap(),
                json!({
                    "subject": format!("Obligation mirror {suffix}"),
                    "body": format!("Obligation: Send the NAS follow-up package {suffix} by Friday.")
                }),
                format!("manual://obligation-mirror/{suffix}"),
            )
            .confidence(0.92),
        )
        .await
        .expect("manual obligation observation");

    let obligation = ObligationStore::new(pool.clone())
        .upsert_with_evidence(
            &makosh_hub_backend::domains::obligations::models::obligation::NewObligation::new(
                makosh_hub_backend::domains::obligations::models::entity_kind::ObligationEntityKind::Knowledge,
                format!("knowledge:v1:mirror:{suffix}"),
                format!("Send NAS follow-up package {suffix}"),
                0.82,
                makosh_hub_backend::domains::obligations::models::states::ObligationReviewState::Suggested,
            ),
            &[
                makosh_hub_backend::domains::obligations::models::evidence::NewObligationEvidence::observation(
                    observation.observation_id.clone(),
                )
                .quote(format!(
                    "Obligation: Send the NAS follow-up package {suffix} by Friday."
                )),
            ],
        )
        .await
        .expect("seed suggested obligation");

    let mirrored = sync_obligations_to_review_for_observations(
        &pool,
        std::slice::from_ref(&observation.observation_id),
    )
    .await
    .expect("mirror obligations to review");
    assert_eq!(mirrored, 1);

    let review_item = review_store
        .list_open(20)
        .await
        .expect("list review items")
        .into_iter()
        .find(|item| item.metadata["mirrored_from"] == json!("obligations"))
        .expect("mirrored obligation review item");

    let promoted =
        makosh_hub_backend::workflows::review_promotion::ReviewPromotionService::new(pool.clone())
            .promote(
                &review_item.review_item_id,
                ReviewPromotionTarget::new(
                    "obligations",
                    "obligation",
                    format!("obligation:v1:mirror:{suffix}"),
                ),
            )
            .await
            .expect("promote mirrored obligation review item");
    assert_eq!(promoted.status, ReviewItemStatus::Promoted);

    let obligation_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM obligations WHERE obligation_id = $1")
            .bind(&obligation.obligation_id)
            .fetch_one(&pool)
            .await
            .expect("obligation count");
    assert_eq!(obligation_count, 1);
    let review_state: String =
        sqlx::query_scalar("SELECT review_state FROM obligations WHERE obligation_id = $1")
            .bind(&obligation.obligation_id)
            .fetch_one(&pool)
            .await
            .expect("obligation review state");
    assert_eq!(review_state, "user_confirmed");

    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'obligations'
           AND entity_kind = 'obligation'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&obligation.obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation promotion observation link");
    let observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: serde_json::Value = link_row.try_get("metadata").expect("metadata");
    assert_eq!(
        metadata["review_item_id"],
        json!(review_item.review_item_id)
    );

    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("obligation promotion observation origin");
    assert_eq!(origin_kind, "manual");
}

#[tokio::test]
async fn relationship_review_mirror_promotes_existing_relationship_against_postgres() {
    let Some((pool, observation_store, review_store)) =
        live_review_context("relationship mirrored review").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observation = observation_store
        .capture(
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc.with_ymd_and_hms(2026, 6, 18, 14, 25, 0).unwrap(),
                json!({
                    "subject": format!("Relationship mirror {suffix}"),
                    "body": format!("Relationship: Ivan collaborates with NAS project {suffix}.")
                }),
                format!("manual://relationship-mirror/{suffix}"),
            )
            .confidence(0.9),
        )
        .await
        .expect("manual relationship observation");

    let relationship = RelationshipStore::new(pool.clone())
        .upsert_with_evidence(
            &NewRelationship {
                source_entity_kind: RelationshipEntityKind::Persona,
                source_entity_id: format!("persona:source:{suffix}"),
                target_entity_kind: RelationshipEntityKind::Project,
                target_entity_id: format!("project:v1:target:{suffix}"),
                relationship_type: "collaborates_with".to_owned(),
                trust_score: 0.61,
                strength_score: 0.59,
                confidence: 0.88,
                review_state: RelationshipReviewState::Suggested,
                valid_from: None,
                valid_to: None,
                metadata: json!({"source": "review_mirror_test"}),
            },
            &[
                NewRelationshipEvidence::observation(observation.observation_id.clone()).excerpt(
                    format!("Relationship: Ivan collaborates with NAS project {suffix}."),
                ),
            ],
        )
        .await
        .expect("seed suggested relationship");

    let mirrored = sync_relationships_to_review_for_observations(
        &pool,
        std::slice::from_ref(&observation.observation_id),
    )
    .await
    .expect("mirror relationships to review");
    assert_eq!(mirrored, 1);

    let review_item = review_store
        .list_open(20)
        .await
        .expect("list review items")
        .into_iter()
        .find(|item| item.metadata["mirrored_from"] == json!("relationships"))
        .expect("mirrored relationship review item");

    let promoted =
        makosh_hub_backend::workflows::review_promotion::ReviewPromotionService::new(pool.clone())
            .promote(
                &review_item.review_item_id,
                ReviewPromotionTarget::new(
                    "relationships",
                    "relationship",
                    format!("relationship:v1:mirror:{suffix}"),
                ),
            )
            .await
            .expect("promote mirrored relationship review item");
    assert_eq!(promoted.status, ReviewItemStatus::Promoted);

    let relationship_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM relationships WHERE relationship_id = $1")
            .bind(&relationship.relationship_id)
            .fetch_one(&pool)
            .await
            .expect("relationship count");
    assert_eq!(relationship_count, 1);
    let review_state: String =
        sqlx::query_scalar("SELECT review_state FROM relationships WHERE relationship_id = $1")
            .bind(&relationship.relationship_id)
            .fetch_one(&pool)
            .await
            .expect("relationship review state");
    assert_eq!(review_state, "user_confirmed");

    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'relationships'
           AND entity_kind = 'relationship'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&relationship.relationship_id)
    .fetch_one(&pool)
    .await
    .expect("relationship promotion observation link");
    let observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: serde_json::Value = link_row.try_get("metadata").expect("metadata");
    assert_eq!(
        metadata["review_item_id"],
        json!(review_item.review_item_id)
    );

    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("relationship promotion observation origin");
    assert_eq!(origin_kind, "manual");
}

#[tokio::test]
async fn identity_candidate_review_mirror_promotes_existing_candidate_against_postgres() {
    let Some((pool, _observation_store, review_store)) =
        live_review_context("identity candidate mirrored review").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let person_store = PersonaProjectionStore::new(pool.clone());
    let identity_store = PersonaIdentityReviewStore::new(pool.clone());
    let display_name = format!("Identity Mirror {suffix}");

    let left = person_store
        .upsert_email_persona(&format!("identity-mirror-left-{suffix}@example.com"))
        .await
        .expect("left persona");
    let right = person_store
        .upsert_email_persona(&format!("identity-mirror-right-{suffix}@example.com"))
        .await
        .expect("right persona");
    sqlx::query("UPDATE personas SET display_name = $1 WHERE persona_id = $2 OR persona_id = $3")
        .bind(&display_name)
        .bind(&left.persona_id)
        .bind(&right.persona_id)
        .execute(&pool)
        .await
        .expect("seed display name");

    let refreshed = identity_store
        .refresh_candidates(100)
        .await
        .expect("refresh identity candidates");
    assert!(refreshed >= 1);
    let _ = project_identity_review_events(&pool, 0).await;

    let identity_candidate_id = if left.persona_id <= right.persona_id {
        format!(
            "identity_candidate:v1:merge_personas:{}:{}",
            left.persona_id, right.persona_id
        )
    } else {
        format!(
            "identity_candidate:v1:merge_personas:{}:{}",
            right.persona_id, left.persona_id
        )
    };
    let (left_persona_id, right_persona_id) = if left.persona_id <= right.persona_id {
        (left.persona_id.clone(), right.persona_id.clone())
    } else {
        (right.persona_id.clone(), left.persona_id.clone())
    };

    let review_item = review_store
        .list_open(50)
        .await
        .expect("list review items")
        .into_iter()
        .find(|item| item.metadata["identity_candidate_id"] == json!(identity_candidate_id))
        .expect("mirrored identity review item");
    assert_eq!(review_item.item_kind, ReviewItemKind::IdentityCandidate);
    assert_eq!(
        review_item.metadata["left_persona_id"],
        json!(left_persona_id)
    );
    assert_eq!(
        review_item.metadata["right_persona_id"],
        json!(right_persona_id)
    );
    assert!(review_item.metadata.get("left_person_id").is_none());
    assert!(review_item.metadata.get("right_person_id").is_none());

    let promoted =
        makosh_hub_backend::workflows::review_promotion::ReviewPromotionService::new(pool.clone())
            .promote(
                &review_item.review_item_id,
                ReviewPromotionTarget::new(
                    "personas",
                    "identity_candidate",
                    identity_candidate_id.clone(),
                ),
            )
            .await
            .expect("promote mirrored identity candidate");
    assert_eq!(promoted.status, ReviewItemStatus::Promoted);

    let review_state: String = sqlx::query_scalar(
        "SELECT review_state FROM persona_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&pool)
    .await
    .expect("identity candidate review state");
    assert_eq!(review_state, "user_confirmed");

    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'personas'
           AND entity_kind = 'identity_candidate'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&pool)
    .await
    .expect("identity promotion observation link");
    let observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: serde_json::Value = link_row.try_get("metadata").expect("metadata");
    assert_eq!(
        metadata["review_item_id"],
        json!(review_item.review_item_id)
    );

    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("identity promotion observation origin");
    assert_eq!(origin_kind, "manual");
}

#[tokio::test]
async fn identity_candidate_review_mirror_reuses_review_item_and_attaches_new_evidence_against_postgres()
 {
    let Some((pool, _observation_store, review_store)) =
        live_review_context("identity candidate mirrored review idempotency").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let person_store = PersonaProjectionStore::new(pool.clone());
    let identity_store = PersonaIdentityReviewStore::new(pool.clone());
    let display_name = format!("Identity Mirror Reuse {suffix}");

    let left = person_store
        .upsert_email_persona(&format!("identity-reuse-left-{suffix}@example.com"))
        .await
        .expect("left persona");
    let right = person_store
        .upsert_email_persona(&format!("identity-reuse-right-{suffix}@example.com"))
        .await
        .expect("right persona");
    sqlx::query("UPDATE personas SET display_name = $1 WHERE persona_id = $2 OR persona_id = $3")
        .bind(&display_name)
        .bind(&left.persona_id)
        .bind(&right.persona_id)
        .execute(&pool)
        .await
        .expect("seed display name");

    let refreshed = identity_store
        .refresh_candidates(100)
        .await
        .expect("refresh identity candidates");
    assert!(refreshed >= 1);
    let mut event_position = 0;
    event_position = project_identity_review_events(&pool, event_position).await;

    let identity_candidate_id = if left.persona_id <= right.persona_id {
        format!(
            "identity_candidate:v1:merge_personas:{}:{}",
            left.persona_id, right.persona_id
        )
    } else {
        format!(
            "identity_candidate:v1:merge_personas:{}:{}",
            right.persona_id, left.persona_id
        )
    };

    let first_review_item = review_store
        .list_open(50)
        .await
        .expect("list first review items")
        .into_iter()
        .find(|item| item.metadata["identity_candidate_id"] == json!(identity_candidate_id))
        .expect("first mirrored identity review item");

    let refreshed_again = identity_store
        .refresh_candidates(100)
        .await
        .expect("refresh identity candidates again");
    assert!(refreshed_again >= 1);
    let _ = project_identity_review_events(&pool, event_position).await;

    let review_rows: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT review_item_id
        FROM review_items
        WHERE metadata->>'identity_candidate_id' = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(&identity_candidate_id)
    .fetch_all(&pool)
    .await
    .expect("identity candidate review items");
    assert_eq!(review_rows.len(), 1);
    assert_eq!(review_rows[0].0, first_review_item.review_item_id);

    let evidence_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM review_item_evidence
        WHERE review_item_id = $1
        "#,
    )
    .bind(&first_review_item.review_item_id)
    .fetch_one(&pool)
    .await
    .expect("identity candidate review evidence count");
    assert_eq!(evidence_count, 2);
}

#[tokio::test]
async fn project_link_candidate_review_promotes_existing_candidate_against_postgres() {
    let Some((pool, _observation_store, review_store)) =
        live_review_context("project link candidate mirrored review").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let project_id = format!("project:v1:review-link:{suffix}");
    ProjectStore::new(pool.clone())
        .upsert_project(
            &makosh_hub_backend::domains::projects::core::models::NewProject::active(
                &project_id,
                format!("Review Link Project {suffix}"),
                "Product Development",
                "Review promotion for project link candidate",
                "Alex Morgan",
                vec![format!("ReviewLink{suffix}")],
            ),
        )
        .await
        .expect("upsert project");
    let imported = DocumentImportStore::new(pool.clone())
        .import_document(&NewDocumentImport::markdown(
            format!("review_link_doc_{suffix}"),
            format!("ReviewLink{suffix} architecture"),
            "# Architecture\n\nReviewLink evidence",
        ))
        .await
        .expect("import document");

    let item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::ProjectLinkCandidate,
                format!("ReviewLink{suffix} architecture"),
                format!("ReviewLink evidence {suffix}"),
                0.72,
            )
            .metadata(json!({
                "mirrored_from": "project_link_candidates",
                "project_id": project_id,
                "target_kind": "document",
                "target_id": imported.document_id,
            })),
            &[NewReviewItemEvidence::new(imported.observation_id.clone())],
        )
        .await
        .expect("create project link review item");

    let promoted =
        makosh_hub_backend::workflows::review_promotion::ReviewPromotionService::new(pool.clone())
            .promote(
                &item.review_item_id,
                ReviewPromotionTarget::new(
                    "projects",
                    "project_link_candidate",
                    format!("{}:document:{}", project_id, imported.document_id),
                ),
            )
            .await
            .expect("promote project link candidate");
    assert_eq!(promoted.status, ReviewItemStatus::Promoted);

    let review_state: String = sqlx::query_scalar(
        r#"
        SELECT review_state
        FROM project_link_reviews
        WHERE project_id = $1
          AND target_kind = 'document'
          AND target_id = $2
        "#,
    )
    .bind(&project_id)
    .bind(&imported.document_id)
    .fetch_one(&pool)
    .await
    .expect("project link review state");
    assert_eq!(review_state, "user_confirmed");

    let event_id: String = sqlx::query_scalar(
        r#"
        SELECT event_id
        FROM project_link_reviews
        WHERE project_id = $1
          AND target_kind = 'document'
          AND target_id = $2
        "#,
    )
    .bind(&project_id)
    .bind(&imported.document_id)
    .fetch_one(&pool)
    .await
    .expect("project link event id");
    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'projects'
           AND entity_kind = 'project_link_review'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .expect("project link promotion observation link");
    let observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: serde_json::Value = link_row.try_get("metadata").expect("metadata");
    assert_eq!(metadata["review_item_id"], json!(item.review_item_id));

    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("project link promotion observation origin");
    assert_eq!(origin_kind, "manual");
}

async fn assert_materialized_target(
    pool: &PgPool,
    domain: &str,
    target_id: &str,
    review_item_id: &str,
    observation_id: &str,
) {
    match domain {
        "personas" => {
            let count =
                sqlx::query_scalar::<_, i64>("SELECT count(*) FROM personas WHERE persona_id = $1")
                    .bind(target_id)
                    .fetch_one(pool)
                    .await
                    .expect("person count");
            assert_eq!(count, 1);
            let support_link_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM observation_links
                WHERE observation_id = $1
                  AND domain = 'personas'
                  AND entity_kind = 'persona'
                  AND entity_id = $2
                  AND relationship_kind = 'supports'
                "#,
            )
            .bind(observation_id)
            .bind(target_id)
            .fetch_one(pool)
            .await
            .expect("person support link count");
            assert_eq!(support_link_count, 1);

            let review_transition_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM observation_links
                WHERE domain = 'personas'
                  AND entity_kind = 'persona'
                  AND entity_id = $1
                  AND relationship_kind = 'review_transition'
                  AND metadata->>'review_item_id' = $2
                "#,
            )
            .bind(target_id)
            .bind(review_item_id)
            .fetch_one(pool)
            .await
            .expect("person review transition link count");
            assert_eq!(review_transition_count, 1);
        }
        "organizations" => {
            let count = sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM organizations WHERE organization_id = $1",
            )
            .bind(target_id)
            .fetch_one(pool)
            .await
            .expect("organization count");
            assert_eq!(count, 1);
            let support_link_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM observation_links
                WHERE observation_id = $1
                  AND domain = 'organizations'
                  AND entity_kind = 'organization'
                  AND entity_id = $2
                  AND relationship_kind = 'supports'
                "#,
            )
            .bind(observation_id)
            .bind(target_id)
            .fetch_one(pool)
            .await
            .expect("organization support link count");
            assert_eq!(support_link_count, 1);

            let review_transition_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM observation_links
                WHERE domain = 'organizations'
                  AND entity_kind = 'organization'
                  AND entity_id = $1
                  AND relationship_kind = 'review_transition'
                  AND metadata->>'review_item_id' = $2
                "#,
            )
            .bind(target_id)
            .bind(review_item_id)
            .fetch_one(pool)
            .await
            .expect("organization review transition link count");
            assert_eq!(review_transition_count, 1);
        }
        "tasks" => {
            let task = TaskStore::new(pool.clone())
                .get(target_id)
                .await
                .expect("load task")
                .expect("task exists");
            assert_eq!(task.provenance_kind, "review_item");
            assert_eq!(task.provenance_id, review_item_id);
            assert_eq!(task.source_kind, "observation");
            assert_eq!(task.source_type, "observation");
            assert_eq!(task.source_id, observation_id);

            let task_create_link_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM observation_links
                WHERE observation_id = $1
                  AND domain = 'tasks'
                  AND entity_kind = 'task'
                  AND entity_id = $2
                  AND relationship_kind = 'task_create'
                "#,
            )
            .bind(observation_id)
            .bind(target_id)
            .fetch_one(pool)
            .await
            .expect("task create link count");
            assert_eq!(task_create_link_count, 1);

            let support_link_count: i64 = sqlx::query_scalar(
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
            .bind(observation_id)
            .bind(target_id)
            .fetch_one(pool)
            .await
            .expect("task support link count");
            assert_eq!(support_link_count, 1);

            let review_transition_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM observation_links
                WHERE domain = 'tasks'
                  AND entity_kind = 'task'
                  AND entity_id = $1
                  AND relationship_kind = 'review_transition'
                  AND metadata->>'review_item_id' = $2
                "#,
            )
            .bind(target_id)
            .bind(review_item_id)
            .fetch_one(pool)
            .await
            .expect("task review transition link count");
            assert_eq!(review_transition_count, 1);
        }
        "obligations" => {
            let obligations = ObligationStore::new(pool.clone())
                .list_by_review_state(
                    makosh_hub_backend::domains::obligations::models::states::ObligationReviewState::UserConfirmed,
                    100,
                )
                .await
                .expect("list obligations");
            assert!(
                obligations
                    .iter()
                    .any(|item| item.obligation_id == target_id)
            );
            assert_observation_evidence(
                pool,
                "obligation_evidence",
                "obligation_id",
                target_id,
                observation_id,
            )
            .await;

            let review_transition_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM observation_links
                WHERE domain = 'obligations'
                  AND entity_kind = 'obligation'
                  AND entity_id = $1
                  AND relationship_kind = 'review_transition'
                  AND metadata->>'review_item_id' = $2
                "#,
            )
            .bind(target_id)
            .bind(review_item_id)
            .fetch_one(pool)
            .await
            .expect("obligation review transition link count");
            assert_eq!(review_transition_count, 1);
        }
        "decisions" => {
            let decisions = DecisionStore::new(pool.clone())
                .list_by_review_state(
                    makosh_hub_backend::domains::decisions::models::states::DecisionReviewState::UserConfirmed,
                    100,
                )
                .await
                .expect("list decisions");
            assert!(decisions.iter().any(|item| item.decision_id == target_id));
            assert_observation_evidence(
                pool,
                "decision_evidence",
                "decision_id",
                target_id,
                observation_id,
            )
            .await;

            let review_transition_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM observation_links
                WHERE domain = 'decisions'
                  AND entity_kind = 'decision'
                  AND entity_id = $1
                  AND relationship_kind = 'review_transition'
                  AND metadata->>'review_item_id' = $2
                "#,
            )
            .bind(target_id)
            .bind(review_item_id)
            .fetch_one(pool)
            .await
            .expect("decision review transition link count");
            assert_eq!(review_transition_count, 1);
        }
        "relationships" => {
            let relationships = RelationshipStore::new(pool.clone())
                .list_by_review_state(RelationshipReviewState::UserConfirmed, 100)
                .await
                .expect("list relationships");
            assert!(
                relationships
                    .iter()
                    .any(|item| item.relationship_id == target_id)
            );
            assert_observation_evidence(
                pool,
                "relationship_evidence",
                "relationship_id",
                target_id,
                observation_id,
            )
            .await;

            let review_transition_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM observation_links
                WHERE domain = 'relationships'
                  AND entity_kind = 'relationship'
                  AND entity_id = $1
                  AND relationship_kind = 'review_transition'
                  AND metadata->>'review_item_id' = $2
                "#,
            )
            .bind(target_id)
            .bind(review_item_id)
            .fetch_one(pool)
            .await
            .expect("relationship review transition link count");
            assert_eq!(review_transition_count, 1);
        }
        "projects" => {
            let project = ProjectStore::new(pool.clone())
                .project_detail(target_id)
                .await
                .expect("project detail");
            assert!(project.is_some());
            let support_link_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM observation_links
                WHERE observation_id = $1
                  AND domain = 'projects'
                  AND entity_kind = 'project'
                  AND entity_id = $2
                  AND relationship_kind = 'supports'
                "#,
            )
            .bind(observation_id)
            .bind(target_id)
            .fetch_one(pool)
            .await
            .expect("project support link count");
            assert_eq!(support_link_count, 1);

            let review_transition_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM observation_links
                WHERE domain = 'projects'
                  AND entity_kind = 'project'
                  AND entity_id = $1
                  AND relationship_kind = 'review_transition'
                  AND metadata->>'review_item_id' = $2
                "#,
            )
            .bind(target_id)
            .bind(review_item_id)
            .fetch_one(pool)
            .await
            .expect("project review transition link count");
            assert_eq!(review_transition_count, 1);
        }
        "documents" => {
            let document: (String, String) = sqlx::query_as(
                "SELECT document_id, observation_id FROM documents WHERE document_id = $1",
            )
            .bind(target_id)
            .fetch_one(pool)
            .await
            .expect("document row");
            assert_eq!(document.0, target_id);
            let support_link_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM observation_links
                WHERE observation_id = $1
                  AND domain = 'documents'
                  AND entity_kind = 'document'
                  AND entity_id = $2
                  AND relationship_kind = 'supports'
                "#,
            )
            .bind(observation_id)
            .bind(target_id)
            .fetch_one(pool)
            .await
            .expect("document support link count");
            assert_eq!(support_link_count, 1);

            let review_transition_count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)
                FROM observation_links
                WHERE domain = 'documents'
                  AND entity_kind = 'document'
                  AND entity_id = $1
                  AND relationship_kind = 'review_transition'
                  AND metadata->>'review_item_id' = $2
                "#,
            )
            .bind(target_id)
            .bind(review_item_id)
            .fetch_one(pool)
            .await
            .expect("document review transition link count");
            assert_eq!(review_transition_count, 1);
        }
        other => panic!("unexpected target domain {other}"),
    }
}

async fn assert_observation_evidence(
    pool: &PgPool,
    table_name: &str,
    owner_column: &str,
    owner_id: &str,
    observation_id: &str,
) {
    let sql = format!(
        "SELECT count(*) FROM {table_name} WHERE {owner_column} = $1 AND source_kind = 'observation' AND source_id = $2 AND observation_id = $2"
    );
    let count = sqlx::query_scalar::<_, i64>(&sql)
        .bind(owner_id)
        .bind(observation_id)
        .fetch_one(pool)
        .await
        .expect("observation evidence count");
    assert_eq!(count, 1);
}

async fn live_review_context(
    _test_name: &str,
) -> Option<(PgPool, ObservationStore, ReviewInboxStore)> {
    let test_context = TestContext::new().await;
    let pool = test_context.pool().clone();
    Box::leak(Box::new(test_context));
    Some((
        pool.clone(),
        ObservationStore::new(pool.clone()),
        ReviewInboxStore::new(pool),
    ))
}

async fn project_identity_review_events(pool: &PgPool, after_position: i64) -> i64 {
    let events = EventStore::new(pool.clone())
        .list_after_position(after_position, 100)
        .await
        .expect("list persona identity events");
    let mut last_position = after_position;
    for event in events {
        last_position = event.position;
        if event.event.event_type == "persona_identity.candidate.detected" {
            project_persona_identity_review_event(pool.clone(), event)
                .await
                .expect("project persona identity review event");
        }
    }
    last_position
}

async fn seed_manual_note(store: &ObservationStore, suffix: u128) -> String {
    let observation = store
        .capture(
            &NewObservation::new(
                "DOCUMENT",
                ObservationOriginKind::Manual,
                Utc.with_ymd_and_hms(2026, 6, 18, 11, 0, 0).unwrap(),
                json!({
                    "title": format!("Review source note {suffix}"),
                    "body": "Potential task, decision and relationship candidates are evidence."
                }),
                format!("manual://note/{suffix}"),
            )
            .confidence(0.88),
        )
        .await
        .expect("seed manual note observation");

    observation.observation_id
}

async fn build_review_api_app(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            REVIEW_API_TOKEN,
            database_url,
        ),
        database,
    )
}

const REVIEW_API_TOKEN: &str = "review-api-test-token";

fn path_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

async fn json_response(response: axum::response::Response) -> serde_json::Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json response")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX_EPOCH")
        .as_nanos()
}
