use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode};
use chrono::{TimeZone, Utc};
use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::review::{
    models::{NewReviewItem, NewReviewItemEvidence, ReviewItemKind, ReviewItemStatus},
    store::ReviewInboxStore,
};
use makosh_hub_backend::platform::storage::database::Database;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn review_attention_cards_api_returns_grouped_explainable_cards_against_postgres() {
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
    let first_observation_id = seed_manual_note(&observation_store, suffix).await;
    let second_observation_id = seed_manual_note(&observation_store, suffix + 1).await;
    let dismissed_observation_id = seed_manual_note(&observation_store, suffix + 2).await;
    let group_key = format!("contract-deadline:{suffix}");

    let first_item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialTask,
                format!("Contract review reminder {suffix}"),
                "A source-backed message suggests a deadline-backed task.",
                0.78,
            )
            .metadata(json!({
                "attention_group_key": group_key,
            })),
            &[NewReviewItemEvidence::new(first_observation_id)],
        )
        .await
        .expect("create first review item");
    let second_item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialTask,
                format!("Contract review follow-up {suffix}"),
                "A second source repeats the same task signal.",
                0.91,
            )
            .metadata(json!({
                "attention_group_key": group_key,
            })),
            &[NewReviewItemEvidence::new(second_observation_id)],
        )
        .await
        .expect("create second review item");
    let dismissed_item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialTask,
                format!("Dismissed attention card {suffix}"),
                "Dismissed review items should not remain on active attention cards.",
                0.99,
            ),
            &[NewReviewItemEvidence::new(dismissed_observation_id)],
        )
        .await
        .expect("create dismissed review item");
    review_store
        .set_status(&dismissed_item.review_item_id, ReviewItemStatus::Dismissed)
        .await
        .expect("dismiss review item");

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/review/attention-cards?status=active&limit=20")
                .header("x-makosh-secret", REVIEW_API_TOKEN)
                .body(Body::empty())
                .expect("attention cards request"),
        )
        .await
        .expect("attention cards response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_response(response).await;
    let cards = body["cards"].as_array().expect("attention cards array");
    assert_eq!(cards.len(), 1);
    let card = &cards[0];
    assert_eq!(
        card["id"],
        json!(format!("attention:review:contract-deadline:{suffix}"))
    );
    assert_eq!(card["importance"], json!("high"));
    assert_eq!(card["confidence"], json!(0.91));
    assert_eq!(card["evidence_count"], json!(2));
    assert!(
        card["trace_id"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    assert_eq!(card["suggested_actions"].as_array().unwrap().len(), 3);
    assert_eq!(
        card["explainability"]["evidence"].as_array().unwrap().len(),
        2
    );
    assert!(
        card["explainability"]["why_this_matters"]
            .as_str()
            .unwrap()
            .contains("potential task")
    );

    let review_item_ids = card["review_item_ids"]
        .as_array()
        .expect("review item ids")
        .iter()
        .map(|value| value.as_str().expect("review item id").to_owned())
        .collect::<Vec<_>>();
    assert!(review_item_ids.contains(&first_item.review_item_id));
    assert!(review_item_ids.contains(&second_item.review_item_id));
    assert!(!review_item_ids.contains(&dismissed_item.review_item_id));
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
                    "body": "Potential task candidates are evidence."
                }),
                format!("manual://attention-note/{suffix}"),
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

const REVIEW_API_TOKEN: &str = "review-api-test-token";
