use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode};
use chrono::{TimeZone, Utc};
use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::review::{
    models::{NewReviewItem, NewReviewItemEvidence, ReviewItemKind},
    store::ReviewInboxStore,
};
use makosh_hub_backend::engines::context_packs::{
    models::ContextPackSourceKind, store::ContextPackStore,
};
use makosh_hub_backend::platform::storage::database::Database;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn review_context_pack_api_persists_review_pack_with_sources_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let observation_store = ObservationStore::new(pool.clone());
    let review_store = ReviewInboxStore::new(pool.clone());
    let context_pack_store = ContextPackStore::new(pool);
    let app = build_review_api_app(&database_url).await;
    let suffix = unique_suffix();
    let first_observation_id = seed_manual_note(&observation_store, suffix).await;
    let second_observation_id = seed_manual_note(&observation_store, suffix + 1).await;

    let review_item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialTask,
                format!("Prepare source-backed contract context {suffix}"),
                "A source-backed message suggests a deadline-backed task.",
                0.91,
            )
            .metadata(json!({
                "attention_group_key": format!("contract-context:{suffix}"),
            })),
            &[
                NewReviewItemEvidence::new(first_observation_id.clone()).role("primary"),
                NewReviewItemEvidence::new(second_observation_id.clone()).role("supporting"),
            ],
        )
        .await
        .expect("create review item");

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/api/v1/review/items/{}/context-pack",
                    review_item.review_item_id
                ))
                .header("x-makosh-secret", REVIEW_API_TOKEN)
                .body(Body::empty())
                .expect("review context pack request"),
        )
        .await
        .expect("review context pack response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_response(response).await;
    assert_eq!(body["kind"], json!("review"));
    assert_eq!(body["subject_id"], json!(review_item.review_item_id));
    assert_eq!(
        body["metadata"]["owner"],
        json!("engines.context_packs.review")
    );
    assert_eq!(
        body["content"]["review_item"]["review_item_id"],
        json!(review_item.review_item_id)
    );
    assert!(
        body["content"]["trace"]["trace_id"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    assert_context_section_has_at_least_one_item(&body["content"], "timeline");
    assert_context_section_has_at_least_one_item(&body["content"], "messages");
    assert_context_section_is_empty(&body["content"], "documents");
    assert_context_section_is_empty(&body["content"], "people");
    assert_context_section_is_empty(&body["content"], "organizations");
    assert_context_section_is_empty(&body["content"], "previous_reviews");
    assert_context_section_is_empty(&body["content"], "open_tasks");
    assert_context_section_is_empty(&body["content"], "open_obligations");

    let evidence_ids = body["content"]["evidence"]
        .as_array()
        .expect("context pack evidence array")
        .iter()
        .map(|value| value["observation_id"].as_str().expect("observation id"))
        .collect::<Vec<_>>();
    let mut evidence_ids = evidence_ids;
    evidence_ids.sort_unstable();

    let mut expected_ids = vec![
        first_observation_id.as_str(),
        second_observation_id.as_str(),
    ];
    expected_ids.sort_unstable();
    assert_eq!(evidence_ids, expected_ids);

    let context_pack_id = body["context_pack_id"].as_str().expect("context pack id");
    let sources = context_pack_store
        .list_sources(context_pack_id)
        .await
        .expect("list context pack sources");
    assert_eq!(sources.len(), 3);
    assert!(sources.iter().any(|source| {
        source.source_kind == ContextPackSourceKind::ReviewItem
            && source.source_id == review_item.review_item_id
            && source.role == "subject"
    }));
    assert!(sources.iter().any(|source| {
        source.source_kind == ContextPackSourceKind::Observation
            && source.source_id == first_observation_id
            && source.role == "primary"
    }));
    assert!(sources.iter().any(|source| {
        source.source_kind == ContextPackSourceKind::Observation
            && source.source_id == second_observation_id
            && source.role == "supporting"
    }));
}

async fn seed_manual_note(store: &ObservationStore, suffix: u128) -> String {
    let observation = store
        .capture(
            &NewObservation::new(
                "DOCUMENT",
                ObservationOriginKind::Manual,
                Utc.with_ymd_and_hms(2026, 6, 18, 11, 0, 0).unwrap(),
                json!({
                    "title": format!("Review context source note {suffix}"),
                    "body": "Review context packs preserve canonical evidence."
                }),
                format!("manual://review-context-note/{suffix}"),
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

async fn json_response(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json response")
}

fn assert_context_section_has_at_least_one_item(value: &Value, key: &str) {
    assert!(
        !value[key]
            .as_array()
            .expect("context pack section array")
            .is_empty(),
        "{key} should include context from evidence or payload"
    );
}

fn assert_context_section_is_empty(value: &Value, key: &str) {
    assert_eq!(
        value[key].as_array().expect("context pack section array"),
        &Vec::<Value>::new(),
        "{key} should remain empty for this test fixture"
    );
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX_EPOCH")
        .as_nanos()
}

const REVIEW_API_TOKEN: &str = "review-api-test-token";
