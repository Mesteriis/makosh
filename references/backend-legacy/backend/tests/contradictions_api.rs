use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::engines::consistency::{
    models::{
        ContradictionObservation, ContradictionReviewState, ContradictionSeverity,
        ContradictionSourceKind, NewContradictionObservation,
    },
    store::ContradictionObservationStore,
};
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::consistency_review::sync_contradiction_review_item;

const LOCAL_API_TOKEN: &str = "contradictions-api-test-token";

#[tokio::test]
async fn contradictions_list_returns_open_reviewable_observations() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let stored = seed_contradiction_observation(&pool, suffix).await;

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/contradictions?limit=10",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    let item = items
        .iter()
        .find(|item| item["observation_id"] == json!(stored.observation_id))
        .expect("seeded contradiction observation");

    assert_eq!(item["conflict_type"], "direct_contradiction");
    assert_eq!(item["old_claim"], stored.old_claim);
    assert_eq!(item["new_claim"], stored.new_claim);
    assert_eq!(item["review_state"], "suggested");

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT item_kind, status, metadata->>'contradiction_observation_id'
        FROM review_items
        WHERE metadata->>'contradiction_observation_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&stored.observation_id)
    .fetch_one(&pool)
    .await
    .expect("contradiction review item");
    assert_eq!(review_item.0, "contradiction_candidate");
    assert_eq!(review_item.1, "new");
    assert_eq!(review_item.2, stored.observation_id);

    let materialized_link: (String, String, Value, String) = sqlx::query_as(
        r#"
        SELECT
            link.observation_id,
            link.relationship_kind,
            link.metadata,
            kind.code
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'consistency'
          AND link.entity_kind = 'contradiction_observation'
          AND link.entity_id = $1
          AND link.relationship_kind = 'upsert'
        ORDER BY link.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&stored.observation_id)
    .fetch_one(&pool)
    .await
    .expect("contradiction materialized link");
    assert!(materialized_link.0.starts_with("observation:v1:"));
    assert_eq!(materialized_link.1, "upsert");
    assert_eq!(
        materialized_link.2["conflict_type"],
        json!("direct_contradiction")
    );
    assert_eq!(materialized_link.3, "CONTRADICTION_OBSERVATION");
}

#[tokio::test]
async fn put_contradiction_review_updates_review_state_with_observation_trail() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let stored = seed_contradiction_observation(&pool, suffix).await;
    let observation_id = path_segment(&stored.observation_id);

    let response = app
        .oneshot(json_put_request(
            &format!("/api/v1/contradictions/{observation_id}/review"),
            json!({
                "review_state": "user_confirmed",
                "resolution": "confirmed from source review"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["observation_id"], stored.observation_id);
    assert_eq!(body["review_state"], "user_confirmed");
    assert_eq!(body["reviewed_by"], "makosh-frontend");
    assert_eq!(body["resolution"], "confirmed from source review");

    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'consistency'
           AND entity_kind = 'contradiction_observation'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&stored.observation_id)
    .fetch_one(&pool)
    .await
    .expect("contradiction observation link");
    let review_observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: Value = link_row.try_get("metadata").expect("link metadata");
    assert_eq!(metadata["review_state"], "user_confirmed");
    assert_eq!(metadata["resolution"], "confirmed from source review");

    let row: (String, Option<String>, i64) = sqlx::query_as(
        r#"
        SELECT
            review_state,
            resolution,
            (
                SELECT count(*)
                FROM persona_facts
                WHERE value = $2
            ) AS memory_overwrite_count
        FROM contradiction_observations
        WHERE observation_id = $1
        "#,
    )
    .bind(&stored.observation_id)
    .bind(&stored.new_claim)
    .fetch_one(&pool)
    .await
    .expect("stored contradiction review");

    assert_eq!(row.0, "user_confirmed");
    assert_eq!(row.1.as_deref(), Some("confirmed from source review"));
    assert_eq!(row.2, 0);

    let observation_row =
        sqlx::query("SELECT origin_kind, payload FROM observations WHERE observation_id = $1")
            .bind(&review_observation_id)
            .fetch_one(&pool)
            .await
            .expect("review observation");
    let origin_kind: String = observation_row.try_get("origin_kind").expect("origin kind");
    let payload: Value = observation_row.try_get("payload").expect("payload");
    assert_eq!(origin_kind, "manual");
    assert_eq!(
        payload["contradiction_observation_id"],
        json!(stored.observation_id)
    );
    assert_eq!(payload["review_state"], "user_confirmed");

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT item_kind, status, metadata->>'contradiction_observation_id'
        FROM review_items
        WHERE metadata->>'contradiction_observation_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&stored.observation_id)
    .fetch_one(&pool)
    .await
    .expect("updated contradiction review item");
    assert_eq!(review_item.0, "contradiction_candidate");
    assert_eq!(review_item.1, "approved");
    assert_eq!(review_item.2, stored.observation_id);
}

async fn app_and_pool(database_url: &str) -> (axum::Router, PgPool) {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url,
        ),
        database,
    );

    (app, pool)
}

async fn seed_contradiction_observation(pool: &PgPool, suffix: u128) -> ContradictionObservation {
    let observation = NewContradictionObservation {
        old_source_kind: ContradictionSourceKind::Memory,
        old_source_id: format!("memory:contradiction-api:{suffix}"),
        new_source_kind: ContradictionSourceKind::Communication,
        new_source_id: format!("message:contradiction-api:{suffix}"),
        affected_entities: json!([
            {"entity_kind": "persona", "entity_id": format!("person:v1:email:polygraph-{suffix}@example.com")}
        ]),
        conflict_type: "direct_contradiction".to_owned(),
        old_claim: "status=available".to_owned(),
        new_claim: format!("status=unavailable-{suffix}"),
        confidence: 0.86,
        severity: ContradictionSeverity::Medium,
        review_state: ContradictionReviewState::Suggested,
        metadata: json!({"source": "contradictions_api_test"}),
    };

    let stored = ContradictionObservationStore::new(pool.clone())
        .upsert(&observation)
        .await
        .expect("seed contradiction observation");
    sync_contradiction_review_item(pool, &stored)
        .await
        .expect("seed contradiction review item");
    stored
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn json_put_request(uri: &str, value: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

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
